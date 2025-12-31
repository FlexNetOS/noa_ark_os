use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Duration, Utc};
use noa_core::fs as noa_fs;
use serde::{Deserialize, Serialize};
use tokio::fs as tokio_fs;
use tokio::sync::RwLock;
use tracing::warn;
use uuid::Uuid;

use crate::policy::{PolicyDocument, RetentionPolicy, RuntimePolicy, RuntimeRule};
use crate::registry::{FileEntry, FileRegistry};
use crate::state::{
    ActionOutcome, CompletedAction, DuplicateSet, PendingAction, RelocationAction, RelocationState,
    SkippedAction,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    DryRun,
    Enforce,
    Approval,
}

impl FromStr for ExecutionMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "dry" | "dry-run" | "dryrun" => Ok(Self::DryRun),
            "enforce" | "apply" | "run" => Ok(Self::Enforce),
            "approval" | "approve" | "queue" => Ok(Self::Approval),
            other => Err(anyhow!(format!("unknown execution mode: {}", other))),
        }
    }
}

impl std::fmt::Display for ExecutionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            ExecutionMode::DryRun => "dry_run",
            ExecutionMode::Enforce => "enforce",
            ExecutionMode::Approval => "approval",
        };
        write!(f, "{}", value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelocationReport {
    pub generated_at: DateTime<Utc>,
    pub mode: ExecutionMode,
    pub planned: Vec<RelocationAction>,
    pub applied: Vec<CompletedAction>,
    pub pending: Vec<PendingAction>,
    pub skipped: Vec<SkippedAction>,
    pub duplicates: Vec<DuplicateSet>,
}

struct PlannedActions {
    actions: Vec<RelocationAction>,
    skipped: Vec<SkippedAction>,
}

#[derive(Clone)]
pub struct RelocationDaemon {
    policy_path: PathBuf,
    registry_path: PathBuf,
    backups_dir: PathBuf,
    state_path: PathBuf,
    state: Arc<RwLock<RelocationState>>,
}

impl RelocationDaemon {
    pub async fn new(
        policy_path: impl Into<PathBuf>,
        registry_path: impl Into<PathBuf>,
        backups_dir: impl Into<PathBuf>,
    ) -> Result<Self> {
        let policy_path = policy_path.into();
        let registry_path = registry_path.into();
        let backups_dir = backups_dir.into();
        tokio_fs::create_dir_all(&backups_dir)
            .await
            .with_context(|| format!("unable to create backups directory at {:?}", backups_dir))?;

        let state_path = backups_dir.join("relocation_state.json");
        let state = if let Ok(bytes) = tokio_fs::read(&state_path).await {
            serde_json::from_slice(&bytes).with_context(|| "unable to parse relocation state")?
        } else {
            RelocationState::default()
        };

        noa_fs::init().map_err(|err| anyhow!(err))?;

        Ok(Self {
            policy_path,
            registry_path,
            backups_dir,
            state_path,
            state: Arc::new(RwLock::new(state)),
        })
    }

    pub async fn run(&self, mode: ExecutionMode) -> Result<RelocationReport> {
        let policy = self.load_policy().await?;
        let registry = self.load_registry().await?;
        self.hydrate_registry(&registry).await?;

        let now = Utc::now();
        let duplicates = registry
            .duplicates()
            .into_iter()
            .map(|(hash, entries)| DuplicateSet {
                hash,
                files: entries.iter().map(|entry| entry.path.clone()).collect(),
            })
            .collect::<Vec<_>>();

        let plan = self.plan_actions(&policy, &registry, &duplicates, now)?;

        let mut state = self.state.write().await;
        state.last_run = Some(now);
        state.duplicates = duplicates.clone();
        state.clear_transient();

        let mut applied = Vec::new();
        let mut skipped = plan.skipped.clone();

        match mode {
            ExecutionMode::DryRun => {
                state.dry_run = plan.actions.clone();
            }
            ExecutionMode::Approval => {
                let pending_additions: Vec<PendingAction> = plan
                    .actions
                    .iter()
                    .cloned()
                    .map(|action| PendingAction {
                        action,
                        enqueued_at: now,
                    })
                    .collect();
                state.pending.extend(pending_additions);
            }
            ExecutionMode::Enforce => {
                for action in plan.actions.clone() {
                    if action.requires_approval {
                        state.pending.push(PendingAction {
                            action,
                            enqueued_at: now,
                        });
                        continue;
                    }

                    match self.apply_action(&action, &registry, false).await? {
                        ActionOutcome::Applied => {
                            let completed = CompletedAction {
                                action: action.clone(),
                                completed_at: Utc::now(),
                                outcome: ActionOutcome::Applied,
                            };
                            state.completed.push(completed.clone());
                            applied.push(completed);
                        }
                        ActionOutcome::Skipped(reason) => {
                            let completed = CompletedAction {
                                action: action.clone(),
                                completed_at: Utc::now(),
                                outcome: ActionOutcome::Skipped(reason.clone()),
                            };
                            state.completed.push(completed.clone());
                            applied.push(completed);

                            let skipped_action = SkippedAction {
                                action: action.clone(),
                                reason,
                            };
                            skipped.push(skipped_action.clone());
                            state.skipped.push(skipped_action);
                        }
                    }
                }
            }
        }

        state.skipped.extend(plan.skipped.clone());

        drop(state);
        self.persist_state().await?;

        let report = RelocationReport {
            generated_at: now,
            mode,
            planned: plan.actions,
            applied,
            pending: self.state.read().await.pending.clone(),
            skipped,
            duplicates,
        };

        self.persist_checkpoint(&report).await?;

        Ok(report)
    }

    pub async fn status(&self) -> Result<RelocationState> {
        Ok(self.state.read().await.clone())
    }

    pub async fn approve_action(&self, action_id: Uuid) -> Result<Option<CompletedAction>> {
        let action = {
            let mut state = self.state.write().await;
            if let Some(pos) = state
                .pending
                .iter()
                .position(|pending| pending.action.id == action_id)
            {
                let pending = state.pending.remove(pos);
                pending.action
            } else {
                return Ok(None);
            }
        };

        let registry = self.load_registry().await?;
        self.hydrate_registry(&registry).await?;

        let outcome = self.apply_action(&action, &registry, false).await?;
        let completed = CompletedAction {
            action: action.clone(),
            completed_at: Utc::now(),
            outcome: outcome.clone(),
        };

        let mut state = self.state.write().await;
        if let ActionOutcome::Skipped(reason) = &completed.outcome {
            state.skipped.push(SkippedAction {
                action: action.clone(),
                reason: reason.clone(),
            });
        }
        state.completed.push(completed.clone());
        drop(state);
        self.persist_state().await?;

        Ok(Some(completed))
    }

    pub async fn manual_override(
        &self,
        source: String,
        destination: String,
        force: bool,
    ) -> Result<CompletedAction> {
        let registry = self.load_registry().await?;
        self.hydrate_registry(&registry).await?;

        if noa_fs::get_file(&source).is_none() {
            noa_fs::create_file(source.clone(), 0o644)
                .map_err(|err| anyhow!(format!("unable to stage source file: {}", err)))?;
        }

        if force {
            let _ = noa_fs::delete_file(&destination);
        }

        let mut action =
            RelocationAction::new("manual_override", source.clone(), destination.clone());
        action.notes.push("manual override".to_string());

        let outcome = match noa_fs::move_file(&source, destination.clone()) {
            Ok(_) => ActionOutcome::Applied,
            Err(err) => ActionOutcome::Skipped(err.to_string()),
        };

        let completed = CompletedAction {
            action: action.clone(),
            completed_at: Utc::now(),
            outcome: outcome.clone(),
        };

        let mut state = self.state.write().await;
        if let ActionOutcome::Skipped(reason) = &completed.outcome {
            state.skipped.push(SkippedAction {
                action: action.clone(),
                reason: reason.clone(),
            });
        }
        state.completed.push(completed.clone());
        drop(state);
        self.persist_state().await?;

        Ok(completed)
    }

    async fn load_policy(&self) -> Result<RuntimePolicy> {
        let bytes = tokio_fs::read(&self.policy_path)
            .await
            .with_context(|| format!("unable to read policy file at {:?}", self.policy_path))?;
        let doc: PolicyDocument = serde_yaml::from_slice(&bytes)
            .with_context(|| "unable to parse relocation policy document")?;
        doc.into_runtime()
    }

    async fn load_registry(&self) -> Result<FileRegistry> {
        FileRegistry::load(&self.registry_path).await
    }

    async fn hydrate_registry(&self, registry: &FileRegistry) -> Result<()> {
        for entry in registry.entries() {
            if noa_fs::get_file(entry.path()).is_none() {
                if let Err(err) = noa_fs::create_file(entry.path.clone(), entry.permissions()) {
                    warn!("unable to register file {}: {}", entry.path(), err);
                }
            }
        }
        Ok(())
    }

    fn plan_actions(
        &self,
        policy: &RuntimePolicy,
        registry: &FileRegistry,
        duplicates: &[DuplicateSet],
        now: DateTime<Utc>,
    ) -> Result<PlannedActions> {
        let mut actions = Vec::new();
        let mut skipped = Vec::new();
        let mut counters: HashMap<String, u64> = HashMap::new();
        let duplicate_map: HashMap<String, DuplicateSet> = duplicates
            .iter()
            .cloned()
            .map(|set| (set.hash.clone(), set))
            .collect();
        let defaults = policy.defaults();

        'entries: for entry in registry.entries() {
            for rule in policy.rules() {
                if !rule.matches(entry) {
                    continue;
                }

                let file_name = match rule.file_name(entry) {
                    Some(name) => name,
                    None => {
                        skipped.push(SkippedAction {
                            action: RelocationAction::new(&rule.name, entry.path(), entry.path()),
                            reason: "missing file name".to_string(),
                        });
                        continue 'entries;
                    }
                };

                let naming = rule.naming(&defaults.naming);
                let count = if naming.uses_counter() {
                    let entry = counters.entry(rule.name.clone()).or_insert(0);
                    *entry += 1;
                    *entry
                } else {
                    0
                };

                let generated_name = naming.format_name(&file_name, count, now);
                let destination = Self::join_path(&rule.destination, &generated_name);
                let mut action =
                    RelocationAction::new(&rule.name, entry.path(), destination.clone());
                action.requires_approval = rule.requires_approval;

                if let Some(retention_reason) =
                    Self::evaluate_retention(rule, &defaults.retention, entry, registry)
                {
                    skipped.push(SkippedAction {
                        action,
                        reason: retention_reason,
                    });
                    continue 'entries;
                }

                if let Some(set) = duplicate_map.get(&entry.hash) {
                    if set.files.len() > 1 {
                        action.requires_approval = true;
                        action.duplicate_of = set.files.first().cloned();
                        action
                            .notes
                            .push("duplicate detected - approval required".to_string());
                    }
                }

                actions.push(action);
                continue 'entries;
            }
        }

        Ok(PlannedActions { actions, skipped })
    }

    fn join_path(base: &str, leaf: &str) -> String {
        let mut trimmed = base.trim_end_matches('/').to_string();
        if trimmed.is_empty() {
            trimmed.push('/');
        }
        if !trimmed.ends_with('/') {
            trimmed.push('/');
        }
        format!("{}{}", trimmed, leaf.trim_start_matches('/'))
    }

    fn evaluate_retention(
        rule: &RuntimeRule,
        defaults: &RetentionPolicy,
        entry: &FileEntry,
        registry: &FileRegistry,
    ) -> Option<String> {
        let retention = rule.retention(defaults);

        if let Some(ttl_days) = retention.ttl_days {
            if ttl_days > 0 {
                if let Some(last_seen) = entry.last_seen() {
                    let age = Utc::now().signed_duration_since(last_seen);
                    if age < Duration::days(ttl_days) {
                        return Some(format!("file younger than {} days", ttl_days));
                    }
                }
            }
        }

        if let Some(max_versions) = retention.max_versions {
            let count = registry
                .entries()
                .iter()
                .filter(|candidate| candidate.path.starts_with(&rule.destination))
                .count();
            if count >= max_versions {
                return Some(format!("max versions {} reached", max_versions));
            }
        }

        None
    }

    async fn apply_action(
        &self,
        action: &RelocationAction,
        registry: &FileRegistry,
        force: bool,
    ) -> Result<ActionOutcome> {
        self.hydrate_registry(registry).await?;

        if force {
            let _ = noa_fs::delete_file(&action.destination);
        }

        match noa_fs::move_file(&action.source, action.destination.clone()) {
            Ok(_) => Ok(ActionOutcome::Applied),
            Err(err) => Ok(ActionOutcome::Skipped(err.to_string())),
        }
    }

    async fn persist_state(&self) -> Result<()> {
        let state = self.state.read().await;
        let bytes = serde_json::to_vec_pretty(&*state)?;
        tokio_fs::write(&self.state_path, bytes)
            .await
            .with_context(|| "unable to write relocation state")
    }

    async fn persist_checkpoint(&self, report: &RelocationReport) -> Result<()> {
        let filename = format!(
            "relocation-{}.json",
            report.generated_at.format("%Y%m%dT%H%M%S%.3fZ")
        );
        let path = self.backups_dir.join(filename);
        let bytes = serde_json::to_vec_pretty(report)?;
        tokio_fs::write(path, bytes)
            .await
            .with_context(|| "unable to persist relocation checkpoint")
    }
}
