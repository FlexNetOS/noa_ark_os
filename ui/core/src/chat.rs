use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

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
}

/// Command descriptor exposed to the AI chat workspace.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatCommandDescriptor {
    pub command: String,
    pub description: String,
    pub examples: Vec<String>,
    pub action: ChatAction,
}

/// Chat workspace orchestrates command execution via the unified shell.
pub struct ChatWorkspace {
    store: GlobalStore,
    workflow_catalog: WorkflowCatalog,
    commands: HashMap<String, ChatCommandDescriptor>,
    event_sink: Arc<dyn Fn(ShellEvent) + Send + Sync>,
}

impl ChatWorkspace {
    pub fn new(
        store: GlobalStore,
        workflow_catalog: WorkflowCatalog,
        event_sink: Arc<dyn Fn(ShellEvent) + Send + Sync>,
    ) -> Self {
        Self {
            store,
            workflow_catalog,
            commands: HashMap::new(),
            event_sink,
        }
    }

    pub fn set_event_sink(&mut self, sink: Arc<dyn Fn(ShellEvent) + Send + Sync>) {
        self.event_sink = sink;
    }

    pub fn register_command(&mut self, descriptor: ChatCommandDescriptor) {
        self.commands.insert(descriptor.command.clone(), descriptor);
    }

    pub fn handle_message(&self, message: &str) -> Option<String> {
        if let Some(command) = self.commands.get(message) {
            match &command.action {
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
                        let mut nav = NavigationState::default();
                        nav.active_route = Some(route.clone());
                        nav.primary_items = state.navigation.primary_items.clone();
                        nav.secondary_items = state.navigation.secondary_items.clone();
                        state.navigation = nav;
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
            }
        }

        None
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
}
