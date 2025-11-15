use std::collections::{HashMap, HashSet};
use std::path::Path;

use noa_core::hardware::{AcceleratorKind, HardwareProfile};
#[cfg(test)]
use noa_core::hardware::{CpuProfile, GpuBackend, GpuProfile, MemoryProfile};
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod wasm;
pub use wasm::{WasmProbeConfig, WasmProbeError, WasmProbeReport, WasmProbeRunner};

/// Policy describing how runtime backends should be prioritized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimePolicy {
    pub prefer_gpu: bool,
    pub min_gpu_memory_gb: f64,
    pub prefer_lightweight_python_on_low_memory: bool,
    pub lightweight_memory_threshold_gb: f64,
    pub allow_accelerator_experiments: bool,
    #[serde(default)]
    pub enable_wasm_probes: bool,
    #[serde(default)]
    pub wasm_probe_config: WasmProbeConfig,
}

impl Default for RuntimePolicy {
    fn default() -> Self {
        Self {
            prefer_gpu: true,
            min_gpu_memory_gb: 8.0,
            prefer_lightweight_python_on_low_memory: true,
            lightweight_memory_threshold_gb: 6.0,
            allow_accelerator_experiments: true,
            enable_wasm_probes: false,
            wasm_probe_config: WasmProbeConfig::default(),
        }
    }
}

/// Component type managed by the runtime manager.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RuntimeComponent {
    LanguageModelBackend,
    PythonInterpreter,
    AcceleratorOrchestration,
}

/// Available execution backends for each runtime component.
///
/// Note: `Eq` is intentionally NOT derived for this enum because the
/// `LlamaCppGpu` variant contains a floating-point field (`memory_gb: Option<f64>`).
/// Floating-point types do not implement `Eq` due to NaN and comparison semantics.
///
/// # Important
/// - This enum derives `PartialEq` but **not** `Eq`. This means equality comparisons are possible,
///   but may behave unexpectedly if any `memory_gb` field contains `NaN` (since `NaN != NaN`).
/// - As a result, two otherwise identical `ExecutionBackend::LlamaCppGpu` values with `memory_gb: Some(NaN)`
///   will not compare equal.
/// - **Do not use this type as a key in `HashMap` or `HashSet`**, as these collections require `Eq` for correct behavior.
///   Using this type as a key may result in subtle bugs or incorrect behavior.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionBackend {
    LlamaCppCpu,
    LlamaCppGpu {
        vendor: Option<String>,
        memory_gb: Option<f64>,
    },
    PythonLightweight,
    PythonCPython,
    AcceleratorOffload {
        kind: String,
        vendor: Option<String>,
    },
}

/// Assignment of a backend to a component with explanatory context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendSelection {
    pub component: RuntimeComponent,
    pub backend: ExecutionBackend,
    pub reason: String,
}

/// Complete execution plan for a deployment.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimePlan {
    pub selections: Vec<BackendSelection>,
    pub fallbacks: Vec<ExecutionBackend>,
    pub notes: Vec<String>,
}

impl RuntimePlan {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum HostClassification {
    Minimal,
    Standard,
    Accelerated,
}

impl Default for HostClassification {
    fn default() -> Self {
        HostClassification::Standard
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilitySignal {
    pub os: String,
    pub cpu_cores: usize,
    pub memory_gb: f64,
    pub gpu_count: usize,
    pub workloads: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelRuntimeService {
    pub id: String,
    pub requires: Vec<String>,
    #[serde(default)]
    pub optional: Vec<String>,
    #[serde(default)]
    pub supported_classes: Vec<HostClassification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelRuntimeGraph {
    pub boot_order: Vec<String>,
    pub services: Vec<KernelRuntimeService>,
}

impl KernelRuntimeGraph {
    pub fn service(&self, id: &str) -> Option<&KernelRuntimeService> {
        self.services.iter().find(|service| service.id == id)
    }

    /// Validate the runtime graph to ensure dependencies are resolvable and acyclic.
    pub fn validate(&self) -> std::result::Result<(), GraphValidationError> {
        let mut known = HashSet::new();
        for service in &self.services {
            if !known.insert(service.id.clone()) {
                return Err(GraphValidationError::DuplicateService {
                    id: service.id.clone(),
                });
            }
        }

        for id in &self.boot_order {
            if !known.contains(id) {
                return Err(GraphValidationError::UnknownBootService { id: id.clone() });
            }
        }

        for id in &known {
            if !self.boot_order.iter().any(|entry| entry == id) {
                return Err(GraphValidationError::ServiceMissingFromBootOrder { id: id.clone() });
            }
        }

        for service in &self.services {
            for dependency in service.requires.iter().chain(service.optional.iter()) {
                if !known.contains(dependency) {
                    return Err(GraphValidationError::UnknownDependency {
                        service: service.id.clone(),
                        dependency: dependency.clone(),
                    });
                }
            }
        }

        self.detect_cycles()
    }

    fn detect_cycles(&self) -> std::result::Result<(), GraphValidationError> {
        let adjacency: HashMap<&str, Vec<&str>> = self
            .services
            .iter()
            .map(|service| {
                (
                    service.id.as_str(),
                    service.requires.iter().map(|dep| dep.as_str()).collect(),
                )
            })
            .collect();

        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();
        let mut stack = Vec::new();

        for node in adjacency.keys() {
            if !visited.contains(*node) {
                dfs(node, &adjacency, &mut visited, &mut visiting, &mut stack)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum GraphValidationError {
    #[error("duplicate service id '{id}' in kernel runtime graph")]
    DuplicateService { id: String },
    #[error("service '{service}' references unknown dependency '{dependency}'")]
    UnknownDependency { service: String, dependency: String },
    #[error("service '{id}' missing from boot order")]
    ServiceMissingFromBootOrder { id: String },
    #[error("boot order references unknown service '{id}'")]
    UnknownBootService { id: String },
    #[error("cyclic dependency detected: {cycle:?}")]
    CyclicDependency { cycle: Vec<String> },
}

fn dfs<'a>(
    node: &'a str,
    adjacency: &HashMap<&'a str, Vec<&'a str>>,
    visited: &mut HashSet<&'a str>,
    visiting: &mut HashSet<&'a str>,
    stack: &mut Vec<&'a str>,
) -> std::result::Result<(), GraphValidationError> {
    visiting.insert(node);
    stack.push(node);

    if let Some(children) = adjacency.get(node) {
        for child in children {
            if visiting.contains(child) {
                let start = stack
                    .iter()
                    .rposition(|candidate| candidate == child)
                    .unwrap_or(0);
                let mut cycle: Vec<String> = stack[start..]
                    .iter()
                    .map(|value| (*value).to_string())
                    .collect();
                cycle.push((*child).to_string());
                return Err(GraphValidationError::CyclicDependency { cycle });
            }

            if !visited.contains(child) {
                dfs(child, adjacency, visited, visiting, stack)?;
            }
        }
    }

    visiting.remove(node);
    visited.insert(node);
    stack.pop();
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilityAssessment {
    pub classification: HostClassification,
    pub signal: CapabilitySignal,
    pub plan: RuntimePlan,
    pub unsupported_dependencies: Vec<String>,
    pub fallback_notes: Vec<String>,
}

pub struct AdaptiveRuntimeController {
    policy: RuntimePolicy,
    graph: KernelRuntimeGraph,
}

impl AdaptiveRuntimeController {
    pub fn new(policy: RuntimePolicy, graph: KernelRuntimeGraph) -> Self {
        Self { policy, graph }
    }

    pub fn detect(&self, profile: &HardwareProfile, workloads: &[String]) -> CapabilitySignal {
        CapabilitySignal {
            os: std::env::consts::OS.to_string(),
            cpu_cores: profile.cpu.logical_cores,
            memory_gb: profile.total_memory_gb(),
            gpu_count: profile.gpus.len(),
            workloads: workloads.to_vec(),
        }
    }

    fn classify(&self, signal: &CapabilitySignal) -> HostClassification {
        if signal.gpu_count > 0 && signal.memory_gb >= self.policy.min_gpu_memory_gb {
            HostClassification::Accelerated
        } else if signal.memory_gb >= self.policy.lightweight_memory_threshold_gb {
            HostClassification::Standard
        } else {
            HostClassification::Minimal
        }
    }

    fn unsupported(
        &self,
        classification: &HostClassification,
        workloads: &[String],
    ) -> Vec<String> {
        let mut unsupported = Vec::new();
        for workload in workloads {
            if let Some(service) = self.graph.service(workload) {
                if !service.supported_classes.is_empty()
                    && !service.supported_classes.contains(classification)
                {
                    unsupported.push(workload.clone());
                }
            }
        }
        unsupported
    }

    pub fn plan(
        &self,
        profile: &HardwareProfile,
        workloads: &[String],
    ) -> Result<CapabilityAssessment> {
        let signal = self.detect(profile, workloads);
        let classification = self.classify(&signal);
        let mut plan = select_execution_plan(profile, &self.policy)?;
        plan.notes
            .push(format!("Host classified as {:?}", classification));

        let unsupported = self.unsupported(&classification, workloads);
        let mut fallback_notes = Vec::new();
        if !unsupported.is_empty() {
            fallback_notes.push(format!(
                "Unsupported workloads for classification {:?}: {:?}",
                classification, unsupported
            ));
            plan.notes
                .push("Applied degraded mode for unsupported workloads".to_string());
        }

        if classification == HostClassification::Minimal {
            plan.fallbacks.push(ExecutionBackend::PythonLightweight);
            plan.notes
                .push("Enforced lightweight fallbacks for minimal profile".to_string());
            deduplicate_fallbacks(&mut plan);
        }

        Ok(CapabilityAssessment {
            classification,
            signal,
            plan,
            unsupported_dependencies: unsupported,
            fallback_notes,
        })
    }

    pub fn run_wasm_probe<P: AsRef<Path>>(
        &self,
        module_path: P,
        args: &[String],
    ) -> Result<Option<WasmProbeReport>> {
        if !self.policy.enable_wasm_probes {
            return Ok(None);
        }
        let runner = WasmProbeRunner::new(self.policy.wasm_probe_config.clone())?;
        let report = runner.run_probe(module_path, args)?;
        Ok(Some(report))
    }
}
/// Errors reported when a suitable backend cannot be selected.
#[derive(Debug, Error)]
pub enum RuntimeSelectionError {
    #[error("no backend available for {component:?}")]
    NoBackend { component: RuntimeComponent },
    #[error("wasm probe failed: {source}")]
    WasmProbe {
        #[from]
        source: WasmProbeError,
    },
}

pub type Result<T> = std::result::Result<T, RuntimeSelectionError>;

/// Select execution backends for the supplied hardware profile.
pub fn select_execution_plan(
    profile: &HardwareProfile,
    policy: &RuntimePolicy,
) -> Result<RuntimePlan> {
    let mut plan = RuntimePlan::default();

    // Language model backend selection
    let llama_backend = choose_llama_backend(profile, policy);
    plan.fallbacks.push(ExecutionBackend::LlamaCppCpu);
    match &llama_backend {
        ExecutionBackend::LlamaCppGpu { .. } => {
            plan.selections.push(BackendSelection {
                component: RuntimeComponent::LanguageModelBackend,
                reason: describe_gpu_choice(profile, policy),
                backend: llama_backend.clone(),
            });
            plan.notes
                .push("GPU acceleration enabled for llama.cpp".to_string());
        }
        ExecutionBackend::LlamaCppCpu => {
            plan.selections.push(BackendSelection {
                component: RuntimeComponent::LanguageModelBackend,
                reason: "GPU acceleration unavailable or does not meet policy".to_string(),
                backend: llama_backend.clone(),
            });
        }
        _ => {
            return Err(RuntimeSelectionError::NoBackend {
                component: RuntimeComponent::LanguageModelBackend,
            });
        }
    }

    // Python runtime selection
    let python_backend = choose_python_backend(profile, policy);
    plan.fallbacks.push(ExecutionBackend::PythonLightweight);
    plan.selections.push(BackendSelection {
        component: RuntimeComponent::PythonInterpreter,
        reason: describe_python_choice(profile, policy, &python_backend),
        backend: python_backend,
    });

    // Optional accelerator selection
    if policy.allow_accelerator_experiments {
        if let Some(accelerator) = profile
            .accelerators
            .iter()
            .find(|accel| accel.kind != AcceleratorKind::Gpu)
        {
            plan.selections.push(BackendSelection {
                component: RuntimeComponent::AcceleratorOrchestration,
                reason: format!(
                    "Detected additional accelerator kind {:?}",
                    accelerator.kind
                ),
                backend: ExecutionBackend::AcceleratorOffload {
                    kind: format!("{:?}", accelerator.kind),
                    vendor: accelerator.vendor.clone(),
                },
            });
            plan.notes
                .push("Experimental accelerator pathways enabled".to_string());
        }
    }

    deduplicate_fallbacks(&mut plan);

    Ok(plan)
}

fn choose_llama_backend(profile: &HardwareProfile, policy: &RuntimePolicy) -> ExecutionBackend {
    if policy.prefer_gpu {
        let candidate = profile
            .gpus
            .iter()
            .filter_map(|gpu| {
                gpu.memory_total_bytes
                    .map(|bytes| (gpu, bytes as f64 / (1024.0 * 1024.0 * 1024.0)))
            })
            .find(|(_, memory_gb)| *memory_gb >= policy.min_gpu_memory_gb);

        if let Some((gpu, memory_gb)) = candidate {
            return ExecutionBackend::LlamaCppGpu {
                vendor: Some(gpu.backend.vendor_name().to_string()),
                memory_gb: Some(memory_gb),
            };
        }
    }

    ExecutionBackend::LlamaCppCpu
}

fn describe_gpu_choice(profile: &HardwareProfile, policy: &RuntimePolicy) -> String {
    if let Some((gpu, memory_gb)) = profile
        .gpus
        .iter()
        .filter_map(|gpu| {
            gpu.memory_total_bytes
                .map(|bytes| (gpu, bytes as f64 / (1024.0 * 1024.0 * 1024.0)))
        })
        .find(|(_, memory_gb)| *memory_gb >= policy.min_gpu_memory_gb)
    {
        format!(
            "Using {} GPU with {:.1} GiB for llama.cpp backend",
            gpu.backend.vendor_name(),
            memory_gb
        )
    } else {
        "GPU preference enabled but no GPU met the policy thresholds".to_string()
    }
}

fn choose_python_backend(profile: &HardwareProfile, policy: &RuntimePolicy) -> ExecutionBackend {
    let total_gb = profile.total_memory_gb();
    let available_gb = profile.available_memory_gb();

    if policy.prefer_lightweight_python_on_low_memory
        && (total_gb < policy.lightweight_memory_threshold_gb
            || available_gb < policy.lightweight_memory_threshold_gb)
    {
        ExecutionBackend::PythonLightweight
    } else {
        ExecutionBackend::PythonCPython
    }
}

fn describe_python_choice(
    profile: &HardwareProfile,
    policy: &RuntimePolicy,
    backend: &ExecutionBackend,
) -> String {
    match backend {
        ExecutionBackend::PythonLightweight => format!(
            "Selected lightweight Python runtime due to {:.1} GiB total / {:.1} GiB available memory",
            profile.total_memory_gb(),
            profile.available_memory_gb()
        ),
        ExecutionBackend::PythonCPython => {
            if policy.prefer_lightweight_python_on_low_memory {
                format!(
                    "Full CPython runtime enabled (memory {:.1} GiB total, {:.1} GiB available)",
                    profile.total_memory_gb(),
                    profile.available_memory_gb()
                )
            } else {
                "Policy prefers full CPython runtime".to_string()
            }
        }
        _ => "Unexpected backend for Python runtime".to_string(),
    }
}

fn deduplicate_fallbacks(plan: &mut RuntimePlan) {
    let mut seen = HashSet::new();
    plan.fallbacks
        .retain(|backend| seen.insert(std::mem::discriminant(backend)));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use tempfile::tempdir;
    use wat::parse_str;

    fn cpu() -> CpuProfile {
        CpuProfile {
            brand: "Test CPU".into(),
            vendor: "TestVendor".into(),
            physical_cores: 4,
            logical_cores: 8,
            frequency_mhz: Some(2400),
        }
    }

    fn mem(total_gb: u64, available_gb: u64) -> MemoryProfile {
        MemoryProfile {
            total_bytes: total_gb * 1024 * 1024 * 1024,
            available_bytes: available_gb * 1024 * 1024 * 1024,
        }
    }

    #[test]
    fn selects_gpu_when_available() {
        let profile = HardwareProfile {
            cpu: cpu(),
            memory: mem(32, 20),
            gpus: vec![GpuProfile {
                name: "NVIDIA RTX".into(),
                backend: GpuBackend::Nvidia,
                memory_total_bytes: Some(16 * 1024 * 1024 * 1024),
                driver: Some("550".into()),
            }],
            accelerators: vec![],
        };
        let policy = RuntimePolicy::default();

        let plan = select_execution_plan(&profile, &policy).unwrap();
        let llama_backend = &plan.selections[0];
        assert!(matches!(
            llama_backend.backend,
            ExecutionBackend::LlamaCppGpu { .. }
        ));
    }

    #[test]
    fn falls_back_to_cpu_without_gpu() {
        let profile = HardwareProfile {
            cpu: cpu(),
            memory: mem(16, 12),
            gpus: vec![],
            accelerators: vec![],
        };
        let policy = RuntimePolicy::default();

        let plan = select_execution_plan(&profile, &policy).unwrap();
        let llama_backend = &plan.selections[0];
        assert!(matches!(
            llama_backend.backend,
            ExecutionBackend::LlamaCppCpu
        ));
    }

    #[test]
    fn selects_lightweight_python_on_low_memory() {
        let profile = HardwareProfile {
            cpu: cpu(),
            memory: mem(4, 2),
            gpus: vec![],
            accelerators: vec![],
        };
        let policy = RuntimePolicy::default();

        let plan = select_execution_plan(&profile, &policy).unwrap();
        let python_backend = &plan.selections[1];
        assert!(matches!(
            python_backend.backend,
            ExecutionBackend::PythonLightweight
        ));
    }

    fn runtime_graph() -> KernelRuntimeGraph {
        KernelRuntimeGraph {
            boot_order: vec!["kernel".into(), "runtime-manager".into(), "gateway".into()],
            services: vec![
                KernelRuntimeService {
                    id: "kernel".into(),
                    requires: vec![],
                    optional: vec![],
                    supported_classes: vec![
                        HostClassification::Minimal,
                        HostClassification::Standard,
                        HostClassification::Accelerated,
                    ],
                },
                KernelRuntimeService {
                    id: "observability".into(),
                    requires: vec!["kernel".into()],
                    optional: vec![],
                    supported_classes: vec![
                        HostClassification::Minimal,
                        HostClassification::Standard,
                        HostClassification::Accelerated,
                    ],
                },
                KernelRuntimeService {
                    id: "runtime-manager".into(),
                    requires: vec!["kernel".into()],
                    optional: vec!["observability".into()],
                    supported_classes: vec![
                        HostClassification::Minimal,
                        HostClassification::Standard,
                        HostClassification::Accelerated,
                    ],
                },
                KernelRuntimeService {
                    id: "gateway".into(),
                    requires: vec!["runtime-manager".into()],
                    optional: vec![],
                    supported_classes: vec![
                        HostClassification::Standard,
                        HostClassification::Accelerated,
                    ],
                },
            ],
        }
    }

    #[test]
    fn validates_kernel_graph_fixture() {
        runtime_graph().validate().unwrap();
    }

    #[test]
    fn detects_cycles_in_graph() {
        let graph = KernelRuntimeGraph {
            boot_order: vec!["a".into(), "b".into()],
            services: vec![
                KernelRuntimeService {
                    id: "a".into(),
                    requires: vec!["b".into()],
                    optional: vec![],
                    supported_classes: vec![HostClassification::Standard],
                },
                KernelRuntimeService {
                    id: "b".into(),
                    requires: vec!["a".into()],
                    optional: vec![],
                    supported_classes: vec![HostClassification::Standard],
                },
            ],
        };

        let err = graph.validate().unwrap_err();
        assert!(matches!(err, GraphValidationError::CyclicDependency { .. }));
    }

    #[test]
    fn adaptive_controller_classifies_accelerated() {
        let profile = HardwareProfile {
            cpu: cpu(),
            memory: mem(64, 48),
            gpus: vec![GpuProfile {
                name: "NVIDIA RTX".into(),
                backend: GpuBackend::Nvidia,
                memory_total_bytes: Some(16 * 1024 * 1024 * 1024),
                driver: Some("550".into()),
            }],
            accelerators: vec![],
        };
        let controller = AdaptiveRuntimeController::new(RuntimePolicy::default(), runtime_graph());
        let workloads = vec!["gateway".to_string()];
        let assessment = controller.plan(&profile, &workloads).unwrap();
        assert_eq!(assessment.classification, HostClassification::Accelerated);
        assert!(assessment.unsupported_dependencies.is_empty());
        assert!(assessment
            .plan
            .selections
            .iter()
            .any(|selection| matches!(selection.backend, ExecutionBackend::LlamaCppGpu { .. })));
    }

    #[test]
    fn adaptive_controller_flags_unsupported_for_minimal_host() {
        let profile = HardwareProfile {
            cpu: cpu(),
            memory: mem(4, 2),
            gpus: vec![],
            accelerators: vec![],
        };
        let controller = AdaptiveRuntimeController::new(RuntimePolicy::default(), runtime_graph());
        let workloads = vec!["gateway".to_string()];
        let assessment = controller.plan(&profile, &workloads).unwrap();
        assert_eq!(assessment.classification, HostClassification::Minimal);
        assert_eq!(
            assessment.unsupported_dependencies,
            vec!["gateway".to_string()]
        );
        assert!(assessment
            .plan
            .fallbacks
            .iter()
            .any(|backend| matches!(backend, ExecutionBackend::PythonLightweight)));
        assert!(!assessment.fallback_notes.is_empty());
    }

    #[test]
    fn wasm_probe_runner_executes_minimal_module() {
        let dir = tempdir().unwrap();
        let module_path = dir.path().join("probe.wasm");
        let wasm_bytes = parse_str(
            r#"(module
                (import "wasi_snapshot_preview1" "proc_exit" (func $__wasi_proc_exit (param i32)))
                (memory (export "memory") 1)
                (func $_start
                    i32.const 0
                    call $__wasi_proc_exit)
                (export "_start" (func $_start)))"#,
        )
        .unwrap();
        fs::write(&module_path, wasm_bytes).unwrap();

        let mut policy = RuntimePolicy::default();
        policy.enable_wasm_probes = true;
        policy.wasm_probe_config = WasmProbeConfig {
            allow_network: false,
            ..WasmProbeConfig::default()
        };

        let controller = AdaptiveRuntimeController::new(policy, runtime_graph());
        let report = controller
            .run_wasm_probe(&module_path, &[])
            .expect("probe run should succeed")
            .expect("runner enabled");
        assert!(report.duration_ms < 1_000);
        assert!(report.stdout.is_empty());
        assert!(report.stderr.is_empty());
    }

    #[test]
    fn wasm_probe_respects_timeout_budget() {
        let dir = tempdir().unwrap();
        let module_path = dir.path().join("timeout.wasm");
        let wasm_bytes = parse_str(
            r#"(module
                (import "wasi_snapshot_preview1" "proc_exit" (func $__wasi_proc_exit (param i32)))
                (memory (export "memory") 1)
                (func $_start
                    (loop
                        br 0))
                (export "_start" (func $_start)))"#,
        )
        .unwrap();
        fs::write(&module_path, wasm_bytes).unwrap();

        let mut policy = RuntimePolicy::default();
        policy.enable_wasm_probes = true;
        policy.wasm_probe_config = WasmProbeConfig {
            max_execution_time_ms: 1,
            ..WasmProbeConfig::default()
        };

        let controller = AdaptiveRuntimeController::new(policy, runtime_graph());
        let result = controller.run_wasm_probe(&module_path, &[]);
        assert!(matches!(
            result,
            Err(RuntimeSelectionError::WasmProbe { .. })
        ));
    }

    #[test]
    fn wasm_probe_enforces_memory_limits() {
        let dir = tempdir().unwrap();
        let module_path = dir.path().join("memory.wasm");
        let wasm_bytes = parse_str(
            r#"(module
                (import "wasi_snapshot_preview1" "proc_exit" (func $__wasi_proc_exit (param i32)))
                (memory (export "memory") 1)
                (func $_start
                    i32.const 0
                    loop
                        i32.const 1
                        memory.grow
                        drop
                        br 0
                    end
                    i32.const 0
                    call $__wasi_proc_exit)
                (export "_start" (func $_start)))"#,
        )
        .unwrap();
        fs::write(&module_path, wasm_bytes).unwrap();

        let mut policy = RuntimePolicy::default();
        policy.enable_wasm_probes = true;
        policy.wasm_probe_config = WasmProbeConfig {
            max_memory_mb: 1,
            ..WasmProbeConfig::default()
        };

        let controller = AdaptiveRuntimeController::new(policy, runtime_graph());
        let result = controller.run_wasm_probe(&module_path, &[]);
        assert!(matches!(
            result,
            Err(RuntimeSelectionError::WasmProbe { .. })
        ));
    }
}
