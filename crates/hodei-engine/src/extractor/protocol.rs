//! Cap'n Proto protocol definitions for extractor communication
//!
//! This module defines the message format for communication between
//! hodei-scan orchestrator and external extractor processes.
//! Currently uses JSON for serialization, but structured for Cap'n Proto migration.

use crate::extractor::error::OrchestratorError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Configuration for an extractor definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractorConfig {
    /// List of extractors to run
    pub extractors: Vec<ExtractorDef>,
    /// Maximum number of extractors to run concurrently
    pub max_concurrent: Option<usize>,
    /// Default timeout for all extractors (if not overridden)
    pub default_timeout: Option<Duration>,
    /// Global memory limit in bytes for all extractors combined
    pub global_memory_limit: Option<u64>,
    /// Global CPU limit as percentage (0-100)
    pub global_cpu_limit: Option<u8>,
    /// Default nice value for CPU priority
    pub default_nice: Option<i32>,
    /// Default I/O priority class
    pub default_io_priority: Option<u8>,
}

/// Definition of a single extractor
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtractorDef {
    /// Unique name for this extractor
    pub name: String,
    /// Command to execute (e.g., "ruff-to-hodei")
    pub command: String,
    /// Arguments to pass to the command
    pub args: Vec<String>,
    /// Timeout for this specific extractor
    pub timeout: Option<Duration>,
    /// Environment variables
    pub env: Option<HashMap<String, String>>,
    /// Memory limit in bytes (None for unlimited)
    pub memory_limit: Option<u64>,
    /// CPU priority (nice value: -20 to 19, None for default)
    pub cpu_priority: Option<i32>,
    /// I/O priority (ionice class: 0-3, None for default)
    pub io_priority: Option<u8>,
}

/// Request sent to extractor via stdin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorRequest {
    /// Unique request ID for tracking
    pub request_id: u64,
    /// Path to project being analyzed
    pub project_path: String,
    /// Programming language of the project
    pub language: String,
    /// JSON configuration for the extractor
    pub config: String,
    /// Timeout in milliseconds
    pub timeout_ms: u32,
    /// Protocol version
    pub version: String,
}

impl ExtractorRequest {
    /// Serialize to JSON (temporary, will be Cap'n Proto in future)
    pub fn to_json(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from JSON (temporary, will be Cap'n Proto in future)
    pub fn from_json(buffer: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(buffer)
    }
}

/// Response received from extractor via stdout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorResponse {
    /// Must match request ID
    pub request_id: u64,
    /// Whether the extraction succeeded
    pub success: bool,
    /// Serialized IR data
    pub ir: Vec<u8>,
    /// Metadata about the extraction (version, stats, etc.)
    pub metadata: String,
    /// Time taken for extraction in milliseconds
    pub processing_time_ms: u32,
}

impl ExtractorResponse {
    /// Serialize to JSON (temporary, will be Cap'n Proto in future)
    pub fn to_json(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from JSON (temporary, will be Cap'n Proto in future)
    pub fn from_json(buffer: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(buffer)
    }
}

/// Error response for failed extractions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Must match request ID
    pub request_id: u64,
    /// Error code
    pub error_code: u32,
    /// Human-readable error message
    pub error_message: String,
    /// Additional error details (JSON)
    pub error_details: String,
}

impl ErrorResponse {
    /// Serialize to JSON (temporary, will be Cap'n Proto in future)
    pub fn to_json(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from JSON (temporary, will be Cap'n Proto in future)
    pub fn from_json(buffer: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(buffer)
    }
}

/// Heartbeat for liveness checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    /// Unix timestamp
    pub timestamp: u64,
    /// Name of extractor
    pub extractor_name: String,
    /// Status: "running", "idle", etc.
    pub status: String,
}

impl Heartbeat {
    /// Serialize to JSON (temporary, will be Cap'n Proto in future)
    pub fn to_json(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from JSON (temporary, will be Cap'n Proto in future)
    pub fn from_json(buffer: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(buffer)
    }
}

/// Main message union for request/response pattern
#[derive(Debug, Serialize, Deserialize)]
pub enum ExtractorMessage {
    Request(ExtractorRequest),
    Response(ExtractorResponse),
    Error(ErrorResponse),
    Heartbeat(Heartbeat),
}

impl ExtractorMessage {
    /// Serialize message to bytes (currently JSON, will be Cap'n Proto)
    pub fn serialize(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize message from bytes (currently JSON, will be Cap'n Proto)
    pub fn deserialize(buffer: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(buffer)
    }
}

/// Aggregated IR from multiple extractors
#[derive(Debug)]
pub struct AggregatedIR {
    /// Combined facts from all extractors
    pub facts: Vec<hodei_ir::Fact>,
    /// Metadata from all extractors
    pub metadata: HashMap<String, String>,
    /// Success/failure status per extractor
    pub extractor_status: HashMap<String, bool>,
}

impl AggregatedIR {
    pub fn new() -> Self {
        Self {
            facts: Vec::new(),
            metadata: HashMap::new(),
            extractor_status: HashMap::new(),
        }
    }

    pub fn add_extractor_results(
        &mut self,
        extractor_name: &str,
        response: &ExtractorResponse,
    ) -> Result<(), OrchestratorError> {
        if !response.success {
            self.extractor_status
                .insert(extractor_name.to_string(), false);
            return Err(OrchestratorError::AggregatorError(
                "Extractor failed".to_string(),
            ));
        }

        self.extractor_status
            .insert(extractor_name.to_string(), true);

        // Parse metadata
        if !response.metadata.is_empty() {
            let metadata: HashMap<String, String> = serde_json::from_str(&response.metadata)
                .map_err(|e| OrchestratorError::ProtoError(e.to_string()))?;
            self.metadata.extend(metadata);
        }

        Ok(())
    }
}

impl Default for AggregatedIR {
    fn default() -> Self {
        Self::new()
    }
}

/// Protocol version for backward compatibility
pub const PROTOCOL_VERSION: &str = "1.0.0";

/// Validate extractor configuration
pub fn validate_config(config: &ExtractorConfig) -> Result<(), OrchestratorError> {
    if config.extractors.is_empty() {
        return Err(OrchestratorError::ConfigError(
            "No extractors defined".to_string(),
        ));
    }

    for extractor in &config.extractors {
        if extractor.name.is_empty() {
            return Err(OrchestratorError::ValidationError(
                "Extractor name cannot be empty".to_string(),
            ));
        }
        if extractor.command.is_empty() {
            return Err(OrchestratorError::ValidationError(
                "Extractor command cannot be empty".to_string(),
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_config_validation() {
        let config = ExtractorConfig {
            extractors: vec![ExtractorDef {
                name: "test".to_string(),
                command: "test-command".to_string(),
                args: vec![],
                timeout: None,
                env: None,
                memory_limit: None,
                cpu_priority: None,
                io_priority: None,
            }],
            max_concurrent: Some(4),
            default_timeout: Some(Duration::from_secs(30)),
            global_memory_limit: None,
            global_cpu_limit: None,
            default_nice: None,
            default_io_priority: None,
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_message_serialization() {
        let request = ExtractorRequest {
            request_id: 12345,
            project_path: "/path/to/project".to_string(),
            language: "rust".to_string(),
            config: r#"{"rule": "test"}"#.to_string(),
            timeout_ms: 30000,
            version: "1.0.0".to_string(),
        };

        // Serialize to JSON
        let message = ExtractorMessage::Request(request.clone());
        let serialized = message.serialize().unwrap();

        // Deserialize from JSON
        let deserialized = ExtractorMessage::deserialize(&serialized).unwrap();

        match deserialized {
            ExtractorMessage::Request(deserialized_request) => {
                assert_eq!(request.request_id, deserialized_request.request_id);
                assert_eq!(request.project_path, deserialized_request.project_path);
                assert_eq!(request.language, deserialized_request.language);
                assert_eq!(request.timeout_ms, deserialized_request.timeout_ms);
            }
            _ => panic!("Expected Request message"),
        }
    }

    #[test]
    fn test_response_serialization() {
        let response = ExtractorResponse {
            request_id: 12345,
            success: true,
            ir: vec![1, 2, 3, 4],
            metadata: r#"{"version": "1.0"}"#.to_string(),
            processing_time_ms: 150,
        };

        let message = ExtractorMessage::Response(response.clone());
        let serialized = message.serialize().unwrap();
        let deserialized = ExtractorMessage::deserialize(&serialized).unwrap();

        match deserialized {
            ExtractorMessage::Response(deserialized_response) => {
                assert_eq!(response.request_id, deserialized_response.request_id);
                assert_eq!(response.success, deserialized_response.success);
                assert_eq!(response.ir, deserialized_response.ir);
            }
            _ => panic!("Expected Response message"),
        }
    }
}
