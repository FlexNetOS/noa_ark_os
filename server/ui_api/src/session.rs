use chrono::{Duration, Utc};
use noa_workflow::{WorkflowEvent, WorkflowEventStream, WorkflowResumeToken};
use serde_json::json;
use tokio_stream::wrappers::BroadcastStream;

use crate::schema::{RealTimeEvent, ResumeToken};

#[derive(Clone)]
pub struct SessionBridge {
    stream: WorkflowEventStream,
}

impl SessionBridge {
    pub fn new(stream: WorkflowEventStream) -> Self {
        Self { stream }
    }

    pub fn subscribe(&self) -> BroadcastStream<WorkflowEvent> {
        BroadcastStream::new(self.stream.subscribe())
    }

    pub fn latest_resume_token(&self) -> Option<ResumeToken> {
        // Subscribe and immediately drop; receiver receives no events yet.
        // This is a placeholder for future persistence integration.
        None
    }
}

impl SessionBridge {
    pub fn map_event(event: WorkflowEvent) -> RealTimeEvent {
        match event {
            WorkflowEvent::WorkflowState {
                workflow_id,
                state,
                timestamp,
            } => RealTimeEvent {
                event_type: "workflow/state".into(),
                workflow_id,
                payload: json!({
                    "state": state,
                }),
                timestamp,
            },
            WorkflowEvent::StageState {
                workflow_id,
                stage_id,
                state,
                timestamp,
            } => RealTimeEvent {
                event_type: "workflow/stage".into(),
                workflow_id,
                payload: json!({
                    "stage_id": stage_id,
                    "state": state,
                }),
                timestamp,
            },
            WorkflowEvent::ResumeOffered {
                workflow_id,
                token,
                timestamp,
            } => RealTimeEvent {
                event_type: "workflow/resume".into(),
                workflow_id,
                payload: json!({
                    "resumeToken": ResumeToken::from(token.clone()),
                }),
                timestamp,
            },
        }
    }
}

impl From<WorkflowResumeToken> for ResumeToken {
    fn from(token: WorkflowResumeToken) -> Self {
        Self {
            workflow_id: token.workflow_id,
            stage_id: token.stage_id,
            checkpoint: token.checkpoint,
            issued_at: token.issued_at,
            expires_at: token.expires_at,
        }
    }
}

pub fn default_resume_token(workflow_id: &str, stage_id: &str) -> ResumeToken {
    ResumeToken {
        workflow_id: workflow_id.to_string(),
        stage_id: Some(stage_id.to_string()),
        checkpoint: format!("stage://{workflow_id}/{stage_id}"),
        issued_at: Utc::now().to_rfc3339(),
        expires_at: (Utc::now() + Duration::hours(4)).to_rfc3339(),
    }
}
