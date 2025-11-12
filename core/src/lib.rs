//! NOA ARK OS - Core Operating System Layer
//!
//! This is the foundational layer providing:
//! - Process management
//! - Memory management
//! - Inter-process communication
//! - System calls
//! - Resource scheduling

pub mod fs;
pub mod gateway;
pub mod hardware;
pub mod ipc;
pub mod kernel;
pub mod memory;
pub mod ipc;
pub mod kernel;
pub mod memory;
pub mod metrics;
pub mod process;
pub mod security;

/// Core OS version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the core OS
pub fn init() -> Result<(), &'static str> {
    println!("NOA ARK OS Core v{}", VERSION);
    println!("Initializing core services...");

    // Initialize subsystems
    kernel::init()?;
    memory::init()?;
    process::init()?;
    ipc::init()?;
    fs::init()?;
    security::init()?;
    gateway::init().map_err(|_| "gateway initialization failed")?;

    println!("Core OS initialized successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
