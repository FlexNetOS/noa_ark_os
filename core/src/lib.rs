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
pub mod ipc;
pub mod kernel;
pub mod memory;
pub mod process;
pub mod runtime;
pub mod security;

/// Core OS version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the core OS using the kernel capability system.
pub fn init() -> Result<capabilities::KernelHandle, kernel::KernelError> {
    println!("NOA ARK OS Core v{}", VERSION);
    println!("Initializing kernel-managed capabilities...");
    let handle = kernel::init()?;
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
