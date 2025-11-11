use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::chat::{ChatAction, ChatCommandDescriptor};
use crate::events::ShellEvent;
use crate::state::{
    GlobalStore, NavigationItem, Notification, NotificationLevel, Workspace, WorkspacePersona,
};
use crate::workflows::{Workflow, WorkflowCatalog};

/// Describes the capabilities surfaced by a module.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModuleCapability {
    Workflows,
    Agents,
    ContinuousDelivery,
    Sandbox,
    Storage,
    Analytics,
    Chat,
}

/// Static descriptor for shell modules used by navigation and workspace planners.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleDescriptor {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub routes: Vec<String>,
    pub persona: WorkspacePersona,
    pub capabilities: Vec<ModuleCapability>,
}

impl ModuleDescriptor {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        icon: impl Into<String>,
        persona: WorkspacePersona,
        routes: Vec<String>,
        capabilities: Vec<ModuleCapability>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: icon.into(),
            persona,
            routes,
            capabilities,
        }
    }
}

/// Context object handed to modules during hydration and event handling.
#[derive(Clone)]
pub struct ModuleContext {
    pub store: GlobalStore,
    pub workflows: WorkflowCatalog,
    pub emit: Arc<dyn Fn(ShellEvent) + Send + Sync>,
}

impl ModuleContext {
    pub fn register_navigation(&self, descriptor: &ModuleDescriptor) {
        self.store.update(|state| {
            let mut nav = state.navigation.clone();
            nav.primary_items.push(NavigationItem {
                id: descriptor.id.clone(),
                label: descriptor.label.clone(),
                icon: descriptor.icon.clone(),
                route: descriptor
                    .routes
                    .first()
                    .cloned()
                    .unwrap_or_else(|| format!("/{}", descriptor.id)),
            });
            nav.active_route = nav
                .active_route
                .or_else(|| nav.primary_items.first().map(|i| i.route.clone()));
            state.navigation = nav;
        });
    }

    pub fn register_workspace(&self, descriptor: &ModuleDescriptor) {
        self.store.upsert_workspace(Workspace {
            id: descriptor.id.clone(),
            label: descriptor.label.clone(),
            persona: descriptor.persona,
            routes: descriptor.routes.clone(),
        });
    }

    pub fn register_workflows(&self, workflows: &[Workflow]) {
        for workflow in workflows {
            self.workflows.register(workflow.clone());
        }
    }

    pub fn notify(&self, message: impl Into<String>, level: NotificationLevel) {
        self.store
            .push_notification(Notification::new(level, message.into()));
    }
}

/// Trait implemented by each module integrated into the unified shell.
pub trait ShellModule: Send + Sync {
    fn descriptor(&self) -> &ModuleDescriptor;
    fn hydrate(&self, context: &ModuleContext);
    fn handle_event(&self, _event: &ShellEvent, _context: &ModuleContext) {}
    fn chat_commands(&self) -> Vec<ChatCommandDescriptor> {
        vec![]
    }
}

/// Module dedicated to orchestrating workflows.
pub struct WorkflowModule {
    descriptor: ModuleDescriptor,
    workflows: Vec<Workflow>,
}

impl WorkflowModule {
    pub fn new(workflows: Vec<Workflow>) -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "workflow-command-center",
                "Workflow Command Center",
                "workflow",
                WorkspacePersona::Operator,
                vec!["/workflows".into(), "/workflows/history".into()],
                vec![ModuleCapability::Workflows, ModuleCapability::Sandbox],
            ),
            workflows,
        }
    }
}

impl ShellModule for WorkflowModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn hydrate(&self, context: &ModuleContext) {
        context.register_navigation(&self.descriptor);
        context.register_workspace(&self.descriptor);
        context.register_workflows(&self.workflows);
        context.notify(
            "Workflow Command Center ready with unified pipeline telemetry.",
            NotificationLevel::Success,
        );
    }

    fn chat_commands(&self) -> Vec<ChatCommandDescriptor> {
        self.workflows
            .iter()
            .map(|workflow| ChatCommandDescriptor {
                command: format!("run {}", workflow.id),
                description: format!("Trigger the '{}' workflow", workflow.label),
                examples: vec![format!("run {}", workflow.id)],
                action: ChatAction::TriggerWorkflow {
                    workflow_id: workflow.id.clone(),
                    payload: serde_json::json!({"source": "chat"}),
                },
            })
            .collect()
    }
}

/// Module presenting the agent hive status board.
pub struct AgentModule {
    descriptor: ModuleDescriptor,
}

impl AgentModule {
    pub fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "agent-hive",
                "Agent Hive Explorer",
                "users",
                WorkspacePersona::Operator,
                vec!["/agents".into(), "/agents/runtime".into()],
                vec![ModuleCapability::Agents],
            ),
        }
    }
}

impl ShellModule for AgentModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn hydrate(&self, context: &ModuleContext) {
        context.register_navigation(&self.descriptor);
        context.register_workspace(&self.descriptor);
    }

    fn chat_commands(&self) -> Vec<ChatCommandDescriptor> {
        vec![ChatCommandDescriptor {
            command: "show agents".into(),
            description: "Open the Agent Hive Explorer".into(),
            examples: vec!["show agents".into(), "open agent hive".into()],
            action: ChatAction::Navigate {
                route: "/agents".into(),
            },
        }]
    }
}

/// Module providing CI/CD observability and controls.
pub struct CiModule {
    descriptor: ModuleDescriptor,
}

impl CiModule {
    pub fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "ci-console",
                "CI/CD Console",
                "activity",
                WorkspacePersona::Developer,
                vec!["/ci".into(), "/ci/history".into()],
                vec![
                    ModuleCapability::ContinuousDelivery,
                    ModuleCapability::Sandbox,
                ],
            ),
        }
    }
}

impl ShellModule for CiModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn hydrate(&self, context: &ModuleContext) {
        context.register_navigation(&self.descriptor);
        context.register_workspace(&self.descriptor);
        context.notify(
            "CI/CD Console linked with sandbox promotions and rollout health.",
            NotificationLevel::Info,
        );
    }

    fn chat_commands(&self) -> Vec<ChatCommandDescriptor> {
        vec![ChatCommandDescriptor {
            command: "open ci".into(),
            description: "Navigate to the CI/CD Console".into(),
            examples: vec!["open ci".into()],
            action: ChatAction::Navigate {
                route: "/ci".into(),
            },
        }]
    }
}

/// Module surfacing artifact storage and knowledge base.
pub struct StorageModule {
    descriptor: ModuleDescriptor,
}

impl StorageModule {
    pub fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "storage-hub",
                "Storage & Artifact Hub",
                "database",
                WorkspacePersona::Executive,
                vec!["/storage".into(), "/storage/audit".into()],
                vec![ModuleCapability::Storage],
            ),
        }
    }
}

impl ShellModule for StorageModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn hydrate(&self, context: &ModuleContext) {
        context.register_navigation(&self.descriptor);
        context.register_workspace(&self.descriptor);
    }

    fn chat_commands(&self) -> Vec<ChatCommandDescriptor> {
        vec![ChatCommandDescriptor {
            command: "show artifacts".into(),
            description: "Navigate to the Storage & Artifact Hub".into(),
            examples: vec!["show artifacts".into()],
            action: ChatAction::Navigate {
                route: "/storage".into(),
            },
        }]
    }
}

/// Module exposing telemetry and analytics overlays.
pub struct AnalyticsModule {
    descriptor: ModuleDescriptor,
}

impl AnalyticsModule {
    pub fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "analytics",
                "Value Analytics",
                "bar-chart-3",
                WorkspacePersona::Executive,
                vec!["/analytics".into()],
                vec![ModuleCapability::Analytics],
            ),
        }
    }
}

impl ShellModule for AnalyticsModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn hydrate(&self, context: &ModuleContext) {
        context.register_navigation(&self.descriptor);
        context.register_workspace(&self.descriptor);
    }

    fn chat_commands(&self) -> Vec<ChatCommandDescriptor> {
        vec![ChatCommandDescriptor {
            command: "show roi".into(),
            description: "Open Value Analytics to review ROI metrics".into(),
            examples: vec!["show roi".into()],
            action: ChatAction::ExplainFeature {
                feature: "analytics".into(),
            },
        }]
    }
}

/// Module enabling the chat workspace to register itself in navigation.
pub struct ChatModule {
    descriptor: ModuleDescriptor,
}

impl ChatModule {
    pub fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "ai-studio",
                "AI Ops Studio",
                "message-circle",
                WorkspacePersona::Developer,
                vec!["/chat".into()],
                vec![ModuleCapability::Chat, ModuleCapability::Workflows],
            ),
        }
    }
}

impl ShellModule for ChatModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn hydrate(&self, context: &ModuleContext) {
        context.register_navigation(&self.descriptor);
        context.register_workspace(&self.descriptor);
    }

    fn chat_commands(&self) -> Vec<ChatCommandDescriptor> {
        vec![ChatCommandDescriptor {
            command: "open chat".into(),
            description: "Switch to the AI Ops Studio chat workspace".into(),
            examples: vec!["open chat".into(), "return to chat".into()],
            action: ChatAction::Navigate {
                route: "/chat".into(),
            },
        }]
    }
}

/// Helper for bundling the stock modules shipped with the shell.
pub fn default_modules() -> Vec<Arc<dyn ShellModule>> {
    vec![
        Arc::new(ChatModule::new()),
        Arc::new(WorkflowModule::new(vec![
            Workflow::builder("build")
                .label("Build")
                .description("Compile and run tests")
                .with_stage("lint")
                .with_stage("unit-tests")
                .with_stage("package")
                .finish(),
            Workflow::builder("deploy")
                .label("Deploy")
                .description("Promote artifacts across environments")
                .with_stage("stage")
                .with_stage("verify")
                .with_stage("promote")
                .finish(),
        ])),
        Arc::new(AgentModule::new()),
        Arc::new(CiModule::new()),
        Arc::new(StorageModule::new()),
        Arc::new(AnalyticsModule::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GlobalState;

    #[test]
    fn workflow_module_registers_navigation() {
        let store = GlobalStore::new(GlobalState::default());
        let context = ModuleContext {
            store: store.clone(),
            workflows: WorkflowCatalog::default(),
            emit: Arc::new(|_| {}),
        };
        let module =
            WorkflowModule::new(vec![Workflow::builder("test").with_stage("lint").finish()]);
        module.hydrate(&context);

        let state = store.read();
        assert!(!state.navigation.primary_items.is_empty());
        assert!(state.workspaces.contains_key("workflow-command-center"));
    }
}
