use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

/// Unique identifier for nodes inside the CRC IR graph.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(#[serde_as(as = "DisplayFromStr")] pub uuid::Uuid);

impl NodeId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lane {
    Fast,
    Deep,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FacetKind {
    Source,
    Analysis,
    Decision,
    Transformation,
    Verification,
    Persistence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Facet {
    pub kind: FacetKind,
    pub lane: Lane,
    pub metadata: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub origin: String,
    pub description: Option<String>,
    pub captured_at: chrono::DateTime<chrono::Utc>,
    pub trust_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub path: Option<PathBuf>,
    pub hash: String,
    pub content_type: Option<String>,
    pub metadata: BTreeMap<String, serde_json::Value>,
}

impl ArtifactRef {
    pub fn new(path: Option<PathBuf>, data: &[u8], content_type: Option<String>) -> Self {
        Self {
            path,
            hash: blake3::hash(data).to_hex().to_string(),
            content_type,
            metadata: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataHandle {
    pub key: String,
    pub artifact: ArtifactRef,
    pub facets: Vec<Facet>,
    pub provenance: Provenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIo {
    pub inputs: BTreeMap<String, DataHandle>,
    pub outputs: BTreeMap<String, DataHandle>,
}

impl NodeIo {
    pub fn new() -> Self {
        Self {
            inputs: BTreeMap::new(),
            outputs: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeState {
    pub id: NodeId,
    pub kind: crate::graph::NodeKind,
    pub lane: Lane,
    pub facets: Vec<Facet>,
    pub io: NodeIo,
    pub dependencies: BTreeSet<NodeId>,
    pub cache_key: String,
}

impl NodeState {
    pub fn compute_cache_key(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.id.0.as_bytes());
        hasher.update(&[self.kind as u8]);
        hasher.update(&[self.lane as u8]);
        for dep in &self.dependencies {
            hasher.update(dep.0.as_bytes());
        }
        for (name, handle) in &self.io.inputs {
            hasher.update(name.as_bytes());
            hasher.update(handle.artifact.hash.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub nodes: Vec<NodeState>,
    pub captured_at: chrono::DateTime<chrono::Utc>,
    pub description: Option<String>,
}

impl Snapshot {
    pub fn new(nodes: Vec<NodeState>, description: Option<String>) -> Self {
        Self {
            nodes,
            captured_at: chrono::Utc::now(),
            description,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_changes_with_inputs() {
        let mut node = NodeState {
            id: NodeId::new(),
            kind: crate::graph::NodeKind::Analyze,
            lane: Lane::Fast,
            facets: vec![],
            io: NodeIo::new(),
            dependencies: BTreeSet::new(),
            cache_key: String::new(),
        };
        let provenance = Provenance {
            origin: "unit-test".into(),
            description: None,
            captured_at: chrono::Utc::now(),
            trust_score: 1.0,
        };
        node.io.inputs.insert(
            "input".into(),
            DataHandle {
                key: "input".into(),
                artifact: ArtifactRef::new(None, b"hello", None),
                facets: vec![],
                provenance: provenance.clone(),
            },
        );
        let key1 = node.compute_cache_key();
        node.io.inputs.get_mut("input").unwrap().artifact = ArtifactRef::new(None, b"world", None);
        let key2 = node.compute_cache_key();
        assert_ne!(key1, key2);
    }
}
