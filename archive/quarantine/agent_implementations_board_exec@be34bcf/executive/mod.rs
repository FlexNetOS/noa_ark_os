//! Executive Agents Module
//!
//! Executive-level agents (L2 Reasoning layer)
//! Root CECCA and executive command structure

pub mod emergency;
pub mod noa;
pub mod orchestrator;
pub mod priority;
pub mod resources;

// Re-export for convenience
pub use emergency::EmergencyAgent;
pub use noa::NoaCommander;
pub use orchestrator::OrchestratorAgent;
pub use priority::PriorityAgent;
pub use resources::ResourceAgent;
