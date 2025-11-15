//! ARK-OS Production-Ready Core Library
//! 
//! This library unifies the best capabilities from all three Rust ecosystem repositories:
//! - rustecosys: Tauri desktop application framework
//! - rustecosys2: Advanced orchestration and execution engine  
//! - agentrs: Comprehensive multi-agent system
//! 
//! Following the "Heal, Don't Harm" principle, all capabilities are preserved and enhanced.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

// Re-export core modules
pub mod agents;
pub mod orchestration;
pub mod execution;
pub mod ui;
pub mod config;
pub mod utils;

// Core types and traits

/// Unified configuration for ARK-OS components
/// Combines features from rustecosys ArkConfig and rustecosys2 production config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArkOsConfig {
    pub component_id: Uuid,
    pub name: String,
    pub version: String,
    pub environment: Environment,
    pub settings: HashMap<String, String>,
    pub agent_config: agents::AgentSystemConfig,
    pub orchestration_config: orchestration::OrchestrationConfig,
    pub ui_config: ui::UiConfig,
    pub execution_config: execution::ExecutionConfig,
}

impl Default for ArkOsConfig {
    fn default() -> Self {
        Self {
            component_id: Uuid::new_v4(),
            name: "ark-os-production".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: Environment::Development,
            settings: HashMap::new(),
            agent_config: agents::AgentSystemConfig::default(),
            orchestration_config: orchestration::OrchestrationConfig::default(),
            ui_config: ui::UiConfig::default(),
            execution_config: execution::ExecutionConfig::default(),
        }
    }
}

/// Environment types for configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

/// Health check status for components (enhanced from rustecosys)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
    Maintenance,
}

/// System metrics aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub health_status: HealthStatus,
    pub component_metrics: HashMap<String, ComponentMetrics>,
    pub agent_metrics: agents::AgentSystemMetrics,
    pub orchestration_metrics: orchestration::OrchestrationMetrics,
    pub execution_metrics: execution::ExecutionMetrics,
}

/// Individual component metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetrics {
    pub health: HealthStatus,
    pub uptime: Duration,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub active_tasks: usize,
    pub completed_tasks: u64,
    pub error_count: u64,
}

/// Common trait for ARK-OS components (enhanced from rustecosys)
#[async_trait::async_trait]
pub trait ArkComponent: Send + Sync {
    /// Component name identifier
    fn name(&self) -> &str;
    
    /// Component version
    fn version(&self) -> &str;
    
    /// Perform health check
    async fn health_check(&self) -> Result<HealthStatus>;
    
    /// Initialize component with configuration
    async fn initialize(&mut self, config: ArkOsConfig) -> Result<()>;
    
    /// Start component operations
    async fn start(&mut self) -> Result<()>;
    
    /// Stop component operations gracefully  
    async fn stop(&mut self) -> Result<()>;
    
    /// Get component metrics
    async fn get_metrics(&self) -> Result<ComponentMetrics>;
    
    /// Handle configuration updates
    async fn update_config(&mut self, config: ArkOsConfig) -> Result<()>;
}

/// Unified ARK-OS system (integrates all three repositories)
pub struct ArkOsSystem {
    config: ArkOsConfig,
    components: HashMap<String, Box<dyn ArkComponent>>,
    agent_system: agents::AgentSystem,
    orchestration_engine: orchestration::OrchestrationEngine,
    execution_engine: execution::ExecutionEngine,
    ui_manager: ui::UiManager,
    metrics: Arc<RwLock<SystemMetrics>>,
    shutdown_signal: Arc<RwLock<bool>>,
}

impl ArkOsSystem {
    /// Create new ARK-OS system instance
    pub fn new(config: ArkOsConfig) -> Self {
        let agent_system = agents::AgentSystem::new(config.agent_config.clone());
        let orchestration_engine = orchestration::OrchestrationEngine::new(config.orchestration_config.clone());
        let execution_engine = execution::ExecutionEngine::new(config.execution_config.clone());
        let ui_manager = ui::UiManager::new(config.ui_config.clone());
        
        let initial_metrics = SystemMetrics {
            timestamp: chrono::Utc::now(),
            health_status: HealthStatus::Unknown,
            component_metrics: HashMap::new(),
            agent_metrics: agents::AgentSystemMetrics::default(),
            orchestration_metrics: orchestration::OrchestrationMetrics::default(),
            execution_metrics: execution::ExecutionMetrics::default(),
        };

        Self {
            config,
            components: HashMap::new(),
            agent_system,
            orchestration_engine,
            execution_engine,
            ui_manager,
            metrics: Arc::new(RwLock::new(initial_metrics)),
            shutdown_signal: Arc::new(RwLock::new(false)),
        }
    }

    /// Initialize the entire system
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing ARK-OS Production System");

        // Initialize core components
        self.agent_system.initialize(self.config.clone()).await?;
        self.orchestration_engine.initialize(self.config.clone()).await?;
        self.execution_engine.initialize(self.config.clone()).await?;
        self.ui_manager.initialize(self.config.clone()).await?;

        // Register components
        self.register_component("agent_system", Box::new(self.agent_system.clone())).await?;
        self.register_component("orchestration_engine", Box::new(self.orchestration_engine.clone())).await?;
        self.register_component("execution_engine", Box::new(self.execution_engine.clone())).await?;
        self.register_component("ui_manager", Box::new(self.ui_manager.clone())).await?;

        tracing::info!("ARK-OS System initialized successfully");
        Ok(())
    }

    /// Start the system
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting ARK-OS Production System");

        // Start all components
        for (name, component) in &mut self.components {
            tracing::info!("Starting component: {}", name);
            component.start().await?;
        }

        // Start metrics collection
        self.start_metrics_collection().await?;

        tracing::info!("ARK-OS System started successfully");
        Ok(())
    }

    /// Stop the system gracefully
    pub async fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping ARK-OS Production System");

        // Set shutdown signal
        *self.shutdown_signal.write().await = true;

        // Stop all components
        for (name, component) in &mut self.components {
            tracing::info!("Stopping component: {}", name);
            component.stop().await?;
        }

        tracing::info!("ARK-OS System stopped successfully");
        Ok(())
    }

    /// Register a component
    async fn register_component(&mut self, name: &str, component: Box<dyn ArkComponent>) -> Result<()> {
        self.components.insert(name.to_string(), component);
        Ok(())
    }

    /// Get system health status
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let metrics = self.metrics.read().await;
        Ok(metrics.health_status.clone())
    }

    /// Get system metrics
    pub async fn get_metrics(&self) -> Result<SystemMetrics> {
        let metrics = self.metrics.read().await;
        Ok(metrics.clone())
    }

    /// Start metrics collection background task
    async fn start_metrics_collection(&self) -> Result<()> {
        let metrics = self.metrics.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                // Check shutdown signal
                if *shutdown_signal.read().await {
                    break;
                }

                // Collect metrics (placeholder implementation)
                let mut metrics_guard = metrics.write().await;
                metrics_guard.timestamp = chrono::Utc::now();
                // TODO: Collect actual metrics from components
            }
        });

        Ok(())
    }
}

/// Utility functions (enhanced from rustecosys)
pub mod utils {
    use super::*;
    
    /// Create component configuration with defaults
    pub fn create_component_config(name: &str) -> ArkOsConfig {
        ArkOsConfig {
            component_id: Uuid::new_v4(),
            name: name.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: Environment::Development,
            settings: HashMap::new(),
            agent_config: agents::AgentSystemConfig::default(),
            orchestration_config: orchestration::OrchestrationConfig::default(),
            ui_config: ui::UiConfig::default(),
            execution_config: execution::ExecutionConfig::default(),
        }
    }
    
    /// Setup comprehensive tracing and logging
    pub fn setup_tracing(environment: &Environment) -> Result<()> {
        let filter = match environment {
            Environment::Development => "debug",
            Environment::Testing => "info", 
            Environment::Staging => "info",
            Environment::Production => "warn",
        };

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(true)
            .try_init()
            .map_err(|e| anyhow::anyhow!("Failed to initialize tracing: {}", e))?;
            
        Ok(())
    }

    /// Load configuration from file
    pub async fn load_config(path: &str) -> Result<ArkOsConfig> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: ArkOsConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to file
    pub async fn save_config(config: &ArkOsConfig, path: &str) -> Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        tokio::fs::write(path, content).await?;
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
        assert_eq!(config.environment, Environment::Development);
    }

    #[test]
    fn test_default_config() {
        let config = ArkOsConfig::default();
        assert_eq!(config.name, "ark-os-production");
        assert!(config.settings.is_empty());
        assert_eq!(config.environment, Environment::Development);
    }

    #[tokio::test]
    async fn test_system_creation() {
        let config = ArkOsConfig::default();
        let system = ArkOsSystem::new(config);
        
        let health = system.health_check().await.unwrap();
        assert_eq!(health, HealthStatus::Unknown);
    }
}