//! Security subsystem

use crate::time::current_timestamp_millis;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

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

    fn audit_trail(&self) -> Vec<SignedOperation> {
        self.records.clone()
    }
}

static USER_TABLE: OnceLock<Mutex<HashMap<UserId, User>>> = OnceLock::new();
static POLICY_ENFORCER: OnceLock<Mutex<PolicyEnforcer>> = OnceLock::new();
static OPERATION_COUNTER: AtomicU64 = AtomicU64::new(1);

fn user_table() -> &'static Mutex<HashMap<UserId, User>> {
    USER_TABLE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn policy_enforcer() -> &'static Mutex<PolicyEnforcer> {
    POLICY_ENFORCER.get_or_init(|| {
        Mutex::new(PolicyEnforcer::new(
            std::env::var("NOA_POLICY_SECRET")
                .unwrap_or_else(|_| "noa-ark-default-policy".to_string()),
        ))
    })
}

fn next_operation_id() -> String {
    let counter = OPERATION_COUNTER.fetch_add(1, Ordering::SeqCst);
    let timestamp = current_timestamp_millis();
    format!("op-{}-{}", timestamp, counter)
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

    let mut table = user_table().lock().unwrap();
    table.insert(0, root);

    Ok(())
}

fn check_permission_inner(user_id: UserId, permission: Permission) -> bool {
    let table = user_table().lock().unwrap();
    if let Some(user) = table.get(&user_id) {
        user.permissions.contains(&Permission::Admin) || user.permissions.contains(&permission)
    } else {
        false
    }
}

fn register_user_inner(user: User) {
    let mut table = user_table().lock().unwrap();
    table.insert(user.id, user);
}

/// Sign and register an operation in the policy ledger.
pub fn enforce_operation(record: OperationRecord) -> Result<SignedOperation, PolicyError> {
    let mut enforcer = policy_enforcer()
        .lock()
        .map_err(|_| PolicyError::SigningFailed("policy mutex poisoned".to_string()))?;
    enforcer.sign_and_register(record)
}

/// Verify a signed operation using the policy enforcement secret.
pub fn verify_signed_operation(operation: &SignedOperation) -> bool {
    let enforcer = policy_enforcer().lock();
    if let Ok(enforcer) = enforcer {
        enforcer.verify(operation)
    } else {
        false
    }
}

/// Retrieve a snapshot of the audit trail.
pub fn audit_trail() -> Vec<SignedOperation> {
    let enforcer = policy_enforcer().lock();
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
