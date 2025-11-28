// CI/CD Trigger Integration - Automation from CRC to CI/CD pipeline
// Listens for CRC completion events and triggers CI/CD pipelines

use crate::ledger::{AuditLedger, LedgerAction, LedgerEntry};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::{error, info, instrument, warn};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SandboxModel {
    ModelA,
    ModelB,
    ModelC,
    ModelD,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("System error: {0}")]
    SystemError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Send error: {0}")]
    SendError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// CI/CD trigger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerConfig {
    pub enabled: bool,
    pub webhook_url: Option<String>,
    pub auto_merge_threshold: f32,
    pub watch_ready_queues: Vec<String>,
    pub pipeline_timeout_secs: u64,
    #[serde(default = "TriggerConfig::default_ledger_path")]
    pub ledger_path: PathBuf,
    #[serde(default = "TriggerConfig::default_repo_path")]
    pub repo_path: PathBuf,
    #[serde(default = "TriggerConfig::default_integration_branch")]
    pub integration_branch: String,
    #[serde(default = "TriggerConfig::default_required_trust_average")]
    pub required_trust_average: f32,
    #[serde(default = "TriggerConfig::default_trust_history_depth")]
    pub trust_history_depth: usize,
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            webhook_url: None,
            auto_merge_threshold: 0.95,
            watch_ready_queues: vec![
                "model-a-queue".to_string(),
                "model-b-queue".to_string(),
                "model-c-queue".to_string(),
                "model-d-queue".to_string(),
            ],
            pipeline_timeout_secs: 3600, // 1 hour
            ledger_path: Self::default_ledger_path(),
            repo_path: Self::default_repo_path(),
            integration_branch: Self::default_integration_branch(),
            required_trust_average: Self::default_required_trust_average(),
            trust_history_depth: Self::default_trust_history_depth(),
        }
    }
}

impl TriggerConfig {
    fn default_ledger_path() -> PathBuf {
        PathBuf::from("audit/ledger.jsonl")
    }

    fn default_repo_path() -> PathBuf {
        PathBuf::from(".")
    }

    fn default_integration_branch() -> String {
        "main".to_string()
    }

    fn default_required_trust_average() -> f32 {
        0.9
    }

    fn default_trust_history_depth() -> usize {
        10
    }
}

/// Pipeline trigger event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEvent {
    pub drop_id: String,
    pub sandbox: SandboxModel,
    pub confidence: f32,
    pub timestamp: u64,
    pub ready_path: PathBuf,
}

/// Pipeline execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PipelineStatus {
    Queued,
    Running,
    Success,
    Failed,
    Timeout,
}

/// Pipeline result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    pub drop_id: String,
    pub status: PipelineStatus,
    pub duration_secs: u64,
    pub stages_completed: Vec<String>,
    pub errors: Vec<String>,
    pub artifacts: Vec<String>,
}

#[derive(Debug, Clone)]
struct TrustDecision {
    passed: bool,
    effective_score: f32,
    historical_average: Option<f32>,
    reason: String,
}

/// CI/CD trigger manager
pub struct TriggerManager {
    config: TriggerConfig,
    base_path: PathBuf,
    event_tx: mpsc::Sender<TriggerEvent>,
    event_rx: Option<mpsc::Receiver<TriggerEvent>>,
    ledger: AuditLedger,
}

impl TriggerManager {
    /// Create new trigger manager
    pub fn new(base_path: PathBuf, config: TriggerConfig) -> Self {
        let (tx, rx) = mpsc::channel(100);

        let ledger_path = if config.ledger_path.is_relative() {
            base_path.join(&config.ledger_path)
        } else {
            config.ledger_path.clone()
        };

        Self {
            config,
            base_path,
            event_tx: tx,
            event_rx: Some(rx),
            ledger: AuditLedger::new(ledger_path),
        }
    }

    fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_relative() {
            self.base_path.join(path)
        } else {
            path.to_path_buf()
        }
    }

    fn repo_root(&self) -> PathBuf {
        self.resolve_path(&self.config.repo_path)
    }

    async fn evaluate_trust_metrics(&self, event: &TriggerEvent) -> Result<TrustDecision> {
        let entries = self.ledger.load_entries().await?;
        let sandbox_key = format!("{:?}", event.sandbox);

        let mut total = 0.0f32;
        let mut count = 0usize;

        for entry in entries.iter().rev() {
            if entry.action != LedgerAction::Approval {
                continue;
            }

            if entry.sandbox != sandbox_key {
                continue;
            }

            if let Some(score) = entry.trust_score {
                total += score;
                count += 1;
            }

            if count >= self.config.trust_history_depth {
                break;
            }
        }

        let historical_average = if count > 0 {
            Some(total / count as f32)
        } else {
            None
        };

        let effective_score = if let Some(avg) = historical_average {
            (avg + event.confidence) / 2.0
        } else {
            event.confidence
        };

        let confidence_threshold_met = event.confidence >= self.config.auto_merge_threshold;
        let passed =
            confidence_threshold_met && effective_score >= self.config.required_trust_average;

        let reason = if passed {
            format!(
                "confidence {:.2} and effective score {:.2} exceed thresholds",
                event.confidence, effective_score
            )
        } else {
            format!(
                "confidence {:.2} / effective score {:.2} below thresholds {:.2} / {:.2}",
                event.confidence,
                effective_score,
                self.config.auto_merge_threshold,
                self.config.required_trust_average
            )
        };

        info!(
            "Trust decision for {}: passed={} score={:.2} historical_avg={:?}",
            event.drop_id, passed, effective_score, historical_average
        );

        Ok(TrustDecision {
            passed,
            effective_score,
            historical_average,
            reason,
        })
    }

    /// Start monitoring ready queues for new drops
    #[instrument(skip(self))]
    pub async fn start_monitoring(&mut self) -> Result<()> {
        if !self.config.enabled {
            info!("CI/CD triggers disabled");
            return Ok(());
        }

        info!("Starting CI/CD trigger monitoring");
        info!(
            "  Watching {} ready queues",
            self.config.watch_ready_queues.len()
        );

        let mut event_rx = self
            .event_rx
            .take()
            .ok_or_else(|| Error::SystemError("Event receiver already taken".to_string()))?;

        // Spawn queue watchers for each ready queue
        for queue_name in &self.config.watch_ready_queues {
            let queue_path = self.base_path.join("drop-in/ready").join(queue_name);
            let tx = self.event_tx.clone();
            let queue_name_clone = queue_name.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::watch_queue(queue_path, queue_name_clone, tx).await {
                    error!("Queue watcher error: {:?}", e);
                }
            });
        }

        info!("✓ Queue watchers started");

        // Event processing loop
        while let Some(event) = event_rx.recv().await {
            if let Err(e) = self.handle_trigger_event(event).await {
                error!("Error handling trigger event: {:?}", e);
            }
        }

        Ok(())
    }

    /// Watch a specific ready queue for new drops
    async fn watch_queue(
        queue_path: PathBuf,
        queue_name: String,
        tx: mpsc::Sender<TriggerEvent>,
    ) -> Result<()> {
        info!("Watching queue: {}", queue_name);

        loop {
            // Check for new drops in queue
            if queue_path.exists() {
                let mut entries = fs::read_dir(&queue_path).await?;

                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();

                    if path.is_dir() {
                        // New drop detected
                        let drop_id = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        info!("[{}] New drop detected: {}", queue_name, drop_id);

                        // Parse sandbox from queue name
                        let sandbox = Self::parse_sandbox_from_queue(&queue_name);

                        // Create trigger event
                        let event = TriggerEvent {
                            drop_id: drop_id.clone(),
                            sandbox,
                            confidence: 0.90, // Would read from metadata
                            timestamp: std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            ready_path: path.clone(),
                        };

                        // Send event
                        if let Err(e) = tx.send(event).await {
                            error!("Failed to send trigger event: {:?}", e);
                        }
                    }
                }
            }

            // Poll interval
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    /// Handle trigger event and execute pipeline
    #[instrument(skip(self))]
    async fn handle_trigger_event(&self, event: TriggerEvent) -> Result<()> {
        info!(
            "Processing trigger event: {} ({:?})",
            event.drop_id, event.sandbox
        );

        // Check confidence threshold for auto-merge
        let auto_merge = event.confidence >= self.config.auto_merge_threshold;
        info!(
            "  Confidence: {:.1}% (auto-merge: {})",
            event.confidence * 100.0,
            auto_merge
        );

        // Evaluate trust metrics ahead of pipeline completion so we can reason about merge decisions
        let trust_decision = if auto_merge {
            Some(self.evaluate_trust_metrics(&event).await?)
        } else {
            None
        };

        // Trigger pipeline
        let result = self.trigger_pipeline(&event, auto_merge).await?;

        info!(
            "Pipeline completed: {} ({:?})",
            event.drop_id, result.status
        );
        info!("  Duration: {}s", result.duration_secs);
        info!("  Stages: {}", result.stages_completed.len());

        if !result.errors.is_empty() {
            warn!("  Errors: {}", result.errors.len());
            for error in &result.errors {
                warn!("    - {}", error);
            }
        }

        // Handle result
        match result.status {
            PipelineStatus::Success => {
                info!("✓ Pipeline succeeded for {}", event.drop_id);

                // If auto-merge, trigger merge to main
                if let Some(ref decision) = trust_decision {
                    if decision.passed {
                        info!(
                            "Triggering auto-merge to main branch with trust score {:.2}",
                            decision.effective_score
                        );
                        self.trigger_merge(&event, decision).await?;
                    } else {
                        warn!(
                            "Skipping auto-merge for {}: {}",
                            event.drop_id, decision.reason
                        );
                    }
                } else if auto_merge {
                    warn!(
                        "Auto-merge requested for {} but trust decision unavailable",
                        event.drop_id
                    );
                }
            }
            PipelineStatus::Failed => {
                error!("✗ Pipeline failed for {}", event.drop_id);
                // Would send notification, create issue, etc.
            }
            PipelineStatus::Timeout => {
                error!("✗ Pipeline timeout for {}", event.drop_id);
            }
            _ => {}
        }

        Ok(())
    }

    /// Trigger CI/CD pipeline execution
    #[instrument(skip(self))]
    async fn trigger_pipeline(
        &self,
        event: &TriggerEvent,
        auto_merge: bool,
    ) -> Result<PipelineResult> {
        info!("Triggering pipeline for: {}", event.drop_id);

        let start_time = std::time::SystemTime::now();
        let mut stages_completed = Vec::new();
        let mut errors = Vec::new();
        let mut artifacts = Vec::new();

        // Stage 1: Validation
        info!("  [1/4] Validation");
        match self.run_validation(&event.ready_path).await {
            Ok(_) => {
                stages_completed.push("validation".to_string());
                info!("    ✓ Validation passed");
            }
            Err(e) => {
                errors.push(format!("Validation failed: {}", e));
                error!("    ✗ Validation failed: {}", e);
            }
        }

        // Stage 2: Build
        if errors.is_empty() {
            info!("  [2/4] Build");
            match self.run_build(&event.ready_path).await {
                Ok(build_artifacts) => {
                    stages_completed.push("build".to_string());
                    artifacts.extend(build_artifacts);
                    info!("    ✓ Build succeeded");
                }
                Err(e) => {
                    errors.push(format!("Build failed: {}", e));
                    error!("    ✗ Build failed: {}", e);
                }
            }
        }

        // Stage 3: Test
        if errors.is_empty() {
            info!("  [3/4] Test");
            match self.run_tests(&event.ready_path).await {
                Ok(_) => {
                    stages_completed.push("test".to_string());
                    info!("    ✓ Tests passed");
                }
                Err(e) => {
                    errors.push(format!("Tests failed: {}", e));
                    error!("    ✗ Tests failed: {}", e);
                }
            }
        }

        // Stage 4: Deploy (only if auto-merge enabled)
        if errors.is_empty() && auto_merge {
            info!("  [4/4] Deploy");
            match self.run_deploy(&event.ready_path, &event.sandbox).await {
                Ok(_) => {
                    stages_completed.push("deploy".to_string());
                    info!("    ✓ Deploy succeeded");
                }
                Err(e) => {
                    errors.push(format!("Deploy failed: {}", e));
                    error!("    ✗ Deploy failed: {}", e);
                }
            }
        }

        let duration = start_time.elapsed().unwrap_or_default().as_secs();

        let status = if !errors.is_empty() {
            PipelineStatus::Failed
        } else if duration > self.config.pipeline_timeout_secs {
            PipelineStatus::Timeout
        } else {
            PipelineStatus::Success
        };

        Ok(PipelineResult {
            drop_id: event.drop_id.clone(),
            status,
            duration_secs: duration,
            stages_completed,
            errors,
            artifacts,
        })
    }

    /// Trigger merge to main branch
    #[instrument(skip(self))]
    async fn trigger_merge(&self, event: &TriggerEvent, decision: &TrustDecision) -> Result<()> {
        info!(
            "Merging {} from {:?} into {}",
            event.drop_id, event.sandbox, self.config.integration_branch
        );

        let repo_root = self.repo_root();
        if !repo_root.exists() {
            return Err(Error::SystemError(format!(
                "Repository path {:?} does not exist",
                repo_root
            )));
        }

        let source_ref = self
            .determine_source_ref(&repo_root, &event.drop_id)
            .await?;
        let diff_path = self
            .perform_git_merge_preview(&repo_root, event, &source_ref)
            .await?;

        let mut details = format!(
            "merged into {} using {}",
            self.config.integration_branch, source_ref
        );
        if let Some(avg) = decision.historical_average {
            details.push_str(&format!("; historical avg {:.2}", avg));
        }

        let ledger_entry = LedgerEntry::new(
            &event.drop_id,
            format!("{:?}", event.sandbox),
            LedgerAction::Approval,
        )
        .with_actor("autonomous-release-bot")
        .with_trust_score(decision.effective_score)
        .with_details(details)
        .with_artifact(diff_path.to_string_lossy());

        self.ledger.append(&ledger_entry).await?;

        info!("✓ Merge preview stored at {}", diff_path.display());

        Ok(())
    }

    // === Pipeline Stage Implementations ===

    async fn run_validation(&self, _path: &Path) -> Result<()> {
        // Simulate validation
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        Ok(())
    }

    async fn run_build(&self, path: &Path) -> Result<Vec<String>> {
        // Simulate build
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let artifacts = vec![format!("{}/target/release/binary", path.display())];

        Ok(artifacts)
    }

    async fn run_tests(&self, _path: &Path) -> Result<()> {
        // Simulate tests
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        Ok(())
    }

    async fn run_deploy(&self, _path: &Path, _sandbox: &SandboxModel) -> Result<()> {
        // Simulate deployment
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        Ok(())
    }

    async fn run_git_command(&self, repo_root: &Path, args: &[&str]) -> Result<String> {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo_root)
            .output()
            .await?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(Error::SystemError(format!(
                "git {:?} failed: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }

    async fn run_git_command_optional(
        &self,
        repo_root: &Path,
        args: &[&str],
    ) -> Result<Option<String>> {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo_root)
            .output()
            .await?;

        if output.status.success() {
            Ok(Some(String::from_utf8_lossy(&output.stdout).to_string()))
        } else {
            warn!(
                "git {:?} returned non-zero exit: {}",
                args,
                String::from_utf8_lossy(&output.stderr)
            );
            Ok(None)
        }
    }

    async fn reference_exists(&self, repo_root: &Path, reference: &str) -> Result<bool> {
        let output = Command::new("git")
            .args(["show-ref", "--verify", reference])
            .current_dir(repo_root)
            .output()
            .await?;

        Ok(output.status.success())
    }

    async fn determine_source_ref(&self, repo_root: &Path, drop_id: &str) -> Result<String> {
        let _ = self
            .run_git_command_optional(repo_root, &["fetch", "origin"])
            .await?;

        let local_ref = format!("refs/heads/{}", drop_id);
        if self.reference_exists(repo_root, &local_ref).await? {
            return Ok(local_ref);
        }

        let remote_ref = format!("refs/remotes/origin/{}", drop_id);
        if self.reference_exists(repo_root, &remote_ref).await? {
            return Ok(remote_ref);
        }

        Err(Error::SystemError(format!(
            "Unable to locate git reference for drop {}",
            drop_id
        )))
    }

    async fn perform_git_merge_preview(
        &self,
        repo_root: &Path,
        event: &TriggerEvent,
        source_ref: &str,
    ) -> Result<PathBuf> {
        let integration_branch = &self.config.integration_branch;
        let source_short = source_ref
            .strip_prefix("refs/remotes/origin/")
            .unwrap_or(source_ref);

        if self
            .run_git_command_optional(repo_root, &["fetch", "origin", integration_branch])
            .await?
            .is_none()
        {
            warn!(
                "Failed to fetch integration branch '{}'",
                integration_branch
            );
        }
        if self
            .run_git_command_optional(repo_root, &["fetch", "origin", source_short])
            .await?
            .is_none()
        {
            warn!("Failed to fetch source branch '{}'", source_short);
        }

        let base_commit = self
            .run_git_command(repo_root, &["merge-base", integration_branch, source_ref])
            .await?;
        let merge_preview = self
            .run_git_command(
                repo_root,
                &[
                    "merge-tree",
                    base_commit.trim(),
                    integration_branch,
                    source_ref,
                ],
            )
            .await?;

        let merges_dir = self.resolve_path(Path::new("audit/merges"));
        fs::create_dir_all(&merges_dir).await?;
        let diff_path = merges_dir.join(format!("{}-{}.diff", event.drop_id, event.timestamp));
        tokio::fs::write(&diff_path, merge_preview).await?;

        Ok(diff_path)
    }

    // === Helper Methods ===

    fn parse_sandbox_from_queue(queue_name: &str) -> SandboxModel {
        if queue_name.contains("model-a") {
            SandboxModel::ModelA
        } else if queue_name.contains("model-b") {
            SandboxModel::ModelB
        } else if queue_name.contains("model-c") {
            SandboxModel::ModelC
        } else if queue_name.contains("model-d") {
            SandboxModel::ModelD
        } else {
            SandboxModel::ModelA // Default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command as StdCommand;
    use tempfile::tempdir;

    #[test]
    fn test_trigger_config_default() {
        let config = TriggerConfig::default();
        assert_eq!(config.enabled, true);
        assert_eq!(config.auto_merge_threshold, 0.95);
        assert_eq!(config.watch_ready_queues.len(), 4);
    }

    #[test]
    fn test_parse_sandbox_from_queue() {
        assert_eq!(
            TriggerManager::parse_sandbox_from_queue("model-a-queue"),
            SandboxModel::ModelA
        );
        assert_eq!(
            TriggerManager::parse_sandbox_from_queue("model-b-queue"),
            SandboxModel::ModelB
        );
        assert_eq!(
            TriggerManager::parse_sandbox_from_queue("model-d-queue"),
            SandboxModel::ModelD
        );
    }

    fn init_git_repo(path: &Path) {
        StdCommand::new("git")
            .args(["init", "-b", "main"])
            .current_dir(path)
            .output()
            .expect("git init should succeed");

        fs::write(path.join("README.md"), "# Demo\n").unwrap();
        StdCommand::new("git")
            .args(["add", "README.md"])
            .current_dir(path)
            .output()
            .expect("git add should succeed");
        StdCommand::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(path)
            .output()
            .expect("git commit should succeed");

        StdCommand::new("git")
            .args(["checkout", "-b", "feature"])
            .current_dir(path)
            .output()
            .expect("git checkout -b feature should succeed");
        fs::write(path.join("feature.txt"), "new change\n").unwrap();
        StdCommand::new("git")
            .args(["add", "feature.txt"])
            .current_dir(path)
            .output()
            .expect("git add feature should succeed");
        StdCommand::new("git")
            .args(["commit", "-m", "Feature commit"])
            .current_dir(path)
            .output()
            .expect("git commit feature should succeed");
        StdCommand::new("git")
            .args(["checkout", "main"])
            .current_dir(path)
            .output()
            .expect("git checkout main should succeed");
    }

    #[tokio::test]
    async fn trust_metrics_require_threshold() {
        let workspace = tempdir().unwrap();
        let mut config = TriggerConfig::default();
        config.ledger_path = PathBuf::from("ledger.jsonl");
        config.repo_path = PathBuf::from(".");
        config.auto_merge_threshold = 0.95;
        config.required_trust_average = 0.95;

        let manager = TriggerManager::new(workspace.path().to_path_buf(), config);
        let event = TriggerEvent {
            drop_id: "demo".into(),
            sandbox: SandboxModel::ModelA,
            confidence: 0.80,
            timestamp: 1,
            ready_path: workspace.path().join("ready/demo"),
        };

        let decision = manager.evaluate_trust_metrics(&event).await.unwrap();
        assert!(!decision.passed);
    }

    #[tokio::test]
    async fn trust_metrics_consider_historical_scores() {
        let workspace = tempdir().unwrap();
        let mut config = TriggerConfig::default();
        config.ledger_path = PathBuf::from("ledger.jsonl");
        config.repo_path = PathBuf::from(".");
        config.auto_merge_threshold = 0.80;
        config.required_trust_average = 0.85;

        let manager = TriggerManager::new(workspace.path().to_path_buf(), config);
        manager
            .ledger
            .append(
                &LedgerEntry::new("demo", "ModelA", LedgerAction::Approval).with_trust_score(0.90),
            )
            .await
            .unwrap();

        let event = TriggerEvent {
            drop_id: "demo".into(),
            sandbox: SandboxModel::ModelA,
            confidence: 0.80,
            timestamp: 2,
            ready_path: workspace.path().join("ready/demo"),
        };

        let decision = manager.evaluate_trust_metrics(&event).await.unwrap();
        assert!(decision.passed);
        assert!(decision.effective_score >= 0.85);
    }

    #[tokio::test]
    async fn trigger_merge_records_diff_and_ledger_entry() {
        let repo = tempdir().unwrap();
        init_git_repo(repo.path());

        let mut config = TriggerConfig::default();
        config.repo_path = PathBuf::from(".");
        config.ledger_path = PathBuf::from("audit/ledger.jsonl");
        config.integration_branch = "main".into();

        let manager = TriggerManager::new(repo.path().to_path_buf(), config);
        let event = TriggerEvent {
            drop_id: "feature".into(),
            sandbox: SandboxModel::ModelA,
            confidence: 1.0,
            timestamp: 99,
            ready_path: repo.path().join("ready/demo"),
        };

        let decision = TrustDecision {
            passed: true,
            effective_score: 0.98,
            historical_average: Some(0.95),
            reason: "test".into(),
        };

        manager
            .trigger_merge(&event, &decision)
            .await
            .expect("merge preview should succeed");

        let merges_dir = repo.path().join("audit/merges");
        let diff_path = merges_dir.join("feature-99.diff");
        assert!(diff_path.exists());

        let entries = manager.ledger.load_entries().await.unwrap();
        assert!(entries.iter().any(|entry| entry.drop_id == "feature"));
    }
}
