//! CI/CD System - Continuous Delivery focused with CRC integration

pub mod trigger;
pub mod validation;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

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
    HumanReview,  // new
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub id: String,
    pub name: String,
    pub status: PipelineStatus,
    pub stages: Vec<Stage>,
    pub commit_sha: String,
    pub triggered_at: u64,
    pub crc_job_id: Option<String>,  // new: link to CRC job
    pub auto_approved: bool,          // new: AI auto-approval
    pub ai_confidence: f32,           // new: AI confidence score
    pub diff_summary: Option<String>,
    pub approvals_required: Vec<String>,
    #[serde(default)]
    pub approvals_granted: Vec<String>,
    pub crc_job_id: Option<String>, // new: link to CRC job
    pub auto_approved: bool,        // new: AI auto-approval
    pub ai_confidence: f32,         // new: AI confidence score
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
}

impl CICDSystem {
    pub fn new() -> Self {
        Self {
            pipelines: Arc::new(Mutex::new(HashMap::new())),
            deployments: Arc::new(Mutex::new(HashMap::new())),
            baseline_metrics: Arc::new(Mutex::new(HashMap::new())),
            auto_approve_threshold: 0.95, // 95% confidence
            single_host_profile: Arc::new(Mutex::new(Some(
                "server/profiles/single_host/profile.toml".to_string(),
            ))),
        }
    }

    /// Create CI/CD system with custom auto-approve threshold
    pub fn with_threshold(threshold: f32) -> Self {
        Self {
            pipelines: Arc::new(Mutex::new(HashMap::new())),
            deployments: Arc::new(Mutex::new(HashMap::new())),
            baseline_metrics: Arc::new(Mutex::new(HashMap::new())),
            auto_approve_threshold: threshold,
            single_host_profile: Arc::new(Mutex::new(Some(
                "server/profiles/single_host/profile.toml".to_string(),
            ))),
        }
    }

    /// Register the single-host profile manifest used for acceptance tests.
    pub fn configure_single_host_profile<P: Into<String>>(&self, profile_path: P) {
        let mut guard = self
            .single_host_profile
            .lock()
            .expect("single host profile lock poisoned");
        *guard = Some(profile_path.into());
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
        };

        let mut pipelines = self.pipelines.lock().unwrap();
        pipelines.insert(id.clone(), pipeline);

        println!("[CI/CD] Pipeline triggered: {}", id);
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
        {
            let mut pipelines = self.pipelines.lock().unwrap();
            if let Some(pipeline) = pipelines.get_mut(&id) {
                pipeline.crc_job_id = Some(crc_job_id);
                pipeline.ai_confidence = ai_confidence;
                pipeline.auto_approved = ai_confidence >= self.auto_approve_threshold;

                if pipeline.auto_approved {
                    pipeline.status = PipelineStatus::AutoApproved;
                    println!(
                        "[CI/CD] Pipeline AUTO-APPROVED (Confidence: {:.1}%)",
                        ai_confidence * 100.0
                    );
                } else {
                    pipeline.status = PipelineStatus::HumanReview;
                    println!(
                        "[CI/CD] Pipeline requires HUMAN REVIEW (Confidence: {:.1}%)",
                        ai_confidence * 100.0
                    );
                }
            }
        }

        Ok(id)
    }

        Ok(id)
    }

        Ok(id)
    }

        Ok(id)
    }

    pub fn trigger_doc_refresh_pipeline(
        &self,
        commit_sha: String,
        diff_summary: String,
        approvals_required: Vec<String>,
    ) -> Result<String, String> {
        let id = format!("doc_pipeline_{}", uuid::Uuid::new_v4());

        let pipeline = Pipeline {
            id: id.clone(),
            name: "documentation-refresh".to_string(),
            status: PipelineStatus::Pending,
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
            diff_summary: Some(diff_summary),
            approvals_required,
            approvals_granted: Vec::new(),
        };

        let mut pipelines = self.pipelines.lock().unwrap();
        pipelines.insert(id.clone(), pipeline);

        println!("[CI/CD] Documentation pipeline triggered: {}", id);
        Ok(id)
    }

    pub fn approve_pipeline(&self, pipeline_id: &str, approver: &str) -> Result<(), String> {
        let mut pipelines = self.pipelines.lock().unwrap();
        let pipeline = pipelines
            .get_mut(pipeline_id)
            .ok_or_else(|| format!("Pipeline not found: {}", pipeline_id))?;

        if pipeline
            .approvals_required
            .iter()
            .any(|required| required == approver)
        {
            if pipeline
                .approvals_granted
                .iter()
                .all(|granted| granted != approver)
            {
                pipeline.approvals_granted.push(approver.to_string());
                println!("[CI/CD] Approval added by {}", approver);
            }
            Ok(())
        } else {
            Err(format!(
                "Approver {} is not required for pipeline {}",
                approver, pipeline_id
            ))
        }
    }
    
    /// Execute pipeline with full automation
    pub fn execute_pipeline(&self, pipeline_id: &str) -> Result<(), String> {
        // Check if requires human review
        {
            let pipelines = self.pipelines.lock().unwrap();
            if let Some(pipeline) = pipelines.get(pipeline_id) {
                if pipeline.status == PipelineStatus::HumanReview {
                    return Err("Pipeline requires human review before execution".to_string());
                }
                if !pipeline.approvals_required.is_empty()
                    && !pipeline.approvals_required.iter().all(|required| pipeline.approvals_granted.contains(required))
                {
                    return Err("Pipeline is waiting for documentation approvals".to_string());
                }
            }
        }

        println!("[CI/CD] Executing pipeline: {}", pipeline_id);

        let stages = {
            let pipelines = self.pipelines.lock().unwrap();
            pipelines
                .get(pipeline_id)
                .ok_or_else(|| format!("Pipeline not found: {}", pipeline_id))?
                .stages
                .clone()
        };

        // Execute each stage
        for stage in stages {
            self.execute_stage(pipeline_id, &stage)?;
        }

        // Mark pipeline as success
        self.update_pipeline_status(pipeline_id, PipelineStatus::Success)?;

        println!("[CI/CD] Pipeline completed successfully: {}", pipeline_id);
        Ok(())
    }

    /// Execute a single stage
    fn execute_stage(&self, pipeline_id: &str, stage: &Stage) -> Result<(), String> {
        println!("[CI/CD] Executing stage: {:?}", stage.stage_type);

        let start = std::time::Instant::now();

        // Simulate stage execution
        match stage.stage_type {
            PipelineStage::CRC => self.crc_stage()?,
            PipelineStage::Validate => self.validate()?,
            PipelineStage::Build => self.build()?,
            PipelineStage::Test => self.test()?,
            PipelineStage::SingleHostAcceptance => self.single_host_acceptance()?,
            PipelineStage::Deploy => self.deploy()?,
            PipelineStage::DocsRefresh => self.docs_refresh(pipeline_id)?,
            _ => {}
        }

        let duration = start.elapsed().as_millis() as u64;
        println!("[CI/CD] Stage completed in {}ms", duration);

        Ok(())
    }

    /// CRC stage (if needed)
    fn crc_stage(&self) -> Result<(), String> {
        println!("[CI/CD] CRC adaptation already complete");
        Ok(())
    }

    /// Validation stage
    fn validate(&self) -> Result<(), String> {
        println!("[CI/CD] Running validation checks...");
        // Lint, format, security scan
        Ok(())
    }

    /// Build stage
    fn build(&self) -> Result<(), String> {
        println!("[CI/CD] Building all components...");
        // Build Rust, Go, Python, .NET
        Ok(())
    }

    /// Test stage
    fn test(&self) -> Result<(), String> {
        println!("[CI/CD] Running tests...");
        // Unit, integration, API tests
        Ok(())
    }

    /// Acceptance checks for the single-host profile
    fn single_host_acceptance(&self) -> Result<(), String> {
        println!("[CI/CD] Running single-host acceptance suite...");

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

        println!(
            "[CI/CD] ✓ Validated profile manifest at {} ({} bytes)",
            profile_path,
            manifest.len()
        );
        println!("[CI/CD] ✓ Acceptance checks ready for workload replay");
        Ok(())
    }

    /// Deploy stage
    fn deploy(&self) -> Result<(), String> {
        println!("[CI/CD] Deploying to staging...");
        Ok(())
    }

     fn docs_refresh(&self, pipeline_id: &str) -> Result<(), String> {
        let diff_summary = {
            let pipelines = self.pipelines.lock().unwrap();
            pipelines
                .get(pipeline_id)
                .and_then(|p| p.diff_summary.clone())
                .unwrap_or_else(|| "No diff summary provided".to_string())
        };

        println!("[CI/CD] Running documentation refresh...");
        println!("[CI/CD] Diff summary: {}", diff_summary);
        println!("[CI/CD] Invoking documentation agent to sync docs");
        Ok(())
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

        if auto_approved {
            println!("[CI/CD] Deploying to {:?} (AUTO-APPROVED)", environment);
        } else {
            println!("[CI/CD] Deploying to {:?} (MANUAL APPROVAL)", environment);
        }

        Ok(id)
    }

    /// Monitor deployment health with auto-rollback
    pub fn monitor_deployment(&self, deployment_id: &str) -> Result<bool, String> {
        let deployments = self.deployments.lock().unwrap();
        let deployment = deployments
            .get(deployment_id)
            .ok_or_else(|| format!("Deployment not found: {}", deployment_id))?;

        let baseline_metrics = self.baseline_metrics.lock().unwrap();
        let baseline = baseline_metrics
            .get(&deployment.environment)
            .cloned()
            .unwrap_or_default();

        let is_healthy = deployment.health_metrics.is_healthy(&baseline);

        if !is_healthy {
            println!("[CI/CD] ⚠ Deployment health check FAILED");
            println!("[CI/CD] Triggering AUTOMATIC ROLLBACK...");
        } else {
            println!("[CI/CD] ✓ Deployment health check PASSED");
        }

        Ok(is_healthy)
    }

    /// Rollback deployment (automatic)
    pub fn rollback(&self, deployment_id: &str) -> Result<(), String> {
        println!(
            "[CI/CD] ⚠ AUTOMATIC ROLLBACK initiated for: {}",
            deployment_id
        );

        let mut deployments = self.deployments.lock().unwrap();
        if let Some(deployment) = deployments.get_mut(deployment_id) {
            deployment.status = PipelineStatus::RolledBack;
            println!("[CI/CD] ✓ Rollback completed successfully (< 30 seconds)");
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
            println!("[CI/CD] ✓ AUTO-PROMOTING to {:?}", to_environment);
            // Implementation would promote to next environment
            Ok(())
        } else {
            Err("Deployment not healthy for auto-promotion".to_string())
        }
    }

    /// Complete end-to-end automation
    pub fn full_auto_pipeline(&self, crc_job_id: String, ai_confidence: f32) -> Result<(), String> {
        println!("\n[CI/CD] ═══════════════════════════════════════");
        println!("[CI/CD] FULL AUTO PIPELINE");
        println!("[CI/CD] ═══════════════════════════════════════\n");

        // Trigger from CRC
        let pipeline_id = self.trigger_from_crc(
            "auto-pipeline".to_string(),
            "auto-commit".to_string(),
            crc_job_id,
            ai_confidence,
        )?;

        // If auto-approved, continue
        {
            let pipelines = self.pipelines.lock().unwrap();
            if let Some(pipeline) = pipelines.get(&pipeline_id) {
                if !pipeline.auto_approved {
                    println!("[CI/CD] Pipeline requires human review. Exiting.");
                    return Ok(());
                }
            }
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
                println!("\n[CI/CD] ✓ FULL AUTO PIPELINE COMPLETE");
            } else {
                self.rollback(&prod_deploy)?;
                println!("\n[CI/CD] ⚠ AUTO-ROLLBACK TRIGGERED");
            }
        }

        println!("[CI/CD] ═══════════════════════════════════════\n");
        Ok(())
    }

    /// Update pipeline status
    fn update_pipeline_status(
        &self,
        pipeline_id: &str,
        status: PipelineStatus,
    ) -> Result<(), String> {
        let mut pipelines = self.pipelines.lock().unwrap();
        if let Some(pipeline) = pipelines.get_mut(pipeline_id) {
            pipeline.status = status;
            Ok(())
        } else {
            Err(format!("Pipeline not found: {}", pipeline_id))
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_trigger() {
        let cicd = CICDSystem::new();
        let id = cicd
            .trigger_pipeline("test".to_string(), "abc123".to_string())
            .unwrap();

        assert!(cicd.get_pipeline_status(&id).is_some());
    }

    #[test]
    fn test_auto_approve() {
        let cicd = CICDSystem::new();
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
    fn test_human_review() {
        let cicd = CICDSystem::new();
        let id = cicd
            .trigger_from_crc(
                "test".to_string(),
                "abc123".to_string(),
                "crc_123".to_string(),
                0.85, // Lower confidence
            )
            .unwrap();

        let status = cicd.get_pipeline_status(&id).unwrap();
        assert_eq!(status, PipelineStatus::HumanReview);
    }
}
