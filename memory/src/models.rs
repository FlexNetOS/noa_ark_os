use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum MemoryRole {
    #[default]
    Observation,
    Action,
    Reflection,
    Summary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct MemoryCursor {
    pub long_term: u64,
    pub session: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRecord {
    pub id: u64,
    pub agent: String,
    pub role: MemoryRole,
    pub timestamp: DateTime<Utc>,
    pub content: String,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl MemoryRecord {
    pub fn new(
        id: u64,
        agent: impl Into<String>,
        role: MemoryRole,
        content: impl Into<String>,
        metadata: HashMap<String, String>,
        tags: Vec<String>,
    ) -> Self {
        Self {
            id,
            agent: agent.into(),
            role,
            timestamp: Utc::now(),
            content: content.into(),
            metadata,
            tags,
        }
    }

    pub fn content_len(&self) -> usize {
        self.content.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRetrieval {
    pub records: Vec<MemoryRecord>,
    pub next_cursor: MemoryCursor,
    pub total_bytes: usize,
    pub took_ms: u128,
}

impl MemoryRetrieval {
    pub fn empty(cursor: MemoryCursor) -> Self {
        Self {
            records: Vec::new(),
            next_cursor: cursor,
            total_bytes: 0,
            took_ms: 0,
        }
    }
}
