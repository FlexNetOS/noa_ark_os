use std::path::Path;

use anyhow::Result;
use serde::{Deserialize, Serialize};

pub mod api;
pub mod bin;
pub mod config;
pub mod git;
pub mod sbom;

pub trait Digestor: Send + Sync {
    fn name(&self) -> &str;
    fn digest(&self, root: &Path) -> Result<Vec<AssetRecord>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRecord {
    pub path: String,
    pub digest: String,
    pub kind: AssetKind,
    pub provenance: String,
    pub trust: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetKind {
    Git,
    Config,
    Api,
    Sbom,
    Binary,
    Other,
}

pub fn compute_trust(provenance: &str, success: bool) -> f32 {
    let base = if success { 0.9 } else { 0.2 };
    if provenance.contains("verified") {
        1.0
    } else {
        base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trust_increases_with_verified_tag() {
        assert!(compute_trust("verified", true) > compute_trust("source", true));
    }
}
