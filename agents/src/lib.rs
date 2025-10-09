//! Agent Factory - Multi-layered AI agent system
//! 
//! Integrates with NOA ARK OS agent registry and implementations

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use uuid::Uuid;

// Module declarations
pub mod factory;
pub mod registry;
pub mod implementations;
pub mod inference;

// Re-export key types
pub use registry::{AgentRegistry, AGENT_REGISTRY};
pub use inference::{InferenceEngine, InferenceConfig, LlamaInferenceEngine};

// Type aliases
pub type AgentId = String;

/// Version of the agent system
pub const VERSION: &str = "0.1.0";

/// Total number of agents in the NOA ecosystem
pub const TOTAL_AGENTS: usize = 928;

// Agent metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMetadata {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub layer: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub agent_id: String,
    #[serde(default)]
    pub health_status: String,
    #[serde(default)]
    pub agent_category: AgentCategory,
    #[serde(default)]
    pub agent_layer: AgentLayer,
}

impl AgentMetadata {
    pub fn new(name: String, description: String, category: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.clone(),
            description,
            category,
            tags: vec![],
            layer: String::from("worker"),
            status: String::from("healthy"),
            agent_id: name,
            health_status: String::from("healthy"),
            agent_category: AgentCategory::Other,
            agent_layer: AgentLayer::L4Operations,
        }
    }
    
    pub fn layer_name(&self) -> &str {
        &self.layer
    }
    
    pub fn is_healthy(&self) -> bool {
        self.status == "healthy"
    }
    
    pub fn needs_repair(&self) -> bool {
        self.status == "needs_repair" || self.status == "error"
    }
    
    pub fn set_layer(&mut self, layer: String) {
        self.layer = layer;
    }
    
    pub fn set_status(&mut self, status: String) {
        self.status = status;
    }
}

// Agent types and states
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentType {
    Master,
    Worker,
    SubAgent,
    Swarm,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentLanguage {
    Python,
    Rust,
    Go,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    Created,
    Initializing,
    Ready,
    Running,
    Paused,
    Terminating,
    Terminated,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentLayer {
    L1Autonomy,
    L2Reasoning,
    L3Orchestration,
    L4Operations,
    L5Infrastructure,
}

impl Default for AgentLayer {
    fn default() -> Self {
        Self::L4Operations
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentCategory {
    Analysis,
    Code,
    DevOps,
    Testing,
    Documentation,
    Security,
    Other,
}

impl Default for AgentCategory {
    fn default() -> Self {
        Self::Other
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    NeedsRepair,
    Error,
    Unknown,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

// Agent structure
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub agent_type: AgentType,
    pub language: AgentLanguage,
    pub state: AgentState,
    pub parent_id: Option<AgentId>,
    pub capabilities: Vec<String>,
    pub disposable: bool,
}

impl Agent {
    pub fn new(
        id: AgentId,
        name: String,
        agent_type: AgentType,
        language: AgentLanguage,
    ) -> Self {
        Self {
            id,
            name,
            agent_type,
            language,
            state: AgentState::Created,
            parent_id: None,
            capabilities: Vec::new(),
            disposable: false,
        }
    }
    
    pub fn make_disposable(mut self) -> Self {
        self.disposable = true;
        self
    }
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    #[error("Agent error: {0}")]
    AgentError(String),
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

// Factory implementation
pub struct AgentFactory {
    agents: Arc<Mutex<HashMap<AgentId, Agent>>>,
    next_id: Arc<Mutex<u64>>,
}

impl AgentFactory {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
        }
    }
    
    /// Create a new agent
    pub fn create_agent(
        &self,
        name: String,
        agent_type: AgentType,
        language: AgentLanguage,
        disposable: bool,
    ) -> Result<AgentId> {
        let id = {
            let mut next_id = self.next_id.lock().unwrap();
            let id = format!("agent_{}", *next_id);
            *next_id += 1;
            id
        };
        
        let mut agent = Agent::new(id.clone(), name, agent_type, language);
        if disposable {
            agent = agent.make_disposable();
        }
        
        let mut agents = self.agents.lock().unwrap();
        agents.insert(id.clone(), agent);
        
        println!("[FACTORY] Created agent: {}", id);
        Ok(id)
    }
    
    /// Get agent by ID
    pub fn get_agent(&self, id: &str) -> Option<Agent> {
        let agents = self.agents.lock().unwrap();
        agents.get(id).cloned()
    }
    
    /// Update agent state
    pub fn update_state(&self, id: &str, state: AgentState) -> Result<()> {
        let mut agents = self.agents.lock().unwrap();
        if let Some(agent) = agents.get_mut(id) {
            agent.state = state;
            Ok(())
        } else {
            Err(Error::AgentNotFound(id.to_string()))
        }
    }
    
    /// Terminate and cleanup disposable agent
    pub fn cleanup_agent(&self, id: &str) -> Result<()> {
        let mut agents = self.agents.lock().unwrap();
        if let Some(agent) = agents.get(id) {
            if agent.disposable {
                agents.remove(id);
                println!("[FACTORY] Cleaned up disposable agent: {}", id);
                Ok(())
            } else {
                Err(Error::AgentError(format!("Agent {} is not disposable", id)))
            }
        } else {
            Err(Error::AgentNotFound(id.to_string()))
        }
    }
    
    /// List all agents
    pub fn list_agents(&self) -> Vec<Agent> {
        let agents = self.agents.lock().unwrap();
        agents.values().cloned().collect()
    }
    
    /// Create a swarm of agents
    pub fn create_swarm(
        &self,
        swarm_name: String,
        count: usize,
        language: AgentLanguage,
    ) -> Result<Vec<AgentId>> {
        let mut swarm_ids = Vec::new();
        
        for i in 0..count {
            let name = format!("{}_{}", swarm_name, i);
            let id = self.create_agent(name, AgentType::Swarm, language.clone(), false)?;
            swarm_ids.push(id);
        }
        
        println!("[FACTORY] Created swarm: {} with {} agents", swarm_name, count);
        Ok(swarm_ids)
    }
}

impl Default for AgentFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_agent() {
        let factory = AgentFactory::new();
        let id = factory.create_agent(
            "test_agent".to_string(),
            AgentType::Worker,
            AgentLanguage::Rust,
            false,
        ).unwrap();
        
        let agent = factory.get_agent(&id).unwrap();
        assert_eq!(agent.name, "test_agent");
    }
    
    #[test]
    fn test_disposable_agent() {
        let factory = AgentFactory::new();
        let id = factory.create_agent(
            "disposable".to_string(),
            AgentType::SubAgent,
            AgentLanguage::Python,
            true,
        ).unwrap();
        
        let agent = factory.get_agent(&id).unwrap();
        assert!(agent.disposable);
        
        factory.cleanup_agent(&id).unwrap();
        assert!(factory.get_agent(&id).is_none());
    }
}
