// Error types for CRC system

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Drop not found: {0}")]
    DropNotFound(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Unknown source type for path: {0}")]
    UnknownSourceType(String),

    #[error("Invalid priority: {0}")]
    InvalidPriority(String),

    #[error("Watcher error: {0}")]
    WatcherError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("System error: {0}")]
    SystemError(String),

    #[error("Unsupported archive type: {0}")]
    UnsupportedArchive(String),

    #[error("Archive error: {0}")]
    ArchiveError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
