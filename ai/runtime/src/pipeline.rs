use crate::config::{EvaluationPlan, TrainingPlan};
use crate::storage::{ArtifactRecord, ModelArtifactStore, RegistryGateway};
use agents::implementations::ml_controller::{
    ControllerResult, EvaluationRequest, ModelLifecycleController, TrainingRequest,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;

/// Report summarising the pipeline execution.
#[derive(Debug, Serialize)]
pub struct PipelineReport {
    pub lifecycle_id: String,
    pub training_agent_id: String,
    pub artifact: ArtifactRecord,
    pub evaluation_metrics: serde_json::Value,
    pub completed_at: DateTime<Utc>,
}

/// Training pipeline orchestrated through agents.
pub struct TrainingPipeline<C: ModelLifecycleController, S: ModelArtifactStore, R: RegistryGateway>
{
    controller: Arc<C>,
    store: Arc<S>,
    registry: Arc<R>,
}

impl<C, S, R> TrainingPipeline<C, S, R>
where
    C: ModelLifecycleController,
    S: ModelArtifactStore,
    R: RegistryGateway,
{
    pub fn new(controller: Arc<C>, store: Arc<S>, registry: Arc<R>) -> Self {
        Self {
            controller,
            store,
            registry,
        }
    }

    pub async fn execute(&self, plan: TrainingPlan) -> ControllerResult<PipelineReport> {
        self.registry.ensure_capability_registered()?;

        let training_request = TrainingRequest {
            lifecycle_id: plan.lifecycle_id.clone(),
            dataset_path: plan.dataset_path.clone(),
            hyperparameters: plan.hyperparameters.clone(),
            agent_profile: plan.agent_profile.clone(),
        };

        let training_response = self.controller.schedule_training(training_request).await?;
        let artifact = self
            .store
            .persist_artifact(&training_response.artifact_path, &plan)
            .await?;

        let evaluation_metrics = if let Some(evaluation_plan) = plan.evaluation.clone() {
            self.run_evaluation(training_response.agent_id.clone(), evaluation_plan)
                .await?
        } else {
            serde_json::json!({ "status": "skipped" })
        };

        self.store
            .record_verification(&artifact, &evaluation_metrics)
            .await?;
        self.registry
            .append_metadata(&artifact, &evaluation_metrics)
            .await?;

        Ok(PipelineReport {
            lifecycle_id: plan.lifecycle_id,
            training_agent_id: training_response.agent_id,
            artifact,
            evaluation_metrics,
            completed_at: Utc::now(),
        })
    }

    async fn run_evaluation(
        &self,
        agent_id: String,
        evaluation: EvaluationPlan,
    ) -> ControllerResult<serde_json::Value> {
        let request = EvaluationRequest {
            agent_id,
            dataset_path: evaluation.dataset_path,
            metric_thresholds: evaluation.metric_thresholds,
            notes: evaluation.notes,
        };

        let report = self.controller.trigger_evaluation(request).await?;
        Ok(report.metrics)
    }
}
