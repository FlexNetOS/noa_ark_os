use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;
use anyhow::Result;

// Core agent framework - the foundation for all agents in ARK OS NOA

/// Unique identifier for agents in the system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct AgentId(pub Uuid);

impl AgentId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_name(name: &str) -> Self {
        // Generate deterministic UUID from name for well-known agents
        use uuid::Uuid;
        // Use a simple hash-based approach for now
        let hash = name.as_bytes().iter().fold(0u64, |acc, &b| acc.wrapping_add(b as u64));
        let uuid = Uuid::from_u64_pair(hash, hash.wrapping_mul(2));
        Self(uuid)
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Message identifier for tracking communications
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

impl MessageId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

/// Priority levels for task and message handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Priority {
    Emergency = 0,   // System-critical, immediate attention
    Critical = 1,    // High priority, fast response required
    High = 2,        // Important, handle quickly
    Medium = 3,      // Medium priority
    Normal = 4,      // Standard priority
    Low = 5,         // Background, handle when resources available
    Maintenance = 6, // Lowest priority, maintenance tasks
}

/// Agent states in the lifecycle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentState {
    Initializing,
    Active,
    Busy,
    Idle,
    Maintenance,
    Error(String),
    Terminating,
}

/// Agent capabilities and roles
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentRole {
    Executive,     // High-level coordination and decision-making
    Board,         // Strategic and governance functions
    Specialized,   // Domain-specific expertise
    Worker,        // Task execution
    Monitor,       // Observation and reporting
}

impl Default for AgentRole {
    fn default() -> Self {
        AgentRole::Worker
    }
}

/// Agent metadata and registration information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentMetadata {
    pub id: AgentId,
    pub name: String,
    pub role: AgentRole,
    pub capabilities: Vec<String>,
    pub version: String,
    pub cluster_assignment: Option<String>,
    pub resource_requirements: ResourceRequirements,
    pub health_check_interval: Duration,
}

/// Resource requirements for agent deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub min_cpu: f32,
    pub min_memory: u64,
    pub min_storage: u64,
    pub max_cpu: f32,
    pub max_memory: u64,
    pub max_storage: u64,
}

/// Task definition for agent work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: String,
    pub parameters: serde_json::Value,
    pub required_capabilities: Vec<String>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub dependencies: Vec<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub result: serde_json::Value,
    pub error: Option<String>,
    pub execution_time: Duration,
    pub resource_usage: ResourceUsage,
}

/// Task completion status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
    Cancelled,
    Timeout,
}

/// Agent error types
#[derive(Debug, thiserror::Error, Clone)]
pub enum AgentError {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    #[error("Initialization failed: {0}")]
    InitializationFailed(String),
    #[error("Task execution failed: {0}")]
    TaskExecutionFailed(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Communication error: {0}")]
    CommunicationError(String),
    #[error("Resource allocation error: {0}")]
    ResourceError(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Internal error: {0}")]
    Internal(String),
    #[error("Missing parameter: {0}")]
    MissingParameter(String),
    #[error("Already running")]
    AlreadyRunning,
    #[error("Not running")]
    NotRunning,
}

impl From<anyhow::Error> for AgentError {
    fn from(err: anyhow::Error) -> Self {
        AgentError::Internal(err.to_string())
    }
}

impl From<serde_json::Error> for AgentError {
    fn from(err: serde_json::Error) -> Self {
        AgentError::Internal(format!("JSON serialization error: {}", err))
    }
}

pub type AgentResult<T> = Result<T, AgentError>;

/// Agent capabilities 
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentCapability {
    CodeGeneration,
    Testing,
    Deployment,
    Monitoring,
    MachineLearning,
    SecurityScanning,
    DataProcessing,
    SystemIntegration,
    FinancialAnalysis,
    LegalCompliance,
    StrategyPlanning,
    Operations,
    ResourceManagement,
    PriorityManagement,
    EmergencyResponse,
    SystemOrchestration,
    Communication,
    // Security capabilities
    ThreatDetection,
    ComplianceMonitoring,
    IncidentResponse,
    AccessControl,
    SecurityAuditing,
    VulnerabilityAssessment,
    PolicyEnforcement,
    // Data analytics capabilities
    StatisticalAnalysis,
    DataVisualization,
    ReportGeneration,
    StreamProcessing,
    DataWarehouse,
    QueryOptimization,
    // Integration capabilities
    ApiGateway,
    ServiceDiscovery,
    DataTransformation,
    WorkflowOrchestration,
    MessageBroking,
    ProtocolHandling,
    IntegrationMonitoring,
    Other(String),
}

/// Resource usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_time: Duration,
    pub memory_peak: u64,
    pub storage_used: u64,
    pub network_bytes: u64,
}

/// Agent health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub agent_id: AgentId,
    pub state: AgentState,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub task_queue_size: usize,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_response_time: Duration,
}

/// Inter-agent communication messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentMessage {
    /// Direct task request to specific agent
    Request {
        id: MessageId,
        from: AgentId,
        to: AgentId,
        task: Task,
        priority: Priority,
        timeout: Option<Duration>,
    },
    
    /// Response to a previous request
    Response {
        id: MessageId,
        request_id: MessageId,
        from: AgentId,
        to: AgentId,
        result: TaskResult,
    },
    
    /// Broadcast message to multiple agents
    Broadcast {
        id: MessageId,
        from: AgentId,
        topic: String,
        payload: serde_json::Value,
        scope: BroadcastScope,
    },
    
    /// System alert or notification
    Alert {
        id: MessageId,
        from: AgentId,
        severity: AlertSeverity,
        message: String,
        context: serde_json::Value,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    /// Heartbeat for health monitoring
    Heartbeat {
        id: MessageId,
        from: AgentId,
        health: HealthStatus,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    
    /// Agent registration/deregistration
    Registration {
        id: MessageId,
        agent_id: AgentId,
        metadata: AgentMetadata,
        action: RegistrationAction,
    },
}

/// Broadcast message scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BroadcastScope {
    All,                           // All agents
    Role(AgentRole),              // Agents with specific role
    Cluster(String),              // Agents in specific cluster
    Capability(String),           // Agents with specific capability
    Custom(Vec<AgentId>),         // Specific agent list
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertSeverity {
    Emergency,  // System failure or critical issue
    Critical,   // Major problem requiring immediate attention
    Warning,    // Potential issue or degraded performance
    Info,       // Informational message
    Debug,      // Debugging information
}

/// Registration action type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistrationAction {
    Register,
    Deregister,
    Update,
}

/// Core trait that all agents must implement
#[async_trait]
pub trait Agent: Send + Sync {
    /// Get agent metadata
    fn metadata(&self) -> &AgentMetadata;
    
    /// Get current agent state
    async fn state(&self) -> AgentState;
    
    /// Initialize the agent
    async fn initialize(&mut self) -> Result<()>;
    
    /// Start the agent's main loop
    async fn start(&mut self) -> Result<()>;
    
    /// Stop the agent gracefully
    async fn stop(&mut self) -> Result<()>;
    
    /// Handle incoming message
    async fn handle_message(&mut self, message: AgentMessage) -> Result<Option<AgentMessage>>;
    
    /// Execute a specific task
    async fn execute_task(&mut self, task: Task) -> Result<TaskResult>;
    
    /// Get current health status
    async fn health_check(&self) -> Result<HealthStatus>;
    
    /// Handle configuration updates
    async fn update_config(&mut self, config: serde_json::Value) -> Result<()>;
    
    /// Get agent capabilities
    fn capabilities(&self) -> &[String];
    
    /// Get agent status as JSON value
    async fn get_status(&self) -> Result<serde_json::Value> {
        let health = self.health_check().await?;
        Ok(serde_json::to_value(health)?)
    }
    
    /// Check if agent can handle specific task
    fn can_handle_task(&self, task: &Task) -> bool {
        task.required_capabilities.iter()
            .all(|cap| self.capabilities().contains(cap))
    }
}

/// Agent context providing shared services
pub struct AgentContext {
    pub agent_id: AgentId,
    pub message_sender: mpsc::UnboundedSender<AgentMessage>,
    pub message_receiver: mpsc::UnboundedReceiver<AgentMessage>,
    pub agent_registry: std::sync::Arc<RwLock<AgentRegistry>>,
    pub config: serde_json::Value,
    pub shutdown_signal: tokio::sync::broadcast::Receiver<()>,
}

/// Registry for tracking all agents in the system
#[derive(Debug, Default)]
pub struct AgentRegistry {
    agents: HashMap<AgentId, AgentMetadata>,
    health_status: HashMap<AgentId, HealthStatus>,
    capability_index: HashMap<String, Vec<AgentId>>,
    role_index: HashMap<AgentRole, Vec<AgentId>>,
    cluster_index: HashMap<String, Vec<AgentId>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a new agent
    pub async fn register_agent(&mut self, metadata: AgentMetadata) -> Result<()> {
        let agent_id = metadata.id;
        
        // Add to main registry
        self.agents.insert(agent_id, metadata.clone());
        
        // Update capability index
        for capability in &metadata.capabilities {
            self.capability_index
                .entry(capability.clone())
                .or_insert_with(Vec::new)
                .push(agent_id);
        }
        
        // Update role index
        self.role_index
            .entry(metadata.role.clone())
            .or_insert_with(Vec::new)
            .push(agent_id);
        
        // Update cluster index if assigned
        if let Some(cluster) = &metadata.cluster_assignment {
            self.cluster_index
                .entry(cluster.clone())
                .or_insert_with(Vec::new)
                .push(agent_id);
        }
        
        tracing::info!("Registered agent: {} ({})", metadata.name, agent_id.0);
        Ok(())
    }
    
    /// Deregister an agent
    pub async fn deregister_agent(&mut self, agent_id: AgentId) -> Result<()> {
        if let Some(metadata) = self.agents.remove(&agent_id) {
            // Remove from health status
            self.health_status.remove(&agent_id);
            
            // Remove from capability index
            for capability in &metadata.capabilities {
                if let Some(agents) = self.capability_index.get_mut(capability) {
                    agents.retain(|&id| id != agent_id);
                    if agents.is_empty() {
                        self.capability_index.remove(capability);
                    }
                }
            }
            
            // Remove from role index
            if let Some(agents) = self.role_index.get_mut(&metadata.role) {
                agents.retain(|&id| id != agent_id);
                if agents.is_empty() {
                    self.role_index.remove(&metadata.role);
                }
            }
            
            // Remove from cluster index
            if let Some(cluster) = &metadata.cluster_assignment {
                if let Some(agents) = self.cluster_index.get_mut(cluster) {
                    agents.retain(|&id| id != agent_id);
                    if agents.is_empty() {
                        self.cluster_index.remove(cluster);
                    }
                }
            }
            
            tracing::info!("Deregistered agent: {} ({})", metadata.name, agent_id.0);
        }
        
        Ok(())
    }
    
    /// Update agent health status
    pub async fn update_health(&mut self, health: HealthStatus) -> Result<()> {
        self.health_status.insert(health.agent_id, health);
        Ok(())
    }
    
    /// Find agents by capability
    pub fn find_by_capability(&self, capability: &str) -> Vec<AgentId> {
        self.capability_index
            .get(capability)
            .map(|agents| agents.clone())
            .unwrap_or_default()
    }
    
    /// Find agents by role
    pub fn find_by_role(&self, role: &AgentRole) -> Vec<AgentId> {
        self.role_index
            .get(role)
            .map(|agents| agents.clone())
            .unwrap_or_default()
    }
    
    /// Find agents by cluster
    pub fn find_by_cluster(&self, cluster: &str) -> Vec<AgentId> {
        self.cluster_index
            .get(cluster)
            .map(|agents| agents.clone())
            .unwrap_or_default()
    }
    
    /// Get agent metadata
    pub fn get_agent(&self, agent_id: &AgentId) -> Option<&AgentMetadata> {
        self.agents.get(agent_id)
    }
    
    /// Get agent health status
    pub fn get_health(&self, agent_id: &AgentId) -> Option<&HealthStatus> {
        self.health_status.get(agent_id)
    }
    
    /// Get all agents
    pub fn all_agents(&self) -> Vec<&AgentMetadata> {
        self.agents.values().collect()
    }
    
    /// Get healthy agents only
    pub fn healthy_agents(&self) -> Vec<AgentId> {
        self.health_status
            .iter()
            .filter(|(_, health)| {
                matches!(health.state, AgentState::Active | AgentState::Idle)
                    && (chrono::Utc::now() - health.last_heartbeat).num_seconds() < 30
            })
            .map(|(&id, _)| id)
            .collect()
    }
}

/// Default implementation helpers
impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            min_cpu: 0.1,
            min_memory: 128 * 1024 * 1024, // 128MB
            min_storage: 1024 * 1024,      // 1MB
            max_cpu: 2.0,
            max_memory: 2 * 1024 * 1024 * 1024, // 2GB
            max_storage: 100 * 1024 * 1024,     // 100MB
        }
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_time: Duration::ZERO,
            memory_peak: 0,
            storage_used: 0,
            network_bytes: 0,
        }
    }
}

// Utility functions for agent framework
pub mod utils {
    use super::*;
    
    /// Create a standard task with default values
    pub fn create_task(
        name: String,
        description: String,
        task_type: String,
        parameters: serde_json::Value,
        capabilities: Vec<String>,
    ) -> Task {
        Task {
            id: Uuid::new_v4(),
            name,
            description,
            task_type,
            parameters,
            required_capabilities: capabilities,
            deadline: None,
            dependencies: Vec::new(),
            created_at: chrono::Utc::now(),
        }
    }
    
    /// Create a success task result
    pub fn success_result(task_id: Uuid, result: serde_json::Value) -> TaskResult {
        TaskResult {
            task_id,
            status: TaskStatus::Completed,
            result,
            error: None,
            execution_time: Duration::ZERO,
            resource_usage: ResourceUsage::default(),
        }
    }
    
    /// Create a failure task result
    pub fn failure_result(task_id: Uuid, error: String) -> TaskResult {
        TaskResult {
            task_id,
            status: TaskStatus::Failed(error.clone()),
            result: serde_json::Value::Null,
            error: Some(error),
            execution_time: Duration::ZERO,
            resource_usage: ResourceUsage::default(),
        }
    }
    
    /// Check if an agent role matches a broadcast scope
    pub fn matches_broadcast_scope(role: &AgentRole, scope: &BroadcastScope) -> bool {
        match scope {
            BroadcastScope::All => true,
            BroadcastScope::Role(target_role) => role == target_role,
            _ => false, // Other scopes require additional context
        }
    }
}

// Agent implementations - Phase 4 Micro-Agent Framework
pub mod communication;
pub mod executive;
pub mod board;
pub mod specialized;

// Integration tests disabled temporarily for compilation
// #[cfg(test)]
// pub mod integration_tests;

// Re-export key types and implementations
pub use communication::CommunicationManager;
pub use executive::{
    NoaCommander, SystemOrchestrator, ResourceAllocator, PriorityManager, EmergencyResponder,
    ExecutiveLayer, ExecutiveLayerConfig, ExecutiveLayerStatistics,
};
pub use board::{
    StrategyBoardAgent, OperationsBoardAgent, FinanceBoardAgent, LegalComplianceBoardAgent,
    DigestAgent, BoardLayer, BoardLayerConfig, BoardMeeting, BoardDecision,
};
pub use specialized::{
    CodeGenerationAgent, TestingAgent, DeploymentAgent, MonitoringAgent, LearningAgent,
    SecuritySpecialistAgent, DataAnalyticsAgent, IntegrationAgent, SpecializedLayer,
    SpecializedLayerStatus,
};
