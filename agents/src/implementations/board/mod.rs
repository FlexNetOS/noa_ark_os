//! Board Agents Module
//! 
//! Board-level agents (L2 Reasoning layer)
//! Strategic intelligence and governance

pub mod digest;
pub mod finance;
pub mod legal;
pub mod operations;
pub mod strategy;

// Re-export for convenience
pub use digest::DigestAgent;
pub use finance::FinanceAgent;
pub use legal::LegalAgent;
pub use operations::OperationsAgent;
pub use strategy::StrategyAgent;

