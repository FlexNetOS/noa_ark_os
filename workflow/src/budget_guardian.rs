use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::instrumentation::{BudgetDecisionRecord, PipelineInstrumentation};
use crate::Stage;

const DEFAULT_SAMPLE_SIZE: usize = 50;
const DEFAULT_TELEMETRY_PATH: &str = "storage/telemetry/gateway_events.log";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetLimits {
    pub max_tokens: f64,
    pub max_latency_ms: f64,
}

impl Default for BudgetLimits {
    fn default() -> Self {
        Self {
            max_tokens: 2_000.0,
            max_latency_ms: 1_200.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BudgetUsage {
    pub tokens: f64,
    pub average_latency_ms: f64,
    pub samples: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BudgetAction {
    Proceed,
    RewritePlan,
    Escalate,
}

impl BudgetAction {
    fn as_str(&self) -> &'static str {
        match self {
            BudgetAction::Proceed => "proceed",
            BudgetAction::RewritePlan => "rewrite",
            BudgetAction::Escalate => "escalate",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetDecision {
    pub action: BudgetAction,
    pub usage: BudgetUsage,
    pub limits: BudgetLimits,
    pub stage: Stage,
    pub receipt: BudgetDecisionRecord,
}

#[derive(Debug, Error)]
pub enum BudgetGuardianError {
    #[error("telemetry read failed: {0}")]
    TelemetryIo(String),
    #[error("telemetry parse failed: {0}")]
    TelemetryParse(String),
    #[error("instrumentation error: {0}")]
    Instrumentation(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}

pub struct BudgetGuardian {
    instrumentation: Arc<PipelineInstrumentation>,
    limits: BudgetLimits,
    telemetry_path: PathBuf,
    sample_size: usize,
}

impl BudgetGuardian {
    pub fn new(instrumentation: Arc<PipelineInstrumentation>) -> Self {
        Self {
            instrumentation,
            limits: BudgetLimits::default(),
            telemetry_path: PathBuf::from(DEFAULT_TELEMETRY_PATH),
            sample_size: DEFAULT_SAMPLE_SIZE,
        }
    }

    pub fn with_limits(mut self, limits: BudgetLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn with_telemetry_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.telemetry_path = path.into();
        self
    }

    pub fn with_sample_size(mut self, size: usize) -> Self {
        self.sample_size = size.max(1);
        self
    }

    pub fn limits(&self) -> &BudgetLimits {
        &self.limits
    }

    pub fn assess_stage(
        &self,
        workflow_id: &str,
        stage: &Stage,
    ) -> Result<BudgetDecision, BudgetGuardianError> {
        let usage = self.collect_usage()?;
        let mut action = BudgetAction::Proceed;
        let mut resulting_stage = stage.clone();
        let mut rewritten_plan: Option<Value> = None;

        if usage.tokens > self.limits.max_tokens || usage.average_latency_ms > self.limits.max_latency_ms {
            if let Some(rewritten) = self.rewrite_stage(stage) {
                action = BudgetAction::RewritePlan;
                resulting_stage = rewritten.clone();
                rewritten_plan = Some(
                    serde_json::to_value(&rewritten)
                        .map_err(|err| BudgetGuardianError::Serialization(err.to_string()))?,
                );
            } else {
                action = BudgetAction::Escalate;
            }
        }

        let receipt = self
            .instrumentation
            .record_budget_decision(
                workflow_id,
                &stage.name,
                usage.tokens,
                self.limits.max_tokens,
                usage.average_latency_ms,
                self.limits.max_latency_ms,
                action.as_str(),
                rewritten_plan.clone(),
            )
            .map_err(|err| BudgetGuardianError::Instrumentation(err.to_string()))?;

        Ok(BudgetDecision {
            action,
            usage,
            limits: self.limits.clone(),
            stage: resulting_stage,
            receipt,
        })
    }

    fn rewrite_stage(&self, stage: &Stage) -> Option<Stage> {
        let filtered: Vec<_> = stage
            .tasks
            .iter()
            .filter(|task| {
                !task
                    .parameters
                    .get("budget_sensitive")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        if filtered.is_empty() || filtered.len() == stage.tasks.len() {
            None
        } else {
            let mut rewritten = stage.clone();
            rewritten.tasks = filtered;
            Some(rewritten)
        }
    }

    fn collect_usage(&self) -> Result<BudgetUsage, BudgetGuardianError> {
        let mut usage = BudgetUsage::default();
        let path = &self.telemetry_path;
        let Ok(raw) = fs::read_to_string(path) else {
            return Ok(usage);
        };

        for line in raw.lines().rev().take(self.sample_size) {
            if line.trim().is_empty() {
                continue;
            }
            let event: Value = serde_json::from_str(line)
                .map_err(|err| BudgetGuardianError::TelemetryParse(err.to_string()))?;
            self.accumulate_usage(&event, &mut usage);
        }

        if usage.samples > 0 {
            usage.average_latency_ms /= usage.samples as f64;
        }

        Ok(usage)
    }

    fn accumulate_usage(&self, event: &Value, usage: &mut BudgetUsage) {
        if let Some(obj) = event.as_object() {
            if let Some(tokens) = self.extract_token_count(obj) {
                usage.tokens += tokens;
            }
            if let Some(latency) = self.extract_latency(obj) {
                usage.average_latency_ms += latency;
                usage.samples += 1;
            }
            if let Some(span) = obj.get("otel_span").and_then(|value| value.as_object()) {
                if let Some(tokens) = self.extract_token_count(span) {
                    usage.tokens += tokens;
                }
                if let Some(latency) = self.extract_latency(span) {
                    usage.average_latency_ms += latency;
                    usage.samples += 1;
                }
            }
        }
    }

    fn extract_token_count(&self, map: &serde_json::Map<String, Value>) -> Option<f64> {
        // Use find_map to return early on first match for "token" in key
        map.iter().find_map(|(key, value)| {
            if key.to_lowercase().contains("token") {
                numeric(value)
            } else {
                None
            }
        })
    }

    fn extract_latency(&self, map: &serde_json::Map<String, Value>) -> Option<f64> {
        // Use find_map to return early on first match for "latency" or "duration" in key
        map.iter().find_map(|(key, value)| {
            let key_lower = key.to_lowercase();
            if key_lower.contains("latency") || key_lower.contains("duration") {
                numeric(value)
            } else {
                None
            }
        })
    }
}

fn numeric(value: &Value) -> Option<f64> {
    if let Some(num) = value.as_f64() {
        return Some(num);
    }
    if let Some(num) = value.as_i64() {
        return Some(num as f64);
    }
    if let Some(num) = value.as_u64() {
        return Some(num as f64);
    }
    if let Some(text) = value.as_str() {
        return text.parse().ok();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StageType, Task};
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use std::io::Write;
    use std::sync::Arc;
    use tempfile::{NamedTempFile, TempDir};

    use crate::instrumentation::PipelineInstrumentation;

    fn instrumentation_with_temp_root() -> (Arc<PipelineInstrumentation>, TempDir) {
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

        (instrumentation, temp_root)
    }

    fn guardian_with_events(lines: &[Value]) -> (BudgetGuardian, TempDir) {
        let mut temp = NamedTempFile::new().expect("telemetry file");
        for line in lines {
            writeln!(temp, "{}", line).expect("write telemetry");
        }
        let path = temp.into_temp_path();
        let path_buf = path.to_path_buf();
        path.keep().expect("persist telemetry");
        let (instrumentation, temp_root) = instrumentation_with_temp_root();
        let guardian = BudgetGuardian::new(instrumentation)
            .with_telemetry_path(path_buf)
            .with_sample_size(lines.len().max(1));
        (guardian, temp_root)
    }

    fn sample_stage() -> Stage {
        let mut sensitive_params = HashMap::new();
        sensitive_params.insert("budget_sensitive".to_string(), Value::Bool(true));
        let mut normal_params = HashMap::new();
        Stage {
            name: "stage".to_string(),
            stage_type: StageType::Sequential,
            depends_on: vec![],
            tasks: vec![
                Task {
                    agent: "lint".to_string(),
                    action: "lint".to_string(),
                    parameters: sensitive_params,
                    agent_role: None,
                    tool_requirements: vec![],
                },
                Task {
                    agent: "type".to_string(),
                    action: "type".to_string(),
                    parameters: normal_params,
                    agent_role: None,
                    tool_requirements: vec![],
                },
            ],
        }
    }

    #[test]
    fn rewrites_budget_sensitive_tasks_when_threshold_exceeded() {
        let (guardian, _temp_dir) = guardian_with_events(&[
            json!({"otel_span": {"tokens_total": 5000, "latency_ms": 2400}}),
        ]);
        let stage = sample_stage();
        let decision = guardian
            .assess_stage("workflow", &stage)
            .expect("budget decision");
        assert_eq!(decision.action, BudgetAction::RewritePlan);
        assert_eq!(decision.stage.tasks.len(), 1);
        assert!(decision.receipt.snapshot_path.exists());
    }

    #[test]
    fn proceeds_when_usage_within_limits() {
        let (guardian, _temp_dir) = guardian_with_events(&[
            json!({"otel_span": {"tokens_total": 10, "latency_ms": 20}}),
        ]);
        let stage = sample_stage();
        let decision = guardian
            .assess_stage("workflow", &stage)
            .expect("budget decision");
        assert_eq!(decision.action, BudgetAction::Proceed);
        assert_eq!(decision.stage.tasks.len(), stage.tasks.len());
    }
}
