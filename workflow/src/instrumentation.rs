use crate::{Stage, StageType, Task};
use noa_core::security::{self, OperationKind, OperationRecord, SignedOperation};
use noa_core::utils::{current_timestamp_millis, simple_hash};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const INDEX_DIR: &str = ".workspace/indexes";
const STORAGE_MIRROR_DIR: &str = "storage/db";
const RELOCATION_LOG: &str = "relocation";
const DOCUMENT_LOG: &str = "documentation";
const STAGE_RECEIPT_LOG: &str = "stage_receipts";
const SECURITY_SCAN_LOG: &str = "security_scans";
const EVIDENCE_LEDGER_DIR: &str = "storage/db/evidence";
const EVIDENCE_LEDGER_FILE: &str = "ledger.jsonl";

#[derive(Debug)]
pub enum InstrumentationError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
    Security(security::PolicyError),
}

impl std::fmt::Display for InstrumentationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstrumentationError::Io(err) => write!(f, "io error: {}", err),
            InstrumentationError::Serialization(err) => write!(f, "serialization error: {}", err),
            InstrumentationError::Security(err) => write!(f, "policy error: {}", err),
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
    evidence_ledger_path: PathBuf,
}

impl PipelineInstrumentation {
    pub fn new() -> Result<Self, InstrumentationError> {
        let index_dir = resolve_path(INDEX_DIR);
        let mirror_dir = resolve_path(STORAGE_MIRROR_DIR);
        let evidence_dir = resolve_path(EVIDENCE_LEDGER_DIR);
        fs::create_dir_all(&index_dir)?;
        fs::create_dir_all(&mirror_dir)?;
        fs::create_dir_all(&evidence_dir)?;

        let evidence_ledger_path = evidence_dir.join(EVIDENCE_LEDGER_FILE);

        let instrumentation = Self {
            index_dir,
            mirror_dir,
            evidence_dir,
            evidence_ledger_path,
        };

        instrumentation.ensure_genesis(RELOCATION_LOG, OperationKind::FileMove)?;
        instrumentation.ensure_genesis(DOCUMENT_LOG, OperationKind::DocumentUpdate)?;
        instrumentation.ensure_genesis(STAGE_RECEIPT_LOG, OperationKind::StageReceipt)?;
        instrumentation.ensure_genesis(SECURITY_SCAN_LOG, OperationKind::SecurityScan)?;
        instrumentation.ensure_evidence_ledger()?;

        Ok(instrumentation)
    }

    fn ensure_genesis(
        &self,
        log_name: &str,
        kind: OperationKind,
    ) -> Result<(), InstrumentationError> {
        with_log_lock(|| {
            let path = self.log_path(log_name);
            // Try to atomically create the file if it doesn't exist
            let file_result = OpenOptions::new().write(true).create_new(true).open(&path);
            match file_result {
                Ok(mut file) => {
                    // File was created, write genesis entry
                    let event = PipelineLogEvent {
                        event_type: format!("{}::genesis", log_name),
                        actor: "system/bootstrap".to_string(),
                        scope: "instrumentation".to_string(),
                        source: None,
                        target: None,
                        metadata: json!({"message": "ledger initialised"}),
                        timestamp: current_timestamp_millis(),
                    };
                    let record =
                        OperationRecord::new(kind.clone(), "system/bootstrap", "instrumentation")
                            .with_metadata(json!({"initialised": true}));
                    let signed = security::enforce_operation(record)?;
                    let entry = ImmutableLogEntry::new(event, signed, "GENESIS".to_string())?;
                    // Write the entry directly to the new file
                    let entry_str = serde_json::to_string(&entry)?;
                    writeln!(file, "{}", entry_str)?;
                    Ok(())
                }
                Err(ref e) if e.kind() == ErrorKind::AlreadyExists => {
                    // File already exists, check if it is empty
                    if let Ok(content) = fs::read_to_string(&path) {
                        if !content.trim().is_empty() {
                            return Ok(());
                        }
                    }
                    // File exists but is empty, write genesis entry
                    let event = PipelineLogEvent {
                        event_type: format!("{}::genesis", log_name),
                        actor: "system/bootstrap".to_string(),
                        scope: "instrumentation".to_string(),
                        source: None,
                        target: None,
                        metadata: json!({"message": "ledger initialised"}),
                        timestamp: current_timestamp_millis(),
                    };
                    let record =
                        OperationRecord::new(kind.clone(), "system/bootstrap", "instrumentation")
                            .with_metadata(json!({"initialised": true}));
                    let signed = security::enforce_operation(record)?;
                    let entry = ImmutableLogEntry::new(event, signed, "GENESIS".to_string())?;
                    // Open for appending and check again before writing
                    let mut file = OpenOptions::new().append(true).open(&path)?;
                    let content = fs::read_to_string(&path)?;
                    if !content.trim().is_empty() {
                        return Ok(());
                    }
                    let entry_str = serde_json::to_string(&entry)?;
                    writeln!(file, "{}", entry_str)?;
                    Ok(())
                }
                Err(e) => Err(InstrumentationError::Io(e)),
            }
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
        Self {
            index_dir: self.index_dir.clone(),
            mirror_dir: self.mirror_dir.clone(),
            evidence_dir: self.evidence_dir.clone(),
            evidence_ledger_path: self.evidence_ledger_path.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;
    use tempfile::tempdir;

    struct EnvGuard {
        key: &'static str,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &PathBuf) -> Self {
            std::env::set_var(key, value);
            Self { key }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            std::env::remove_var(self.key);
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
