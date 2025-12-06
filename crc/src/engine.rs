use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::graph::{CRCGraph, GraphNode, NodeKind};
use crate::ir::{NodeId, NodeState, Snapshot};

#[derive(Default)]
struct CacheEntry {
    outputs: BTreeMap<String, crate::ir::DataHandle>,
}

#[derive(Default)]
struct NodeCache {
    entries: std::collections::HashMap<NodeId, CacheEntry>,
}

impl NodeCache {
    fn get(&self, id: &NodeId) -> Option<&CacheEntry> {
        self.entries.get(id)
    }

    fn insert(&mut self, id: NodeId, entry: CacheEntry) {
        self.entries.insert(id, entry);
    }
}

#[derive(Clone)]
pub struct Engine {
    graph: CRCGraph,
    cache: std::sync::Arc<Mutex<NodeCache>>,
}

impl Engine {
    pub fn new(graph: CRCGraph) -> Self {
        Self {
            graph,
            cache: std::sync::Arc::new(Mutex::new(NodeCache::default())),
        }
    }

    pub async fn run(&self, checkpoint_dir: &Path) -> Result<ExecutionSummary> {
        fs::create_dir_all(checkpoint_dir)?;
        let mut executed = Vec::new();
        for id in self.graph.topo_order()? {
            let node = self
                .graph
                .node(&id)
                .cloned()
                .ok_or_else(|| anyhow!("node missing"))?;
            let state = self.execute_node(node).await?;
            executed.push(state.clone());
        }
        let snapshot = Snapshot::new(executed.clone(), Some("run completion".into()));
        self.persist_snapshot(checkpoint_dir, &snapshot)?;
        Ok(ExecutionSummary {
            executed,
            checkpoint: checkpoint_dir.to_path_buf(),
        })
    }

    async fn execute_node(&self, node: GraphNode) -> Result<NodeState> {
        let cache = self.cache.lock().await;
        if let Some(entry) = cache.get(&node.id) {
            let state = NodeState {
                id: node.id.clone(),
                kind: node.kind,
                lane: node.lane,
                facets: vec![],
                io: crate::ir::NodeIo {
                    inputs: Default::default(),
                    outputs: entry.outputs.clone(),
                },
                dependencies: Default::default(),
                cache_key: blake3::hash(node.name.as_bytes()).to_hex().to_string(),
            };
            return Ok(state);
        }
        drop(cache);

        // Simulate execution based on kind.
        let mut outputs = BTreeMap::new();
        let provenance = crate::ir::Provenance {
            origin: format!("{}", node.kind as u8),
            description: Some(format!("Executed {}", node.name)),
            captured_at: chrono::Utc::now(),
            trust_score: match node.kind {
                NodeKind::Analyze | NodeKind::Verify => 0.9,
                NodeKind::Decide => 0.8,
                NodeKind::Transform => 0.7,
                NodeKind::Persist => 1.0,
            },
        };
        outputs.insert(
            "stdout".into(),
            crate::ir::DataHandle {
                key: "stdout".into(),
                artifact: crate::ir::ArtifactRef::new(
                    None,
                    node.name.as_bytes(),
                    Some("text/plain".into()),
                ),
                facets: vec![crate::ir::Facet {
                    kind: match node.kind {
                        NodeKind::Analyze => crate::ir::FacetKind::Analysis,
                        NodeKind::Decide => crate::ir::FacetKind::Decision,
                        NodeKind::Transform => crate::ir::FacetKind::Transformation,
                        NodeKind::Verify => crate::ir::FacetKind::Verification,
                        NodeKind::Persist => crate::ir::FacetKind::Persistence,
                    },
                    lane: node.lane,
                    metadata: Default::default(),
                }],
                provenance,
            },
        );

        let mut cache = self.cache.lock().await;
        cache.insert(
            node.id.clone(),
            CacheEntry {
                outputs: outputs.clone(),
            },
        );
        drop(cache);

        Ok(NodeState {
            id: node.id,
            kind: node.kind,
            lane: node.lane,
            facets: vec![],
            io: crate::ir::NodeIo {
                inputs: Default::default(),
                outputs,
            },
            dependencies: Default::default(),
            cache_key: blake3::hash(node.name.as_bytes()).to_hex().to_string(),
        })
    }

    fn persist_snapshot(&self, checkpoint_dir: &Path, snapshot: &Snapshot) -> Result<()> {
        let path = checkpoint_dir.join(format!("snapshot-{}.json", chrono::Utc::now().timestamp()));
        let data = serde_json::to_vec_pretty(snapshot)?;
        fs::write(path, data)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub executed: Vec<NodeState>,
    pub checkpoint: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::Lane;

    #[tokio::test]
    async fn cache_hits_skip_execution() {
        let mut graph = CRCGraph::new();
        let _node_id = graph.add_node(GraphNode::new("analyze", NodeKind::Analyze, Lane::Fast));
        let engine = Engine::new(graph);
        let tmp = tempfile::tempdir().unwrap();
        let first = engine.run(tmp.path()).await.unwrap();
        assert_eq!(first.executed.len(), 1);
        let second = engine.run(tmp.path()).await.unwrap();
        assert_eq!(second.executed.len(), 1);
        assert_eq!(first.executed[0].cache_key, second.executed[0].cache_key);
    }
}
