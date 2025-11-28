use std::fs;
use std::path::Path;

use serde::Serialize;

use super::graph::{NodeKind, WorldGraph};

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DriftIssue {
    Missing,
    KindMismatch,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Drift {
    pub node_id: String,
    pub relative_path: String,
    pub path: String,
    pub expected_kind: NodeKind,
    pub issue: DriftIssue,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RemediationStep {
    pub node_id: String,
    pub action: String,
    pub relative_path: String,
    pub path: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ReconciliationReport {
    pub drifts: Vec<Drift>,
    pub remediation: Vec<RemediationStep>,
}

impl ReconciliationReport {
    pub fn is_clean(&self) -> bool {
        self.drifts.is_empty()
    }
}

#[derive(Debug)]
pub struct Reconciler {
    graph: WorldGraph,
}

impl Reconciler {
    pub fn new(graph: WorldGraph) -> Self {
        Self { graph }
    }

    pub fn graph(&self) -> &WorldGraph {
        &self.graph
    }

    pub fn diff(&self, repo_root: impl AsRef<Path>) -> ReconciliationReport {
        let repo_root = repo_root.as_ref();
        let mut drifts = Vec::new();

        for node in &self.graph.nodes {
            let target_path = node.as_path(repo_root);
            match fs::metadata(&target_path) {
                Ok(metadata) => {
                    if !self.kind_matches(node, &metadata) {
                        drifts.push(Drift {
                            node_id: node.id.clone(),
                            relative_path: node.path.clone(),
                            path: target_path.display().to_string(),
                            expected_kind: node.kind.clone(),
                            issue: DriftIssue::KindMismatch,
                        });
                    }
                }
                Err(_) => {
                    drifts.push(Drift {
                        node_id: node.id.clone(),
                        relative_path: node.path.clone(),
                        path: target_path.display().to_string(),
                        expected_kind: node.kind.clone(),
                        issue: DriftIssue::Missing,
                    });
                }
            }
        }

        let remediation = drifts
            .iter()
            .map(|drift| self.plan_for_drift(drift))
            .collect();

        ReconciliationReport {
            drifts,
            remediation,
        }
    }

    fn kind_matches(&self, node: &super::graph::Node, metadata: &fs::Metadata) -> bool {
        match node.kind {
            NodeKind::Directory => metadata.is_dir(),
            NodeKind::File => metadata.is_file(),
            NodeKind::Service | NodeKind::Dataset => metadata.is_dir() || metadata.is_file(),
        }
    }

    fn plan_for_drift(&self, drift: &Drift) -> RemediationStep {
        let action = match (&drift.issue, &drift.expected_kind) {
            (DriftIssue::Missing, NodeKind::Directory) => "create_directory",
            (DriftIssue::Missing, NodeKind::File) => "create_file",
            (DriftIssue::Missing, NodeKind::Service) => "register_service",
            (DriftIssue::Missing, NodeKind::Dataset) => "register_dataset",
            (DriftIssue::KindMismatch, NodeKind::Directory) => "ensure_directory",
            (DriftIssue::KindMismatch, NodeKind::File) => "ensure_file",
            (DriftIssue::KindMismatch, NodeKind::Service) => "ensure_service",
            (DriftIssue::KindMismatch, NodeKind::Dataset) => "ensure_dataset",
        };

        let description = match drift.issue {
            DriftIssue::Missing => format!(
                "Path `{}` is missing. Provision the {} and update the world graph once complete.",
                drift.path,
                drift.expected_kind_label()
            ),
            DriftIssue::KindMismatch => format!(
                "Path `{}` exists but does not match expected kind {}. Harmonize filesystem layout or update the world graph.",
                drift.path,
                drift.expected_kind_label()
            ),
        };

        RemediationStep {
            node_id: drift.node_id.clone(),
            action: action.to_string(),
            relative_path: drift.relative_path.clone(),
            path: drift.path.clone(),
            description,
        }
    }
}

impl Drift {
    fn expected_kind_label(&self) -> &'static str {
        match self.expected_kind {
            NodeKind::Directory => "directory",
            NodeKind::File => "file",
            NodeKind::Service => "service",
            NodeKind::Dataset => "dataset",
        }
    }
}
