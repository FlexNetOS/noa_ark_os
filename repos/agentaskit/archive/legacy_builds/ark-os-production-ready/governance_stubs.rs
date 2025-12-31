//! Governance stubs to replace noa_core dependencies
//! These are simplified versions for framework integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Approval identifier
pub type ApprovalId = Uuid;

/// Approval request for governance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: ApprovalId,
    pub task_id: String,
    pub description: String,
    pub priority: Priority,
    pub requested_by: String,
}

/// Approval status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

/// Priority levels for governance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

/// Governance errors
#[derive(Debug, thiserror::Error)]
pub enum GovernanceError {
    #[error("Approval not found: {0}")]
    ApprovalNotFound(ApprovalId),
    #[error("Approval expired: {0}")]
    ApprovalExpired(ApprovalId),
    #[error("Approval rejected: {0}")]
    ApprovalRejected(ApprovalId),
    #[error("Governance system error: {0}")]
    SystemError(String),
}

/// Governance controller for managing approvals
#[derive(Debug, Clone)]
pub struct GovernanceController {
    approvals: Arc<RwLock<HashMap<ApprovalId, (ApprovalRequest, ApprovalStatus)>>>,
}

impl GovernanceController {
    /// Create a new governance controller
    pub fn new() -> Self {
        Self {
            approvals: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Request approval for a task
    pub async fn request_approval(&self, request: ApprovalRequest) -> Result<ApprovalId, GovernanceError> {
        let id = request.id;
        let mut approvals = self.approvals.write().await;
        approvals.insert(id, (request, ApprovalStatus::Pending));
        Ok(id)
    }

    /// Check approval status
    pub async fn check_approval(&self, id: ApprovalId) -> Result<ApprovalStatus, GovernanceError> {
        let approvals = self.approvals.read().await;
        match approvals.get(&id) {
            Some((_, status)) => Ok(status.clone()),
            None => Err(GovernanceError::ApprovalNotFound(id)),
        }
    }

    /// Approve a request
    pub async fn approve(&self, id: ApprovalId) -> Result<(), GovernanceError> {
        let mut approvals = self.approvals.write().await;
        match approvals.get_mut(&id) {
            Some((_, status)) => {
                *status = ApprovalStatus::Approved;
                Ok(())
            }
            None => Err(GovernanceError::ApprovalNotFound(id)),
        }
    }

    /// Reject a request
    pub async fn reject(&self, id: ApprovalId) -> Result<(), GovernanceError> {
        let mut approvals = self.approvals.write().await;
        match approvals.get_mut(&id) {
            Some((_, status)) => {
                *status = ApprovalStatus::Rejected;
                Ok(())
            }
            None => Err(GovernanceError::ApprovalNotFound(id)),
        }
    }

    /// Check if governance is enabled
    pub fn is_enabled(&self) -> bool {
        true // Always enabled for this stub
    }

    /// Submit a request for approval
    pub async fn submit_for_approval(&self, request: ApprovalRequest) -> Result<ApprovalId, GovernanceError> {
        self.request_approval(request).await
    }

    /// Wait for approval with timeout
    pub async fn wait_for_approval(&self, id: ApprovalId, _timeout: std::time::Duration) -> Result<ApprovalStatus, GovernanceError> {
        self.check_approval(id).await
    }

    /// Check status (alias for check_approval)
    pub async fn check_status(&self, id: ApprovalId) -> Result<ApprovalStatus, GovernanceError> {
        self.check_approval(id).await
    }

    /// Get pending approvals
    pub async fn get_pending_approvals(&self) -> Result<Vec<ApprovalRequest>, GovernanceError> {
        let approvals = self.approvals.read().await;
        let pending: Vec<ApprovalRequest> = approvals
            .values()
            .filter(|(_, status)| *status == ApprovalStatus::Pending)
            .map(|(request, _)| request.clone())
            .collect();
        Ok(pending)
    }
}

impl ApprovalRequest {
    /// Create a new approval request
    pub fn new(task_id: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id,
            description,
            priority: Priority::Normal,
            requested_by: "system".to_string(),
        }
    }
}

impl Default for GovernanceController {
    fn default() -> Self {
        Self::new()
    }
}
