use std::fs::File;
use std::path::{Path, PathBuf};

use noa_core::world::{Reconciler, ReconciliationReport, WorldGraph};
use serde_json::Value;

fn repo_root() -> PathBuf {
    WorldGraph::repo_root()
}

fn load_json_fixture(name: &str) -> Value {
    let path = repo_root().join("tests/world_model").join(name);
    let file = File::open(&path).expect("fixture must exist");
    serde_json::from_reader(file).expect("fixture must parse")
}

fn assert_report_matches_fixture(report: &ReconciliationReport, expected: &Value) {
    let expected_drifts = expected
        .get("drifts")
        .and_then(Value::as_array)
        .expect("expected drift array");
    assert_eq!(
        expected_drifts.len(),
        report.drifts.len(),
        "drift count mismatch"
    );

    for (drift, expected_value) in report.drifts.iter().zip(expected_drifts.iter()) {
        let expected_node_id = expected_value
            .get("node_id")
            .and_then(Value::as_str)
            .expect("expected node id");
        assert_eq!(drift.node_id, expected_node_id, "unexpected node id");

        let expected_relative_path = expected_value
            .get("relative_path")
            .and_then(Value::as_str)
            .expect("expected relative path");
        assert_eq!(
            drift.relative_path, expected_relative_path,
            "unexpected relative path"
        );

        if let Some(expected_kind) = expected_value.get("expected_kind").and_then(Value::as_str) {
            let actual_kind = serde_json::to_value(&drift.expected_kind).unwrap();
            assert_eq!(
                Some(expected_kind),
                actual_kind.as_str(),
                "unexpected node kind"
            );
        }

        if let Some(expected_issue) = expected_value.get("issue").and_then(Value::as_str) {
            let actual_issue = serde_json::to_value(&drift.issue).unwrap();
            assert_eq!(
                Some(expected_issue),
                actual_issue.as_str(),
                "unexpected issue type"
            );
        }

        let relative_path = Path::new(&drift.relative_path);
        let absolute_path = Path::new(&drift.path);
        assert!(
            absolute_path.ends_with(relative_path),
            "absolute path {:?} should end with {:?}",
            absolute_path,
            relative_path
        );
    }

    let expected_plan = expected
        .get("remediation")
        .and_then(Value::as_array)
        .expect("expected remediation array");
    assert_eq!(
        expected_plan.len(),
        report.remediation.len(),
        "plan length mismatch"
    );

    for (step, expected_value) in report.remediation.iter().zip(expected_plan.iter()) {
        let expected_node_id = expected_value
            .get("node_id")
            .and_then(Value::as_str)
            .expect("expected remediation node id");
        assert_eq!(step.node_id, expected_node_id, "unexpected node id");

        let expected_action = expected_value
            .get("action")
            .and_then(Value::as_str)
            .expect("expected remediation action");
        assert_eq!(step.action, expected_action, "unexpected action");

        let expected_relative_path = expected_value
            .get("relative_path")
            .and_then(Value::as_str)
            .expect("expected remediation relative path");
        assert_eq!(
            step.relative_path, expected_relative_path,
            "unexpected remediation relative path"
        );
    }
}

#[test]
fn default_world_graph_parses() {
    let graph = WorldGraph::load_default().expect("seed graph should parse");
    assert!(
        graph.nodes.len() >= 3,
        "seed graph should enumerate core assets"
    );
}

#[test]
fn seed_graph_matches_repo_state() {
    let graph = WorldGraph::load_default().expect("seed graph should parse");
    let reconciler = Reconciler::new(graph);
    let report = reconciler.diff(repo_root());
    assert!(report.is_clean(), "seed graph should be in sync with repo");

    let expected = load_json_fixture("expected_clean_report.json");
    assert_report_matches_fixture(&report, &expected);
}

#[test]
fn missing_fixture_emits_remediation_plan() {
    let graph_path = repo_root().join("tests/world_model/sample_missing_graph.json");
    let graph = WorldGraph::load_from_path(graph_path).expect("fixture graph should parse");
    let reconciler = Reconciler::new(graph);
    let report = reconciler.diff(repo_root());
    assert!(!report.is_clean(), "fixture should report drift");

    let expected = load_json_fixture("expected_missing_report.json");
    assert_report_matches_fixture(&report, &expected);
}
