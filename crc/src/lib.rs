//! CRC - Continuous ReCode System with Sandbox Models
//!
//! Intelligent code adaptation with AI supervision and sandbox model isolation.
//!
//! ## Sandbox Models:
//! - Model A: Feature development sandbox
//! - Model B: Bug fix sandbox
//! - Model C: Experimental sandbox
//! - Model D: Integration sandbox (merged from A, B, C)

// Re-export modules
pub mod archive;
pub mod build;
pub mod commands;
pub mod digestors;
pub mod engine;
pub mod error;
pub mod graph;
pub mod ir;
pub mod orchestrator;
pub mod parallel;
pub mod processor;
pub mod transform;
pub mod types;
pub mod watcher;

// Re-export common types
pub use build::{BuildArtifact, BuildManifest, TargetProfile};
pub use error::{Error, Result};
pub use types::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SourceType {
    StaleCodebase,
    ExternalRepo,
    Fork,
    Mirror,
    Internal,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SandboxModel {
    ModelA, // Feature development
    ModelB, // Bug fixes
    ModelC, // Experimental
    ModelD, // Integration (merged from A, B, C)
}

impl SandboxModel {
    pub fn description(&self) -> &str {
        match self {
            SandboxModel::ModelA => "Feature Development Sandbox",
            SandboxModel::ModelB => "Bug Fix Sandbox",
            SandboxModel::ModelC => "Experimental Sandbox",
            SandboxModel::ModelD => "Integration Sandbox (A+B+C)",
        }
    }

    pub fn can_merge_to_d(&self) -> bool {
        matches!(
            self,
            SandboxModel::ModelA | SandboxModel::ModelB | SandboxModel::ModelC
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CRCState {
    Incoming,
    Queued,
    Analyzing,
    Adapting,
    Validating,
    InSandbox(SandboxModel),
    ReadyToMerge,
    Merged,
    Ready,
    Failed,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeDrop {
    pub id: String,
    pub source_type: SourceType,
    pub source_path: PathBuf,
    pub state: CRCState,
    pub sandbox: Option<SandboxModel>,
    pub manifest: DropManifest,
    pub analysis: Option<AnalysisResult>,
    pub adaptation: Option<AdaptationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropManifest {
    pub name: String,
    pub source: String,
    pub source_type: SourceType,
    pub timestamp: u64,
    pub priority: Priority,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub files_count: usize,
    pub lines_count: usize,
    pub languages: Vec<String>,
    pub dependencies: Vec<Dependency>,
    pub patterns_found: Vec<String>,
    pub issues: Vec<String>,
    pub ai_confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: Option<String>,
    pub source: String,
    pub embedded_alternative: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationResult {
    pub changes_made: usize,
    pub files_modified: usize,
    pub tests_generated: usize,
    pub ai_confidence: f32,
    pub auto_approved: bool,
    pub diff_summary: String,
    pub sandbox_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveInfo {
    pub hash: String,
    pub archive_path: PathBuf,
    pub created: u64,
    pub size: u64,
    pub index: ArchiveIndex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveIndex {
    pub files: Vec<FileEntry>,
    pub symbols: Vec<SymbolEntry>,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub hash: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolEntry {
    pub name: String,
    pub file: String,
    pub line: usize,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxState {
    pub model: SandboxModel,
    pub drops: Vec<String>,
    pub validated: bool,
    pub ready_to_merge: bool,
}

pub struct CRCSystem {
    drops: Arc<Mutex<HashMap<String, CodeDrop>>>,
    archives: Arc<Mutex<HashMap<String, ArchiveInfo>>>,
    sandboxes: Arc<Mutex<HashMap<SandboxModel, SandboxState>>>,
    config: Arc<Mutex<CRCConfig>>,
}

impl Clone for CRCSystem {
    fn clone(&self) -> Self {
        Self {
            drops: Arc::clone(&self.drops),
            archives: Arc::clone(&self.archives),
            sandboxes: Arc::clone(&self.sandboxes),
            config: Arc::clone(&self.config),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CRCConfig {
    pub drop_in_path: PathBuf,
    pub archive_path: PathBuf,
    pub temp_path: PathBuf,
    pub auto_approve_threshold: f32,
    pub compression_level: i32,
    pub retention_days: HashMap<SourceType, u32>,
    // Service automation config
    pub max_concurrent: usize,
    pub auto_archive: bool,
    pub trigger_cicd: bool,
    pub compression_algorithm: String,
}

impl Default for CRCConfig {
    fn default() -> Self {
        let mut retention = HashMap::new();
        retention.insert(SourceType::StaleCodebase, 90);
        retention.insert(SourceType::ExternalRepo, 180);
        retention.insert(SourceType::Fork, 90);
        retention.insert(SourceType::Mirror, 30);
        retention.insert(SourceType::Internal, 365);

        Self {
            drop_in_path: PathBuf::from("crc/drop-in"),
            archive_path: PathBuf::from("crc/archive"),
            temp_path: PathBuf::from("crc/temp"),
            auto_approve_threshold: 0.95,
            compression_level: 3,
            retention_days: retention,
            // Service automation defaults
            max_concurrent: 4,
            auto_archive: true,
            trigger_cicd: true,
            compression_algorithm: "zstd".to_string(),
        }
    }
}

impl CRCSystem {
    pub fn new(config: CRCConfig) -> Self {
        let mut sandboxes = HashMap::new();

        // Initialize sandbox models
        for model in [
            SandboxModel::ModelA,
            SandboxModel::ModelB,
            SandboxModel::ModelC,
            SandboxModel::ModelD,
        ] {
            sandboxes.insert(
                model,
                SandboxState {
                    model,
                    drops: vec![],
                    validated: false,
                    ready_to_merge: false,
                },
            );
        }

        Self {
            drops: Arc::new(Mutex::new(HashMap::new())),
            archives: Arc::new(Mutex::new(HashMap::new())),
            sandboxes: Arc::new(Mutex::new(sandboxes)),
            config: Arc::new(Mutex::new(config)),
        }
    }

    /// Create test instance
    #[cfg(test)]
    pub fn new_test() -> Self {
        Self::new(CRCConfig::default())
    }

    /// Scan for new drops in incoming folder
    pub fn scan_incoming(&self) -> std::result::Result<Vec<String>, String> {
        println!("[CRC] Scanning for new code drops...");

        // In real implementation, scan filesystem
        // For now, return empty
        Ok(vec![])
    }

    /// Register a new code drop
    pub fn register_drop(
        &self,
        path: PathBuf,
        manifest: DropManifest,
    ) -> std::result::Result<String, String> {
        let id = format!("drop_{}", uuid::Uuid::new_v4());

        let drop = CodeDrop {
            id: id.clone(),
            source_type: manifest.source_type.clone(),
            source_path: path,
            state: CRCState::Incoming,
            sandbox: None,
            manifest,
            analysis: None,
            adaptation: None,
        };

        let mut drops = self.drops.lock().unwrap();
        drops.insert(id.clone(), drop);

        println!("[CRC] Registered code drop: {}", id);
        Ok(id)
    }

    /// Analyze code drop
    pub fn analyze(&self, drop_id: &str) -> std::result::Result<AnalysisResult, String> {
        println!("[CRC] Analyzing code drop: {}", drop_id);

        // Update state
        self.update_state(drop_id, CRCState::Analyzing)?;

        // Simulate analysis
        let analysis = AnalysisResult {
            files_count: 100,
            lines_count: 10000,
            languages: vec!["Rust".to_string()],
            dependencies: vec![],
            patterns_found: vec![],
            issues: vec![],
            ai_confidence: 0.90,
        };

        // Store analysis
        let mut drops = self.drops.lock().unwrap();
        if let Some(drop) = drops.get_mut(drop_id) {
            drop.analysis = Some(analysis.clone());
            drop.state = CRCState::Validating;
        }

        Ok(analysis)
    }

    /// Update drop state
    fn update_state(&self, drop_id: &str, state: CRCState) -> std::result::Result<(), String> {
        let mut drops = self.drops.lock().unwrap();
        if let Some(drop) = drops.get_mut(drop_id) {
            drop.state = state;
            Ok(())
        } else {
            Err(format!("Drop not found: {}", drop_id))
        }
    }

    /// Get drop by ID
    pub fn get_drop(&self, drop_id: &str) -> Option<CodeDrop> {
        let drops = self.drops.lock().unwrap();
        drops.get(drop_id).cloned()
    }
}

impl Default for CRCSystem {
    fn default() -> Self {
        Self::new(CRCConfig::default())
    }
}
