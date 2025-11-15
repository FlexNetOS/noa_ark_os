use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRegistry {
    #[serde(default)]
    pub generated_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub hash: String,
    pub size: u64,
    pub permissions: Option<u32>,
    #[serde(default)]
    pub last_seen: Option<DateTime<Utc>>,
    #[serde(default)]
    pub labels: Vec<String>,
}

impl FileEntry {
    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn permissions(&self) -> u32 {
        self.permissions.unwrap_or(0o644)
    }

    pub fn last_seen(&self) -> Option<DateTime<Utc>> {
        self.last_seen
    }
}

impl FileRegistry {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self> {
        let bytes = fs::read(path.as_ref())
            .await
            .with_context(|| format!("unable to read registry {:?}", path.as_ref()))?;
        let registry: FileRegistry = serde_json::from_slice(&bytes)
            .with_context(|| "unable to deserialize file registry")?;
        Ok(registry)
    }

    pub fn entries(&self) -> &[FileEntry] {
        &self.files
    }

    pub fn by_hash(&self) -> HashMap<String, Vec<&FileEntry>> {
        let mut map: HashMap<String, Vec<&FileEntry>> = HashMap::new();
        for entry in &self.files {
            map.entry(entry.hash.clone()).or_default().push(entry);
        }
        map
    }

    pub fn duplicates(&self) -> HashMap<String, Vec<&FileEntry>> {
        self.by_hash()
            .into_iter()
            .filter(|(_, entries)| entries.len() > 1)
            .collect()
    }
}
