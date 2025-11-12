use std::fs;
use std::path::PathBuf;

use serde_json::Value;

use crate::components::ShellChrome;
use crate::renderer::renderer::Renderer;
use crate::UIState;

/// Trait describing how shells are mounted on different targets.
pub trait PlatformAdapter {
    fn mount(
        &self,
        renderer: &Renderer,
        chrome: &ShellChrome,
        state: &UIState,
    ) -> Result<(), String>;
}

/// No-op adapter used for server orchestration surfaces.
pub struct ServerAdapter;

impl PlatformAdapter for ServerAdapter {
    fn mount(
        &self,
        renderer: &Renderer,
        _chrome: &ShellChrome,
        _state: &UIState,
    ) -> Result<(), String> {
        renderer.render("server-shell")
    }
}

/// Adapter for desktop/web React-based surfaces.
pub struct ReactAdapter;

impl PlatformAdapter for ReactAdapter {
    fn mount(
        &self,
        renderer: &Renderer,
        chrome: &ShellChrome,
        _state: &UIState,
    ) -> Result<(), String> {
        let root = workspace_root()?.join("ui/noa-dashboard/index.html");
        let html = fs::read_to_string(&root)
            .map_err(|err| format!("failed to read {}: {}", root.display(), err))?;
        if !html.contains("id=\"root\"") && !html.contains("id='root'") {
            return Err(format!(
                "React host missing root element in {}",
                root.display()
            ));
        }

        renderer.render(&format!(
            "react-shell:{} routes knowledge:{}",
            chrome.navigation.items.len(),
            chrome.knowledge.articles.len()
        ))
    }
}

/// Adapter for desktop experiences powered by Tauri.
pub struct TauriAdapter {
    manifest: PathBuf,
}

impl Default for TauriAdapter {
    fn default() -> Self {
        Self {
            manifest: workspace_root()
                .map(|root| root.join("apps/desktop-shell/tauri.conf.json"))
                .unwrap_or_else(|_| PathBuf::from("apps/desktop-shell/tauri.conf.json")),
        }
    }
}

impl TauriAdapter {
    pub fn new(manifest: PathBuf) -> Self {
        Self { manifest }
    }
}

impl PlatformAdapter for TauriAdapter {
    fn mount(
        &self,
        renderer: &Renderer,
        chrome: &ShellChrome,
        _state: &UIState,
    ) -> Result<(), String> {
        let manifest = fs::read_to_string(&self.manifest)
            .map_err(|err| format!("failed to read {}: {}", self.manifest.display(), err))?;
        let json: Value = serde_json::from_str(&manifest).map_err(|err| {
            format!(
                "invalid tauri manifest {}: {}",
                self.manifest.display(),
                err
            )
        })?;
        let windows = json
            .pointer("/tauri/windows")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);
        let identifier = json
            .pointer("/tauri/bundle/identifier")
            .and_then(|v| v.as_str())
            .unwrap_or("noa.ark.shell");

        renderer.render(&format!(
            "tauri-shell:{} windows persona:{} identifier:{}",
            windows,
            chrome
                .workspace_switcher
                .active
                .as_deref()
                .unwrap_or("none"),
            identifier
        ))
    }
}

/// Adapter for React Native mobile builds.
pub struct ReactNativeAdapter {
    app_manifest: PathBuf,
}

impl Default for ReactNativeAdapter {
    fn default() -> Self {
        Self {
            app_manifest: workspace_root()
                .map(|root| root.join("apps/mobile-shell/app.json"))
                .unwrap_or_else(|_| PathBuf::from("apps/mobile-shell/app.json")),
        }
    }
}

impl ReactNativeAdapter {
    pub fn new(app_manifest: PathBuf) -> Self {
        Self { app_manifest }
    }
}

impl PlatformAdapter for ReactNativeAdapter {
    fn mount(
        &self,
        renderer: &Renderer,
        chrome: &ShellChrome,
        state: &UIState,
    ) -> Result<(), String> {
        let manifest = fs::read_to_string(&self.app_manifest)
            .map_err(|err| format!("failed to read {}: {}", self.app_manifest.display(), err))?;
        let json: Value = serde_json::from_str(&manifest).map_err(|err| {
            format!(
                "invalid React Native manifest {}: {}",
                self.app_manifest.display(),
                err
            )
        })?;
        let name = json
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or("noa-mobile");
        let display = json
            .get("displayName")
            .and_then(Value::as_str)
            .unwrap_or(name);
        let capabilities = state.context.capabilities.len();

        renderer.render(&format!(
            "react-native-shell:{} navigation:{} knowledge:{}",
            display,
            chrome.navigation.items.len(),
            capabilities
        ))
    }
}

/// Adapter for XR visualizations.
pub struct SpatialAdapter {
    scene_manifest: PathBuf,
}

impl Default for SpatialAdapter {
    fn default() -> Self {
        Self {
            scene_manifest: workspace_root()
                .map(|root| root.join("apps/xr-shell/scene.graph.json"))
                .unwrap_or_else(|_| PathBuf::from("apps/xr-shell/scene.graph.json")),
        }
    }
}

impl SpatialAdapter {
    pub fn new(scene_manifest: PathBuf) -> Self {
        Self { scene_manifest }
    }
}

impl PlatformAdapter for SpatialAdapter {
    fn mount(
        &self,
        renderer: &Renderer,
        chrome: &ShellChrome,
        _state: &UIState,
    ) -> Result<(), String> {
        let manifest = fs::read_to_string(&self.scene_manifest)
            .map_err(|err| format!("failed to read {}: {}", self.scene_manifest.display(), err))?;
        let json: Value = serde_json::from_str(&manifest).map_err(|err| {
            format!(
                "invalid XR scene {}: {}",
                self.scene_manifest.display(),
                err
            )
        })?;
        let nodes = json
            .get("nodes")
            .and_then(Value::as_array)
            .map(|nodes| nodes.len())
            .unwrap_or(0);
        let edges = json
            .get("edges")
            .and_then(Value::as_array)
            .map(|edges| edges.len())
            .unwrap_or(0);

        renderer.render(&format!(
            "spatial-shell:nodes={} edges={} workspaces={}",
            nodes,
            edges,
            chrome.workspace_switcher.workspaces.len()
        ))
    }
}

fn workspace_root() -> Result<PathBuf, String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .map(|path| path.to_path_buf())
        .ok_or_else(|| "failed to resolve workspace root".into())
}
