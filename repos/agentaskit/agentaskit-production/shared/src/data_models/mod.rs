use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Common agent identifier type used across all AgentAsKit systems
pub type AgentId = Uuid;

/// Common task identifier type
pub type TaskId = Uuid;

/// Universal agent status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    Inactive,
    Initializing,
    Active,
    Busy,
    Error,
    Maintenance,
    Shutdown,
}

/// Universal task status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

/// Priority levels used across all systems
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
    Emergency = 5,
}

/// Health status for NOA monitoring integration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    NeedsRepair,
    Critical,
    Unknown,
}

/// Common agent metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub id: AgentId,
    pub name: String,
    pub agent_type: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub status: AgentStatus,
    pub health_status: HealthStatus,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub resource_requirements: ResourceRequirements,
    pub tags: HashMap<String, String>,
}

/// Resource requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: Option<u32>,
    pub memory_mb: Option<u64>,
    pub storage_mb: Option<u64>,
    pub network_bandwidth_mbps: Option<u32>,
    pub gpu_required: bool,
    pub special_capabilities: Vec<String>,
}

/// Common task structure used across FlexNetOS and NOA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub priority: Priority,
    pub status: TaskStatus,
    pub assigned_agent: Option<AgentId>,
    pub dependencies: Vec<TaskId>,
    pub input_data: serde_json::Value,
    pub output_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub timeout: Option<DateTime<Utc>>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub error_message: Option<String>,
    pub tags: HashMap<String, String>,
}

/// Capability token structure for FlexNetOS integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub token_id: Uuid,
    pub capability: String,
    pub granted_to: AgentId,
    pub granted_by: AgentId,
    pub valid_until: DateTime<Utc>,
    pub permissions: Vec<String>,
    pub restrictions: HashMap<String, serde_json::Value>,
    pub signature: String,
}

/// NOA deployment manifest entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentManifestEntry {
    pub agent_id: AgentId,
    pub agent_name: String,
    pub agent_type: String,
    pub deployment_config: serde_json::Value,
    pub health_checks: Vec<HealthCheck>,
    pub scaling_policy: ScalingPolicy,
    pub dependencies: Vec<AgentId>,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub check_type: String,
    pub interval_seconds: u64,
    pub timeout_seconds: u64,
    pub threshold: u32,
    pub endpoint: Option<String>,
    pub command: Option<String>,
}

/// Scaling policy for NOA deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingPolicy {
    pub min_instances: u32,
    pub max_instances: u32,
    pub target_cpu_percent: Option<f64>,
    pub target_memory_percent: Option<f64>,
    pub scale_up_cooldown_seconds: u64,
    pub scale_down_cooldown_seconds: u64,
}

/// Message structure for inter-agent communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub message_id: Uuid,
    pub from_agent: AgentId,
    pub to_agent: AgentId,
    pub message_type: String,
    pub priority: Priority,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub correlation_id: Option<Uuid>,
    pub reply_to: Option<AgentId>,
    pub ttl: Option<DateTime<Utc>>,
}

/// System metrics for monitoring and health assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub agent_id: AgentId,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: u64,
    pub memory_usage_percent: f64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub disk_usage_mb: u64,
    pub disk_usage_percent: f64,
    pub task_queue_size: u32,
    pub active_tasks: u32,
    pub completed_tasks_last_hour: u32,
    pub error_count_last_hour: u32,
    pub response_time_ms: f64,
    pub custom_metrics: HashMap<String, f64>,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Normal
    }
}

impl Default for AgentStatus {
    fn default() -> Self {
        AgentStatus::Inactive
    }
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

impl Default for HealthStatus {
    fn default() -> Self {
        HealthStatus::Unknown
    }
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_cores: None,
            memory_mb: None,
            storage_mb: None,
            network_bandwidth_mbps: None,
            gpu_required: false,
            special_capabilities: Vec::new(),
        }
    }
}