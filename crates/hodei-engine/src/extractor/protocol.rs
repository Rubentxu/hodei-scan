//! Cap'n Proto protocol definitions for extractor communication
//!
//! This module defines the message format for communication between
//! hodei-scan orchestrator and external extractor processes.

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
}

/// Request sent to extractor via stdin
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractorRequest {
    /// Path to project being analyzed
    pub project_path: String,
    /// Programming language of the project
    pub language: String,
    /// JSON configuration for the extractor
    pub config: String,
    /// Timeout in milliseconds
    pub timeout_ms: u32,
}

/// Response received from extractor via stdout
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractorResponse {
    /// Whether the extraction succeeded
    pub success: bool,
    /// Serialized IR data (Cap'n Proto format)
    pub ir: Vec<u8>,
    /// Error message if success = false
    pub error_message: Option<String>,
    /// Metadata about the extraction (version, stats, etc.)
    pub metadata: String,
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
            self.extractor_status.insert(extractor_name.to_string(), false);
            return Err(OrchestratorError::AggregatorError(
                response.error_message.clone().unwrap_or_default(),
            ));
        }

        self.extractor_status.insert(extractor_name.to_string(), true);

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
            }],
            max_concurrent: Some(4),
            default_timeout: Some(Duration::from_secs(30)),
        };

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_extractor_request_serialization() {
        let request = ExtractorRequest {
            project_path: "/path/to/project".to_string(),
            language: "rust".to_string(),
            config: r#"{"rule": "test"}"#.to_string(),
            timeout_ms: 30000,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: ExtractorRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.project_path, deserialized.project_path);
        assert_eq!(request.language, deserialized.language);
        assert_eq!(request.timeout_ms, deserialized.timeout_ms);
    }
}
