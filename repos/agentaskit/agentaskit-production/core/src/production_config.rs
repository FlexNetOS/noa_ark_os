use std::collections::BTreeMap;
use thiserror::Error;
use serde::{Deserialize, Serialize};

/// High-level orchestration settings for the framework.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionConfig {
    optimization: OptimizationFeatures,
    cloud: CloudFeatures,
    framework: FrameworkConfig,
}

impl ProductionConfig {
    /// Create a new production config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable auto-scaling optimization
    pub fn enable_auto_scaling(&mut self) {
        self.optimization.auto_scaling = true;
    }

    /// Enable multi-device optimization
    pub fn enable_multi_device(&mut self) {
        self.optimization.multi_device = true;
    }

    /// Enable distributed processing
    pub fn enable_distributed_processing(&mut self) {
        self.optimization.distributed_processing = true;
    }

    /// Returns the optimization features
    pub fn optimization_features(&self) -> &OptimizationFeatures {
        &self.optimization
    }

    /// Returns the cloud features
    pub fn cloud_features(&self) -> &CloudFeatures {
        &self.cloud
    }

    /// Returns the framework configuration
    pub fn framework(&self) -> &FrameworkConfig {
        &self.framework
    }

    /// Create production config from a generic config (stub for compatibility)
    pub fn from_noa_config(_config: &serde_json::Value) -> Result<Self, ProductionConfigError> {
        Ok(Self::default())
    }
}

impl Default for ProductionConfig {
    fn default() -> Self {
        Self {
            optimization: OptimizationFeatures::default(),
            cloud: CloudFeatures::default(),
            framework: FrameworkConfig::default(),
        }
    }
}

/// Optimization features configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationFeatures {
    pub auto_scaling: bool,
    pub multi_device: bool,
    pub distributed_processing: bool,
}

impl Default for OptimizationFeatures {
    fn default() -> Self {
        Self {
            auto_scaling: false,
            multi_device: false,
            distributed_processing: false,
        }
    }
}

/// Cloud features configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudFeatures {
    pub enabled: bool,
    pub provider: String,
}

impl Default for CloudFeatures {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: "local".to_string(),
        }
    }
}

/// Framework-specific configuration derived from feature flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkConfig {
    pub rust_workers: RustWorkerConfig,
    pub max_workers: usize,
    pub enable_distributed: bool,
}

impl FrameworkConfig {
    pub fn from_features(optimization: &OptimizationFeatures, _cloud: &CloudFeatures) -> Self {
        let max_workers = if optimization.auto_scaling {
            32
        } else if optimization.distributed_processing {
            16
        } else if optimization.multi_device {
            8
        } else {
            4
        };

        Self {
            rust_workers: RustWorkerConfig::default(),
            max_workers,
            enable_distributed: optimization.distributed_processing,
        }
    }
}

impl Default for FrameworkConfig {
    fn default() -> Self {
        Self {
            rust_workers: RustWorkerConfig::default(),
            max_workers: 4,
            enable_distributed: false,
        }
    }
}

/// Rust worker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustWorkerConfig {
    pub enabled: bool,
    pub max_workers: usize,
    pub optimization_level: String,
}

impl Default for RustWorkerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_workers: 4,
            optimization_level: "release".to_string(),
        }
    }
}

/// Production configuration errors
#[derive(Error, Debug)]
pub enum ProductionConfigError {
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}
