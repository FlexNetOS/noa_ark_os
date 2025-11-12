use crate::inference::{InferenceConfig, InferenceEngine};
use crate::{AgentId, AgentMetadata, Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Model information for selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub file_path: String,
    pub size_mb: usize,
    pub capabilities: Vec<String>,
    pub performance_score: f32,
    pub cost_score: f32,
    pub privacy_tier: PrivacyTier,
    pub use_cases: Vec<UseCase>,
}

/// Privacy tiers for model selection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrivacyTier {
    Public,       // Can use external APIs
    Internal,     // Internal data only
    Confidential, // Highly sensitive, local only
    Restricted,   // Maximum security
}

/// Use cases for model specialization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UseCase {
    CodeGeneration,
    CodeAnalysis,
    Reasoning,
    QuestionAnswering,
    Documentation,
    Translation,
    Summarization,
    AgentTask,
    FunctionCalling,
    Vision,
    General,
}

/// Task requirements for model selection
#[derive(Debug, Clone)]
pub struct TaskRequirements {
    pub use_case: UseCase,
    pub privacy_tier: PrivacyTier,
    pub max_latency_ms: Option<u64>,
    pub max_cost: Option<f32>,
    pub min_quality: f32,
    pub context_size: Option<usize>,
}

/// Model selection result
#[derive(Debug, Clone)]
pub struct ModelSelection {
    pub model: ModelInfo,
    pub confidence: f32,
    pub rationale: String,
    pub alternatives: Vec<ModelInfo>,
}

/// ModelSelector Agent - Selects optimal model for each task
pub struct ModelSelectorAgent {
    metadata: AgentMetadata,
    models: Arc<RwLock<HashMap<String, ModelInfo>>>,
    usage_stats: Arc<RwLock<HashMap<String, UsageStats>>>,
}

#[derive(Debug, Clone)]
struct UsageStats {
    total_uses: usize,
    successes: usize,
    failures: usize,
    avg_latency_ms: f32,
    avg_quality_score: f32,
}

impl ModelSelectorAgent {
    pub fn new() -> Self {
        let metadata = AgentMetadata::new(
            "ModelSelector".to_string(),
            "Intelligent model selection agent".to_string(),
            "Orchestration".to_string(),
        );

        Self {
            metadata,
            models: Arc::new(RwLock::new(HashMap::new())),
            usage_stats: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a model for selection
    pub fn register_model(&self, model: ModelInfo) -> Result<()> {
        let mut models = self.models.write().unwrap();
        models.insert(model.name.clone(), model);
        Ok(())
    }

    /// Select optimal model for task
    pub fn select_model(&self, requirements: TaskRequirements) -> Result<ModelSelection> {
        let models = self.models.read().unwrap();

        if models.is_empty() {
            return Err(Error::AgentError("No models registered".to_string()));
        }

        // Filter models by privacy tier
        let mut candidates: Vec<_> = models
            .values()
            .filter(|m| self.matches_privacy_tier(m, &requirements.privacy_tier))
            .cloned()
            .collect();

        if candidates.is_empty() {
            return Err(Error::AgentError(format!(
                "No models match privacy tier: {:?}",
                requirements.privacy_tier
            )));
        }

        // Score each candidate
        let mut scored: Vec<(ModelInfo, f32, String)> = candidates
            .iter()
            .map(|model| {
                let (score, rationale) = self.score_model(model, &requirements);
                (model.clone(), score, rationale)
            })
            .collect();

        // Sort by score (highest first)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let best = scored.first().unwrap();
        let alternatives: Vec<ModelInfo> = scored
            .iter()
            .skip(1)
            .take(3)
            .map(|(m, _, _)| m.clone())
            .collect();

        Ok(ModelSelection {
            model: best.0.clone(),
            confidence: best.1,
            rationale: best.2.clone(),
            alternatives,
        })
    }

    /// Record usage statistics for learning
    pub fn record_usage(
        &self,
        model_name: &str,
        success: bool,
        latency_ms: u64,
        quality_score: f32,
    ) {
        let mut stats = self.usage_stats.write().unwrap();

        let entry = stats.entry(model_name.to_string()).or_insert(UsageStats {
            total_uses: 0,
            successes: 0,
            failures: 0,
            avg_latency_ms: 0.0,
            avg_quality_score: 0.0,
        });

        entry.total_uses += 1;
        if success {
            entry.successes += 1;
        } else {
            entry.failures += 1;
        }

        // Update running averages
        let alpha = 0.1; // Learning rate
        entry.avg_latency_ms = entry.avg_latency_ms * (1.0 - alpha) + latency_ms as f32 * alpha;
        entry.avg_quality_score = entry.avg_quality_score * (1.0 - alpha) + quality_score * alpha;
    }

    /// List all registered models
    pub fn list_models(&self) -> Vec<ModelInfo> {
        let models = self.models.read().unwrap();
        models.values().cloned().collect()
    }

    /// Get usage statistics
    pub fn get_stats(&self, model_name: &str) -> Option<UsageStats> {
        let stats = self.usage_stats.read().unwrap();
        stats.get(model_name).cloned()
    }

    // Private helper methods

    fn matches_privacy_tier(&self, model: &ModelInfo, required: &PrivacyTier) -> bool {
        match required {
            PrivacyTier::Restricted => model.privacy_tier == PrivacyTier::Restricted,
            PrivacyTier::Confidential => {
                matches!(
                    model.privacy_tier,
                    PrivacyTier::Confidential | PrivacyTier::Restricted
                )
            }
            PrivacyTier::Internal => {
                matches!(
                    model.privacy_tier,
                    PrivacyTier::Internal | PrivacyTier::Confidential | PrivacyTier::Restricted
                )
            }
            PrivacyTier::Public => true, // All models acceptable
        }
    }

    fn score_model(&self, model: &ModelInfo, requirements: &TaskRequirements) -> (f32, String) {
        let mut score = 0.0;
        let mut rationale = Vec::new();

        // Use case match (40% weight)
        if model.use_cases.contains(&requirements.use_case) {
            score += 0.4;
            rationale.push(format!("Specialized for {:?}", requirements.use_case));
        } else if model.use_cases.contains(&UseCase::General) {
            score += 0.2;
            rationale.push("General purpose model".to_string());
        }

        // Performance score (30% weight)
        score += model.performance_score * 0.3;
        rationale.push(format!("Performance score: {:.2}", model.performance_score));

        // Cost efficiency (15% weight)
        score += model.cost_score * 0.15;
        rationale.push(format!("Cost score: {:.2}", model.cost_score));

        // Historical performance (15% weight)
        if let Some(stats) = self.get_stats(&model.name) {
            let success_rate = stats.successes as f32 / stats.total_uses as f32;
            score += success_rate * 0.15;
            rationale.push(format!(
                "Historical success rate: {:.1}%",
                success_rate * 100.0
            ));
        }

        // Penalties for not meeting requirements
        if let Some(max_cost) = requirements.max_cost {
            if model.cost_score > max_cost {
                score *= 0.5; // Heavy penalty
                rationale.push("PENALTY: Exceeds cost limit".to_string());
            }
        }

        if model.performance_score < requirements.min_quality {
            score *= 0.7;
            rationale.push("PENALTY: Below quality threshold".to_string());
        }

        (score, rationale.join("; "))
    }
}

impl Default for ModelSelectorAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registration() {
        let selector = ModelSelectorAgent::new();

        let model = ModelInfo {
            name: "test-model".to_string(),
            file_path: "/models/test.gguf".to_string(),
            size_mb: 2000,
            capabilities: vec!["code".to_string()],
            performance_score: 0.8,
            cost_score: 0.7,
            privacy_tier: PrivacyTier::Internal,
            use_cases: vec![UseCase::CodeGeneration],
        };

        selector.register_model(model).unwrap();
        assert_eq!(selector.list_models().len(), 1);
    }

    #[test]
    fn test_model_selection() {
        let selector = ModelSelectorAgent::new();

        // Register multiple models
        let model1 = ModelInfo {
            name: "code-expert".to_string(),
            file_path: "/models/code.gguf".to_string(),
            size_mb: 7000,
            capabilities: vec!["code".to_string()],
            performance_score: 0.9,
            cost_score: 0.6,
            privacy_tier: PrivacyTier::Internal,
            use_cases: vec![UseCase::CodeGeneration, UseCase::CodeAnalysis],
        };

        let model2 = ModelInfo {
            name: "general".to_string(),
            file_path: "/models/general.gguf".to_string(),
            size_mb: 3000,
            capabilities: vec!["general".to_string()],
            performance_score: 0.7,
            cost_score: 0.9,
            privacy_tier: PrivacyTier::Public,
            use_cases: vec![UseCase::General],
        };

        selector.register_model(model1).unwrap();
        selector.register_model(model2).unwrap();

        // Select for code generation task
        let requirements = TaskRequirements {
            use_case: UseCase::CodeGeneration,
            privacy_tier: PrivacyTier::Internal,
            max_latency_ms: None,
            max_cost: None,
            min_quality: 0.6,
            context_size: None,
        };

        let selection = selector.select_model(requirements).unwrap();
        assert_eq!(selection.model.name, "code-expert");
    }
}
