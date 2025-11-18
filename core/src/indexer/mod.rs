//! Workspace indexer responsible for generating dependency graphs.

mod ast;
mod config;
mod ownership;

pub use ast::{AstEdge, AstGraph, AstNode};
pub use config::{ConfigDependency, ConfigGraph, ManifestNode};
pub use ownership::{ComponentOwnership, FileOwnership, OwnerInfo, OwnershipGraph};

use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use thiserror::Error;

use crate::memory::RegistryError;

/// Directories containing generated artifacts or imported repositories that are
/// not part of the first-party workspace code we want to index.
const SKIP_PREFIXES: &[&str] = &[
    "archive",
    "build_output",
    "repos",
    "tools/.pnpm-home",
    ".workspace/indexes",
    // Portable Cargo registry mirrors third-party crates (including malformed test fixtures)
    // that should not block indexing.
    "server/tools/cargo-portable/registry",
];

const DEFAULT_OUTPUT_DIR: &str = ".workspace/indexes";
const AST_INDEX: &str = "ast_graph.json";
const OWNERSHIP_INDEX: &str = "ownership_graph.json";
const CONFIG_INDEX: &str = "config_graph.json";

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("syntax error: {0}")]
    Syntax(#[from] syn::Error),
    #[error("config error: {0}")]
    Config(#[from] toml::de::Error),
    #[error("registry error: {0}")]
    Registry(#[from] RegistryError),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexArtifacts {
    pub generated_at: u128,
    pub ast: AstGraph,
    pub ownership: OwnershipGraph,
    pub config: ConfigGraph,
}

pub struct IndexerService {
    source: PathBuf,
    output: PathBuf,
}

impl IndexerService {
    pub fn new(source: impl AsRef<Path>) -> Self {
        Self {
            source: source.as_ref().to_path_buf(),
            output: PathBuf::from(DEFAULT_OUTPUT_DIR),
        }
    }

    pub fn for_workspace() -> Self {
        Self::new(Path::new("."))
    }

    pub fn with_output_dir(mut self, output: impl AsRef<Path>) -> Self {
        self.output = output.as_ref().to_path_buf();
        self
    }

    pub fn refresh(&self) -> Result<IndexArtifacts, IndexerError> {
        let ast = AstGraph::build(&self.source)?;
        let ownership = OwnershipGraph::build()?;
        let config = ConfigGraph::build(&self.source)?;

        let artifacts = IndexArtifacts {
            generated_at: crate::utils::current_timestamp_millis(),
            ast,
            ownership,
            config,
        };
        self.persist(&artifacts)?;
        Ok(artifacts)
    }

    pub fn persist(&self, artifacts: &IndexArtifacts) -> Result<(), IndexerError> {
        fs::create_dir_all(&self.output)?;
        write_json(self.output.join(AST_INDEX), &artifacts.ast)?;
        write_json(self.output.join(OWNERSHIP_INDEX), &artifacts.ownership)?;
        write_json(self.output.join(CONFIG_INDEX), &artifacts.config)?;
        Ok(())
    }
}

fn write_json(path: PathBuf, value: &impl Serialize) -> Result<(), IndexerError> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, value)?;
    Ok(())
}

pub(crate) fn should_skip(path: &Path) -> bool {
    SKIP_PREFIXES.iter().any(|prefix| path.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn refresh_generates_indexes() {
        let dir = tempdir().unwrap();
        let service = IndexerService::new(Path::new("src")).with_output_dir(dir.path());
        let artifacts = service.refresh().expect("indexing succeeds");
        assert!(!artifacts.ast.nodes.is_empty());
        assert!(dir.path().join(AST_INDEX).exists());
        assert!(dir.path().join(OWNERSHIP_INDEX).exists());
        assert!(dir.path().join(CONFIG_INDEX).exists());
    }
}
