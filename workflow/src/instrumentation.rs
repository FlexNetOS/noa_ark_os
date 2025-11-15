use crate::{Stage, StageType, Task, TaskDispatchReceipt};
use crate::reward::{
    AgentApprovalStatus, AgentStandingSummary, RewardAgentSnapshot, RewardInputs, RewardScorekeeper,
};
use crate::reward::RewardError;
use chrono::Utc;
use noa_core::security::{self, OperationKind, OperationRecord, SignedOperation};
use noa_core::utils::{current_timestamp_millis, simple_hash};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, ErrorKind, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const INDEX_DIR: &str = ".workspace/indexes";
const STORAGE_MIRROR_DIR: &str = "storage/db";
const RELOCATION_LOG: &str = "relocation";
const DOCUMENT_LOG: &str = "documentation";
const STAGE_RECEIPT_LOG: &str = "stage_receipts";
const SECURITY_SCAN_LOG: &str = "security_scans";
const TASK_DISPATCH_LOG: &str = "task_dispatches";
const AUTO_FIX_LOG: &str = "auto_fix_actions";
const BUDGET_DECISION_LOG: &str = "budget_guardian";
const AUTO_FIX_DIR: &str = "auto_fix";
const BUDGET_GUARDIAN_DIR: &str = "budget_guardian";
const INFERENCE_LOG: &str = "inference_metrics";
const PIPELINE_EVENT_LOG: &str = "pipeline_events";
const EVIDENCE_LEDGER_DIR: &str = "storage/db/evidence";
const EVIDENCE_LEDGER_FILE: &str = "ledger.jsonl";
const GOAL_ANALYTICS_DIR: &str = "storage/db/analytics";
const GOAL_ANALYTICS_FILE: &str = "goal_kpis.json";
const METRICS_DIR: &str = "metrics";
const REWARD_HISTORY_FILE: &str = "reward_history.json";
const DEPLOYMENT_REPORT_PATH: &str = "docs/reports/AGENT_DEPLOYMENT_OUTCOMES.md";
const LOG_CHANNELS: [&str; 9] = [
    RELOCATION_LOG,
    DOCUMENT_LOG,
    STAGE_RECEIPT_LOG,
    TASK_DISPATCH_LOG,
    AUTO_FIX_LOG,
    BUDGET_DECISION_LOG,
    SECURITY_SCAN_LOG,
    INFERENCE_LOG,
    PIPELINE_EVENT_LOG,
];

#[derive(Debug)]
pub enum InstrumentationError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
    Security(security::PolicyError),
    Reward(RewardError),
}

impl std::fmt::Display for InstrumentationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstrumentationError::Io(err) => write!(f, "io error: {}", err),
            InstrumentationError::Serialization(err) => write!(f, "serialization error: {}", err),
            InstrumentationError::Security(err) => write!(f, "policy error: {}", err),
            InstrumentationError::Reward(err) => write!(f, "reward error: {}", err),
        }
    }
}

impl std::error::Error for InstrumentationError {}

impl From<std::io::Error> for InstrumentationError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for InstrumentationError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err)
    }
}

impl From<security::PolicyError> for InstrumentationError {
    fn from(err: security::PolicyError) -> Self {
        Self::Security(err)
    }
}

impl From<RewardError> for InstrumentationError {
    fn from(err: RewardError) -> Self {
        Self::Reward(err)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PipelineLogEvent {
    event_type: String,
    actor: String,
    scope: String,
    source: Option<String>,
    target: Option<String>,
    metadata: Value,
    timestamp: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ImmutableLogEntry {
    event: PipelineLogEvent,
    policy: SignedOperation,
    previous_hash: String,
    entry_hash: String,
}

impl ImmutableLogEntry {
    fn new(
        event: PipelineLogEvent,
        policy: SignedOperation,
        previous_hash: String,
    ) -> Result<Self, InstrumentationError> {
        let materialised = json!({
            "event": &event,
            "policy": &policy,
            "previous_hash": &previous_hash,
        });
        let entry_hash = simple_hash(&serde_json::to_string(&materialised)?);
        Ok(Self {
            event,
            policy,
            previous_hash,
            entry_hash,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecisionRecord {
    pub decision: String,
    pub reason: String,
    #[serde(default)]
    pub signals: Vec<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoFixActionReceipt {
    pub fixer: String,
    pub target: String,
    pub snapshot_path: PathBuf,
    pub plan: Value,
    pub policy: PolicyDecisionRecord,
    pub signed_operation: SignedOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetDecisionRecord {
    pub workflow_id: String,
    pub stage_id: String,
    pub tokens_used: f64,
    pub token_limit: f64,
    pub latency_ms: f64,
    pub latency_limit: f64,
    pub action: String,
    #[serde(default)]
    pub rewritten_plan: Option<Value>,
    pub snapshot_path: PathBuf,
    pub signed_operation: SignedOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleLeaf {
    pub index: usize,
    pub hash: String,
    pub task_hash: String,
    pub artifact_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleLevel {
    pub level: usize,
    pub nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskReceipt {
    pub task_index: usize,
    pub task: Task,
    pub task_hash: String,
    pub artifact_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageReceipt {
    pub workflow_id: String,
    pub stage_id: String,
    pub stage_type: StageType,
    pub generated_at: u128,
    pub merkle_root: String,
    pub levels: Vec<MerkleLevel>,
    pub leaves: Vec<MerkleLeaf>,
    pub tasks: Vec<TaskReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionResult {
    pub agent: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalOutcomeRecord {
    pub goal_id: String,
    pub workflow_id: String,
    pub started_at: u128,
    pub completed_at: u128,
    pub duration_ms: u128,
    pub success: bool,
    #[serde(default)]
    pub agents: Vec<AgentExecutionResult>,
    #[serde(default)]
    pub reward_inputs: Option<RewardInputs>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentOutcomeRecord {
    pub workflow_id: String,
    pub stage_id: String,
    pub agent_role: String,
    pub agent_id: String,
    pub action: String,
    pub status: String,
    pub notes: Value,
    pub recorded_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct AgentAggregate {
    total_runs: u64,
    successful_runs: u64,
}

impl AgentAggregate {
    fn record(&mut self, success: bool) {
        self.total_runs += 1;
        if success {
            self.successful_runs += 1;
        }
    }

    fn to_metric(&self, agent: &str) -> GoalAgentMetric {
        GoalAgentMetric {
            agent: agent.to_string(),
            total_runs: self.total_runs,
            successful_runs: self.successful_runs,
            success_rate: if self.total_runs == 0 {
                0.0
            } else {
                self.successful_runs as f64 / self.total_runs as f64
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GoalAggregate {
    goal_id: String,
    workflow_id: String,
    total_runs: u64,
    successful_runs: u64,
    total_duration_ms: u128,
    last_started_at: Option<u128>,
    last_completed_at: Option<u128>,
    agents: HashMap<String, AgentAggregate>,
}

impl GoalAggregate {
    fn new(goal_id: &str, workflow_id: &str) -> Self {
        Self {
            goal_id: goal_id.to_string(),
            workflow_id: workflow_id.to_string(),
            total_runs: 0,
            successful_runs: 0,
            total_duration_ms: 0,
            last_started_at: None,
            last_completed_at: None,
            agents: HashMap::new(),
        }
    }

    fn record(&mut self, outcome: &GoalOutcomeRecord) {
        self.total_runs += 1;
        if outcome.success {
            self.successful_runs += 1;
        }
        self.total_duration_ms += outcome.duration_ms;
        self.last_started_at = Some(outcome.started_at);
        self.last_completed_at = Some(outcome.completed_at);

        for agent in &outcome.agents {
            self.agents
                .entry(agent.agent.clone())
                .or_default()
                .record(agent.success);
        }
    }

    fn to_snapshot(&self, penalty: Option<ContextPenaltySummary>) -> GoalMetricSnapshot {
        let average_lead_time_ms = if self.total_runs == 0 {
            0.0
        } else {
            self.total_duration_ms as f64 / self.total_runs as f64
        };
        let success_rate = if self.total_runs == 0 {
            0.0
        } else {
            self.successful_runs as f64 / self.total_runs as f64
        };
        let mut agents: Vec<GoalAgentMetric> = self
            .agents
            .iter()
            .map(|(agent, aggregate)| aggregate.to_metric(agent))
            .collect();
        agents.sort_by(|a, b| {
            b.success_rate
                .partial_cmp(&a.success_rate)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut snapshot = GoalMetricSnapshot {
            goal_id: self.goal_id.clone(),
            workflow_id: self.workflow_id.clone(),
            total_runs: self.total_runs,
            successful_runs: self.successful_runs,
            average_lead_time_ms,
            success_rate,
            agents,
            updated_at: Utc::now().to_rfc3339(),
            context_penalty_score: 0.0,
            context_p95_bytes: 0,
            context_p95_latency_ms: 0,
        };

        if let Some(penalty) = penalty {
            snapshot.context_penalty_score = penalty.penalty_score;
            snapshot.context_p95_bytes = penalty.p95_bytes;
            snapshot.context_p95_latency_ms = penalty.p95_latency_ms;
        }

        snapshot
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContextUsageSample {
    agent: String,
    context_bytes: usize,
    penalty: f64,
    retrieval_ms: u128,
    timestamp: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ContextPenaltyAggregate {
    workflow_id: String,
    samples: Vec<ContextUsageSample>,
}

impl ContextPenaltyAggregate {
    fn new(workflow_id: &str) -> Self {
        Self {
            workflow_id: workflow_id.to_string(),
            samples: Vec::new(),
        }
    }

    fn record(&mut self, agent: &str, context_bytes: usize, threshold: usize, retrieval_ms: u128) {
        let penalty = if context_bytes > threshold {
            (context_bytes.saturating_sub(threshold)) as f64 / threshold as f64
        } else {
            0.0
        };

        self.samples.push(ContextUsageSample {
            agent: agent.to_string(),
            context_bytes,
            penalty,
            retrieval_ms,
            timestamp: current_timestamp_millis(),
        });

        self.trim();
    }

    fn push_summary(&mut self, penalty_score: f64, context_bytes: usize, retrieval_ms: u64) {
        self.samples.push(ContextUsageSample {
            agent: "scorekeeper/restore".into(),
            context_bytes,
            penalty: penalty_score,
            retrieval_ms: retrieval_ms as u128,
            timestamp: current_timestamp_millis(),
        });
        self.trim();
    }

    fn trim(&mut self) {
        if self.samples.len() > 256 {
            let overflow = self.samples.len() - 256;
            self.samples.drain(0..overflow);
        }
    }

    fn summary(&self) -> ContextPenaltySummary {
        if self.samples.is_empty() {
            return ContextPenaltySummary {
                workflow_id: self.workflow_id.clone(),
                penalty_score: 0.0,
                p95_bytes: 0,
                p95_latency_ms: 0,
            };
        }

        let mut bytes: Vec<usize> = self
            .samples
            .iter()
            .map(|sample| sample.context_bytes)
            .collect();
        bytes.sort_unstable();
        let mut latency: Vec<u128> = self
            .samples
            .iter()
            .map(|sample| sample.retrieval_ms)
            .collect();
        latency.sort_unstable();

        // Use p95 as a standard metric for tail latency/usage. Change PERCENTILE to adjust.
        const PERCENTILE: f64 = 0.95;
        let percentile_index = |len: usize| -> usize {
            if len == 0 {
                return 0;
            }
            let raw = ((len as f64) * PERCENTILE).ceil() as usize;
            raw.saturating_sub(1).min(len - 1)
        };

        let idx_bytes = percentile_index(bytes.len());
        let idx_latency = percentile_index(latency.len());
        let avg_penalty = self
            .samples
            .iter()
            .map(|sample| sample.penalty)
            .sum::<f64>()
            / (self.samples.len() as f64);

        ContextPenaltySummary {
            workflow_id: self.workflow_id.clone(),
            penalty_score: avg_penalty,
            p95_bytes: bytes[idx_bytes],
            p95_latency_ms: latency[idx_latency] as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContextPenaltySummary {
    workflow_id: String,
    penalty_score: f64,
    p95_bytes: usize,
    p95_latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct GoalMetricStore {
    goals: HashMap<String, GoalAggregate>,
    #[serde(default)]
    context: HashMap<String, ContextPenaltyAggregate>,
}

impl GoalMetricStore {
    fn record(&mut self, outcome: &GoalOutcomeRecord) {
        self.goals
            .entry(outcome.goal_id.clone())
            .or_insert_with(|| GoalAggregate::new(&outcome.goal_id, &outcome.workflow_id))
            .record(outcome);
    }

    fn penalize_context(
        &mut self,
        workflow_id: &str,
        agent: &str,
        context_bytes: usize,
        threshold: usize,
        retrieval_ms: u128,
    ) {
        self.context
            .entry(workflow_id.to_string())
            .or_insert_with(|| ContextPenaltyAggregate::new(workflow_id))
            .record(agent, context_bytes, threshold, retrieval_ms);
    }

    fn snapshots(&self) -> Vec<GoalMetricSnapshot> {
        let mut entries: Vec<GoalMetricSnapshot> = self
            .goals
            .values()
            .map(|aggregate| {
                let penalty = self
                    .context
                    .get(&aggregate.workflow_id)
                    .map(ContextPenaltyAggregate::summary);
                aggregate.to_snapshot(penalty)
            })
            .collect();
        entries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        entries
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalAgentMetric {
    pub agent: String,
    pub total_runs: u64,
    pub successful_runs: u64,
    pub success_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalMetricSnapshot {
    pub goal_id: String,
    pub workflow_id: String,
    pub total_runs: u64,
    pub successful_runs: u64,
    pub average_lead_time_ms: f64,
    pub success_rate: f64,
    pub agents: Vec<GoalAgentMetric>,
    pub updated_at: String,
    #[serde(default)]
    pub context_penalty_score: f64,
    #[serde(default)]
    pub context_p95_bytes: usize,
    #[serde(default)]
    pub context_p95_latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceMetric {
    pub provider: String,
    pub model: String,
    pub status: String,
    pub latency_ms: u128,
    pub tokens_prompt: usize,
    pub tokens_completion: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl StageReceipt {
    pub fn new(
        workflow_id: &str,
        stage: &Stage,
        artifacts: &[Value],
    ) -> Result<Self, InstrumentationError> {
        let generated_at = current_timestamp_millis();
        let mut leaves = Vec::new();
        let mut tasks = Vec::new();

        for (index, task) in stage.tasks.iter().enumerate() {
            let artifact = artifacts.get(index).cloned().unwrap_or(Value::Null);
            let artifact_repr = serde_json::to_string(&artifact)?;
            let artifact_hash = simple_hash(&artifact_repr);
            let task_repr = serde_json::to_string(task)?;
            let task_hash = simple_hash(&task_repr);
            let leaf_hash = simple_hash(&format!("{}::{}", task_hash, artifact_hash));
            leaves.push(MerkleLeaf {
                index,
                hash: leaf_hash,
                task_hash: task_hash.clone(),
                artifact_hash: artifact_hash.clone(),
            });
            tasks.push(TaskReceipt {
                task_index: index,
                task: task.clone(),
                task_hash,
                artifact_hash,
            });
        }

        let (levels, merkle_root) = build_merkle_tree(workflow_id, &stage.name, &leaves);

        Ok(Self {
            workflow_id: workflow_id.to_string(),
            stage_id: stage.name.clone(),
            stage_type: stage.stage_type.clone(),
            generated_at,
            merkle_root,
            levels,
            leaves,
            tasks,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SecurityScanStatus {
    Skipped,
    Passed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanReport {
    pub subject: String,
    pub tool: String,
    pub status: SecurityScanStatus,
    pub issues: Vec<String>,
    pub report_artifact: Option<String>,
    pub signed_operation: SignedOperation,
    pub ledger_reference: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceLedgerKind {
    Genesis,
    StageReceipt,
    SecurityScan,
    TaskDispatch,
    AutoFixAction,
    BudgetDecision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceLedgerEntry {
    pub kind: EvidenceLedgerKind,
    pub timestamp: u128,
    pub reference: String,
    pub payload: Value,
    pub signed_operation: SignedOperation,
}

impl EvidenceLedgerEntry {
    fn stage_receipt(receipt: &StageReceipt, signed: SignedOperation) -> Self {
        Self {
            kind: EvidenceLedgerKind::StageReceipt,
            timestamp: receipt.generated_at,
            reference: receipt.merkle_root.clone(),
            payload: json!({
                "workflow_id": receipt.workflow_id,
                "stage_id": receipt.stage_id,
                "stage_type": receipt.stage_type,
                "levels": receipt.levels,
                "leaves": receipt.leaves,
            }),
            signed_operation: signed,
        }
    }

    fn security_scan(subject: &str, report: &SecurityScanReport) -> Self {
        Self {
            kind: EvidenceLedgerKind::SecurityScan,
            timestamp: current_timestamp_millis(),
            reference: report.ledger_reference.clone(),
            payload: json!({
                "subject": subject,
                "tool": report.tool,
                "status": report.status,
                "issues": report.issues,
                "report_artifact": report.report_artifact,
                "metadata": report.metadata,
            }),
            signed_operation: report.signed_operation.clone(),
        }
    }

    fn task_dispatch(
        workflow_id: &str,
        stage_id: &str,
        receipt: &TaskDispatchReceipt,
        signed: SignedOperation,
    ) -> Self {
        Self {
            kind: EvidenceLedgerKind::TaskDispatch,
            timestamp: current_timestamp_millis(),
            reference: signed.signature.clone(),
            payload: json!({
                "workflow_id": workflow_id,
                "stage_id": stage_id,
                "agent": receipt.agent_metadata.agent_id,
                "agent_name": receipt.agent_metadata.name,
                "tool_receipts": receipt.tool_receipts,
                "output": receipt.output,
            }),
            signed_operation: signed,
        }
    }

    fn auto_fix_action(
        fixer: &str,
        target: &str,
        snapshot: &PathBuf,
        plan: &Value,
        policy: &PolicyDecisionRecord,
        signed: SignedOperation,
    ) -> Self {
        Self {
            kind: EvidenceLedgerKind::AutoFixAction,
            timestamp: current_timestamp_millis(),
            reference: signed.signature.clone(),
            payload: json!({
                "fixer": fixer,
                "target": target,
                "snapshot": snapshot,
                "plan": plan,
                "policy": policy,
            }),
            signed_operation: signed,
        }
    }

    fn budget_decision(
        workflow_id: &str,
        stage_id: &str,
        tokens_used: f64,
        token_limit: f64,
        latency_ms: f64,
        latency_limit: f64,
        action: &str,
        rewritten_plan: &Option<Value>,
        snapshot: &PathBuf,
        signed: SignedOperation,
    ) -> Self {
        Self {
            kind: EvidenceLedgerKind::BudgetDecision,
            timestamp: current_timestamp_millis(),
            reference: signed.signature.clone(),
            payload: json!({
                "workflow_id": workflow_id,
                "stage_id": stage_id,
                "tokens_used": tokens_used,
                "token_limit": token_limit,
                "latency_ms": latency_ms,
                "latency_limit": latency_limit,
                "action": action,
                "rewritten_plan": rewritten_plan,
                "snapshot": snapshot,
            }),
            signed_operation: signed,
        }
    }

    fn genesis() -> Self {
        let record =
            OperationRecord::new(OperationKind::Other, "system/bootstrap", "evidence_ledger")
                .with_metadata(json!({"message": "ledger initialised"}));
        let signed = security::enforce_operation(record).expect("genesis signing");
        Self {
            kind: EvidenceLedgerKind::Genesis,
            timestamp: current_timestamp_millis(),
            reference: "GENESIS".to_string(),
            payload: json!({"message": "ledger initialised"}),
            signed_operation: signed,
        }
    }
}

#[derive(Debug)]
pub struct PipelineInstrumentation {
    index_dir: PathBuf,
    mirror_dir: PathBuf,
    evidence_dir: PathBuf,
    auto_fix_dir: PathBuf,
    budget_guardian_dir: PathBuf,
    evidence_ledger_path: PathBuf,
    goal_metrics_path: PathBuf,
    deployment_report_path: PathBuf,
    goal_metrics: Mutex<GoalMetricStore>,
    metrics_dir: PathBuf,
    reward_history_path: PathBuf,
    reward_scorekeeper: Mutex<RewardScorekeeper>,
}

impl PipelineInstrumentation {
    pub fn new() -> Result<Self, InstrumentationError> {
        let index_dir = resolve_path(INDEX_DIR);
        let mirror_dir = resolve_path(STORAGE_MIRROR_DIR);
        let evidence_dir = resolve_path(EVIDENCE_LEDGER_DIR);
        let analytics_dir = resolve_path(GOAL_ANALYTICS_DIR);
        let metrics_dir = resolve_path(METRICS_DIR);
        fs::create_dir_all(&index_dir)?;
        fs::create_dir_all(&mirror_dir)?;
        fs::create_dir_all(&evidence_dir)?;
        fs::create_dir_all(&analytics_dir)?;
        let auto_fix_dir = mirror_dir.join(AUTO_FIX_DIR);
        let budget_guardian_dir = mirror_dir.join(BUDGET_GUARDIAN_DIR);
        fs::create_dir_all(&auto_fix_dir)?;
        fs::create_dir_all(&budget_guardian_dir)?;
        fs::create_dir_all(&metrics_dir)?;

        let evidence_ledger_path = evidence_dir.join(EVIDENCE_LEDGER_FILE);
        let goal_metrics_path = analytics_dir.join(GOAL_ANALYTICS_FILE);
        let deployment_report_path = resolve_path(DEPLOYMENT_REPORT_PATH);
        if let Some(parent) = deployment_report_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let goal_metrics = Mutex::new(load_goal_metrics(&goal_metrics_path)?);
        let reward_history_path = metrics_dir.join(REWARD_HISTORY_FILE);
        let reward_scorekeeper = Mutex::new(RewardScorekeeper::new(reward_history_path.clone())?);

        let instrumentation = Self {
            index_dir,
            mirror_dir,
            evidence_dir,
            auto_fix_dir,
            budget_guardian_dir,
            evidence_ledger_path,
            goal_metrics_path,
            deployment_report_path,
            goal_metrics,
            metrics_dir,
            reward_history_path,
            reward_scorekeeper,
        };

        instrumentation.ensure_genesis(RELOCATION_LOG, OperationKind::FileMove)?;
        instrumentation.ensure_genesis(DOCUMENT_LOG, OperationKind::DocumentUpdate)?;
        instrumentation.ensure_genesis(STAGE_RECEIPT_LOG, OperationKind::StageReceipt)?;
        instrumentation.ensure_genesis(TASK_DISPATCH_LOG, OperationKind::Other)?;
        instrumentation.ensure_genesis(AUTO_FIX_LOG, OperationKind::Other)?;
        instrumentation.ensure_genesis(BUDGET_DECISION_LOG, OperationKind::Other)?;
        instrumentation.ensure_genesis(SECURITY_SCAN_LOG, OperationKind::SecurityScan)?;
        instrumentation.ensure_genesis(INFERENCE_LOG, OperationKind::Other)?;
        instrumentation.ensure_genesis(PIPELINE_EVENT_LOG, OperationKind::Other)?;
        instrumentation.ensure_evidence_ledger()?;
        instrumentation.ensure_goal_metrics()?;
        instrumentation.ensure_reward_history()?;
        instrumentation.ensure_deployment_report()?;

        Ok(instrumentation)
    }

    pub fn log_channels() -> &'static [&'static str] {
        &LOG_CHANNELS
    }

    fn ensure_genesis(
        &self,
        log_name: &str,
        kind: OperationKind,
    ) -> Result<(), InstrumentationError> {
        with_log_lock(|| {
            let path = self.log_path(log_name);
            let needs_genesis = if path.exists() {
                let content = fs::read_to_string(&path)?;
                content.trim().is_empty()
            } else {
                true
            };

            if !needs_genesis {
                return Ok(());
            }

            let event = PipelineLogEvent {
                event_type: format!("{}::genesis", log_name),
                actor: "system/bootstrap".to_string(),
                scope: "instrumentation".to_string(),
                source: None,
                target: None,
                metadata: json!({"message": "ledger initialised"}),
                timestamp: current_timestamp_millis(),
            };
            let record = OperationRecord::new(kind.clone(), "system/bootstrap", "instrumentation")
                .with_metadata(json!({"initialised": true}));
            let signed = security::enforce_operation(record)?;
            let entry = ImmutableLogEntry::new(event, signed, "GENESIS".to_string())?;
            self.write_entry(log_name, &entry)?;
            Ok(())
        })
    }

    pub fn log_relocation(
        &self,
        actor: &str,
        source: &str,
        target: &str,
        metadata: Value,
    ) -> Result<SignedOperation, InstrumentationError> {
        let record_metadata = json!({
            "pipeline": "relocation",
            "details": metadata.clone(),
        });
        let event = PipelineLogEvent {
            event_type: "relocation".to_string(),
            actor: actor.to_string(),
            scope: "relocation_pipeline".to_string(),
            source: Some(source.to_string()),
            target: Some(target.to_string()),
            metadata,
            timestamp: current_timestamp_millis(),
        };
        let record = OperationRecord::new(
            OperationKind::FileMove,
            actor.to_string(),
            target.to_string(),
        )
        .with_context(Some(source.to_string()), Some(target.to_string()))
        .with_metadata(record_metadata);
        self.append_entry(RELOCATION_LOG, event, record)
    }

    pub fn log_document_update(
        &self,
        actor: &str,
        document_path: &str,
        metadata: Value,
    ) -> Result<SignedOperation, InstrumentationError> {
        let record_metadata = json!({
            "pipeline": "documentation",
            "details": metadata.clone(),
        });
        let event = PipelineLogEvent {
            event_type: "documentation".to_string(),
            actor: actor.to_string(),
            scope: document_path.to_string(),
            source: None,
            target: Some(document_path.to_string()),
            metadata,
            timestamp: current_timestamp_millis(),
        };
        let record = OperationRecord::new(
            OperationKind::DocumentUpdate,
            actor.to_string(),
            document_path.to_string(),
        )
        .with_context(None, Some(document_path.to_string()))
        .with_metadata(record_metadata);
        self.append_entry(DOCUMENT_LOG, event, record)
    }

    pub fn log_task_dispatch(
        &self,
        workflow_id: &str,
        stage_id: &str,
        receipt: &TaskDispatchReceipt,
    ) -> Result<(), InstrumentationError> {
        let event = PipelineLogEvent {
            event_type: "task.dispatch".to_string(),
            actor: receipt.agent_metadata.agent_id.clone(),
            scope: format!("{}::{}", workflow_id, stage_id),
            source: Some(workflow_id.to_string()),
            target: Some(stage_id.to_string()),
            metadata: json!({
                "agent": receipt.agent_metadata.agent_id,
                "tool_receipts": receipt.tool_receipts,
                "output": receipt.output,
            }),
            timestamp: current_timestamp_millis(),
        };
        let record = OperationRecord::new(
            OperationKind::Other,
            receipt.agent_metadata.agent_id.clone(),
            stage_id.to_string(),
        )
        .with_context(Some(workflow_id.to_string()), Some(stage_id.to_string()))
        .with_metadata(json!({
            "agent_name": receipt.agent_metadata.name,
            "tool_receipts": receipt.tool_receipts,
        }));
        let signed = self.append_entry(TASK_DISPATCH_LOG, event, record)?;
        self.append_evidence_ledger(EvidenceLedgerEntry::task_dispatch(
            workflow_id,
            stage_id,
            receipt,
            signed,
        ))
    }

    pub fn log_stage_receipt(
        &self,
        workflow_id: &str,
        stage: &Stage,
        artifacts: &[Value],
    ) -> Result<StageReceipt, InstrumentationError> {
        let receipt = StageReceipt::new(workflow_id, stage, artifacts)?;
        let stage_name = stage.name.clone();
        let stage_type = stage.stage_type.clone();
        let stage_name_for_metadata = stage_name.clone();
        let stage_name_for_record = stage_name.clone();
        let event_scope = format!("{}::{}", workflow_id, stage_name);
        let record_metadata = json!({
            "workflow_id": workflow_id,
            "stage_id": stage_name_for_metadata,
            "stage_type": stage_type,
            "merkle_root": receipt.merkle_root,
            "leaf_count": receipt.leaves.len(),
        });
        let event = PipelineLogEvent {
            event_type: "stage_receipt".to_string(),
            actor: "workflow_engine".to_string(),
            scope: event_scope,
            source: None,
            target: None,
            metadata: json!({ "receipt": receipt.clone() }),
            timestamp: current_timestamp_millis(),
        };
        let record = OperationRecord::new(
            OperationKind::StageReceipt,
            workflow_id.to_string(),
            stage_name_for_record,
        )
        .with_metadata(record_metadata);
        let signed = self.append_entry(STAGE_RECEIPT_LOG, event, record)?;
        self.append_evidence_ledger(EvidenceLedgerEntry::stage_receipt(&receipt, signed))?;
        Ok(receipt)
    }

    pub fn record_budget_decision(
        &self,
        workflow_id: &str,
        stage_id: &str,
        tokens_used: f64,
        token_limit: f64,
        latency_ms: f64,
        latency_limit: f64,
        action: &str,
        rewritten_plan: Option<Value>,
    ) -> Result<BudgetDecisionRecord, InstrumentationError> {
        let timestamp = current_timestamp_millis();
        let filename = format!(
            "{}-{}-{}-budget.json",
            timestamp,
            workflow_id,
            stage_id.replace('/', "_")
        );
        let snapshot_path = self.budget_guardian_dir.join(filename);
        if let Some(parent) = snapshot_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let manifest = json!({
            "workflow_id": workflow_id,
            "stage_id": stage_id,
            "recorded_at": timestamp,
            "tokens_used": tokens_used,
            "token_limit": token_limit,
            "latency_ms": latency_ms,
            "latency_limit": latency_limit,
            "action": action,
            "rewritten_plan": rewritten_plan,
        });
        fs::write(&snapshot_path, serde_json::to_string_pretty(&manifest)?)?;

        let event = PipelineLogEvent {
            event_type: "budget.guardian".to_string(),
            actor: workflow_id.to_string(),
            scope: stage_id.to_string(),
            source: None,
            target: Some(snapshot_path.to_string_lossy().to_string()),
            metadata: manifest.clone(),
            timestamp,
        };
        let record = OperationRecord::new(
            OperationKind::Other,
            workflow_id.to_string(),
            stage_id.to_string(),
        )
        .with_metadata(json!({
            "tokens_used": tokens_used,
            "token_limit": token_limit,
            "latency_ms": latency_ms,
            "latency_limit": latency_limit,
            "action": action,
            "snapshot": snapshot_path.to_string_lossy(),
        }));
        let signed = self.append_entry(BUDGET_DECISION_LOG, event, record)?;
        let record = BudgetDecisionRecord {
            workflow_id: workflow_id.to_string(),
            stage_id: stage_id.to_string(),
            tokens_used,
            token_limit,
            latency_ms,
            latency_limit,
            action: action.to_string(),
            rewritten_plan: rewritten_plan.clone(),
            snapshot_path: snapshot_path.clone(),
            signed_operation: signed.clone(),
        };
        self.append_evidence_ledger(EvidenceLedgerEntry::budget_decision(
            workflow_id,
            stage_id,
            tokens_used,
            token_limit,
            latency_ms,
            latency_limit,
            action,
            &record.rewritten_plan,
            &record.snapshot_path,
            signed,
        ))?;
        Ok(record)
    }

    pub fn record_auto_fix_action(
        &self,
        fixer: &str,
        target: &str,
        plan: &Value,
        policy: &PolicyDecisionRecord,
    ) -> Result<AutoFixActionReceipt, InstrumentationError> {
        let timestamp = current_timestamp_millis();
        let filename = format!("{}-{}-auto-fix.json", timestamp, fixer.replace('/', "_"));
        let snapshot_path = self.auto_fix_dir.join(filename);
        if let Some(parent) = snapshot_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let manifest = json!({
            "fixer": fixer,
            "target": target,
            "recorded_at": timestamp,
            "plan": plan,
            "policy": policy,
        });
        fs::write(&snapshot_path, serde_json::to_string_pretty(&manifest)?)?;

        let plan_serialised = serde_json::to_string(plan)?;
        let policy_serialised = serde_json::to_string(policy)?;

        let event = PipelineLogEvent {
            event_type: "auto_fix.action".to_string(),
            actor: fixer.to_string(),
            scope: target.to_string(),
            source: None,
            target: Some(snapshot_path.to_string_lossy().to_string()),
            metadata: manifest.clone(),
            timestamp,
        };
        let record =
            OperationRecord::new(OperationKind::Other, fixer.to_string(), target.to_string())
                .with_metadata(json!({
                    "snapshot": snapshot_path.to_string_lossy(),
                    "plan_digest": simple_hash(&plan_serialised),
                    "policy_digest": simple_hash(&policy_serialised),
                }));
        let signed = self.append_entry(AUTO_FIX_LOG, event, record)?;
        let receipt = AutoFixActionReceipt {
            fixer: fixer.to_string(),
            target: target.to_string(),
            snapshot_path: snapshot_path.clone(),
            plan: plan.clone(),
            policy: policy.clone(),
            signed_operation: signed.clone(),
        };
        self.append_evidence_ledger(EvidenceLedgerEntry::auto_fix_action(
            fixer,
            target,
            &snapshot_path,
            plan,
            policy,
            signed,
        ))?;
        Ok(receipt)
    }

    pub fn log_security_scan(
        &self,
        subject: &str,
        tool: &str,
        status: SecurityScanStatus,
        issues: Vec<String>,
        report_artifact: Option<String>,
        metadata: Value,
    ) -> Result<SecurityScanReport, InstrumentationError> {
        let issues_for_event = issues.clone();
        let metadata_for_event = metadata.clone();
        let report_artifact_for_event = report_artifact.clone();
        let report_artifact_for_record = report_artifact.clone();
        let report_artifact_for_report = report_artifact.clone();
        let event = PipelineLogEvent {
            event_type: "security_scan".to_string(),
            actor: tool.to_string(),
            scope: subject.to_string(),
            source: None,
            target: None,
            metadata: json!({
                "status": status,
                "issues": issues_for_event,
                "report_artifact": report_artifact_for_event,
                "metadata": metadata_for_event,
            }),
            timestamp: current_timestamp_millis(),
        };
        let record = OperationRecord::new(
            OperationKind::SecurityScan,
            tool.to_string(),
            subject.to_string(),
        )
        .with_metadata(json!({
            "status": status,
            "issue_count": issues.len(),
            "report_artifact": report_artifact_for_record,
        }));
        let signed = self.append_entry(SECURITY_SCAN_LOG, event, record)?;
        let report = SecurityScanReport {
            subject: subject.to_string(),
            tool: tool.to_string(),
            status,
            issues,
            report_artifact: report_artifact_for_report,
            signed_operation: signed.clone(),
            ledger_reference: signed.signature.clone(),
            metadata,
        };
        self.append_evidence_ledger(EvidenceLedgerEntry::security_scan(subject, &report))?;
        Ok(report)
    }

    pub fn log_pipeline_event(
        &self,
        actor: &str,
        subject: &str,
        event_type: &str,
        metadata: Value,
    ) -> Result<SignedOperation, InstrumentationError> {
        let metadata_for_event = metadata.clone();
        let event = PipelineLogEvent {
            event_type: event_type.to_string(),
            actor: actor.to_string(),
            scope: subject.to_string(),
            source: Some(actor.to_string()),
            target: Some(subject.to_string()),
            metadata: metadata_for_event,
            timestamp: current_timestamp_millis(),
        };
        let record =
            OperationRecord::new(OperationKind::Other, actor.to_string(), subject.to_string())
                .with_context(Some(actor.to_string()), Some(subject.to_string()))
                .with_metadata(metadata);
        self.append_entry(PIPELINE_EVENT_LOG, event, record)
    }
    pub fn record_deployment_outcome(
        &self,
        record: DeploymentOutcomeRecord,
    ) -> Result<(), InstrumentationError> {
        self.ensure_deployment_report()?;
        let notes = serde_json::to_string(&record.notes)?;
        let sanitized_notes = notes.replace('|', "\\|").replace('\n', " ");
        let row = format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            record.recorded_at,
            record.workflow_id,
            record.stage_id,
            record.agent_role,
            record.agent_id,
            record.action,
            record.status,
            sanitized_notes
        );
        with_log_lock(|| {
            if let Some(parent) = self.deployment_report_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.deployment_report_path)?;
            writeln!(file, "{}", row)?;
            file.flush()?;
            file.sync_all()?;
            Ok(())
        })
    }

    pub fn record_goal_outcome(
        &self,
        outcome: GoalOutcomeRecord,
    ) -> Result<(), InstrumentationError> {
        {
            let mut store = self.goal_metrics.lock().unwrap();
            store.record(&outcome);
        }
        if let Some(inputs) = outcome.reward_inputs.clone() {
            let agent_snapshots: Vec<RewardAgentSnapshot> = outcome
                .agents
                .iter()
                .map(|agent| RewardAgentSnapshot {
                    agent: agent.agent.clone(),
                    success: agent.success,
                })
                .collect();
            let mut keeper = self.reward_scorekeeper.lock().unwrap();
            let delta = keeper.record(
                &outcome.goal_id,
                &outcome.workflow_id,
                inputs,
                &agent_snapshots,
            );
            self.persist_reward_history(&*keeper)?;
            drop(keeper);
            println!(
                "[REWARD] {}::{} delta={:.2} coverage={:.2} flake={:.2} tokens={:.2} rollback={:.2}",
                outcome.goal_id,
                outcome.workflow_id,
                delta.total_reward,
                delta.coverage_delta,
                delta.flake_delta,
                delta.token_delta,
                delta.rollback_delta
            );
        }
        self.persist_goal_metrics()
    }

    pub fn record_context_usage(
        &self,
        workflow_id: &str,
        agent: &str,
        context_bytes: usize,
        threshold: usize,
        retrieval_ms: u128,
    ) -> Result<(), InstrumentationError> {
        {
            let mut store = self.goal_metrics.lock().unwrap();
            store.penalize_context(workflow_id, agent, context_bytes, threshold, retrieval_ms);
        }
        self.persist_goal_metrics()
    }

    pub fn goal_metrics_snapshot(&self) -> Result<Vec<GoalMetricSnapshot>, InstrumentationError> {
        let store = self.goal_metrics.lock().unwrap();
        Ok(store.snapshots())
    }

    pub fn evaluate_agent_for_execution(&self, agent: &str) -> AgentApprovalStatus {
        let keeper = self.reward_scorekeeper.lock().unwrap();
        keeper.approval_status(agent)
    }

    pub fn flagged_agents(&self) -> Vec<AgentStandingSummary> {
        let keeper = self.reward_scorekeeper.lock().unwrap();
        keeper.flagged_agents()
    }

    pub fn log_inference_metric(
        &self,
        metric: InferenceMetric,
    ) -> Result<(), InstrumentationError> {
        let provider = metric.provider;
        let model = metric.model;
        let status = metric.status;
        let latency_ms = metric.latency_ms;
        let tokens_prompt = metric.tokens_prompt;
        let tokens_completion = metric.tokens_completion;
        let error = metric.error;
        let metadata = json!({
            "provider": provider,
            "model": model,
            "status": status,
            "latency_ms": latency_ms,
            "tokens_prompt": tokens_prompt,
            "tokens_completion": tokens_completion,
            "error": error,
        });
        let event = PipelineLogEvent {
            event_type: "inference.metric".to_string(),
            actor: provider.clone(),
            scope: model.clone(),
            source: None,
            target: None,
            metadata: metadata.clone(),
            timestamp: current_timestamp_millis(),
        };
        let record =
            OperationRecord::new(OperationKind::Other, provider, model).with_metadata(metadata);

        self.append_entry(INFERENCE_LOG, event, record)?;
        Ok(())
    }

    fn append_entry(
        &self,
        log_name: &str,
        event: PipelineLogEvent,
        record: OperationRecord,
    ) -> Result<SignedOperation, InstrumentationError> {
        with_log_lock(move || {
            let previous_hash = self.tail_hash_locked(log_name)?;
            let signed = security::enforce_operation(record)?;
            let entry = ImmutableLogEntry::new(event, signed.clone(), previous_hash)?;
            self.write_entry(log_name, &entry)?;
            Ok(signed)
        })
    }

    fn ensure_evidence_ledger(&self) -> Result<(), InstrumentationError> {
        with_log_lock(|| {
            if self.evidence_ledger_path.exists() {
                let content = fs::read_to_string(&self.evidence_ledger_path)?;
                if !content.trim().is_empty() {
                    return Ok(());
                }
            }

            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.evidence_ledger_path)?;
            let entry = EvidenceLedgerEntry::genesis();
            let payload = serde_json::to_string(&entry)?;
            writeln!(file, "{}", payload)?;
            file.flush()?;
            file.sync_all()?;
            Ok(())
        })
    }

    fn ensure_goal_metrics(&self) -> Result<(), InstrumentationError> {
        with_log_lock(|| {
            if self.goal_metrics_path.exists() {
                return Ok(());
            }
            let store = self.goal_metrics.lock().unwrap();
            let payload = serde_json::to_string_pretty(&*store)?;
            drop(store);
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&self.goal_metrics_path)?;
            file.write_all(payload.as_bytes())?;
            file.flush()?;
            file.sync_all()?;
            Ok(())
        })
    }

    fn ensure_reward_history(&self) -> Result<(), InstrumentationError> {
        with_log_lock(|| {
            if self.reward_history_path.exists() {
                return Ok(());
            }
            let keeper = self.reward_scorekeeper.lock().unwrap();
            keeper.save()?;
            Ok(())
        })
    }

    fn ensure_deployment_report(&self) -> Result<(), InstrumentationError> {
        with_log_lock(|| {
            if self.deployment_report_path.exists() {
                let content = fs::read_to_string(&self.deployment_report_path)?;
                if !content.trim().is_empty() {
                    return Ok(());
                }
            }

            if let Some(parent) = self.deployment_report_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&self.deployment_report_path)?;
            file.write_all(b"# Agent Deployment Outcomes\n\n| Timestamp | Workflow | Stage | Agent Role | Agent ID | Action | Status | Notes |\n| --- | --- | --- | --- | --- | --- | --- | --- |\n")?;
            file.flush()?;
            file.sync_all()?;
            Ok(())
        })
    }

    fn persist_goal_metrics(&self) -> Result<(), InstrumentationError> {
        let store = self.goal_metrics.lock().unwrap();
        let snapshots = store.snapshots();
        drop(store);
        let payload = serde_json::to_string_pretty(&snapshots)?;
        with_log_lock(|| {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&self.goal_metrics_path)?;
            file.write_all(payload.as_bytes())?;
            file.flush()?;
            file.sync_all()?;
            Ok(())
        })
    }

    fn persist_reward_history(
        &self,
        keeper: &RewardScorekeeper,
    ) -> Result<(), InstrumentationError> {
        let payload = serde_json::to_string_pretty(keeper.history())?;
        with_log_lock(|| {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&self.reward_history_path)?;
            file.write_all(payload.as_bytes())?;
            file.flush()?;
            file.sync_all()?;
            Ok(())
        })
    }

    fn append_evidence_ledger(
        &self,
        entry: EvidenceLedgerEntry,
    ) -> Result<(), InstrumentationError> {
        with_log_lock(|| {
            let payload = serde_json::to_string(&entry)?;
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.evidence_ledger_path)?;
            writeln!(file, "{}", payload)?;
            file.flush()?;
            file.sync_all()?;
            Ok(())
        })
    }

    fn tail_hash_locked(&self, log_name: &str) -> Result<String, InstrumentationError> {
        let path = self.log_path(log_name);
        if !path.exists() {
            return Ok("GENESIS".to_string());
        }
        let content = fs::read_to_string(path)?;
        for line in content.lines().rev() {
            if line.trim().is_empty() {
                continue;
            }
            let entry: ImmutableLogEntry = serde_json::from_str(line)?;
            return Ok(entry.entry_hash);
        }
        Ok("GENESIS".to_string())
    }

    fn log_path(&self, log_name: &str) -> PathBuf {
        self.index_dir.join(format!("{}.log", log_name))
    }

    fn write_entry(
        &self,
        log_name: &str,
        entry: &ImmutableLogEntry,
    ) -> Result<(), InstrumentationError> {
        let serialised = serde_json::to_string(entry)?;
        let payload = format!("{}\n", serialised);

        for base in [&self.index_dir, &self.mirror_dir] {
            let path = base.join(format!("{}.log", log_name));
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(path)?;
            file.write_all(payload.as_bytes())?;
            file.flush()?;
            file.sync_all()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineStorageLayout {
    pub index_dir: PathBuf,
    pub mirror_dir: PathBuf,
}

impl PipelineStorageLayout {
    pub fn new() -> Self {
        Self {
            index_dir: resolve_path(INDEX_DIR),
            mirror_dir: resolve_path(STORAGE_MIRROR_DIR),
        }
    }

    pub fn log_pair(&self, log_name: &str) -> (PathBuf, PathBuf) {
        let file = format!("{}.log", log_name);
        (self.index_dir.join(&file), self.mirror_dir.join(&file))
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StorageDoctorStatus {
    Healthy,
    Degraded,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogMirrorReport {
    pub name: String,
    pub index_path: String,
    pub storage_path: String,
    pub index_exists: bool,
    pub storage_exists: bool,
    pub index_genesis: Option<bool>,
    pub storage_genesis: Option<bool>,
    pub drift: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct StorageDoctorReport {
    pub status: StorageDoctorStatus,
    pub mirrors: Vec<LogMirrorReport>,
    pub drift: Vec<String>,
}

impl StorageDoctorReport {
    pub fn is_healthy(&self) -> bool {
        self.status == StorageDoctorStatus::Healthy
    }
}

pub fn run_storage_doctor() -> Result<StorageDoctorReport, InstrumentationError> {
    let layout = PipelineStorageLayout::new();
    let mut mirrors = Vec::new();
    let mut drift_channels = Vec::new();
    let mut healthy = true;

    for log_name in PipelineInstrumentation::log_channels() {
        let (index_path, storage_path) = layout.log_pair(log_name);
        let index_exists = index_path.exists();
        let storage_exists = storage_path.exists();

        let index_genesis = if index_exists {
            Some(first_entry_is_genesis(&index_path, log_name)?)
        } else {
            None
        };
        let storage_genesis = if storage_exists {
            Some(first_entry_is_genesis(&storage_path, log_name)?)
        } else {
            None
        };

        let drift = if index_exists && storage_exists {
            fs::read_to_string(&index_path)? != fs::read_to_string(&storage_path)?
        } else {
            false
        };

        if drift {
            drift_channels.push(log_name.to_string());
        }

        if !index_exists
            || !storage_exists
            || index_genesis != Some(true)
            || storage_genesis != Some(true)
            || drift
        {
            healthy = false;
        }

        mirrors.push(LogMirrorReport {
            name: log_name.to_string(),
            index_path: index_path.to_string_lossy().to_string(),
            storage_path: storage_path.to_string_lossy().to_string(),
            index_exists,
            storage_exists,
            index_genesis,
            storage_genesis,
            drift,
        });
    }

    Ok(StorageDoctorReport {
        status: if healthy {
            StorageDoctorStatus::Healthy
        } else {
            StorageDoctorStatus::Degraded
        },
        mirrors,
        drift: drift_channels,
    })
}

fn first_entry_is_genesis(path: &PathBuf, log_name: &str) -> Result<bool, InstrumentationError> {
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(trimmed)?;
        let event_type = value
            .get("event")
            .and_then(|event| event.get("event_type"))
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let previous_hash = value
            .get("previous_hash")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let expected_event = format!("{}::genesis", log_name);
        return Ok(event_type == expected_event && previous_hash == "GENESIS");
    }
    Ok(false)
}

fn log_write_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn with_log_lock<T>(
    f: impl FnOnce() -> Result<T, InstrumentationError>,
) -> Result<T, InstrumentationError> {
    let _guard = log_write_lock()
        .lock()
        .map_err(|_| std::io::Error::new(ErrorKind::Other, "log write lock poisoned"))?;
    f()
}

fn build_merkle_tree(
    workflow_id: &str,
    stage_id: &str,
    leaves: &[MerkleLeaf],
) -> (Vec<MerkleLevel>, String) {
    if leaves.is_empty() {
        let root = simple_hash(&format!("{}::{}::empty", workflow_id, stage_id));
        return (
            vec![MerkleLevel {
                level: 0,
                nodes: vec![root.clone()],
            }],
            root,
        );
    }

    let mut levels = Vec::new();
    let mut current: Vec<String> = leaves.iter().map(|leaf| leaf.hash.clone()).collect();
    levels.push(MerkleLevel {
        level: 0,
        nodes: current.clone(),
    });
    let mut level_index = 1;

    while current.len() > 1 {
        let mut next = Vec::new();
        for chunk in current.chunks(2) {
            let left = &chunk[0];
            let right = if chunk.len() == 2 { &chunk[1] } else { left };
            next.push(simple_hash(&format!("{}::{}", left, right)));
        }
        levels.push(MerkleLevel {
            level: level_index,
            nodes: next.clone(),
        });
        current = next;
        level_index += 1;
    }

    let root = current
        .first()
        .cloned()
        .unwrap_or_else(|| simple_hash(&format!("{}::{}::empty", workflow_id, stage_id)));
    (levels, root)
}

// Manual Clone implementation for PipelineInstrumentation.
// While PathBuf implements Clone, this explicit implementation is provided for
// clarity and future extensibility. PathBuf::clone is cheap (Arc-based internally).
impl Clone for PipelineInstrumentation {
    fn clone(&self) -> Self {
        let metrics = self.goal_metrics.lock().unwrap().clone();
        let reward = self.reward_scorekeeper.lock().unwrap().clone();
        Self {
            index_dir: self.index_dir.clone(),
            mirror_dir: self.mirror_dir.clone(),
            evidence_dir: self.evidence_dir.clone(),
            auto_fix_dir: self.auto_fix_dir.clone(),
            budget_guardian_dir: self.budget_guardian_dir.clone(),
            evidence_ledger_path: self.evidence_ledger_path.clone(),
            goal_metrics_path: self.goal_metrics_path.clone(),
            deployment_report_path: self.deployment_report_path.clone(),
            goal_metrics: Mutex::new(metrics),
            metrics_dir: self.metrics_dir.clone(),
            reward_history_path: self.reward_history_path.clone(),
            reward_scorekeeper: Mutex::new(reward),
        }
    }
}

fn resolve_path(relative: &str) -> PathBuf {
    if let Ok(root) = std::env::var("NOA_WORKFLOW_ROOT") {
        return PathBuf::from(root).join(relative);
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .map(|root| root.join(relative))
        .unwrap_or_else(|| PathBuf::from(relative))
}

fn load_goal_metrics(path: &PathBuf) -> Result<GoalMetricStore, InstrumentationError> {
    if !path.exists() {
        return Ok(GoalMetricStore::default());
    }
    let raw = fs::read_to_string(path)?;
    if raw.trim().is_empty() {
        return Ok(GoalMetricStore::default());
    }
    match serde_json::from_str::<GoalMetricStore>(&raw) {
        Ok(store) => Ok(store),
        Err(_) => {
            let snapshots: Vec<GoalMetricSnapshot> = serde_json::from_str(&raw)?;
            let mut store = GoalMetricStore::default();
            for snapshot in snapshots {
                let mut aggregate = GoalAggregate::new(&snapshot.goal_id, &snapshot.workflow_id);
                aggregate.total_runs = snapshot.total_runs;
                aggregate.successful_runs = snapshot.successful_runs;
                let duration =
                    (snapshot.average_lead_time_ms * snapshot.total_runs as f64).round() as u128;
                aggregate.total_duration_ms = duration;
                for agent in snapshot.agents {
                    aggregate.agents.insert(
                        agent.agent.clone(),
                        AgentAggregate {
                            total_runs: agent.total_runs,
                            successful_runs: agent.successful_runs,
                        },
                    );
                }
                store.goals.insert(snapshot.goal_id.clone(), aggregate);
                if snapshot.context_penalty_score > 0.0
                    || snapshot.context_p95_bytes > 0
                    || snapshot.context_p95_latency_ms > 0
                {
                    let mut context = ContextPenaltyAggregate::new(&snapshot.workflow_id);
                    context.push_summary(
                        snapshot.context_penalty_score,
                        snapshot.context_p95_bytes,
                        snapshot.context_p95_latency_ms,
                    );
                    store.context.insert(snapshot.workflow_id.clone(), context);
                }
            }
            Ok(store)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::{Mutex, OnceLock};
    use tempfile::tempdir;

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    struct EnvGuard {
        key: &'static str,
        prev: Option<std::ffi::OsString>,
        lock: Option<std::sync::MutexGuard<'static, ()>>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &PathBuf) -> Self {
            let guard = env_lock().lock().expect("workflow env lock poisoned");
            let prev = std::env::var_os(key);
            std::env::set_var(key, value);
            Self {
                key,
                prev,
                lock: Some(guard),
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(ref val) = self.prev {
                std::env::set_var(self.key, val);
            } else {
                std::env::remove_var(self.key);
            }
            self.lock.take();
        }
    }

    fn sample_stage() -> Stage {
        Stage {
            name: "build".to_string(),
            stage_type: StageType::Sequential,
            depends_on: vec![],
            tasks: vec![Task {
                agent: "builder".to_string(),
                action: "compile".to_string(),
                parameters: HashMap::from([("target".to_string(), json!({"path": "src/main.rs"}))]),
                tool_requirements: Vec::new(),
                agent_role: None,
            }],
        }
    }

    #[test]
    fn merkle_roots_are_deterministic() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let _guard = EnvGuard::set("NOA_WORKFLOW_ROOT", &root);
        let instrumentation = PipelineInstrumentation::new().unwrap();
        let stage = sample_stage();
        let artifacts = vec![json!({"status": "ok"})];

        let first = instrumentation
            .log_stage_receipt("wf", &stage, &artifacts)
            .unwrap();
        let second = instrumentation
            .log_stage_receipt("wf", &stage, &artifacts)
            .unwrap();

        assert_eq!(first.merkle_root, second.merkle_root);
        assert_eq!(first.leaves.len(), second.leaves.len());
        assert_eq!(first.leaves[0].hash, second.leaves[0].hash);
    }

    #[test]
    fn evidence_ledger_appends_stage_receipts() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();
        let _guard = EnvGuard::set("NOA_WORKFLOW_ROOT", &root);
        let instrumentation = PipelineInstrumentation::new().unwrap();
        let stage = sample_stage();
        let artifacts = vec![json!({"status": "ok"})];

        instrumentation
            .log_stage_receipt("wf", &stage, &artifacts)
            .unwrap();

        let ledger_path = root.join(EVIDENCE_LEDGER_DIR).join(EVIDENCE_LEDGER_FILE);
        let content = fs::read_to_string(ledger_path).unwrap();
        assert!(content.lines().count() >= 2); // genesis + receipt
    }
}
