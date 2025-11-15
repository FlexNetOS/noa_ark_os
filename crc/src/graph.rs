use std::collections::{BTreeMap, BTreeSet};

use anyhow::{anyhow, Result};
use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};

use crate::ir::{Lane, NodeId, NodeState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum NodeKind {
    Analyze = 1,
    Decide = 2,
    Transform = 3,
    Verify = 4,
    Persist = 5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: NodeId,
    pub name: String,
    pub kind: NodeKind,
    pub lane: Lane,
    pub metadata: BTreeMap<String, serde_json::Value>,
}

impl GraphNode {
    pub fn new(name: impl Into<String>, kind: NodeKind, lane: Lane) -> Self {
        Self {
            id: NodeId::new(),
            name: name.into(),
            kind,
            lane,
            metadata: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct CRCGraph {
    graph: DiGraph<GraphNode, ()>,
    nodes: BTreeMap<NodeId, NodeIndex>,
    dependencies: BTreeMap<NodeId, BTreeSet<NodeId>>,
}

impl CRCGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            nodes: BTreeMap::new(),
            dependencies: BTreeMap::new(),
        }
    }

    pub fn add_node(&mut self, node: GraphNode) -> NodeId {
        let id = node.id.clone();
        let idx = self.graph.add_node(node);
        self.nodes.insert(id.clone(), idx);
        self.dependencies.entry(id.clone()).or_default();
        id
    }

    pub fn add_edge(&mut self, from: &NodeId, to: &NodeId) -> Result<()> {
        let from_idx = *self
            .nodes
            .get(from)
            .ok_or_else(|| anyhow!("unknown from node"))?;
        let to_idx = *self
            .nodes
            .get(to)
            .ok_or_else(|| anyhow!("unknown to node"))?;
        self.graph.update_edge(from_idx, to_idx, ());
        self.dependencies
            .entry(to.clone())
            .or_default()
            .insert(from.clone());
        Ok(())
    }

    pub fn node(&self, id: &NodeId) -> Option<&GraphNode> {
        self.nodes
            .get(id)
            .and_then(|idx| self.graph.node_weight(*idx))
    }

    pub fn nodes(&self) -> impl Iterator<Item = &GraphNode> {
        self.graph.node_weights()
    }

    pub fn topo_order(&self) -> Result<Vec<NodeId>> {
        let order = toposort(&self.graph, None).map_err(|cycle| {
            anyhow!(
                "cycle detected at node {:?}",
                self.graph.node_weight(cycle.node_id())
            )
        })?;
        Ok(order
            .into_iter()
            .filter_map(|idx| self.graph.node_weight(idx).map(|node| node.id.clone()))
            .collect())
    }

    pub fn to_states(&self) -> Vec<NodeState> {
        self.graph
            .node_indices()
            .filter_map(|idx| {
                self.graph.node_weight(idx).map(|node| NodeState {
                    id: node.id.clone(),
                    kind: node.kind,
                    lane: node.lane,
                    facets: vec![],
                    io: crate::ir::NodeIo::new(),
                    dependencies: self.dependencies.get(&node.id).cloned().unwrap_or_default(),
                    cache_key: blake3::hash(node.name.as_bytes()).to_hex().to_string(),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn topo_returns_sorted_nodes() {
        let mut graph = CRCGraph::new();
        let a = graph.add_node(GraphNode::new("analyze", NodeKind::Analyze, Lane::Fast));
        let b = graph.add_node(GraphNode::new("decide", NodeKind::Decide, Lane::Fast));
        graph.add_edge(&a, &b).unwrap();
        let order = graph.topo_order().unwrap();
        assert_eq!(order, vec![a, b]);
    }
}
