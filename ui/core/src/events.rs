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
    LogStreamUpdate { stream_id: String, line: String },
    DiffAvailable { artifact_id: String, summary: String },
    ArtifactReady { artifact_id: String, url: String },
    AgentSpawned { agent_id: String, role: String },
    QuickActionTriggered { action: String },
}

/// Lightweight client that resolves WebSocket endpoints for shell channels.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubscriptionClient {
    base_url: String,
    channels: Vec<String>,
}

impl SubscriptionClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            channels: vec![],
        }
    }

    pub fn with_channel(mut self, channel: impl Into<String>) -> Self {
        self.channels.push(channel.into());
        self
    }

    pub fn channels(&self) -> &[String] {
        &self.channels
    }

    pub fn endpoint_for(&self, channel: &str) -> String {
        format!("{}/ws/{}", self.base_url.trim_end_matches('/'), channel)
    }

    pub fn endpoints(&self) -> Vec<String> {
        if self.channels.is_empty() {
            vec![self.endpoint_for("events")]
        } else {
            self.channels
                .iter()
                .map(|channel| self.endpoint_for(channel))
                .collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subscription_client_builds_endpoints() {
        let client = SubscriptionClient::new("http://localhost:8000")
            .with_channel("workflows")
            .with_channel("agents");

        assert_eq!(
            client.endpoints(),
            vec![
                "http://localhost:8000/ws/workflows".to_string(),
                "http://localhost:8000/ws/agents".to_string()
            ]
        );
    }
}
