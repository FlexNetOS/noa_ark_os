//! Runtime coordination utilities for the NOA unified server.
//!
//! The adaptive scaling policy consumes telemetry emitted by `noa_core`
//! and decides how aggressively colocated services should operate on a
//! single host deployment profile.

pub mod adaptive_scaling;
pub mod orchestrator;

pub use adaptive_scaling::{AdaptiveScalingPolicy, InferenceMode, ScalingDecision};
pub use orchestrator::UnifiedOrchestrator;
