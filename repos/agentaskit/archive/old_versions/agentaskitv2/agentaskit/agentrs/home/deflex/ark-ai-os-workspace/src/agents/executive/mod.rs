pub mod noa_commander;
pub mod system_orchestrator;
pub mod resource_allocator;
pub mod priority_manager;
pub mod emergency_responder;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

pub use noa_commander::{NoaCommander, CommanderConfig};
pub use system_orchestrator::{SystemOrchestrator, OrchestratorConfig};
pub use resource_allocator::{ResourceAllocator, ResourceAllocatorConfig};
pub use priority_manager::{PriorityManager, PriorityManagerConfig};
pub use emergency_responder::{EmergencyResponder, EmergencyResponderConfig};

use crate::agents::{Agent, AgentContext, AgentId, AgentRegistry, communication::CommunicationManager};

/// Executive Agent Layer - High-level coordination and decision-making agents
/// 
/// The Executive Layer consists of 5 specialized agents that handle system-wide
/// coordination, resource management, and emergency response:
/// 
/// 1. NOA Commander - Chief Executive Agent for strategic decisions
/// 2. System Orchestrator - Workflow and task coordination
/// 3. Resource Allocator - Dynamic resource management  
/// 4. Priority Manager - Task prioritization and queuing
/// 5. Emergency Responder - System protection and recovery
pub struct ExecutiveLayer {
    /// All executive agents
    agents: Vec<Box<dyn Agent>>,
    
    /// Communication manager for inter-agent messaging
    communication_manager: Arc<CommunicationManager>,
    
    /// Agent registry
    registry: Arc<RwLock<AgentRegistry>>,
    
    /// Layer configuration
    config: ExecutiveLayerConfig,
}

/// Configuration for the Executive Layer
#[derive(Debug, Clone)]
pub struct ExecutiveLayerConfig {
    pub commander_config: CommanderConfig,
    pub orchestrator_config: OrchestratorConfig,
    pub resource_allocator_config: ResourceAllocatorConfig,
    pub priority_manager_config: PriorityManagerConfig,
    pub emergency_responder_config: EmergencyResponderConfig,
}

impl Default for ExecutiveLayerConfig {
    fn default() -> Self {
        Self {
            commander_config: CommanderConfig::default(),
            orchestrator_config: OrchestratorConfig::default(),
            resource_allocator_config: ResourceAllocatorConfig::default(),
            priority_manager_config: PriorityManagerConfig::default(),
            emergency_responder_config: EmergencyResponderConfig::default(),
        }
    }
}

impl ExecutiveLayer {
    pub fn new(
        communication_manager: Arc<CommunicationManager>,
        registry: Arc<RwLock<AgentRegistry>>,
        config: ExecutiveLayerConfig,
    ) -> Self {
        Self {
            agents: Vec::new(),
            communication_manager,
            registry,
            config,
        }
    }
    
    /// Initialize all executive agents
    pub async fn initialize(&mut self) -> Result<()> {
        tracing::info!("Initializing Executive Layer");
        
        // Create all executive agents
        let mut agents: Vec<Box<dyn Agent>> = vec![
            Box::new(NoaCommander::new(self.config.commander_config.clone())),
            Box::new(SystemOrchestrator::new(self.config.orchestrator_config.clone())),
            Box::new(ResourceAllocator::new(self.config.resource_allocator_config.clone())),
            Box::new(PriorityManager::new(self.config.priority_manager_config.clone())),
            Box::new(EmergencyResponder::new(self.config.emergency_responder_config.clone())),
        ];
        
        // Initialize each agent
        for agent in &mut agents {
            agent.initialize().await?;
            
            // Register with communication system
            let metadata = agent.metadata();
            self.registry.write().await.register_agent(metadata.clone()).await?;
            
            tracing::info!("Initialized executive agent: {}", metadata.name);
        }
        
        self.agents = agents;
        
        tracing::info!("Executive Layer initialized with {} agents", self.agents.len());
        Ok(())
    }
    
    /// Start all executive agents
    pub async fn start(&mut self) -> Result<()> {
        tracing::info!("Starting Executive Layer");
        
        for agent in &mut self.agents {
            agent.start().await?;
            tracing::info!("Started executive agent: {}", agent.metadata().name);
        }
        
        tracing::info!("Executive Layer started successfully");
        Ok(())
    }
    
    /// Stop all executive agents
    pub async fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping Executive Layer");
        
        for agent in &mut self.agents {
            agent.stop().await?;
            tracing::info!("Stopped executive agent: {}", agent.metadata().name);
        }
        
        tracing::info!("Executive Layer stopped successfully");
        Ok(())
    }
    
    /// Get health status of all executive agents
    pub async fn health_check(&self) -> Result<Vec<crate::agents::HealthStatus>> {
        let mut health_statuses = Vec::new();
        
        for agent in &self.agents {
            let health = agent.health_check().await?;
            health_statuses.push(health);
        }
        
        Ok(health_statuses)
    }
    
    /// Get executive layer statistics
    pub async fn get_statistics(&self) -> Result<ExecutiveLayerStatistics> {
        let health_statuses = self.health_check().await?;
        
        let total_agents = self.agents.len();
        let active_agents = health_statuses.iter()
            .filter(|h| matches!(h.state, crate::agents::AgentState::Active))
            .count();
        
        let avg_cpu_usage = health_statuses.iter()
            .map(|h| h.cpu_usage)
            .sum::<f32>() / health_statuses.len() as f32;
        
        let total_memory_usage = health_statuses.iter()
            .map(|h| h.memory_usage)
            .sum::<u64>();
        
        let total_completed_tasks = health_statuses.iter()
            .map(|h| h.completed_tasks)
            .sum::<u64>();
        
        let total_failed_tasks = health_statuses.iter()
            .map(|h| h.failed_tasks)
            .sum::<u64>();
        
        Ok(ExecutiveLayerStatistics {
            total_agents,
            active_agents,
            avg_cpu_usage,
            total_memory_usage,
            total_completed_tasks,
            total_failed_tasks,
            success_rate: if total_completed_tasks + total_failed_tasks > 0 {
                total_completed_tasks as f64 / (total_completed_tasks + total_failed_tasks) as f64
            } else {
                1.0
            },
        })
    }
    
    /// Get specific executive agent by role
    pub fn get_agent_by_name(&self, name: &str) -> Option<&dyn Agent> {
        self.agents.iter()
            .find(|agent| agent.metadata().name == name)
            .map(|agent| agent.as_ref())
    }
    
    /// Get NOA Commander specifically
    pub fn get_noa_commander(&self) -> Option<&NoaCommander> {
        self.agents.iter()
            .find(|agent| agent.metadata().name == "NOA Commander")
            .and_then(|agent| {
                // Note: This is a simplified cast - in a real implementation,
                // you'd want a more robust way to downcast to specific agent types
                unsafe { 
                    let ptr = agent.as_ref() as *const dyn Agent;
                    let commander_ptr = ptr as *const NoaCommander;
                    Some(&*commander_ptr)
                }
            })
    }
}

/// Statistics for the Executive Layer
#[derive(Debug, Clone)]
pub struct ExecutiveLayerStatistics {
    pub total_agents: usize,
    pub active_agents: usize,
    pub avg_cpu_usage: f32,
    pub total_memory_usage: u64,
    pub total_completed_tasks: u64,
    pub total_failed_tasks: u64,
    pub success_rate: f64,
}

/// Executive layer coordination utilities
pub mod coordination {
    use super::*;
    use crate::agents::{AgentMessage, Task, MessageId, Priority};
    use uuid::Uuid;
    use std::time::Duration;
    
    /// Coordinate a strategic decision across multiple executive agents
    pub async fn coordinate_strategic_decision(
        communication_manager: &CommunicationManager,
        decision_task: Task,
        timeout: Duration,
    ) -> Result<serde_json::Value> {
        tracing::info!("Coordinating strategic decision: {}", decision_task.name);
        
        // Send task to NOA Commander for strategic decision-making
        let commander_id = AgentId::from_name("noa-commander");
        let message_id = MessageId::new();
        
        let request = AgentMessage::Request {
            id: message_id,
            from: AgentId::from_name("executive-layer-coordinator"),
            to: commander_id,
            task: decision_task,
            priority: Priority::High,
            timeout: Some(timeout),
        };
        
        communication_manager.send_message(request).await?;
        
        // TODO: Wait for response and handle coordination
        // This would involve setting up a response handler and timeout
        
        Ok(serde_json::json!({
            "status": "coordinated",
            "decision_initiated": true,
            "coordinator": "noa-commander",
        }))
    }
    
    /// Broadcast emergency alert to all executive agents
    pub async fn broadcast_emergency_alert(
        communication_manager: &CommunicationManager,
        alert_message: String,
        context: serde_json::Value,
    ) -> Result<()> {
        tracing::error!("Broadcasting emergency alert: {}", alert_message);
        
        let alert = AgentMessage::Alert {
            id: MessageId::new(),
            from: AgentId::from_name("executive-layer-coordinator"),
            severity: crate::agents::AlertSeverity::Emergency,
            message: alert_message,
            context,
            timestamp: std::time::Instant::now(),
        };
        
        // Broadcast to all executive agents
        communication_manager.send_message(AgentMessage::Broadcast {
            id: MessageId::new(),
            from: AgentId::from_name("executive-layer-coordinator"),
            topic: "emergency-alert".to_string(),
            payload: serde_json::to_value(alert)?,
            scope: crate::agents::BroadcastScope::Role(crate::agents::AgentRole::Executive),
        }).await?;
        
        Ok(())
    }
    
    /// Coordinate resource reallocation across the system
    pub async fn coordinate_resource_reallocation(
        communication_manager: &CommunicationManager,
        reallocation_request: serde_json::Value,
    ) -> Result<serde_json::Value> {
        tracing::info!("Coordinating resource reallocation");
        
        // Create task for resource allocator
        let task = Task {
            id: Uuid::new_v4(),
            name: "resource-reallocation".to_string(),
            description: "Coordinate system-wide resource reallocation".to_string(),
            parameters: reallocation_request,
            required_capabilities: vec!["resource-allocation".to_string()],
            deadline: Some(std::time::Instant::now() + Duration::from_secs(300)),
            dependencies: Vec::new(),
        };
        
        let resource_allocator_id = AgentId::from_name("resource-allocator");
        let message = AgentMessage::Request {
            id: MessageId::new(),
            from: AgentId::from_name("executive-layer-coordinator"),
            to: resource_allocator_id,
            task,
            priority: Priority::High,
            timeout: Some(Duration::from_secs(300)),
        };
        
        communication_manager.send_message(message).await?;
        
        Ok(serde_json::json!({
            "status": "reallocation_initiated",
            "coordinator": "resource-allocator",
            "estimated_completion": 300,
        }))
    }
}
