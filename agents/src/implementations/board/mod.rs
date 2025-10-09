//! Board Agents Module
//! 
//! Board-level agents (L2 Reasoning layer)
//! Strategic intelligence and governance

pub mod digest;

// Re-export for convenience
pub use digest::DigestAgent;

/// Placeholder for board-level agents
pub struct BoardAgent {
    pub name: String,
}

impl BoardAgent {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

// Re-export for convenience
pub use self::BoardAgent as FinanceBoardAgent;
pub use self::BoardAgent as LegalComplianceBoardAgent;
pub use self::BoardAgent as OperationsBoardAgent;
pub use self::BoardAgent as StrategyBoardAgent;

