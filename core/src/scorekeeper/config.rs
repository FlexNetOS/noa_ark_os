use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NorthStarPolicy {
    pub version: String,
    pub metrics: Vec<MetricDefinition>,
    #[serde(default)]
    pub escalation_policies: Vec<EscalationPolicy>,
    #[serde(default)]
    pub defaults: serde_json::Value,
}

impl NorthStarPolicy {
    pub fn metric(&self, id: &str) -> Option<&MetricDefinition> {
        self.metrics.iter().find(|metric| metric.id == id)
    }

    pub fn escalation(&self, id: &str) -> Option<&EscalationPolicy> {
        self.escalation_policies
            .iter()
            .find(|policy| policy.id == id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    pub id: String,
    pub description: String,
    pub weight: f32,
    pub thresholds: Thresholds,
    #[serde(default)]
    pub escalation_policy: Option<String>,
    #[serde(default)]
    pub signals: Vec<String>,
    #[serde(default)]
    pub scope_reduction: Option<ScopeReduction>,
}

impl MetricDefinition {
    pub fn scope_reduction(&self) -> ScopeReduction {
        self.scope_reduction.clone().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thresholds {
    pub warning: f32,
    pub critical: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationPolicy {
    pub id: String,
    pub summary: String,
    pub actions: Vec<String>,
    pub owners: Vec<String>,
    #[serde(default)]
    pub response_time_minutes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScopeReduction {
    #[serde(default = "default_warning_multiplier")]
    pub warning_multiplier: f32,
    #[serde(default = "default_critical_multiplier")]
    pub critical_multiplier: f32,
    #[serde(default)]
    pub minimum_optional: usize,
}

fn default_warning_multiplier() -> f32 {
    0.75
}

fn default_critical_multiplier() -> f32 {
    0.5
}
