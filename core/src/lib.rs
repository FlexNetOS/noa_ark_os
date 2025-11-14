//! NOA ARK OS - Core Operating System Layer
//!
//! This is the foundational layer providing:
//! - Process management
//! - Memory management
//! - Inter-process communication
//! - System calls
//! - Resource scheduling

pub mod capabilities;
pub mod config;
pub mod fs;
pub mod gateway;
pub mod hardware;
pub mod host_control;
pub mod ipc;
pub mod kernel;
pub mod memory;
pub mod metrics;
pub mod process;
pub mod runtime;
pub mod security;
pub mod symbols;
pub mod time;
pub mod token;
pub mod utils;

/// Core OS version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the core OS using the kernel capability system.
pub fn init() -> Result<capabilities::KernelHandle, kernel::KernelError> {
    println!("NOA ARK OS Core v{}", VERSION);
    println!("Initializing kernel-managed capabilities...");
    let handle = kernel::init()?;
    println!("Initializing core services...");

    // Initialize subsystems
    kernel::init()?;
    memory::init().map_err(|e| kernel::KernelError::Init(e.to_string()))?;
    process::init().map_err(|e| kernel::KernelError::Init(e.to_string()))?;
    ipc::init().map_err(|e| kernel::KernelError::Init(e.to_string()))?;
    fs::init().map_err(|e| kernel::KernelError::Init(e.to_string()))?;
    security::init().map_err(|e| kernel::KernelError::Init(e.to_string()))?;
    gateway::init()
        .map_err(|_| kernel::KernelError::Init("gateway initialization failed".to_string()))?;

    println!("Core OS initialized successfully");
    Ok(handle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
