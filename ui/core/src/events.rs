use serde::{Deserialize, Serialize};

use crate::workflows::WorkflowRun;

/// Events flowing through the unified shell event bus.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShellEvent {
    ModuleRegistered { module_id: String },
    RouteActivated { route: String },
    WorkflowTriggered { run: WorkflowRun },
    NotificationDismissed { notification_id: String },
    ChatCommandIssued { command: String },
}
