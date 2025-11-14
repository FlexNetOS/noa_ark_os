//! Agent Implementations Module
//! Contains actual agent code from agentaskit drop

pub mod board;
pub mod documentation;
pub mod executive;
pub mod micro;
pub mod ml_controller;
pub mod model_selector;
pub mod orchestrator;
pub mod specialist;

// Auto-generated agents (862 agents from registry)
pub mod generated;

// Re-export for convenience
pub use board::*;
pub use documentation::*;
pub use executive::*;
pub use generated::*;
pub use micro::*;
pub use ml_controller::*;
pub use model_selector::*;
pub use orchestrator::*;
pub use specialist::*;
