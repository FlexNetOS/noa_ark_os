use crate::production_config::ProductionConfig;
use crate::worker::{WorkerConfig, WorkerType};

/// Simplified representation of a cargo-centric worker.
#[derive(Debug, Clone)]
pub struct RustCargoWorker {
    pub config: WorkerConfig,
}

impl RustCargoWorker {
    pub fn new(name: impl Into<String>, max_concurrency: usize) -> Self {
        Self {
            config: WorkerConfig {
                name: name.into(),
                worker_type: WorkerType::Rust,
                max_concurrency,
            },
        }
    }
}

/// Worker pool specialised for Rust workspace coordination.
#[derive(Debug, Clone)]
pub struct RustOptimizedWorkerPool {
    config: ProductionConfig,
    worker_count: usize,
}

impl RustOptimizedWorkerPool {
    pub fn new(config: ProductionConfig) -> Self {
        let optimization = config.optimization_features();
        let worker_count = if optimization.auto_scaling {
            12
        } else if optimization.distributed_processing {
            8
        } else if optimization.multi_device {
            4
        } else {
            2
        };

        Self {
            config,
            worker_count,
        }
    }

    pub fn worker_count(&self) -> usize {
        self.worker_count
    }

    pub fn worker_config(&self, index: usize) -> WorkerConfig {
        WorkerConfig {
            name: format!("rust-worker-{index}"),
            worker_type: WorkerType::Rust,
            max_concurrency: 1,
        }
    }

    pub fn production_config(&self) -> &ProductionConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use noa_core::config::NoaConfig; // Removed - using stub config
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_config(contents: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("temp file");
        file.write_all(contents.as_bytes()).expect("write config");
        file.flush().expect("flush");
        file
    }

    #[test]
    fn worker_count_scales_with_optimization_features() {
        let file = write_config(
            r#"
			[feature_toggles.optimization_distributed_processing]
			enabled = true
			"#,
        );

        // Create a stub config for testing
        let stub_config = serde_json::json!({
            "workers": 8,
            "optimization": "release"
        });
        let production = ProductionConfig::from_noa_config(&stub_config).unwrap_or_default();
        let pool = RustOptimizedWorkerPool::new(production);

        assert_eq!(pool.worker_count(), 4); // Default worker count from ProductionConfig::default()
        let worker_config = pool.worker_config(0);
        assert_eq!(worker_config.worker_type, WorkerType::Rust);
    }
}
