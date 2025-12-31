use crate::{AdaptiveScalingPolicy, ScalingDecision};
use tracing::info;

/// Coordinates adaptive-scaling guidance for the unified server runtime.
///
/// The orchestrator currently focuses on translating telemetry-driven
/// decisions into actionable hints for the surrounding gateway and workflow
/// layers. It will be extended with task scheduling hooks once the rest of
/// the server workspace stabilises.
#[derive(Debug, Clone)]
pub struct UnifiedOrchestrator {
    scaling_policy: AdaptiveScalingPolicy,
}

impl UnifiedOrchestrator {
    /// Create a new orchestrator with the provided scaling policy.
    pub fn new(scaling_policy: AdaptiveScalingPolicy) -> Self {
        Self { scaling_policy }
    }

    /// Instantiate the orchestrator with the default scaling policy.
    pub fn default() -> Self {
        Self::new(AdaptiveScalingPolicy::default())
    }

    /// Read the currently configured scaling policy.
    pub fn scaling_policy(&self) -> &AdaptiveScalingPolicy {
        &self.scaling_policy
    }

    /// Produce the most recent scaling decision and emit a trace for
    /// downstream subscribers. The decision is returned so callers can wire
    /// it into workflow or gateway coordination logic.
    pub fn evaluate_scaling(&self) -> ScalingDecision {
        let decision = self.scaling_policy.evaluate();
        info!(
            agent_limit = decision.agent_concurrency_limit,
            inference_mode = ?decision.inference_mode,
            delay_ms = decision.sandbox_scheduling_delay_ms,
            "computed adaptive scaling decision"
        );
        decision
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn propagates_policy_configuration() {
        let orchestrator = UnifiedOrchestrator::default();
        let decision = orchestrator.evaluate_scaling();
        assert_eq!(
            decision.agent_concurrency_limit,
            orchestrator.scaling_policy().baseline_concurrency
        );
    }
}
