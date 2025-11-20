use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::events::ShellEvent;
use crate::state::{GlobalStore, NavigationState};
use crate::workflows::{WorkflowCatalog, WorkflowRun};

/// Describes a chat action triggered from a command utterance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatAction {
    TriggerWorkflow {
        workflow_id: String,
        payload: serde_json::Value,
    },
    Navigate {
        route: String,
    },
    ExplainFeature {
        feature: String,
    },
    InspectRepository {
        repo: String,
        path: Option<String>,
    },
    LaunchSandbox {
        sandbox_id: String,
        parameters: serde_json::Value,
    },
    StreamLogs {
        stream_id: String,
    },
    ShowDiff {
        artifact_id: String,
    },
    FetchArtifact {
        artifact_id: String,
    },
    SpawnAgent {
        role: String,
        goal: String,
    },
    TriggerQuickAction {
        action_id: String,
    },
}

/// Command descriptor exposed to the AI chat workspace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatCommandDescriptor {
    pub command: String,
    pub description: String,
    pub examples: Vec<String>,
    pub action: ChatAction,
}

/// Describes a one-tap quick action exposed in the chat UI.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuickAction {
    pub id: String,
    pub label: String,
    pub description: String,
    pub command: String,
    pub action: ChatAction,
}

/// Chat workspace orchestrates command execution via the unified shell.
pub struct ChatWorkspace {
    store: GlobalStore,
    workflow_catalog: WorkflowCatalog,
    commands: HashMap<String, ChatCommandDescriptor>,
    quick_actions: HashMap<String, QuickAction>,
    event_sink: Arc<dyn Fn(ShellEvent) + Send + Sync>,
}

impl ChatWorkspace {
    pub fn new(
        store: GlobalStore,
        workflow_catalog: WorkflowCatalog,
        event_sink: Arc<dyn Fn(ShellEvent) + Send + Sync>,
    ) -> Self {
        let mut workspace = Self {
            store,
            workflow_catalog,
            commands: HashMap::new(),
            quick_actions: HashMap::new(),
            event_sink,
        };
        workspace.seed_quick_actions();
        workspace
    }

    pub fn set_event_sink(&mut self, sink: Arc<dyn Fn(ShellEvent) + Send + Sync>) {
        self.event_sink = sink;
    }

    pub fn register_command(&mut self, descriptor: ChatCommandDescriptor) {
        self.commands.insert(descriptor.command.clone(), descriptor);
    }

    pub fn register_quick_action(&mut self, action: QuickAction) {
        let command = action.command.clone();
        let descriptor = ChatCommandDescriptor {
            command: command.clone(),
            description: action.description.clone(),
            examples: vec![command.clone()],
            action: ChatAction::TriggerQuickAction {
                action_id: action.id.clone(),
            },
        };
        self.commands.insert(command, descriptor);
        self.quick_actions.insert(action.id.clone(), action);
    }

    pub fn quick_actions(&self) -> Vec<QuickAction> {
        self.quick_actions.values().cloned().collect()
    }

    pub fn trigger_quick_action(&self, action_id: &str) -> Option<String> {
        self.execute_quick_action(action_id)
    }

    pub fn handle_message(&self, message: &str) -> Option<String> {
        if let Some(command) = self.commands.get(message) {
            if let ChatAction::TriggerQuickAction { action_id } = &command.action {
                return self.execute_quick_action(action_id);
            }

            return self.execute_action(&command.action);
        }

        None
    }

    fn execute_quick_action(&self, action_id: &str) -> Option<String> {
        if let Some(action) = self.quick_actions.get(action_id) {
            (self.event_sink)(ShellEvent::QuickActionTriggered {
                action: action_id.to_string(),
            });
            self.execute_action(&action.action)
        } else {
            None
        }
    }

    fn execute_action(&self, action: &ChatAction) -> Option<String> {
        match action {
            ChatAction::TriggerWorkflow {
                workflow_id,
                payload,
            } => {
                if let Some(workflow) = self.workflow_catalog.get(workflow_id) {
                    let run = WorkflowRun::new(workflow.clone(), payload.clone());
                    (self.event_sink)(ShellEvent::WorkflowTriggered { run });
                    return Some(format!(
                        "Workflow '{}' triggered successfully.",
                        workflow_id
                    ));
                }
            }
            ChatAction::Navigate { route } => {
                self.store.update(|state| {
                    state.navigation = NavigationState {
                        active_route: Some(route.clone()),
                        primary_items: state.navigation.primary_items.clone(),
                        secondary_items: state.navigation.secondary_items.clone(),
                    };
                });
                (self.event_sink)(ShellEvent::RouteActivated {
                    route: route.clone(),
                });
                return Some(format!("Navigated to {}", route));
            }
            ChatAction::ExplainFeature { feature } => {
                return Some(format!(
                    "Here's how '{}' enhances the platform: {}",
                    feature,
                    self.workflow_catalog.feature_explanation(feature)
                ));
            }
            ChatAction::InspectRepository { repo, path } => {
                self.store.put_data(
                    "chat.last_inspection",
                    json!({ "repo": repo, "path": path }),
                );
                (self.event_sink)(ShellEvent::ChatCommandIssued {
                    command: format!("inspect {}", repo),
                });
                return Some(format!(
                    "Inspection request queued for {}{}",
                    repo,
                    path.as_ref()
                        .map(|p| format!(" at {}", p))
                        .unwrap_or_default()
                ));
            }
            ChatAction::LaunchSandbox {
                sandbox_id,
                parameters,
            } => {
                self.store
                    .put_data(format!("chat.sandbox.{}", sandbox_id), parameters.clone());
                (self.event_sink)(ShellEvent::LogStreamUpdate {
                    stream_id: sandbox_id.clone(),
                    line: "Sandbox launch requested".into(),
                });
                return Some(format!("Launching sandbox {}", sandbox_id));
            }
            ChatAction::StreamLogs { stream_id } => {
                self.store.put_data(
                    format!("chat.log_stream.{}", stream_id),
                    json!({"status": "streaming"}),
                );
                (self.event_sink)(ShellEvent::LogStreamUpdate {
                    stream_id: stream_id.clone(),
                    line: "Streaming initiated".into(),
                });
                return Some(format!("Streaming logs for {}", stream_id));
            }
            ChatAction::ShowDiff { artifact_id } => {
                self.store
                    .put_data("chat.last_diff", json!({"artifact": artifact_id}));
                (self.event_sink)(ShellEvent::DiffAvailable {
                    artifact_id: artifact_id.clone(),
                    summary: "Diff ready for review".into(),
                });
                return Some(format!("Diff ready for artifact {}", artifact_id));
            }
            ChatAction::FetchArtifact { artifact_id } => {
                (self.event_sink)(ShellEvent::ArtifactReady {
                    artifact_id: artifact_id.clone(),
                    url: format!("/api/storage/artifacts/{}", artifact_id),
                });
                return Some(format!("Artifact {} prepared for download", artifact_id));
            }
            ChatAction::SpawnAgent { role, goal } => {
                self.store
                    .put_data(format!("chat.agent_goal.{}", role), json!({"goal": goal}));
                (self.event_sink)(ShellEvent::AgentSpawned {
                    agent_id: format!("{}-agent", role),
                    role: role.clone(),
                });
                return Some(format!("Spawning {} agent to {}", role, goal));
            }
            ChatAction::TriggerQuickAction { .. } => {
                // handled earlier to avoid recursion.
            }
        }

        None
    }

    fn seed_quick_actions(&mut self) {
        self.register_quick_action(QuickAction {
            id: "continue-development".into(),
            label: "Continue Development".into(),
            description: "Resume the last build workflow".into(),
            command: "continue development".into(),
            action: ChatAction::TriggerWorkflow {
                workflow_id: "build".into(),
                payload: json!({"resume": true}),
            },
        });
        self.register_quick_action(QuickAction {
            id: "open-suggestions".into(),
            label: "Open Code Suggestions".into(),
            description: "Inspect repository suggestions".into(),
            command: "open suggestions".into(),
            action: ChatAction::InspectRepository {
                repo: "core".into(),
                path: Some("src".into()),
            },
        });
        self.register_quick_action(QuickAction {
            id: "trigger-automated-fix".into(),
            label: "Trigger Automated Fix".into(),
            description: "Launch automated remediation sandbox".into(),
            command: "trigger automated fix".into(),
            action: ChatAction::LaunchSandbox {
                sandbox_id: "auto-fix".into(),
                parameters: json!({"strategy": "repair"}),
            },
        });
        self.register_quick_action(QuickAction {
            id: "spawn-agent".into(),
            label: "Spawn Specialist Agent".into(),
            description: "Create a fixer agent for the current task".into(),
            command: "spawn repair agent".into(),
            action: ChatAction::SpawnAgent {
                role: "fixer".into(),
                goal: "resolve failing tests".into(),
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::state::{GlobalState, NavigationItem};
    use crate::workflows::Workflow;

    #[test]
    fn trigger_workflow_via_chat_command() {
        let store = GlobalStore::new(GlobalState::default());
        let catalog = WorkflowCatalog::default();
        catalog.register(Workflow::builder("build").with_stage("lint").finish());
        let sink_events = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        let sink_clone = sink_events.clone();
        let mut chat = ChatWorkspace::new(
            store,
            catalog,
            Arc::new(move |event| sink_clone.lock().unwrap().push(event)),
        );

        chat.register_command(ChatCommandDescriptor {
            command: "run build".into(),
            description: "Run CI pipeline".into(),
            examples: vec!["run build".into()],
            action: ChatAction::TriggerWorkflow {
                workflow_id: "build".into(),
                payload: json!({"branch": "main"}),
            },
        });

        let response = chat.handle_message("run build");
        assert_eq!(
            response,
            Some("Workflow 'build' triggered successfully.".into())
        );
        assert_eq!(
            sink_events.lock().unwrap().len(),
            1,
            "expected WorkflowTriggered event"
        );
    }

    #[test]
    fn navigation_command_updates_state() {
        let mut state = GlobalState::default();
        state.navigation.primary_items.push(NavigationItem {
            id: "chat".into(),
            label: "Chat".into(),
            icon: "message-square".into(),
            route: "/chat".into(),
            allowed_roles: vec![],
        });
        let store = GlobalStore::new(state);
        let catalog = WorkflowCatalog::default();
        let captured = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        {
            let sink = captured.clone();
            let mut chat = ChatWorkspace::new(
                store.clone(),
                catalog,
                Arc::new(move |event| sink.lock().unwrap().push(event)),
            );
            chat.register_command(ChatCommandDescriptor {
                command: "go chat".into(),
                description: "Navigate to chat".into(),
                examples: vec!["go chat".into()],
                action: ChatAction::Navigate {
                    route: "/chat".into(),
                },
            });

            let response = chat.handle_message("go chat");
            assert_eq!(response, Some("Navigated to /chat".into()));
        }

        let state = store.read();
        assert_eq!(state.navigation.active_route, Some("/chat".into()));
        assert_eq!(captured.lock().unwrap().len(), 1);
    }

    #[test]
    fn quick_action_spawns_agent() {
        let store = GlobalStore::new(GlobalState::default());
        let catalog = WorkflowCatalog::default();
        let captured = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
        let sink = captured.clone();
        let chat = ChatWorkspace::new(
            store,
            catalog,
            Arc::new(move |event| sink.lock().unwrap().push(event)),
        );

        let response = chat.trigger_quick_action("spawn-agent");
        assert!(response.unwrap().contains("Spawning fixer agent"));

        let events = captured.lock().unwrap();
        assert!(events.iter().any(|event| matches!(
            event,
            ShellEvent::AgentSpawned { agent_id, .. } if agent_id == "fixer-agent"
        )));
    }
}
