use std::collections::HashSet;

use noa_core::hardware::{
    AcceleratorKind, CpuProfile, GpuBackend, GpuProfile, HardwareProfile, MemoryProfile,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Policy describing how runtime backends should be prioritized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimePolicy {
    pub prefer_gpu: bool,
    pub min_gpu_memory_gb: f64,
    pub prefer_lightweight_python_on_low_memory: bool,
    pub lightweight_memory_threshold_gb: f64,
    pub allow_accelerator_experiments: bool,
}

impl Default for RuntimePolicy {
    fn default() -> Self {
        Self {
            prefer_gpu: true,
            min_gpu_memory_gb: 8.0,
            prefer_lightweight_python_on_low_memory: true,
            lightweight_memory_threshold_gb: 6.0,
            allow_accelerator_experiments: true,
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
/// Errors reported when a suitable backend cannot be selected.
#[derive(Debug, Error)]
pub enum RuntimeSelectionError {
    #[error("no backend available for {component:?}")]
    NoBackend { component: RuntimeComponent },
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
}
