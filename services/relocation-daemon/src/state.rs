use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelocationAction {
    pub id: Uuid,
    pub rule: String,
    pub source: String,
    pub destination: String,
    pub requires_approval: bool,
    pub duplicate_of: Option<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

impl RelocationAction {
    pub fn new(
        rule: impl Into<String>,
        source: impl Into<String>,
        destination: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            rule: rule.into(),
            source: source.into(),
            destination: destination.into(),
            requires_approval: false,
            duplicate_of: None,
            notes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAction {
    pub action: RelocationAction,
    pub enqueued_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedAction {
    pub action: RelocationAction,
    pub completed_at: DateTime<Utc>,
    pub outcome: ActionOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkippedAction {
    pub action: RelocationAction,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateSet {
    pub hash: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionOutcome {
    Applied,
    Skipped(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RelocationState {
    pub last_run: Option<DateTime<Utc>>,
    #[serde(default)]
    pub pending: Vec<PendingAction>,
    #[serde(default)]
    pub completed: Vec<CompletedAction>,
    #[serde(default)]
    pub dry_run: Vec<RelocationAction>,
    #[serde(default)]
    pub skipped: Vec<SkippedAction>,
    #[serde(default)]
    pub duplicates: Vec<DuplicateSet>,
}

impl RelocationState {
    pub fn clear_transient(&mut self) {
        self.dry_run.clear();
        self.skipped.clear();
    }
}
