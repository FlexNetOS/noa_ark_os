use crate::router::Protocol;
use noa_core::security::{
    self, OperationKind, OperationRecord, Permission, SignedOperation, UserId,
};
use parking_lot::Mutex;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Declarative trust tiers that link into the schema contract docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrustTier {
    Baseline,
    Elevated,
    Critical,
}

/// Route intent definition mirroring docs/architecture/gateway_symbol_schema.md fields.
#[derive(Debug, Clone)]
pub struct IntentEnvelope {
    pub name: &'static str,
    pub capabilities: &'static [&'static str],
    pub zones: &'static [&'static str],
    pub max_latency_ms: u32,
    pub requires_encryption: bool,
}

/// Trust metadata required for the route.
#[derive(Debug, Clone)]
pub struct TrustEnvelope {
    pub tier: TrustTier,
    pub min_score: f32,
    pub trusted_issuers: &'static [&'static str],
}

/// Compliance metadata mapping back to schema + evidence tags.
#[derive(Debug, Clone)]
pub struct ComplianceEnvelope {
    pub schema_ids: &'static [&'static str],
    pub required_tags: &'static [&'static str],
}

/// Declarative policy representation tying permissions to intent.
#[derive(Debug, Clone)]
pub struct GatewayPolicy {
    pub id: &'static str,
    pub permission: Permission,
    pub description: &'static str,
    pub intent: IntentEnvelope,
    pub trust: TrustEnvelope,
    pub compliance: ComplianceEnvelope,
}

/// Request-time context derived from intents.
#[derive(Debug, Clone, Serialize)]
pub struct EnforcementContext {
    pub intent_name: String,
    pub capabilities: Vec<String>,
    pub zones: Vec<String>,
    pub max_latency_ms: u32,
    pub encrypted: bool,
    pub trust_score: f32,
    pub attestation_issuers: Vec<String>,
    pub compliance_tags: Vec<String>,
    pub schema_ids: Vec<String>,
}

impl EnforcementContext {
    pub fn from_payload(protocol: &Protocol, payload: &Value) -> Self {
        let intent = payload.get("intent");
        let constraints = intent.and_then(|v| v.get("constraints"));

        let intent_name = intent
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}-intent", protocol_label(protocol)));

        let mut capabilities = read_string_array(intent.and_then(|v| v.get("capabilities")));
        if capabilities.is_empty() {
            capabilities.push(default_capability(protocol));
        }

        let mut zones = read_string_array(constraints.and_then(|v| v.get("zones")));
        if zones.is_empty() {
            zones.push("global".into());
        }

        let max_latency_ms = constraints
            .and_then(|v| v.get("max_latency_ms"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or(250);

        let encrypted = constraints
            .and_then(|v| v.get("encryption"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let trust_score = constraints
            .and_then(|v| v.get("observed_trust_score"))
            .and_then(|v| v.as_f64())
            .or_else(|| {
                constraints
                    .and_then(|v| v.get("min_trust_score"))
                    .and_then(|v| v.as_f64())
            })
            .unwrap_or(1.0) as f32;

        let mut attestation_issuers =
            read_string_array(constraints.and_then(|v| v.get("trusted_issuers")));
        if attestation_issuers.is_empty() {
            attestation_issuers = default_attesters(protocol);
        }

        let mut compliance_tags =
            read_string_array(constraints.and_then(|v| v.get("compliance_tags")));
        if compliance_tags.is_empty() {
            compliance_tags = default_compliance_tags(protocol);
        }

        let mut schema_ids = read_string_array(intent.and_then(|v| v.get("schema_ids")));
        if schema_ids.is_empty() {
            schema_ids = default_schema_ids(protocol);
        }

        Self {
            intent_name,
            capabilities,
            zones,
            max_latency_ms,
            encrypted,
            trust_score,
            attestation_issuers,
            compliance_tags,
            schema_ids,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub enum DecisionOutcome {
    Allowed,
    Denied,
}

#[derive(Debug, Error)]
pub enum PolicyError {
    #[error("user {user_id} is missing required permission {permission:?}")]
    MissingPermission {
        user_id: UserId,
        permission: Permission,
    },
    #[error("no gateway policy registered for permission {0:?}")]
    PolicyNotFound(Permission),
    #[error("intent violation: {0}")]
    IntentViolation(String),
    #[error("trust violation: {0}")]
    TrustViolation(String),
    #[error("compliance violation: {0}")]
    ComplianceViolation(String),
    #[error("audit failure: {0}")]
    AuditFailure(String),
}

/// Policy enforcer bridging to the core security module.
#[derive(Debug)]
pub struct PolicyEnforcer {
    policies: Vec<GatewayPolicy>,
    audit_sink: PolicyAuditSink,
}

impl PolicyEnforcer {
    pub fn new() -> Self {
        Self {
            policies: vec![
                GatewayPolicy {
                    id: "policy.read.intent",
                    permission: Permission::Read,
                    description: "Read access for catalog + telemetry routes",
                    intent: IntentEnvelope {
                        name: "read-stream",
                        capabilities: &["stream", "analytics", "graphql_route"],
                        zones: &["global", "edge"],
                        max_latency_ms: 250,
                        requires_encryption: true,
                    },
                    trust: TrustEnvelope {
                        tier: TrustTier::Elevated,
                        min_score: 0.7,
                        trusted_issuers: &["noa.attest"],
                    },
                    compliance: ComplianceEnvelope {
                        schema_ids: &["gateway.core.route"],
                        required_tags: &["pii_safe", "audited"],
                    },
                },
                GatewayPolicy {
                    id: "policy.write.intent",
                    permission: Permission::Write,
                    description: "Write access for mutation + workflow routes",
                    intent: IntentEnvelope {
                        name: "mutation-write",
                        capabilities: &["mutation", "workflow", "grpc_proxy"],
                        zones: &["global"],
                        max_latency_ms: 150,
                        requires_encryption: true,
                    },
                    trust: TrustEnvelope {
                        tier: TrustTier::Critical,
                        min_score: 0.85,
                        trusted_issuers: &["noa.attest", "noa.secure"],
                    },
                    compliance: ComplianceEnvelope {
                        schema_ids: &["gateway.core.route", "gateway.workflow.mutation"],
                        required_tags: &["pii_safe", "audited", "change_control"],
                    },
                },
                GatewayPolicy {
                    id: "policy.execute.intent",
                    permission: Permission::Execute,
                    description: "Execute access for autonomous workflow triggers",
                    intent: IntentEnvelope {
                        name: "workflow-trigger",
                        capabilities: &["execute", "automation", "websocket_multiplex"],
                        zones: &["global", "edge"],
                        max_latency_ms: 120,
                        requires_encryption: true,
                    },
                    trust: TrustEnvelope {
                        tier: TrustTier::Critical,
                        min_score: 0.9,
                        trusted_issuers: &["noa.attest", "noa.secure"],
                    },
                    compliance: ComplianceEnvelope {
                        schema_ids: &["gateway.core.route", "gateway.workflow.trigger"],
                        required_tags: &["pii_safe", "audited", "workflow_certified"],
                    },
                },
            ],
            audit_sink: PolicyAuditSink::default(),
        }
    }

    pub fn enforce(
        &self,
        user_id: UserId,
        permission: Permission,
        context: &EnforcementContext,
    ) -> Result<(), PolicyError> {
        let policy = self
            .policies
            .iter()
            .find(|policy| policy.permission == permission)
            .ok_or_else(|| PolicyError::PolicyNotFound(permission.clone()))?;

        if !security::check_permission(user_id, permission.clone()) {
            self.record_decision(
                policy,
                user_id,
                context,
                DecisionOutcome::Denied,
                "missing permission",
            )?;
            return Err(PolicyError::MissingPermission {
                user_id,
                permission,
            });
        }

        if let Err(reason) = self.assert_intent(policy, context) {
            self.record_decision(policy, user_id, context, DecisionOutcome::Denied, &reason)?;
            return Err(PolicyError::IntentViolation(reason));
        }

        if let Err(reason) = self.assert_trust(policy, context) {
            self.record_decision(policy, user_id, context, DecisionOutcome::Denied, &reason)?;
            return Err(PolicyError::TrustViolation(reason));
        }

        if let Err(reason) = self.assert_compliance(policy, context) {
            self.record_decision(policy, user_id, context, DecisionOutcome::Denied, &reason)?;
            return Err(PolicyError::ComplianceViolation(reason));
        }

        self.record_decision(
            policy,
            user_id,
            context,
            DecisionOutcome::Allowed,
            "policy satisfied",
        )?;
        Ok(())
    }

    pub fn policies(&self) -> &[GatewayPolicy] {
        &self.policies
    }

    fn assert_intent(
        &self,
        policy: &GatewayPolicy,
        context: &EnforcementContext,
    ) -> Result<(), String> {
        if context.max_latency_ms > policy.intent.max_latency_ms {
            return Err(format!(
                "requested latency {}ms exceeds policy budget {}ms",
                context.max_latency_ms, policy.intent.max_latency_ms
            ));
        }

        if policy.intent.requires_encryption && !context.encrypted {
            return Err("encrypted transport required".into());
        }

        if !subset_of_static(policy.intent.capabilities, &context.capabilities) {
            return Err("requested capabilities not allowed by policy".into());
        }

        if !subset_of_static(policy.intent.zones, &context.zones) {
            return Err("requested zones fall outside policy envelope".into());
        }

        Ok(())
    }

    fn assert_trust(
        &self,
        policy: &GatewayPolicy,
        context: &EnforcementContext,
    ) -> Result<(), String> {
        if context.trust_score < policy.trust.min_score {
            return Err(format!(
                "trust score {:.2} below tier requirement {:.2}",
                context.trust_score, policy.trust.min_score
            ));
        }

        if !intersection_with_static(policy.trust.trusted_issuers, &context.attestation_issuers) {
            return Err("no trusted attesters present".into());
        }

        Ok(())
    }

    fn assert_compliance(
        &self,
        policy: &GatewayPolicy,
        context: &EnforcementContext,
    ) -> Result<(), String> {
        if !superset_of_static(&context.schema_ids, policy.compliance.schema_ids) {
            return Err("schema ids missing from request context".into());
        }

        if !superset_of_static(&context.compliance_tags, policy.compliance.required_tags) {
            return Err("compliance tags missing from request context".into());
        }

        Ok(())
    }

    fn record_decision(
        &self,
        policy: &GatewayPolicy,
        user_id: UserId,
        context: &EnforcementContext,
        outcome: DecisionOutcome,
        reason: &str,
    ) -> Result<(), PolicyError> {
        let metadata = json!({
            "policy_id": policy.id,
            "policy_description": policy.description,
            "intent_name": context.intent_name,
            "capabilities": context.capabilities,
            "zones": context.zones,
            "max_latency_ms": context.max_latency_ms,
            "encrypted": context.encrypted,
            "trust_score": context.trust_score,
            "trust_tier": policy.trust.tier,
            "required_trust": policy.trust.min_score,
            "attesters": context.attestation_issuers,
            "compliance_tags": context.compliance_tags,
            "schema_ids": context.schema_ids,
            "decision": outcome,
            "reason": reason,
        });

        let record = OperationRecord::new(
            OperationKind::GatewayPolicy,
            format!("user-{}", user_id),
            policy.id,
        )
        .with_metadata(metadata);

        let signed = security::enforce_operation(record)
            .map_err(|err| PolicyError::AuditFailure(err.to_string()))?;
        self.audit_sink
            .record(&signed)
            .map_err(PolicyError::AuditFailure)
    }
}

impl Default for PolicyEnforcer {
    fn default() -> Self {
        Self::new()
    }
}

fn subset_of_static(allowed: &[&'static str], requested: &[String]) -> bool {
    let allowed_set: HashSet<String> = allowed
        .iter()
        .map(|item| item.to_ascii_lowercase())
        .collect();
    requested
        .iter()
        .all(|item| allowed_set.contains(&item.to_ascii_lowercase()))
}

fn superset_of_static(requested: &[String], required: &[&'static str]) -> bool {
    let requested_set: HashSet<String> = requested
        .iter()
        .map(|item| item.to_ascii_lowercase())
        .collect();
    required
        .iter()
        .all(|item| requested_set.contains(&item.to_ascii_lowercase()))
}

fn intersection_with_static(allowed: &[&'static str], requested: &[String]) -> bool {
    let allowed_set: HashSet<String> = allowed
        .iter()
        .map(|item| item.to_ascii_lowercase())
        .collect();
    requested
        .iter()
        .any(|item| allowed_set.contains(&item.to_ascii_lowercase()))
}

fn read_string_array(entry: Option<&Value>) -> Vec<String> {
    entry
        .and_then(|value| value.as_array().cloned())
        .map(|arr| {
            arr.into_iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn default_capability(protocol: &Protocol) -> String {
    match protocol {
        Protocol::GraphQl => "stream".into(),
        Protocol::Grpc => "mutation".into(),
        Protocol::WebSocket => "websocket_multiplex".into(),
    }
}

fn protocol_label(protocol: &Protocol) -> &'static str {
    match protocol {
        Protocol::GraphQl => "graphql",
        Protocol::Grpc => "grpc",
        Protocol::WebSocket => "websocket",
    }
}

fn default_attesters(protocol: &Protocol) -> Vec<String> {
    match protocol {
        Protocol::GraphQl => vec!["noa.attest".into()],
        Protocol::Grpc => vec!["noa.attest".into(), "noa.secure".into()],
        Protocol::WebSocket => vec!["noa.attest".into(), "noa.secure".into()],
    }
}

fn default_compliance_tags(protocol: &Protocol) -> Vec<String> {
    match protocol {
        Protocol::GraphQl => vec!["pii_safe".into(), "audited".into()],
        Protocol::Grpc => vec!["pii_safe".into(), "audited".into(), "change_control".into()],
        Protocol::WebSocket => vec![
            "pii_safe".into(),
            "audited".into(),
            "workflow_certified".into(),
        ],
    }
}

fn default_schema_ids(protocol: &Protocol) -> Vec<String> {
    match protocol {
        Protocol::GraphQl => vec!["gateway.core.route".into()],
        Protocol::Grpc => vec![
            "gateway.core.route".into(),
            "gateway.workflow.mutation".into(),
        ],
        Protocol::WebSocket => vec![
            "gateway.core.route".into(),
            "gateway.workflow.trigger".into(),
        ],
    }
}

#[derive(Debug)]
struct PolicyAuditSink {
    path: PathBuf,
    lock: Mutex<()>,
}

impl PolicyAuditSink {
    fn new<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        if let Some(parent) = path.as_ref().parent() {
            create_dir_all(parent)?;
        }
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            lock: Mutex::new(()),
        })
    }

    fn record(&self, signed: &SignedOperation) -> Result<(), String> {
        let _guard = self.lock.lock();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|err| err.to_string())?;
        let line = serde_json::to_string(signed).map_err(|err| err.to_string())?;
        file.write_all(line.as_bytes())
            .map_err(|err| err.to_string())?;
        file.write_all(b"\n").map_err(|err| err.to_string())?;
        Ok(())
    }
}

impl Default for PolicyAuditSink {
    fn default() -> Self {
        PolicyAuditSink::new(default_evidence_path()).unwrap_or_else(|err| {
            panic!(
                "Failed to initialise policy audit sink at {:?}: {}",
                default_evidence_path(),
                err
            )
        })
    }
}

fn default_evidence_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.pop(); // gateway
    path.pop(); // server
    path.push("docs");
    path.push("verification");
    path.push("gateway_policy_audit.jsonl");
    path
}
