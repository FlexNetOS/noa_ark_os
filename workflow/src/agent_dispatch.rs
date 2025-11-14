use std::sync::Arc;

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
        let metadata = self
            .registry
            .get(&task.agent)
            .or_else(|| {
                self.registry
                    .all()
                    .into_iter()
                    .find(|agent| agent.name == task.agent)
            })
            .ok_or_else(|| AgentDispatchError::AgentNotFound(task.agent.clone()))?;

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
