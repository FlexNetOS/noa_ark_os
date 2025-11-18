use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::indexer::{should_skip, IndexerError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigGraph {
    pub generated_at: u128,
    pub manifests: Vec<ManifestNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestNode {
    pub path: String,
    pub package: Option<String>,
    pub dependencies: Vec<ConfigDependency>,
    pub workspace_members: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDependency {
    pub name: String,
    pub requirement: Option<String>,
    pub optional: bool,
}

impl ConfigGraph {
    pub fn build(root: impl AsRef<Path>) -> Result<Self, IndexerError> {
        let root = root.as_ref();
        let mut manifests = Vec::new();

        for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.file_name() != "Cargo.toml" {
                continue;
            }
            let relative = entry
                .path()
                .strip_prefix(root)
                .unwrap_or(entry.path())
                .to_path_buf();
            if should_skip(&relative) {
                continue;
            }
            let manifest = parse_manifest(entry.path(), root, &relative)?;
            manifests.push(manifest);
        }

        manifests.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(Self {
            generated_at: crate::utils::current_timestamp_millis(),
            manifests,
        })
    }
}

fn parse_manifest(path: &Path, root: &Path, relative: &Path) -> Result<ManifestNode, IndexerError> {
    let contents = fs::read_to_string(path)?;
    let value: toml::Value = toml::from_str(&contents)?;
    let package = value
        .get("package")
        .and_then(|pkg| pkg.get("name"))
        .and_then(toml::Value::as_str)
        .map(|name| name.to_string());

    let dependencies = value
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .map(parse_dependencies)
        .unwrap_or_default();

    let manifest_dir = path.parent().unwrap_or(root);
    let workspace_members = value
        .get("workspace")
        .and_then(|ws| ws.get("members"))
        .and_then(toml::Value::as_array)
        .map(|array| resolve_workspace_members(array, manifest_dir, root))
        .unwrap_or_default();

    Ok(ManifestNode {
        path: relative.to_string_lossy().to_string(),
        package,
        dependencies,
        workspace_members,
    })
}

fn parse_dependencies(table: &toml::map::Map<String, toml::Value>) -> Vec<ConfigDependency> {
    let mut deps = Vec::new();
    for (name, value) in table {
        match value {
            toml::Value::String(req) => deps.push(ConfigDependency {
                name: name.clone(),
                requirement: Some(req.clone()),
                optional: false,
            }),
            toml::Value::Boolean(flag) => deps.push(ConfigDependency {
                name: name.clone(),
                requirement: None,
                optional: !flag,
            }),
            toml::Value::Table(table) => {
                let requirement = table.get("version").or_else(|| table.get("path")).and_then(
                    |value| match value {
                        toml::Value::String(s) => Some(s.clone()),
                        _ => None,
                    },
                );
                let optional = table
                    .get("optional")
                    .and_then(toml::Value::as_bool)
                    .unwrap_or(false);
                deps.push(ConfigDependency {
                    name: name.clone(),
                    requirement,
                    optional,
                });
            }
            _ => deps.push(ConfigDependency {
                name: name.clone(),
                requirement: None,
                optional: false,
            }),
        }
    }
    deps
}

fn resolve_workspace_members(
    entries: &[toml::Value],
    manifest_dir: &Path,
    workspace_root: &Path,
) -> Vec<String> {
    let mut members = Vec::new();
    for raw in entries.iter().filter_map(toml::Value::as_str) {
        if contains_glob(raw) {
            members.push(raw.to_string());
            continue;
        }

        let candidate = manifest_dir.join(raw);
        let manifest_path = normalize_manifest_path(candidate);
        let relative = manifest_path
            .strip_prefix(workspace_root)
            .unwrap_or(&manifest_path)
            .to_path_buf();

        members.push(relative.to_string_lossy().to_string());
    }
    members
}

fn contains_glob(value: &str) -> bool {
    value.contains('*') || value.contains('?') || value.contains('[')
}

fn normalize_manifest_path(mut path: PathBuf) -> PathBuf {
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("toml"))
        == Some(true)
    {
        return path;
    }

    if path.ends_with("Cargo.toml") {
        return path;
    }

    path.push("Cargo.toml");
    path
}
