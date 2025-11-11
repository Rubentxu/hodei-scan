//! Ruff adapter for parsing Ruff linter output and converting to hodei-ir
//!
//! Ruff (https://github.com/astral-sh/ruff) is an extremely fast Python linter
//! written in Rust. This adapter parses Ruff's JSON output and converts it to
//! hodei-ir Facts for integration with the hodei-scan pipeline.

use crate::extractor::error::OrchestratorError;
use hodei_ir::types::{
    ColumnNumber, Confidence, ExtractorId, LineNumber, ProjectPath, Provenance, Severity,
    SourceLocation,
};
use hodei_ir::{Fact, FactType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::process::Command;

/// Configuration for the Ruff adapter
#[derive(Debug, Clone)]
pub struct RuffConfig {
    /// Maximum number of files to process in parallel
    pub max_parallel: usize,
    /// Whether to include fix suggestions
    pub include_fixes: bool,
}

impl Default for RuffConfig {
    fn default() -> Self {
        Self {
            max_parallel: 4,
            include_fixes: false,
        }
    }
}

/// Ruff diagnostic output structure
/// Based on Ruff's JSON output format
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct RuffDiagnostic {
    /// Absolute or relative path to the file
    filename: String,
    /// The rule that was triggered (e.g., "E501", "F401")
    code: String,
    /// The unique identifier of the diagnostic (e.g., "E501")
    #[serde(rename = "rule")]
    rule_id: String,
    /// The textual message
    message: String,
    /// The severity level
    severity: RuffSeverity,
    /// Line number (1-based)
    line: u32,
    /// Column number (1-based, character offset)
    column: Option<u32>,
    /// End line number (1-based)
    end_line: Option<u32>,
    /// End column number (1-based, character offset)
    end_column: Option<u32>,
    /// The URL for documentation on this rule
    #[serde(skip_serializing_if = "Option::is_none")]
    url: Option<String>,
}

/// Ruff severity levels
#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum RuffSeverity {
    /// Error severity
    Error,
    /// Warning severity
    Warning,
    /// Info severity
    Info,
}

/// Error types specific to Ruff adapter
#[derive(Debug, thiserror::Error)]
pub enum RuffError {
    #[error("Failed to execute Ruff: {0}")]
    ExecutionFailed(#[from] std::io::Error),

    #[error("Ruff command failed with exit code {exit_code}: {stderr}")]
    RuffCommandFailed { exit_code: i32, stderr: String },

    #[error("Failed to parse Ruff JSON output: {0}")]
    JsonParseError(#[from] serde_json::Error),

    #[error("Invalid file path: {0}")]
    InvalidPath(String),

    #[error("Ruff output is not UTF-8: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
}

/// Core Ruff adapter that handles parsing Ruff JSON output
pub struct RuffAdapter {
    config: RuffConfig,
}

impl RuffAdapter {
    /// Create a new RuffAdapter with default configuration
    pub fn new() -> Self {
        Self {
            config: RuffConfig::default(),
        }
    }

    /// Create a new RuffAdapter with custom configuration
    pub fn with_config(config: RuffConfig) -> Self {
        Self { config }
    }

    /// Parse Ruff JSON output string to IR Facts
    ///
    /// # Arguments
    /// * `ruff_json` - The JSON output from Ruff
    /// * `project_root` - The root path of the project
    ///
    /// # Returns
    /// Vector of Facts representing the Ruff diagnostics
    pub async fn parse_str(
        &self,
        ruff_json: &str,
        project_root: &PathBuf,
    ) -> Result<Vec<Fact>, RuffError> {
        let diagnostics: Vec<RuffDiagnostic> = serde_json::from_str(ruff_json)?;
        self.convert_diagnostics_to_facts(diagnostics, project_root)
    }

    /// Parse Ruff output from a JSON file
    ///
    /// # Arguments
    /// * `json_path` - Path to the Ruff JSON output file
    /// * `project_root` - The root path of the project
    ///
    /// # Returns
    /// Vector of Facts representing the Ruff diagnostics
    pub async fn parse_file<P: AsRef<std::path::Path>>(
        &self,
        json_path: P,
        project_root: &PathBuf,
    ) -> Result<Vec<Fact>, RuffError> {
        let json_content = tokio::fs::read_to_string(json_path.as_ref())
            .await
            .map_err(RuffError::ExecutionFailed)?;
        self.parse_str(&json_content, project_root).await
    }

    /// Execute Ruff on a project and parse the output
    ///
    /// # Arguments
    /// * `project_path` - Path to the Python project to analyze
    /// * `extra_args` - Additional arguments to pass to Ruff
    ///
    /// # Returns
    /// Vector of Facts representing the Ruff diagnostics
    pub async fn run_ruff<P: AsRef<std::path::Path>>(
        &self,
        project_path: P,
        extra_args: Option<Vec<&str>>,
    ) -> Result<Vec<Fact>, RuffError> {
        let mut command = Command::new("ruff");
        command
            .arg("check")
            .arg("--format=json")
            .arg("--output-format=json");

        if let Some(args) = extra_args {
            command.args(args);
        }

        command.arg(project_path.as_ref());

        let output = command.output().await.map_err(RuffError::ExecutionFailed)?;

        if !output.status.success() {
            return Err(RuffError::RuffCommandFailed {
                exit_code: output.status.code().unwrap_or(-1),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let stdout = String::from_utf8(output.stdout).map_err(RuffError::InvalidUtf8)?;
        let project_root = project_path.as_ref().to_path_buf();
        self.parse_str(&stdout, &project_root).await
    }

    /// Convert Ruff diagnostics to hodei-ir Facts
    fn convert_diagnostics_to_facts(
        &self,
        diagnostics: Vec<RuffDiagnostic>,
        project_root: &PathBuf,
    ) -> Result<Vec<Fact>, RuffError> {
        let mut facts = Vec::new();
        let version = format!("ruff-adapter-{}", env!("CARGO_PKG_VERSION"));

        for diagnostic in diagnostics {
            // Convert file path to relative path from project root
            let file_path = PathBuf::from(&diagnostic.filename);
            let relative_path = if file_path.is_absolute() {
                file_path
                    .strip_prefix(project_root)
                    .unwrap_or(&file_path)
                    .to_path_buf()
            } else {
                file_path
            };

            // Create source location directly (matching SARIF adapter style)
            let source_location = SourceLocation {
                file: ProjectPath::new(relative_path),
                start_line: LineNumber::new(diagnostic.line).map_err(|_| {
                    RuffError::InvalidPath(format!("Invalid line number: {}", diagnostic.line))
                })?,
                start_column: diagnostic.column.map(|col| ColumnNumber::new(col).unwrap()),
                end_line: LineNumber::new(diagnostic.end_line.unwrap_or(diagnostic.line))
                    .map_err(|_| RuffError::InvalidPath(format!("Invalid end line number")))?,
                end_column: diagnostic
                    .end_column
                    .map(|col| ColumnNumber::new(col).unwrap()),
            };

            // Determine confidence based on rule severity and presence
            // Ruff diagnostics are typically high confidence as they're from a trusted tool
            let confidence = Confidence::HIGH;

            // Map severity to FactType
            let severity = match diagnostic.severity {
                RuffSeverity::Error => Severity::Critical,
                RuffSeverity::Warning => Severity::Major,
                RuffSeverity::Info => Severity::Minor,
            };

            let fact_type = FactType::Vulnerability {
                cwe_id: None,
                owasp_category: Some(format!("ruff-{}", diagnostic.code)),
                severity,
                cvss_score: None,
                description: format!("[{}] {}", diagnostic.code, diagnostic.message),
                confidence,
            };

            // Create provenance
            let provenance = Provenance::new(ExtractorId::RuffAdapter, version.clone(), confidence);

            // Create fact using Fact::new()
            let fact = Fact::new(fact_type, source_location, provenance);

            facts.push(fact);
        }

        Ok(facts)
    }
}

impl Default for RuffAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodei_ir::FactType;

    #[tokio::test]
    async fn test_parse_ruff_json_error() {
        let adapter = RuffAdapter::new();
        let ruff_json = r#"[
            {
                "filename": "test.py",
                "code": "E501",
                "rule": "E501",
                "message": "Line too long",
                "severity": "error",
                "line": 10,
                "column": 80,
                "endLine": 10,
                "endColumn": 85
            }
        ]"#;

        let project_root = PathBuf::from("/project");
        let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

        assert_eq!(facts.len(), 1);
        let fact = &facts[0];

        assert!(matches!(fact.fact_type, FactType::Vulnerability { .. }));
        assert_eq!(fact.location.start_line.get(), 10);
        assert_eq!(
            fact.location.start_column.as_ref().map(|c| c.get()),
            Some(80)
        );
        if let FactType::Vulnerability { description, .. } = &fact.fact_type {
            assert!(description.contains("Line too long"));
        }
    }

    #[tokio::test]
    async fn test_parse_ruff_json_warning() {
        let adapter = RuffAdapter::new();
        let ruff_json = r#"[
            {
                "filename": "test.py",
                "code": "F401",
                "rule": "F401",
                "message": "Unused import",
                "severity": "warning",
                "line": 5,
                "column": 1
            }
        ]"#;

        let project_root = PathBuf::from("/project");
        let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

        assert_eq!(facts.len(), 1);
        let fact = &facts[0];

        assert!(matches!(fact.fact_type, FactType::Vulnerability { .. }));
        if let FactType::Vulnerability { owasp_category, .. } = &fact.fact_type {
            assert_eq!(owasp_category.as_ref().unwrap(), "ruff-F401");
        }
    }

    #[tokio::test]
    async fn test_parse_ruff_json_multiple() {
        let adapter = RuffAdapter::new();
        let ruff_json = r#"[
            {
                "filename": "test.py",
                "code": "E501",
                "rule": "E501",
                "message": "Line too long",
                "severity": "error",
                "line": 10
            },
            {
                "filename": "test.py",
                "code": "F401",
                "rule": "F401",
                "message": "Unused import",
                "severity": "warning",
                "line": 5
            }
        ]"#;

        let project_root = PathBuf::from("/project");
        let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

        assert_eq!(facts.len(), 2);
    }

    #[test]
    fn test_severity_mapping_error() {
        let severity = RuffSeverity::Error;
        // This tests that our enum serialization works
        let json = serde_json::to_string(&severity).unwrap();
        assert_eq!(json, "\"error\"");
    }

    #[test]
    fn test_severity_mapping_warning() {
        let severity = RuffSeverity::Warning;
        let json = serde_json::to_string(&severity).unwrap();
        assert_eq!(json, "\"warning\"");
    }

    #[test]
    fn test_severity_mapping_info() {
        let severity = RuffSeverity::Info;
        let json = serde_json::to_string(&severity).unwrap();
        assert_eq!(json, "\"info\"");
    }
}

impl From<RuffError> for OrchestratorError {
    fn from(error: RuffError) -> Self {
        OrchestratorError::AggregatorError(error.to_string())
    }
}
