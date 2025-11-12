//! Core traits and types for the hodei-extractors system
//!
//! This module defines the fundamental abstractions for the extractor architecture,
//! following the EPIC-14 specification for Phase 1 (Adapters).

use hodei_ir::{Fact, IntermediateRepresentation, ProjectPath};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur during extraction
#[derive(Error, Debug)]
pub enum ExtractorError {
    #[error("Extractor '{id}' failed to spawn: {error}")]
    SpawnFailed { id: String, error: String },

    #[error("Extractor '{id}' failed with exit code {exit_code:?}: {stderr}")]
    ExecutionFailed {
        id: String,
        exit_code: Option<i32>,
        stderr: String,
    },

    #[error("Extractor '{id}' exceeded timeout of {timeout:?}")]
    Timeout { id: String, timeout: Duration },

    #[error("Failed to parse IR from extractor '{id}': {error}")]
    InvalidIR { id: String, error: String },

    #[error("IO error in extractor '{id}': {error}")]
    Io {
        id: String,
        #[source]
        error: std::io::Error,
    },

    #[error("JSON error in extractor '{id}': {error}")]
    Json {
        id: String,
        #[source]
        error: serde_json::Error,
    },

    #[error("All extractors failed, cannot proceed")]
    AllExtractorsFailed,

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Configuration for an individual extractor
///
/// This is read from the `hodei.toml` configuration file and defines
/// how a specific extractor should be invoked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorDefinition {
    /// Unique identifier for this extractor
    pub id: String,

    /// Command to execute (must be in PATH or absolute path)
    pub command: String,

    /// Whether this extractor is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Timeout in seconds for this extractor
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Extractor-specific configuration
    #[serde(default)]
    pub config: serde_json::Value,
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> u64 {
    300 // 5 minutes
}

/// Input configuration passed to an extractor via stdin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorConfig {
    /// Root path of the project being analyzed
    pub project_path: PathBuf,

    /// Extractor-specific configuration from hodei.toml
    pub config: serde_json::Value,

    /// Optional file filters (globs to include/exclude)
    #[serde(default)]
    pub file_filters: FileFilters,
}

/// File filtering configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileFilters {
    /// Glob patterns to include
    #[serde(default)]
    pub include: Vec<String>,

    /// Glob patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Metadata about an extractor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractorMetadata {
    /// Unique identifier
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Version string
    pub version: String,

    /// Supported file extensions
    pub supported_extensions: Vec<String>,

    /// Languages this extractor analyzes
    pub languages: Vec<String>,

    /// Brief description
    pub description: String,
}

/// Result of running a single extractor
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExtractorRun {
    /// Extractor ID
    pub id: String,

    /// Whether execution was successful
    pub success: bool,

    /// Duration of the extraction
    #[serde(serialize_with = "serialize_duration")]
    pub duration: Duration,

    /// Number of facts extracted (0 if failed)
    pub facts_extracted: usize,

    /// Error message if failed
    pub error: Option<String>,

    /// Metadata from the extractor
    pub metadata: Option<ExtractorMetadata>,
}

fn serialize_duration<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_f64(duration.as_secs_f64())
}

/// Trait for extractors that can be executed asynchronously
///
/// Extractors are typically run as separate processes that:
/// 1. Read configuration from stdin (JSON format)
/// 2. Analyze source code
/// 3. Write IR to stdout (Cap'n Proto or JSON format)
/// 4. Write logs to stderr
#[async_trait::async_trait]
pub trait Extractor: Send + Sync {
    /// Extract facts from source code
    ///
    /// # Arguments
    /// * `config` - Configuration for this extraction run
    ///
    /// # Returns
    /// * `Ok(IR)` - Successfully extracted intermediate representation
    /// * `Err(ExtractorError)` - Extraction failed
    async fn extract(&self, config: ExtractorConfig) -> Result<IntermediateRepresentation, ExtractorError>;

    /// Get metadata about this extractor
    fn metadata(&self) -> ExtractorMetadata;
}

/// Builder for constructing an IntermediateRepresentation
///
/// This provides a convenient API for extractors to build up facts incrementally.
pub struct IRBuilder {
    facts: Vec<Fact>,
    project_name: Option<String>,
    project_version: Option<String>,
    project_path: Option<PathBuf>,
}

impl IRBuilder {
    /// Create a new IR builder
    pub fn new() -> Self {
        Self {
            facts: Vec::new(),
            project_name: None,
            project_version: None,
            project_path: None,
        }
    }

    /// Set the project name
    pub fn project_name(&mut self, name: String) -> &mut Self {
        self.project_name = Some(name);
        self
    }

    /// Set the project version
    pub fn project_version(&mut self, version: String) -> &mut Self {
        self.project_version = Some(version);
        self
    }

    /// Set the project path
    pub fn project_path(&mut self, path: PathBuf) -> &mut Self {
        self.project_path = Some(path);
        self
    }

    /// Add a single fact
    pub fn add_fact(&mut self, fact: Fact) -> &mut Self {
        self.facts.push(fact);
        self
    }

    /// Add multiple facts
    pub fn add_facts(&mut self, facts: impl IntoIterator<Item = Fact>) -> &mut Self {
        self.facts.extend(facts);
        self
    }

    /// Get the current number of facts
    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }

    /// Build the final IntermediateRepresentation
    pub fn build(self) -> IntermediateRepresentation {
        use hodei_ir::{ProjectMetadata, ProjectPath};

        let metadata = ProjectMetadata::new(
            self.project_name.unwrap_or_else(|| "unknown".to_string()),
            self.project_version.unwrap_or_else(|| "0.0.0".to_string()),
            ProjectPath::new(self.project_path.unwrap_or_else(|| PathBuf::from("."))),
        );

        IntermediateRepresentation::new(metadata)
            .with_facts(self.facts)
    }
}

impl Default for IRBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_definition_deserialization() {
        let json = r#"{
            "id": "test-extractor",
            "command": "test-cmd",
            "enabled": true,
            "timeout_seconds": 120,
            "config": {"key": "value"}
        }"#;

        let def: ExtractorDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.id, "test-extractor");
        assert_eq!(def.command, "test-cmd");
        assert_eq!(def.timeout_seconds, 120);
        assert!(def.enabled);
    }

    #[test]
    fn test_extractor_definition_defaults() {
        let json = r#"{
            "id": "test",
            "command": "cmd"
        }"#;

        let def: ExtractorDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.timeout_seconds, 300); // default
        assert!(def.enabled); // default true
    }

    #[test]
    fn test_ir_builder() {
        let mut builder = IRBuilder::new();
        builder.project_path(PathBuf::from("/test/project"));

        // Note: Creating actual facts would require full IR setup
        // This is a structural test
        assert_eq!(builder.fact_count(), 0);

        let ir = builder.build();
        assert_eq!(ir.facts.len(), 0);
    }
}
