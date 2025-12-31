use crate::config::TrainingPlan;
use agents::implementations::ml_controller::{ControllerError, ControllerResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Serialize, Clone)]
pub struct ArtifactRecord {
    pub lifecycle_id: String,
    pub artifact_path: PathBuf,
    pub checksum: String,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait ModelArtifactStore: Send + Sync {
    async fn persist_artifact(
        &self,
        generated_path: &Path,
        plan: &TrainingPlan,
    ) -> ControllerResult<ArtifactRecord>;

    async fn record_verification(
        &self,
        artifact: &ArtifactRecord,
        evaluation_metrics: &serde_json::Value,
    ) -> ControllerResult<()>;

    async fn flush_registry(
        &self,
        registry: &(dyn RegistryGateway + Send + Sync),
        artifact: &ArtifactRecord,
        evaluation_metrics: &serde_json::Value,
    ) -> ControllerResult<()> {
        registry.append_metadata(artifact, evaluation_metrics).await
    }
}

#[async_trait]
pub trait RegistryGateway: Send + Sync {
    fn ensure_capability_registered(&self) -> ControllerResult<()>;
    async fn append_metadata(
        &self,
        artifact: &ArtifactRecord,
        evaluation_metrics: &serde_json::Value,
    ) -> ControllerResult<()>;
}

pub struct FilesystemArtifactStore {
    root: PathBuf,
    evidence_ledger: PathBuf,
}

impl FilesystemArtifactStore {
    pub fn new(root: PathBuf, evidence_ledger: PathBuf) -> Self {
        Self {
            root,
            evidence_ledger,
        }
    }

    fn destination_path(&self, plan: &TrainingPlan) -> PathBuf {
        let timestamp = Utc::now().format("%Y%m%dT%H%M%S");
        self.root
            .join(&plan.lifecycle_id)
            .join(plan.agent_profile.replace(' ', "_"))
            .join(timestamp.to_string())
            .join("model.bin")
    }

    fn checksum(path: &Path) -> ControllerResult<String> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 4096];
        loop {
            let bytes = reader.read(&mut buffer)?;
            if bytes == 0 {
                break;
            }
            hasher.update(&buffer[..bytes]);
        }
        Ok(hex::encode(hasher.finalize()))
    }

    async fn ensure_parent(path: &Path) -> ControllerResult<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        Ok(())
    }

    async fn persist_manifest(&self, record: &ArtifactRecord) -> ControllerResult<()> {
        let manifest_path = record.artifact_path.parent().unwrap().join("manifest.json");
        let manifest = serde_json::json!({
            "lifecycle_id": record.lifecycle_id,
            "artifact": record.artifact_path.to_string_lossy().to_string(),
            "checksum": record.checksum,
            "created_at": record.created_at,
        });
        let mut file = fs::File::create(&manifest_path).await?;
        file.write_all(manifest.to_string().as_bytes()).await?;
        Ok(())
    }
}

#[async_trait]
impl ModelArtifactStore for FilesystemArtifactStore {
    async fn persist_artifact(
        &self,
        generated_path: &Path,
        plan: &TrainingPlan,
    ) -> ControllerResult<ArtifactRecord> {
        let destination = self.destination_path(plan);
        Self::ensure_parent(&destination).await?;
        fs::copy(generated_path, &destination).await?;

        let checksum = Self::checksum(&destination)?;
        let record = ArtifactRecord {
            lifecycle_id: plan.lifecycle_id.clone(),
            artifact_path: destination.clone(),
            checksum,
            created_at: Utc::now(),
        };

        self.persist_manifest(&record).await?;

        Ok(record)
    }

    async fn record_verification(
        &self,
        artifact: &ArtifactRecord,
        evaluation_metrics: &serde_json::Value,
    ) -> ControllerResult<()> {
        let entry = serde_json::json!({
            "timestamp": Utc::now(),
            "artifact": {
                "lifecycle_id": artifact.lifecycle_id,
                "path": artifact.artifact_path.to_string_lossy().to_string(),
                "checksum": artifact.checksum,
            },
            "evaluation": evaluation_metrics,
        });

        Self::ensure_parent(&self.evidence_ledger).await?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.evidence_ledger)
            .await?;

        file.write_all(entry.to_string().as_bytes()).await?;
        file.write_all(b"\n").await?;
        Ok(())
    }
}

pub struct CapabilityRegistry {
    manifest: PathBuf,
    capability_id: String,
}

impl CapabilityRegistry {
    pub fn new(manifest: PathBuf, capability_id: impl Into<String>) -> Self {
        Self {
            manifest,
            capability_id: capability_id.into(),
        }
    }
}

#[async_trait]
impl RegistryGateway for CapabilityRegistry {
    fn ensure_capability_registered(&self) -> ControllerResult<()> {
        let data = std::fs::read_to_string(&self.manifest)?;
        let mut manifest: serde_json::Value = serde_json::from_str(&data)?;

        let capabilities = manifest
            .get_mut("capabilities")
            .and_then(|value| value.as_array_mut())
            .ok_or_else(|| {
                ControllerError::Registry("Registry manifest missing capabilities array".into())
            })?;

        let already_present = capabilities.iter().any(|entry| {
            entry
                .get("id")
                .and_then(|value| value.as_str())
                .map(|id| id == self.capability_id)
                .unwrap_or(false)
        });

        if !already_present {
            let new_entry = serde_json::json!({
                "id": self.capability_id,
                "type": "ml",
                "semver": "0.1.0",
                "requires": ["storage.artifacts"],
                "provides": ["ml.training", "ml.evaluation"],
            });
            capabilities.push(new_entry);
            let content = serde_json::to_string_pretty(&manifest)?;
            std::fs::write(&self.manifest, content)?;
        }
        Ok(())
    }

    async fn append_metadata(
        &self,
        artifact: &ArtifactRecord,
        evaluation_metrics: &serde_json::Value,
    ) -> ControllerResult<()> {
        let mut metadata_path = self.manifest.clone();
        metadata_path.set_file_name("ml_artifacts.log");
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&metadata_path)
            .await?;

        let entry = serde_json::json!({
            "capability": self.capability_id,
            "artifact": {
                "path": artifact.artifact_path.to_string_lossy().to_string(),
                "checksum": artifact.checksum,
                "lifecycle_id": artifact.lifecycle_id,
            },
            "metrics": evaluation_metrics,
            "timestamp": Utc::now(),
        });
        file.write_all(entry.to_string().as_bytes()).await?;
        file.write_all(b"\n").await?;
        Ok(())
    }
}
