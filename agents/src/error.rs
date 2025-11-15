// Error types for agent registry

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Agent not found: {0}")]
    AgentNotFound(String),
    
    #[error("Invalid agent ID: {0}")]
    InvalidAgentId(String),
    
    #[error("Registry parse error: {0}")]
    ParseError(String),
    
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Agent registry not initialized")]
    NotInitialized,
    
    #[error("Duplicate agent ID: {0}")]
    DuplicateAgent(String),
}

pub type Result<T> = std::result::Result<T, Error>;
