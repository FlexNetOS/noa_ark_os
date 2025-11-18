use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{OnceLock, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_HISTORY: usize = 32;

fn registry() -> &'static RwLock<TelemetryRegistry> {
    static REGISTRY: OnceLock<RwLock<TelemetryRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| RwLock::new(TelemetryRegistry::default()))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoadLevel {
    Idle,
    Steady,
    Elevated,
    Saturated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySnapshot {
    pub timestamp: u128,
    pub cpu_utilisation: f32,
    pub memory_utilisation: f32,
    pub agent_concurrency: u32,
    pub inference_queue_depth: u32,
    pub sandbox_queue_depth: u32,
}

impl TelemetrySnapshot {
    pub fn now(
        cpu_utilisation: f32,
        memory_utilisation: f32,
        agent_concurrency: u32,
        inference_queue_depth: u32,
        sandbox_queue_depth: u32,
    ) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            cpu_utilisation,
            memory_utilisation,
            agent_concurrency,
            inference_queue_depth,
            sandbox_queue_depth,
        }
    }

    pub fn load_level(&self) -> LoadLevel {
        derive_load_level(
            self.cpu_utilisation,
            self.memory_utilisation,
            self.inference_queue_depth as f32,
            self.sandbox_queue_depth as f32,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedTelemetry {
    pub recent: TelemetrySnapshot,
    pub rolling_cpu_utilisation: f32,
    pub rolling_memory_utilisation: f32,
    pub rolling_agent_concurrency: f32,
    pub rolling_inference_queue_depth: f32,
    pub rolling_sandbox_queue_depth: f32,
    pub load_level: LoadLevel,
}

#[derive(Default)]
struct TelemetryRegistry {
    history: VecDeque<TelemetrySnapshot>,
}

impl TelemetryRegistry {
    fn record(&mut self, snapshot: TelemetrySnapshot) {
        if self.history.len() >= MAX_HISTORY {
            self.history.pop_front();
        }
        self.history.push_back(snapshot);
    }

    fn latest(&self) -> Option<TelemetrySnapshot> {
        self.history.back().cloned()
    }

    fn averages(&self) -> Option<(f32, f32, f32, f32, f32)> {
        if self.history.is_empty() {
            return None;
        }

        let len = self.history.len() as f64;
        let mut cpu = 0.0f64;
        let mut memory = 0.0f64;
        let mut concurrency = 0.0f64;
        let mut inference_queue = 0.0f64;
        let mut sandbox_queue = 0.0f64;

        for snapshot in &self.history {
            cpu += snapshot.cpu_utilisation as f64;
            memory += snapshot.memory_utilisation as f64;
            concurrency += snapshot.agent_concurrency as f64;
            inference_queue += snapshot.inference_queue_depth as f64;
            sandbox_queue += snapshot.sandbox_queue_depth as f64;
        }

        Some((
            (cpu / len) as f32,
            (memory / len) as f32,
            (concurrency / len) as f32,
            (inference_queue / len) as f32,
            (sandbox_queue / len) as f32,
        ))
    }

    fn aggregated(&self) -> Option<AggregatedTelemetry> {
        let recent = self.latest()?;
        let (cpu, memory, concurrency, inference, sandbox) =
            self.averages().unwrap_or((0.0, 0.0, 0.0, 0.0, 0.0));

        Some(AggregatedTelemetry {
            recent,
            rolling_cpu_utilisation: cpu,
            rolling_memory_utilisation: memory,
            rolling_agent_concurrency: concurrency,
            rolling_inference_queue_depth: inference,
            rolling_sandbox_queue_depth: sandbox,
            load_level: derive_load_level(cpu, memory, inference, sandbox),
        })
    }

    #[cfg(test)]
    fn clear(&mut self) {
        self.history.clear();
    }
}

fn derive_load_level(cpu: f32, memory: f32, inference_queue: f32, sandbox_queue: f32) -> LoadLevel {
    let queue_pressure = inference_queue.max(sandbox_queue);

    if cpu >= 0.93 || memory >= 0.93 || queue_pressure >= 96.0 {
        LoadLevel::Saturated
    } else if cpu >= 0.82 || memory >= 0.85 || queue_pressure >= 64.0 {
        LoadLevel::Elevated
    } else if cpu >= 0.65 || memory >= 0.7 || queue_pressure >= 32.0 {
        LoadLevel::Steady
    } else {
        LoadLevel::Idle
    }
}

pub fn record(snapshot: TelemetrySnapshot) {
    let mut registry = registry().write().expect("metrics registry lock poisoned");
    registry.record(snapshot);
}

pub fn current_snapshot() -> Option<TelemetrySnapshot> {
    let registry = registry().read().expect("metrics registry lock poisoned");
    registry.latest()
}

pub fn aggregated() -> Option<AggregatedTelemetry> {
    let registry = registry().read().expect("metrics registry lock poisoned");
    registry.aggregated()
}

pub fn current_load_level() -> LoadLevel {
    aggregated()
        .map(|agg| agg.load_level)
        .unwrap_or(LoadLevel::Idle)
}

#[cfg(test)]
pub fn reset() {
    let mut registry = registry().write().expect("metrics registry lock poisoned");
    registry.clear();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};

    fn test_guard() -> std::sync::MutexGuard<'static, ()> {
        static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();
        TEST_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .expect("metrics test mutex poisoned")
    }

    #[test]
    fn load_levels_follow_thresholds() {
        let _guard = test_guard();
        reset();
        record(TelemetrySnapshot::now(0.20, 0.25, 4, 4, 4));
        assert_eq!(current_load_level(), LoadLevel::Idle);

        reset();
        record(TelemetrySnapshot::now(0.70, 0.60, 12, 20, 18));
        assert_eq!(current_load_level(), LoadLevel::Steady);

        reset();
        record(TelemetrySnapshot::now(0.86, 0.80, 16, 40, 50));
        assert_eq!(current_load_level(), LoadLevel::Elevated);

        reset();
        record(TelemetrySnapshot::now(0.95, 0.96, 24, 110, 40));
        assert_eq!(current_load_level(), LoadLevel::Saturated);
    }

    #[test]
    fn aggregated_returns_recent_snapshot() {
        let _guard = test_guard();
        reset();
        let s1 = TelemetrySnapshot::now(0.20, 0.20, 4, 4, 4);
        let s2 = TelemetrySnapshot::now(0.40, 0.35, 6, 6, 6);
        record(s1.clone());
        record(s2.clone());

        let aggregated = aggregated().expect("expected aggregated telemetry");
        assert_eq!(aggregated.recent.timestamp, s2.timestamp);
        let expected_avg = (s1.cpu_utilisation + s2.cpu_utilisation) / 2.0;
        let tolerance = 1e-6;
        assert!((aggregated.rolling_cpu_utilisation - expected_avg).abs() <= tolerance);
        assert!(
            (aggregated.rolling_cpu_utilisation - 0.30).abs() <= tolerance,
            "unexpected rolling cpu utilisation: {}",
            aggregated.rolling_cpu_utilisation
        );
    }
}
