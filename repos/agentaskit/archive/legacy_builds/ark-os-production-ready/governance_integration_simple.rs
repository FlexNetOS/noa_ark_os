//! Simplified governance integration for framework compatibility
//!
//! This module provides a basic governance integration that compiles and works
//! with the orchestration engine while maintaining the interface for future enhancement.

use std::sync::Arc;
use std::time::Duration;

use crate::governance_stubs::{
    ApprovalId, ApprovalRequest, ApprovalStatus, GovernanceController, GovernanceError, Priority,
};
use crate::task::{Task, TaskPriority};

/// Simplified governance-enabled orchestrator
#[derive(Clone)]
pub struct GovernanceEnabledOrchestrator {
    governance: Arc<GovernanceController>,
    agent_name: String,
}

impl GovernanceEnabledOrchestrator {
    /// Create a new governance-enabled orchestrator
    pub fn new(agent_name: String) -> Self {
        Self {
            governance: Arc::new(GovernanceController::new()),
            agent_name,
        }
    }

    /// Check if governance is enabled
    pub fn is_governance_enabled(&self) -> bool {
        self.governance.is_enabled()
    }

    /// Submit a task for approval
    pub async fn submit_task_for_approval(&self, task: &Task) -> Result<ApprovalId, GovernanceError> {
        let request = ApprovalRequest::new(task.id.to_string(), task.definition.name.clone());
        self.governance.submit_for_approval(request).await
    }

    /// Wait for task approval
    pub async fn wait_for_task_approval(
        &self,
        approval_id: ApprovalId,
        timeout: Option<Duration>,
    ) -> Result<bool, GovernanceError> {
        let timeout = timeout.unwrap_or(Duration::from_secs(300));
        let status = self.governance.wait_for_approval(approval_id, timeout).await?;
        Ok(matches!(status, ApprovalStatus::Approved))
    }

    /// Check task approval status
    pub async fn check_task_approval(&self, approval_id: ApprovalId) -> Result<ApprovalStatus, GovernanceError> {
        self.governance.check_status(approval_id).await
    }

    /// Approve a task
    pub async fn approve_task(&self, approval_id: ApprovalId) -> Result<(), GovernanceError> {
        self.governance.approve(approval_id).await
    }

    /// Reject a task
    pub async fn reject_task(&self, approval_id: ApprovalId) -> Result<(), GovernanceError> {
        self.governance.reject(approval_id).await
    }

    /// Get pending approvals
    pub async fn get_pending_approvals(&self) -> Result<Vec<ApprovalRequest>, GovernanceError> {
        self.governance.get_pending_approvals().await
    }
}

/// Simple pending task approval structure
#[derive(Debug, Clone)]
pub struct PendingTaskApproval {
    pub approval_id: ApprovalId,
    pub task_id: String,
    pub description: String,
    pub requester: String,
    pub priority: Priority,
}

/// Governance extensions trait for tasks
pub trait GovernanceExtensions {
    /// Request approval for this task
    async fn request_approval(&self, orchestrator: &GovernanceEnabledOrchestrator) -> Result<ApprovalId, GovernanceError>;
}

impl GovernanceExtensions for Task {
    async fn request_approval(&self, orchestrator: &GovernanceEnabledOrchestrator) -> Result<ApprovalId, GovernanceError> {
        orchestrator.submit_task_for_approval(self).await
    }
}

/// Map task priority to governance priority
fn map_task_priority(task_priority: TaskPriority) -> Priority {
    match task_priority {
        TaskPriority::Low => Priority::Low,
        TaskPriority::Normal => Priority::Normal,
        TaskPriority::High => Priority::High,
        TaskPriority::Critical => Priority::Critical,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::{TaskDefinition, TaskStatus};
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_task() -> Task {
        Task {
            id: Uuid::new_v4(),
            definition: TaskDefinition {
                name: "test_task".to_string(),
                description: "Test task for governance".to_string(),
                category: "test".to_string(),
                tags: vec!["test".to_string()],
                priority: TaskPriority::Normal,
                resources: crate::task::ResourceRequirements::default(),
                dependencies: vec![],
                context: crate::task::ExecutionContext::Shell {
                    command: "echo".to_string(),
                    args: vec!["test".to_string()],
                    working_dir: None,
                    env: std::collections::HashMap::new(),
                },
                metadata: std::collections::HashMap::new(),
                max_retries: 3,
                parallel_safe: true,
            },
            status: TaskStatus::Pending,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            retry_count: 0,
            results: None,
            parent_task: None,
            child_tasks: vec![],
        }
    }

    #[tokio::test]
    async fn test_governance_orchestrator_creation() {
        let orchestrator = GovernanceEnabledOrchestrator::new("test_agent".to_string());
        assert!(orchestrator.is_governance_enabled());
    }

    #[tokio::test]
    async fn test_task_approval_workflow() {
        let orchestrator = GovernanceEnabledOrchestrator::new("test_agent".to_string());
        let task = create_test_task();

        // Submit for approval
        let approval_id = orchestrator.submit_task_for_approval(&task).await.unwrap();

        // Check status
        let status = orchestrator.check_task_approval(approval_id).await.unwrap();
        assert_eq!(status, ApprovalStatus::Pending);

        // Approve the task
        orchestrator.approve_task(approval_id).await.unwrap();

        // Check status again
        let status = orchestrator.check_task_approval(approval_id).await.unwrap();
        assert_eq!(status, ApprovalStatus::Approved);
    }
}
