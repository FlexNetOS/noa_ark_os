//! Automated training and evaluation orchestration runtime.
//!
//! This crate coordinates model training pipelines driven by NOA ARK OS agents.
//!
//! The orchestrator consumes training plans, provisions specialist agents,
//! and persists resulting artifacts via the storage gateways while recording
//! registry metadata and verification evidence.

pub mod config;
pub mod orchestrator;
pub mod pipeline;
pub mod storage;

pub use orchestrator::{bootstrap_default_orchestrator, LifecycleEvent, TrainingOrchestrator};
