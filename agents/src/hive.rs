//! Hive Mind - Collective intelligence coordination

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crate::{AgentId, Agent};

#[derive(Debug, Clone)]
pub struct SharedKnowledge {
    pub key: String,
    pub value: String,
    pub contributor: AgentId,
    pub timestamp: u64,
}

pub struct HiveMind {
    knowledge_base: Arc<Mutex<HashMap<String, SharedKnowledge>>>,
    connected_agents: Arc<Mutex<Vec<AgentId>>>,
}

impl HiveMind {
    pub fn new() -> Self {
        Self {
            knowledge_base: Arc::new(Mutex::new(HashMap::new())),
            connected_agents: Arc::new(Mutex::new(Vec::new())),
        }
    }
    
    /// Connect agent to hive mind
    pub fn connect_agent(&self, agent_id: AgentId) {
        let mut agents = self.connected_agents.lock().unwrap();
        if !agents.contains(&agent_id) {
            agents.push(agent_id.clone());
            println!("[HIVE] Agent {} connected to hive mind", agent_id);
        }
    }
    
    /// Disconnect agent from hive mind
    pub fn disconnect_agent(&self, agent_id: &str) {
        let mut agents = self.connected_agents.lock().unwrap();
        agents.retain(|id| id != agent_id);
        println!("[HIVE] Agent {} disconnected from hive mind", agent_id);
    }
    
    /// Share knowledge with hive
    pub fn share_knowledge(
        &self,
        key: String,
        value: String,
        contributor: AgentId,
    ) -> Result<(), String> {
        let knowledge = SharedKnowledge {
            key: key.clone(),
            value,
            contributor,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        let mut kb = self.knowledge_base.lock().unwrap();
        kb.insert(key, knowledge);
        
        Ok(())
    }
    
    /// Query knowledge from hive
    pub fn query_knowledge(&self, key: &str) -> Option<SharedKnowledge> {
        let kb = self.knowledge_base.lock().unwrap();
        kb.get(key).cloned()
    }
    
    /// Get all connected agents
    pub fn get_connected_agents(&self) -> Vec<AgentId> {
        let agents = self.connected_agents.lock().unwrap();
        agents.clone()
    }
    
    /// Broadcast message to all agents in hive
    pub fn broadcast(&self, message: String) {
        let agents = self.connected_agents.lock().unwrap();
        println!("[HIVE] Broadcasting to {} agents: {}", agents.len(), message);
        // Implementation would send to all agents via IPC
    }
}

impl Default for HiveMind {
    fn default() -> Self {
        Self::new()
    }
}
