use crate::components::ShellChrome;
use crate::renderer::renderer::Renderer;
use crate::UIState;

/// Trait describing how shells are mounted on different targets.
pub trait PlatformAdapter {
    fn mount(&self, renderer: &Renderer, chrome: &ShellChrome, state: &UIState);
}

/// No-op adapter used for server orchestration surfaces.
pub struct ServerAdapter;

impl PlatformAdapter for ServerAdapter {
    fn mount(&self, renderer: &Renderer, _chrome: &ShellChrome, _state: &UIState) {
        renderer
            .render("server-shell")
            .expect("render server shell");
    }
}

/// Adapter for desktop/web React-based surfaces.
pub struct ReactAdapter;

impl PlatformAdapter for ReactAdapter {
    fn mount(&self, renderer: &Renderer, chrome: &ShellChrome, _state: &UIState) {
        let _ = renderer.render(&format!(
            "react-shell:{} routes",
            chrome.navigation.items.len()
        ));
    }
}

/// Adapter for XR visualizations.
pub struct SpatialAdapter;

impl PlatformAdapter for SpatialAdapter {
    fn mount(&self, renderer: &Renderer, chrome: &ShellChrome, _state: &UIState) {
        let _ = renderer.render(&format!(
            "spatial-shell:{} workspaces",
            chrome.workspace_switcher.workspaces.len()
        ));
    }
}
