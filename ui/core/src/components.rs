use crate::state::{KnowledgeArticle, NavigationItem, Workspace, WorkspacePersona};

/// Declarative navigation rail component representation.
pub struct NavigationRail {
    pub items: Vec<NavigationItem>,
    pub active_route: Option<String>,
}

impl NavigationRail {
    pub fn new(items: Vec<NavigationItem>, active_route: Option<String>) -> Self {
        Self {
            items,
            active_route,
        }
    }
}

/// Workspace switcher component used across platforms.
pub struct WorkspaceSwitcher {
    pub workspaces: Vec<Workspace>,
    pub active: Option<String>,
}

impl WorkspaceSwitcher {
    pub fn new(workspaces: Vec<Workspace>, active: Option<String>) -> Self {
        Self { workspaces, active }
    }
}

/// Knowledge base overlay tailored to the active persona.
pub struct KnowledgeOverlay {
    pub persona: WorkspacePersona,
    pub articles: Vec<KnowledgeArticle>,
}

impl KnowledgeOverlay {
    pub fn new(persona: WorkspacePersona, articles: Vec<KnowledgeArticle>) -> Self {
        Self { persona, articles }
    }
}

/// Composite shell chrome returned to platform renderers.
pub struct ShellChrome {
    pub navigation: NavigationRail,
    pub workspace_switcher: WorkspaceSwitcher,
    pub knowledge: KnowledgeOverlay,
}

impl ShellChrome {
    pub fn new(
        navigation: NavigationRail,
        workspace_switcher: WorkspaceSwitcher,
        knowledge: KnowledgeOverlay,
    ) -> Self {
        Self {
            navigation,
            workspace_switcher,
            knowledge,
        }
    }
}
