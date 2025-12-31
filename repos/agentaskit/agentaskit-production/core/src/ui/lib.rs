use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Core configuration for ARK-OS components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArkConfig {
    pub component_id: Uuid,
    pub name: String,
    pub version: String,
    pub settings: HashMap<String, String>,
}

impl Default for ArkConfig {
    fn default() -> Self {
        Self {
            component_id: Uuid::new_v4(),
            name: "ark-os-component".to_string(),
            version: "0.1.0".to_string(),
            settings: HashMap::new(),
        }
    }
}

/// Health check status for components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Common trait for ARK-OS components
pub trait ArkComponent {
    fn name(&self) -> &str;
    fn health_check(&self) -> Result<HealthStatus>;
    fn initialize(&mut self, config: ArkConfig) -> Result<()>;
    fn shutdown(&mut self) -> Result<()>;
}

/// Utility functions
pub mod utils {
    use super::*;
    
    pub fn create_component_config(name: &str) -> ArkConfig {
        ArkConfig {
            component_id: Uuid::new_v4(),
            name: name.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            settings: HashMap::new(),
        }
    }
    
    pub fn setup_tracing() -> Result<()> {
        tracing_subscriber::fmt()
            .with_env_filter("info")
            .try_init()
            .map_err(|e| anyhow::anyhow!("Failed to initialize tracing: {}", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = utils::create_component_config("test-component");
        assert_eq!(config.name, "test-component");
        assert_eq!(config.version, "0.1.0");
    }

    #[test]
    fn test_default_config() {
        let config = ArkConfig::default();
        assert_eq!(config.name, "ark-os-component");
        assert!(config.settings.is_empty());
    }
}
