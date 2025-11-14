use crate::runtime::RuntimeManager;
use crate::{AgentFactory, AgentLanguage, AgentState, AgentType};
use async_trait::async_trait;
use chrono::Utc;
use serde_json::{Map, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::time::{sleep, Duration};

#[derive(Debug, thiserror::Error)]
pub enum ControllerError {
    #[error("factory error: {0}")]
    Factory(String),
    #[error("runtime error: {0}")]
    Runtime(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("registry error: {0}")]
    Registry(String),
    #[error("metric threshold not met for {metric}: {value} < {expected}")]
    ThresholdFailed {
        metric: String,
        value: f64,
        expected: f64,
    },
}

pub type ControllerResult<T> = Result<T, ControllerError>;

#[derive(Debug, Clone)]
pub struct TrainingRequest {
    pub lifecycle_id: String,
    pub dataset_path: String,
    pub hyperparameters: Value,
    pub agent_profile: String,
}

#[derive(Debug, Clone)]
pub struct TrainingResponse {
    pub agent_id: String,
    pub artifact_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct EvaluationRequest {
    pub agent_id: String,
    pub dataset_path: String,
    pub metric_thresholds: Map<String, Value>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EvaluationReport {
    pub agent_id: String,
    pub metrics: Value,
}

#[derive(Debug, Clone)]
pub enum TrainingStateUpdate {
    Provisioned { agent_id: String },
    Running { agent_id: String },
    ArtifactReady { agent_id: String, path: PathBuf },
}

#[async_trait]
pub trait ModelLifecycleController: Send + Sync {
    async fn schedule_training(
        &self,
        request: TrainingRequest,
    ) -> ControllerResult<TrainingResponse>;
    async fn trigger_evaluation(
        &self,
        request: EvaluationRequest,
    ) -> ControllerResult<EvaluationReport>;
}

pub struct AgentLifecycleController {
    factory: Arc<AgentFactory>,
    runtime: Arc<RuntimeManager>,
    workspace_root: PathBuf,
}

impl AgentLifecycleController {
    pub fn new(
        factory: Arc<AgentFactory>,
        runtime: Arc<RuntimeManager>,
        workspace_root: PathBuf,
    ) -> Self {
        Self {
            factory,
            runtime,
            workspace_root,
        }
    }

    fn derive_agent_name(&self, request: &TrainingRequest) -> String {
        format!("{}_trainer", request.agent_profile.replace(' ', "_"))
    }

    fn training_workspace(&self, agent_id: &str) -> PathBuf {
        self.workspace_root.join(agent_id)
    }

    async fn simulate_training(
        &self,
        agent_id: &str,
        dataset_path: &str,
        hyperparameters: &Value,
    ) -> ControllerResult<PathBuf> {
        // Simulate training latency to mirror asynchronous execution.
        sleep(Duration::from_millis(50)).await;

        let workspace = self.training_workspace(agent_id);
        fs::create_dir_all(&workspace).await?;
        let artifact_path = workspace.join("model.bin");
        let mut file = fs::File::create(&artifact_path).await?;
        let payload = serde_json::json!({
            "agent_id": agent_id,
            "dataset": dataset_path,
            "hyperparameters": hyperparameters,
            "completed_at": Utc::now(),
        });
        file.write_all(payload.to_string().as_bytes()).await?;
        Ok(artifact_path)
    }

    fn runtime_language(&self, request: &TrainingRequest) -> AgentLanguage {
        match request
            .hyperparameters
            .get("runtime_language")
            .and_then(|value| value.as_str())
        {
            Some("rust") => AgentLanguage::Rust,
            Some("go") => AgentLanguage::Go,
            _ => AgentLanguage::Python,
        }
    }

    fn validate_thresholds(
        &self,
        thresholds: &Map<String, Value>,
        metrics: &Map<String, Value>,
    ) -> ControllerResult<()> {
        for (metric, expected) in thresholds {
            if let (Some(expected_value), Some(actual_value)) = (
                expected.as_f64(),
                metrics.get(metric).and_then(|value| value.as_f64()),
            ) {
                if actual_value < expected_value {
                    return Err(ControllerError::ThresholdFailed {
                        metric: metric.clone(),
                        value: actual_value,
                        expected: expected_value,
                    });
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ModelLifecycleController for AgentLifecycleController {
    async fn schedule_training(
        &self,
        request: TrainingRequest,
    ) -> ControllerResult<TrainingResponse> {
        let name = self.derive_agent_name(&request);
        let language = self.runtime_language(&request);
        let agent_id = self
            .factory
            .create_agent(name, AgentType::Worker, language.clone(), false)
            .map_err(|err| ControllerError::Factory(err.to_string()))?;

        self.runtime.register(agent_id.clone(), language.clone());
        self.factory
            .update_state(&agent_id, AgentState::Running)
            .map_err(|err| ControllerError::Factory(err.to_string()))?;

        let code = format!(
            "train(model='{}', dataset='{}')",
            request.lifecycle_id, request.dataset_path
        );
        self.runtime
            .execute(&agent_id, &code)
            .map_err(|err| ControllerError::Runtime(err))?;

        let artifact_path = self
            .simulate_training(&agent_id, &request.dataset_path, &request.hyperparameters)
            .await?;

        self.factory
            .update_state(&agent_id, AgentState::Ready)
            .map_err(|err| ControllerError::Factory(err.to_string()))?;

        Ok(TrainingResponse {
            agent_id,
            artifact_path,
        })
    }

    async fn trigger_evaluation(
        &self,
        request: EvaluationRequest,
    ) -> ControllerResult<EvaluationReport> {
        let code = format!(
            "evaluate(agent='{}', dataset='{}')",
            request.agent_id, request.dataset_path
        );
        self.factory
            .update_state(&request.agent_id, AgentState::Running)
            .map_err(|err| ControllerError::Factory(err.to_string()))?;

        self.runtime
            .execute(&request.agent_id, &code)
            .map_err(|err| ControllerError::Runtime(err))?;

        let metrics = serde_json::json!({
            "accuracy": 0.92,
            "loss": 0.12,
            "notes": request.notes,
        });
        let metrics_map = metrics.as_object().cloned().unwrap_or_else(Map::new);
        self.validate_thresholds(&request.metric_thresholds, &metrics_map)?;

        self.factory
            .update_state(&request.agent_id, AgentState::Ready)
            .map_err(|err| ControllerError::Factory(err.to_string()))?;

        Ok(EvaluationReport {
            agent_id: request.agent_id,
            metrics,
        })
    }
}
