//! Dynamic UI Core - Multi-platform UI framework

pub mod adapters;
pub mod analytics;
pub mod chat;
pub mod components;
pub mod events;
pub mod module;
pub mod renderer;
pub mod services;
pub mod shell;
pub mod state;
pub mod workflows;

use std::collections::HashMap;
use std::sync::Arc;

use noa_core::capabilities::{CapabilityError, KernelHandle};
use noa_core::config::manifest::CAPABILITY_PROCESS;
use noa_core::process::ProcessService;

pub use module::{ModuleCapability, ModuleDescriptor, ShellModule};
pub use shell::{ShellBuilder, UnifiedShell};
use state::GlobalStore;

#[derive(Debug, Clone, PartialEq)]
pub enum Platform {
    Server,
    Mobile,
    Desktop,
    Web,
    ARGlasses,
    XRHeadset,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Capability {
    Touch,
    Mouse,
    Keyboard,
    Voice,
    Gesture,
    SpatialTracking,
    HandTracking,
    EyeTracking,
}

#[derive(Debug, Clone)]
pub struct UIContext {
    pub platform: Platform,
    pub screen_width: u32,
    pub screen_height: u32,
    pub dpi: f32,
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone)]
pub struct UIState {
    pub data: HashMap<String, serde_json::Value>,
    pub context: UIContext,
}

impl UIState {
    pub fn new(context: UIContext) -> Self {
        Self {
            data: HashMap::new(),
            context,
        }
    }

    pub fn set(&mut self, key: String, value: serde_json::Value) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.data.get(key)
    }

    pub fn merge(&mut self, other: &HashMap<String, serde_json::Value>) {
        self.data.extend(other.clone());
    }
}

/// Initialize UI system for given platform
pub fn init(platform: Platform) -> Result<UIContext, &'static str> {
    println!("[UI] Initializing UI system for platform: {:?}", platform);

    let context = match platform {
        Platform::Server => UIContext {
            platform,
            screen_width: 0,
            screen_height: 0,
            dpi: 1.0,
            capabilities: vec![],
        },
        Platform::Mobile => UIContext {
            platform,
            screen_width: 1080,
            screen_height: 2400,
            dpi: 3.0,
            capabilities: vec![Capability::Touch, Capability::Voice],
        },
        Platform::Desktop => UIContext {
            platform,
            screen_width: 1920,
            screen_height: 1080,
            dpi: 1.0,
            capabilities: vec![Capability::Mouse, Capability::Keyboard],
        },
        Platform::Web => UIContext {
            platform,
            screen_width: 1920,
            screen_height: 1080,
            dpi: 1.0,
            capabilities: vec![Capability::Mouse, Capability::Keyboard, Capability::Touch],
        },
        Platform::ARGlasses => UIContext {
            platform,
            screen_width: 1280,
            screen_height: 720,
            dpi: 2.0,
            capabilities: vec![
                Capability::Gesture,
                Capability::Voice,
                Capability::EyeTracking,
            ],
        },
        Platform::XRHeadset => UIContext {
            platform,
            screen_width: 2560,
            screen_height: 1440,
            dpi: 2.0,
            capabilities: vec![
                Capability::SpatialTracking,
                Capability::HandTracking,
                Capability::Voice,
            ],
        },
    };

    Ok(context)
}

/// Kernel-aware bridge for UI modules to request capabilities lazily.
#[derive(Clone)]
pub struct KernelUiBridge {
    kernel: KernelHandle,
}

impl KernelUiBridge {
    pub fn new(kernel: KernelHandle) -> Self {
        Self { kernel }
    }

    pub fn request_process_service(&self) -> Result<Arc<ProcessService>, CapabilityError> {
        self.kernel.request::<ProcessService>(CAPABILITY_PROCESS)
    }
}

/// Initialize the UI system and attach a kernel handle for capability requests.
pub fn init_with_kernel(
    platform: Platform,
    kernel: KernelHandle,
) -> Result<(UIContext, KernelUiBridge), &'static str> {
    let context = init(platform)?;
    Ok((context, KernelUiBridge::new(kernel)))
}

/// Convenience helper to bootstrap the unified shell and synchronise data into [`UIState`].
pub fn bootstrap(platform: Platform) -> Result<(UnifiedShell, UIState, GlobalStore), &'static str> {
    let shell = UnifiedShell::builder(platform.clone()).build()?;
    let mut ui_state = UIState::new(shell.context().clone());
    ui_state.merge(&shell.export_state());
    let store = shell.store_handle();
    Ok((shell, ui_state, store))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_init() {
        let context = init(Platform::Desktop).unwrap();
        assert_eq!(context.platform, Platform::Desktop);
    }

    #[test]
    fn bootstrap_shell_syncs_state() {
        let (shell, ui_state, _) = bootstrap(Platform::Web).expect("bootstrap");
        assert!(shell.export_state().contains_key("analytics.metrics"));
        assert!(ui_state.get("analytics.metrics").is_some());
    }
}
