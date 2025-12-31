use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use wasmtime::ResourceLimiter;
use wasmtime::{Config, Engine, Linker, Module, Store};
use wasmtime_wasi::p1::{WasiP1Ctx, add_to_linker_sync};
use wasmtime_wasi::p2::pipe::MemoryOutputPipe;
use wasmtime_wasi::{WasiCtxBuilder, I32Exit};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmProbeConfig {
    #[serde(default = "default_max_memory_mb")]
    pub max_memory_mb: u64,
    #[serde(default = "default_max_execution_time_ms")]
    pub max_execution_time_ms: u64,
    #[serde(default = "default_fuel_budget")]
    pub fuel_budget: u64,
    #[serde(default)]
    pub allowed_directories: Vec<PathBuf>,
    #[serde(default)]
    pub allow_network: bool,
}

fn default_max_memory_mb() -> u64 {
    256
}

fn default_max_execution_time_ms() -> u64 {
    5_000
}

fn default_fuel_budget() -> u64 {
    10_000_000
}

impl Default for WasmProbeConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: default_max_memory_mb(),
            max_execution_time_ms: default_max_execution_time_ms(),
            fuel_budget: default_fuel_budget(),
            allowed_directories: Vec::new(),
            allow_network: false,
        }
    }
}

#[derive(Debug, Error)]
pub enum WasmProbeError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("wasmtime error: {0}")]
    Wasmtime(#[from] wasmtime::Error),
    #[error("wasi error: {0}")]
    Wasi(String),
    #[error("utf8 error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("probe exceeded allowed execution time ({0} ms)")]
    Timeout(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmProbeReport {
    pub duration_ms: u128,
    pub stdout: String,
    pub stderr: String,
}

pub struct WasmProbeRunner {
    engine: Engine,
    config: WasmProbeConfig,
}

impl WasmProbeRunner {
    pub fn new(config: WasmProbeConfig) -> Result<Self, WasmProbeError> {
        let mut engine_config = Config::default();
        engine_config.consume_fuel(true);
        engine_config.wasm_multi_memory(true);
        engine_config.wasm_multi_value(true);
        // Memory limits are now configured through ResourceLimiter instead of static_memory_maximum_size
        let engine = Engine::new(&engine_config)?;
        Ok(Self { engine, config })
    }

    pub fn run_probe<P: AsRef<Path>>(
        &self,
        module_path: P,
        args: &[String],
    ) -> Result<WasmProbeReport, WasmProbeError> {
        let module = Module::from_file(&self.engine, module_path.as_ref())?;
        let mut store = self.build_store(args)?;
        store.set_fuel(self.config.fuel_budget)?;

        let mut linker = Linker::new(&self.engine);
        add_to_linker_sync(&mut linker, |state: &mut ProbeState| &mut state.wasi)
            .map_err(|err| WasmProbeError::Wasi(err.to_string()))?;

        let start = Instant::now();
        let instance = linker.instantiate(&mut store, &module)?;
        if let Some(func) = instance.get_func(&mut store, "_start") {
            let start_fn = func
                .typed::<(), ()>(&mut store)
                .map_err(WasmProbeError::Wasmtime)?;
            if let Err(err) = start_fn.call(&mut store, ()) {
                if let Some(exit) = err.downcast_ref::<I32Exit>() {
                    if exit.0 != 0 {
                        return Err(WasmProbeError::Wasi(format!(
                            "probe exited with status {}",
                            exit.0
                        )));
                    }
                } else {
                    match err.downcast::<wasmtime::Error>() {
                        Ok(wasmtime_err) => {
                            return Err(WasmProbeError::Wasmtime(wasmtime_err));
                        }
                        Err(other) => {
                            return Err(WasmProbeError::Wasi(other.to_string()));
                        }
                    }
                }
            }
        }
        let duration = start.elapsed();

        // Extract output from the pipes before dropping store
        let stdout_bytes: Vec<u8> = store.data().stdout.contents().to_vec();
        let stderr_bytes: Vec<u8> = store.data().stderr.contents().to_vec();

        drop(store);

        if duration.as_millis() > u128::from(self.config.max_execution_time_ms) {
            return Err(WasmProbeError::Timeout(self.config.max_execution_time_ms));
        }

        Ok(WasmProbeReport {
            duration_ms: duration.as_millis(),
            stdout: String::from_utf8(stdout_bytes)?,
            stderr: String::from_utf8(stderr_bytes)?,
        })
    }

    fn build_store(&self, args: &[String]) -> Result<Store<ProbeState>, WasmProbeError> {
        let stdout_stream = MemoryOutputPipe::new(1024 * 1024);
        let stderr_stream = MemoryOutputPipe::new(1024 * 1024);

        let mut builder = WasiCtxBuilder::new();
        builder.stdout(stdout_stream.clone());
        builder.stderr(stderr_stream.clone());
        builder.args(args);

        if self.config.allow_network {
            return Err(WasmProbeError::Wasi(
                "network access for probes is not yet supported".to_string(),
            ));
        }

        for dir in &self.config.allowed_directories {
            // Canonicalize the directory to prevent path traversal and ensure absolute path
            let canonical_dir = std::fs::canonicalize(dir).map_err(|err| {
                WasmProbeError::Wasi(format!(
                    "Failed to canonicalize directory '{}': {}",
                    dir.display(),
                    err
                ))
            })?;
            if !canonical_dir.is_dir() {
                return Err(WasmProbeError::Wasi(format!(
                    "Allowed directory '{}' is not a directory",
                    canonical_dir.display()
                )));
            }
            builder
                .preopened_dir(
                    &canonical_dir, 
                    canonical_dir.to_string_lossy().as_ref(),
                    wasmtime_wasi::DirPerms::all(), 
                    wasmtime_wasi::FilePerms::all()
                )
                .map_err(|err| WasmProbeError::Wasi(err.to_string()))?;
        }

        let wasi = builder.build_p1();
        let mut store = Store::new(
            &self.engine,
            ProbeState {
                wasi,
                limiter: ProbeLimiter {
                    max_memory_bytes: (self.config.max_memory_mb * 1024 * 1024) as usize,
                },
                stdout: stdout_stream,
                stderr: stderr_stream,
            },
        );
        store.limiter(|state| &mut state.limiter);
        Ok(store)
    }
}

struct ProbeLimiter {
    max_memory_bytes: usize,
}

impl ResourceLimiter for ProbeLimiter {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        let capped_max = maximum.unwrap_or(self.max_memory_bytes);
        Ok(desired <= self.max_memory_bytes && desired <= capped_max)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        if let Some(max) = maximum {
            Ok(desired <= max)
        } else {
            Ok(true)
        }
    }
}

struct ProbeState {
    wasi: WasiP1Ctx,
    limiter: ProbeLimiter,
    stdout: MemoryOutputPipe,
    stderr: MemoryOutputPipe,
}
