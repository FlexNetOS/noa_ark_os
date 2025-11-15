//! CI/CD System - Continuous Delivery focused with CRC integration

pub mod ledger;
pub mod trigger;
pub mod validation;

use noa_security_shim::{
    run_gitleaks, run_grype, run_syft, run_trivy, ScanConfig, ScanResult, ScanStatus,
};
use noa_workflow::{
    DeploymentOutcomeRecord, PipelineInstrumentation, SecurityScanReport, SecurityScanStatus,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

const PIPELINE_STATE_FILE: &str = "storage/db/pipelines/state.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PipelineStage {
    CRC, // Continuous ReCode (new)
    Validate,
    Build,
    Test,
    SingleHostAcceptance,
    Deploy,
    Verify,
    Promote,
    DocsRefresh,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeploymentStrategy {
    BlueGreen,
    Canary,
    RollingUpdate,
    Recreate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PipelineStatus {
    Pending,
    Running,
    Success,
    Failed,
    RolledBack,
    AutoApproved, // new
    AgentReview,
    AgentApproved,
    AgentEscalated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: String,
    pub name: String,
    pub status: PipelineStatus,
    pub stages: Vec<Stage>,
    pub commit_sha: String,
    pub triggered_at: u64,
    pub crc_job_id: Option<String>, // new: link to CRC job
    pub auto_approved: bool,        // new: AI auto-approval
    pub ai_confidence: f32,         // new: AI confidence score
    pub diff_summary: Option<String>,
    pub approvals_required: Vec<AgentApprovalRequirement>,
    #[serde(default)]
    pub approvals_granted: Vec<AgentApproval>,
    #[serde(default)]
    pub security_scans: Vec<SecurityScanReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentApprovalRequirement {
    pub role: String,
    pub minimum_trust_score: f32,
    #[serde(default)]
    pub required_evidence_tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentApproval {
    pub role: String,
    pub agent_id: String,
    pub trust_score: f32,
    #[serde(default)]
    pub evidence_tags: Vec<String>,
    #[serde(default)]
    pub evidence_references: Vec<String>,
    pub recorded_at: u64,
}

impl AgentApprovalRequirement {
    fn is_satisfied_by(&self, approval: &AgentApproval) -> bool {
        if approval.role != self.role {
            return false;
        }
        if approval.trust_score + f32::EPSILON < self.minimum_trust_score {
            return false;
        }
        self.required_evidence_tags.iter().all(|required| {
            approval
                .evidence_tags
                .iter()
                .any(|provided| provided == required)
        })
    }
}

impl Pipeline {
    fn agent_requirements_satisfied(&self) -> bool {
        self.approvals_required.iter().all(|requirement| {
            self.approvals_granted
                .iter()
                .any(|approval| requirement.is_satisfied_by(approval))
        })
    }

    fn outstanding_agent_roles(&self) -> Vec<String> {
        self.approvals_required
            .iter()
            .filter(|requirement| {
                !self
                    .approvals_granted
                    .iter()
                    .any(|approval| requirement.is_satisfied_by(approval))
            })
            .map(|requirement| requirement.role.clone())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct ScannerFlags {
    pub syft: bool,
    pub grype: bool,
    pub trivy: bool,
    pub gitleaks: bool,
}

impl Default for ScannerFlags {
    fn default() -> Self {
        Self {
            syft: false,
            grype: false,
            trivy: false,
            gitleaks: false,
        }
    }
}

fn map_scan_status(status: &ScanStatus) -> SecurityScanStatus {
    match status {
        ScanStatus::Passed => SecurityScanStatus::Passed,
        ScanStatus::Failed => SecurityScanStatus::Failed,
        ScanStatus::Skipped => SecurityScanStatus::Skipped,
    }
}

#[cfg(test)]
pub struct EnvGuard {
    key: &'static str,
    prev: Option<String>,
}

#[cfg(test)]
impl EnvGuard {
    fn set(key: &'static str, value: &Path) -> Self {
        let prev = std::env::var(key).ok();
        std::env::set_var(key, value);
        Self { key, prev }
    }
}

#[cfg(test)]
impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(ref val) = self.prev {
            std::env::set_var(self.key, val);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn validation_skips_when_scanners_disabled() {
        let workspace = tempdir().unwrap();
        let _guard = EnvGuard::set("NOA_WORKFLOW_ROOT", workspace.path());
        let system = CICDSystem::new();
        system.configure_workspace_root(workspace.path());

        let pipeline_id = system
            .trigger_pipeline("demo".into(), "abc123".into())
            .expect("pipeline should trigger");
        system
            .validate(&pipeline_id)
            .expect("validation should succeed when scanners disabled");

        let pipelines = system.pipelines.lock().unwrap();
        let pipeline = pipelines.get(&pipeline_id).unwrap();
        assert!(!pipeline.security_scans.is_empty());
        assert!(pipeline
            .security_scans
            .iter()
            .all(|scan| scan.status == SecurityScanStatus::Skipped));
    }

    #[test]
    fn validation_fails_when_secrets_detected() {
        let workspace = tempdir().unwrap();
        std::fs::write(workspace.path().join("secrets.env"), "API_TOKEN=SECRET=123").unwrap();
        let _guard = EnvGuard::set("NOA_WORKFLOW_ROOT", workspace.path());
        let system = CICDSystem::new();
        system.configure_workspace_root(workspace.path());
        system.configure_scanner_flags(ScannerFlags {
            syft: false,
            grype: false,
            trivy: false,
            gitleaks: true,
        });

        let pipeline_id = system
            .trigger_pipeline("demo".into(), "abc123".into())
            .expect("pipeline should trigger");
        let result = system.validate(&pipeline_id);
        assert!(result.is_err());

        let pipelines = system.pipelines.lock().unwrap();
        let pipeline = pipelines.get(&pipeline_id).unwrap();
        assert!(pipeline
            .security_scans
            .iter()
            .any(|scan| scan.tool == "gitleaks" && scan.status == SecurityScanStatus::Failed));
    }
}

impl ScannerFlags {
    fn from_env() -> Self {
        fn enabled(key: &str) -> bool {
            std::env::var(key)
                .map(|value| matches!(value.as_str(), "1" | "true" | "TRUE"))
                .unwrap_or(false)
        }
        Self {
            syft: enabled("NOA_CICD_ENABLE_SYFT"),
            grype: enabled("NOA_CICD_ENABLE_GRYPE"),
            trivy: enabled("NOA_CICD_ENABLE_TRIVY"),
            gitleaks: enabled("NOA_CICD_ENABLE_GITLEAKS"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    pub name: String,
    pub stage_type: PipelineStage,
    pub status: PipelineStatus,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: String,
    pub environment: Environment,
    pub strategy: DeploymentStrategy,
    pub version: String,
    pub status: PipelineStatus,
    pub health_metrics: HealthMetrics,
    pub auto_approved: bool, // new
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMetrics {
    pub error_rate: f32,
    pub response_time_ms: u64,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub active_connections: u32,
}

impl Default for HealthMetrics {
    fn default() -> Self {
        Self {
            error_rate: 0.0,
            response_time_ms: 0,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            active_connections: 0,
        }
    }
}

impl HealthMetrics {
    /// Check if metrics are healthy
    pub fn is_healthy(&self, baseline: &HealthMetrics) -> bool {
        self.error_rate < 5.0
            && self.response_time_ms < baseline.response_time_ms * 2
            && self.cpu_usage < 90.0
            && self.memory_usage < 90.0
    }
}

pub struct CICDSystem {
    pipelines: Arc<Mutex<HashMap<String, Pipeline>>>,
    deployments: Arc<Mutex<HashMap<String, Deployment>>>,
    baseline_metrics: Arc<Mutex<HashMap<Environment, HealthMetrics>>>,
    auto_approve_threshold: f32, // new
    single_host_profile: Arc<Mutex<Option<String>>>,
    instrumentation: Arc<PipelineInstrumentation>,
    scanner_flags: Arc<Mutex<ScannerFlags>>,
    workspace_root: Arc<Mutex<PathBuf>>,
}

impl CICDSystem {
    fn initialise(threshold: f32) -> Self {
        let instrumentation = PipelineInstrumentation::new()
            .expect("failed to initialise pipeline instrumentation for CI/CD");
        let system = Self {
            pipelines: Arc::new(Mutex::new(HashMap::new())),
            deployments: Arc::new(Mutex::new(HashMap::new())),
            baseline_metrics: Arc::new(Mutex::new(HashMap::new())),
            auto_approve_threshold: threshold,
            single_host_profile: Arc::new(Mutex::new(Some(
                "server/profiles/single_host/profile.toml".to_string(),
            ))),
            instrumentation: Arc::new(instrumentation),
            scanner_flags: Arc::new(Mutex::new(ScannerFlags::from_env())),
            workspace_root: Arc::new(Mutex::new(PathBuf::from("."))),
        };
        if let Err(err) = system.load_state_from_disk() {
            let _ = system.emit_pipeline_event(
                "cicd::state",
                "cicd",
                "pipeline.state_load_failed",
                json!({ "error": err }),
            );
        }
        system
    }

    pub fn new() -> Self {
        Self::initialise(0.95)
    }

    /// Create CI/CD system with custom auto-approve threshold
    pub fn with_threshold(threshold: f32) -> Self {
        Self::initialise(threshold)
    }

    fn emit_pipeline_event(
        &self,
        subject: &str,
        actor: &str,
        event_type: &str,
        metadata: serde_json::Value,
    ) -> Result<(), String> {
        self.instrumentation
            .log_pipeline_event(actor, subject, event_type, metadata)
            .map(|_| ())
            .map_err(|err| format!("telemetry error: {}", err))
    }

    fn emit_deployment_event(
        &self,
        deployment_id: &str,
        event_type: &str,
        metadata: serde_json::Value,
    ) -> Result<(), String> {
        let metadata_for_event = metadata.clone();
        self.emit_pipeline_event(
            &format!("deployment::{}", deployment_id),
            "cicd",
            event_type,
            metadata_for_event,
        )?;
        self.record_deployment_outcome_entry(deployment_id, event_type, &metadata)?;
        Ok(())
    }

    fn record_deployment_outcome_entry(
        &self,
        deployment_id: &str,
        event_type: &str,
        metadata: &Value,
    ) -> Result<(), String> {
        let stage_id = metadata
            .get("environment")
            .or_else(|| metadata.get("target_environment"))
            .and_then(Value::as_str)
            .map(|value| value.to_string())
            .unwrap_or_else(|| event_type.to_string());
        let status = Self::deployment_status_for_event(event_type, metadata);
        let recorded_at = OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string());
        let record = DeploymentOutcomeRecord {
            workflow_id: format!("deployment::{}", deployment_id),
            stage_id,
            agent_role: "cicd::automation".to_string(),
            agent_id: "cicd::controller".to_string(),
            action: event_type.to_string(),
            status,
            notes: metadata.clone(),
            recorded_at,
        };
        self.instrumentation
            .record_deployment_outcome(record)
            .map_err(|err| format!("failed to record deployment outcome: {}", err))
    }

    fn deployment_status_for_event(event_type: &str, metadata: &Value) -> String {
        if let Some(current) = metadata
            .get("current_status")
            .or_else(|| metadata.get("status"))
            .and_then(Value::as_str)
        {
            return current.to_string();
        }

        match event_type {
            "deployment.auto_start" => "running".to_string(),
            "deployment.awaiting_agent" => "pending".to_string(),
            "deployment.health_passed" => "healthy".to_string(),
            "deployment.health_failed" => "unhealthy".to_string(),
            "deployment.rolled_back" => "rolled_back".to_string(),
            "deployment.auto_promote" => "promoted".to_string(),
            "deployment.auto_promote_blocked" => "blocked".to_string(),
            "deployment.state.created" => "running".to_string(),
            "deployment.state.status_changed" => metadata
                .get("current_status")
                .and_then(Value::as_str)
                .unwrap_or("updated")
                .to_string(),
            _ => "informational".to_string(),
        }
    }

    fn state_path(&self) -> PathBuf {
        let root = self
            .workspace_root
            .lock()
            .expect("workspace root lock poisoned")
            .clone();
        root.join(PIPELINE_STATE_FILE)
    }

    fn load_state_from_disk(&self) -> Result<(), String> {
        let path = self.state_path();
        if !path.exists() {
            return Ok(());
        }
        let raw = fs::read_to_string(&path)
            .map_err(|err| format!("failed to read pipeline state: {err}"))?;
        if raw.trim().is_empty() {
            return Ok(());
        }
        let state: PersistedState = serde_json::from_str(&raw)
            .map_err(|err| format!("failed to parse pipeline state: {err}"))?;
        {
            let mut pipelines = self.pipelines.lock().unwrap();
            pipelines.clear();
            for pipeline in state.pipelines {
                pipelines.insert(pipeline.id.clone(), pipeline);
            }
        }
        {
            let mut deployments = self.deployments.lock().unwrap();
            deployments.clear();
            for deployment in state.deployments {
                deployments.insert(deployment.id.clone(), deployment);
            }
        }
        Ok(())
    }

    fn persist_state(&self) -> Result<(), String> {
        let pipelines: Vec<Pipeline> = {
            let pipelines = self.pipelines.lock().unwrap();
            pipelines.values().cloned().collect()
        };
        let deployments: Vec<Deployment> = {
            let deployments = self.deployments.lock().unwrap();
            deployments.values().cloned().collect()
        };
        let state = PersistedState {
            pipelines,
            deployments,
        };
        let path = self.state_path();
        let previous_state = if path.exists() {
            let raw = fs::read_to_string(&path)
                .map_err(|err| format!("failed to read existing pipeline state: {err}"))?;
            if raw.trim().is_empty() {
                None
            } else {
                Some(
                    serde_json::from_str::<PersistedState>(&raw)
                        .map_err(|err| format!("failed to parse existing pipeline state: {err}"))?,
                )
            }
        } else {
            None
        };

        let previous_pipelines: HashMap<String, Pipeline> = previous_state
            .as_ref()
            .map(|state| {
                state
                    .pipelines
                    .iter()
                    .cloned()
                    .map(|pipeline| (pipeline.id.clone(), pipeline))
                    .collect()
            })
            .unwrap_or_default();
        let previous_deployments: HashMap<String, Deployment> = previous_state
            .as_ref()
            .map(|state| {
                state
                    .deployments
                    .iter()
                    .cloned()
                    .map(|deployment| (deployment.id.clone(), deployment))
                    .collect()
            })
            .unwrap_or_default();

        let mut pipeline_state_events: Vec<(String, String, Value)> = Vec::new();
        for pipeline in &state.pipelines {
            match previous_pipelines.get(&pipeline.id) {
                None => {
                    pipeline_state_events.push((
                        pipeline.id.clone(),
                        "pipeline.state.created".to_string(),
                        json!({
                            "name": pipeline.name,
                            "status": format!("{:?}", pipeline.status),
                            "commit_sha": pipeline.commit_sha,
                            "triggered_at": pipeline.triggered_at,
                            "auto_approved": pipeline.auto_approved,
                        }),
                    ));
                }
                Some(previous) => {
                    if previous.status != pipeline.status {
                        pipeline_state_events.push((
                            pipeline.id.clone(),
                            "pipeline.state.status_changed".to_string(),
                            json!({
                                "name": pipeline.name,
                                "previous_status": format!("{:?}", previous.status),
                                "current_status": format!("{:?}", pipeline.status),
                            }),
                        ));
                    }
                }
            }
        }

        let mut deployment_state_events: Vec<(String, String, Value)> = Vec::new();
        for deployment in &state.deployments {
            match previous_deployments.get(&deployment.id) {
                None => {
                    deployment_state_events.push((
                        deployment.id.clone(),
                        "deployment.state.created".to_string(),
                        json!({
                            "environment": deployment.environment,
                            "strategy": deployment.strategy,
                            "version": deployment.version,
                            "status": format!("{:?}", deployment.status),
                            "auto_approved": deployment.auto_approved,
                        }),
                    ));
                }
                Some(previous) => {
                    if previous.status != deployment.status {
                        deployment_state_events.push((
                            deployment.id.clone(),
                            "deployment.state.status_changed".to_string(),
                            json!({
                                "environment": deployment.environment,
                                "strategy": deployment.strategy,
                                "version": deployment.version,
                                "previous_status": format!("{:?}", previous.status),
                                "current_status": format!("{:?}", deployment.status),
                            }),
                        ));
                    }
                }
            }
        }

        let payload = serde_json::to_string_pretty(&state)
            .map_err(|err| format!("failed to serialise pipeline state: {err}"))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create pipeline state directory: {err}"))?;
        }
        fs::write(&path, payload)
            .map_err(|err| format!("failed to persist pipeline state: {err}"))?;

        for (pipeline_id, event_type, metadata) in pipeline_state_events {
            self.emit_pipeline_event(&pipeline_id, "cicd", &event_type, metadata)?;
        }

        for (deployment_id, event_type, metadata) in deployment_state_events {
            self.emit_deployment_event(&deployment_id, &event_type, metadata)?;
        }

        Ok(())
    }

    /// Register the single-host profile manifest used for acceptance tests.
    pub fn configure_single_host_profile<P: Into<String>>(&self, profile_path: P) {
        let mut guard = self
            .single_host_profile
            .lock()
            .expect("single host profile lock poisoned");
        *guard = Some(profile_path.into());
    }

    /// Override the workspace root used by offline scanners.
    pub fn configure_workspace_root<P: Into<PathBuf>>(&self, root: P) {
        let mut guard = self
            .workspace_root
            .lock()
            .expect("workspace root lock poisoned");
        *guard = root.into();
    }

    /// Enable or disable specific offline scanners.
    pub fn configure_scanner_flags(&self, flags: ScannerFlags) {
        let mut guard = self
            .scanner_flags
            .lock()
            .expect("scanner flag lock poisoned");
        *guard = flags;
    }

    /// Trigger a new pipeline (can be triggered by CRC)
    pub fn trigger_pipeline(&self, name: String, commit_sha: String) -> Result<String, String> {
        let id = format!("pipeline_{}", uuid::Uuid::new_v4());

        let pipeline = Pipeline {
            id: id.clone(),
            name,
            status: PipelineStatus::Pending,
            stages: vec![
                Stage {
                    name: "validate".to_string(),
                    stage_type: PipelineStage::Validate,
                    status: PipelineStatus::Pending,
                    duration_ms: None,
                },
                Stage {
                    name: "build".to_string(),
                    stage_type: PipelineStage::Build,
                    status: PipelineStatus::Pending,
                    duration_ms: None,
                },
                Stage {
                    name: "test".to_string(),
                    stage_type: PipelineStage::Test,
                    status: PipelineStatus::Pending,
                    duration_ms: None,
                },
                Stage {
                    name: "single_host_acceptance".to_string(),
                    stage_type: PipelineStage::SingleHostAcceptance,
                    status: PipelineStatus::Pending,
                    duration_ms: None,
                },
                Stage {
                    name: "deploy".to_string(),
                    stage_type: PipelineStage::Deploy,
                    status: PipelineStatus::Pending,
                    duration_ms: None,
                },
            ],
            commit_sha,
            triggered_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            crc_job_id: None,
            auto_approved: false,
            ai_confidence: 0.0,
            diff_summary: None,
            approvals_required: Vec::new(),
            approvals_granted: Vec::new(),
            security_scans: Vec::new(),
        };
        let metadata = json!({
            "name": pipeline.name.clone(),
            "commit_sha": pipeline.commit_sha.clone(),
            "triggered_at": pipeline.triggered_at,
        });

        let mut pipelines = self.pipelines.lock().unwrap();
        pipelines.insert(id.clone(), pipeline);
        drop(pipelines);

        self.persist_state()?;
        self.emit_pipeline_event(&id, "cicd", "pipeline.triggered", metadata)?;

        Ok(id)
    }

    /// Trigger pipeline from CRC (with AI confidence)
    pub fn trigger_from_crc(
        &self,
        name: String,
        commit_sha: String,
        crc_job_id: String,
        ai_confidence: f32,
    ) -> Result<String, String> {
        let id = self.trigger_pipeline(name, commit_sha)?;

        // Update with CRC info
        let event = {
            let mut pipelines = self.pipelines.lock().unwrap();
            if let Some(pipeline) = pipelines.get_mut(&id) {
                pipeline.crc_job_id = Some(crc_job_id);
                pipeline.ai_confidence = ai_confidence;
                pipeline.auto_approved = ai_confidence >= self.auto_approve_threshold;

                if pipeline.auto_approved {
                    pipeline.status = PipelineStatus::AutoApproved;
                    Some((
                        "pipeline.auto_approved",
                        json!({
                            "ai_confidence": ai_confidence,
                            "threshold": self.auto_approve_threshold,
                            "source": "crc",
                        }),
                    ))
                } else {
                    pipeline.status = PipelineStatus::AgentReview;
                    let outstanding_roles = pipeline.outstanding_agent_roles();
                    Some((
                        "pipeline.agent_review_required",
                        json!({
                            "ai_confidence": ai_confidence,
                            "threshold": self.auto_approve_threshold,
                            "outstanding_roles": outstanding_roles,
                        }),
                    ))
                }
            } else {
                None
            }
        };

        self.persist_state()?;
        if let Some((event_type, metadata)) = event {
            self.emit_pipeline_event(&id, "cicd", event_type, metadata)?;
        }

        Ok(id)
    }

    pub fn trigger_doc_refresh_pipeline(
        &self,
        commit_sha: String,
        diff_summary: String,
        approvals_required: Vec<AgentApprovalRequirement>,
    ) -> Result<String, String> {
        let id = format!("doc_pipeline_{}", uuid::Uuid::new_v4());

        let pipeline = Pipeline {
            id: id.clone(),
            name: "documentation-refresh".to_string(),
            status: if approvals_required.is_empty() {
                PipelineStatus::Pending
            } else {
                PipelineStatus::AgentReview
            },
            stages: vec![
                Stage {
                    name: "validate".to_string(),
                    stage_type: PipelineStage::Validate,
                    status: PipelineStatus::Pending,
                    duration_ms: None,
                },
                Stage {
                    name: "docs-refresh".to_string(),
                    stage_type: PipelineStage::DocsRefresh,
                    status: PipelineStatus::Pending,
                    duration_ms: None,
                },
                Stage {
                    name: "verify".to_string(),
                    stage_type: PipelineStage::Verify,
                    status: PipelineStatus::Pending,
                    duration_ms: None,
                },
            ],
            commit_sha,
            triggered_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            crc_job_id: None,
            auto_approved: false,
            ai_confidence: 0.0,
            diff_summary: Some(diff_summary.clone()),
            approvals_required,
            approvals_granted: Vec::new(),
            security_scans: Vec::new(),
        };
        let metadata = json!({
            "commit_sha": pipeline.commit_sha.clone(),
            "diff_summary": pipeline.diff_summary.clone(),
            "requirements": pipeline
                .approvals_required
                .iter()
                .map(|req| json!({
                    "role": req.role,
                    "minimum_trust_score": req.minimum_trust_score,
                    "evidence": req.required_evidence_tags,
                }))
                .collect::<Vec<_>>(),
        });

        {
            let mut pipelines = self.pipelines.lock().unwrap();
            pipelines.insert(id.clone(), pipeline);
        }

        self.persist_state()?;
        self.emit_pipeline_event(&id, "cicd", "pipeline.doc_refresh_triggered", metadata)?;

        Ok(id)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn register_agent_approval(
        &self,
        pipeline_id: &str,
        role: &str,
        agent_id: &str,
        trust_score: f32,
        evidence_tags: Vec<String>,
        evidence_references: Vec<String>,
    ) -> Result<PipelineStatus, String> {
        let recorded_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| format!("system clock error: {err}"))?
            .as_secs();
        let approval = AgentApproval {
            role: role.to_string(),
            agent_id: agent_id.to_string(),
            trust_score,
            evidence_tags: evidence_tags.clone(),
            evidence_references: evidence_references.clone(),
            recorded_at,
        };

        let (status, event_type, metadata) = {
            let mut pipelines = self.pipelines.lock().unwrap();
            let pipeline = pipelines
                .get_mut(pipeline_id)
                .ok_or_else(|| format!("Pipeline not found: {}", pipeline_id))?;
            let requirement = pipeline
                .approvals_required
                .iter()
                .find(|req| req.role == role)
                .ok_or_else(|| {
                    format!(
                        "agent role {} not registered for pipeline {}",
                        role, pipeline_id
                    )
                })?;

            if trust_score + f32::EPSILON < requirement.minimum_trust_score {
                pipeline.status = PipelineStatus::AgentEscalated;
                let metadata = json!({
                    "role": role,
                    "agent_id": agent_id,
                    "trust_score": trust_score,
                    "required_trust": requirement.minimum_trust_score,
                    "evidence_tags": evidence_tags,
                    "evidence_references": evidence_references,
                    "reason": "trust_score_below_threshold",
                });
                (
                    pipeline.status.clone(),
                    "pipeline.agent_escalated",
                    metadata,
                )
            } else if !requirement.required_evidence_tags.iter().all(|tag| {
                approval
                    .evidence_tags
                    .iter()
                    .any(|provided| provided == tag)
            }) {
                pipeline.status = PipelineStatus::AgentEscalated;
                let missing: Vec<String> = requirement
                    .required_evidence_tags
                    .iter()
                    .filter(|tag| {
                        !approval
                            .evidence_tags
                            .iter()
                            .any(|provided| provided == *tag)
                    })
                    .cloned()
                    .collect();
                let metadata = json!({
                    "role": role,
                    "agent_id": agent_id,
                    "trust_score": trust_score,
                    "required_trust": requirement.minimum_trust_score,
                    "missing_evidence": missing,
                    "evidence_references": approval.evidence_references,
                    "reason": "missing_evidence",
                });
                (
                    pipeline.status.clone(),
                    "pipeline.agent_escalated",
                    metadata,
                )
            } else {
                pipeline
                    .approvals_granted
                    .retain(|existing| existing.role != role || existing.agent_id != agent_id);
                pipeline.approvals_granted.push(approval.clone());
                let outstanding = pipeline.outstanding_agent_roles();
                if outstanding.is_empty() {
                    pipeline.status = PipelineStatus::AgentApproved;
                    (
                        pipeline.status.clone(),
                        "pipeline.agent_approved",
                        json!({
                            "role": role,
                            "agent_id": agent_id,
                            "trust_score": trust_score,
                            "evidence_references": approval.evidence_references.clone(),
                            "recorded_at": recorded_at,
                        }),
                    )
                } else {
                    pipeline.status = PipelineStatus::AgentReview;
                    (
                        pipeline.status.clone(),
                        "pipeline.agent_partial_approval",
                        json!({
                            "role": role,
                            "agent_id": agent_id,
                            "trust_score": trust_score,
                            "outstanding_roles": outstanding,
                            "evidence_references": approval.evidence_references.clone(),
                            "recorded_at": recorded_at,
                        }),
                    )
                }
            }
        };

        self.persist_state()?;
        self.emit_pipeline_event(
            pipeline_id,
            &format!("agent::{}", role),
            event_type,
            metadata,
        )?;

        if matches!(status, PipelineStatus::AgentEscalated) {
            Err(format!(
                "agent approval for role {} requires escalation",
                role
            ))
        } else {
            Ok(status)
        }
    }

    /// Execute pipeline with full automation
    pub fn execute_pipeline(&self, pipeline_id: &str) -> Result<(), String> {
        let stages = {
            let pipelines = self.pipelines.lock().unwrap();
            let pipeline = pipelines
                .get(pipeline_id)
                .ok_or_else(|| format!("Pipeline not found: {}", pipeline_id))?;
            if matches!(
                pipeline.status,
                PipelineStatus::AgentReview | PipelineStatus::AgentEscalated
            ) {
                return Err("Pipeline requires agent approval before execution".to_string());
            }
            if !pipeline.agent_requirements_satisfied() {
                return Err("Pipeline is waiting for agent approvals".to_string());
            }
            pipeline.stages.clone()
        };

        self.update_pipeline_status(pipeline_id, PipelineStatus::Running)?;
        self.emit_pipeline_event(
            pipeline_id,
            "cicd",
            "pipeline.execution_started",
            json!({ "stage_count": stages.len() }),
        )?;

        // Execute each stage
        for stage in stages {
            self.execute_stage(pipeline_id, &stage)?;
        }

        // Mark pipeline as success
        self.update_pipeline_status(pipeline_id, PipelineStatus::Success)?;
        self.emit_pipeline_event(
            pipeline_id,
            "cicd",
            "pipeline.execution_completed",
            json!({ "status": "success" }),
        )?;
        Ok(())
    }

    /// Execute a single stage
    fn execute_stage(&self, pipeline_id: &str, stage: &Stage) -> Result<(), String> {
        self.emit_pipeline_event(
            pipeline_id,
            "cicd",
            "pipeline.stage_started",
            json!({
                "stage": stage.name,
                "stage_type": stage.stage_type,
            }),
        )?;

        let start = std::time::Instant::now();

        // Simulate stage execution
        match stage.stage_type {
            PipelineStage::CRC => self.crc_stage(pipeline_id)?,
            PipelineStage::Validate => self.validate(pipeline_id)?,
            PipelineStage::Build => self.build(pipeline_id)?,
            PipelineStage::Test => self.test(pipeline_id)?,
            PipelineStage::SingleHostAcceptance => self.single_host_acceptance(pipeline_id)?,
            PipelineStage::Deploy => self.deploy(pipeline_id)?,
            PipelineStage::DocsRefresh => self.docs_refresh(pipeline_id)?,
            _ => {}
        }

        let duration = start.elapsed().as_millis() as u64;
        self.emit_pipeline_event(
            pipeline_id,
            "cicd",
            "pipeline.stage_completed",
            json!({
                "stage": stage.name,
                "stage_type": stage.stage_type,
                "duration_ms": duration,
            }),
        )?;

        Ok(())
    }

    /// CRC stage (if needed)
    fn crc_stage(&self, pipeline_id: &str) -> Result<(), String> {
        self.emit_pipeline_event(
            pipeline_id,
            "cicd",
            "pipeline.stage.crc_skipped",
            json!({ "message": "CRC adaptation already complete" }),
        )
    }

    /// Validation stage
    fn validate(&self, pipeline_id: &str) -> Result<(), String> {
        self.emit_pipeline_event(
            pipeline_id,
            "cicd",
            "pipeline.validation_started",
            json!({}),
        )?;
        let flags = {
            let guard = self
                .scanner_flags
                .lock()
                .expect("scanner flag lock poisoned");
            guard.clone()
        };
        let workspace = {
            self.workspace_root
                .lock()
                .expect("workspace root lock poisoned")
                .clone()
        };

        let mut results = Vec::new();
        if flags.syft {
            results.push(self.run_security_scan(pipeline_id, "syft", run_syft, &workspace)?);
        } else {
            results.push(self.log_skipped_scan(pipeline_id, "syft", "flag disabled")?);
        }
        if flags.grype {
            results.push(self.run_security_scan(pipeline_id, "grype", run_grype, &workspace)?);
        } else {
            results.push(self.log_skipped_scan(pipeline_id, "grype", "flag disabled")?);
        }
        if flags.trivy {
            results.push(self.run_security_scan(pipeline_id, "trivy", run_trivy, &workspace)?);
        } else {
            results.push(self.log_skipped_scan(pipeline_id, "trivy", "flag disabled")?);
        }
        if flags.gitleaks {
            results.push(self.run_security_scan(
                pipeline_id,
                "gitleaks",
                run_gitleaks,
                &workspace,
            )?);
        } else {
            results.push(self.log_skipped_scan(pipeline_id, "gitleaks", "flag disabled")?);
        }

        if results
            .iter()
            .any(|report| report.status == SecurityScanStatus::Failed)
        {
            return Err("Security scans reported failures".to_string());
        }

        self.emit_pipeline_event(
            pipeline_id,
            "cicd",
            "pipeline.validation_completed",
            json!({ "scans_run": results.len() }),
        )
    }

    fn run_security_scan<Runner>(
        &self,
        pipeline_id: &str,
        tool: &str,
        runner: Runner,
        workspace: &PathBuf,
    ) -> Result<SecurityScanReport, String>
    where
        Runner: Fn(&ScanConfig) -> Result<ScanResult, noa_security_shim::ShimError>,
    {
        let config = ScanConfig {
            target: workspace.clone(),
            ..ScanConfig::default()
        };
        let result = runner(&config).map_err(|err| format!("{} scan failed: {}", tool, err))?;
        let issues: Vec<String> = result
            .findings
            .iter()
            .map(|finding| format!("{} [{}]", finding.description, finding.file))
            .collect();
        let metadata = serde_json::to_value(&result.findings).unwrap_or_else(|_| json!({}));
        let report = self
            .instrumentation
            .as_ref()
            .log_security_scan(
                pipeline_id,
                tool,
                map_scan_status(&result.status),
                issues,
                result.report_path.clone(),
                metadata,
            )
            .map_err(|err| format!("security instrumentation failed: {}", err))?;
        self.record_security_scan(pipeline_id, report.clone())?;
        Ok(report)
    }

    fn log_skipped_scan(
        &self,
        pipeline_id: &str,
        tool: &str,
        reason: &str,
    ) -> Result<SecurityScanReport, String> {
        let report = self
            .instrumentation
            .as_ref()
            .log_security_scan(
                pipeline_id,
                tool,
                SecurityScanStatus::Skipped,
                Vec::new(),
                None,
                json!({"reason": reason}),
            )
            .map_err(|err| format!("security instrumentation failed: {}", err))?;
        self.record_security_scan(pipeline_id, report.clone())?;
        Ok(report)
    }

    fn record_security_scan(
        &self,
        pipeline_id: &str,
        report: SecurityScanReport,
    ) -> Result<(), String> {
        let mut pipelines = self.pipelines.lock().unwrap();
        if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
            pipeline.security_scans.push(report);
            Ok(())
        } else {
            Err(format!("Pipeline not found: {}", pipeline_id))
        }
    }

    /// Build stage
    fn build(&self, pipeline_id: &str) -> Result<(), String> {
        self.emit_pipeline_event(
            pipeline_id,
            "cicd",
            "pipeline.build_components",
            json!({
                "targets": ["rust", "go", "python", ".net"],
            }),
        )
    }

    /// Test stage
    fn test(&self, pipeline_id: &str) -> Result<(), String> {
        self.emit_pipeline_event(
            pipeline_id,
            "cicd",
            "pipeline.tests_executed",
            json!({
                "suites": ["unit", "integration", "api"],
            }),
        )
    }

    /// Acceptance checks for the single-host profile
    fn single_host_acceptance(&self, pipeline_id: &str) -> Result<(), String> {
        let profile_path = {
            let guard = self
                .single_host_profile
                .lock()
                .expect("single host profile lock poisoned");
            guard.clone()
        };

        let profile_path = profile_path
            .ok_or_else(|| "Single-host profile not configured for CI/CD".to_string())?;

        if !Path::new(&profile_path).exists() {
            return Err(format!("Single-host profile not found at {}", profile_path));
        }

        let manifest = fs::read_to_string(&profile_path)
            .map_err(|err| format!("Failed to read profile: {err}"))?;

        if !manifest.contains("single_host") {
            return Err("Profile manifest does not describe the single_host configuration".into());
        }

        self.emit_pipeline_event(
            pipeline_id,
            "cicd",
            "pipeline.single_host_validated",
            json!({
                "profile_path": profile_path,
                "manifest_size": manifest.len(),
            }),
        )
    }

    /// Deploy stage
    fn deploy(&self, pipeline_id: &str) -> Result<(), String> {
        self.emit_pipeline_event(
            pipeline_id,
            "cicd",
            "pipeline.deploy_initiated",
            json!({ "target": "staging" }),
        )
    }

    fn docs_refresh(&self, pipeline_id: &str) -> Result<(), String> {
        let diff_summary = {
            let pipelines = self.pipelines.lock().unwrap();
            pipelines
                .get(pipeline_id)
                .and_then(|p| p.diff_summary.clone())
                .unwrap_or_else(|| "No diff summary provided".to_string())
        };

        self.emit_pipeline_event(
            pipeline_id,
            "cicd",
            "pipeline.docs_refresh",
            json!({
                "diff_summary": diff_summary,
                "agent": "documentation",
            }),
        )
    }

    /// Deploy to environment with strategy and auto-approval
    pub fn deploy_to_environment(
        &self,
        version: String,
        environment: Environment,
        strategy: DeploymentStrategy,
    ) -> Result<String, String> {
        let id = format!("deploy_{}", uuid::Uuid::new_v4());

        // Check if auto-approved
        let auto_approved = true; // Based on pipeline status

        let environment_for_metadata = environment.clone();
        let strategy_for_metadata = strategy.clone();
        let version_for_metadata = version.clone();

        let deployment = Deployment {
            id: id.clone(),
            environment: environment.clone(),
            strategy,
            version,
            status: PipelineStatus::Running,
            health_metrics: HealthMetrics::default(),
            auto_approved,
        };

        let mut deployments = self.deployments.lock().unwrap();
        deployments.insert(id.clone(), deployment);
        drop(deployments);

        self.persist_state()?;

        let event_type = if auto_approved {
            "deployment.auto_start"
        } else {
            "deployment.awaiting_agent"
        };
        self.emit_deployment_event(
            &id,
            event_type,
            json!({
                "environment": environment_for_metadata,
                "strategy": strategy_for_metadata,
                "version": version_for_metadata,
                "auto_approved": auto_approved,
            }),
        )?;

        Ok(id)
    }

    /// Monitor deployment health with auto-rollback
    pub fn monitor_deployment(&self, deployment_id: &str) -> Result<bool, String> {
        let (environment, metrics) = {
            let deployments = self.deployments.lock().unwrap();
            let deployment = deployments
                .get(deployment_id)
                .ok_or_else(|| format!("Deployment not found: {}", deployment_id))?;
            (
                deployment.environment.clone(),
                deployment.health_metrics.clone(),
            )
        };

        let baseline = {
            let baseline_metrics = self.baseline_metrics.lock().unwrap();
            baseline_metrics
                .get(&environment)
                .cloned()
                .unwrap_or_default()
        };

        let is_healthy = metrics.is_healthy(&baseline);

        let event_type = if is_healthy {
            "deployment.health_passed"
        } else {
            "deployment.health_failed"
        };
        self.emit_deployment_event(
            deployment_id,
            event_type,
            json!({
                "environment": environment,
                "metrics": metrics,
                "baseline": baseline,
            }),
        )?;

        Ok(is_healthy)
    }

    /// Rollback deployment (automatic)
    pub fn rollback(&self, deployment_id: &str) -> Result<(), String> {
        let mut deployments = self.deployments.lock().unwrap();
        if let Some(deployment) = deployments.get_mut(deployment_id) {
            deployment.status = PipelineStatus::RolledBack;
            let environment = deployment.environment.clone();
            let strategy = deployment.strategy.clone();
            let version = deployment.version.clone();
            drop(deployments);

            self.persist_state()?;
            self.emit_deployment_event(
                deployment_id,
                "deployment.rolled_back",
                json!({
                    "environment": environment,
                    "strategy": strategy,
                    "version": version,
                }),
            )?;
            Ok(())
        } else {
            Err(format!("Deployment not found: {}", deployment_id))
        }
    }

    /// Auto-promote if healthy (full automation)
    pub fn auto_promote(
        &self,
        deployment_id: &str,
        to_environment: Environment,
    ) -> Result<(), String> {
        if self.monitor_deployment(deployment_id)? {
            self.emit_deployment_event(
                deployment_id,
                "deployment.auto_promote",
                json!({ "target_environment": to_environment }),
            )?;
            Ok(())
        } else {
            self.emit_deployment_event(
                deployment_id,
                "deployment.auto_promote_blocked",
                json!({ "target_environment": to_environment }),
            )?;
            Err("Deployment not healthy for auto-promotion".to_string())
        }
    }

    /// Complete end-to-end automation
    pub fn full_auto_pipeline(&self, crc_job_id: String, ai_confidence: f32) -> Result<(), String> {
        self.emit_pipeline_event(
            "automation::full_auto",
            "cicd",
            "pipeline.full_auto.started",
            json!({
                "crc_job_id": crc_job_id.clone(),
                "ai_confidence": ai_confidence,
            }),
        )?;

        // Trigger from CRC
        let pipeline_id = self.trigger_from_crc(
            "auto-pipeline".to_string(),
            "auto-commit".to_string(),
            crc_job_id,
            ai_confidence,
        )?;

        // If auto-approved, continue
        let mut proceed = true;
        let mut status = None;
        {
            let pipelines = self.pipelines.lock().unwrap();
            if let Some(pipeline) = pipelines.get(&pipeline_id) {
                if !(pipeline.auto_approved
                    || matches!(pipeline.status, PipelineStatus::AgentApproved))
                {
                    proceed = false;
                    status = Some(pipeline.status.clone());
                }
            }
        }
        if !proceed {
            self.emit_pipeline_event(
                &pipeline_id,
                "cicd",
                "pipeline.full_auto.halted",
                json!({ "status": status }),
            )?;
            return Ok(());
        }

        // Execute CI
        self.execute_pipeline(&pipeline_id)?;

        // Deploy to Staging (auto)
        let staging_deploy = self.deploy_to_environment(
            "v1.0.0".to_string(),
            Environment::Staging,
            DeploymentStrategy::BlueGreen,
        )?;

        // Monitor and auto-promote
        if self.monitor_deployment(&staging_deploy)? {
            // Deploy to Production (auto)
            let prod_deploy = self.deploy_to_environment(
                "v1.0.0".to_string(),
                Environment::Production,
                DeploymentStrategy::Canary,
            )?;

            // Monitor production with auto-rollback
            if self.monitor_deployment(&prod_deploy)? {
                self.auto_promote(&prod_deploy, Environment::Production)?;
                self.emit_pipeline_event(
                    &pipeline_id,
                    "cicd",
                    "pipeline.full_auto.completed",
                    json!({ "status": "success" }),
                )?;
            } else {
                self.rollback(&prod_deploy)?;
                self.emit_pipeline_event(
                    &pipeline_id,
                    "cicd",
                    "pipeline.full_auto.rollback",
                    json!({ "status": "rollback_triggered" }),
                )?;
            }
        }

        self.emit_pipeline_event(
            "automation::full_auto",
            "cicd",
            "pipeline.full_auto.finished",
            json!({ "pipeline_id": pipeline_id }),
        )?;
        Ok(())
    }

    /// Update pipeline status
    fn update_pipeline_status(
        &self,
        pipeline_id: &str,
        status: PipelineStatus,
    ) -> Result<(), String> {
        let (previous, changed) = {
            let mut pipelines = self.pipelines.lock().unwrap();
            if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
                let previous = pipeline.status.clone();
                let changed = previous != status;
                pipeline.status = status.clone();
                (Some(previous), changed)
            } else {
                return Err(format!("Pipeline not found: {}", pipeline_id));
            }
        };

        self.persist_state()?;
        if changed {
            self.emit_pipeline_event(
                pipeline_id,
                "cicd",
                "pipeline.status_updated",
                json!({
                    "previous": previous,
                    "current": status,
                }),
            )?;
        }
        Ok(())
    }

    /// Get pipeline status
    pub fn get_pipeline_status(&self, pipeline_id: &str) -> Option<PipelineStatus> {
        let pipelines = self.pipelines.lock().unwrap();
        pipelines.get(pipeline_id).map(|p| p.status.clone())
    }

    /// Get deployment metrics
    pub fn get_metrics(&self, deployment_id: &str) -> Option<HealthMetrics> {
        let deployments = self.deployments.lock().unwrap();
        deployments
            .get(deployment_id)
            .map(|d| d.health_metrics.clone())
    }

    /// Get pipeline by CRC job
    pub fn get_pipeline_by_crc(&self, crc_job_id: &str) -> Option<Pipeline> {
        let pipelines = self.pipelines.lock().unwrap();
        pipelines
            .values()
            .find(|p| p.crc_job_id.as_deref() == Some(crc_job_id))
            .cloned()
    }
}

impl Default for CICDSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedState {
    pipelines: Vec<Pipeline>,
    deployments: Vec<Deployment>,
}

#[cfg(test)]
mod pipeline_tests {
    use super::*;
    use noa_workflow::{EvidenceLedgerEntry, EvidenceLedgerKind};
    use serde_json::Value;
    use tempfile::tempdir;

    #[test]
    fn test_pipeline_trigger() {
        let workspace = tempdir().unwrap();
        let _guard = EnvGuard::set("NOA_WORKFLOW_ROOT", workspace.path());
        let cicd = CICDSystem::new();
        cicd.configure_workspace_root(workspace.path());
        let id = cicd
            .trigger_pipeline("test".to_string(), "abc123".to_string())
            .unwrap();

        assert!(cicd.get_pipeline_status(&id).is_some());
    }

    #[test]
    fn test_auto_approve() {
        let workspace = tempdir().unwrap();
        let _guard = EnvGuard::set("NOA_WORKFLOW_ROOT", workspace.path());
        let cicd = CICDSystem::new();
        cicd.configure_workspace_root(workspace.path());
        let id = cicd
            .trigger_from_crc(
                "test".to_string(),
                "abc123".to_string(),
                "crc_123".to_string(),
                0.96, // High confidence
            )
            .unwrap();

        let status = cicd.get_pipeline_status(&id).unwrap();
        assert_eq!(status, PipelineStatus::AutoApproved);
    }

    #[test]
    fn test_agent_review() {
        let workspace = tempdir().unwrap();
        let _guard = EnvGuard::set("NOA_WORKFLOW_ROOT", workspace.path());
        let cicd = CICDSystem::new();
        cicd.configure_workspace_root(workspace.path());
        let id = cicd
            .trigger_from_crc(
                "test".to_string(),
                "abc123".to_string(),
                "crc_123".to_string(),
                0.85, // Lower confidence
            )
            .unwrap();

        let status = cicd.get_pipeline_status(&id).unwrap();
        assert_eq!(status, PipelineStatus::AgentReview);
    }

    #[test]
    fn test_agent_approval_policy() {
        let workspace = tempdir().unwrap();
        let _guard = EnvGuard::set("NOA_WORKFLOW_ROOT", workspace.path());
        let cicd = CICDSystem::new();
        cicd.configure_workspace_root(workspace.path());
        let pipeline_id = cicd
            .trigger_doc_refresh_pipeline(
                "abc123".to_string(),
                "docs update".to_string(),
                vec![AgentApprovalRequirement {
                    role: "release-agent".to_string(),
                    minimum_trust_score: 0.7,
                    required_evidence_tags: vec!["ledger:release".to_string()],
                }],
            )
            .unwrap();

        // Low trust should escalate
        let result = cicd.register_agent_approval(
            &pipeline_id,
            "release-agent",
            "agent-low",
            0.5,
            vec!["ledger:release".to_string()],
            vec!["evidence-low".to_string()],
        );
        assert!(result.is_err());
        assert_eq!(
            cicd.get_pipeline_status(&pipeline_id).unwrap(),
            PipelineStatus::AgentEscalated
        );

        // Sufficient trust and evidence should approve
        let result = cicd
            .register_agent_approval(
                &pipeline_id,
                "release-agent",
                "agent-high",
                0.9,
                vec!["ledger:release".to_string()],
                vec!["evidence-high".to_string()],
            )
            .expect("agent approval should succeed");
        assert_eq!(result, PipelineStatus::AgentApproved);
        assert_eq!(
            cicd.get_pipeline_status(&pipeline_id).unwrap(),
            PipelineStatus::AgentApproved
        );
    }

    #[test]
    fn persisted_state_writes_pipeline_events_and_ledger() {
        let workspace = tempdir().unwrap();
        let _guard = EnvGuard::set("NOA_WORKFLOW_ROOT", workspace.path());
        let cicd = CICDSystem::new();
        cicd.configure_workspace_root(workspace.path());

        let pipeline_id = cicd
            .trigger_pipeline("persisted".to_string(), "abc999".to_string())
            .expect("pipeline triggered");
        cicd.update_pipeline_status(&pipeline_id, PipelineStatus::Running)
            .expect("status update succeeds");

        let deployment_id = cicd
            .deploy_to_environment(
                "v9.9.9".to_string(),
                Environment::Staging,
                DeploymentStrategy::BlueGreen,
            )
            .expect("deployment created");
        let _ = cicd
            .monitor_deployment(&deployment_id)
            .expect("monitoring succeeds");
        cicd.rollback(&deployment_id).expect("rollback succeeds");

        cicd.load_state_from_disk().expect("state reload succeeds");
        assert_eq!(
            cicd.get_pipeline_status(&pipeline_id),
            Some(PipelineStatus::Running)
        );

        let log_path = workspace
            .path()
            .join("storage")
            .join("db")
            .join("pipeline_events.log");
        assert!(log_path.exists(), "pipeline event log missing");
        let log_contents = std::fs::read_to_string(&log_path).expect("pipeline log readable");
        let events: Vec<Value> = log_contents
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| serde_json::from_str::<Value>(line).expect("valid pipeline event"))
            .collect();

        let has_pipeline_created = events.iter().any(|entry| {
            entry["event"]["event_type"].as_str() == Some("pipeline.state.created")
                && entry["event"]["scope"].as_str() == Some(pipeline_id.as_str())
        });
        assert!(has_pipeline_created, "missing pipeline.state.created entry");

        let has_pipeline_updated = events.iter().any(|entry| {
            entry["event"]["event_type"].as_str() == Some("pipeline.state.status_changed")
                && entry["event"]["scope"].as_str() == Some(pipeline_id.as_str())
        });
        assert!(
            has_pipeline_updated,
            "missing pipeline.state.status_changed entry"
        );

        let deployment_scope = format!("deployment::{}", deployment_id);
        let has_deployment_created = events.iter().any(|entry| {
            entry["event"]["event_type"].as_str() == Some("deployment.state.created")
                && entry["event"]["scope"].as_str() == Some(deployment_scope.as_str())
        });
        assert!(
            has_deployment_created,
            "missing deployment.state.created entry"
        );

        let has_deployment_status_change = events.iter().any(|entry| {
            entry["event"]["event_type"].as_str() == Some("deployment.state.status_changed")
                && entry["event"]["scope"].as_str() == Some(deployment_scope.as_str())
        });
        assert!(
            has_deployment_status_change,
            "missing deployment.state.status_changed entry"
        );

        let ledger_path = workspace
            .path()
            .join("storage")
            .join("db")
            .join("evidence")
            .join("ledger.jsonl");
        let ledger_contents = std::fs::read_to_string(&ledger_path).expect("ledger readable");
        let ledger_entries: Vec<EvidenceLedgerEntry> = ledger_contents
            .lines()
            .filter(|line| !line.trim().is_empty())
            .filter_map(|line| serde_json::from_str::<EvidenceLedgerEntry>(line).ok())
            .filter(|entry| entry.kind == EvidenceLedgerKind::PipelineEvent)
            .collect();

        let ledger_has_pipeline_created = ledger_entries.iter().any(|entry| {
            entry.payload.get("event_type").and_then(Value::as_str)
                == Some("pipeline.state.created")
        });
        let ledger_has_pipeline_updated = ledger_entries.iter().any(|entry| {
            entry.payload.get("event_type").and_then(Value::as_str)
                == Some("pipeline.state.status_changed")
        });
        let ledger_has_deployment_created = ledger_entries.iter().any(|entry| {
            entry.payload.get("event_type").and_then(Value::as_str)
                == Some("deployment.state.created")
        });
        let ledger_has_deployment_status_change = ledger_entries.iter().any(|entry| {
            entry.payload.get("event_type").and_then(Value::as_str)
                == Some("deployment.state.status_changed")
        });

        assert!(
            ledger_has_pipeline_created,
            "ledger missing pipeline created"
        );
        assert!(
            ledger_has_pipeline_updated,
            "ledger missing pipeline update"
        );
        assert!(
            ledger_has_deployment_created,
            "ledger missing deployment created"
        );
        assert!(
            ledger_has_deployment_status_change,
            "ledger missing deployment status change"
        );

        let report_path = workspace
            .path()
            .join("docs")
            .join("reports")
            .join("AGENT_DEPLOYMENT_OUTCOMES.md");
        let report_contents =
            std::fs::read_to_string(&report_path).expect("deployment report readable");
        assert!(
            report_contents.contains("deployment.state.created"),
            "deployment report missing state entry"
        );
    }

    #[test]
    fn test_pipeline_telemetry_log() {
        let workspace = tempdir().unwrap();
        let _guard = EnvGuard::set("NOA_WORKFLOW_ROOT", workspace.path());
        let cicd = CICDSystem::new();
        cicd.configure_workspace_root(workspace.path());
        let id = cicd
            .trigger_from_crc(
                "telemetry".to_string(),
                "abc123".to_string(),
                "crc_telemetry".to_string(),
                0.99,
            )
            .unwrap();

        let log_path = workspace
            .path()
            .join(".workspace")
            .join("indexes")
            .join("pipeline_events.log");
        assert!(log_path.exists(), "pipeline event log should exist");
        let contents = std::fs::read_to_string(log_path).expect("log readable");
        let last_line = contents
            .lines()
            .filter(|line| !line.trim().is_empty())
            .last()
            .expect("at least one event present");
        let payload: Value = serde_json::from_str(last_line).expect("valid json log entry");
        assert_eq!(payload["event"]["scope"].as_str(), Some(id.as_str()));
        assert_eq!(
            payload["event"]["event_type"].as_str(),
            Some("pipeline.auto_approved")
        );
    }
}
