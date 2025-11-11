//! Error types for extractor orchestration

use std::io;
use thiserror::Error;

/// Result type for extractor operations
pub type Result<T> = std::result::Result<T, OrchestratorError>;

/// Error types for ExtractorOrchestrator
#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Async IO error: {0}")]
    AsyncIo(#[from] tokio::io::Error),

    #[error("Failed to spawn extractor process: {0}")]
    SpawnFailed(#[source] std::io::Error),

    #[error("Extractor process failed with exit code {0}")]
    ProcessFailed(i32),

    #[error("Timeout waiting for extractor")]
    Timeout,

    #[error("Failed to kill extractor process: {0}")]
    KillFailed(#[source] std::io::Error),

    #[error("Invalid Cap'n Proto message: {0}")]
    ProtoError(String),

    #[error("JSON serialization/deserialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Extractor validation failed: {0}")]
    ValidationError(String),

    #[error("Resource limit exceeded")]
    ResourceLimitExceeded,

    #[error("Aggregator error: {0}")]
    AggregatorError(String),

    #[error("Join error: {0}")]
    JoinError(String),
}
