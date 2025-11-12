use std::fs;
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use walkdir::WalkDir;

use super::{compute_trust, AssetKind, AssetRecord, Digestor};

#[derive(Debug, Deserialize)]
struct OpenApiDoc {
    #[serde(default)]
    pub openapi: Option<String>,
    #[serde(default)]
    pub info: Option<serde_json::Value>,
}

pub struct ApiDigestor;

impl Digestor for ApiDigestor {
    fn name(&self) -> &str {
        "api"
    }

    fn digest(&self, root: &Path) -> Result<Vec<AssetRecord>> {
        let mut assets = Vec::new();
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if !(name.ends_with("openapi.json") || name.ends_with("openapi.yaml")) {
                continue;
            }
            let contents = fs::read_to_string(path)?;
            let trust = serde_json::from_str::<OpenApiDoc>(&contents).is_ok();
            let digest = blake3::hash(contents.as_bytes());
            assets.push(AssetRecord {
                path: path
                    .strip_prefix(root)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .into(),
                digest: digest.to_string(),
                kind: AssetKind::Api,
                provenance: "openapi".into(),
                trust: compute_trust("api", trust),
            });
        }
        Ok(assets)
    }
}
