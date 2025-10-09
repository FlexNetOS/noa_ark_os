//! Agent Implementations Module
//! Contains actual agent code from agentaskit drop

pub mod board;
pub mod executive;
pub mod specialist;
pub mod micro;
pub mod orchestrator;
pub mod model_selector;

// Auto-generated agents (862 agents from registry)
pub mod generated;

// Re-export for convenience
pub use board::*;
pub use executive::*;
pub use specialist::*;
pub use micro::*;
pub use orchestrator::*;
pub use model_selector::*;
pub use generated::*;
