//! Agent Factory - Multi-layered AI agent system
//! 
//! Integrates with NOA ARK OS agent registry and implementations

pub mod factory;
pub mod hive;
pub mod swarm;
pub mod runtime;
pub mod registry;      // NEW: Agent registry from CRC drops
pub mod types;         // NEW: Enhanced types
pub mod error;         // NEW: Error handling
pub mod implementations; // NEW: Agent implementations from agentaskit
pub mod agentaskit;    // AgentAsKit integration scaffolding
pub mod communication; // Agent communication hub

use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub type AgentId = String;

// Re-export registry components
pub use registry::AgentRegistry;
pub use types::{AgentMetadata, AgentLayer, AgentCategory, HealthStatus};
pub use error::{Error, Result};
pub use communication::{AgentCommunicationHub, AgentMessage};

/// Version of the agent system
pub const VERSION: &str = "0.1.0";

/// Total number of agents in the NOA ecosystem
pub const TOTAL_AGENTS: usize = 928;

#[derive(Debug, Clone, PartialEq)]
pub enum AgentType {
    Master,
    Worker,
    SubAgent,
    Swarm,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentLanguage {
    Python,
    Rust,
    Go,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentState {
    Created,
    Initializing,
    Ready,
    Running,
    Paused,
    Terminating,
    Terminated,
}

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
                Err(Error::AgentNotFound(format!("Agent {} is not disposable", id)))
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
