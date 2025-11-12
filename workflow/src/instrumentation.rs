use noa_core::security::{self, OperationKind, OperationRecord, SignedOperation};
use noa_core::time::current_timestamp_millis;
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

#[derive(Debug)]
pub struct PipelineInstrumentation {
    index_dir: PathBuf,
    mirror_dir: PathBuf,
}

impl PipelineInstrumentation {
    pub fn new() -> Result<Self, InstrumentationError> {
        let index_dir = resolve_path(INDEX_DIR);
        let mirror_dir = resolve_path(STORAGE_MIRROR_DIR);
        fs::create_dir_all(&index_dir)?;
        fs::create_dir_all(&mirror_dir)?;

        let instrumentation = Self {
            index_dir,
            mirror_dir,
        };

        instrumentation.ensure_genesis(RELOCATION_LOG, OperationKind::FileMove)?;
        instrumentation.ensure_genesis(DOCUMENT_LOG, OperationKind::DocumentUpdate)?;

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

impl Clone for PipelineInstrumentation {
    fn clone(&self) -> Self {
        Self {
            index_dir: self.index_dir.clone(),
            mirror_dir: self.mirror_dir.clone(),
        }
    }
}

fn simple_hash(value: &str) -> String {
    const OFFSET_BASIS: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    let mut hash = OFFSET_BASIS;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }

    format!("{:016x}", hash)
}

fn resolve_path(relative: &str) -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(root) = manifest.parent() {
        root.join(relative)
    } else {
        PathBuf::from(relative)
    }
}
