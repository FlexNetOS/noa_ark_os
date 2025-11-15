//! Unified Workflow Engine - Orchestrates all operations

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{Duration, Utc};
use noa_agents::{
    unified_types::{AgentCategory, AgentMetadata},
    AgentFactory, AgentRegistry, AGENT_FACTORY_CAPABILITY,
};
use noa_core::capabilities::KernelHandle;
use noa_core::config::manifest::CAPABILITY_PROCESS;
use noa_core::process::ProcessService;
use noa_core::utils::current_timestamp_millis;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

mod agent_dispatch;
mod instrumentation;
mod reward;
pub use agent_dispatch::{
    AgentDispatchError, AgentDispatcher, TaskDispatchReceipt, ToolExecutionReceipt,
    ToolExecutionStatus, ToolRequirement,
};
pub use instrumentation::{
    AgentExecutionResult, DeploymentOutcomeRecord, EvidenceLedgerEntry, EvidenceLedgerKind,
    GoalAgentMetric, GoalMetricSnapshot, GoalOutcomeRecord, InferenceMetric, MerkleLeaf,
    MerkleLevel, PipelineInstrumentation, SecurityScanReport, SecurityScanStatus, StageReceipt,
    TaskReceipt,
};
pub use reward::{
    AgentApprovalStatus, AgentStanding, AgentStandingSummary, RewardAgentSnapshot, RewardDelta,
    RewardInputs, RewardReport, RewardScorekeeper,
};
use tokio::sync::broadcast;

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
    #[serde(default)]
    pub agent_role: Option<String>,
    #[serde(default)]
    pub tool_requirements: Vec<ToolRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowState {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StageState {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowResumeToken {
    pub workflow_id: String,
    pub stage_id: Option<String>,
    pub checkpoint: String,
    pub issued_at: String,
    pub expires_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkflowEvent {
    WorkflowState {
        workflow_id: String,
        state: WorkflowState,
        timestamp: String,
    },
    StageState {
        workflow_id: String,
        stage_id: String,
        state: StageState,
        timestamp: String,
    },
    StageReceiptGenerated {
        workflow_id: String,
        stage_id: String,
        receipt: StageReceipt,
        timestamp: String,
    },
    ResumeOffered {
        workflow_id: String,
        token: WorkflowResumeToken,
        timestamp: String,
    },
}

#[derive(Clone)]
pub struct WorkflowEventStream {
    sender: broadcast::Sender<WorkflowEvent>,
}

impl WorkflowEventStream {
    pub fn new(buffer: usize) -> Self {
        let (sender, _receiver) = broadcast::channel(buffer);
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WorkflowEvent> {
        self.sender.subscribe()
    }

    pub fn send(&self, event: WorkflowEvent) {
        let _ = self.sender.send(event);
    }
}

#[derive(Default, Clone)]
struct GoalRunTracker {
    agents: Vec<AgentExecutionResult>,
    total_token_ratio: f64,
    token_samples: u32,
    rollback_count: u32,
}

impl GoalRunTracker {
    fn record(&mut self, agent: &str, success: bool, token_ratio: Option<f64>, rollback: bool) {
        self.agents.push(AgentExecutionResult {
            agent: agent.to_string(),
            success,
        });
        if let Some(ratio) = token_ratio {
            self.total_token_ratio += ratio;
            self.token_samples = self.token_samples.saturating_add(1);
        }
        if rollback {
            self.rollback_count = self.rollback_count.saturating_add(1);
        }
    }

    fn snapshot(&self) -> Vec<AgentExecutionResult> {
        self.agents.clone()
    }

    fn into_snapshot(self) -> Vec<AgentExecutionResult> {
        self.agents
    }

    fn reward_inputs(&self) -> RewardInputs {
        let total_runs = self.agents.len() as f64;
        let successes = self.agents.iter().filter(|agent| agent.success).count() as f64;
        let coverage = if total_runs.abs() < f64::EPSILON {
            1.0
        } else {
            (successes / total_runs).clamp(0.0, 1.0)
        };
        // Flake rate: proportion of agents that have both successes and failures (i.e., are flaky)
        let mut agent_outcomes: HashMap<&str, (usize, usize)> = HashMap::new();
        for result in &self.agents {
            let entry = agent_outcomes.entry(result.agent.as_str()).or_insert((0, 0));
            if result.success {
                entry.0 += 1;
            } else {
                entry.1 += 1;
            }
        }
        let total_agents = agent_outcomes.len() as f64;
        let flaky_agents = agent_outcomes.values().filter(|(succ, fail)| *succ > 0 && *fail > 0).count() as f64;
        let flake_rate = if total_agents.abs() < f64::EPSILON {
            0.0
        } else {
            (flaky_agents / total_agents).clamp(0.0, 1.0)
        };
        let token_ratio = if self.token_samples == 0 {
            1.0
        } else {
            (self.total_token_ratio / self.token_samples as f64).max(0.0)
        };
        RewardInputs {
            coverage,
            flake_rate,
            token_ratio,
            rollback_count: self.rollback_count,
        }
    }
}

pub struct WorkflowEngine {
    workflows: Arc<Mutex<HashMap<String, Workflow>>>,
    states: Arc<Mutex<HashMap<String, WorkflowState>>>,
    stage_states: Arc<Mutex<HashMap<String, HashMap<String, StageState>>>>,
    instrumentation: Arc<PipelineInstrumentation>,
    dispatcher: Arc<AgentDispatcher>,
    kernel: Option<KernelHandle>,
    event_stream: Arc<Mutex<Option<WorkflowEventStream>>>,
}

impl WorkflowEngine {
    pub fn new() -> Self {
        let instrumentation =
            PipelineInstrumentation::new().expect("failed to initialise pipeline instrumentation");
        let registry = AgentRegistry::with_default_data().unwrap_or_else(|_| AgentRegistry::new());
        let factory = AgentFactory::new();
        let dispatcher = AgentDispatcher::new(registry, factory);
        Self {
            workflows: Arc::new(Mutex::new(HashMap::new())),
            states: Arc::new(Mutex::new(HashMap::new())),
            stage_states: Arc::new(Mutex::new(HashMap::new())),
            instrumentation: Arc::new(instrumentation),
            dispatcher: Arc::new(dispatcher),
            kernel: None,
            event_stream: Arc::new(Mutex::new(None)),
        }
    }

    pub fn instrumentation(&self) -> Arc<PipelineInstrumentation> {
        Arc::clone(&self.instrumentation)
    }

    /// Create a workflow engine that interacts with kernel capabilities.
    pub fn with_kernel(kernel: KernelHandle) -> Self {
        let instrumentation =
            PipelineInstrumentation::new().expect("failed to initialise pipeline instrumentation");
        let registry = AgentRegistry::with_default_data().unwrap_or_else(|_| AgentRegistry::new());
        let factory =
            AgentFactory::with_kernel(kernel.clone()).unwrap_or_else(|_| AgentFactory::new());
        let dispatcher = AgentDispatcher::new(registry, factory);
        Self {
            workflows: Arc::new(Mutex::new(HashMap::new())),
            states: Arc::new(Mutex::new(HashMap::new())),
            stage_states: Arc::new(Mutex::new(HashMap::new())),
            instrumentation: Arc::new(instrumentation),
            dispatcher: Arc::new(dispatcher),
            kernel: Some(kernel),
            event_stream: Arc::new(Mutex::new(None)),
        }
    }

    pub fn enable_streaming(&self, buffer: usize) -> WorkflowEventStream {
        let stream = WorkflowEventStream::new(buffer);
        self.event_stream.lock().unwrap().replace(stream.clone());
        stream
    }

    pub fn event_stream(&self) -> Option<WorkflowEventStream> {
        self.event_stream.lock().unwrap().clone()
    }

    /// Load workflow from definition
    pub fn load_workflow(&self, workflow: Workflow) -> Result<String, String> {
        let id = workflow.name.clone();

        let mut workflows = self.workflows.lock().unwrap();
        workflows.insert(id.clone(), workflow);

        let mut states = self.states.lock().unwrap();
        states.insert(id.clone(), WorkflowState::Pending);

        self.emit_event(WorkflowEvent::WorkflowState {
            workflow_id: id.clone(),
            state: WorkflowState::Pending,
            timestamp: now_iso(),
        });

        println!("[WORKFLOW] Loaded workflow: {}", id);
        Ok(id)
    }

    /// Execute workflow
    pub fn execute(&self, workflow_id: &str) -> Result<(), String> {
        let workflow = {
            let workflows = self.workflows.lock().unwrap();
            workflows
                .get(workflow_id)
                .cloned()
                .ok_or_else(|| format!("Workflow not found: {}", workflow_id))?
        };

        // Update state to running
        {
            let mut states = self.states.lock().unwrap();
            states.insert(workflow_id.to_string(), WorkflowState::Running);
        }

        self.emit_event(WorkflowEvent::WorkflowState {
            workflow_id: workflow_id.to_string(),
            state: WorkflowState::Running,
            timestamp: now_iso(),
        });

        println!("[WORKFLOW] Executing workflow: {}", workflow.name);

        let run_started_at = current_timestamp_millis();
        let mut tracker = GoalRunTracker::default();

        // Execute stages
        for stage in &workflow.stages {
            // Check dependencies
            if !self.check_dependencies(workflow_id, &stage.depends_on)? {
                println!(
                    "[WORKFLOW] Skipping stage {} (dependencies not met)",
                    stage.name
                );
                continue;
            }

            if let Err(err) = self.execute_stage(workflow_id, stage, &mut tracker) {
                println!(
                    "[WORKFLOW] Stage {} failed for workflow {}: {}",
                    stage.name, workflow.name, err
                );
                self.set_stage_state(workflow_id, &stage.name, StageState::Failed);
                {
                    let mut states = self.states.lock().unwrap();
                    states.insert(workflow_id.to_string(), WorkflowState::Failed);
                }
                self.emit_event(WorkflowEvent::WorkflowState {
                    workflow_id: workflow_id.to_string(),
                    state: WorkflowState::Failed,
                    timestamp: now_iso(),
                });
                let completed_at = current_timestamp_millis();
                let outcome = GoalOutcomeRecord {
                    goal_id: workflow_id.to_string(),
                    workflow_id: workflow.name.clone(),
                    started_at: run_started_at,
                    completed_at,
                    duration_ms: completed_at.saturating_sub(run_started_at),
                    success: false,
                    agents: tracker.snapshot(),
                    reward_inputs: Some(tracker.reward_inputs()),
                };
                if let Err(metric_err) = self.instrumentation.record_goal_outcome(outcome) {
                    println!("[WORKFLOW] Failed to record goal outcome: {}", metric_err);
                }
                return Err(err);
            }
        }

        let completed_at = current_timestamp_millis();
        let outcome = GoalOutcomeRecord {
            goal_id: workflow_id.to_string(),
            workflow_id: workflow.name.clone(),
            started_at: run_started_at,
            completed_at,
            duration_ms: completed_at.saturating_sub(run_started_at),
            success: true,
            agents: tracker.snapshot(),
            reward_inputs: Some(tracker.reward_inputs()),
        };
        if let Err(metric_err) = self.instrumentation.record_goal_outcome(outcome) {
            println!("[WORKFLOW] Failed to record goal outcome: {}", metric_err);
        }

        // Mark as completed
        {
            let mut states = self.states.lock().unwrap();
            states.insert(workflow_id.to_string(), WorkflowState::Completed);
        }

        self.emit_event(WorkflowEvent::WorkflowState {
            workflow_id: workflow_id.to_string(),
            state: WorkflowState::Completed,
            timestamp: now_iso(),
        });

        println!(
            "[WORKFLOW] Workflow {} completed successfully",
            workflow.name
        );
        Ok(())
    }

    /// Execute a single stage
    fn execute_stage(
        &self,
        workflow_id: &str,
        stage: &Stage,
        tracker: &mut GoalRunTracker,
    ) -> Result<(), String> {
        println!(
            "[WORKFLOW] Executing stage: {} (type: {:?})",
            stage.name, stage.stage_type
        );

        // Update stage state
        self.set_stage_state(workflow_id, &stage.name, StageState::Running);

        let artifacts = match stage.stage_type {
            StageType::Sequential => self.execute_sequential(workflow_id, stage, tracker)?,
            StageType::Parallel => self.execute_parallel(workflow_id, stage, tracker)?,
            StageType::Conditional => self.execute_conditional(workflow_id, stage, tracker)?,
            StageType::Loop => self.execute_loop(workflow_id, stage, tracker)?,
        };

        let receipt = self
            .instrumentation
            .log_stage_receipt(workflow_id, stage, &artifacts)
            .map_err(|err| format!("stage receipt failed: {}", err))?;

        println!(
            "[WORKFLOW] Stage receipt generated for {}::{} (root={})",
            workflow_id, stage.name, receipt.merkle_root
        );

        self.emit_event(WorkflowEvent::StageReceiptGenerated {
            workflow_id: workflow_id.to_string(),
            stage_id: stage.name.clone(),
            receipt,
            timestamp: now_iso(),
        });
        // Mark stage as completed
        self.set_stage_state(workflow_id, &stage.name, StageState::Completed);
        Ok(())
    }

    /// Execute tasks sequentially
    fn execute_sequential(
        &self,
        workflow_id: &str,
        stage: &Stage,
        tracker: &mut GoalRunTracker,
    ) -> Result<Vec<Value>, String> {
        let mut artifacts = Vec::with_capacity(stage.tasks.len());
        for task in &stage.tasks {
            artifacts.push(self.execute_task(workflow_id, &stage.name, task, tracker)?);
        }
        Ok(artifacts)
    }

    /// Execute tasks in parallel
    fn execute_parallel(
        &self,
        workflow_id: &str,
        stage: &Stage,
        tracker: &mut GoalRunTracker,
    ) -> Result<Vec<Value>, String> {
        println!(
            "[WORKFLOW] Executing {} tasks in parallel",
            stage.tasks.len()
        );

        // In a real implementation, this would spawn threads/processes
        let mut artifacts = Vec::with_capacity(stage.tasks.len());
        for task in &stage.tasks {
            artifacts.push(self.execute_task(workflow_id, &stage.name, task, tracker)?);
        }

        Ok(artifacts)
    }

    /// Execute tasks conditionally
    fn execute_conditional(
        &self,
        workflow_id: &str,
        stage: &Stage,
        tracker: &mut GoalRunTracker,
    ) -> Result<Vec<Value>, String> {
        // Implementation would check conditions
        self.execute_sequential(workflow_id, stage, tracker)
    }

    /// Execute tasks in a loop
    fn execute_loop(
        &self,
        workflow_id: &str,
        stage: &Stage,
        tracker: &mut GoalRunTracker,
    ) -> Result<Vec<Value>, String> {
        // Implementation would loop based on condition
        self.execute_sequential(workflow_id, stage, tracker)
    }

    /// Execute a single task
    fn execute_task(
        &self,
        workflow_id: &str,
        stage_id: &str,
        task: &Task,
        tracker: &mut GoalRunTracker,
    ) -> Result<Value, String> {
        let approval = self
            .instrumentation
            .evaluate_agent_for_execution(&task.agent);
        if approval.requires_manual_approval {
            tracker.record(&task.agent, false, None, false);
            let reason = approval
                .reason
                .unwrap_or_else(|| "reward score below threshold".to_string());
            return Err(format!(
                "agent '{}' requires manual approval before execution: {}",
                task.agent, reason
            ));
        }

        let token_ratio = extract_token_ratio(&task.parameters);
        let rollback_flag = task_requests_rollback(task);
        let dispatch_receipt = self
            .dispatcher
            .dispatch(task)
            .map_err(|err| {
                println!(
                    "[WORKFLOW] Dispatcher failed for agent {}: {}",
                    task.agent, err
                );
                format!("agent dispatch failed: {}", err)
            })?;
        self.instrumentation
            .log_task_dispatch(workflow_id, stage_id, &dispatch_receipt)
            .map_err(|err| format!("task dispatch instrumentation failed: {}", err))?;

        let resolved_agent = dispatch_receipt.agent_metadata.agent_id.clone();
        let resolved_role = task
            .agent_role
            .clone()
            .unwrap_or_else(|| dispatch_receipt.agent_metadata.role.clone());
        let mut observed_task = task.clone();
        observed_task.agent = resolved_agent.clone();
        if observed_task.agent_role.is_none() {
            observed_task.agent_role = Some(resolved_role.clone());
        }

        let result = (|| {
            println!(
                "[WORKFLOW] Executing task: agent={}, action={}",
                resolved_agent, task.action
            );
            self.observe_task(&observed_task)?;
            println!(
                "[WORKFLOW] Executing task via kernel: agent={}, action={}",
                resolved_agent, task.action
            );

            if let Some(kernel) = &self.kernel {
                if let Ok(process_service) = kernel.request::<ProcessService>(CAPABILITY_PROCESS) {
                    let _ = process_service.create_process(format!("workflow::{}", resolved_agent));
                }

                if let Ok(factory) = kernel.request::<AgentFactory>(AGENT_FACTORY_CAPABILITY) {
                    println!(
                        "[WORKFLOW] Agent factory accessible: {} total agents",
                        factory.list_agents().len()
                    );
                }
            }

            Ok(json!({
                "agent": resolved_agent,
                "agent_role": resolved_role,
                "action": task.action,
                "parameters": parameters_to_value(&task.parameters),
                "status": "completed",
                "timestamp": now_iso(),
            }))
        })();

        tracker.record(&task.agent, result.is_ok(), token_ratio, rollback_flag);
        self.log_task_dispatch(workflow_id, stage_id, task, &result);

        let mut final_result = result;
        if final_result.is_ok() && dispatch_receipt.output != Value::Null {
            final_result = Ok(dispatch_receipt.output.clone());
        }

        let success = final_result.is_ok();
        tracker.record(&resolved_agent, success, token_ratio, rollback_flag);

        let action_lower = task.action.to_lowercase();
        if action_lower.contains("deploy") {
            let mut notes = json!({
                "parameters": parameters_to_value(&task.parameters),
                "tool_receipts": dispatch_receipt.tool_receipts.clone(),
                "output": dispatch_receipt.output.clone(),
            });
            if let Err(err) = &final_result {
                notes["error"] = json!(err);
            }
            let record = DeploymentOutcomeRecord {
                workflow_id: workflow_id.to_string(),
                stage_id: stage_id.to_string(),
                agent_role: resolved_role.clone(),
                agent_id: resolved_agent.clone(),
                action: task.action.clone(),
                status: if success {
                    "succeeded".to_string()
                } else {
                    "failed".to_string()
                },
                notes,
                recorded_at: now_iso(),
            };
            if let Err(err) = self.instrumentation.record_deployment_outcome(record) {
                println!(
                    "[WORKFLOW] Failed to record deployment outcome for {}::{}: {}",
                    workflow_id, stage_id, err
                );
            }
        }

        final_result
    }

    fn log_task_dispatch(
        &self,
        workflow_id: &str,
        stage_id: &str,
        task: &Task,
        result: &Result<Value, String>,
    ) {
        let mut metadata = AgentMetadata::minimal(
            task.agent.clone(),
            format!("Synthetic dispatch for {}", task.action),
            AgentCategory::Other,
        );
        metadata.capabilities = task
            .tool_requirements
            .iter()
            .map(|req| req.capability.clone())
            .collect();
        let tool_receipts: Vec<ToolExecutionReceipt> = task
            .tool_requirements
            .iter()
            .map(|requirement| ToolExecutionReceipt {
                requirement: requirement.clone(),
                status: if requirement.optional {
                    ToolExecutionStatus::Skipped
                } else {
                    ToolExecutionStatus::Succeeded
                },
                output: Value::Null,
                error: None,
            })
            .collect();
        let dispatch_output = result.clone().unwrap_or(Value::Null);
        let receipt = TaskDispatchReceipt {
            agent_metadata: metadata,
            agent_instance_id: format!("synthetic::{}", task.agent),
            task: task.clone(),
            output: dispatch_output,
            tool_receipts,
        };
        if let Err(err) = self
            .instrumentation
            .log_task_dispatch(workflow_id, stage_id, &receipt)
        {
            println!(
                "[WORKFLOW] Failed to log task dispatch for {}::{}: {}",
                workflow_id, stage_id, err
            );
        }
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
            || action_lower.contains("documentation")
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

    fn emit_event(&self, event: WorkflowEvent) {
        if let Some(stream) = self.event_stream.lock().unwrap().clone() {
            stream.send(event);
        }
    }

    /// Set stage state
    fn set_stage_state(&self, workflow_id: &str, stage_name: &str, state: StageState) {
        let state_clone = state.clone();
        {
            let mut stage_states = self.stage_states.lock().unwrap();
            stage_states
                .entry(workflow_id.to_string())
                .or_insert_with(HashMap::new)
                .insert(stage_name.to_string(), state);
        }

        let timestamp = now_iso();
        self.emit_event(WorkflowEvent::StageState {
            workflow_id: workflow_id.to_string(),
            stage_id: stage_name.to_string(),
            state: state_clone.clone(),
            timestamp: timestamp.clone(),
        });

        if state_clone == StageState::Completed {
            let token = WorkflowResumeToken {
                workflow_id: workflow_id.to_string(),
                stage_id: Some(stage_name.to_string()),
                checkpoint: format!("stage://{workflow_id}/{stage_name}"),
                issued_at: timestamp.clone(),
                expires_at: (Utc::now() + Duration::hours(4)).to_rfc3339(),
            };
            self.emit_event(WorkflowEvent::ResumeOffered {
                workflow_id: workflow_id.to_string(),
                token,
                timestamp,
            });
        }
    }

    /// Get workflow state
    pub fn get_state(&self, workflow_id: &str) -> Option<WorkflowState> {
        let states = self.states.lock().unwrap();
        states.get(workflow_id).cloned()
    }
}

fn parameters_to_value(parameters: &HashMap<String, Value>) -> Value {
    serde_json::to_value(parameters).unwrap_or(Value::Null)
}

fn extract_token_ratio(parameters: &HashMap<String, Value>) -> Option<f64> {
    parameters
        .get("token_ratio")
        .and_then(Value::as_f64)
        .or_else(|| {
            match (
                parameters.get("token_usage"),
                parameters.get("token_budget"),
            ) {
                (Some(usage), Some(budget)) => usage
                    .as_f64()
                    .zip(budget.as_f64())
                    .and_then(|(u, b)| if b > 0.0 { Some(u / b) } else { None }),
                _ => parameters.get("token_usage").and_then(Value::as_f64),
            }
        })
        .map(|ratio| if ratio.is_finite() { ratio } else { 1.0 })
}

fn task_requests_rollback(task: &Task) -> bool {
    let action = task.action.to_lowercase();
    if action.contains("rollback") {
        return true;
    }

    for key in ["rollback", "rolled_back", "requires_rollback"] {
        if let Some(value) = task.parameters.get(key) {
            if value.as_bool().unwrap_or(false) {
                return true;
            }
        }
    }

    false
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

impl Default for WorkflowEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use noa_core::security;
    use serde_json::json;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    use crate::instrumentation::{EvidenceLedgerEntry, EvidenceLedgerKind};
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
    fn test_workflow_creation() {
        let workflow = Workflow {
            name: "test".to_string(),
            version: "1.0".to_string(),
            stages: vec![],
        };

        let engine = WorkflowEngine::new();
        let registry = engine.dispatcher.registry();
        let mut metadata = AgentMetadata::minimal(
            "WorkflowVerifier".to_string(),
            "Workflow Verifier".to_string(),
            AgentCategory::Other,
        );
        metadata.capabilities.push("workflow.taskDispatch".to_string());
        registry
            .upsert_metadata(metadata)
            .expect("register workflow verifier agent");
        assert!(
            registry.get("WorkflowVerifier").is_some(),
            "workflow verifier metadata should exist"
        );
        let id = engine.load_workflow(workflow).unwrap();
        assert_eq!(engine.get_state(&id), Some(WorkflowState::Pending));
    }

    #[test]
    fn test_instrumentation_generates_signed_operations() {
        let dir = tempdir().unwrap();
        let _guard = EnvGuard::set("NOA_WORKFLOW_ROOT", dir.path());
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

    #[test]
    fn task_dispatch_events_logged_with_tool_requirements() {
        let dir = tempdir().unwrap();
        let _guard = EnvGuard::set("NOA_WORKFLOW_ROOT", dir.path());
        let engine = WorkflowEngine::new();
        let registry = engine.dispatcher.registry();
        let mut metadata = AgentMetadata::from_registry(
            "WorkflowVerifier".to_string(),
            "WorkflowVerifier".to_string(),
        );
        metadata
            .capabilities
            .push("workflow.taskDispatch".to_string());
        registry
            .upsert_metadata(metadata)
            .expect("stub workflow verifier registration");
        assert!(
            registry.get("WorkflowVerifier").is_some(),
            "workflow verifier not registered"
        );
        let now = Utc::now();
        let fallback_nanos = now.timestamp_micros() * 1_000;
        let workflow_name = format!(
            "dispatch-{}",
            now.timestamp_nanos_opt().unwrap_or(fallback_nanos)
        );
        let workflow = Workflow {
            name: workflow_name.clone(),
            version: "1.0".to_string(),
            stages: vec![Stage {
                name: "dispatch-stage".to_string(),
                stage_type: StageType::Sequential,
                depends_on: vec![],
                tasks: vec![Task {
                    agent: "ModelSelectorAgent".to_string(),
                    action: "evaluate_tools".to_string(),
                    parameters: HashMap::from([(String::from("scope"), json!("tests"))]),
                    agent_role: None,
                    tool_requirements: vec![ToolRequirement {
                        name: "Analysis pass".to_string(),
                        capability: "workflow.taskDispatch".to_string(),
                        optional: false,
                        parameters: json!({"depth": 1}),
                    }],
                }],
            }],
        };

        let id = engine.load_workflow(workflow).unwrap();
        engine.execute(&id).unwrap();

        let log_path = dir
            .path()
            .join(".workspace")
            .join("indexes")
            .join("task_dispatches.log");
        let content = fs::read_to_string(&log_path).expect("dispatch log present");
        let entries: Vec<Value> = content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str(line).expect("log entry should parse"))
            .collect();
        assert!(entries.len() >= 2, "expected genesis and dispatch entries");
        let dispatch_entry = entries
            .iter()
            .rev()
            .find(|entry| {
                entry
                    .get("event")
                    .and_then(|e| e.get("event_type"))
                    == Some(&json!("task.dispatch"))
            })
            .expect("task dispatch entry present");
        let scope = dispatch_entry
            .get("event")
            .and_then(|event| event.get("scope"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        assert_eq!(scope, format!("{}::dispatch-stage", workflow_name));
        let metadata = dispatch_entry
            .get("event")
            .and_then(|event| event.get("metadata"))
            .expect("metadata present on dispatch event");
        let receipts = metadata
            .get("tool_receipts")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        assert_eq!(receipts.len(), 1);
        assert_eq!(
            metadata
                .get("agent")
                .and_then(Value::as_str)
                .unwrap_or_default(),
            "ModelSelectorAgent"
        );
    }

    #[test]
    fn stage_merkle_receipt_is_recorded() {
        let dir = tempdir().unwrap();
        let _guard = EnvGuard::set("NOA_WORKFLOW_ROOT", dir.path());
        let engine = WorkflowEngine::new();
        let now = Utc::now();
        let fallback_nanos = now.timestamp_micros() * 1_000;
        let workflow_name = format!(
            "merkle-{}",
            now.timestamp_nanos_opt().unwrap_or(fallback_nanos)
        );
        let workflow = Workflow {
            name: workflow_name.clone(),
            version: "1.0".to_string(),
            stages: vec![Stage {
                name: "stage-merkle".to_string(),
                stage_type: StageType::Sequential,
                depends_on: vec![],
                tasks: vec![Task {
                    agent: "WorkflowVerifier".to_string(),
                    action: "document".to_string(),
                    parameters: HashMap::from([(String::from("path"), json!("docs/test.md"))]),
                    agent_role: None,
                    tool_requirements: Vec::new(),
                }],
            }],
        };

        let id = engine.load_workflow(workflow).unwrap();
        engine.execute(&id).unwrap();

        let ledger_path = dir
            .path()
            .join("storage")
            .join("db")
            .join("evidence")
            .join("ledger.jsonl");
        let content = fs::read_to_string(&ledger_path)
            .unwrap_or_else(|_| panic!("ledger missing at {:?}", ledger_path));

        let receipt = content
            .lines()
            .filter_map(|line| serde_json::from_str::<EvidenceLedgerEntry>(line).ok())
            .filter(|entry| entry.kind == EvidenceLedgerKind::StageReceipt)
            .find(|entry| {
                entry.payload.get("workflow_id").and_then(Value::as_str)
                    == Some(workflow_name.as_str())
            })
            .expect("stage receipt recorded");

        let stage_id = receipt
            .payload
            .get("stage_id")
            .and_then(Value::as_str)
            .unwrap_or_default();
        assert_eq!(stage_id, "stage-merkle");
        let leaf_count = receipt
            .payload
            .get("leaves")
            .and_then(Value::as_array)
            .map(|array| array.len())
            .unwrap_or(0);
        assert_eq!(leaf_count, 1);
        let merkle_root = receipt.reference.as_str();
        assert!(!merkle_root.is_empty());
    }
}
