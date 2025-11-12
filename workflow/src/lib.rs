//! Unified Workflow Engine - Orchestrates all operations

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod instrumentation;
use instrumentation::PipelineInstrumentation;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub version: String,
    pub stages: Vec<Stage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    pub name: String,
    pub stage_type: StageType,
    pub depends_on: Vec<String>,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum StageType {
    Sequential,
    Parallel,
    Conditional,
    Loop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub agent: String,
    pub action: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowState {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StageState {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

pub struct WorkflowEngine {
    workflows: Arc<Mutex<HashMap<String, Workflow>>>,
    states: Arc<Mutex<HashMap<String, WorkflowState>>>,
    stage_states: Arc<Mutex<HashMap<String, HashMap<String, StageState>>>>,
    instrumentation: Arc<PipelineInstrumentation>,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        let instrumentation = PipelineInstrumentation::new()
            .expect("failed to initialise pipeline instrumentation");
        Self {
            workflows: Arc::new(Mutex::new(HashMap::new())),
            states: Arc::new(Mutex::new(HashMap::new())),
            stage_states: Arc::new(Mutex::new(HashMap::new())),
            instrumentation: Arc::new(instrumentation),
        }
    }

    pub fn instrumentation(&self) -> Arc<PipelineInstrumentation> {
        Arc::clone(&self.instrumentation)
    }
    
    /// Load workflow from definition
    pub fn load_workflow(&self, workflow: Workflow) -> Result<String, String> {
        let id = workflow.name.clone();
        
        let mut workflows = self.workflows.lock().unwrap();
        workflows.insert(id.clone(), workflow);
        
        let mut states = self.states.lock().unwrap();
        states.insert(id.clone(), WorkflowState::Pending);
        
        println!("[WORKFLOW] Loaded workflow: {}", id);
        Ok(id)
    }
    
    /// Execute workflow
    pub fn execute(&self, workflow_id: &str) -> Result<(), String> {
        let workflow = {
            let workflows = self.workflows.lock().unwrap();
            workflows.get(workflow_id).cloned()
                .ok_or_else(|| format!("Workflow not found: {}", workflow_id))?
        };
        
        // Update state to running
        {
            let mut states = self.states.lock().unwrap();
            states.insert(workflow_id.to_string(), WorkflowState::Running);
        }
        
        println!("[WORKFLOW] Executing workflow: {}", workflow.name);
        
        // Execute stages
        for stage in &workflow.stages {
            // Check dependencies
            if !self.check_dependencies(workflow_id, &stage.depends_on)? {
                println!("[WORKFLOW] Skipping stage {} (dependencies not met)", stage.name);
                continue;
            }
            
            self.execute_stage(workflow_id, stage)?;
        }
        
        // Mark as completed
        {
            let mut states = self.states.lock().unwrap();
            states.insert(workflow_id.to_string(), WorkflowState::Completed);
        }
        
        println!("[WORKFLOW] Workflow {} completed successfully", workflow.name);
        Ok(())
    }
    
    /// Execute a single stage
    fn execute_stage(&self, workflow_id: &str, stage: &Stage) -> Result<(), String> {
        println!("[WORKFLOW] Executing stage: {} (type: {:?})", stage.name, stage.stage_type);
        
        // Update stage state
        self.set_stage_state(workflow_id, &stage.name, StageState::Running);
        
        match stage.stage_type {
            StageType::Sequential => self.execute_sequential(stage)?,
            StageType::Parallel => self.execute_parallel(stage)?,
            StageType::Conditional => self.execute_conditional(stage)?,
            StageType::Loop => self.execute_loop(stage)?,
        }
        
        // Mark stage as completed
        self.set_stage_state(workflow_id, &stage.name, StageState::Completed);
        Ok(())
    }
    
    /// Execute tasks sequentially
    fn execute_sequential(&self, stage: &Stage) -> Result<(), String> {
        for task in &stage.tasks {
            self.execute_task(task)?;
        }
        Ok(())
    }
    
    /// Execute tasks in parallel
    fn execute_parallel(&self, stage: &Stage) -> Result<(), String> {
        println!("[WORKFLOW] Executing {} tasks in parallel", stage.tasks.len());
        
        // In a real implementation, this would spawn threads/processes
        for task in &stage.tasks {
            self.execute_task(task)?;
        }
        
        Ok(())
    }
    
    /// Execute tasks conditionally
    fn execute_conditional(&self, stage: &Stage) -> Result<(), String> {
        // Implementation would check conditions
        self.execute_sequential(stage)
    }
    
    /// Execute tasks in a loop
    fn execute_loop(&self, stage: &Stage) -> Result<(), String> {
        // Implementation would loop based on condition
        self.execute_sequential(stage)
    }
    
    /// Execute a single task
    fn execute_task(&self, task: &Task) -> Result<(), String> {
        println!("[WORKFLOW] Executing task: agent={}, action={}", task.agent, task.action);
        self.observe_task(task)?;
        // Implementation would dispatch to appropriate agent
        Ok(())
    }

    fn observe_task(&self, task: &Task) -> Result<(), String> {
        let action_lower = task.action.to_lowercase();
        let metadata = parameters_to_value(&task.parameters);

        if action_lower.contains("relocat") {
            let source = task
                .parameters
                .get("source")
                .and_then(Value::as_str)
                .unwrap_or("unknown-source");
            let target = task
                .parameters
                .get("target")
                .or_else(|| task.parameters.get("destination"))
                .and_then(Value::as_str)
                .unwrap_or("unknown-target");
            self.instrumentation
                .log_relocation(&task.agent, source, target, metadata.clone())
                .map_err(|err| format!("relocation instrumentation failed: {}", err))?;
        }

        if action_lower.contains("doc")
            || action_lower.contains("handbook")
            || action_lower.contains("update")
        {
            let document_path = task
                .parameters
                .get("document")
                .or_else(|| task.parameters.get("doc"))
                .or_else(|| task.parameters.get("path"))
                .and_then(Value::as_str)
                .unwrap_or("docs/unknown.md");
            self.instrumentation
                .log_document_update(&task.agent, document_path, metadata)
                .map_err(|err| format!("documentation instrumentation failed: {}", err))?;
        }

        Ok(())
    }
    
    /// Check if dependencies are met
    fn check_dependencies(&self, workflow_id: &str, depends_on: &[String]) -> Result<bool, String> {
        if depends_on.is_empty() {
            return Ok(true);
        }
        
        let stage_states = self.stage_states.lock().unwrap();
        if let Some(states) = stage_states.get(workflow_id) {
            for dep in depends_on {
                if let Some(state) = states.get(dep) {
                    if *state != StageState::Completed {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    /// Set stage state
    fn set_stage_state(&self, workflow_id: &str, stage_name: &str, state: StageState) {
        let mut stage_states = self.stage_states.lock().unwrap();
        stage_states
            .entry(workflow_id.to_string())
            .or_insert_with(HashMap::new)
            .insert(stage_name.to_string(), state);
    }
    
    /// Get workflow state
    pub fn get_state(&self, workflow_id: &str) -> Option<WorkflowState> {
        let states = self.states.lock().unwrap();
        states.get(workflow_id).cloned()
    }
}

fn parameters_to_value(parameters: &HashMap<String, Value>) -> Value {
    let mut map = serde_json::Map::new();
    for (key, value) in parameters {
        map.insert(key.clone(), value.clone());
    }
    Value::Object(map)
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noa_core::security;
    use serde_json::json;

    #[test]
    fn test_workflow_creation() {
        let workflow = Workflow {
            name: "test".to_string(),
            version: "1.0".to_string(),
            stages: vec![],
        };

        let engine = WorkflowEngine::new();
        let id = engine.load_workflow(workflow).unwrap();
        assert_eq!(engine.get_state(&id), Some(WorkflowState::Pending));
    }

    #[test]
    fn test_instrumentation_generates_signed_operations() {
        let engine = WorkflowEngine::new();
        let instrumentation = engine.instrumentation();

        let relocation = instrumentation
            .log_relocation(
                "tester",
                "/tmp/source",
                "/tmp/target",
                json!({ "ticket": "rel-001" }),
            )
            .expect("relocation log should succeed");
        assert!(security::verify_signed_operation(&relocation));

        let documentation = instrumentation
            .log_document_update(
                "tester",
                "docs/audits/AUDITORS_HANDBOOK.md",
                json!({ "change": "test" }),
            )
            .expect("documentation log should succeed");
        assert!(security::verify_signed_operation(&documentation));
    }
}
