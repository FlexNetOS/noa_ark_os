use std::path::{Path, PathBuf};
use std::sync::Arc;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

use crate::instrumentation::{
    AutoFixActionReceipt, PipelineInstrumentation, PolicyDecisionRecord,
};

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum AutoFixerKind {
    Lint,
    Type,
    FlakyTest,
    Refactor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoFixActionPlan {
    pub description: String,
    pub commands: Vec<String>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoFixRequest {
    pub kind: AutoFixerKind,
    pub target: PathBuf,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoFixOutcome {
    pub request: AutoFixRequest,
    pub plan: AutoFixActionPlan,
    pub receipt: AutoFixActionReceipt,
}

#[derive(Debug, Error)]
pub enum AutoFixError {
    #[error("instrumentation failure: {0}")]
    Instrumentation(String),
    #[error("unsupported fixer: {0}")]
    Unsupported(String),
}

pub struct AutoFixCoordinator {
    instrumentation: Arc<PipelineInstrumentation>,
}

impl AutoFixCoordinator {
    pub fn new(instrumentation: Arc<PipelineInstrumentation>) -> Self {
        Self { instrumentation }
    }

    pub fn plan_for(&self, kind: AutoFixerKind, target: &Path, dry_run: bool) -> AutoFixActionPlan {
        match kind {
            AutoFixerKind::Lint => AutoFixActionPlan {
                description: format!("Run multi-language lint suite for {}", target.display()),
                commands: vec![
                    "cargo fmt --all".into(),
                    "cargo clippy --all-targets -- -D warnings".into(),
                    "pnpm lint --filter noa-ark".into(),
                ],
                dry_run,
            },
            AutoFixerKind::Type => AutoFixActionPlan {
                description: format!("Execute type checking for {}", target.display()),
                commands: vec![
                    "mypy --config-file pyproject.toml".into(),
                    "pnpm ts:check".into(),
                    "cargo check --all-targets".into(),
                ],
                dry_run,
            },
            AutoFixerKind::FlakyTest => AutoFixActionPlan {
                description: format!("Stabilise flaky tests detected around {}", target.display()),
                commands: vec![
                    "pytest -n auto --maxfail=1 --reruns 2".into(),
                    "cargo test -- --include-ignored".into(),
                    "pnpm test --filter flaky".into(),
                ],
                dry_run,
            },
            AutoFixerKind::Refactor => AutoFixActionPlan {
                description: format!("Perform refactor pass for {}", target.display()),
                commands: vec![
                    "cargo clippy --workspace --fix --allow-dirty --allow-staged".into(),
                    "python scripts/maintenance/refactor_guard.py".into(),
                    "pnpm format --filter noa-ark".into(),
                ],
                dry_run,
            },
        }
    }

    pub fn execute(&self, request: AutoFixRequest) -> Result<AutoFixOutcome, AutoFixError> {
        let plan = self.plan_for(request.kind, &request.target, request.dry_run);
        let kind_name = request
            .kind
            .to_possible_value()
            .map(|v| v.get_name().to_string())
            .unwrap_or_else(|| "custom".to_string());
        let fixer_id = format!("workflow.auto_fix.{}", kind_name);
        let target_str = request.target.to_string_lossy().to_string();
        let plan_value = serde_json::to_value(&plan).map_err(|err| AutoFixError::Instrumentation(err.to_string()))?;
        let policy = PolicyDecisionRecord {
            decision: if request.dry_run { "review" } else { "allow" }.to_string(),
            reason: format!("auto-fix::{}", kind_name),
            signals: vec![format!("auto_fix::{}", kind_name)],
            metadata: json!({
                "dry_run": request.dry_run,
                "target": target_str,
                "request_metadata": request.metadata,
            }),
        };

        let receipt = self
            .instrumentation
            .record_auto_fix_action(&fixer_id, &target_str, &plan_value, &policy)
            .map_err(|err| AutoFixError::Instrumentation(err.to_string()))?;

        Ok(AutoFixOutcome {
            request,
            plan,
            receipt,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use tempfile::TempDir;

    use crate::instrumentation::PipelineInstrumentation;

    fn coordinator() -> (AutoFixCoordinator, TempDir) {
        let temp_root = TempDir::new().expect("temp workflow root");
        let previous = std::env::var_os("NOA_WORKFLOW_ROOT");
        std::env::set_var("NOA_WORKFLOW_ROOT", temp_root.path());
        let instrumentation = Arc::new(
            PipelineInstrumentation::new().expect("instrumentation bootstrap"),
        );
        if let Some(value) = previous {
            std::env::set_var("NOA_WORKFLOW_ROOT", value);
        } else {
            std::env::remove_var("NOA_WORKFLOW_ROOT");
        }

        (AutoFixCoordinator::new(instrumentation), temp_root)
    }

    #[test]
    fn plans_include_expected_commands() {
        let (coord, _temp_dir) = coordinator();
        let plan = coord.plan_for(AutoFixerKind::Lint, Path::new("."), true);
        assert!(plan.commands.iter().any(|cmd| cmd.contains("clippy")));
        assert!(plan.dry_run);
    }

    #[test]
    fn execute_records_policy_and_snapshot() {
        let (coord, _temp_dir) = coordinator();
        let request = AutoFixRequest {
            kind: AutoFixerKind::Type,
            target: PathBuf::from("."),
            dry_run: true,
            metadata: Value::Null,
        };
        let outcome = coord.execute(request).expect("auto fix execute");
        assert_eq!(outcome.receipt.policy.decision, "review");
        assert!(outcome.receipt.snapshot_path.exists());
    }
}
