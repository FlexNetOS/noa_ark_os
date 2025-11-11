use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::adapters::{PlatformAdapter, ReactAdapter, ServerAdapter, SpatialAdapter};
use crate::analytics::{AnalyticsEngine, Metric};
use crate::chat::ChatWorkspace;
use crate::components::{NavigationRail, ShellChrome, WorkspaceSwitcher};
use crate::events::ShellEvent;
use crate::module::{default_modules, ModuleContext, ShellModule};
use crate::renderer::renderer::Renderer;
use crate::renderer::RenderFrame;
use crate::state::{GlobalState, GlobalStore, UserSession, WorkspacePersona};
use crate::workflows::WorkflowCatalog;
use crate::{init, Platform, UIContext, UIState};

/// Builder to configure a [`UnifiedShell`].
pub struct ShellBuilder {
    platform: Platform,
    modules: Option<Vec<Arc<dyn ShellModule>>>,
}

impl ShellBuilder {
    pub fn new(platform: Platform) -> Self {
        Self {
            platform,
            modules: None,
        }
    }

    pub fn with_modules(mut self, modules: Vec<Arc<dyn ShellModule>>) -> Self {
        self.modules = Some(modules);
        self
    }

    pub fn build(self) -> Result<UnifiedShell, &'static str> {
        UnifiedShell::new(self.platform, self.modules.unwrap_or_else(default_modules))
    }
}

/// Core orchestrator that wires modules, chat workspace, analytics, and renderers.
pub struct UnifiedShell {
    context: UIContext,
    renderer: Renderer,
    state: UIState,
    store: GlobalStore,
    modules: Vec<Arc<dyn ShellModule>>,
    workflow_catalog: WorkflowCatalog,
    chat_workspace: Arc<Mutex<ChatWorkspace>>,
    analytics: Mutex<AnalyticsEngine>,
    event_log: Arc<Mutex<Vec<ShellEvent>>>,
}

impl UnifiedShell {
    fn new(platform: Platform, modules: Vec<Arc<dyn ShellModule>>) -> Result<Self, &'static str> {
        let context = init(platform.clone())?;
        let renderer = Renderer::new(context.clone());
        let state = UIState::new(context.clone());
        let store = GlobalStore::new(GlobalState {
            session: UserSession {
                user_id: "ops-admin".into(),
                display_name: "Ops Admin".into(),
                roles: vec!["admin".into(), "developer".into()],
                active_workspace: None,
                auth_token: None,
            },
            ..GlobalState::default()
        });
        let workflow_catalog = WorkflowCatalog::default();
        let event_log: Arc<Mutex<Vec<ShellEvent>>> = Arc::new(Mutex::new(vec![]));
        let event_sink = {
            let log = event_log.clone();
            Arc::new(move |event: ShellEvent| {
                log.lock().unwrap().push(event);
            })
        };

        let chat_workspace = Arc::new(Mutex::new(ChatWorkspace::new(
            store.clone(),
            workflow_catalog.clone(),
            event_sink.clone(),
        )));

        let shell = Self {
            context,
            renderer,
            state,
            store,
            modules,
            workflow_catalog,
            chat_workspace,
            analytics: Mutex::new(AnalyticsEngine::default()),
            event_log,
        };

        shell.bootstrap_modules(event_sink);
        shell.seed_analytics();

        Ok(shell)
    }

    fn bootstrap_modules(&self, event_sink: Arc<dyn Fn(ShellEvent) + Send + Sync>) {
        let context = ModuleContext {
            store: self.store.clone(),
            workflows: self.workflow_catalog.clone(),
            emit: event_sink,
        };

        let mut commands = vec![];

        for module in &self.modules {
            module.hydrate(&context);
            commands.extend(module.chat_commands());
        }

        if let Ok(mut chat) = self.chat_workspace.lock() {
            for command in commands {
                chat.register_command(command);
            }

            // Provide fallback command to resume work.
            chat.register_command(crate::chat::ChatCommandDescriptor {
                command: "continue work".into(),
                description: "Continue the last development workflow".into(),
                examples: vec!["continue work".into()],
                action: crate::chat::ChatAction::TriggerWorkflow {
                    workflow_id: "build".into(),
                    payload: serde_json::json!({"resume": true}),
                },
            });
        }

        // Ensure navigation sorted by persona priority.
        self.store.update(|state| {
            state
                .navigation
                .primary_items
                .sort_by_key(|item| item.label.clone());
            if state.session.active_workspace.is_none() {
                state.session.active_workspace = state.workspaces.keys().next().cloned();
            }
        });
    }

    fn seed_analytics(&self) {
        let mut analytics = self.analytics.lock().unwrap();
        analytics.ingest(Metric {
            id: "developer_productivity".into(),
            label: "Developer Productivity".into(),
            value: 132.0,
            unit: "story points/week".into(),
        });
        analytics.ingest(Metric {
            id: "infrastructure_cost".into(),
            label: "Infrastructure Cost".into(),
            value: 44.0,
            unit: "credits/week".into(),
        });
        analytics.sync_to_state(&self.store);
    }

    pub fn builder(platform: Platform) -> ShellBuilder {
        ShellBuilder::new(platform)
    }

    pub fn emit(&self, event: ShellEvent) {
        self.event_log.lock().unwrap().push(event.clone());
        for module in &self.modules {
            module.handle_event(
                &event,
                &ModuleContext {
                    store: self.store.clone(),
                    workflows: self.workflow_catalog.clone(),
                    emit: Arc::new(|_| {}),
                },
            );
        }
    }

    fn drain_events(&self) -> Vec<ShellEvent> {
        let mut log = self.event_log.lock().unwrap();
        let events = log.clone();
        log.clear();
        events
    }

    pub fn handle_chat_message(&self, message: &str) -> Option<String> {
        let response = self
            .chat_workspace
            .lock()
            .ok()
            .and_then(|chat| chat.handle_message(message));

        for event in self.drain_events() {
            self.emit(event);
        }

        response
    }

    pub fn render(&self, adapter: &dyn PlatformAdapter) -> Result<(), String> {
        let state = self.store.read();
        let navigation = NavigationRail::from_state(&state.navigation);
        let workspaces = WorkspaceSwitcher::new(state.workspaces.values().cloned().collect());
        let chrome = ShellChrome::new(navigation, workspaces);
        let frame = RenderFrame { chrome: &chrome };
        adapter.mount(&self.renderer, &chrome, &self.state);
        self.renderer.render_frame(&frame)
    }

    pub fn recommended_adapter(&self) -> Box<dyn PlatformAdapter> {
        match self.context.platform {
            Platform::Server => Box::new(ServerAdapter),
            Platform::Desktop | Platform::Web | Platform::Mobile => Box::new(ReactAdapter),
            Platform::ARGlasses | Platform::XRHeadset => Box::new(SpatialAdapter),
        }
    }

    pub fn export_state(&self) -> HashMap<String, serde_json::Value> {
        self.store.read().data.clone()
    }

    pub fn personas(&self) -> Vec<WorkspacePersona> {
        let state = self.store.read();
        state
            .workspaces
            .values()
            .map(|workspace| workspace.persona)
            .collect()
    }

    pub fn context(&self) -> &UIContext {
        &self.context
    }

    pub fn store_handle(&self) -> GlobalStore {
        self.store.clone()
    }
}

impl Default for UnifiedShell {
    fn default() -> Self {
        Self::builder(Platform::Web)
            .build()
            .expect("default unified shell")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unified_shell_registers_default_modules() {
        let shell = UnifiedShell::builder(Platform::Web).build().unwrap();
        let state = shell.store.read();
        assert!(!state.navigation.primary_items.is_empty());
        assert!(shell.workflow_catalog.get("build").is_some());
    }

    #[test]
    fn chat_command_triggers_workflow() {
        let shell = UnifiedShell::builder(Platform::Web).build().unwrap();
        let response = shell.handle_chat_message("run build");
        assert_eq!(
            response,
            Some("Workflow 'build' triggered successfully.".into())
        );
    }
}
