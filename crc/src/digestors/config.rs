use std::fs;
use std::path::Path;

use anyhow::Result;
use walkdir::WalkDir;

use super::{compute_trust, AssetKind, AssetRecord, Digestor};

pub struct ConfigDigestor;

impl Digestor for ConfigDigestor {
    fn name(&self) -> &str {
        "config"
    }

    fn digest(&self, root: &Path) -> Result<Vec<AssetRecord>> {
        let mut assets = Vec::new();
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default();
            if !(file_name.ends_with(".toml")
                || file_name.ends_with(".yaml")
                || file_name.ends_with(".yml")
                || file_name.ends_with(".json"))
            {
                continue;
            }
            let contents = fs::read(path)?;
            let digest = blake3::hash(&contents);
            assets.push(AssetRecord {
                path: path
                    .strip_prefix(root)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .into(),
                digest: digest.to_string(),
                kind: AssetKind::Config,
                provenance: "config-scan".into(),
                trust: compute_trust("config", true),
            });
        }
        Ok(assets)
    }
}
