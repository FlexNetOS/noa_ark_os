//! Factory module - Agent creation and lifecycle management

use crate::{Agent, AgentFactory, AgentId, AgentLanguage, AgentType, Result};

impl AgentFactory {
    /// Create multiple agents at once
    pub fn create_batch(
        &self,
        count: usize,
        name_prefix: String,
        agent_type: AgentType,
        language: AgentLanguage,
    ) -> Result<Vec<AgentId>> {
        let mut agent_ids = Vec::new();

        for i in 0..count {
            let name = format!("{}_{}", name_prefix, i);
            let id = self.create_agent(name, agent_type.clone(), language.clone(), false)?;
            agent_ids.push(id);
        }

        Ok(agent_ids)
    }

    /// Get all agents of a specific type
    pub fn get_agents_by_type(&self, agent_type: AgentType) -> Vec<Agent> {
        let agents = self.agents.lock().unwrap();
        agents
            .values()
            .filter(|a| a.agent_type == agent_type)
            .cloned()
            .collect()
    }

    /// Get all disposable agents
    pub fn get_disposable_agents(&self) -> Vec<Agent> {
        let agents = self.agents.lock().unwrap();
        agents.values().filter(|a| a.disposable).cloned().collect()
    }

    /// Cleanup all disposable agents
    pub fn cleanup_all_disposable(&self) -> Result<usize> {
        let disposable_ids: Vec<String> = {
            let agents = self.agents.lock().unwrap();
            agents
                .values()
                .filter(|a| a.disposable)
                .map(|a| a.id.clone())
                .collect()
        };

        let count = disposable_ids.len();
        for id in disposable_ids {
            self.cleanup_agent(&id)?;
        }

        Ok(count)
    }
}
