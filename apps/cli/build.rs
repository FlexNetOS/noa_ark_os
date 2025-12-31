use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Gather descriptors from tools-agent
    let list = noa_tools_agent::descriptors::agent_tool_descriptors();
    let git_sha = git_rev();
    let build_ts = chrono::Utc::now().to_rfc3339();
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // Workspace root assumed two levels up (apps/cli)
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&manifest_dir);
    let registry_dir = workspace_root.join("registry");
    let catalog_path = registry_dir.join("tooling.catalog.json");
    fs::create_dir_all(&registry_dir).ok();
    let value = serde_json::json!({
        "metadata": {
            "version": env!("CARGO_PKG_VERSION"),
            "git_sha": git_sha,
            "build_timestamp": build_ts,
            "descriptor_sources": ["tools-agent"],
            "tool_count": list.len()
        },
        "tools": list
    });
    if let Err(e) = fs::write(&catalog_path, serde_json::to_vec_pretty(&value).unwrap()) {
        eprintln!("Failed to write tooling catalog: {e}");
    } else {
        println!("cargo:rerun-if-changed=server/tools_agent/src/descriptors.rs");
    }
}

fn git_rev() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".into())
}
