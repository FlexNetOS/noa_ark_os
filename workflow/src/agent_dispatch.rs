use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use noa_agents::registry::AgentRegistry;
use noa_agents::unified_types::AgentMetadata;
use noa_agents::AgentFactory;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::Task;

#[derive(Debug, Error)]
pub enum AgentDispatchError {
    #[error("agent '{0}' not found in registry")]
    AgentNotFound(String),
    #[error("failed to instantiate agent: {0}")]
    AgentFactory(String),
}

#[derive(Debug, Clone, Deserialize)]
struct RoleMapping {
    agent_id: String,
    #[serde(default)]
    description: Option<String>,
}

fn role_directory() -> &'static HashMap<String, RoleMapping> {
    static ROLE_DIRECTORY: OnceLock<HashMap<String, RoleMapping>> = OnceLock::new();
    ROLE_DIRECTORY.get_or_init(|| {
        let raw = include_str!("../../agents/data/agent_roles.json");
        let parsed: HashMap<String, RoleMapping> = serde_json::from_str(raw).unwrap_or_default();
        parsed
            .into_iter()
            .map(|(role, mapping)| (role.to_lowercase(), mapping))
            .collect()
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRequirement {
    pub name: String,
    pub capability: String,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub parameters: Value,
}

impl ToolRequirement {
    pub fn matches(&self, provided: &str) -> bool {
        provided.eq_ignore_ascii_case(&self.capability)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolExecutionStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionReceipt {
    pub requirement: ToolRequirement,
    pub status: ToolExecutionStatus,
    pub output: Value,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDispatchReceipt {
    pub agent_metadata: AgentMetadata,
    pub agent_instance_id: String,
    pub task: Task,
    pub output: Value,
    pub tool_receipts: Vec<ToolExecutionReceipt>,
}

pub struct AgentDispatcher {
    registry: Arc<AgentRegistry>,
    factory: Arc<AgentFactory>,
}

impl AgentDispatcher {
    pub fn new(registry: AgentRegistry, factory: AgentFactory) -> Self {
        Self::with_handles(Arc::new(registry), Arc::new(factory))
    }

    pub fn with_handles(registry: Arc<AgentRegistry>, factory: Arc<AgentFactory>) -> Self {
        Self { registry, factory }
    }

    pub fn registry(&self) -> Arc<AgentRegistry> {
        Arc::clone(&self.registry)
    }

    pub fn factory(&self) -> Arc<AgentFactory> {
        Arc::clone(&self.factory)
    }

    pub fn dispatch(&self, task: &Task) -> Result<TaskDispatchReceipt, AgentDispatchError> {
        let metadata = self.resolve_agent_metadata(task)?;

        let instance_id = self
            .factory
            .create_agent(
                metadata.name.clone(),
                metadata.agent_type.clone(),
                metadata.language.clone(),
                true,
            )
            .map_err(|err| AgentDispatchError::AgentFactory(err.to_string()))?;

        let mut tool_receipts = Vec::new();
        for requirement in &task.tool_requirements {
            let status = if metadata
                .capabilities
                .iter()
                .any(|cap| requirement.matches(cap))
            {
                ToolExecutionStatus::Succeeded
            } else if requirement.optional {
                ToolExecutionStatus::Skipped
            } else {
                ToolExecutionStatus::Failed
            };

            let error = if matches!(status, ToolExecutionStatus::Failed) {
                Some(format!(
                    "Agent '{}' is missing required capability '{}'.",
                    metadata.agent_id, requirement.capability
                ))
            } else {
                None
            };

            tool_receipts.push(ToolExecutionReceipt {
                requirement: requirement.clone(),
                status,
                output: Value::Null,
                error,
            });
        }

        let mut overall_output = Value::Null;
        if tool_receipts
            .iter()
            .all(|receipt| !matches!(receipt.status, ToolExecutionStatus::Failed))
        {
            overall_output = serde_json::json!({
                "agent": metadata.agent_id,
                "status": "completed",
            });
        }

        Ok(TaskDispatchReceipt {
            agent_metadata: metadata,
            agent_instance_id: instance_id,
            task: task.clone(),
            output: overall_output,
            tool_receipts,
        })
    }
}

impl AgentDispatcher {
    fn resolve_agent_metadata(&self, task: &Task) -> Result<AgentMetadata, AgentDispatchError> {
        if let Some(role) = task
            .agent_role
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            return self.resolve_agent_by_role(task, role);
        }

        if let Some(role) = task
            .agent
            .strip_prefix("role::")
            .or_else(|| task.agent.strip_prefix("role:"))
        {
            return self.resolve_agent_by_role(task, role.trim());
        }

        self.find_agent(&task.agent)
            .ok_or_else(|| AgentDispatchError::AgentNotFound(task.agent.clone()))
    }

    fn resolve_agent_by_role(
        &self,
        task: &Task,
        role: &str,
    ) -> Result<AgentMetadata, AgentDispatchError> {
        let role_key = role.to_lowercase();
        let mapping = role_directory()
            .get(&role_key)
            .ok_or_else(|| AgentDispatchError::AgentNotFound(format!("role::{role}")))?;

        let mut metadata = self
            .find_agent(&mapping.agent_id)
            .ok_or_else(|| AgentDispatchError::AgentNotFound(mapping.agent_id.clone()))?;
        metadata.role = task.agent_role.clone().unwrap_or_else(|| role.to_string());
        Ok(metadata)
    }

    fn find_agent(&self, identifier: &str) -> Option<AgentMetadata> {
        self.registry.get(identifier).or_else(|| {
            let name = identifier.to_lowercase();
            self.registry.all().into_iter().find(|agent| {
                agent.agent_id.eq_ignore_ascii_case(identifier) || agent.name.to_lowercase() == name
            })
        })
    }
}
