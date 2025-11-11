//! Real SARIF Adapter Extractor - Production Implementation

use hodei_ir::{
    ColumnNumber, Confidence, ExtractorId, Fact, FactType, LineNumber, ProjectPath, Provenance,
    Severity, SourceLocation,
};
use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::debug;

use super::error::OrchestratorError;

#[derive(Debug, Clone)]
pub struct SarifConfig {
    pub max_results: Option<usize>,
}

impl Default for SarifConfig {
    fn default() -> Self {
        Self {
            max_results: Some(10000),
        }
    }
}

#[derive(Debug)]
pub struct SarifAdapter {
    config: SarifConfig,
}

impl SarifAdapter {
    pub fn new(config: SarifConfig) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self::new(SarifConfig::default())
    }

    pub async fn parse_file<P: AsRef<Path>>(&self, sarif_path: P) -> Result<Vec<Fact>, SarifError> {
        let file = File::open(sarif_path.as_ref()).map_err(SarifError::Io)?;
        let mut reader = BufReader::new(file);
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        self.parse_str(&content).await
    }

    pub async fn parse_str(&self, sarif_json: &str) -> Result<Vec<Fact>, SarifError> {
        debug!("Parsing SARIF JSON ({} bytes)", sarif_json.len());

        let value: Value = serde_json::from_str(sarif_json).map_err(SarifError::ParseError)?;

        let facts = self.convert_document(&value)?;
        debug!("Converted {} SARIF results to facts", facts.len());
        Ok(facts)
    }

    fn convert_document(&self, value: &Value) -> Result<Vec<Fact>, SarifError> {
        let runs = value
            .get("runs")
            .and_then(|r| r.as_array())
            .ok_or_else(|| SarifError::InvalidStructure("runs must be an array".to_string()))?;

        let mut facts = Vec::new();

        for run_value in runs {
            let run_facts = self.convert_run(run_value)?;
            facts.extend(run_facts);
        }

        Ok(facts)
    }

    fn convert_run(&self, run_value: &Value) -> Result<Vec<Fact>, SarifError> {
        let results_value = run_value
            .get("results")
            .ok_or_else(|| SarifError::MissingField("results".to_string()))?;

        let results_array = results_value
            .as_array()
            .ok_or_else(|| SarifError::InvalidStructure("results must be an array".to_string()))?;

        let mut facts = Vec::new();

        for result_value in results_array {
            let result = self.parse_result(result_value)?;

            // Create source location
            let file_path = result.file;
            let line = result.start_line;
            let col = result.start_column;

            let source_location = SourceLocation {
                file: ProjectPath::new(PathBuf::from(file_path)),
                start_line: LineNumber::new(line).unwrap(),
                start_column: Some(ColumnNumber::new(col).unwrap()),
                end_line: LineNumber::new(line).unwrap(),
                end_column: Some(ColumnNumber::new(col).unwrap()),
            };

            // Create fact based on level
            let fact = match result.level.as_deref() {
                Some("error") => FactType::Vulnerability {
                    cwe_id: result.rule_id,
                    owasp_category: None,
                    severity: Severity::Critical,
                    cvss_score: None,
                    description: result.message.clone(),
                    confidence: Confidence::new(0.9).unwrap(),
                },
                Some("warning") => FactType::CodeSmell {
                    smell_type: "warning".to_string(),
                    severity: Severity::Major,
                    message: result.message.clone(),
                },
                _ => FactType::Vulnerability {
                    cwe_id: result.rule_id,
                    owasp_category: None,
                    severity: Severity::Minor,
                    cvss_score: None,
                    description: result.message.clone(),
                    confidence: Confidence::new(0.5).unwrap(),
                },
            };

            let provenance = Provenance::new(
                ExtractorId::SarifAdapter,
                "1.0.0".to_string(),
                Confidence::new(0.8).unwrap(),
            );

            facts.push(Fact::new(fact, source_location, provenance));
        }

        Ok(facts)
    }

    fn parse_result(&self, value: &Value) -> Result<ParsedResult, SarifError> {
        let rule_id = value
            .get("ruleId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let level = value
            .get("level")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let message = value
            .get("message")
            .and_then(|m| m.get("text"))
            .and_then(|t| t.as_str())
            .unwrap_or("No message")
            .to_string();

        let locations = value
            .get("locations")
            .and_then(|l| l.as_array())
            .ok_or_else(|| SarifError::MissingField("locations".to_string()))?;

        let first_location = locations
            .first()
            .ok_or_else(|| SarifError::MissingField("at least one location".to_string()))?;

        let phys_loc = first_location
            .get("physicalLocation")
            .ok_or_else(|| SarifError::MissingField("physicalLocation".to_string()))?;

        let artifact_loc = phys_loc
            .get("artifactLocation")
            .ok_or_else(|| SarifError::MissingField("artifactLocation".to_string()))?;

        let file = artifact_loc
            .get("uri")
            .and_then(|u| u.as_str())
            .ok_or_else(|| SarifError::MissingField("uri".to_string()))?
            .to_string();

        let region = phys_loc
            .get("region")
            .ok_or_else(|| SarifError::MissingField("region".to_string()))?;

        let start_line = region
            .get("startLine")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| SarifError::MissingField("startLine".to_string()))?
            as u32;

        let start_column = region
            .get("startColumn")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| SarifError::MissingField("startColumn".to_string()))?
            as u32;

        Ok(ParsedResult {
            rule_id,
            level,
            message,
            file,
            start_line,
            start_column,
        })
    }
}

#[derive(Debug)]
struct ParsedResult {
    rule_id: Option<String>,
    level: Option<String>,
    message: String,
    file: String,
    start_line: u32,
    start_column: u32,
}

#[derive(Error, Debug)]
pub enum SarifError {
    #[error("Parse error: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("Invalid structure: {0}")]
    InvalidStructure(String),
    #[error("Missing field: {0}")]
    MissingField(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<SarifError> for OrchestratorError {
    fn from(error: SarifError) -> Self {
        OrchestratorError::AggregatorError(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SARIF: &str = r#"{
  "version": "2.1.0",
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "test-tool"
        }
      },
      "results": [
        {
          "ruleId": "R001",
          "level": "error",
          "message": { "text": "Test finding" },
          "locations": [
            {
              "physicalLocation": {
                "artifactLocation": { "uri": "test.py" },
                "region": { "startLine": 10, "startColumn": 5 }
              }
            }
          ]
        }
      ]
    }
  ]
}"#;

    #[tokio::test]
    async fn test_parse_minimal_sarif() {
        let adapter = SarifAdapter::default();
        let facts = adapter.parse_str(SAMPLE_SARIF).await.unwrap();
        assert_eq!(facts.len(), 1);
    }
}
