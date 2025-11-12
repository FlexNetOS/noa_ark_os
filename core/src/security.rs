//! Security subsystem

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

pub type UserId = u64;

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub name: String,
    pub permissions: Vec<Permission>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Permission {
    Read,
    Write,
    Execute,
    Admin,
}

/// The different types of operations that must be policy enforced.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OperationKind {
    FileMove,
    DocumentUpdate,
    #[serde(other)]
    Other,
}

/// Structured record describing a sensitive operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRecord {
    pub operation_id: String,
    pub kind: OperationKind,
    pub actor: String,
    pub scope: String,
    pub source: Option<String>,
    pub target: Option<String>,
    pub metadata: Value,
    pub timestamp: u128,
}

impl OperationRecord {
    /// Construct a new record with sensible defaults.
    pub fn new(kind: OperationKind, actor: impl Into<String>, scope: impl Into<String>) -> Self {
        Self {
            operation_id: next_operation_id(),
            kind,
            actor: actor.into(),
            scope: scope.into(),
            source: None,
            target: None,
            metadata: json!({}),
            timestamp: current_timestamp_millis(),
        }
    }

    /// Attach optional context to the record.
    pub fn with_context(mut self, source: Option<String>, target: Option<String>) -> Self {
        self.source = source;
        self.target = target;
        self
    }

    /// Attach metadata payload to the record.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Immutable signature bundle for an operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedOperation {
    pub record: OperationRecord,
    pub hash: String,
    pub signature: String,
    pub previous_signature: String,
}

impl SignedOperation {
    /// Produce a human friendly verification statement.
    pub fn verification_statement(&self) -> String {
        format!(
            "operation={} kind={:?} actor={} signature={}",
            self.record.operation_id, self.record.kind, self.record.actor, self.signature
        )
    }
}

#[derive(Debug)]
pub enum PolicyError {
    PermissionDenied(String),
    SigningFailed(String),
}

impl Display for PolicyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PolicyError::PermissionDenied(msg) => write!(f, "permission denied: {}", msg),
            PolicyError::SigningFailed(msg) => write!(f, "signing failed: {}", msg),
        }
    }
}

impl std::error::Error for PolicyError {}

/// Maximum number of signed operations to keep in memory.
/// Older operations are evicted in FIFO order when this limit is reached.
/// For complete audit trails, use persistent ledger storage.
const MAX_IN_MEMORY_RECORDS: usize = 1000;

/// Policy enforcer that signs operations and maintains a bounded in-memory audit trail.
///
/// **Memory Management**: This enforcer keeps only the most recent operations in memory
/// (up to `MAX_IN_MEMORY_RECORDS`). When the limit is reached, the oldest operations
/// are evicted in FIFO order. For production systems requiring complete audit trails,
/// integrate with persistent ledger storage (e.g., blockchain, append-only database).
#[derive(Debug)]
struct PolicyEnforcer {
    secret: String,
    records: Vec<SignedOperation>,
    last_signature: String,
}

impl PolicyEnforcer {
    fn new(secret: String) -> Self {
        Self {
            secret,
            records: Vec::new(),
            last_signature: "GENESIS".to_string(),
        }
    }

    fn compute_hash(record: &OperationRecord) -> Result<String, PolicyError> {
        let serialised = serde_json::to_string(record)
            .map_err(|err| PolicyError::SigningFailed(err.to_string()))?;
        Ok(simple_hash(&serialised))
    }

    fn apply_secret(&self, payload: &str) -> String {
        simple_hash(&format!("{}::{}", self.secret, payload))
    }

    fn sign_and_register(
        &mut self,
        mut record: OperationRecord,
    ) -> Result<SignedOperation, PolicyError> {
        record.timestamp = current_timestamp_millis();
        let hash = Self::compute_hash(&record)?;
        let payload = format!("{}::{}", self.last_signature, hash);
        let signature = self.apply_secret(&payload);
        let signed = SignedOperation {
            record,
            hash,
            signature: signature.clone(),
            previous_signature: self.last_signature.clone(),
        };
        self.last_signature = signature;
        
        // Implement bounded buffer with FIFO eviction
        if self.records.len() >= MAX_IN_MEMORY_RECORDS {
            self.records.remove(0); // Remove oldest record
        }
        self.records.push(signed.clone());
        
        Ok(signed)
    }

    fn verify(&self, signed: &SignedOperation) -> bool {
        if let Ok(hash) = Self::compute_hash(&signed.record) {
            if hash != signed.hash {
                return false;
            }
            let payload = format!("{}::{}", signed.previous_signature, signed.hash);
            let expected = self.apply_secret(&payload);
            expected == signed.signature
        } else {
            false
        }
    }

    /// Returns the in-memory audit trail of recent operations.
    ///
    /// Note: This only includes the most recent operations (up to MAX_IN_MEMORY_RECORDS).
    /// For complete historical audit trails, retrieve records from persistent ledger storage.
    fn audit_trail(&self) -> Vec<SignedOperation> {
        self.records.clone()
    }
}

lazy_static! {
    static ref USER_TABLE: Arc<Mutex<HashMap<UserId, User>>> = Arc::new(Mutex::new(HashMap::new()));
    static ref POLICY_ENFORCER: Arc<Mutex<PolicyEnforcer>> =
        Arc::new(Mutex::new(PolicyEnforcer::new(
            std::env::var("NOA_POLICY_SECRET")
                .unwrap_or_else(|_| "noa-ark-default-policy".to_string())
        )));
    static ref OPERATION_COUNTER: AtomicU64 = AtomicU64::new(1);
}

fn next_operation_id() -> String {
    let counter = OPERATION_COUNTER.fetch_add(1, Ordering::SeqCst);
    let timestamp = current_timestamp_millis();
    format!("op-{}-{}", timestamp, counter)
}

fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or(0)
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

/// Initialize security subsystem
pub fn init() -> Result<(), &'static str> {
    println!("[SECURITY] Initializing security subsystem...");

    // Create root user
    let root = User {
        id: 0,
        name: "root".to_string(),
        permissions: vec![Permission::Admin],
    };

    let mut table = USER_TABLE.lock().unwrap();
    table.insert(0, root);

    Ok(())
}

fn check_permission_inner(user_id: UserId, permission: Permission) -> bool {
    let table = USER_TABLE.lock().unwrap();
    if let Some(user) = table.get(&user_id) {
        user.permissions.contains(&Permission::Admin) || user.permissions.contains(&permission)
    } else {
        false
    }
}

/// Sign and register an operation in the policy ledger.
pub fn enforce_operation(record: OperationRecord) -> Result<SignedOperation, PolicyError> {
    let mut enforcer = POLICY_ENFORCER
        .lock()
        .map_err(|_| PolicyError::SigningFailed("policy mutex poisoned".to_string()))?;
    enforcer.sign_and_register(record)
}

/// Verify a signed operation using the policy enforcement secret.
pub fn verify_signed_operation(operation: &SignedOperation) -> bool {
    let enforcer = POLICY_ENFORCER.lock();
    if let Ok(enforcer) = enforcer {
        enforcer.verify(operation)
    } else {
        false
    }
}

/// Retrieve a snapshot of recent operations from the in-memory audit trail.
///
/// **Important**: This returns only the most recent operations kept in memory
/// (up to 1000 records). For complete historical audit trails across the entire
/// system lifetime, retrieve signed operations from persistent ledger storage
/// (e.g., blockchain, append-only database, or external audit systems).
pub fn audit_trail() -> Vec<SignedOperation> {
    let enforcer = POLICY_ENFORCER.lock();
    if let Ok(enforcer) = enforcer {
        enforcer.audit_trail()
    } else {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_enforcer_signs_and_verifies() {
        init().unwrap();
        let record = OperationRecord::new(OperationKind::DocumentUpdate, "tester", "docs/test")
            .with_context(Some("/tmp/source".into()), Some("docs/test.md".into()))
            .with_metadata(json!({ "change": "unit-test" }));
        let signed = enforce_operation(record).expect("operation should sign");
        assert!(verify_signed_operation(&signed));
        assert!(signed.signature.len() > 10);
    }

    #[test]
    fn test_policy_enforcer_bounded_memory() {
        init().unwrap();
        
        // Clear any existing records by creating a fresh enforcer
        let mut enforcer = PolicyEnforcer::new("test-secret".to_string());
        
        // Add more records than MAX_IN_MEMORY_RECORDS
        let test_count = MAX_IN_MEMORY_RECORDS + 100;
        for i in 0..test_count {
            let record = OperationRecord::new(
                OperationKind::FileMove,
                format!("user-{}", i),
                format!("scope-{}", i),
            );
            enforcer.sign_and_register(record).expect("should sign");
        }
        
        // Verify that only MAX_IN_MEMORY_RECORDS are kept
        assert_eq!(
            enforcer.records.len(),
            MAX_IN_MEMORY_RECORDS,
            "Records should be bounded to MAX_IN_MEMORY_RECORDS"
        );
        
        // Verify that the oldest records were evicted (first 100 should be gone)
        // and the most recent records are kept
        let trail = enforcer.audit_trail();
        assert_eq!(trail.len(), MAX_IN_MEMORY_RECORDS);
        
        // The first record in memory should be from iteration 100 (0-99 evicted)
        assert!(
            trail[0].record.actor.contains("100"),
            "First record should be from iteration 100 after eviction"
        );
        
        // The last record should be from the last iteration
        assert!(
            trail.last().unwrap().record.actor.contains(&format!("{}", test_count - 1)),
            "Last record should be from the final iteration"
        );
    }
}

fn register_user_inner(user: User) {
    let mut table = USER_TABLE.lock().unwrap();
    table.insert(user.id, user);
}

/// Kernel-managed security capability.
#[derive(Clone, Default)]
pub struct SecurityService;

impl SecurityService {
    /// Register or update a user.
    pub fn register_user(&self, user: User) {
        register_user_inner(user);
    }

    /// Validate a permission check.
    pub fn check_permission(&self, user_id: UserId, permission: Permission) -> bool {
        check_permission_inner(user_id, permission)
    }
}

/// Check if user has permission.
pub fn check_permission(user_id: UserId, permission: Permission) -> bool {
    SecurityService::default().check_permission(user_id, permission)
}
