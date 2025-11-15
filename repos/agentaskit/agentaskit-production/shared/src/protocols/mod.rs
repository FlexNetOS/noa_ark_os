use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;

use crate::data_models::*;

/// Universal communication protocol for AgentAsKit systems
#[async_trait]
pub trait AgentCommunicationProtocol {
    /// Send a message to another agent
    async fn send_message(&self, message: AgentMessage) -> Result<()>;
    
    /// Receive messages for this agent
    async fn receive_messages(&self) -> Result<Vec<AgentMessage>>;
    
    /// Broadcast a message to multiple agents
    async fn broadcast_message(&self, message: AgentMessage, targets: Vec<AgentId>) -> Result<()>;
    
    /// Subscribe to messages of specific types
    async fn subscribe(&self, message_types: Vec<String>) -> Result<()>;
    
    /// Unsubscribe from message types
    async fn unsubscribe(&self, message_types: Vec<String>) -> Result<()>;
}

/// Health monitoring protocol for NOA integration
#[async_trait]
pub trait HealthMonitoringProtocol {
    /// Report current health status
    async fn report_health(&self, agent_id: AgentId, health_status: HealthStatus) -> Result<()>;
    
    /// Get health status of an agent
    async fn get_health_status(&self, agent_id: AgentId) -> Result<HealthStatus>;
    
    /// Get system-wide health metrics
    async fn get_system_health(&self) -> Result<HashMap<AgentId, HealthStatus>>;
    
    /// Register health check callback
    async fn register_health_check(&self, agent_id: AgentId, check: HealthCheck) -> Result<()>;
    
    /// Execute health checks for an agent
    async fn execute_health_checks(&self, agent_id: AgentId) -> Result<Vec<HealthCheckResult>>;
}

/// Task orchestration protocol for FlexNetOS integration
#[async_trait]
pub trait TaskOrchestrationProtocol {
    /// Submit a new task for execution
    async fn submit_task(&self, task: Task) -> Result<TaskId>;
    
    /// Get task status
    async fn get_task_status(&self, task_id: TaskId) -> Result<TaskStatus>;
    
    /// Get task details
    async fn get_task(&self, task_id: TaskId) -> Result<Task>;
    
    /// Cancel a task
    async fn cancel_task(&self, task_id: TaskId) -> Result<()>;
    
    /// Assign task to agent
    async fn assign_task(&self, task_id: TaskId, agent_id: AgentId) -> Result<()>;
    
    /// Report task completion
    async fn complete_task(&self, task_id: TaskId, result: serde_json::Value) -> Result<()>;
    
    /// Report task failure
    async fn fail_task(&self, task_id: TaskId, error: String) -> Result<()>;
}

/// Agent lifecycle management protocol
#[async_trait]
pub trait AgentLifecycleProtocol {
    /// Initialize an agent
    async fn initialize_agent(&self, metadata: AgentMetadata) -> Result<AgentId>;
    
    /// Start an agent
    async fn start_agent(&self, agent_id: AgentId) -> Result<()>;
    
    /// Stop an agent gracefully
    async fn stop_agent(&self, agent_id: AgentId) -> Result<()>;
    
    /// Force shutdown an agent
    async fn force_shutdown_agent(&self, agent_id: AgentId) -> Result<()>;
    
    /// Update agent metadata
    async fn update_agent_metadata(&self, agent_id: AgentId, metadata: AgentMetadata) -> Result<()>;
    
    /// Get agent metadata
    async fn get_agent_metadata(&self, agent_id: AgentId) -> Result<AgentMetadata>;
    
    /// List all agents with optional filter
    async fn list_agents(&self, filter: Option<AgentFilter>) -> Result<Vec<AgentMetadata>>;
}

/// Capability management protocol for FlexNetOS
#[async_trait]
pub trait CapabilityManagementProtocol {
    /// Grant capability to an agent
    async fn grant_capability(&self, token: CapabilityToken) -> Result<()>;
    
    /// Revoke capability from an agent
    async fn revoke_capability(&self, token_id: uuid::Uuid) -> Result<()>;
    
    /// Check if agent has capability
    async fn has_capability(&self, agent_id: AgentId, capability: String) -> Result<bool>;
    
    /// List agent capabilities
    async fn list_agent_capabilities(&self, agent_id: AgentId) -> Result<Vec<CapabilityToken>>;
    
    /// Validate capability token
    async fn validate_capability_token(&self, token: CapabilityToken) -> Result<bool>;
}

/// Deployment management protocol for NOA
#[async_trait]
pub trait DeploymentManagementProtocol {
    /// Deploy agent from manifest
    async fn deploy_agent(&self, manifest_entry: DeploymentManifestEntry) -> Result<AgentId>;
    
    /// Undeploy agent
    async fn undeploy_agent(&self, agent_id: AgentId) -> Result<()>;
    
    /// Scale agent instances
    async fn scale_agent(&self, agent_id: AgentId, target_instances: u32) -> Result<()>;
    
    /// Update agent deployment
    async fn update_deployment(&self, agent_id: AgentId, manifest_entry: DeploymentManifestEntry) -> Result<()>;
    
    /// Get deployment status
    async fn get_deployment_status(&self, agent_id: AgentId) -> Result<DeploymentStatus>;
    
    /// List all deployments
    async fn list_deployments(&self) -> Result<Vec<DeploymentStatus>>;
}

/// Metrics collection protocol
#[async_trait]
pub trait MetricsCollectionProtocol {
    /// Report metrics for an agent
    async fn report_metrics(&self, metrics: SystemMetrics) -> Result<()>;
    
    /// Get metrics for an agent
    async fn get_agent_metrics(&self, agent_id: AgentId, time_range: TimeRange) -> Result<Vec<SystemMetrics>>;
    
    /// Get system-wide metrics
    async fn get_system_metrics(&self, time_range: TimeRange) -> Result<HashMap<AgentId, Vec<SystemMetrics>>>;
    
    /// Subscribe to metrics updates
    async fn subscribe_to_metrics(&self, agent_ids: Vec<AgentId>) -> Result<()>;
    
    /// Get performance dashboard data
    async fn get_dashboard_data(&self) -> Result<DashboardData>;
}

/// Supporting structures for protocols

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub check_type: String,
    pub status: HealthStatus,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message: Option<String>,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFilter {
    pub agent_type: Option<String>,
    pub status: Option<AgentStatus>,
    pub health_status: Option<HealthStatus>,
    pub tags: Option<HashMap<String, String>>,
    pub capabilities: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentStatus {
    pub agent_id: AgentId,
    pub deployment_id: uuid::Uuid,
    pub status: String,
    pub instances: u32,
    pub target_instances: u32,
    pub healthy_instances: u32,
    pub last_updated: chrono::DateTime<chrono::Utc>,
    pub deployment_config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: chrono::DateTime<chrono::Utc>,
    pub end: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub total_agents: u32,
    pub active_agents: u32,
    pub healthy_agents: u32,
    pub total_tasks: u32,
    pub pending_tasks: u32,
    pub running_tasks: u32,
    pub completed_tasks: u32,
    pub failed_tasks: u32,
    pub system_load: f64,
    pub memory_usage: f64,
    pub network_throughput: f64,
    pub error_rate: f64,
    pub response_time_avg: f64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Protocol message types for standard communication patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StandardMessageType {
    // Lifecycle messages
    AgentStartup,
    AgentShutdown,
    AgentHeartbeat,
    
    // Task messages
    TaskAssignment,
    TaskCompletion,
    TaskFailure,
    TaskProgress,
    
    // Health messages
    HealthCheck,
    HealthAlert,
    HealthRecovery,
    
    // Capability messages
    CapabilityRequest,
    CapabilityGrant,
    CapabilityRevoke,
    
    // Deployment messages
    DeploymentStart,
    DeploymentComplete,
    DeploymentError,
    ScaleRequest,
    
    // Metrics messages
    MetricsReport,
    PerformanceAlert,
    ResourceExhaustion,
    
    // Custom message types
    Custom(String),
}

impl std::fmt::Display for StandardMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StandardMessageType::Custom(custom) => write!(f, "{}", custom),
            other => write!(f, "{:?}", other),
        }
    }
}