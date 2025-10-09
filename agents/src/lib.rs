//! Agent Factory - Multi-layered AI agent system
//! 
//! Integrates with NOA ARK OS agent registry and implementations

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

// Module declarations
pub mod unified_types;
pub mod factory;
pub mod registry;
pub mod implementations;
pub mod inference;

// Re-export unified types
pub use unified_types::*;

// Re-export key components
pub use registry::{AgentRegistry, AGENT_REGISTRY};
pub use inference::{InferenceEngine, InferenceConfig, LlamaInferenceEngine};

/// Version of the agent system
pub const VERSION: &str = "0.1.0";

/// Total number of agents in the NOA ecosystem
pub const TOTAL_AGENTS: usize = 928;

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
