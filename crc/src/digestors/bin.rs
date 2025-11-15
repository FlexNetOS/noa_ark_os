use std::fs;
use std::path::Path;

use anyhow::Result;
use walkdir::WalkDir;

use super::{compute_trust, AssetKind, AssetRecord, Digestor};

pub struct BinaryDigestor;

impl Digestor for BinaryDigestor {
    fn name(&self) -> &str {
        "binary"
    }

    fn digest(&self, root: &Path) -> Result<Vec<AssetRecord>> {
        let mut assets = Vec::new();
        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let metadata = entry.metadata()?;
            if metadata.len() == 0 {
                continue;
            }
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ["exe", "bin", "wasm"].contains(&ext) {
                    let contents = fs::read(path)?;
                    let digest = blake3::hash(&contents);
                    assets.push(AssetRecord {
                        path: path
                            .strip_prefix(root)
                            .unwrap_or(path)
                            .to_string_lossy()
                            .into(),
                        digest: digest.to_string(),
                        kind: AssetKind::Binary,
                        provenance: "binary".into(),
                        trust: compute_trust("binary", true),
                    });
                }
            }
        }
        Ok(assets)
    }
}
