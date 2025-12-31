use anyhow::Result;
use std::fs;
use std::path::Path;
use noa_abi::{Task, Queue, AgentResult};
use crate::{Agent, ExecutionContext};

pub struct NoOpAgent;

impl Agent for NoOpAgent {
    fn name(&self) -> &'static str {
        "noop"
    }

    fn execute(
        &self,
        task: &Task,
        queue: &Queue,
        _hooks: &serde_json::Value,
        context: &ExecutionContext,
    ) -> Result<AgentResult> {
        // Create logs directory if it doesn't exist
        fs::create_dir_all(&context.logs_dir)?;

        // Generate evidence file
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%SZ");
        let evidence_path = Path::new(&context.logs_dir)
            .join(format!("{}_noop_{}.txt", task.id, timestamp));

        let evidence_content = format!(
            "NOOP: {} in {} at {}\nDry run: {}\nTask layer: {}\n",
            task.id, queue.name, timestamp, context.dry_run, task.layer
        );

        fs::write(&evidence_path, evidence_content)?;

        Ok(AgentResult {
            task: task.id.clone(),
            queue: queue.name.clone(),
            agent: self.name().to_string(),
            status: "noop".to_string(),
            evidence: Some(evidence_path.to_string_lossy().to_string()),
            error: None,
            commands: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_noop_agent_execution() {
        let agent = NoOpAgent;
        let temp_dir = TempDir::new().unwrap();
        
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
        
        let context = ExecutionContext {
            logs_dir: temp_dir.path().to_string_lossy().to_string(),
            allow_shell: false,
            dry_run: true,
        };

        let result = agent.execute(&task, &queue, &json!({}), &context).unwrap();
        
        assert_eq!(result.agent, "noop");
        assert_eq!(result.status, "noop");
        assert_eq!(result.task, "TASK-001");
        assert_eq!(result.queue, "Test Queue");
        assert!(result.evidence.is_some());
        
        // Verify evidence file was created
        let evidence_path = result.evidence.unwrap();
        assert!(Path::new(&evidence_path).exists());
        
        let content = fs::read_to_string(&evidence_path).unwrap();
        assert!(content.contains("NOOP: TASK-001"));
        assert!(content.contains("Test Queue"));
    }
}
