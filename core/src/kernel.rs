//! Kernel initialization and core loop

use std::sync::atomic::{AtomicBool, Ordering};

static KERNEL_RUNNING: AtomicBool = AtomicBool::new(false);

/// Initialize the kernel
pub fn init() -> Result<(), &'static str> {
    println!("[KERNEL] Initializing kernel...");
    KERNEL_RUNNING.store(true, Ordering::SeqCst);
    Ok(())
}

/// Check if kernel is running
pub fn is_running() -> bool {
    KERNEL_RUNNING.load(Ordering::SeqCst)
}

/// Shutdown the kernel
pub fn shutdown() {
    println!("[KERNEL] Shutting down kernel...");
    KERNEL_RUNNING.store(false, Ordering::SeqCst);
}
