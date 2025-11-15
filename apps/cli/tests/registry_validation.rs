use std::path::PathBuf;

use noa_plugin_sdk::ToolRegistry;

#[test]
fn tool_registry_is_validated_for_cli_usage() -> anyhow::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|dir| dir.parent())
        .expect("workspace root for cli");
    let registry_path = workspace_root
        .join("registry")
        .join("tools.registry.json");

    let registry = ToolRegistry::from_path(&registry_path)?;
    registry.ensure_categories(&["observability", "automation", "analysis", "collaboration", "plugin"])?;

    for tool in registry.tools {
        assert!(
            !tool.summary.is_empty(),
            "tool {} must include a summary",
            tool.id
        );
        assert!(
            !tool.parameters.is_empty(),
            "tool {} must include at least one parameter",
            tool.id
        );
        assert!(
            tool.outputs.iter().any(|output| output.format.contains("json")),
            "tool {} must emit a JSON-compatible output",
            tool.id
        );
        assert!(
            tool.side_effects.len() <= 3,
            "tool {} should declare targeted side effects",
            tool.id
        );
    }

    Ok(())
}
