use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

use crate::{SymbolGraph, SymbolNode};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotebookSymbolMetadata {
    pub stable_id: String,
    pub name: String,
    pub kind: String,
    pub file: String,
    pub signature: String,
    pub span: (usize, usize),
}

impl From<&SymbolNode> for NotebookSymbolMetadata {
    fn from(node: &SymbolNode) -> Self {
        Self {
            stable_id: node.stable_id.clone(),
            name: node.name.clone(),
            kind: node.kind.clone(),
            file: node.file.clone(),
            signature: node.signature.clone(),
            span: node.span,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotebookSymbolChangeKind {
    Added,
    Removed,
    Updated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotebookSymbolChange {
    pub stable_id: String,
    pub change: NotebookSymbolChangeKind,
    pub metadata: Option<NotebookSymbolMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NotebookMetadataDiff {
    pub generated_at: String,
    pub changed: Vec<NotebookSymbolChange>,
}

impl NotebookMetadataDiff {
    pub fn empty() -> Self {
        Self {
            generated_at: OffsetDateTime::now_utc()
                .format(&Rfc3339)
                .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into()),
            changed: Vec::new(),
        }
    }

    pub fn new(changed: Vec<NotebookSymbolChange>) -> Self {
        Self {
            generated_at: OffsetDateTime::now_utc()
                .format(&Rfc3339)
                .unwrap_or_else(|_| "1970-01-01T00:00:00Z".into()),
            changed,
        }
    }

    pub fn from_graphs(previous: &SymbolGraph, current: &SymbolGraph) -> Self {
        let mut changes: Vec<NotebookSymbolChange> = Vec::new();
        let mut previous_map: BTreeMap<&String, &SymbolNode> = BTreeMap::new();
        for (stable_id, node) in &previous.nodes {
            previous_map.insert(stable_id, node);
        }

        for (stable_id, node) in &current.nodes {
            match previous_map.get(stable_id) {
                None => changes.push(NotebookSymbolChange {
                    stable_id: stable_id.clone(),
                    change: NotebookSymbolChangeKind::Added,
                    metadata: Some(node.into()),
                }),
                Some(prev_node) => {
                    if metadata_changed(prev_node, node) {
                        changes.push(NotebookSymbolChange {
                            stable_id: stable_id.clone(),
                            change: NotebookSymbolChangeKind::Updated,
                            metadata: Some(node.into()),
                        });
                    }
                }
            }
        }

        for (stable_id, node) in &previous.nodes {
            if !current.nodes.contains_key(stable_id) {
                changes.push(NotebookSymbolChange {
                    stable_id: stable_id.clone(),
                    change: NotebookSymbolChangeKind::Removed,
                    metadata: Some(node.into()),
                });
            }
        }

        Self::new(changes)
    }

    pub fn has_changes(&self) -> bool {
        !self.changed.is_empty()
    }
}

fn metadata_changed(previous: &SymbolNode, current: &SymbolNode) -> bool {
    previous.name != current.name
        || previous.kind != current.kind
        || previous.file != current.file
        || previous.signature != current.signature
        || previous.span != current.span
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_node(name: &str, file: &str) -> SymbolNode {
        SymbolNode {
            stable_id: format!("id::{name}"),
            language: "rust".into(),
            name: name.into(),
            kind: "function".into(),
            file: file.into(),
            signature: format!("fn {name}()"),
            span: (1, 2),
        }
    }

    #[test]
    fn detects_added_symbol() {
        let previous = SymbolGraph::default();
        let mut current = SymbolGraph::default();
        let node = sample_node("alpha", "lib.rs");
        current.nodes.insert(node.stable_id.clone(), node.clone());

        let diff = NotebookMetadataDiff::from_graphs(&previous, &current);
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(diff.changed[0].change, NotebookSymbolChangeKind::Added);
        assert_eq!(diff.changed[0].metadata.as_ref().unwrap().name, "alpha");
    }

    #[test]
    fn detects_updated_symbol() {
        let mut previous = SymbolGraph::default();
        let mut current = SymbolGraph::default();
        let node_prev = sample_node("beta", "lib.rs");
        let mut node_cur = node_prev.clone();
        node_cur.file = "src/lib.rs".into();
        previous
            .nodes
            .insert(node_prev.stable_id.clone(), node_prev);
        current
            .nodes
            .insert(node_cur.stable_id.clone(), node_cur.clone());

        let diff = NotebookMetadataDiff::from_graphs(&previous, &current);
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(diff.changed[0].change, NotebookSymbolChangeKind::Updated);
        assert_eq!(
            diff.changed[0].metadata.as_ref().unwrap().file,
            "src/lib.rs"
        );
    }

    #[test]
    fn detects_removed_symbol() {
        let mut previous = SymbolGraph::default();
        let current = SymbolGraph::default();
        let node = sample_node("gamma", "lib.rs");
        previous.nodes.insert(node.stable_id.clone(), node.clone());

        let diff = NotebookMetadataDiff::from_graphs(&previous, &current);
        assert_eq!(diff.changed.len(), 1);
        assert_eq!(diff.changed[0].change, NotebookSymbolChangeKind::Removed);
        assert_eq!(diff.changed[0].metadata.as_ref().unwrap().name, "gamma");
    }
}
