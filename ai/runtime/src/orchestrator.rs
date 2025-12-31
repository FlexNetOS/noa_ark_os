use crate::config::TrainingPlan;
use crate::pipeline::{PipelineReport, TrainingPipeline};
use crate::storage::{
    ArtifactRecord, CapabilityRegistry, FilesystemArtifactStore, ModelArtifactStore,
    RegistryGateway,
};
use agents::implementations::ml_controller::{
    AgentLifecycleController, ControllerResult, ModelLifecycleController,
};
use agents::runtime::RuntimeManager;
use agents::AgentFactory;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Events emitted during the lifecycle of a training pipeline.
#[derive(Debug, Clone)]
pub enum LifecycleEvent {
    Submitted {
        lifecycle_id: String,
    },
    AgentProvisioned {
        lifecycle_id: String,
        agent_id: String,
    },
    TrainingCompleted {
        lifecycle_id: String,
        artifact: ArtifactRecord,
    },
    EvaluationLogged {
        lifecycle_id: String,
        metrics: serde_json::Value,
    },
    PromotionReady {
        lifecycle_id: String,
    },
}

pub struct TrainingOrchestrator<
    C: ModelLifecycleController,
    S: ModelArtifactStore,
    R: RegistryGateway,
> {
    pipeline: TrainingPipeline<C, S, R>,
    notifier: broadcast::Sender<LifecycleEvent>,
}

impl<C, S, R> TrainingOrchestrator<C, S, R>
where
    C: ModelLifecycleController + 'static,
    S: ModelArtifactStore + 'static,
    R: RegistryGateway + 'static,
{
    pub fn new(controller: Arc<C>, store: Arc<S>, registry: Arc<R>) -> Self {
        let pipeline = TrainingPipeline::new(controller, store, registry);
        let (sender, _) = broadcast::channel(128);
        Self {
            pipeline,
            notifier: sender,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<LifecycleEvent> {
        self.notifier.subscribe()
    }

    pub async fn execute_plan(&self, plan: TrainingPlan) -> ControllerResult<PipelineReport> {
        let _ = self.notifier.send(LifecycleEvent::Submitted {
            lifecycle_id: plan.lifecycle_id.clone(),
        });

        let lifecycle_id = plan.lifecycle_id.clone();
        let report = self.pipeline.execute(plan).await?;
        let _ = self.notifier.send(LifecycleEvent::AgentProvisioned {
            lifecycle_id: lifecycle_id.clone(),
            agent_id: report.training_agent_id.clone(),
        });
        let _ = self.notifier.send(LifecycleEvent::TrainingCompleted {
            lifecycle_id: lifecycle_id.clone(),
            artifact: report.artifact.clone(),
        });

        let _ = self.notifier.send(LifecycleEvent::EvaluationLogged {
            lifecycle_id: lifecycle_id.clone(),
            metrics: report.evaluation_metrics.clone(),
        });

        let _ = self.notifier.send(LifecycleEvent::PromotionReady {
            lifecycle_id: lifecycle_id.clone(),
        });

        Ok(report)
    }

    pub fn notifier(&self) -> broadcast::Sender<LifecycleEvent> {
        self.notifier.clone()
    }
}

/// Convenience helper that wires the default agent lifecycle controller into the orchestrator.
pub fn bootstrap_default_orchestrator(
    factory: Arc<AgentFactory>,
    runtime: Arc<RuntimeManager>,
    storage_root: PathBuf,
    ledger_path: PathBuf,
    registry_manifest: PathBuf,
    capability_id: &str,
) -> TrainingOrchestrator<AgentLifecycleController, FilesystemArtifactStore, CapabilityRegistry> {
    let controller = Arc::new(AgentLifecycleController::new(
        factory,
        runtime,
        storage_root.join("workspaces"),
    ));
    let store = Arc::new(FilesystemArtifactStore::new(storage_root, ledger_path));
    let registry = Arc::new(CapabilityRegistry::new(registry_manifest, capability_id));
    TrainingOrchestrator::new(controller, store, registry)
}
