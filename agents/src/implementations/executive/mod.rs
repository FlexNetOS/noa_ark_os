//! Executive Agents Module
//! 
//! Executive-level agents (L2 Reasoning layer)
//! Root CECCA and executive command structure

pub mod noa;
pub mod emergency;
pub mod priority;
pub mod resources;
pub mod orchestrator;

// Re-export for convenience
pub use noa::NoaCommander;
pub use emergency::EmergencyAgent;
pub use priority::PriorityAgent;
pub use resources::ResourceAgent;
pub use orchestrator::OrchestratorAgent;
