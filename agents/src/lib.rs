//! Agent Factory - Multi-layered AI agent system
//!
//! Integrates with NOA ARK OS agent registry and implementations

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::implementations::specialist::PolicyEnforcementAgent;
use noa_core::capabilities::{
    CapabilityDefinition, CapabilityError, CapabilityResult, DynCapability, KernelHandle,
};
use noa_core::config::manifest::{
    CAPABILITY_GATEWAY, CAPABILITY_MEMORY, CAPABILITY_PROCESS, CAPABILITY_SECURITY,
};
use noa_core::process::ProcessService;

// Module declarations
pub mod factory;
pub mod implementations;
pub mod inference;
pub mod registry;
pub mod runtime;
pub mod unified_types;

// Re-export unified types
pub use unified_types::*;

// Re-export key components
pub use inference::{InferenceConfig, InferenceEngine, LlamaInferenceEngine};
pub use registry::AgentRegistry;
pub use runtime::RuntimeManager;

/// Version of the agent system
pub const VERSION: &str = "0.1.0";

/// Total number of agents in the NOA ecosystem
pub const TOTAL_AGENTS: usize = 928;

/// Capability identifier exposing the agent factory through the kernel.
pub const AGENT_FACTORY_CAPABILITY: &str = "agents.factory";

/// Register the agent factory capability with the kernel registry.
pub fn register_kernel_capabilities(kernel: &KernelHandle) -> CapabilityResult<()> {
    let registry = kernel.registry();
    let definition = CapabilityDefinition::builder(AGENT_FACTORY_CAPABILITY)
        .description("Kernel-managed agent factory and registry")
        .depends_on([
            CAPABILITY_PROCESS,
            CAPABILITY_MEMORY,
            CAPABILITY_SECURITY,
            CAPABILITY_GATEWAY,
        ])
        .init_with(|context| {
            let factory = AgentFactory::with_kernel(context.kernel().clone()).map_err(|err| {
                CapabilityError::InitializationFailed(
                    AGENT_FACTORY_CAPABILITY.to_string(),
                    err.to_string(),
                )
            })?;
            Ok(Arc::new(factory) as DynCapability)
        })
        .build();

    match registry.register_definition(definition) {
        Ok(()) => Ok(()),
        Err(CapabilityError::AlreadyRegistered(_)) => Ok(()),
        Err(err) => Err(err),
    }
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    #[error("Agent error: {0}")]
    AgentError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
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
    kernel: Option<KernelHandle>,
}

impl AgentFactory {
    fn new_with_kernel(kernel: Option<KernelHandle>) -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(1)),
            kernel,
        }
    }

    pub fn new() -> Self {
        let factory = Self::new_with_kernel(None);
        if let Err(err) = factory.bootstrap_policy_specialist() {
            eprintln!(
                "[FACTORY] Failed to seed policy specialist (pe-ssp): {}",
                err
            );
        }
        factory
    }

    /// Create an agent factory bound to a kernel handle.
    pub fn with_kernel(kernel: KernelHandle) -> Result<Self> {
        let factory = Self::new_with_kernel(Some(kernel));
        factory.bootstrap_policy_specialist()?;
        Ok(factory)
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

        if let Some(kernel) = &self.kernel {
            if let Ok(process_service) = kernel.request::<ProcessService>(CAPABILITY_PROCESS) {
                let _ = process_service.create_process(format!("agent::{id}"));
            }
        }

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

        println!(
            "[FACTORY] Created swarm: {} with {} agents",
            swarm_name, count
        );
        Ok(swarm_ids)
    }

    fn bootstrap_policy_specialist(&self) -> Result<()> {
        let policy_agent = PolicyEnforcementAgent::new();
        policy_agent.ensure_factory_registration(self)?;
        Ok(())
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
        let id = factory
            .create_agent(
                "test_agent".to_string(),
                AgentType::Worker,
                AgentLanguage::Rust,
                false,
            )
            .unwrap();

        let agent = factory.get_agent(&id).unwrap();
        assert_eq!(agent.name, "test_agent");
    }

    #[test]
    fn test_disposable_agent() {
        let factory = AgentFactory::new();
        let id = factory
            .create_agent(
                "disposable".to_string(),
                AgentType::SubAgent,
                AgentLanguage::Python,
                true,
            )
            .unwrap();

        let agent = factory.get_agent(&id).unwrap();
        assert!(agent.disposable);

        factory.cleanup_agent(&id).unwrap();
        assert!(factory.get_agent(&id).is_none());
    }
}
