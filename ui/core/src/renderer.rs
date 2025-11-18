//! UI Renderer - Multi-platform rendering engine

use crate::components::ShellChrome;
use crate::{Platform, UIContext};

/// Data passed from the shell to downstream renderers per frame.
pub struct RenderFrame<'a> {
    pub chrome: &'a ShellChrome,
}

pub struct Renderer {
    context: UIContext,
}

impl Renderer {
    pub fn new(context: UIContext) -> Self {
        Self { context }
    }

    pub fn render(&self, component: &str) -> Result<(), String> {
        match self.context.platform {
            Platform::Server => self.render_api(component),
            Platform::Mobile => self.render_mobile(component),
            Platform::Desktop => self.render_desktop(component),
            Platform::Web => self.render_web(component),
            Platform::ARGlasses => self.render_ar(component),
            Platform::XRHeadset => self.render_xr(component),
        }
    }

    pub fn render_frame(&self, frame: &RenderFrame<'_>) -> Result<(), String> {
        let component = format!(
            "shell-navigation:{} workspaces:{} active:{} knowledge:{}",
            frame.chrome.navigation.items.len(),
            frame.chrome.workspace_switcher.workspaces.len(),
            frame
                .chrome
                .workspace_switcher
                .active
                .as_deref()
                .unwrap_or("none"),
            frame.chrome.knowledge.articles.len()
        );
        self.render(&component)
    }

    fn render_api(&self, _component: &str) -> Result<(), String> {
        Ok(())
    }

    fn render_mobile(&self, _component: &str) -> Result<(), String> {
        Ok(())
    }

    fn render_desktop(&self, _component: &str) -> Result<(), String> {
        Ok(())
    }

    fn render_web(&self, _component: &str) -> Result<(), String> {
        Ok(())
    }

    fn render_ar(&self, _component: &str) -> Result<(), String> {
        Ok(())
    }

    fn render_xr(&self, _component: &str) -> Result<(), String> {
        Ok(())
    }
}

pub mod state {
    //! State management lives in `crate::state`.
}

pub mod components {
    //! UI Components live in `crate::components`.
}

pub mod adapters {
    //! Platform adapters live in `crate::adapters`.
}
