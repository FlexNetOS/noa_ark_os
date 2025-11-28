use std::sync::Arc;

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

use crate::chat::{ChatAction, ChatCommandDescriptor};
use crate::events::ShellEvent;
use crate::services::ShellServices;
use crate::state::{GlobalStore, NavigationItem, NotificationLevel, Workspace, WorkspacePersona};
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

/// Describes how a module should be mounted within the shell.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModuleMount {
    InternalComponent { name: String },
    StaticAsset { base_path: String, entry: String },
    ExternalUrl { url: String },
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
    pub mount: ModuleMount,
    pub allowed_roles: Vec<String>,
}

impl ModuleDescriptor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        icon: impl Into<String>,
        persona: WorkspacePersona,
        routes: Vec<String>,
        capabilities: Vec<ModuleCapability>,
        mount: ModuleMount,
        allowed_roles: Vec<String>,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: icon.into(),
            persona,
            routes,
            capabilities,
            mount,
            allowed_roles,
        }
    }
}

/// Context object handed to modules during hydration and event handling.
#[derive(Clone)]
pub struct ModuleContext {
    pub store: GlobalStore,
    pub workflows: WorkflowCatalog,
    pub emit: Arc<dyn Fn(ShellEvent) + Send + Sync>,
    pub services: ShellServices,
}

impl ModuleContext {
    pub fn register_navigation(&self, descriptor: &ModuleDescriptor) {
        if !self.module_accessible(descriptor) {
            return;
        }

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
                allowed_roles: descriptor.allowed_roles.clone(),
            });
            nav.active_route = nav
                .active_route
                .or_else(|| nav.primary_items.first().map(|i| i.route.clone()));
            state.navigation = nav;
        });
    }

    pub fn register_workspace(&self, descriptor: &ModuleDescriptor) {
        if !self.module_accessible(descriptor) {
            return;
        }

        self.store.upsert_workspace(Workspace {
            id: descriptor.id.clone(),
            label: descriptor.label.clone(),
            persona: descriptor.persona,
            routes: descriptor.routes.clone(),
            allowed_roles: descriptor.allowed_roles.clone(),
        });
    }

    pub fn register_workflows(&self, workflows: &[Workflow]) {
        for workflow in workflows {
            self.workflows.register(workflow.clone());
        }
    }

    pub fn notify(&self, message: impl Into<String>, level: NotificationLevel) {
        self.services.notify(message, level);
    }

    pub fn services(&self) -> ShellServices {
        self.services.clone()
    }
}

/// Wrapper that defers module hydration until first use.
pub struct LazyModule {
    descriptor: ModuleDescriptor,
    loader: Arc<dyn Fn() -> Arc<dyn ShellModule> + Send + Sync>,
    instance: OnceCell<Arc<dyn ShellModule>>,
}

impl LazyModule {
    pub fn new(
        descriptor: ModuleDescriptor,
        loader: Arc<dyn Fn() -> Arc<dyn ShellModule> + Send + Sync>,
    ) -> Self {
        Self {
            descriptor,
            loader,
            instance: OnceCell::new(),
        }
    }

    fn ensure_loaded(&self) -> Arc<dyn ShellModule> {
        self.instance.get_or_init(|| (self.loader)()).clone()
    }
}

impl ShellModule for LazyModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn hydrate(&self, context: &ModuleContext) {
        self.ensure_loaded().hydrate(context);
    }

    fn handle_event(&self, event: &ShellEvent, context: &ModuleContext) {
        self.ensure_loaded().handle_event(event, context);
    }

    fn chat_commands(&self) -> Vec<ChatCommandDescriptor> {
        self.ensure_loaded().chat_commands()
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
                ModuleMount::InternalComponent {
                    name: "WorkflowCommandCenter".into(),
                },
                vec!["operator".into(), "admin".into()],
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
                ModuleMount::InternalComponent {
                    name: "AgentHiveExplorer".into(),
                },
                vec!["operator".into(), "admin".into()],
            ),
        }
    }
}

impl Default for AgentModule {
    fn default() -> Self {
        Self::new()
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

/// Module bridging the legacy dashboard into the unified shell.
pub struct DashboardModule {
    descriptor: ModuleDescriptor,
}

impl DashboardModule {
    pub fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "noa-dashboard",
                "NOA Dashboard",
                "layout-dashboard",
                WorkspacePersona::Executive,
                vec!["/dashboard".into()],
                vec![ModuleCapability::Analytics, ModuleCapability::Workflows],
                ModuleMount::StaticAsset {
                    base_path: "ui/noa-dashboard".into(),
                    entry: "index.html".into(),
                },
                vec!["executive".into(), "admin".into()],
            ),
        }
    }
}

impl Default for DashboardModule {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellModule for DashboardModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn hydrate(&self, context: &ModuleContext) {
        context.register_navigation(&self.descriptor);
        context.register_workspace(&self.descriptor);
        context.services().notify(
            "NOA Dashboard available as a lazy-mounted surface.",
            NotificationLevel::Success,
        );
    }
}

/// Module wrapping the Vibe Kanban Next.js experience.
pub struct KanbanModule {
    descriptor: ModuleDescriptor,
}

impl KanbanModule {
    pub fn new() -> Self {
        Self {
            descriptor: ModuleDescriptor::new(
                "vibe-kanban",
                "Vibe Kanban",
                "columns",
                WorkspacePersona::Developer,
                vec!["/kanban".into()],
                vec![ModuleCapability::Workflows, ModuleCapability::Agents],
                ModuleMount::StaticAsset {
                    base_path: "ui/vibe-kanban".into(),
                    entry: "app".into(),
                },
                vec!["developer".into(), "operator".into(), "admin".into()],
            ),
        }
    }
}

impl Default for KanbanModule {
    fn default() -> Self {
        Self::new()
    }
}

impl ShellModule for KanbanModule {
    fn descriptor(&self) -> &ModuleDescriptor {
        &self.descriptor
    }

    fn hydrate(&self, context: &ModuleContext) {
        context.register_navigation(&self.descriptor);
        context.register_workspace(&self.descriptor);
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
                ModuleMount::InternalComponent {
                    name: "CiConsole".into(),
                },
                vec!["developer".into(), "admin".into()],
            ),
        }
    }
}

impl Default for CiModule {
    fn default() -> Self {
        Self::new()
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
                ModuleMount::InternalComponent {
                    name: "StorageHub".into(),
                },
                vec!["executive".into(), "admin".into()],
            ),
        }
    }
}

impl Default for StorageModule {
    fn default() -> Self {
        Self::new()
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
                ModuleMount::InternalComponent {
                    name: "ValueAnalytics".into(),
                },
                vec!["executive".into(), "admin".into()],
            ),
        }
    }
}

impl Default for AnalyticsModule {
    fn default() -> Self {
        Self::new()
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
                ModuleMount::InternalComponent {
                    name: "AiOpsStudio".into(),
                },
                vec!["developer".into(), "operator".into(), "admin".into()],
            ),
        }
    }
}

impl Default for ChatModule {
    fn default() -> Self {
        Self::new()
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
    fn wrap(module: Arc<dyn ShellModule>) -> Arc<dyn ShellModule> {
        let descriptor = module.descriptor().clone();
        let loader: Arc<dyn Fn() -> Arc<dyn ShellModule> + Send + Sync> = Arc::new({
            let module = module.clone();
            move || module.clone()
        });
        let lazy: Arc<dyn ShellModule> = Arc::new(LazyModule::new(descriptor, loader));
        lazy
    }

    let modules: Vec<Arc<dyn ShellModule>> = vec![
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
        Arc::new(DashboardModule::new()),
        Arc::new(KanbanModule::new()),
        Arc::new(CiModule::new()),
        Arc::new(StorageModule::new()),
        Arc::new(AnalyticsModule::new()),
    ];

    modules.into_iter().map(wrap).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::ShellEvent;
    use crate::services::use_shell_services;
    use crate::state::GlobalState;

    #[test]
    fn workflow_module_registers_navigation() {
        let store = GlobalStore::new(GlobalState::default());
        let sink: Arc<dyn Fn(ShellEvent) + Send + Sync> = Arc::new(|_| {});
        let context = ModuleContext {
            store: store.clone(),
            workflows: WorkflowCatalog::default(),
            emit: sink.clone(),
            services: use_shell_services(&store, sink.clone()),
        };
        let module =
            WorkflowModule::new(vec![Workflow::builder("test").with_stage("lint").finish()]);
        store.update(|state| {
            state.session.roles = vec!["operator".into()];
        });
        module.hydrate(&context);

        let state = store.read();
        assert!(!state.navigation.primary_items.is_empty());
        assert!(state.workspaces.contains_key("workflow-command-center"));
    }

    #[test]
    fn executive_modules_require_role() {
        let store = GlobalStore::new(GlobalState::default());
        store.update(|state| {
            state.session.roles = vec!["developer".into()];
        });
        let sink: Arc<dyn Fn(ShellEvent) + Send + Sync> = Arc::new(|_| {});
        let context = ModuleContext {
            store: store.clone(),
            workflows: WorkflowCatalog::default(),
            emit: sink.clone(),
            services: use_shell_services(&store, sink),
        };
        let analytics = AnalyticsModule::new();
        analytics.hydrate(&context);

        let state = store.read();
        assert!(state
            .navigation
            .primary_items
            .iter()
            .all(|item| item.id != "analytics"));
        assert!(!state.workspaces.contains_key("analytics"));
    }
}

impl ModuleContext {
    fn module_accessible(&self, descriptor: &ModuleDescriptor) -> bool {
        let state = self.store.read();
        if descriptor.allowed_roles.is_empty() {
            return true;
        }

        let session_roles = &state.session.roles;
        descriptor
            .allowed_roles
            .iter()
            .any(|role| session_roles.iter().any(|r| r == role))
    }
}
