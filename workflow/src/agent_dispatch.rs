use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use noa_agents::registry::AgentRegistry;
use noa_agents::unified_types::AgentMetadata;
use noa_agents::AgentFactory;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::Task;
use noa_core::scorekeeper::{MetricStatus, ScopeDirective, Scorekeeper};

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
        let (allowed_optional, directive) = compute_trust_guardrails(&task.tool_requirements);
        let mut optional_budget = allowed_optional;
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
            if requirement.optional {
                if optional_budget == 0 {
                    tool_receipts.push(ToolExecutionReceipt {
                        requirement: requirement.clone(),
                        status: ToolExecutionStatus::Skipped,
                        output: Value::Null,
                        error: Some(format!(
                            "Optional capability '{}' gated by trust status {:?} (multiplier {:.2})",
                            requirement.capability, directive.status, directive.optional_multiplier
                        )),
                    });
                    continue;
                }
                optional_budget = optional_budget.saturating_sub(1);
            }

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

fn compute_trust_guardrails(requirements: &[ToolRequirement]) -> (usize, ScopeDirective) {
    let directive = Scorekeeper::current_scope_directive();
    let optional_total = requirements.iter().filter(|req| req.optional).count();
    let allowed_optional = match directive.status {
        MetricStatus::Nominal => optional_total,
        _ => directive.allowed_optional(optional_total),
    };
    (allowed_optional, directive)
}

#[cfg(test)]
mod tests {
    use super::*;
    use noa_agents::AgentFactory;
    use noa_core::scorekeeper::ScoreInputs;
    use serde_json::Value;
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::Arc;
    use tempfile::tempdir;

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &Path) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(ref previous) = self.previous {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    #[test]
    fn trust_guardrails_skip_optional_tools_on_low_capability() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("trust.json");
        let keeper = Scorekeeper::with_storage(path.clone()).unwrap();
        keeper
            .record(
                ScoreInputs::new()
                    .integrity(120, 0)
                    .reversibility(90, 10)
                    .capability(10, 40),
            )
            .unwrap();
        let _guard = EnvGuard::set("NOA_TRUST_METRICS_PATH", &path);

        let requirements = vec![
            ToolRequirement {
                name: "primary".into(),
                capability: "workflow.taskDispatch".into(),
                optional: false,
                parameters: Value::Null,
            },
            ToolRequirement {
                name: "optional-a".into(),
                capability: "workflow.taskDispatch".into(),
                optional: true,
                parameters: Value::Null,
            },
            ToolRequirement {
                name: "optional-b".into(),
                capability: "workflow.taskDispatch".into(),
                optional: true,
                parameters: Value::Null,
            },
        ];

        let (allowed_optional, directive) = compute_trust_guardrails(&requirements);
        assert!(matches!(
            directive.status,
            MetricStatus::Critical | MetricStatus::Warning
        ));
        assert_eq!(allowed_optional, 1, "expected optional budget to shrink");

        let registry = AgentRegistry::new();
        let mut metadata = AgentMetadata::from_registry(
            "WorkflowVerifier".to_string(),
            "WorkflowVerifier".to_string(),
        );
        metadata
            .capabilities
            .push("workflow.taskDispatch".to_string());
        registry
            .upsert_metadata(metadata.clone())
            .expect("stub agent registration");

        let dispatcher =
            AgentDispatcher::with_handles(Arc::new(registry), Arc::new(AgentFactory::new()));

        let task = Task {
            agent: metadata.agent_id.clone(),
            action: "noop".to_string(),
            parameters: HashMap::new(),
            agent_role: None,
            tool_requirements: requirements.clone(),
        };

        let receipt = dispatcher
            .dispatch(&task)
            .expect("dispatch should succeed with gating");
        assert_eq!(receipt.tool_receipts.len(), 3);
        let skipped: Vec<_> = receipt
            .tool_receipts
            .iter()
            .filter(|entry| matches!(entry.status, ToolExecutionStatus::Skipped))
            .collect();
        assert_eq!(skipped.len(), 1, "only one optional tool should be gated");
        let gated_message = skipped[0]
            .error
            .as_ref()
            .expect("skipped tool should provide rationale");
        assert!(gated_message.contains("Optional capability"));
        assert!(gated_message.contains(&format!("{:?}", directive.status)));
    }
}
