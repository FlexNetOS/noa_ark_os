//! Trust scorekeeper derived from the north_star.deflex.json policy.

mod config;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

pub use config::{EscalationPolicy, MetricDefinition, NorthStarPolicy, ScopeReduction, Thresholds};
use serde::{Deserialize, Serialize};

const DEFAULT_STORAGE_PATH: &str = "metrics/trust_score.json";
const POLICY_BYTES: &str = include_str!("../../kernel/north_star.deflex.json");

fn global_policy() -> &'static NorthStarPolicy {
    static POLICY: OnceLock<NorthStarPolicy> = OnceLock::new();
    POLICY
        .get_or_init(|| serde_json::from_str(POLICY_BYTES).expect("invalid north_star.deflex.json"))
}

/// Errors emitted while computing or persisting trust scores.
#[derive(Debug, thiserror::Error)]
pub enum ScorekeeperError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse trust snapshot: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("metric '{0}' missing from policy")]
    MissingMetric(String),
}

/// Input counters used to derive trust scores.
#[derive(Debug, Clone, Copy, Default)]
pub struct ScoreInputs {
    pub integrity_pass: u64,
    pub integrity_fail: u64,
    pub reversibility_pass: u64,
    pub reversibility_fail: u64,
    pub capability_pass: u64,
    pub capability_fail: u64,
}

impl ScoreInputs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn integrity(mut self, pass: u64, fail: u64) -> Self {
        self.integrity_pass = pass;
        self.integrity_fail = fail;
        self
    }

    pub fn reversibility(mut self, pass: u64, fail: u64) -> Self {
        self.reversibility_pass = pass;
        self.reversibility_fail = fail;
        self
    }

    pub fn capability(mut self, pass: u64, fail: u64) -> Self {
        self.capability_pass = pass;
        self.capability_fail = fail;
        self
    }
}

/// Snapshot describing the current trust posture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustSnapshot {
    pub generated_at: u64,
    pub policy_version: String,
    pub composite_score: f32,
    pub metrics: HashMap<String, MetricScore>,
    pub triggered_escalations: Vec<TriggeredEscalation>,
    pub scope_directive: ScopeDirective,
}

impl TrustSnapshot {
    fn baseline(policy: &NorthStarPolicy) -> Self {
        let mut metrics = HashMap::new();
        for metric in &policy.metrics {
            metrics.insert(
                metric.id.clone(),
                MetricScore {
                    score: 1.0,
                    status: MetricStatus::Nominal,
                    description: metric.description.clone(),
                    thresholds: metric.thresholds.clone(),
                    weight: metric.weight,
                },
            );
        }

        Self {
            generated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            policy_version: policy.version.clone(),
            composite_score: 1.0,
            metrics,
            triggered_escalations: Vec::new(),
            scope_directive: ScopeDirective {
                optional_multiplier: 1.0,
                minimum_optional: 0,
                status: MetricStatus::Nominal,
            },
        }
    }
}

/// Per-metric trust score enriched with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricScore {
    pub score: f32,
    pub status: MetricStatus,
    pub description: String,
    pub thresholds: Thresholds,
    pub weight: f32,
}

/// Severity status derived from metric thresholds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MetricStatus {
    Nominal,
    Warning,
    Critical,
}

/// Triggered escalation referencing the policy definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggeredEscalation {
    pub policy_id: String,
    pub summary: String,
    pub severity: MetricStatus,
}

/// Directive consumed by orchestrators to shrink capability scopes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopeDirective {
    pub optional_multiplier: f32,
    pub minimum_optional: usize,
    pub status: MetricStatus,
}

impl ScopeDirective {
    pub fn allowed_optional(&self, total_optional: usize) -> usize {
        if total_optional == 0 {
            return 0;
        }
        let allowed = (total_optional as f32 * self.optional_multiplier).ceil() as usize;
        allowed.max(self.minimum_optional.min(total_optional))
    }
}

/// Scorekeeper responsible for deriving trust posture and persisting snapshots.
#[derive(Debug)]
pub struct Scorekeeper {
    policy: &'static NorthStarPolicy,
    storage_path: PathBuf,
    cache: RwLock<Option<TrustSnapshot>>,
}

impl Scorekeeper {
    /// Create a scorekeeper using the default storage path or overridden by NOA_TRUST_METRICS_PATH.
    pub fn default() -> Result<Self, ScorekeeperError> {
        let storage_path = std::env::var("NOA_TRUST_METRICS_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_STORAGE_PATH));
        Self::with_storage(storage_path)
    }

    /// Create a scorekeeper persisting snapshots to the provided path.
    pub fn with_storage(storage_path: PathBuf) -> Result<Self, ScorekeeperError> {
        Ok(Self {
            policy: global_policy(),
            storage_path,
            cache: RwLock::new(None),
        })
    }

    /// Load the most recent snapshot from disk or create a baseline if none exists.
    pub fn bootstrap(&self) -> Result<TrustSnapshot, ScorekeeperError> {
        match self.load_snapshot()? {
            Some(snapshot) => Ok(snapshot),
            None => {
                let snapshot = TrustSnapshot::baseline(self.policy);
                self.persist(&snapshot)?;
                Ok(snapshot)
            }
        }
    }

    /// Evaluate trust scores from the provided inputs without persisting.
    pub fn evaluate(&self, inputs: ScoreInputs) -> Result<TrustSnapshot, ScorekeeperError> {
        let mut metrics = HashMap::new();
        let mut composite = 0.0;
        let mut triggered = Vec::new();

        let integrity = self
            .policy
            .metric("integrity")
            .ok_or_else(|| ScorekeeperError::MissingMetric("integrity".into()))?;
        let reversibility = self
            .policy
            .metric("reversibility")
            .ok_or_else(|| ScorekeeperError::MissingMetric("reversibility".into()))?;
        let capability = self
            .policy
            .metric("capability")
            .ok_or_else(|| ScorekeeperError::MissingMetric("capability".into()))?;

        let integrity_score = Self::score_ratio(inputs.integrity_pass, inputs.integrity_fail);
        let integrity_status = Self::status_for(&integrity.thresholds, integrity_score);
        composite += integrity.weight * integrity_score;
        metrics.insert(
            integrity.id.clone(),
            MetricScore {
                score: integrity_score,
                status: integrity_status.clone(),
                description: integrity.description.clone(),
                thresholds: integrity.thresholds.clone(),
                weight: integrity.weight,
            },
        );
        triggered.extend(self.map_escalation(integrity, &integrity_status));

        let reversibility_score =
            Self::score_ratio(inputs.reversibility_pass, inputs.reversibility_fail);
        let reversibility_status = Self::status_for(&reversibility.thresholds, reversibility_score);
        composite += reversibility.weight * reversibility_score;
        metrics.insert(
            reversibility.id.clone(),
            MetricScore {
                score: reversibility_score,
                status: reversibility_status.clone(),
                description: reversibility.description.clone(),
                thresholds: reversibility.thresholds.clone(),
                weight: reversibility.weight,
            },
        );
        triggered.extend(self.map_escalation(reversibility, &reversibility_status));

        let capability_score = Self::score_ratio(inputs.capability_pass, inputs.capability_fail);
        let capability_status = Self::status_for(&capability.thresholds, capability_score);
        composite += capability.weight * capability_score;
        metrics.insert(
            capability.id.clone(),
            MetricScore {
                score: capability_score,
                status: capability_status.clone(),
                description: capability.description.clone(),
                thresholds: capability.thresholds.clone(),
                weight: capability.weight,
            },
        );
        triggered.extend(self.map_escalation(capability, &capability_status));

        let directive = ScopeDirective {
            optional_multiplier: match capability_status {
                MetricStatus::Nominal => 1.0,
                MetricStatus::Warning => capability.scope_reduction().warning_multiplier,
                MetricStatus::Critical => capability.scope_reduction().critical_multiplier,
            },
            minimum_optional: capability.scope_reduction().minimum_optional,
            status: capability_status,
        };

        let snapshot = TrustSnapshot {
            generated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            policy_version: self.policy.version.clone(),
            composite_score: composite,
            metrics,
            triggered_escalations: triggered,
            scope_directive: directive,
        };

        Ok(snapshot)
    }

    /// Evaluate trust scores and persist them to disk.
    pub fn record(&self, inputs: ScoreInputs) -> Result<TrustSnapshot, ScorekeeperError> {
        let snapshot = self.evaluate(inputs)?;
        self.persist(&snapshot)?;
        Ok(snapshot)
    }

    /// Persist a snapshot to disk and update the in-memory cache.
    pub fn persist(&self, snapshot: &TrustSnapshot) -> Result<(), ScorekeeperError> {
        if let Some(parent) = self.storage_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_string_pretty(snapshot)?;
        fs::write(&self.storage_path, json)?;
        *self.cache.write().unwrap() = Some(snapshot.clone());
        Ok(())
    }

    /// Load the cached snapshot from disk if present.
    pub fn load_snapshot(&self) -> Result<Option<TrustSnapshot>, ScorekeeperError> {
        if let Some(snapshot) = self.cache.read().unwrap().clone() {
            return Ok(Some(snapshot));
        }
        if !self.storage_path.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&self.storage_path)?;
        let snapshot: TrustSnapshot = serde_json::from_str(&data)?;
        *self.cache.write().unwrap() = Some(snapshot.clone());
        Ok(Some(snapshot))
    }

    /// Return the most recently observed snapshot, computing a baseline if necessary.
    pub fn latest(&self) -> TrustSnapshot {
        self.load_snapshot()
            .ok()
            .flatten()
            .unwrap_or_else(|| TrustSnapshot::baseline(self.policy))
    }

    fn score_ratio(passes: u64, failures: u64) -> f32 {
        let total = passes + failures;
        if total == 0 {
            return 1.0;
        }
        passes as f32 / total as f32
    }

    fn status_for(thresholds: &Thresholds, score: f32) -> MetricStatus {
        if score <= thresholds.critical {
            MetricStatus::Critical
        } else if score <= thresholds.warning {
            MetricStatus::Warning
        } else {
            MetricStatus::Nominal
        }
    }

    fn map_escalation(
        &self,
        metric: &MetricDefinition,
        status: &MetricStatus,
    ) -> Option<TriggeredEscalation> {
        if matches!(status, MetricStatus::Nominal) {
            return None;
        }
        metric
            .escalation_policy
            .as_ref()
            .and_then(|id| self.policy.escalation(id))
            .map(|policy| TriggeredEscalation {
                policy_id: policy.id.clone(),
                summary: policy.summary.clone(),
                severity: status.clone(),
            })
    }

    /// Convenience helper returning the current scope directive from disk.
    pub fn current_scope_directive() -> ScopeDirective {
        let keeper = Self::default().expect("failed to construct scorekeeper");
        keeper
            .load_snapshot()
            .ok()
            .flatten()
            .unwrap_or_else(|| TrustSnapshot::baseline(keeper.policy))
            .scope_directive
    }

    /// Provide the storage path used by the scorekeeper.
    pub fn storage_path(&self) -> &Path {
        &self.storage_path
    }

    /// Obtain a shared handle for HTTP handlers.
    pub fn into_shared(self) -> Arc<Self> {
        Arc::new(self)
    }
}

/// HTTP helpers exposing trust posture under /v1/trust.
pub mod api {
    use super::*;
    use axum::extract::State;
    use axum::routing::get;
    use axum::{Json, Router};

    #[derive(Clone)]
    pub struct TrustState {
        pub scorekeeper: Arc<Scorekeeper>,
    }

    /// Build an Axum router exposing the trust snapshot endpoint.
    pub fn router(scorekeeper: Arc<Scorekeeper>) -> Router {
        Router::new()
            .route("/v1/trust", get(get_trust))
            .with_state(TrustState { scorekeeper })
    }

    async fn get_trust(State(state): State<TrustState>) -> Json<TrustSnapshot> {
        Json(state.scorekeeper.latest())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn baseline_snapshot_has_all_metrics() {
        let policy = global_policy();
        let snapshot = TrustSnapshot::baseline(policy);
        assert_eq!(snapshot.metrics.len(), policy.metrics.len());
        assert!(snapshot.triggered_escalations.is_empty());
        assert_eq!(snapshot.composite_score, 1.0);
    }

    #[test]
    fn evaluate_records_escalations_and_scope() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("trust.json");
        let keeper = Scorekeeper::with_storage(path.clone()).unwrap();
        let inputs = ScoreInputs::default()
            .integrity(80, 20)
            .reversibility(70, 30)
            .capability(40, 60);
        let snapshot = keeper.evaluate(inputs).unwrap();
        assert!(snapshot.composite_score < 1.0);
        assert!(snapshot.metrics["capability"].score < 0.5);
        assert!(matches!(
            snapshot.scope_directive.status,
            MetricStatus::Critical | MetricStatus::Warning
        ));
        if !snapshot.triggered_escalations.is_empty() {
            assert!(snapshot
                .triggered_escalations
                .iter()
                .all(|e| !e.policy_id.is_empty()));
        }
        keeper.persist(&snapshot).unwrap();
        let persisted = keeper.load_snapshot().unwrap().unwrap();
        assert_eq!(persisted.metrics.len(), snapshot.metrics.len());
    }

    #[test]
    fn scope_directive_limits_optional_capabilities() {
        let directive = ScopeDirective {
            optional_multiplier: 0.5,
            minimum_optional: 1,
            status: MetricStatus::Warning,
        };
        assert_eq!(directive.allowed_optional(0), 0);
        assert_eq!(directive.allowed_optional(1), 1);
        assert_eq!(directive.allowed_optional(4), 2);
    }
}
