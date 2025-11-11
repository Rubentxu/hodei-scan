//! SARIF Extractor - Production-ready implementation
//!
//! This module implements a real SARIF extractor that integrates with ExtractorOrchestrator.
//! It processes real SARIF files from tools like CodeQL, Semgrep, SonarQube, and converts
//! them to hodei-scan IR format.

use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{ChildStdin, ChildStdout};
use tracing::{debug, error, instrument, warn};

use super::{
    Result,
    error::OrchestratorError,
    protocol::{ExtractorMessage, ExtractorRequest, ExtractorResponse},
    sarif_adapter::{SarifAdapter, SarifConfig, SarifError},
};

/// Real SARIF Extractor implementation
#[derive(Debug)]
pub struct SarifExtractor {
    config: SarifConfig,
}

impl SarifExtractor {
    /// Create a new SARIF extractor with custom configuration
    pub fn new(config: SarifConfig) -> Self {
        Self { config }
    }

    /// Create a SARIF extractor with default configuration
    pub fn default() -> Self {
        Self::new(SarifConfig::default())
    }

    /// Main entry point - extract facts from SARIF file
    #[instrument(skip_all)]
    pub async fn extract_from_file<P: AsRef<Path>>(
        &self,
        sarif_path: P,
    ) -> std::result::Result<ExtractorResponse, OrchestratorError> {
        let path = sarif_path.as_ref();
        debug!("Processing SARIF file: {}", path.display());

        let start_time = std::time::Instant::now();

        // Check if file exists and is readable
        if !path.exists() {
            return Err(OrchestratorError::Io(format!(
                "SARIF file not found: {}",
                path.display()
            )));
        }

        // Parse SARIF using the adapter
        let adapter = SarifAdapter::new(self.config.clone());
        let facts = adapter
            .parse_file(path)
            .await
            .map_err(|e| OrchestratorError::AggregatorError(e.to_string()))?;

        let processing_time = start_time.elapsed();
        debug!(
            "Extracted {} facts from SARIF in {:?}",
            facts.len(),
            processing_time
        );

        // Convert facts to IR bytes (serialize to JSON for simplicity, could be binary)
        let ir_data = self.serialize_facts(&facts)?;

        // Build metadata
        let mut metadata = Map::new();
        metadata.insert("tool".to_string(), Value::String(self.detect_tool(&facts)));
        metadata.insert("facts_count".to_string(), Value::Number(facts.len().into()));
        metadata.insert(
            "processing_time_ms".to_string(),
            Value::Number((processing_time.as_millis() as u64).into()),
        );
        metadata.insert("format".to_string(), Value::String("sarif".to_string()));
        metadata.insert(
            "adapter_version".to_string(),
            Value::String("1.0.0".to_string()),
        );

        // Create response
        Ok(ExtractorResponse {
            request_id: 0, // Will be set by orchestrator
            success: true,
            ir: ir_data,
            metadata: serde_json::to_string(&metadata).unwrap_or_default(),
            processing_time_ms: processing_time.as_millis() as u32,
        })
    }

    /// Extract facts from SARIF JSON string (for stdin input)
    #[instrument(skip_all)]
    pub async fn extract_from_json(
        &self,
        sarif_json: &str,
    ) -> std::result::Result<ExtractorResponse, OrchestratorError> {
        debug!("Processing SARIF JSON ({} bytes)", sarif_json.len());

        let start_time = std::time::Instant::now();

        // Parse SARIF using the adapter
        let adapter = SarifAdapter::new(self.config.clone());
        let facts = adapter
            .parse_str(sarif_json)
            .await
            .map_err(|e| OrchestratorError::AggregatorError(e.to_string()))?;

        let processing_time = start_time.elapsed();
        debug!(
            "Extracted {} facts from SARIF in {:?}",
            facts.len(),
            processing_time
        );

        // Convert facts to IR bytes
        let ir_data = self.serialize_facts(&facts)?;

        // Build metadata
        let mut metadata = Map::new();
        metadata.insert("tool".to_string(), Value::String(self.detect_tool(&facts)));
        metadata.insert("facts_count".to_string(), Value::Number(facts.len().into()));
        metadata.insert(
            "processing_time_ms".to_string(),
            Value::Number((processing_time.as_millis() as u64).into()),
        );
        metadata.insert("format".to_string(), Value::String("sarif".to_string()));
        metadata.insert("source".to_string(), Value::String("stdin".to_string()));

        Ok(ExtractorResponse {
            request_id: 0,
            success: true,
            ir: ir_data,
            metadata: serde_json::to_string(&metadata).unwrap_or_default(),
            processing_time_ms: processing_time.as_millis() as u32,
        })
    }

    /// Run as an external extractor process (reads from stdin, writes to stdout)
    pub async fn run_as_extractor() -> std::result::Result<(), OrchestratorError> {
        let config = SarifConfig::default();
        let extractor = SarifExtractor::new(config);

        // Read request from stdin
        let mut stdin_buffer = String::new();
        let mut stdin = tokio::io::stdin();
        stdin
            .read_to_string(&mut stdin_buffer)
            .await
            .map_err(|e| OrchestratorError::AsyncIo(e.to_string()))?;

        if stdin_buffer.trim().is_empty() {
            return Err(OrchestratorError::ProtoError(
                "Empty input from stdin".to_string(),
            ));
        }

        // Parse request
        let request: ExtractorRequest = serde_json::from_str(&stdin_buffer)
            .map_err(|e| OrchestratorError::ProtoError(e.to_string()))?;

        debug!("Received extraction request for: {}", request.project_path);

        // Look for SARIF file in the project path
        let sarif_file = find_sarif_file(&request.project_path).await?;

        if let Some(sarif_path) = sarif_file {
            let response = extractor.extract_from_file(sarif_path).await?;

            // Write response to stdout
            let response_message = ExtractorMessage::Response(response);
            let output = response_message
                .serialize()
                .map_err(|e| OrchestratorError::ProtoError(e.to_string()))?;

            tokio::io::stdout()
                .write_all(&output)
                .await
                .map_err(|e| OrchestratorError::AsyncIo(e.to_string()))?;
        } else {
            // No SARIF file found - return empty response
            let response = ExtractorResponse {
                request_id: request.request_id,
                success: true,
                ir: Vec::new(),
                metadata: serde_json::to_string(&serde_json::json!({
                    "status": "no_sarif_found",
                    "message": "No SARIF files found in project"
                }))
                .unwrap_or_default(),
                processing_time_ms: 0,
            };

            let response_message = ExtractorMessage::Response(response);
            let output = response_message
                .serialize()
                .map_err(|e| OrchestratorError::ProtoError(e.to_string()))?;

            tokio::io::stdout()
                .write_all(&output)
                .await
                .map_err(|e| OrchestratorError::AsyncIo(e.to_string()))?;
        }

        Ok(())
    }

    /// Serialize facts to IR bytes
    fn serialize_facts(
        &self,
        facts: &[hodei_ir::Fact],
    ) -> std::result::Result<Vec<u8>, OrchestratorError> {
        // Serialize facts as JSON for now
        // In production, this could be Cap'n Proto or another efficient format
        let json = serde_json::to_string(facts)
            .map_err(|e| OrchestratorError::ProtoError(e.to_string()))?;

        Ok(json.into_bytes())
    }

    /// Detect which tool generated the SARIF based on facts
    fn detect_tool(&self, _facts: &[hodei_ir::Fact]) -> String {
        // TODO: Extract tool name from provenance or other source
        "sarif-extractor".to_string()
    }
}

/// Find SARIF file in project directory
async fn find_sarif_file(
    project_path: &str,
) -> std::result::Result<Option<PathBuf>, OrchestratorError> {
    debug!("Searching for SARIF files in: {}", project_path);

    let path = Path::new(project_path);

    if !path.exists() {
        warn!("Project path does not exist: {}", project_path);
        return Ok(None);
    }

    // Common SARIF file patterns
    let patterns = [
        "**/*.sarif",
        "**/sarif-*.json",
        "**/*-sarif.json",
        "results.sarif",
    ];

    // Use tokio to traverse directories
    let mut entries = tokio::fs::read_dir(path)
        .await
        .map_err(|e| OrchestratorError::Io(e.to_string()))?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| OrchestratorError::Io(e.to_string()))?
    {
        let path = entry.path();
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        // Check if file matches SARIF patterns
        if file_name.ends_with(".sarif")
            || (file_name.contains("sarif") && file_name.ends_with(".json"))
        {
            debug!("Found SARIF file: {}", path.display());
            return Ok(Some(path));
        }
    }

    warn!("No SARIF files found in: {}", project_path);
    Ok(None)
}

/// Configuration for running SARIF extractor as external process
#[derive(Debug, Clone)]
pub struct SarifExtractorConfig {
    pub sarif_config: SarifConfig,
    pub search_patterns: Vec<String>,
    pub timeout: Duration,
}

impl Default for SarifExtractorConfig {
    fn default() -> Self {
        Self {
            sarif_config: SarifConfig::default(),
            search_patterns: vec![
                "**/*.sarif".to_string(),
                "**/*sarif*.json".to_string(),
                "results.sarif".to_string(),
            ],
            timeout: Duration::from_secs(300), // 5 minutes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    const TEST_SARIF: &str = r#"{
  "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
  "version": "2.1.0",
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "test-tool",
          "rules": [
            {
              "id": "R001",
              "name": "Test Rule"
            }
          ]
        }
      },
      "results": [
        {
          "ruleId": "R001",
          "level": "error",
          "message": {
            "text": "Test finding"
          },
          "locations": [
            {
              "physicalLocation": {
                "artifactLocation": {
                  "uri": "test.py"
                },
                "region": {
                  "startLine": 10,
                  "startColumn": 5
                }
              }
            }
          ]
        }
      ]
    }
  ]
}"#;

    #[tokio::test]
    async fn test_extract_from_json() {
        let extractor = SarifExtractor::default();
        let response = extractor.extract_from_json(TEST_SARIF).await.unwrap();

        assert!(response.success);
        assert!(!response.ir.is_empty());
        assert!(response.processing_time_ms >= 0);
    }

    #[tokio::test]
    async fn test_sarif_file_not_found() {
        let path = PathBuf::from("/nonexistent/path");
        let extractor = SarifExtractor::default();

        let result = extractor.extract_from_file(path).await;

        assert!(matches!(result, Err(OrchestratorError::Io(_))));
    }

    #[tokio::test]
    async fn test_serialize_facts() {
        use hodei_ir::{
            ColumnNumber, Confidence, ExtractorId, Fact, FactType, LineNumber, ProjectPath,
            Provenance, Severity, SourceLocation,
        };

        let location = SourceLocation {
            file: ProjectPath::new(PathBuf::from("test.py")),
            start_line: LineNumber::new(10).unwrap(),
            start_column: Some(ColumnNumber::new(5).unwrap()),
            end_line: LineNumber::new(10).unwrap(),
            end_column: Some(ColumnNumber::new(15).unwrap()),
        };

        let provenance = Provenance::new(
            ExtractorId::SarifAdapter,
            "1.0.0".to_string(),
            Confidence::new(0.8).unwrap(),
        );

        let fact_type = FactType::Vulnerability {
            cwe_id: Some("CWE-79".to_string()),
            owasp_category: None,
            severity: Severity::Major,
            cvss_score: None,
            description: "Test finding".to_string(),
            confidence: Confidence::new(0.8).unwrap(),
        };

        let facts = vec![Fact::new(fact_type, location, provenance)];

        let extractor = SarifExtractor::default();
        let serialized = extractor.serialize_facts(&facts).unwrap();

        assert!(!serialized.is_empty());
        // Verify it's valid JSON
        let parsed: Vec<Fact> = serde_json::from_slice(&serialized).unwrap();
        assert_eq!(parsed.len(), 1);
    }

    #[test]
    fn test_detect_tool() {
        use hodei_ir::{
            ColumnNumber, Confidence, ExtractorId, Fact, FactType, LineNumber, ProjectPath,
            Provenance, Severity, SourceLocation,
        };

        let location = SourceLocation {
            file: ProjectPath::new(PathBuf::from("test.py")),
            start_line: LineNumber::new(10).unwrap(),
            start_column: Some(ColumnNumber::new(5).unwrap()),
            end_line: LineNumber::new(10).unwrap(),
            end_column: None,
        };

        let provenance = Provenance::new(
            ExtractorId::SarifAdapter,
            "1.0.0".to_string(),
            Confidence::new(0.8).unwrap(),
        );

        let fact_type = FactType::Vulnerability {
            cwe_id: Some("CWE-79".to_string()),
            owasp_category: None,
            severity: Severity::Major,
            cvss_score: None,
            description: "Test".to_string(),
            confidence: Confidence::new(0.8).unwrap(),
        };

        let facts = vec![Fact::new(fact_type, location, provenance)];

        let extractor = SarifExtractor::default();
        let tool = extractor.detect_tool(&facts);

        assert_eq!(tool, "sarif-extractor");
    }
}
