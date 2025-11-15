use crate::models::{MemoryCursor, MemoryRecord, MemoryRetrieval, MemoryRole};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

const LONG_TERM_FILE: &str = "long_term.cbor";
const SESSION_SUFFIX: &str = "session";

#[derive(Debug, Error)]
pub enum MemoryError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("encoding error: {0}")]
    Encoding(#[from] serde_cbor::Error),
    #[error("memory store '{0}' not found")]
    StoreMissing(String),
}

pub trait MemoryStore {
    fn append(
        &self,
        agent: &str,
        role: MemoryRole,
        content: &str,
        metadata: HashMap<String, String>,
        tags: Vec<String>,
    ) -> Result<MemoryRecord, MemoryError>;

    fn incremental(
        &self,
        cursor: u64,
        limit: usize,
    ) -> Result<(Vec<MemoryRecord>, u64), MemoryError>;
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedStore {
    records: Vec<MemoryRecord>,
}

impl PersistedStore {
    fn load(path: &Path) -> Result<Vec<MemoryRecord>, MemoryError> {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let reader = BufReader::new(File::open(path)?);
        let persisted: PersistedStore = serde_cbor::from_reader(reader)?;
        Ok(persisted.records)
    }

    fn save(path: &Path, records: &[MemoryRecord]) -> Result<(), MemoryError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let writer = BufWriter::new(File::create(path)?);
        let persisted = PersistedStore {
            records: records.to_vec(),
        };
        serde_cbor::to_writer(writer, &persisted)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct LongTermMemory {
    path: PathBuf,
    entries: RwLock<Vec<MemoryRecord>>,
    next_id: AtomicU64,
}

impl LongTermMemory {
    pub fn open(root: impl AsRef<Path>) -> Result<Self, MemoryError> {
        let path = root.as_ref().join(LONG_TERM_FILE);
        let records = PersistedStore::load(&path)?;
        let next_id = records.iter().map(|record| record.id).max().map(|max_id| max_id + 1).unwrap_or(1);
        Ok(Self {
            path,
            entries: RwLock::new(records),
            next_id: AtomicU64::new(next_id),
        })
    }

    fn persist(&self) -> Result<(), MemoryError> {
        let snapshot = self.entries.read().clone();
        PersistedStore::save(&self.path, &snapshot)
    }
}

impl MemoryStore for LongTermMemory {
    fn append(
        &self,
        agent: &str,
        role: MemoryRole,
        content: &str,
        metadata: HashMap<String, String>,
        tags: Vec<String>,
    ) -> Result<MemoryRecord, MemoryError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let record = MemoryRecord::new(
            id,
            agent.to_string(),
            role,
            content.to_string(),
            metadata,
            tags,
        );
        {
            let mut entries = self.entries.write();
            entries.push(record.clone());
        }
        self.persist()?;
        Ok(record)
    }

    fn incremental(
        &self,
        cursor: u64,
        limit: usize,
    ) -> Result<(Vec<MemoryRecord>, u64), MemoryError> {
        let entries = self.entries.read();
        let mut collected = Vec::new();
        let mut last_seen = cursor;
        for record in entries
            .iter()
            .filter(|record| record.id > cursor)
            .take(limit)
        {
            collected.push(record.clone());
            last_seen = record.id;
        }
        Ok((collected, last_seen))
    }
}

#[derive(Debug)]
pub struct SessionMemory {
    session_id: String,
    path: PathBuf,
    entries: RwLock<Vec<MemoryRecord>>,
    next_id: AtomicU64,
}

impl SessionMemory {
    pub fn open(
        root: impl AsRef<Path>,
        session_id: impl Into<String>,
    ) -> Result<Self, MemoryError> {
        let session_id = session_id.into();
        let path = root
            .as_ref()
            .join(format!("{}_{}.cbor", SESSION_SUFFIX, session_id));
        let records = PersistedStore::load(&path)?;
        let next_id = records.last().map(|record| record.id + 1).unwrap_or(1);
        Ok(Self {
            session_id,
            path,
            entries: RwLock::new(records),
            next_id: AtomicU64::new(next_id),
        })
    }

    fn persist(&self) -> Result<(), MemoryError> {
        let snapshot = self.entries.read().clone();
        PersistedStore::save(&self.path, &snapshot)
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

impl MemoryStore for SessionMemory {
    fn append(
        &self,
        agent: &str,
        role: MemoryRole,
        content: &str,
        metadata: HashMap<String, String>,
        tags: Vec<String>,
    ) -> Result<MemoryRecord, MemoryError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let record = MemoryRecord::new(
            id,
            agent.to_string(),
            role,
            content.to_string(),
            metadata,
            tags,
        );
        {
            let mut entries = self.entries.write();
            entries.push(record.clone());
        }
        self.persist()?;
        Ok(record)
    }

    fn incremental(
        &self,
        cursor: u64,
        limit: usize,
    ) -> Result<(Vec<MemoryRecord>, u64), MemoryError> {
        let entries = self.entries.read();
        let mut collected = Vec::new();
        let mut last_seen = cursor;
        for record in entries
            .iter()
            .filter(|record| record.id > cursor)
            .take(limit)
        {
            collected.push(record.clone());
            last_seen = record.id;
        }
        Ok((collected, last_seen))
    }
}

pub fn build_retrieval(
    long_term: &(impl MemoryStore + ?Sized),
    session: &(impl MemoryStore + ?Sized),
    cursor: MemoryCursor,
    limit: usize,
) -> Result<MemoryRetrieval, MemoryError> {
    let start = std::time::Instant::now();
    let (session_records, session_cursor) = session.incremental(cursor.session, limit)?;
    let remaining = limit.saturating_sub(session_records.len());
    let (long_term_records, long_cursor) = if remaining > 0 {
        long_term.incremental(cursor.long_term, remaining)?
    } else {
        (Vec::new(), cursor.long_term)
    };

    let mut records = session_records;
    records.extend(long_term_records);
    let total_bytes = records.iter().map(MemoryRecord::content_len).sum();
    let next_cursor = MemoryCursor {
        long_term: long_cursor,
        session: session_cursor,
    };

    Ok(MemoryRetrieval {
        records,
        next_cursor,
        total_bytes,
        took_ms: start.elapsed().as_millis(),
    })
}
