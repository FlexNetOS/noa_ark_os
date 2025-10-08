//! Configuration Module
//! 
//! Unified configuration system that combines all configuration needs
//! from the three repositories while maintaining flexibility and robustness.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::{agents, orchestration, execution, ui, Environment};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub system: SystemConfig,
    pub agents: agents::AgentSystemConfig,
    pub orchestration: orchestration::OrchestrationConfig, 
    pub execution: execution::ExecutionConfig,
    pub ui: ui::UiConfig,
    pub logging: LoggingConfig,
    pub security: SecurityConfig,
    pub features: FeatureFlags,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            system: SystemConfig::default(),
            agents: agents::AgentSystemConfig::default(),
            orchestration: orchestration::OrchestrationConfig::default(),
            execution: execution::ExecutionConfig::default(),
            ui: ui::UiConfig::default(),
            logging: LoggingConfig::default(),
            security: SecurityConfig::default(),
            features: FeatureFlags::default(),
        }
    }
}

/// System-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub name: String,
    pub version: String,
    pub environment: Environment,
    pub data_directory: String,
    pub log_directory: String,
    pub config_directory: String,
    pub max_threads: Option<usize>,
    pub shutdown_timeout_seconds: u64,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            name: "ark-os-production".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: Environment::Development,
            data_directory: "./data".to_string(),
            log_directory: "./logs".to_string(),
            config_directory: "./configs".to_string(),
            max_threads: None, // Use system default
            shutdown_timeout_seconds: 30,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    pub output: LogOutput,
    pub file_rotation: FileRotationConfig,
    pub structured_logging: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            format: LogFormat::Pretty,
            output: LogOutput::Both,
            file_rotation: FileRotationConfig::default(),
            structured_logging: false,
        }
    }
}

/// Log format options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Pretty,
    Json,
    Compact,
}

/// Log output destinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Stdout,
    File,
    Both,
    None,
}

/// File rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRotationConfig {
    pub max_file_size_mb: u64,
    pub keep_files: u32,
    pub rotate_daily: bool,
}

impl Default for FileRotationConfig {
    fn default() -> Self {
        Self {
            max_file_size_mb: 100,
            keep_files: 5,
            rotate_daily: true,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub encryption_enabled: bool,
    pub tls_enabled: bool,
    pub authentication_required: bool,
    pub session_timeout_minutes: u64,
    pub max_failed_attempts: u32,
    pub allowed_origins: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_enabled: true,
            tls_enabled: false, // Default to false for development
            authentication_required: false,
            session_timeout_minutes: 60,
            max_failed_attempts: 5,
            allowed_origins: vec!["*".to_string()], // Allow all for development
        }
    }
}

/// Feature flags for enabling/disabling capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlags {
    pub agents_enabled: bool,
    pub orchestration_enabled: bool,
    pub execution_enabled: bool,
    pub desktop_ui_enabled: bool,
    pub web_ui_enabled: bool,
    pub api_enabled: bool,
    pub autonomous_mode: bool,
    pub triple_verification: bool,
    pub auto_healing: bool,
    pub metrics_collection: bool,
    pub performance_monitoring: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            agents_enabled: true,
            orchestration_enabled: true,
            execution_enabled: true,
            desktop_ui_enabled: true,
            web_ui_enabled: true,
            api_enabled: true,
            autonomous_mode: false, // Default to false for safety
            triple_verification: true,
            auto_healing: true,
            metrics_collection: true,
            performance_monitoring: true,
        }
    }
}

/// Configuration loader and manager
pub struct ConfigManager {
    config: Config,
    config_path: String,
}

impl ConfigManager {
    /// Create new configuration manager
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            config_path: "config.json".to_string(),
        }
    }

    /// Load configuration from file
    pub async fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path_str = path.as_ref().to_string_lossy().to_string();
        
        if !Path::new(&path_str).exists() {
            tracing::warn!("Configuration file {} not found, using defaults", path_str);
            return Ok(Self {
                config: Config::default(),
                config_path: path_str,
            });
        }

        let content = tokio::fs::read_to_string(&path_str).await?;
        let config: Config = serde_json::from_str(&content)?;
        
        tracing::info!("Loaded configuration from {}", path_str);
        
        Ok(Self {
            config,
            config_path: path_str,
        })
    }

    /// Load configuration from environment variables
    pub fn load_from_env() -> Self {
        let mut config = Config::default();
        
        // Override with environment variables
        if let Ok(env) = std::env::var("ARK_OS_ENVIRONMENT") {
            config.system.environment = match env.to_lowercase().as_str() {
                "development" | "dev" => Environment::Development,
                "testing" | "test" => Environment::Testing,
                "staging" | "stage" => Environment::Staging,
                "production" | "prod" => Environment::Production,
                _ => Environment::Development,
            };
        }

        if let Ok(level) = std::env::var("ARK_OS_LOG_LEVEL") {
            config.logging.level = level;
        }

        if let Ok(data_dir) = std::env::var("ARK_OS_DATA_DIR") {
            config.system.data_directory = data_dir;
        }

        if let Ok(log_dir) = std::env::var("ARK_OS_LOG_DIR") {
            config.system.log_directory = log_dir;
        }

        if let Ok(autonomous) = std::env::var("ARK_OS_AUTONOMOUS_MODE") {
            config.features.autonomous_mode = autonomous.to_lowercase() == "true";
        }

        if let Ok(ui_port) = std::env::var("ARK_OS_UI_PORT") {
            if let Ok(port) = ui_port.parse::<u16>() {
                config.ui.web_config.port = port;
            }
        }

        if let Ok(api_port) = std::env::var("ARK_OS_API_PORT") {
            if let Ok(port) = api_port.parse::<u16>() {
                config.ui.api_config.port = port;
            }
        }

        Self {
            config,
            config_path: "env".to_string(),
        }
    }

    /// Get current configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get mutable configuration
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Save configuration to file
    pub async fn save(&self) -> Result<()> {
        if self.config_path == "env" {
            return Err(anyhow::anyhow!("Cannot save environment-loaded configuration"));
        }

        let content = serde_json::to_string_pretty(&self.config)?;
        tokio::fs::write(&self.config_path, content).await?;
        
        tracing::info!("Saved configuration to {}", self.config_path);
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        let config = &self.config;

        // Validate system configuration
        if config.system.name.is_empty() {
            return Err(anyhow::anyhow!("System name cannot be empty"));
        }

        if config.system.shutdown_timeout_seconds == 0 {
            return Err(anyhow::anyhow!("Shutdown timeout must be greater than 0"));
        }

        // Validate ports are not conflicting
        if config.ui.web_enabled && config.ui.api_enabled {
            if config.ui.web_config.port == config.ui.api_config.port {
                return Err(anyhow::anyhow!("Web and API ports cannot be the same"));
            }
        }

        // Validate directories exist or can be created
        for dir in [&config.system.data_directory, &config.system.log_directory, &config.system.config_directory] {
            if let Err(e) = std::fs::create_dir_all(dir) {
                return Err(anyhow::anyhow!("Cannot create directory {}: {}", dir, e));
            }
        }

        // Validate agent configuration
        if config.agents.max_agents == 0 {
            return Err(anyhow::anyhow!("Max agents must be greater than 0"));
        }

        tracing::info!("Configuration validation passed");
        Ok(())
    }

    /// Apply configuration to environment
    pub fn apply_to_environment(&self) -> Result<()> {
        let config = &self.config;

        // Set log level
        std::env::set_var("RUST_LOG", &config.logging.level);

        // Set directories
        std::env::set_var("ARK_OS_DATA_DIR", &config.system.data_directory);
        std::env::set_var("ARK_OS_LOG_DIR", &config.system.log_directory);
        std::env::set_var("ARK_OS_CONFIG_DIR", &config.system.config_directory);

        // Set feature flags
        std::env::set_var("ARK_OS_AUTONOMOUS_MODE", config.features.autonomous_mode.to_string());
        std::env::set_var("ARK_OS_TRIPLE_VERIFICATION", config.features.triple_verification.to_string());
        std::env::set_var("ARK_OS_AUTO_HEALING", config.features.auto_healing.to_string());

        Ok(())
    }

    /// Get environment-specific configuration
    pub fn for_environment(&self, env: Environment) -> Config {
        let mut config = self.config.clone();
        
        match env {
            Environment::Development => {
                config.logging.level = "debug".to_string();
                config.security.authentication_required = false;
                config.security.tls_enabled = false;
                config.features.autonomous_mode = false;
            }
            Environment::Testing => {
                config.logging.level = "info".to_string();
                config.security.authentication_required = false;
                config.security.tls_enabled = false;
                config.features.autonomous_mode = false;
            }
            Environment::Staging => {
                config.logging.level = "info".to_string();
                config.security.authentication_required = true;
                config.security.tls_enabled = true;
                config.features.autonomous_mode = false;
            }
            Environment::Production => {
                config.logging.level = "warn".to_string();
                config.security.authentication_required = true;
                config.security.tls_enabled = true;
                config.features.autonomous_mode = true;
            }
        }

        config
    }
}

/// Configuration builder for fluent API
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    pub fn system_name(mut self, name: String) -> Self {
        self.config.system.name = name;
        self
    }

    pub fn environment(mut self, env: Environment) -> Self {
        self.config.system.environment = env;
        self
    }

    pub fn log_level(mut self, level: String) -> Self {
        self.config.logging.level = level;
        self
    }

    pub fn data_directory(mut self, dir: String) -> Self {
        self.config.system.data_directory = dir;
        self
    }

    pub fn enable_autonomous_mode(mut self, enabled: bool) -> Self {
        self.config.features.autonomous_mode = enabled;
        self
    }

    pub fn max_agents(mut self, max: usize) -> Self {
        self.config.agents.max_agents = max;
        self
    }

    pub fn ui_ports(mut self, web_port: u16, api_port: u16) -> Self {
        self.config.ui.web_config.port = web_port;
        self.config.ui.api_config.port = api_port;
        self
    }

    pub fn build(self) -> Config {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.system.name, "ark-os-production");
        assert_eq!(config.system.environment, Environment::Development);
        assert!(config.features.agents_enabled);
        assert!(config.features.orchestration_enabled);
        assert!(!config.features.autonomous_mode); // Should be false by default
    }

    #[test]
    fn test_config_builder() {
        let config = ConfigBuilder::new()
            .system_name("test-system".to_string())
            .environment(Environment::Production)
            .log_level("debug".to_string())
            .enable_autonomous_mode(true)
            .max_agents(500)
            .ui_ports(8080, 8081)
            .build();

        assert_eq!(config.system.name, "test-system");
        assert_eq!(config.system.environment, Environment::Production);
        assert_eq!(config.logging.level, "debug");
        assert!(config.features.autonomous_mode);
        assert_eq!(config.agents.max_agents, 500);
        assert_eq!(config.ui.web_config.port, 8080);
        assert_eq!(config.ui.api_config.port, 8081);
    }

    #[tokio::test]
    async fn test_config_save_load() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path().to_string_lossy().to_string();

        let original_config = ConfigBuilder::new()
            .system_name("test-save-load".to_string())
            .environment(Environment::Testing)
            .build();

        let mut manager = ConfigManager::new();
        *manager.config_mut() = original_config.clone();
        manager.config_path = path.clone();

        manager.save().await.unwrap();

        let loaded_manager = ConfigManager::load_from_file(&path).await.unwrap();
        assert_eq!(loaded_manager.config().system.name, original_config.system.name);
        assert_eq!(loaded_manager.config().system.environment, original_config.system.environment);
    }

    #[test]
    fn test_config_validation() {
        let manager = ConfigManager::new();
        assert!(manager.validate().is_ok());

        let mut invalid_config = Config::default();
        invalid_config.system.name = "".to_string();
        
        let mut invalid_manager = ConfigManager::new();
        *invalid_manager.config_mut() = invalid_config;
        
        assert!(invalid_manager.validate().is_err());
    }
}