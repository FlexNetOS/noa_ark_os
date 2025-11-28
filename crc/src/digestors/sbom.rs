use std::fs;
use std::path::Path;

use anyhow::Result;
use walkdir::WalkDir;

use super::{compute_trust, AssetKind, AssetRecord, Digestor};

pub struct SbomDigestor;

impl Digestor for SbomDigestor {
    fn name(&self) -> &str {
        "sbom"
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
            if !name.contains("sbom") {
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
                kind: AssetKind::Sbom,
                provenance: "sbom".into(),
                trust: compute_trust("sbom", true),
            });
        }
        Ok(assets)
    }
}
