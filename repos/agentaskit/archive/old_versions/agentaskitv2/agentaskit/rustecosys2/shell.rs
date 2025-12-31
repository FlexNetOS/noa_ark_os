use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use serde_json::{json, Value};
use noa_abi::{Task, Queue, AgentResult};
use crate::{Agent, ExecutionContext};

pub struct ShellAgent {
    allow_execution: bool,
}

impl ShellAgent {
    pub fn new(allow_execution: bool) -> Self {
        Self { allow_execution }
    }
}

impl Agent for ShellAgent {
    fn name(&self) -> &'static str {
        "shell"
    }

    fn execute(
        &self,
        task: &Task,
        queue: &Queue,
        hooks: &serde_json::Value,
        context: &ExecutionContext,
    ) -> Result<AgentResult> {
        // Create logs directory
        fs::create_dir_all(&context.logs_dir)?;

        // Extract commands from hooks
        let commands = self.extract_commands(task, queue, hooks);
        
        if !self.allow_execution || !context.allow_shell || commands.is_empty() || context.dry_run {
            return Ok(AgentResult {
                task: task.id.clone(),
                queue: queue.name.clone(),
                agent: self.name().to_string(),
                status: "noop".to_string(),
                evidence: None,
                error: None,
                commands: None,
            });
        }

        let mut all_ok = true;
        let mut cmd_logs = Vec::new();

        for (i, cmd_spec) in commands.iter().enumerate() {
            let cmd_result = self.execute_command(task, cmd_spec, i + 1, context)
                .with_context(|| format!("Failed to execute command {}", i + 1))?;
            
            if cmd_result.get("rc").and_then(|v| v.as_i64()).unwrap_or(1) != 0 {
                all_ok = false;
            }
            
            cmd_logs.push(cmd_result);
        }

        let status = if all_ok { "ok" } else { "error" };

        Ok(AgentResult {
            task: task.id.clone(),
            queue: queue.name.clone(),
            agent: self.name().to_string(),
            status: status.to_string(),
            evidence: None,
            error: None,
            commands: Some(cmd_logs),
        })
    }
}

impl ShellAgent {
    fn extract_commands(&self, task: &Task, queue: &Queue, hooks: &Value) -> Vec<Value> {
        let mut commands = Vec::new();

        // Check task-specific hooks first
        if let Some(task_hooks) = hooks.get(&task.id) {
            if let Some(cmds) = task_hooks.get("commands").and_then(|v| v.as_array()) {
                commands.extend(cmds.iter().cloned());
            }
        }

        // Check queue-level hooks if no task-specific commands
        if commands.is_empty() {
            if let Some(queue_hooks) = hooks.get(&queue.name) {
                if let Some(cmds) = queue_hooks.get("commands").and_then(|v| v.as_array()) {
                    commands.extend(cmds.iter().cloned());
                }
            }
        }

        commands
    }

    fn execute_command(
        &self,
        task: &Task,
        cmd_spec: &Value,
        cmd_num: usize,
        context: &ExecutionContext,
    ) -> Result<Value> {
        let cwd = cmd_spec.get("cwd").and_then(|v| v.as_str());
        let cmd_array = cmd_spec.get("cmd").and_then(|v| v.as_array());

        if cmd_array.is_none() {
            return Ok(json!({
                "error": "Invalid command specification - missing 'cmd' array"
            }));
        }

        let cmd_strings: Vec<String> = cmd_array.unwrap()
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        if cmd_strings.is_empty() {
            return Ok(json!({
                "error": "Empty command array"
            }));
        }

        let log_path = Path::new(&context.logs_dir)
            .join(format!("{}_cmd{}.log", task.id, cmd_num));

        // Execute the command
        let mut command = Command::new(&cmd_strings[0]);
        if cmd_strings.len() > 1 {
            command.args(&cmd_strings[1..]);
        }
        
        if let Some(dir) = cwd {
            command.current_dir(dir);
        }

        let output = command.output()
            .with_context(|| format!("Failed to execute command: {:?}", cmd_strings))?;

        // Write log file
        let mut log_content = format!("$ {}\n", cmd_strings.join(" "));
        log_content.push_str(&String::from_utf8_lossy(&output.stdout));
        
        if !output.stderr.is_empty() {
            log_content.push_str("\n[stderr]\n");
            log_content.push_str(&String::from_utf8_lossy(&output.stderr));
        }

        fs::write(&log_path, log_content)?;

        Ok(json!({
            "cmd": cmd_strings,
            "cwd": cwd,
            "rc": output.status.code().unwrap_or(-1),
            "log": log_path.to_string_lossy()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_shell_agent_dry_run() {
        let agent = ShellAgent::new(true);
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
        
        let hooks = json!({
            "TASK-001": {
                "commands": [
                    {"cmd": ["echo", "hello"], "cwd": "/tmp"}
                ]
            }
        });
        
        let context = ExecutionContext {
            logs_dir: temp_dir.path().to_string_lossy().to_string(),
            allow_shell: true,
            dry_run: true, // This should prevent execution
        };

        let result = agent.execute(&task, &queue, &hooks, &context).unwrap();
        
        assert_eq!(result.agent, "shell");
        assert_eq!(result.status, "noop"); // Should be noop due to dry_run
    }

    #[test]
    fn test_command_extraction() {
        let agent = ShellAgent::new(true);
        
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
                "commands": [
                    {"cmd": ["echo", "task-specific"]},
                    {"cmd": ["ls", "-la"]}
                ]
            },
            "Test Queue": {
                "commands": [
                    {"cmd": ["echo", "queue-level"]}
                ]
            }
        });

        let commands = agent.extract_commands(&task, &queue, &hooks);
        assert_eq!(commands.len(), 2); // Should prefer task-specific commands
        
        let first_cmd = commands[0].get("cmd").unwrap().as_array().unwrap();
        assert_eq!(first_cmd[0].as_str().unwrap(), "echo");
        assert_eq!(first_cmd[1].as_str().unwrap(), "task-specific");
    }
}
