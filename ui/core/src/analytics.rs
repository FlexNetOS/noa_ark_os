use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::state::GlobalStore;

/// Represents KPI metrics aggregated across modules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metric {
    pub id: String,
    pub label: String,
    pub value: f64,
    pub unit: String,
}

/// Usage intensity at a specific surface area of the interface.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeatmapPoint {
    pub area: String,
    pub intensity: f64,
}

/// Agent efficiency measurement for value tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentEfficiency {
    pub agent_id: String,
    pub utilization: f64,
    pub impact_score: f64,
}

/// ROI analysis for deployed foundation models.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelRoi {
    pub model: String,
    pub generated_value: f64,
    pub operational_cost: f64,
}

/// Aggregated telemetry insights layered on top of base metrics.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TelemetryInsights {
    pub usage_heatmap: Vec<HeatmapPoint>,
    pub agent_efficiency: Vec<AgentEfficiency>,
    pub model_roi: Vec<ModelRoi>,
}

/// Analytics engine to compute product value metrics.
#[derive(Default)]
pub struct AnalyticsEngine {
    pub metrics: HashMap<String, Metric>,
    pub insights: TelemetryInsights,
}

impl AnalyticsEngine {
    pub fn ingest(&mut self, metric: Metric) {
        self.metrics.insert(metric.id.clone(), metric);
    }

    pub fn compute_roi(&self) -> Option<f64> {
        let productivity = self.metrics.get("developer_productivity")?.value;
        let infrastructure = self.metrics.get("infrastructure_cost")?.value;
        if infrastructure == 0.0 {
            return None;
        }
        Some(productivity / infrastructure)
    }

    pub fn layer_insights(&mut self, insights: TelemetryInsights) {
        self.insights = insights;
    }

    pub fn export(&self) -> Vec<Metric> {
        self.metrics.values().cloned().collect()
    }

    pub fn sync_to_state(&self, store: &GlobalStore) {
        store.put_data(
            "analytics.metrics",
            serde_json::json!(self.metrics.values().collect::<Vec<_>>()),
        );
        store.put_data("analytics.insights", serde_json::json!(&self.insights));
        if let Some(roi) = self.compute_roi() {
            store.put_data("analytics.roi", serde_json::json!({ "ratio": roi }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::GlobalState;

    #[test]
    fn roi_calculation_works() {
        let mut engine = AnalyticsEngine::default();
        engine.ingest(Metric {
            id: "developer_productivity".into(),
            label: "Developer Productivity".into(),
            value: 120.0,
            unit: "story points".into(),
        });
        engine.ingest(Metric {
            id: "infrastructure_cost".into(),
            label: "Infrastructure Cost".into(),
            value: 40.0,
            unit: "credits".into(),
        });

        assert_eq!(engine.compute_roi(), Some(3.0));

        let store = GlobalStore::new(GlobalState::default());
        engine.sync_to_state(&store);
        let snapshot = store.read();
        assert!(snapshot.data.contains_key("analytics.metrics"));
        assert!(snapshot.data.contains_key("analytics.roi"));
    }

    #[test]
    fn layered_insights_are_exported() {
        let mut engine = AnalyticsEngine::default();
        engine.layer_insights(TelemetryInsights {
            usage_heatmap: vec![HeatmapPoint {
                area: "workflow.canvas".into(),
                intensity: 0.82,
            }],
            agent_efficiency: vec![AgentEfficiency {
                agent_id: "deploy-coordinator".into(),
                utilization: 0.91,
                impact_score: 8.7,
            }],
            model_roi: vec![ModelRoi {
                model: "gpt-ops".into(),
                generated_value: 122_000.0,
                operational_cost: 34_000.0,
            }],
        });

        let store = GlobalStore::new(GlobalState::default());
        engine.sync_to_state(&store);
        let snapshot = store.read();
        assert!(snapshot.data.contains_key("analytics.insights"));
    }
}
