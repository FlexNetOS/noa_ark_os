//! Governance Integration for NOA Orchestration
//!
//! This module integrates the governance approval system with the orchestration engine,
//! allowing tasks to require approval before execution when manual governance is enabled.

use std::sync::Arc;
use std::time::Duration;

// Governance integration using local stubs
use crate::governance_stubs::{
    ApprovalId, ApprovalRequest, ApprovalStatus, GovernanceController, GovernanceError, Priority,
};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::task::{Task, TaskPriority};

/// Wrapper that adds governance checks to task execution
#[derive(Clone)]
pub struct GovernanceEnabledOrchestrator {
    governance: Arc<GovernanceController>,
    agent_name: String,
}

impl GovernanceEnabledOrchestrator {
    /// Creates a new governance-enabled orchestrator
    pub fn new(config: Arc<NoaConfig>, agent_name: impl Into<String>) -> Self {
        Self {
            governance: Arc::new(GovernanceController::new(config)),
            agent_name: agent_name.into(),
        }
    }

    /// Returns true if governance is currently enabled
    pub fn is_governance_enabled(&self) -> bool {
        self.governance.is_enabled()
    }

    /// Submits a task for approval before execution
    pub async fn submit_task_for_approval(&self, task: &Task) -> Result<ApprovalId, GovernanceError> {
        let request = ApprovalRequest::new(task.id.to_string(), task.definition.name.clone());

        // Submit the request to the governance system
        self.governance.submit_for_approval(request).await
    }

    /// Waits for task approval with optional timeout
    pub async fn wait_for_task_approval(
        &self,
        approval_id: &ApprovalId,
        timeout: Option<Duration>,
    ) -> Result<bool, GovernanceError> {
        let timeout = timeout.unwrap_or(Duration::from_secs(300)); // Default 5 minutes
        let status = self.governance.wait_for_approval(*approval_id, timeout).await?;

        Ok(matches!(status, ApprovalStatus::Approved))
    }

    /// Checks if a task has been approved without blocking
    pub async fn check_task_approval(&self, approval_id: &ApprovalId) -> Result<ApprovalStatus, GovernanceError> {
        self.governance.check_status(*approval_id).await
    }

    /// Gets all pending task approvals for supervisor review
    pub async fn get_pending_task_approvals(&self) -> Result<Vec<PendingTaskApproval>, GovernanceError> {
        let pending = self.governance.get_pending_approvals().await?;

        Ok(pending
            .into_iter()
            .map(|p| PendingTaskApproval {
                approval_id: p.id,
                task_id: p.request.task_id,
                task_name: p.request.task_name,
                task_type: p.request.task_type,
                description: p.request.description,
                requester: p.request.requester,
                priority: p.request.priority,
                submitted_at: p.submitted_at,
            })
            .collect())
    }

    /// Orchestrator agent approves a task
    pub async fn approve_task(
        &self,
        approval_id: &ApprovalId,
        _reason: Option<String>,
    ) -> Result<(), GovernanceError> {
        self.governance.approve(*approval_id).await
    }

    /// Orchestrator agent rejects a task
    pub async fn reject_task(
        &self,
        approval_id: &ApprovalId,
        _reason: Option<String>,
    ) -> Result<(), GovernanceError> {
        self.governance.reject(*approval_id).await
    }

    /// Batch approve multiple tasks
    pub async fn batch_approve_tasks(
        &self,
        approval_ids: &[ApprovalId],
        _reason: Option<String>,
    ) -> Result<Vec<Result<(), GovernanceError>>, GovernanceError> {
        let mut results = Vec::new();
        for &id in approval_ids {
            results.push(self.governance.approve(id).await);
        }
        Ok(results)
    }

    /// Get the underlying governance controller (for advanced use)
    pub fn governance_controller(&self) -> &Arc<GovernanceController> {
        &self.governance
    }
}

/// Simplified view of a pending task approval for orchestrator UI
#[derive(Debug, Clone)]
pub struct PendingTaskApproval {
    pub approval_id: ApprovalId,
    pub task_id: String,
    pub task_name: String,
    pub task_type: String,
    pub description: String,
    pub requester: String,
    pub priority: Priority,
    pub submitted_at: std::time::SystemTime,
}

/// Map orchestration task priority to governance priority
fn map_task_priority(task_priority: TaskPriority) -> Priority {
    match task_priority {
        TaskPriority::Low => Priority::Low,
        TaskPriority::Normal => Priority::Normal,
        TaskPriority::High => Priority::High,
        TaskPriority::Critical => Priority::Critical,
    }
}

/// Extension trait for Task to add governance integration
pub trait GovernanceExtensions {
    /// Request approval for this task
    fn request_approval(
        &self,
        orchestrator: &GovernanceEnabledOrchestrator,
    ) -> Result<ApprovalId, GovernanceError>;

    /// Check if this task requires approval based on its properties
    fn requires_approval(&self) -> bool;
}

impl GovernanceExtensions for Task {
    fn request_approval(
        &self,
        orchestrator: &GovernanceEnabledOrchestrator,
    ) -> Result<ApprovalId, GovernanceError> {
        orchestrator.request_task_approval(self)
    }

    fn requires_approval(&self) -> bool {
        // By default, all tasks require approval when governance is enabled
        // This can be customized based on task properties

        // Example: Critical priority tasks always need approval
        if matches!(self.definition.priority, TaskPriority::Critical) {
            return true;
        }

        // Example: Tasks with certain tags might require approval
        if self.definition.tags.contains(&"sensitive".to_string())
            || self.definition.tags.contains(&"production".to_string())
        {
            return true;
        }

        // Default: require approval for all tasks when governance is enabled
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{ExecutionContext, ResourceRequirements, TaskDefinition};
    use std::collections::HashMap;

    fn test_config_with_governance() -> Arc<NoaConfig> {
        let config = NoaConfig::load().unwrap();
        Arc::new(config)
    }

    fn create_test_task(name: &str, priority: TaskPriority) -> Task {
        let definition = TaskDefinition {
            name: name.to_string(),
            description: "Test task for governance".to_string(),
            category: "test".to_string(),
            tags: vec!["test".to_string()],
            priority,
            resources: ResourceRequirements::default(),
            dependencies: vec![],
            context: ExecutionContext::Shell {
                command: "echo".to_string(),
                args: vec!["test".to_string()],
                working_dir: None,
                env: HashMap::new(),
            },
            metadata: HashMap::new(),
            max_retries: 0,
            parallel_safe: true,
        };
        Task::new(definition)
    }

    #[test]
    fn governance_integration_works() {
        let config = test_config_with_governance();
        let orchestrator = GovernanceEnabledOrchestrator::new(config, "TestOrchestrator");

        let task = create_test_task("test_task", TaskPriority::Normal);

        // Request approval (will auto-approve if governance disabled)
        let approval_id = orchestrator.request_task_approval(&task).unwrap();

        // Check status
        let status = orchestrator.check_task_approval(&approval_id).unwrap();

        // If governance is disabled, should be auto-approved
        if !orchestrator.is_governance_enabled() {
            assert_eq!(status, ApprovalStatus::Approved);
        }
    }

    #[test]
    fn task_requires_approval_for_critical_priority() {
        let task = create_test_task("critical_task", TaskPriority::Critical);
        assert!(task.requires_approval());
    }

    #[test]
    fn task_requires_approval_for_sensitive_tags() {
        let mut task = create_test_task("sensitive_task", TaskPriority::Normal);
        task.definition.tags.push("sensitive".to_string());
        assert!(task.requires_approval());
    }

    #[test]
    fn orchestrator_can_approve_task() {
        let config = test_config_with_governance();
        let orchestrator = GovernanceEnabledOrchestrator::new(config.clone(), "OrchestratorAgent");

        // Only test if governance is enabled
        if !orchestrator.is_governance_enabled() {
            return;
        }

        let task = create_test_task("test_task", TaskPriority::Normal);
        let approval_id = orchestrator.request_task_approval(&task).unwrap();

        // Approve the task
        orchestrator
            .approve_task(&approval_id, Some("Approved by test".to_string()))
            .unwrap();

        // Verify approval
        let status = orchestrator.check_task_approval(&approval_id).unwrap();
        assert_eq!(status, ApprovalStatus::Approved);
    }

    #[test]
    fn orchestrator_can_reject_task() {
        let config = test_config_with_governance();
        let orchestrator = GovernanceEnabledOrchestrator::new(config.clone(), "OrchestratorAgent");

        // Only test if governance is enabled
        if !orchestrator.is_governance_enabled() {
            return;
        }

        let task = create_test_task("test_task", TaskPriority::Normal);
        let approval_id = orchestrator.request_task_approval(&task).unwrap();

        // Reject the task
        orchestrator
            .reject_task(&approval_id, Some("Rejected by test".to_string()))
            .unwrap();

        // Verify rejection
        let status = orchestrator.check_task_approval(&approval_id).unwrap();
        assert_eq!(status, ApprovalStatus::Rejected);
    }

    #[test]
    fn batch_approval_works() {
        let config = test_config_with_governance();
        let orchestrator = GovernanceEnabledOrchestrator::new(config.clone(), "OrchestratorAgent");

        // Only test if governance is enabled
        if !orchestrator.is_governance_enabled() {
            return;
        }

        let task1 = create_test_task("task1", TaskPriority::Normal);
        let task2 = create_test_task("task2", TaskPriority::Normal);

        let id1 = orchestrator.request_task_approval(&task1).unwrap();
        let id2 = orchestrator.request_task_approval(&task2).unwrap();

        // Batch approve
        let results = orchestrator
            .batch_approve_tasks(&[id1.clone(), id2.clone()], None)
            .unwrap();

        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.is_ok()));
    }
}
