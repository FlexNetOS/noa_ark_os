use anyhow::Result;
use noa_abi::{Task, Queue, AgentResult, EffectEnvelope, Budget, Permit};
use crate::effect::Effect;
use crate::ledger::Ledger;

/// Bridge between the agent system and effect system
/// Allows effects to be used as agents
pub struct EffectAgent<E: Effect> {
    effect: E,
    name: String,
}

impl<E: Effect> EffectAgent<E> {
    pub fn new(effect: E, name: String) -> Self {
        Self { effect, name }
    }
}

/// Trait for converting tasks to effect envelopes
pub trait TaskToEffect {
    fn to_effect_envelope(&self, task: &Task, queue: &Queue, hooks: &serde_json::Value) -> Result<EffectEnvelope>;
}

/// Default implementation for task to effect conversion
pub struct DefaultTaskConverter;

impl TaskToEffect for DefaultTaskConverter {
    fn to_effect_envelope(&self, task: &Task, _queue: &Queue, hooks: &serde_json::Value) -> Result<EffectEnvelope> {
        // Extract effect-specific configuration from hooks
        let effect_config = hooks.get(&task.id)
            .or_else(|| hooks.get("default"))
            .unwrap_or(&serde_json::Value::Null);

        let envelope = EffectEnvelope {
            effect: "generic".to_string(),
            v: 1,
            args: effect_config.clone(),
            invariants: vec![],
            idempotence_key: format!("{}_{}", task.id, chrono::Utc::now().timestamp()),
            budget: Budget { ms: 30000, tokens: 1000, io: 1000 }, // Default budget
            permit: Permit {
                subject: "task_executor".to_string(),
                action: "execute".to_string(),
                resource: task.id.clone(),
                expires_at: chrono::Utc::now().checked_add_signed(chrono::Duration::hours(1))
                    .unwrap_or_else(chrono::Utc::now)
                    .to_rfc3339(),
                caveats: serde_json::Value::Null,
                sig: None,
            },
        };

        Ok(envelope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_task_to_effect_conversion() {
        let converter = DefaultTaskConverter;
        
        let task = Task {
            id: "TASK-001".to_string(),
            title: "Test Task".to_string(),
            layer: "execution".to_string(),
            queue: "Test Queue".to_string(),
            metadata: json!({}),
        };
        
        let queue = Queue {
            name: "Test Queue".to_string(),
            tasks: vec![task.clone()],
            hooks: None,
        };
        
        let hooks = json!({
            "TASK-001": {
                "param1": "value1",
                "param2": 42
            }
        });

        let envelope = converter.to_effect_envelope(&task, &queue, &hooks).unwrap();
        
        assert_eq!(envelope.effect, "generic");
        assert_eq!(envelope.v, 1);
        assert_eq!(envelope.permit.resource, "TASK-001");
        assert_eq!(envelope.args, hooks["TASK-001"]);
    }
}
