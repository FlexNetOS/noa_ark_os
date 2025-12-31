use std::collections::HashSet;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WorldGraphError {
    #[error("io error while reading world graph: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse world graph: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("world graph validation error: {0}")]
    Validation(String),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeKind {
    Directory,
    File,
    Service,
    Dataset,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    pub path: String,
    pub summary: String,
    #[serde(default)]
    pub layer: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub owner: Option<String>,
}

impl Node {
    pub fn as_path(&self, repo_root: &Path) -> PathBuf {
        let node_path = Path::new(&self.path);
        if node_path.is_absolute() {
            node_path.to_path_buf()
        } else {
            repo_root.join(node_path)
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub relationship: String,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Metadata {
    pub generated: String,
    pub description: String,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorldGraph {
    pub version: String,
    pub metadata: Metadata,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl WorldGraph {
    pub fn load_from_reader<R: Read>(reader: R) -> Result<Self, WorldGraphError> {
        let graph: WorldGraph = serde_json::from_reader(reader)?;
        graph.validate()?;
        Ok(graph)
    }

    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, WorldGraphError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::load_from_reader(reader)
    }

    pub fn load_default() -> Result<Self, WorldGraphError> {
        let repo_root = Self::repo_root();
        let graph_path = repo_root.join("kernel/world/world.graph.json");
        Self::load_from_path(graph_path)
    }

    pub fn repo_root() -> PathBuf {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or(manifest_dir)
    }

    pub fn validate(&self) -> Result<(), WorldGraphError> {
        let mut ids = HashSet::new();
        for node in &self.nodes {
            if !ids.insert(&node.id) {
                return Err(WorldGraphError::Validation(format!(
                    "duplicate node id detected: {}",
                    node.id
                )));
            }
            if node.path.trim().is_empty() {
                return Err(WorldGraphError::Validation(format!(
                    "node {} has an empty path",
                    node.id
                )));
            }
        }

        let node_ids: HashSet<_> = self.nodes.iter().map(|n| n.id.as_str()).collect();
        for edge in &self.edges {
            if !node_ids.contains(edge.source.as_str()) {
                return Err(WorldGraphError::Validation(format!(
                    "edge references missing source node {}",
                    edge.source
                )));
            }
            if !node_ids.contains(edge.target.as_str()) {
                return Err(WorldGraphError::Validation(format!(
                    "edge references missing target node {}",
                    edge.target
                )));
            }
        }

        Ok(())
    }
}
