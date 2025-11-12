use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;

/// Represents the persona-driven workspace groupings that
/// drive contextual layouts in the unified shell.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WorkspacePersona {
    Operator,
    Developer,
    Executive,
}

impl Default for WorkspacePersona {
    fn default() -> Self {
        WorkspacePersona::Developer
    }
}

/// Navigation items rendered in the global navigation rail.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NavigationItem {
    pub id: String,
    pub label: String,
    pub icon: String,
    pub route: String,
    pub allowed_roles: Vec<String>,
}

/// Container for navigation state shared across modules.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NavigationState {
    pub primary_items: Vec<NavigationItem>,
    pub secondary_items: Vec<NavigationItem>,
    pub active_route: Option<String>,
}

/// Workspace definitions that group modules and dashboards by persona.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub label: String,
    pub persona: WorkspacePersona,
    pub routes: Vec<String>,
    pub allowed_roles: Vec<String>,
}

impl Workspace {
    pub fn contains_route(&self, route: &str) -> bool {
        self.routes.iter().any(|r| r == route)
    }
}

/// Represents knowledge base entries surfaced in the contextual overlay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeArticle {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub link: String,
}

/// Notification severity levels for the shell notification center.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Notification model surfaced to users across the unified shell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub level: NotificationLevel,
    pub message: String,
    pub timestamp: u64,
}

impl Notification {
    pub fn new(level: NotificationLevel, message: impl Into<String>) -> Self {
        let id = format!("{}-{}", level as u8, uuid());
        Self {
            id,
            level,
            message: message.into(),
            timestamp: unix_time(),
        }
    }
}

/// User session and contextual metadata consumed by every module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: String,
    pub display_name: String,
    pub roles: Vec<String>,
    pub active_workspace: Option<String>,
    pub auth_token: Option<String>,
}

impl Default for UserSession {
    fn default() -> Self {
        Self {
            user_id: "anonymous".into(),
            display_name: "Guest".into(),
            roles: vec![],
            active_workspace: None,
            auth_token: None,
        }
    }
}

/// Global state aggregated by the unified shell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalState {
    pub session: UserSession,
    pub navigation: NavigationState,
    pub workspaces: HashMap<String, Workspace>,
    pub notifications: Vec<Notification>,
    pub data: HashMap<String, serde_json::Value>,
    pub knowledge_base: HashMap<WorkspacePersona, Vec<KnowledgeArticle>>,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
}

/// Notification model surfaced to users across the unified shell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub level: NotificationLevel,
    pub message: String,
    pub timestamp: u64,
}

impl Notification {
    pub fn new(level: NotificationLevel, message: impl Into<String>) -> Self {
        let id = format!("{}-{}", level as u8, uuid());
        Self {
            id,
            level,
            message: message.into(),
            timestamp: unix_time(),
        }
    }
}

/// User session and contextual metadata consumed by every module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: String,
    pub display_name: String,
    pub roles: Vec<String>,
    pub active_workspace: Option<String>,
    pub auth_token: Option<String>,
}

impl Default for UserSession {
    fn default() -> Self {
        Self {
            user_id: "anonymous".into(),
            display_name: "Guest".into(),
            roles: vec![],
            active_workspace: None,
            auth_token: None,
        }
    }
}

/// Global state aggregated by the unified shell.
#[derive(Debug, Clone)]
pub struct GlobalState {
    pub session: UserSession,
    pub navigation: NavigationState,
    pub workspaces: HashMap<String, Workspace>,
    pub notifications: Vec<Notification>,
    pub data: HashMap<String, serde_json::Value>,
}

impl Default for GlobalState {
    fn default() -> Self {
        Self {
            session: UserSession::default(),
            navigation: NavigationState::default(),
            workspaces: HashMap::new(),
            notifications: vec![],
            data: HashMap::new(),
            knowledge_base: HashMap::new(),
        }
    }
}

/// Thread-safe wrapper around [`GlobalState`].
#[derive(Clone)]
pub struct GlobalStore {
    inner: Arc<RwLock<GlobalState>>,
}

impl GlobalStore {
    pub fn new(state: GlobalState) -> Self {
        Self {
            inner: Arc::new(RwLock::new(state)),
        }
    }

    pub fn global() -> &'static GlobalStore {
        static STORE: Lazy<GlobalStore> = Lazy::new(|| GlobalStore::new(GlobalState::default()));
        &STORE
    }

    pub fn read(&self) -> GlobalState {
        self.inner.read().expect("global state poisoned").clone()
    }

    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut GlobalState),
    {
        let mut guard = self.inner.write().expect("global state poisoned");
        f(&mut guard);
    }

    pub fn upsert_workspace(&self, workspace: Workspace) {
        self.update(|state| {
            state.workspaces.insert(workspace.id.clone(), workspace);
        });
    }

    pub fn push_notification(&self, notification: Notification) {
        self.update(|state| {
            state.notifications.push(notification);
        });
    }

    pub fn set_navigation(&self, nav: NavigationState) {
        self.update(|state| state.navigation = nav);
    }

    pub fn put_data(&self, key: impl Into<String>, value: serde_json::Value) {
        self.update(|state| {
            state.data.insert(key.into(), value);
        });
    }

    pub fn persist_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let snapshot = self.read();
        let json = serde_json::to_string_pretty(&snapshot)
            .unwrap_or_else(|_| serde_json::json!({}).to_string());
        fs::create_dir_all(
            path.as_ref()
                .parent()
                .unwrap_or_else(|| Path::new(".")),
        )?;
        fs::write(path, json)
    }

    pub fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<(), std::io::Error> {
        let contents = fs::read_to_string(path)?;
        let parsed: GlobalState = serde_json::from_str(&contents).unwrap_or_default();
        self.update(|state| {
            *state = parsed;
        });
        Ok(())
    pub fn set_active_workspace(&self, workspace_id: impl Into<String>) {
        let id = workspace_id.into();
        self.update(|state| {
            if state.workspaces.contains_key(&id) {
                state.session.active_workspace = Some(id.clone());
            }
        });
    }

    pub fn set_knowledge_base(&self, persona: WorkspacePersona, articles: Vec<KnowledgeArticle>) {
        self.update(|state| {
            state.knowledge_base.insert(persona, articles);
        });
    }

    pub fn knowledge_for(&self, persona: WorkspacePersona) -> Vec<KnowledgeArticle> {
        self.read()
            .knowledge_base
            .get(&persona)
            .cloned()
            .unwrap_or_default()
    }
}

fn unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn uuid() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("{:016x}", id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notification_id_is_unique() {
        let a = Notification::new(NotificationLevel::Info, "hello");
        let b = Notification::new(NotificationLevel::Info, "hello");
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn workspace_upsert_is_thread_safe() {
        let store = GlobalStore::new(GlobalState::default());
        let workspace = Workspace {
            id: "dev".into(),
            label: "Developer".into(),
            persona: WorkspacePersona::Developer,
            routes: vec!["/chat".into()],
            allowed_roles: vec!["developer".into()],
        };

        store.upsert_workspace(workspace.clone());
        let state = store.read();
        assert!(state.workspaces.get("dev").is_some());
        assert!(state.workspaces.get("dev").unwrap().contains_route("/chat"));
    }

    #[test]
    fn persist_and_restore_global_state() {
        let store = GlobalStore::new(GlobalState::default());
        store.update(|state| {
            state.session.user_id = "persist-user".into();
            state.data.insert("key".into(), serde_json::json!({"value": 42}));
        });

        let path = std::env::temp_dir().join("noa_state_test.json");
        store.persist_to_file(&path).expect("persist state");

        let restored = GlobalStore::new(GlobalState::default());
        restored
            .load_from_file(&path)
            .expect("load persisted state");

        let snapshot = restored.read();
        assert_eq!(snapshot.session.user_id, "persist-user");
        assert_eq!(snapshot.data.get("key").unwrap()["value"], 42);

        let _ = std::fs::remove_file(path);
    }
}
