use std::fs;
use std::path::Path;

use anyhow::Result;

use super::{compute_trust, AssetKind, AssetRecord, Digestor};

#[derive(Default)]
pub struct GitDigestor;

impl Digestor for GitDigestor {
    fn name(&self) -> &str {
        "git"
    }

    fn digest(&self, root: &Path) -> Result<Vec<AssetRecord>> {
        let git_dir = root.join(".git");
        if !git_dir.exists() {
            return Ok(vec![]);
        }
        let head = fs::read_to_string(git_dir.join("HEAD")).unwrap_or_default();
        let digest = blake3::hash(head.as_bytes());
        Ok(vec![AssetRecord {
            path: git_dir.to_string_lossy().into(),
            digest: digest.to_string(),
            kind: AssetKind::Git,
            provenance: "git-head".into(),
            trust: compute_trust("git", !head.is_empty()),
        }])
    }
}
