use serde_json::Value;
use std::fs;
use std::path::Path;

#[test]
fn tooling_catalog_generated_with_expected_tools() {
    // Workspace root two levels up from this crate.
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root = Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .unwrap();
    let catalog_path = root.join("registry/tooling.catalog.json");
    assert!(
        catalog_path.exists(),
        "tooling.catalog.json missing at {}",
        catalog_path.display()
    );
    let data: Value = serde_json::from_slice(&fs::read(&catalog_path).unwrap()).unwrap();
    let tools = data
        .get("tools")
        .and_then(|v| v.as_array())
        .expect("tools array");
    let mut ids: Vec<_> = tools
        .iter()
        .filter_map(|t| t.get("id").and_then(|v| v.as_str()))
        .collect();
    ids.sort();
    let expected = [
        "edit_file",
        "list_files",
        "read_file",
        "run_command",
        "run_tests",
    ];
    assert_eq!(ids, expected, "descriptor id mismatch");
    // Basic metadata checks
    let meta = data.get("metadata").expect("metadata present");
    assert!(meta.get("git_sha").and_then(|v| v.as_str()).is_some());
    assert!(meta
        .get("build_timestamp")
        .and_then(|v| v.as_str())
        .is_some());
}
