use crate::state::{
    KnowledgeArticle, NavigationItem, NavigationState, Workspace, WorkspacePersona,
};

/// Declarative navigation rail component representation.
#[derive(Debug, Clone)]
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

    pub fn from_state(state: &NavigationState) -> Self {
        Self::new(state.primary_items.clone(), state.active_route.clone())
    }
}

/// Workspace switcher component used across platforms.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{Workspace, WorkspacePersona};

    #[test]
    fn shell_chrome_wires_navigation_and_knowledge() {
        let nav = NavigationRail::new(vec![], None);
        let workspace = WorkspaceSwitcher::new(
            vec![Workspace {
                id: "ai-studio".into(),
                label: "AI Ops Studio".into(),
                persona: WorkspacePersona::Developer,
                routes: vec!["/chat".into()],
                allowed_roles: vec!["developer".into()],
            }],
            Some("ai-studio".into()),
        );
        let knowledge = KnowledgeOverlay::new(WorkspacePersona::Developer, vec![]);
        let chrome = ShellChrome::new(nav, workspace, knowledge);
        assert_eq!(chrome.knowledge.persona, WorkspacePersona::Developer);
    }
}
