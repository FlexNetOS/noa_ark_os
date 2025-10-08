use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::AgentRegistry;

/// Represents a task that can be executed by an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: Uuid,
    pub task_type: AgentTaskType,
    pub description: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub assigned_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<TaskResult>,
}

/// Types of tasks supported by the orchestrator.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AgentTaskType {
    Conversation,
    TaskManagement,
    CodeGeneration,
    CodeAnalysis,
    Scheduling,
    Learning,
    Monitoring,
}

/// Priority levels for tasks.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Lifecycle status of a task.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Completed,
    Failed,
}

/// Result payload for executed tasks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub error_message: Option<String>,
}

/// Errors produced by the orchestrator.
#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("task {0} not found")]
    TaskNotFound(Uuid),
    #[error("no handler registered for task type {0:?}")]
    MissingHandler(AgentTaskType),
    #[error("task execution failed: {0}")]
    ExecutionFailed(String),
}

/// High-level task orchestrator that assigns work to agents and simulates execution.
///
/// The original CRC drop depended on several external services. For the Phase 1
/// integration we keep the scheduling semantics but simulate execution results so
/// the orchestrator can be exercised in tests without requiring additional crates.
pub struct AgentOrchestrator {
    registry: Arc<RwLock<AgentRegistry>>,
    tasks: Arc<RwLock<HashMap<Uuid, AgentTask>>>,
}

impl AgentOrchestrator {
    /// Create a new orchestrator with an empty `AgentRegistry`.
    pub fn new() -> Self {
        Self::with_registry(Arc::new(RwLock::new(AgentRegistry::new())))
    }

    /// Create an orchestrator backed by an existing registry.
    pub fn with_registry(registry: Arc<RwLock<AgentRegistry>>) -> Self {
        Self {
            registry,
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Submit a new task for execution.
    pub async fn submit_task(
        &self,
        task_type: AgentTaskType,
        description: impl Into<String>,
        priority: TaskPriority,
    ) -> Result<Uuid, OrchestratorError> {
        let task = AgentTask {
            id: Uuid::new_v4(),
            task_type,
            description: description.into(),
            priority,
            status: TaskStatus::Pending,
            assigned_agent: None,
            created_at: Utc::now(),
            completed_at: None,
            result: None,
        };

        let task_id = task.id;
        self.tasks.write().await.insert(task_id, task);

        self.assign_task(task_id).await?;
        Ok(task_id)
    }

    /// Retrieve a copy of a task by id.
    pub async fn get_task(&self, task_id: Uuid) -> Option<AgentTask> {
        self.tasks.read().await.get(&task_id).cloned()
    }

    /// Return all tasks managed by the orchestrator.
    pub async fn list_tasks(&self) -> Vec<AgentTask> {
        self.tasks.read().await.values().cloned().collect()
    }

    /// Expose the underlying registry for callers that need it.
    pub fn registry(&self) -> Arc<RwLock<AgentRegistry>> {
        self.registry.clone()
    }

    async fn assign_task(&self, task_id: Uuid) -> Result<(), OrchestratorError> {
        {
            let mut tasks = self.tasks.write().await;
            let task = tasks
                .get_mut(&task_id)
                .ok_or(OrchestratorError::TaskNotFound(task_id))?;

            let agent_id = Self::default_agent_for(task.task_type);
            task.assigned_agent = Some(agent_id.to_string());
            task.status = TaskStatus::Assigned;
        }

        self.execute_task(task_id).await
    }

    async fn execute_task(&self, task_id: Uuid) -> Result<(), OrchestratorError> {
        let mut tasks = self.tasks.write().await;
        let task = tasks
            .get_mut(&task_id)
            .ok_or(OrchestratorError::TaskNotFound(task_id))?;

        task.status = TaskStatus::InProgress;

        // Simulate execution. In a later integration phase this will delegate to
        // specialised agents/services imported from the AgentAsKit drop.
        let execution = Self::simulate_execution(task.task_type, &task.description);

        match execution {
            Ok(result) => {
                task.status = TaskStatus::Completed;
                task.completed_at = Some(Utc::now());
                task.result = Some(result);
                Ok(())
            }
            Err(err) => {
                task.status = TaskStatus::Failed;
                task.result = Some(TaskResult {
                    success: false,
                    data: json!({ "task_id": task.id }),
                    error_message: Some(err.clone()),
                });
                Err(OrchestratorError::ExecutionFailed(err))
            }
        }
    }

    fn default_agent_for(task_type: AgentTaskType) -> &'static str {
        match task_type {
            AgentTaskType::Conversation => "agent.conversation.coordinator",
            AgentTaskType::TaskManagement => "agent.personal.assistant",
            AgentTaskType::CodeGeneration => "agent.dev.codegen",
            AgentTaskType::CodeAnalysis => "agent.dev.analysis",
            AgentTaskType::Scheduling => "agent.personal.scheduler",
            AgentTaskType::Learning => "agent.learning.curator",
            AgentTaskType::Monitoring => "agent.observability.monitor",
        }
    }

    fn simulate_execution(
        task_type: AgentTaskType,
        description: &str,
    ) -> Result<TaskResult, String> {
        match task_type {
            AgentTaskType::Conversation => Ok(TaskResult {
                success: true,
                data: json!({
                    "summary": "Conversation completed",
                    "transcript_id": Uuid::new_v4(),
                    "notes": description,
                }),
                error_message: None,
            }),
            AgentTaskType::TaskManagement | AgentTaskType::Scheduling => Ok(TaskResult {
                success: true,
                data: json!({
                    "schedule_id": Uuid::new_v4(),
                    "actions": ["review", "notify"],
                    "notes": description,
                }),
                error_message: None,
            }),
            AgentTaskType::CodeGeneration | AgentTaskType::CodeAnalysis => Ok(TaskResult {
                success: true,
                data: json!({
                    "analysis": "Code task completed",
                    "diff_id": Uuid::new_v4(),
                    "summary": description,
                }),
                error_message: None,
            }),
            AgentTaskType::Learning => Ok(TaskResult {
                success: true,
                data: json!({
                    "curriculum_id": Uuid::new_v4(),
                    "outcome": "Learning module executed",
                }),
                error_message: None,
            }),
            AgentTaskType::Monitoring => Ok(TaskResult {
                success: true,
                data: json!({
                    "incident_report": false,
                    "telemetry_id": Uuid::new_v4(),
                }),
                error_message: None,
            }),
        }
    }
}

impl Default for AgentOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn submit_and_complete_task() {
        let orchestrator = AgentOrchestrator::new();

        let task_id = orchestrator
            .submit_task(
                AgentTaskType::Conversation,
                "Route customer request",
                TaskPriority::High,
            )
            .await
            .expect("submit task");

        let task = orchestrator.get_task(task_id).await.expect("task exists");
        assert_eq!(task.status, TaskStatus::Completed);
        assert!(task.result.as_ref().unwrap().success);
        assert!(task.assigned_agent.is_some());
    }

    #[tokio::test]
    async fn list_tasks_returns_all_entries() {
        let orchestrator = AgentOrchestrator::new();

        orchestrator
            .submit_task(
                AgentTaskType::CodeGeneration,
                "Generate service skeleton",
                TaskPriority::Medium,
            )
            .await
            .unwrap();

        orchestrator
            .submit_task(
                AgentTaskType::Monitoring,
                "Check telemetry stream",
                TaskPriority::Low,
            )
            .await
            .unwrap();

        let tasks = orchestrator.list_tasks().await;
        assert_eq!(tasks.len(), 2);
    }
}
