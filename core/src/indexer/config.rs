use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::indexer::IndexerError;

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
            let manifest = parse_manifest(entry.path(), root)?;
            manifests.push(manifest);
        }

        manifests.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(Self {
            generated_at: crate::utils::current_timestamp_millis(),
            manifests,
        })
    }
}

fn parse_manifest(path: &Path, root: &Path) -> Result<ManifestNode, IndexerError> {
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

    let workspace_members = value
        .get("workspace")
        .and_then(|ws| ws.get("members"))
        .and_then(toml::Value::as_array)
        .map(|array| {
            array
                .iter()
                .filter_map(toml::Value::as_str)
                .map(|s| s.to_string())
                .collect()
        })
        .unwrap_or_default();

    let relative = path.strip_prefix(root).unwrap_or(path);

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
