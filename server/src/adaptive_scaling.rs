//! Adaptive service scaling primitives for the single-host profile.
//!
//! The controller inspects telemetry produced by `core::metrics` and
//! derives safe operating points for the colocated gateway, MCP server,
//! workflow orchestrator, and sandbox controllers. When pressure rises,
//! concurrency is reduced and inference falls back to lightweight modes.

use noa_core::metrics::{self, AggregatedTelemetry, LoadLevel};

/// Inference operating modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InferenceMode {
    Full,
    Hybrid,
    Lightweight,
}

/// Scaling decision emitted by [`AdaptiveScalingPolicy::evaluate`].
#[derive(Debug, Clone, PartialEq)]
pub struct ScalingDecision {
    pub agent_concurrency_limit: u32,
    pub inference_mode: InferenceMode,
    pub sandbox_scheduling_delay_ms: u64,
    pub notes: String,
}

/// Configuration for adaptive scaling behaviour.
#[derive(Debug, Clone)]
pub struct AdaptiveScalingPolicy {
    pub min_concurrency: u32,
    pub baseline_concurrency: u32,
    pub max_concurrency: u32,
    pub elevated_backoff_ms: u64,
    pub saturated_backoff_ms: u64,
}

impl Default for AdaptiveScalingPolicy {
    fn default() -> Self {
        Self {
            min_concurrency: 2,
            baseline_concurrency: 8,
            max_concurrency: 16,
            elevated_backoff_ms: 250,
            saturated_backoff_ms: 750,
        }
    }
}

impl AdaptiveScalingPolicy {
    pub fn with_baseline(mut self, baseline_concurrency: u32) -> Self {
        self.baseline_concurrency = baseline_concurrency;
        self
    }

    /// Compute a scaling decision based on the most recent aggregated telemetry.
    pub fn evaluate(&self) -> ScalingDecision {
        let telemetry = metrics::aggregated();
        self.evaluate_with_telemetry(telemetry)
    }

    fn evaluate_with_telemetry(
        &self,
        telemetry: Option<AggregatedTelemetry>,
    ) -> ScalingDecision {
        let default_notes = "Telemetry unavailable; defaulting to baseline configuration".to_string();
        let telemetry = match telemetry {
            Some(telemetry) => telemetry,
            None => {
                return ScalingDecision {
                    agent_concurrency_limit: self.baseline_concurrency,
                    inference_mode: InferenceMode::Full,
                    sandbox_scheduling_delay_ms: 0,
                    notes: default_notes,
                }
            }
        };

        let (limit, mode, delay, notes) = match telemetry.load_level {
            LoadLevel::Idle => (
                self.max_concurrency,
                InferenceMode::Full,
                0,
                format!(
                    "Idle load (cpu={:.2}, mem={:.2}) — resuming full capacity",
                    telemetry.rolling_cpu_utilisation, telemetry.rolling_memory_utilisation
                ),
            ),
            LoadLevel::Steady => (
                self.baseline_concurrency,
                InferenceMode::Full,
                0,
                format!(
                    "Steady load (avg queue {:.1}) — maintaining baseline",
                    telemetry
                        .rolling_inference_queue_depth
                        .max(telemetry.rolling_sandbox_queue_depth)
                ),
            ),
            LoadLevel::Elevated => (
                self
                    .baseline_concurrency
                    .saturating_sub(self.baseline_concurrency / 3)
                    .max(self.min_concurrency),
                InferenceMode::Hybrid,
                self.elevated_backoff_ms,
                format!(
                    "Elevated load (cpu={:.2}, mem={:.2}) — throttling agents",
                    telemetry.rolling_cpu_utilisation, telemetry.rolling_memory_utilisation
                ),
            ),
            LoadLevel::Saturated => (
                self.min_concurrency,
                InferenceMode::Lightweight,
                self.saturated_backoff_ms,
                format!(
                    "Saturated (queue {:.0}) — enabling lightweight inference",
                    telemetry
                        .rolling_inference_queue_depth
                        .max(telemetry.rolling_sandbox_queue_depth)
                ),
            ),
        };

        ScalingDecision {
            agent_concurrency_limit: limit,
            inference_mode: mode,
            sandbox_scheduling_delay_ms: delay,
            notes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noa_core::metrics::TelemetrySnapshot;

    fn telemetry(
        cpu: f32,
        mem: f32,
        concurrency: u32,
        inference_queue: u32,
        sandbox_queue: u32,
    ) -> AggregatedTelemetry {
        let snapshot = TelemetrySnapshot::now(cpu, mem, concurrency, inference_queue, sandbox_queue);
        AggregatedTelemetry {
            recent: snapshot.clone(),
            rolling_cpu_utilisation: cpu,
            rolling_memory_utilisation: mem,
            rolling_agent_concurrency: concurrency as f32,
            rolling_inference_queue_depth: inference_queue as f32,
            rolling_sandbox_queue_depth: sandbox_queue as f32,
            load_level: snapshot.load_level(),
        }
    }

    #[test]
    fn uses_baseline_when_no_telemetry() {
        let policy = AdaptiveScalingPolicy::default();
        let decision = policy.evaluate_with_telemetry(None);
        assert_eq!(decision.agent_concurrency_limit, policy.baseline_concurrency);
        assert_eq!(decision.inference_mode, InferenceMode::Full);
    }

    #[test]
    fn escalates_to_hybrid_and_lightweight_modes() {
        let policy = AdaptiveScalingPolicy::default();

        let elevated = policy.evaluate_with_telemetry(Some(telemetry(0.84, 0.81, 16, 80, 44)));
        assert_eq!(elevated.inference_mode, InferenceMode::Hybrid);
        assert!(elevated.agent_concurrency_limit <= policy.baseline_concurrency);
        assert!(elevated.sandbox_scheduling_delay_ms >= policy.elevated_backoff_ms);

        let saturated = policy.evaluate_with_telemetry(Some(telemetry(0.95, 0.94, 16, 120, 120)));
        assert_eq!(saturated.inference_mode, InferenceMode::Lightweight);
        assert_eq!(saturated.agent_concurrency_limit, policy.min_concurrency);
        assert!(saturated.sandbox_scheduling_delay_ms >= policy.saturated_backoff_ms);
    }

    #[test]
    fn publishes_reasonable_notes() {
        let policy = AdaptiveScalingPolicy::default();
        let decision = policy.evaluate_with_telemetry(Some(telemetry(0.20, 0.25, 4, 4, 4)));
        assert!(decision.notes.contains("Idle"));
    }
}
