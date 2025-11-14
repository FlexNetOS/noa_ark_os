//! Kernel initialization and capability lifecycle orchestration.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use crate::capabilities::builtin::register_default_capabilities;
use crate::capabilities::{CapabilityError, CapabilityRegistry, KernelHandle};
use crate::config::manifest::{KernelManifest, ManifestError};
use crate::config::profile::{CapabilityToken, ProfileDocument, ProfileError};

static KERNEL_RUNNING: AtomicBool = AtomicBool::new(false);

fn global_kernel() -> &'static Mutex<Option<KernelHandle>> {
    static GLOBAL_KERNEL: OnceLock<Mutex<Option<KernelHandle>>> = OnceLock::new();
    GLOBAL_KERNEL.get_or_init(|| Mutex::new(None))
}

fn active_profile_slot() -> &'static Mutex<Option<ActiveProfile>> {
    static ACTIVE_PROFILE: OnceLock<Mutex<Option<ActiveProfile>>> = OnceLock::new();
    ACTIVE_PROFILE.get_or_init(|| Mutex::new(None))
}

#[derive(Clone)]
struct ActiveProfile {
    name: String,
    token: CapabilityToken,
    source: PathBuf,
}

const DEFAULT_PROFILE_TOKEN_TTL_HOURS: u64 = 6;

/// Kernel initialization errors.
#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("kernel already running")]
    AlreadyRunning,
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error(transparent)]
    Capability(#[from] CapabilityError),
    #[error(transparent)]
    Profile(#[from] ProfileError),
    #[error("initialization failed: {0}")]
    Init(String),
}

/// Initialize the kernel with the default manifest.
pub fn init() -> Result<KernelHandle, KernelError> {
    init_with_manifest(KernelManifest::default())
}

/// Initialize the kernel using the provided manifest.
pub fn init_with_manifest(manifest: KernelManifest) -> Result<KernelHandle, KernelError> {
    if KERNEL_RUNNING.load(Ordering::SeqCst) {
        return Err(KernelError::AlreadyRunning);
    }

    manifest.validate()?;

    let manifest = Arc::new(manifest);
    let registry = Arc::new(CapabilityRegistry::new());

    register_default_capabilities(&registry)?;

    let handle = KernelHandle::new(Arc::clone(&registry), Arc::clone(&manifest));
    registry.initialize_autostart(&handle)?;

    {
        let mut global = global_kernel().lock().unwrap();
        *global = Some(handle.clone());
    }

    KERNEL_RUNNING.store(true, Ordering::SeqCst);
    Ok(handle)
}

/// Access the active kernel handle, if initialized.
pub fn handle() -> Option<KernelHandle> {
    global_kernel().lock().unwrap().clone()
}

/// Check if the kernel is running.
pub fn is_running() -> bool {
    KERNEL_RUNNING.load(Ordering::SeqCst)
}

/// Shutdown the kernel and all registered capabilities.
pub fn shutdown() {
    let handle = {
        let mut global = global_kernel().lock().unwrap();
        global.take()
    };

    if let Some(handle) = handle {
        if let Err(err) = handle.shutdown() {
            eprintln!("[KERNEL] shutdown error: {err}");
        }
    }

    KERNEL_RUNNING.store(false, Ordering::SeqCst);
}

/// Load a profile manifest and derive a capability token that mirrors its constraints.
pub fn load_profile(path: impl AsRef<Path>) -> Result<CapabilityToken, KernelError> {
    let path_ref = path.as_ref();
    let document = ProfileDocument::load_from_path(path_ref)?;
    let token =
        document.into_capability_token(Duration::from_secs(DEFAULT_PROFILE_TOKEN_TTL_HOURS * 3600));

    {
        let mut slot = active_profile_slot().lock().unwrap();
        *slot = Some(ActiveProfile {
            name: token.profile_name.clone(),
            token: token.clone(),
            source: path_ref.to_path_buf(),
        });
    }

    Ok(token)
}

/// Return the currently active capability token, if a profile has been loaded.
pub fn active_capability_token() -> Option<CapabilityToken> {
    active_profile_slot()
        .lock()
        .unwrap()
        .as_ref()
        .map(|profile| profile.token.clone())
}

/// Return the name of the currently loaded profile, if any.
pub fn active_profile_name() -> Option<String> {
    active_profile_slot()
        .lock()
        .unwrap()
        .as_ref()
        .map(|profile| profile.name.clone())
}

#[cfg(test)]
pub(crate) fn reset_active_profile() {
    let mut slot = active_profile_slot().lock().unwrap();
    *slot = None;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::profile::{EgressMode, StorageMode};

    fn profile_path(name: &str) -> PathBuf {
        Path::new("../server/profiles")
            .join(name)
            .join("profile.toml")
    }

    #[test]
    fn profile_switching_updates_active_token() {
        reset_active_profile();

        let air_token = load_profile(profile_path("air_gapped")).expect("air_gapped profile");
        assert_eq!(air_token.profile_name, "air_gapped");
        assert_eq!(air_token.egress_mode, EgressMode::Denied);
        assert!(air_token.denied_tools.contains(&"git".to_string()));
        assert!(active_capability_token()
            .unwrap()
            .allowed_tools
            .contains(&"cargo".to_string()));

        let dev_token = load_profile(profile_path("devcontainer")).expect("devcontainer profile");
        assert_eq!(dev_token.profile_name, "devcontainer");
        assert_eq!(active_profile_name().as_deref(), Some("devcontainer"));
        assert_eq!(dev_token.egress_mode, EgressMode::AllowList);
        assert!(dev_token
            .allowed_egress_destinations
            .contains(&"github.com".to_string()));
        assert!(dev_token.allowed_tools.contains(&"docker".to_string()));

        let storage = &dev_token.storage_roots;
        assert!(!storage.is_empty());
        assert!(storage
            .iter()
            .any(|root| root.mode == StorageMode::ReadOnly));

        let edge_token = load_profile(profile_path("edge_lite")).expect("edge_lite profile");
        assert_eq!(edge_token.profile_name, "edge_lite");
        assert!(edge_token.allowed_tools.contains(&"busybox".to_string()));
        assert!(edge_token
            .allowed_egress_destinations
            .contains(&"telemetry.noa-ark.local".to_string()));
        assert!(edge_token
            .storage_roots
            .iter()
            .any(|root| root.name == "telemetry_buffer"));

        reset_active_profile();
    }
}
