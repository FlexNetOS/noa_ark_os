// Shared types for CRC system

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Source type for code drops
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SourceType {
    ExternalRepo,
    Fork,
    Mirror,
    Stale,
    Internal,
}

/// Priority level for processing
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    Critical,
    High,
    Normal,
    Low,
}

/// Sandbox model assignment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SandboxModel {
    ModelA,  // Feature development
    ModelB,  // Bug fixes
    ModelC,  // Experimental
    ModelD,  // Integration
}

/// Drop manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropManifest {
    pub name: String,
    pub source: String,
    pub source_type: SourceType,
    pub timestamp: u64,
    pub priority: Priority,
    pub metadata: HashMap<String, String>,
}

/// Processing status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessingStatus {
    Pending,
    Analyzing,
    Adapting,
    Validating,
    Ready,
    InSandbox,
    Completed,
    Failed,
}
