use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::{self, Display};

/// Common error types used across AgentAsKit systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentAsKitError {
    // Agent lifecycle errors
    AgentNotFound(uuid::Uuid),
    AgentAlreadyExists(uuid::Uuid),
    AgentStartupFailed(String),
    AgentShutdownTimeout(uuid::Uuid),
    
    // Task execution errors
    TaskNotFound(uuid::Uuid),
    TaskExecutionFailed(String),
    TaskTimeout(uuid::Uuid),
    TaskDependencyFailed(uuid::Uuid),
    
    // Communication errors
    MessageDeliveryFailed(String),
    CommunicationTimeout,
    InvalidMessageFormat(String),
    UnauthorizedAccess(uuid::Uuid),
    
    // Resource errors
    InsufficientResources(String),
    ResourceAllocationFailed(String),
    ResourceLimitExceeded(String),
    
    // Health monitoring errors
    HealthCheckFailed(String),
    MetricsCollectionFailed(String),
    AlertingSystemDown,
    
    // Deployment errors
    DeploymentFailed(String),
    ManifestValidationFailed(String),
    ScalingFailed(String),
    RollbackFailed(String),
    
    // Capability errors
    CapabilityDenied(String),
    InvalidCapabilityToken(String),
    CapabilityExpired(uuid::Uuid),
    
    // FlexNetOS specific errors
    SandboxCreationFailed(String),
    WasmExecutionFailed(String),
    ContractValidationFailed(String),
    
    // NOA specific errors
    ManifestParsingFailed(String),
    AgentRegistrationFailed(String),
    HealthMonitoringFailed(String),
    
    // System errors
    ConfigurationError(String),
    DatabaseConnectionFailed(String),
    NetworkError(String),
    FileSystemError(String),
    
    // Generic errors
    Internal(String),
    NotImplemented(String),
    ValidationFailed(String),
}

impl Display for AgentAsKitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentAsKitError::AgentNotFound(id) => write!(f, "Agent not found: {}", id),
            AgentAsKitError::AgentAlreadyExists(id) => write!(f, "Agent already exists: {}", id),
            AgentAsKitError::AgentStartupFailed(msg) => write!(f, "Agent startup failed: {}", msg),
            AgentAsKitError::AgentShutdownTimeout(id) => write!(f, "Agent shutdown timeout: {}", id),
            
            AgentAsKitError::TaskNotFound(id) => write!(f, "Task not found: {}", id),
            AgentAsKitError::TaskExecutionFailed(msg) => write!(f, "Task execution failed: {}", msg),
            AgentAsKitError::TaskTimeout(id) => write!(f, "Task timeout: {}", id),
            AgentAsKitError::TaskDependencyFailed(id) => write!(f, "Task dependency failed: {}", id),
            
            AgentAsKitError::MessageDeliveryFailed(msg) => write!(f, "Message delivery failed: {}", msg),
            AgentAsKitError::CommunicationTimeout => write!(f, "Communication timeout"),
            AgentAsKitError::InvalidMessageFormat(msg) => write!(f, "Invalid message format: {}", msg),
            AgentAsKitError::UnauthorizedAccess(id) => write!(f, "Unauthorized access by agent: {}", id),
            
            AgentAsKitError::InsufficientResources(msg) => write!(f, "Insufficient resources: {}", msg),
            AgentAsKitError::ResourceAllocationFailed(msg) => write!(f, "Resource allocation failed: {}", msg),
            AgentAsKitError::ResourceLimitExceeded(msg) => write!(f, "Resource limit exceeded: {}", msg),
            
            AgentAsKitError::HealthCheckFailed(msg) => write!(f, "Health check failed: {}", msg),
            AgentAsKitError::MetricsCollectionFailed(msg) => write!(f, "Metrics collection failed: {}", msg),
            AgentAsKitError::AlertingSystemDown => write!(f, "Alerting system is down"),
            
            AgentAsKitError::DeploymentFailed(msg) => write!(f, "Deployment failed: {}", msg),
            AgentAsKitError::ManifestValidationFailed(msg) => write!(f, "Manifest validation failed: {}", msg),
            AgentAsKitError::ScalingFailed(msg) => write!(f, "Scaling failed: {}", msg),
            AgentAsKitError::RollbackFailed(msg) => write!(f, "Rollback failed: {}", msg),
            
            AgentAsKitError::CapabilityDenied(msg) => write!(f, "Capability denied: {}", msg),
            AgentAsKitError::InvalidCapabilityToken(msg) => write!(f, "Invalid capability token: {}", msg),
            AgentAsKitError::CapabilityExpired(id) => write!(f, "Capability expired: {}", id),
            
            AgentAsKitError::SandboxCreationFailed(msg) => write!(f, "Sandbox creation failed: {}", msg),
            AgentAsKitError::WasmExecutionFailed(msg) => write!(f, "WASM execution failed: {}", msg),
            AgentAsKitError::ContractValidationFailed(msg) => write!(f, "Contract validation failed: {}", msg),
            
            AgentAsKitError::ManifestParsingFailed(msg) => write!(f, "Manifest parsing failed: {}", msg),
            AgentAsKitError::AgentRegistrationFailed(msg) => write!(f, "Agent registration failed: {}", msg),
            AgentAsKitError::HealthMonitoringFailed(msg) => write!(f, "Health monitoring failed: {}", msg),
            
            AgentAsKitError::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            AgentAsKitError::DatabaseConnectionFailed(msg) => write!(f, "Database connection failed: {}", msg),
            AgentAsKitError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            AgentAsKitError::FileSystemError(msg) => write!(f, "File system error: {}", msg),
            
            AgentAsKitError::Internal(msg) => write!(f, "Internal error: {}", msg),
            AgentAsKitError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            AgentAsKitError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
        }
    }
}

impl std::error::Error for AgentAsKitError {}

/// Result type for AgentAsKit operations
pub type AgentAsKitResult<T> = Result<T, AgentAsKitError>;

/// Configuration structure for AgentAsKit systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAsKitConfig {
    // Core system configuration
    pub system_name: String,
    pub version: String,
    pub environment: Environment,
    
    // Agent configuration
    pub max_agents: u32,
    pub agent_timeout_seconds: u64,
    pub agent_heartbeat_interval_seconds: u64,
    
    // Task configuration
    pub max_concurrent_tasks: u32,
    pub task_timeout_seconds: u64,
    pub task_retry_limit: u32,
    
    // Communication configuration
    pub message_queue_size: u32,
    pub message_timeout_seconds: u64,
    pub broadcast_timeout_seconds: u64,
    
    // Health monitoring configuration
    pub health_check_interval_seconds: u64,
    pub health_check_timeout_seconds: u64,
    pub metrics_collection_interval_seconds: u64,
    
    // Resource limits
    pub memory_limit_mb: Option<u64>,
    pub cpu_limit_percent: Option<f64>,
    pub disk_limit_mb: Option<u64>,
    pub network_limit_mbps: Option<u32>,
    
    // FlexNetOS configuration
    pub flexnetos: FlexNetOSConfig,
    
    // NOA configuration
    pub noa: NoaConfig,
    
    // Security configuration
    pub security: SecurityConfig,
    
    // Logging configuration
    pub logging: LoggingConfig,
    
    // Custom configuration
    pub custom: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlexNetOSConfig {
    pub enable_wasm_host: bool,
    pub sandbox_timeout_seconds: u64,
    pub capability_token_lifetime_seconds: u64,
    pub contract_validation_enabled: bool,
    pub tri_sandbox_enabled: bool,
    pub numa_optimization_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoaConfig {
    pub enable_health_monitoring: bool,
    pub enable_auto_scaling: bool,
    pub enable_auto_repair: bool,
    pub deployment_timeout_seconds: u64,
    pub scaling_cooldown_seconds: u64,
    pub health_check_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_authentication: bool,
    pub enable_authorization: bool,
    pub enable_encryption: bool,
    pub token_lifetime_seconds: u64,
    pub allowed_capabilities: Vec<String>,
    pub audit_logging_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: LogLevel,
    pub format: LogFormat,
    pub output: LogOutput,
    pub max_file_size_mb: u64,
    pub max_files: u32,
    pub enable_metrics_logging: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogFormat {
    Json,
    Text,
    Structured,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogOutput {
    Console,
    File(String),
    Both(String),
}

/// Constants used across AgentAsKit systems
pub mod constants {
    use std::time::Duration;
    
    // Timeouts
    pub const DEFAULT_AGENT_TIMEOUT: Duration = Duration::from_secs(30);
    pub const DEFAULT_TASK_TIMEOUT: Duration = Duration::from_secs(300);
    pub const DEFAULT_MESSAGE_TIMEOUT: Duration = Duration::from_secs(10);
    pub const DEFAULT_HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(5);
    
    // Limits
    pub const MAX_AGENTS_DEFAULT: u32 = 1000;
    pub const MAX_CONCURRENT_TASKS_DEFAULT: u32 = 10000;
    pub const MAX_MESSAGE_SIZE_BYTES: u32 = 1024 * 1024; // 1MB
    pub const MAX_RETRY_ATTEMPTS: u32 = 3;
    
    // Health monitoring
    pub const HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);
    pub const METRICS_COLLECTION_INTERVAL: Duration = Duration::from_secs(60);
    pub const HEALTH_CHECK_FAILURE_THRESHOLD: u32 = 3;
    
    // FlexNetOS
    pub const WASM_MAX_MEMORY_PAGES: u32 = 1000; // ~64MB
    pub const SANDBOX_MAX_EXECUTION_TIME: Duration = Duration::from_secs(60);
    pub const CAPABILITY_TOKEN_DEFAULT_LIFETIME: Duration = Duration::from_secs(3600); // 1 hour
    
    // NOA
    pub const DEPLOYMENT_TIMEOUT: Duration = Duration::from_secs(600); // 10 minutes
    pub const SCALING_COOLDOWN: Duration = Duration::from_secs(300); // 5 minutes
    pub const AUTO_REPAIR_MAX_ATTEMPTS: u32 = 5;
    
    // System
    pub const SYSTEM_NAME: &str = "AgentAsKit";
    pub const API_VERSION: &str = "v1";
    pub const PROTOCOL_VERSION: &str = "1.0";
}

impl Default for AgentAsKitConfig {
    fn default() -> Self {
        Self {
            system_name: constants::SYSTEM_NAME.to_string(),
            version: "0.1.0".to_string(),
            environment: Environment::Development,
            max_agents: constants::MAX_AGENTS_DEFAULT,
            agent_timeout_seconds: constants::DEFAULT_AGENT_TIMEOUT.as_secs(),
            agent_heartbeat_interval_seconds: 30,
            max_concurrent_tasks: constants::MAX_CONCURRENT_TASKS_DEFAULT,
            task_timeout_seconds: constants::DEFAULT_TASK_TIMEOUT.as_secs(),
            task_retry_limit: constants::MAX_RETRY_ATTEMPTS,
            message_queue_size: 10000,
            message_timeout_seconds: constants::DEFAULT_MESSAGE_TIMEOUT.as_secs(),
            broadcast_timeout_seconds: 30,
            health_check_interval_seconds: constants::HEALTH_CHECK_INTERVAL.as_secs(),
            health_check_timeout_seconds: constants::DEFAULT_HEALTH_CHECK_TIMEOUT.as_secs(),
            metrics_collection_interval_seconds: constants::METRICS_COLLECTION_INTERVAL.as_secs(),
            memory_limit_mb: None,
            cpu_limit_percent: None,
            disk_limit_mb: None,
            network_limit_mbps: None,
            flexnetos: FlexNetOSConfig::default(),
            noa: NoaConfig::default(),
            security: SecurityConfig::default(),
            logging: LoggingConfig::default(),
            custom: HashMap::new(),
        }
    }
}

impl Default for FlexNetOSConfig {
    fn default() -> Self {
        Self {
            enable_wasm_host: true,
            sandbox_timeout_seconds: constants::SANDBOX_MAX_EXECUTION_TIME.as_secs(),
            capability_token_lifetime_seconds: constants::CAPABILITY_TOKEN_DEFAULT_LIFETIME.as_secs(),
            contract_validation_enabled: true,
            tri_sandbox_enabled: true,
            numa_optimization_enabled: false,
        }
    }
}

impl Default for NoaConfig {
    fn default() -> Self {
        Self {
            enable_health_monitoring: true,
            enable_auto_scaling: true,
            enable_auto_repair: true,
            deployment_timeout_seconds: constants::DEPLOYMENT_TIMEOUT.as_secs(),
            scaling_cooldown_seconds: constants::SCALING_COOLDOWN.as_secs(),
            health_check_threshold: constants::HEALTH_CHECK_FAILURE_THRESHOLD,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_authentication: true,
            enable_authorization: true,
            enable_encryption: false,
            token_lifetime_seconds: 3600,
            allowed_capabilities: vec![
                "read".to_string(),
                "write".to_string(),
                "execute".to_string(),
            ],
            audit_logging_enabled: true,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: LogLevel::Info,
            format: LogFormat::Json,
            output: LogOutput::Console,
            max_file_size_mb: 100,
            max_files: 10,
            enable_metrics_logging: true,
        }
    }
}