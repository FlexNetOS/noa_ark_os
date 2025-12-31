// Specialized Layer - Phase 4 Micro-Agent Framework
// Domain expert agents providing operational capabilities with autonomous execution

pub mod code_generation_agent;
pub mod testing_agent;
pub mod deployment_agent;
pub mod monitoring_agent;
pub mod learning_agent;
pub mod security_specialist_agent;
pub mod data_analytics_agent;
pub mod integration_agent;

// Re-export all specialized agents for easy access
pub use code_generation_agent::CodeGenerationAgent;
pub use testing_agent::TestingAgent;
pub use deployment_agent::DeploymentAgent;
pub use monitoring_agent::MonitoringAgent;
pub use learning_agent::LearningAgent;
pub use security_specialist_agent::SecuritySpecialistAgent;
pub use data_analytics_agent::DataAnalyticsAgent;
pub use integration_agent::IntegrationAgent;

use crate::agents::{Agent, AgentError, AgentResult, Task, AgentId, AgentMessage};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};
use uuid::Uuid;

/// Specialized Layer Coordinator
/// Manages the collection of domain expert agents and their interactions
pub struct SpecializedLayer {
    agents: Arc<RwLock<HashMap<Uuid, Box<dyn Agent>>>>,
    agent_registry: Arc<RwLock<HashMap<String, Uuid>>>,
}

impl SpecializedLayer {
    /// Create new Specialized Layer with all domain expert agents
    pub async fn new() -> AgentResult<Self> {
        let layer = Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            agent_registry: Arc::new(RwLock::new(HashMap::new())),
        };

        // Initialize all specialized agents
        layer.initialize_agents().await?;

        Ok(layer)
    }

    /// Initialize all specialized agents with their default configurations
    async fn initialize_agents(&self) -> AgentResult<()> {
        info!("Initializing Specialized Layer with all domain expert agents");

        let mut agents = self.agents.write().await;
        let mut registry = self.agent_registry.write().await;

        // Code Generation Agent
        let code_gen_agent = Box::new(CodeGenerationAgent::new(None));
        let code_gen_id = code_gen_agent.metadata().id;
        registry.insert("code_generation".to_string(), code_gen_id.0);
        agents.insert(code_gen_id.0, code_gen_agent);

        // Testing Agent
        let testing_agent = Box::new(TestingAgent::new(None));
        let testing_id = testing_agent.metadata().id;
        registry.insert("testing".to_string(), testing_id.0);
        agents.insert(testing_id.0, testing_agent);

        // Deployment Agent
        let deployment_agent = Box::new(DeploymentAgent::new(None));
        let deployment_id = deployment_agent.metadata().id;
        registry.insert("deployment".to_string(), deployment_id.0);
        agents.insert(deployment_id.0, deployment_agent);

        // Monitoring Agent
        let monitoring_agent = Box::new(MonitoringAgent::new(None));
        let monitoring_id = monitoring_agent.metadata().id;
        registry.insert("monitoring".to_string(), monitoring_id.0);
        agents.insert(monitoring_id.0, monitoring_agent);

        // Learning Agent
        let learning_agent = Box::new(LearningAgent::new(None));
        let learning_id = learning_agent.metadata().id;
        registry.insert("learning".to_string(), learning_id.0);
        agents.insert(learning_id.0, learning_agent);

        // Security Specialist Agent
        let security_agent = Box::new(SecuritySpecialistAgent::new(None));
        let security_id = security_agent.metadata().id;
        registry.insert("security_specialist".to_string(), security_id.0);
        agents.insert(security_id.0, security_agent);

        // Data Analytics Agent
        let analytics_agent = Box::new(DataAnalyticsAgent::new(None));
        let analytics_id = analytics_agent.metadata().id;
        registry.insert("data_analytics".to_string(), analytics_id.0);
        agents.insert(analytics_id.0, analytics_agent);

        // Integration Agent
        let integration_agent = Box::new(IntegrationAgent::new(None));
        let integration_id = integration_agent.metadata().id;
        registry.insert("integration".to_string(), integration_id.0);
        agents.insert(integration_id.0, integration_agent);

        info!("Specialized Layer initialized with {} domain expert agents", agents.len());
        Ok(())
    }

    /// Start all specialized agents
    pub async fn start_all_agents(&self) -> AgentResult<()> {
        info!("Starting all Specialized Layer agents");

        // Note: Agents are started individually when needed
        // The layer doesn't manage the lifecycle of individual agents
        info!("Specialized Layer agents are ready to be started individually");
        Ok(())
    }

    /// Stop all specialized agents
    pub async fn stop_all_agents(&self) -> AgentResult<()> {
        info!("Stopping all Specialized Layer agents");

        // Note: Agents should be stopped individually
        info!("Specialized Layer agents should be stopped individually");
        Ok(())
    }

    /// Get agent by name
    pub async fn get_agent_by_name(&self, name: &str) -> Option<Uuid> {
        let registry = self.agent_registry.read().await;
        registry.get(name).copied()
    }

    /// Get all agent IDs
    pub async fn get_all_agent_ids(&self) -> Vec<Uuid> {
        let agents = self.agents.read().await;
        agents.keys().copied().collect()
    }

    /// Get agent count
    pub async fn agent_count(&self) -> usize {
        let agents = self.agents.read().await;
        agents.len()
    }

    /// Get comprehensive status of all agents
    pub async fn get_layer_status(&self) -> AgentResult<SpecializedLayerStatus> {
        let agents = self.agents.read().await;
        let mut agent_statuses = HashMap::new();
        let mut active_count = 0;
        let mut total_tasks = 0;

        for (id, agent) in agents.iter() {
            match agent.get_status().await {
                Ok(status) => {
                    if let Some(active) = status.get("active").and_then(|v| v.as_bool()) {
                        if active {
                            active_count += 1;
                        }
                    }
                    if let Some(tasks) = status.get("task_count").and_then(|v| v.as_u64()) {
                        total_tasks += tasks;
                    }
                    agent_statuses.insert(*id, status);
                }
                Err(e) => {
                    error!("Failed to get status for agent {}: {}", id, e);
                }
            }
        }

        Ok(SpecializedLayerStatus {
            total_agents: agents.len(),
            active_agents: active_count,
            total_tasks,
            agent_statuses,
            layer_health: if active_count == agents.len() { 100.0 } else { 
                (active_count as f64 / agents.len() as f64) * 100.0 
            },
        })
    }

    /// Find the appropriate agent for a given task
    async fn find_agent_for_task(&self, task: &Task) -> AgentResult<AgentId> {
        let agents = self.agents.read().await;

        // Simple agent selection based on task type
        // This is a basic implementation - could be enhanced with more sophisticated routing
        for (id, agent) in agents.iter() {
            if agent.can_handle_task(task) {
                return Ok(AgentId(*id));
            }
        }

        Err(AgentError::AgentNotFound(format!("No agent found for task: {}", task.name)))
    }

    /// Broadcast message to all specialized agents
    pub async fn broadcast_message(&self, message: AgentMessage) -> AgentResult<()> {
        let mut agents = self.agents.write().await;
        for (id, agent) in agents.iter_mut() {
            if let Err(e) = agent.handle_message(message.clone()).await {
                error!("Failed to handle message for agent: {} ({}): {}", agent.metadata().name, id, e);
            }
        }
        Ok(())
    }

    /// Get specific agent capabilities
    pub async fn get_agent_capabilities(&self, agent_name: &str) -> AgentResult<Vec<String>> {
        let agent_id = self.get_agent_by_name(agent_name).await
            .ok_or_else(|| AgentError::AgentNotFound(agent_name.to_string()))?;

        let agents = self.agents.read().await;
        let agent = agents.get(&agent_id)
            .ok_or_else(|| AgentError::AgentNotFound(agent_name.to_string()))?;

        Ok(agent.capabilities().to_vec())
    }

    /// List all available agent names
    pub async fn list_agent_names(&self) -> Vec<String> {
        let registry = self.agent_registry.read().await;
        registry.keys().cloned().collect()
    }
}

/// Status information for the entire Specialized Layer
#[derive(Debug, serde::Serialize)]
pub struct SpecializedLayerStatus {
    pub total_agents: usize,
    pub active_agents: usize,
    pub total_tasks: u64,
    pub agent_statuses: HashMap<Uuid, serde_json::Value>,
    pub layer_health: f64, // Percentage of agents that are active
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_specialized_layer_creation() {
        let layer = SpecializedLayer::new().await.expect("Failed to create specialized layer");
        assert_eq!(layer.agent_count().await, 8);
    }

    #[tokio::test]
    async fn test_agent_lookup() {
        let layer = SpecializedLayer::new().await.expect("Failed to create specialized layer");
        
        let code_gen_id = layer.get_agent_by_name("code_generation").await;
        assert!(code_gen_id.is_some());

        let security_id = layer.get_agent_by_name("security_specialist").await;
        assert!(security_id.is_some());

        let analytics_id = layer.get_agent_by_name("data_analytics").await;
        assert!(analytics_id.is_some());

        let integration_id = layer.get_agent_by_name("integration").await;
        assert!(integration_id.is_some());

        let nonexistent_id = layer.get_agent_by_name("nonexistent").await;
        assert!(nonexistent_id.is_none());
    }

    #[tokio::test]
    async fn test_agent_names_list() {
        let layer = SpecializedLayer::new().await.expect("Failed to create specialized layer");
        let agent_names = layer.list_agent_names().await;
        
        assert_eq!(agent_names.len(), 8);
        assert!(agent_names.contains(&"code_generation".to_string()));
        assert!(agent_names.contains(&"testing".to_string()));
        assert!(agent_names.contains(&"deployment".to_string()));
        assert!(agent_names.contains(&"monitoring".to_string()));
        assert!(agent_names.contains(&"learning".to_string()));
        assert!(agent_names.contains(&"security_specialist".to_string()));
        assert!(agent_names.contains(&"data_analytics".to_string()));
        assert!(agent_names.contains(&"integration".to_string()));
    }

    #[tokio::test]
    async fn test_layer_status() {
        let layer = SpecializedLayer::new().await.expect("Failed to create specialized layer");
        let status = layer.get_layer_status().await.expect("Failed to get layer status");
        
        assert_eq!(status.total_agents, 8);
        assert_eq!(status.agent_statuses.len(), 8);
    }
}

/// Utility functions for specialized agent management
pub mod utils {
    use super::*;

    /// Get recommended agent for specific capability
    pub fn get_agent_for_capability(capability: crate::agents::AgentCapability) -> Option<&'static str> {
        match capability {
            crate::agents::AgentCapability::CodeGeneration => Some("code_generation"),
            crate::agents::AgentCapability::Testing => Some("testing"),
            crate::agents::AgentCapability::Deployment => Some("deployment"),
            crate::agents::AgentCapability::Monitoring => Some("monitoring"),
            crate::agents::AgentCapability::MachineLearning => Some("learning"),
            crate::agents::AgentCapability::SecurityScanning => Some("security_specialist"),
            crate::agents::AgentCapability::DataProcessing => Some("data_analytics"),
            crate::agents::AgentCapability::SystemIntegration => Some("integration"),
            _ => None,
        }
    }

    /// Get all agents that support a specific capability
    pub async fn get_agents_with_capability(
        layer: &SpecializedLayer,
        capability: &str,
    ) -> AgentResult<Vec<String>> {
        let mut matching_agents = Vec::new();
        let agent_names = layer.list_agent_names().await;

        for name in agent_names {
            let capabilities = layer.get_agent_capabilities(&name).await?;
            if capabilities.contains(&capability.to_string()) {
                matching_agents.push(name);
            }
        }

        Ok(matching_agents)
    }
}

// This completes the Specialized Layer implementation with all 8 domain expert agents:
// 1. Code Generation Agent - Multi-language automated code generation and optimization
// 2. Testing Agent - Comprehensive test automation and quality assurance  
// 3. Deployment Agent - Full CI/CD pipeline and deployment orchestration
// 4. Monitoring Agent - Complete observability and system monitoring
// 5. Learning Agent - ML/AI capabilities with model training and knowledge extraction
// 6. Security Specialist Agent - Security implementation and compliance monitoring
// 7. Data Analytics Agent - Data processing, analytics, and business intelligence
// 8. Integration Agent - System integration, API management, and workflow orchestration

// Together, these agents provide comprehensive operational capabilities to execute
// strategic decisions from the Board Layer with technical excellence and autonomous operation.
