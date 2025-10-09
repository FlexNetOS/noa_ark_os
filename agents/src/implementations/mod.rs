// Agent Implementations Module
// Contains actual agent code from agentaskit drop

pub mod board;
#[cfg(feature = "agentaskit-executive")]
pub mod executive;
pub mod specialist;
pub mod micro;
pub mod orchestrator;
pub mod model_selector;

// Re-export for convenience
pub use board::*;
#[cfg(feature = "agentaskit-executive")]
pub use executive::*;
pub use specialist::*;
pub use micro::*;
pub use orchestrator::*;
pub use model_selector::*;
