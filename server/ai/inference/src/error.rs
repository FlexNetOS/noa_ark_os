//! Error types for inference operations

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("Server error: {0}")]
    ServerError(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Connection error: {0}")]
    ConnectionError(String),
    
    #[error("Timeout: request took too long")]
    Timeout,
}

pub type Result<T> = std::result::Result<T, Error>;
