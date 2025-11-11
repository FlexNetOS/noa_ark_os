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

/// Analytics engine to compute product value metrics.
#[derive(Default)]
pub struct AnalyticsEngine {
    pub metrics: HashMap<String, Metric>,
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

    pub fn export(&self) -> Vec<Metric> {
        self.metrics.values().cloned().collect()
    }

    pub fn sync_to_state(&self, store: &GlobalStore) {
        store.put_data(
            "analytics.metrics",
            serde_json::json!(self.metrics.values().collect::<Vec<_>>()),
        );
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
        assert!(store.read().data.contains_key("analytics.metrics"));
    }
}
