use std::path::PathBuf;

use noa_plugin_sdk::registry::ToolRegistry;

#[test]
fn registry_parses_and_contains_expected_categories() -> anyhow::Result<()> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|dir| dir.parent())
        .expect("workspace root");
    let registry_path = workspace_root
        .join("registry")
        .join("tools.registry.json");

    let registry = ToolRegistry::from_path(&registry_path)?;
    registry.ensure_categories(&["observability", "automation", "analysis", "collaboration", "plugin"])?;

    for tool in registry.tools {
        assert!(tool.budgets.cpu_millis > 0, "tool {} missing cpu budget", tool.id);
        assert!(tool.budgets.max_duration_seconds > 0, "tool {} missing duration budget", tool.id);
        assert!(
            !tool.cli_mappings.is_empty(),
            "tool {} must declare at least one CLI mapping",
            tool.id
        );
        for parameter in tool.parameters.iter() {
            assert!(
                parameter.required || parameter.default.is_some(),
                "parameter {} in tool {} must be required or have a default",
                parameter.name,
                tool.id
            );
        }
    }

    Ok(())
}
