use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LedgerAction {
    Approval,
    Rollback,
    RollbackSimulation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub timestamp: u64,
    pub drop_id: String,
    pub sandbox: String,
    pub action: LedgerAction,
    #[serde(default)]
    pub actor: Option<String>,
    #[serde(default)]
    pub trust_score: Option<f32>,
    #[serde(default)]
    pub details: Option<String>,
    #[serde(default)]
    pub artifacts: Vec<String>,
}

impl LedgerEntry {
    pub fn new(
        drop_id: impl Into<String>,
        sandbox: impl Into<String>,
        action: LedgerAction,
    ) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            drop_id: drop_id.into(),
            sandbox: sandbox.into(),
            action,
            actor: None,
            trust_score: None,
            details: None,
            artifacts: Vec::new(),
        }
    }

    pub fn with_trust_score(mut self, score: f32) -> Self {
        self.trust_score = Some(score);
        self
    }

    pub fn with_actor(mut self, actor: impl Into<String>) -> Self {
        self.actor = Some(actor.into());
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn with_artifact(mut self, artifact: impl Into<String>) -> Self {
        self.artifacts.push(artifact.into());
        self
    }
}

#[derive(Debug, Clone)]
pub struct AuditLedger {
    path: PathBuf,
}

impl AuditLedger {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub async fn append(&self, entry: &LedgerEntry) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .await?;

        let serialized = serde_json::to_string(entry)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        file.write_all(serialized.as_bytes()).await?;
        file.write_all(b"\n").await?;
        file.flush().await
    }

    pub async fn load_entries(&self) -> std::io::Result<Vec<LedgerEntry>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }

        let content = tokio::fs::read_to_string(&self.path).await?;
        let mut entries = Vec::new();

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<LedgerEntry>(line) {
                Ok(entry) => entries.push(entry),
                Err(err) => warn!("Failed to parse ledger entry: {}", err),
            }
        }

        Ok(entries)
    }
}
