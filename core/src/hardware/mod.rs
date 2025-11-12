use serde::{Deserialize, Serialize};
use std::process::Command;
use sysinfo::{CpuExt, System, SystemExt};

/// Summary of CPU capabilities discovered on the host system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuProfile {
    pub brand: String,
    pub vendor: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub frequency_mhz: Option<u64>,
}

/// Summary of physical memory that can be scheduled by the runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProfile {
    pub total_bytes: u64,
    pub available_bytes: u64,
}

/// GPU family identifiers used to drive backend selection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GpuBackend {
    Nvidia,
    Amd,
    Intel,
    Apple,
    Unknown,
}

impl GpuBackend {
    fn from_vendor_hint(vendor_id: Option<&str>, name: &str) -> Self {
        let vendor = vendor_id.unwrap_or("").to_ascii_lowercase();
        if vendor.contains("10de") || name.to_ascii_lowercase().contains("nvidia") {
            GpuBackend::Nvidia
        } else if vendor.contains("1002")
            || vendor.contains("1022")
            || name.to_ascii_lowercase().contains("amd")
            || name.to_ascii_lowercase().contains("radeon")
        {
            GpuBackend::Amd
        } else if vendor.contains("8086") || name.to_ascii_lowercase().contains("intel") {
            GpuBackend::Intel
        } else if vendor.contains("106b") || name.to_ascii_lowercase().contains("apple") {
            GpuBackend::Apple
        } else {
            GpuBackend::Unknown
        }
    }

    pub fn vendor_name(&self) -> &'static str {
        match self {
            GpuBackend::Nvidia => "NVIDIA",
            GpuBackend::Amd => "AMD",
            GpuBackend::Intel => "Intel",
            GpuBackend::Apple => "Apple",
            GpuBackend::Unknown => "Unknown",
        }
    }
}

/// GPU hardware capabilities that are visible to the runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuProfile {
    pub name: String,
    pub backend: GpuBackend,
    pub memory_total_bytes: Option<u64>,
    pub driver: Option<String>,
}

/// Specialized accelerator types that may be exposed to workloads.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AcceleratorKind {
    Gpu,
    Tpu,
    Npu,
    Vpu,
    Other,
}

/// Description of an accelerator instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceleratorProfile {
    pub kind: AcceleratorKind,
    pub vendor: Option<String>,
    pub model: Option<String>,
    pub details: Option<String>,
}

/// Aggregated hardware view shared with higher system layers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub cpu: CpuProfile,
    pub memory: MemoryProfile,
    pub gpus: Vec<GpuProfile>,
    pub accelerators: Vec<AcceleratorProfile>,
}

impl HardwareProfile {
    pub fn total_memory_gb(&self) -> f64 {
        (self.memory.total_bytes as f64) / (1024.0 * 1024.0 * 1024.0)
    }

    pub fn available_memory_gb(&self) -> f64 {
        (self.memory.available_bytes as f64) / (1024.0 * 1024.0 * 1024.0)
    }

    pub fn has_gpu(&self) -> bool {
        !self.gpus.is_empty()
    }
}

/// Detect hardware capabilities on the current host.
pub fn detect_hardware_profile() -> HardwareProfile {
    let mut system = System::new_all();
    system.refresh_all();

    let cpus = system.cpus();
    let logical_cores = cpus.len();
    let cpu = cpus
        .first()
        .map(|cpu| CpuProfile {
            brand: cpu.brand().to_string(),
            vendor: cpu.vendor_id().to_string(),
            physical_cores: system.physical_core_count().unwrap_or(logical_cores),
            logical_cores,
            frequency_mhz: Some(cpu.frequency() as u64),
        })
        .unwrap_or(CpuProfile {
            brand: "unknown".to_string(),
            vendor: "unknown".to_string(),
            physical_cores: system.physical_core_count().unwrap_or(1),
            logical_cores: system.cpus().len().max(1),
            frequency_mhz: None,
        });

    let memory = MemoryProfile {
        total_bytes: system.total_memory() * 1024,
        available_bytes: system.available_memory() * 1024,
    };

    let gpus = detect_gpus(&system);
    let accelerators = detect_accelerators(&gpus);

    HardwareProfile {
        cpu,
        memory,
        gpus,
        accelerators,
    }
}

fn detect_gpus(system: &System) -> Vec<GpuProfile> {
    let mut gpus = Vec::new();

    for card in system.graphics_cards() {
        let backend = GpuBackend::from_vendor_hint(card.vendor_id(), card.name());
        gpus.push(GpuProfile {
            name: card.name().to_string(),
            backend,
            memory_total_bytes: card.memory_total(),
            driver: card.driver_version().map(|d| d.to_string()),
        });
    }

    if gpus.is_empty() {
        gpus.extend(query_nvidia_smi());
    }

    gpus
}

fn query_nvidia_smi() -> Vec<GpuProfile> {
    let mut gpus = Vec::new();
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total,driver_version",
            "--format=csv,noheader,nounits",
        ])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                let parts: Vec<_> = line.split(',').map(|s| s.trim()).collect();
                if parts.len() < 3 || parts[0].is_empty() {
                    continue;
                }

                let name = parts[0].to_string();
                let memory_total_bytes = parts
                    .get(1)
                    .and_then(|value| value.parse::<u64>().ok())
                    .map(|mb| mb * 1024 * 1024);
                let driver = parts.get(2).map(|s| s.to_string());

                gpus.push(GpuProfile {
                    name,
                    backend: GpuBackend::Nvidia,
                    memory_total_bytes,
                    driver,
                });
            }
        }
    }

    gpus
}

fn detect_accelerators(gpus: &[GpuProfile]) -> Vec<AcceleratorProfile> {
    let mut accelerators = Vec::new();

    for gpu in gpus {
        accelerators.push(AcceleratorProfile {
            kind: AcceleratorKind::Gpu,
            vendor: Some(gpu.backend.vendor_name().to_string()),
            model: Some(gpu.name.clone()),
            details: gpu.driver.clone(),
        });
    }

    if std::env::var("TPU_VISIBLE_DEVICES").is_ok() {
        accelerators.push(AcceleratorProfile {
            kind: AcceleratorKind::Tpu,
            vendor: Some("Google".to_string()),
            model: None,
            details: Some("Detected TPU_VISIBLE_DEVICES".to_string()),
        });
    }

    if std::env::var("NEURO_ACCELERATORS").is_ok() {
        accelerators.push(AcceleratorProfile {
            kind: AcceleratorKind::Npu,
            vendor: None,
            model: None,
            details: Some("NEURO_ACCELERATORS environment variable".to_string()),
        });
    }

    accelerators
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_detection_from_vendor_id() {
        assert_eq!(
            GpuBackend::from_vendor_hint(Some("10DE"), "GeForce"),
            GpuBackend::Nvidia
        );
        assert_eq!(
            GpuBackend::from_vendor_hint(Some("1002"), "Radeon"),
            GpuBackend::Amd
        );
        assert_eq!(
            GpuBackend::from_vendor_hint(Some("8086"), "Intel UHD"),
            GpuBackend::Intel
        );
        assert_eq!(
            GpuBackend::from_vendor_hint(None, "Apple M2"),
            GpuBackend::Apple
        );
    }

    #[test]
    fn hardware_profile_memory_helpers() {
        let profile = HardwareProfile {
            cpu: CpuProfile {
                brand: "test".into(),
                vendor: "test".into(),
                physical_cores: 1,
                logical_cores: 1,
                frequency_mhz: None,
            },
            memory: MemoryProfile {
                total_bytes: 8 * 1024 * 1024 * 1024,
                available_bytes: 4 * 1024 * 1024 * 1024,
            },
            gpus: Vec::new(),
            accelerators: Vec::new(),
        };

        assert!((profile.total_memory_gb() - 8.0).abs() < f64::EPSILON);
        assert!((profile.available_memory_gb() - 4.0).abs() < f64::EPSILON);
        assert!(!profile.has_gpu());
    }
}
