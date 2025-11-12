//! Kernel initialization and capability lifecycle orchestration.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::capabilities::builtin::register_default_capabilities;
use crate::capabilities::{CapabilityError, CapabilityRegistry, KernelHandle};
use crate::config::manifest::{KernelManifest, ManifestError};

static KERNEL_RUNNING: AtomicBool = AtomicBool::new(false);

lazy_static::lazy_static! {
    static ref GLOBAL_KERNEL: Mutex<Option<KernelHandle>> = Mutex::new(None);
}

/// Kernel initialization errors.
#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("kernel already running")]
    AlreadyRunning,
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error(transparent)]
    Capability(#[from] CapabilityError),
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
        let mut global = GLOBAL_KERNEL.lock().unwrap();
        *global = Some(handle.clone());
    }

    KERNEL_RUNNING.store(true, Ordering::SeqCst);
    Ok(handle)
}

/// Access the active kernel handle, if initialized.
pub fn handle() -> Option<KernelHandle> {
    GLOBAL_KERNEL.lock().unwrap().clone()
}

/// Check if the kernel is running.
pub fn is_running() -> bool {
    KERNEL_RUNNING.load(Ordering::SeqCst)
}

/// Shutdown the kernel and all registered capabilities.
pub fn shutdown() {
    let handle = {
        let mut global = GLOBAL_KERNEL.lock().unwrap();
        global.take()
    };

    if let Some(handle) = handle {
        if let Err(err) = handle.shutdown() {
            eprintln!("[KERNEL] shutdown error: {err}");
        }
    }

    KERNEL_RUNNING.store(false, Ordering::SeqCst);
}
