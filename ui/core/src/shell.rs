use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::adapters::{
    PlatformAdapter, ReactAdapter, ReactNativeAdapter, ServerAdapter, SpatialAdapter, TauriAdapter,
};
use crate::analytics::{
    AgentEfficiency, AnalyticsEngine, HeatmapPoint, Metric, ModelRoi, TelemetryInsights,
};
use crate::chat::ChatWorkspace;
use crate::components::{KnowledgeOverlay, NavigationRail, ShellChrome, WorkspaceSwitcher};
use crate::events::ShellEvent;
use crate::module::{default_modules, ModuleContext, ShellModule};
use crate::renderer::renderer::Renderer;
use crate::renderer::RenderFrame;
use crate::services::ShellServices;
use crate::state::{GlobalState, GlobalStore, KnowledgeArticle, UserSession, WorkspacePersona};
use crate::workflows::WorkflowCatalog;
use crate::{init, Platform, UIContext, UIState};

/// Builder to configure a [`UnifiedShell`].
pub struct ShellBuilder {
    platform: Platform,
    modules: Option<Vec<Arc<dyn ShellModule>>>,
    session: Option<UserSession>,
}

impl ShellBuilder {
    pub fn new(platform: Platform) -> Self {
        Self {
            platform,
            modules: None,
            session: None,
        }
    }

    pub fn with_modules(mut self, modules: Vec<Arc<dyn ShellModule>>) -> Self {
        self.modules = Some(modules);
        self
    }

    pub fn with_session(mut self, session: UserSession) -> Self {
        self.session = Some(session);
        self
    }

    pub fn build(self) -> Result<UnifiedShell, &'static str> {
        UnifiedShell::new(
            self.platform,
            self.modules.unwrap_or_else(default_modules),
            self.session,
        )
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
    services: ShellServices,
}

impl UnifiedShell {
    fn new(
        platform: Platform,
        modules: Vec<Arc<dyn ShellModule>>,
        session: Option<UserSession>,
    ) -> Result<Self, &'static str> {
        let context = init(platform.clone())?;
        let renderer = Renderer::new(context.clone());
        let state = UIState::new(context.clone());
        // Each shell instance holds its own store to avoid test interference when run in parallel.
        let store = GlobalStore::new(GlobalState::default());

        if let Some(session) = session {
            store.update(|state| state.session = session);
        } else {
            store.update(|state| {
                state.session = UserSession {
                    user_id: "ops-admin".into(),
                    display_name: "Ops Admin".into(),
                    roles: vec![
                        "admin".into(),
                        "developer".into(),
                        "operator".into(),
                        "executive".into(),
                    ],
                    active_workspace: None,
                    auth_token: None,
                };
            });
        }

        let workflow_catalog = WorkflowCatalog::default();
        let event_log: Arc<Mutex<Vec<ShellEvent>>> = Arc::new(Mutex::new(vec![]));
        let event_sink = {
            let log = event_log.clone();
            Arc::new(move |event: ShellEvent| {
                log.lock().unwrap().push(event);
            })
        };

        let services = ShellServices::new(store.clone(), event_sink.clone());
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
            services,
        };

        shell.bootstrap_modules(event_sink);
        shell.seed_analytics();
        shell.seed_knowledge_base();

        Ok(shell)
    }

    fn bootstrap_modules(&self, event_sink: Arc<dyn Fn(ShellEvent) + Send + Sync>) {
        let context = ModuleContext {
            store: self.store.clone(),
            workflows: self.workflow_catalog.clone(),
            emit: event_sink.clone(),
            services: self.services.clone(),
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

        self.store.update(|state| {
            state
                .navigation
                .primary_items
                .sort_by(|a, b| a.label.cmp(&b.label));
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
        analytics.ingest(Metric {
            id: "agent_efficiency_index".into(),
            label: "Agent Efficiency Index".into(),
            value: 87.0,
            unit: "score".into(),
        });
        analytics.layer_insights(TelemetryInsights {
            usage_heatmap: vec![HeatmapPoint {
                area: "ai-studio.command-canvas".into(),
                intensity: 0.78,
            }],
            agent_efficiency: vec![AgentEfficiency {
                agent_id: "deploy-coordinator".into(),
                utilization: 0.92,
                impact_score: 8.8,
            }],
            model_roi: vec![ModelRoi {
                model: "ops-orchestrator".into(),
                generated_value: 128_400.0,
                operational_cost: 36_500.0,
            }],
        });
        analytics.sync_to_state(&self.store);
    }

    fn seed_knowledge_base(&self) {
        let developer_articles = vec![
            KnowledgeArticle {
                id: "dev-workflow".into(),
                title: "Ship workflows from chat".into(),
                summary: "Use the AI Ops Studio to run builds, tests, and deployments without leaving the conversation.".into(),
                link: "docs/workflows/ai-ops-studio.md".into(),
            },
            KnowledgeArticle {
                id: "dev-sandboxes".into(),
                title: "Validate in sandboxes".into(),
                summary: "Trigger sandbox validations and monitor status with real-time telemetry widgets.".into(),
                link: "docs/sandboxes/overview.md".into(),
            },
        ];

        let operator_articles = vec![
            KnowledgeArticle {
                id: "ops-incident".into(),
                title: "Handle incidents with workflow overrides".into(),
                summary: "Learn how to pause, resume, or reroute workflows directly from the command center.".into(),
                link: "docs/workflows/incident-response.md".into(),
            },
            KnowledgeArticle {
                id: "ops-agents".into(),
                title: "Scale the agent hive".into(),
                summary: "Best practices for scaling or draining agents during peak load.".into(),
                link: "docs/agents/scaling.md".into(),
            },
        ];

        let executive_articles = vec![
            KnowledgeArticle {
                id: "exec-roi".into(),
                title: "Interpreting ROI analytics".into(),
                summary: "Understand how value analytics converts telemetry into business impact dashboards.".into(),
                link: "docs/analytics/roi.md".into(),
            },
            KnowledgeArticle {
                id: "exec-governance".into(),
                title: "Storage governance".into(),
                summary: "Review retention policies and compliance guardrails enforced by the artifact hub.".into(),
                link: "docs/storage/governance.md".into(),
            },
        ];

        self.store
            .set_knowledge_base(WorkspacePersona::Developer, developer_articles);
        self.store
            .set_knowledge_base(WorkspacePersona::Operator, operator_articles);
        self.store
            .set_knowledge_base(WorkspacePersona::Executive, executive_articles);
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
                    services: self.services.clone(),
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
        let state_snapshot = self.store.read();
        let session_roles = state_snapshot.session.roles.clone();
        let mut nav_items: Vec<_> = state_snapshot
            .navigation
            .primary_items
            .into_iter()
            .filter(|item| {
                item.allowed_roles.is_empty()
                    || item
                        .allowed_roles
                        .iter()
                        .any(|role| session_roles.iter().any(|r| r == role))
            })
            .collect();
        nav_items.sort_by(|a, b| a.label.cmp(&b.label));

        let mut active_route = state_snapshot
            .navigation
            .active_route
            .clone()
            .filter(|route| nav_items.iter().any(|item| &item.route == route));
        if active_route.is_none() {
            active_route = nav_items.first().map(|item| item.route.clone());
        }

        let mut workspaces: Vec<_> = state_snapshot
            .workspaces
            .values()
            .filter(|workspace| {
                workspace.allowed_roles.is_empty()
                    || workspace
                        .allowed_roles
                        .iter()
                        .any(|role| session_roles.iter().any(|r| r == role))
            })
            .cloned()
            .collect();
        workspaces.sort_by(|a, b| a.label.cmp(&b.label));

        let mut active_workspace = state_snapshot
            .session
            .active_workspace
            .clone()
            .filter(|id| workspaces.iter().any(|workspace| &workspace.id == id));
        if active_workspace.is_none() {
            active_workspace = workspaces.first().map(|workspace| workspace.id.clone());
        }

        if let Some(active) = &active_workspace {
            self.store.set_active_workspace(active.clone());
        }

        self.store.update(|state| {
            state.navigation.active_route = active_route.clone();
        });

        let persona = active_workspace
            .as_ref()
            .and_then(|id| workspaces.iter().find(|workspace| &workspace.id == id))
            .map(|workspace| workspace.persona)
            .unwrap_or(WorkspacePersona::Developer);

        let knowledge_articles = self.store.knowledge_for(persona);

        let navigation = NavigationRail::new(nav_items, active_route.clone());
        let workspace_switcher = WorkspaceSwitcher::new(workspaces, active_workspace.clone());
        let knowledge = KnowledgeOverlay::new(persona, knowledge_articles);
        let chrome = ShellChrome::new(navigation, workspace_switcher, knowledge);
        adapter.mount(&self.renderer, &chrome, &self.state)?;
        let frame = RenderFrame { chrome: &chrome };
        self.renderer.render_frame(&frame)
    }

    pub fn recommended_adapter(&self) -> Box<dyn PlatformAdapter> {
        match self.context.platform {
            Platform::Server => Box::new(ServerAdapter),
            Platform::Desktop => Box::new(TauriAdapter::default()),
            Platform::Web => Box::new(ReactAdapter),
            Platform::Mobile => Box::new(ReactNativeAdapter::default()),
            Platform::ARGlasses | Platform::XRHeadset => Box::new(SpatialAdapter::default()),
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

    pub fn services(&self) -> ShellServices {
        self.services.clone()
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

    #[test]
    fn persona_specific_navigation_filters_modules() {
        let session = UserSession {
            user_id: "dev-user".into(),
            display_name: "Dev".into(),
            roles: vec!["developer".into()],
            active_workspace: None,
            auth_token: None,
        };
        let shell = UnifiedShell::builder(Platform::Web)
            .with_session(session)
            .build()
            .unwrap();
        let state = shell.store.read();
        assert!(state
            .navigation
            .primary_items
            .iter()
            .all(|item| item.allowed_roles.is_empty()
                || item.allowed_roles.contains(&"developer".into())));
        assert!(state
            .workspaces
            .values()
            .all(|workspace| workspace.allowed_roles.is_empty()
                || workspace.allowed_roles.contains(&"developer".into())));
    }

    #[test]
    fn desktop_adapter_mounts_tauri_manifest() {
        let shell = UnifiedShell::builder(Platform::Desktop).build().unwrap();
        let adapter = shell.recommended_adapter();
        shell
            .render(adapter.as_ref())
            .expect("desktop adapter mounts tauri config");
    }
}
