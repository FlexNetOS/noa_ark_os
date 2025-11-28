use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

/// Default directory for the on-disk content addressed store.
const DEFAULT_CAS_DIR: &str = "storage/cas";

/// Simple content addressed storage that uses blake3 for addressing.
#[derive(Clone, Debug)]
pub struct Cas {
    root: PathBuf,
}

impl Cas {
    /// Create a new CAS rooted at the provided path.
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self> {
        let root = root.as_ref().to_path_buf();
        if !root.exists() {
            fs::create_dir_all(&root)
                .with_context(|| format!("failed to create CAS root at {}", root.display()))?;
        }
        Ok(Self { root })
    }

    /// Create a CAS instance using env var `CRC_CAS_DIR` or default directory.
    pub fn from_env_or_default() -> Result<Self> {
        let root = std::env::var("CRC_CAS_DIR").unwrap_or_else(|_| DEFAULT_CAS_DIR.to_string());
        Self::new(root)
    }

    /// Store the provided bytes and return the stable blake3 hash.
    pub fn put_bytes(&self, bytes: &[u8]) -> Result<String> {
        let hash = blake3::hash(bytes).to_hex().to_string();
        let path = self.path_for(&hash);
        if path.exists() {
            return Ok(hash);
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create CAS bucket {}", parent.display()))?;
        }

        let mut tmp = NamedTempFile::new_in(path.parent().unwrap_or(&self.root))?;
        tmp.write_all(bytes)?;
        tmp.flush()?;
        tmp.persist(&path)
            .with_context(|| format!("failed to persist CAS object at {}", path.display()))?;
        Ok(hash)
    }

    /// Store the contents of the provided reader, returning the blake3 hash.
    pub fn put_reader<R: Read>(&self, mut reader: R) -> Result<String> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        self.put_bytes(&buffer)
    }

    /// Returns true if the hash exists in the store.
    pub fn exists(&self, hash: &str) -> bool {
        self.path_for(hash).exists()
    }

    /// Retrieve the bytes for a given hash.
    pub fn get(&self, hash: &str) -> Result<Vec<u8>> {
        let path = self
            .path_for(hash)
            .canonicalize()
            .with_context(|| format!("missing CAS object {hash}"))?;
        Ok(fs::read(path)?)
    }

    /// Retrieve metadata for a given hash.
    pub fn stat(&self, hash: &str) -> Result<CasEntry> {
        let path = self
            .path_for(hash)
            .canonicalize()
            .with_context(|| format!("missing CAS object {hash}"))?;
        let meta = fs::metadata(&path)?;
        let modified = meta
            .modified()
            .ok()
            .and_then(|mtime| mtime.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Ok(CasEntry {
            hash: hash.to_string(),
            path,
            size: meta.len(),
            created_ts: modified,
        })
    }

    fn path_for(&self, hash: &str) -> PathBuf {
        let (bucket, remainder) = hash.split_at(2);
        self.root.join(bucket).join(remainder)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CasEntry {
    pub hash: String,
    pub path: PathBuf,
    pub size: u64,
    pub created_ts: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn put_and_get_round_trip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cas = Cas::new(temp_dir.path()).unwrap();
        let input = b"hello world";
        let hash = cas.put_bytes(input).unwrap();
        assert!(cas.exists(&hash));
        let fetched = cas.get(&hash).unwrap();
        assert_eq!(input.to_vec(), fetched);
        let stats = cas.stat(&hash).unwrap();
        assert_eq!(stats.size, input.len() as u64);
    }

    #[test]
    fn put_reader_produces_same_hash() {
        let temp_dir = tempfile::tempdir().unwrap();
        let cas = Cas::new(temp_dir.path()).unwrap();
        let first = cas.put_bytes(b"example data").unwrap();
        let second = cas
            .put_reader(Cursor::new(Vec::from("example data")))
            .unwrap();
        assert_eq!(first, second);
    }
}
