use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::Result;

/// Target hardware profile the CRC automation optimizes for.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TargetProfile {
    Edge,
    Server,
}

impl TargetProfile {
    pub fn artifact_directory(&self) -> &'static str {
        match self {
            TargetProfile::Edge => "edge",
            TargetProfile::Server => "server",
        }
    }

    pub fn optimization_flags(&self) -> Vec<&'static str> {
        match self {
            TargetProfile::Edge => vec![
                "-C",
                "opt-level=z",
                "-C",
                "lto=fat",
                "-C",
                "codegen-units=1",
            ],
            TargetProfile::Server => vec!["-C", "opt-level=3", "-C", "lto=thin"],
        }
    }

    pub fn feature_gates(&self) -> Vec<&'static str> {
        match self {
            TargetProfile::Edge => vec!["low-power", "quantized-kernels"],
            TargetProfile::Server => vec!["throughput", "gpu-offload"],
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            TargetProfile::Edge => "Edge devices prioritizing thermal and power limits",
            TargetProfile::Server => "Throughput-optimized server with abundant resources",
        }
    }
}

/// Manifest describing an optimized build that CRC produced.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildManifest {
    pub drop_id: String,
    pub profile: TargetProfile,
    pub source_path: String,
    pub optimization_flags: Vec<String>,
    pub feature_gates: Vec<String>,
    pub timestamp: u64,
}

/// Materialized artifact location for a compiled profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifact {
    pub manifest: BuildManifest,
    pub artifact_path: PathBuf,
    pub notes: Vec<String>,
}

/// Generate optimized builds for each target profile and persist manifests.
pub async fn generate_optimized_builds(
    drop_id: &str,
    source_path: &Path,
    artifact_root: &Path,
) -> Result<Vec<BuildArtifact>> {
    let mut artifacts = Vec::new();

    for profile in [TargetProfile::Edge, TargetProfile::Server] {
        let profile_root = artifact_root
            .join(profile.artifact_directory())
            .join(drop_id);
        fs::create_dir_all(&profile_root).await?;

        let manifest = BuildManifest {
            drop_id: drop_id.to_string(),
            profile,
            source_path: source_path.display().to_string(),
            optimization_flags: profile
                .optimization_flags()
                .into_iter()
                .map(|flag| flag.to_string())
                .collect(),
            feature_gates: profile
                .feature_gates()
                .into_iter()
                .map(|flag| flag.to_string())
                .collect(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };

        let manifest_yaml = serde_yaml::to_string(&manifest)?;
        let manifest_path = profile_root.join("manifest.yaml");
        fs::write(&manifest_path, manifest_yaml).await?;

        let flags_path = profile_root.join("flags.txt");
        let flags_body = manifest.optimization_flags.join(" ");
        fs::write(&flags_path, flags_body).await?;

        let notes = vec![
            format!("Optimizations tuned for {}", profile.description()),
            "Artifacts are ready for downstream packaging".to_string(),
        ];

        artifacts.push(BuildArtifact {
            manifest,
            artifact_path: profile_root,
            notes,
        });
    }

    Ok(artifacts)
}
