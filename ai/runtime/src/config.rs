use serde::{Deserialize, Serialize};

/// Configuration describing how a model should be trained.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingPlan {
    /// Unique identifier for the model lifecycle.
    pub lifecycle_id: String,
    /// Agent profile to request from the factory.
    pub agent_profile: String,
    /// Path to the dataset definition expected by the training agent.
    pub dataset_path: String,
    /// Model hyperparameters expressed as arbitrary JSON.
    #[serde(default)]
    pub hyperparameters: serde_json::Value,
    /// Optional evaluation plan to execute immediately after training.
    #[serde(default)]
    pub evaluation: Option<EvaluationPlan>,
}

/// Definition of an evaluation stage executed by verifiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationPlan {
    /// Dataset used for evaluation.
    pub dataset_path: String,
    /// Metric targets that must be met before promotion.
    #[serde(default)]
    pub metric_thresholds: serde_json::Map<String, serde_json::Value>,
    /// Optional notes surfaced to the evidence ledger.
    #[serde(default)]
    pub notes: Option<String>,
}

impl TrainingPlan {
    /// Produce a short tag combining lifecycle and profile for logging.
    pub fn tag(&self) -> String {
        format!("{}::{}", self.lifecycle_id, self.agent_profile)
    }
}
