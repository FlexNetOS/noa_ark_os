use crate::state::{NavigationItem, NavigationState, Workspace};

/// Declarative navigation rail component representation.
pub struct NavigationRail {
    pub items: Vec<NavigationItem>,
    pub active_route: Option<String>,
}

impl NavigationRail {
    pub fn from_state(state: &NavigationState) -> Self {
        Self {
            items: state.primary_items.clone(),
            active_route: state.active_route.clone(),
        }
    }
}

/// Workspace switcher component used across platforms.
pub struct WorkspaceSwitcher {
    pub workspaces: Vec<Workspace>,
}

impl WorkspaceSwitcher {
    pub fn new(workspaces: Vec<Workspace>) -> Self {
        Self { workspaces }
    }
}

/// Composite shell chrome returned to platform renderers.
pub struct ShellChrome {
    pub navigation: NavigationRail,
    pub workspace_switcher: WorkspaceSwitcher,
}

impl ShellChrome {
    pub fn new(navigation: NavigationRail, workspace_switcher: WorkspaceSwitcher) -> Self {
        Self {
            navigation,
            workspace_switcher,
        }
    }
}
