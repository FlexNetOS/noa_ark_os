//! AgentAsKit Shared Components
//! 
//! This crate provides shared data models, protocols, types, and utilities
//! used across all AgentAsKit systems including:
//! - Core multi-agent system
//! - FlexNetOS execution infrastructure  
//! - NOA deployment orchestration
//! 
//! The shared components ensure consistency and interoperability between
//! all AgentAsKit subsystems while following the "Heal, Don't Harm" principle.

// Re-export all public modules
pub mod data_models;
pub mod protocols;
pub mod types;
pub mod utils;

// Re-export commonly used types for convenience
pub use data_models::*;
pub use protocols::*;
pub use types::*;

// Re-export utilities with namespace
pub use utils::{
    agent_utils, task_utils, config_utils, health_utils, 
    metrics_utils, flexnetos_utils, noa_utils
};

/// Version information for the shared components
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize shared components with default configuration
pub fn init() -> crate::types::AgentAsKitResult<()> {
    // Initialize any shared resources, logging, etc.
    Ok(())
}

/// Initialize shared components with custom configuration
pub fn init_with_config(config: crate::types::AgentAsKitConfig) -> crate::types::AgentAsKitResult<()> {
    // Validate configuration
    crate::utils::config_utils::validate_config(&config)?;
    
    // Initialize shared resources with configuration
    Ok(())
}