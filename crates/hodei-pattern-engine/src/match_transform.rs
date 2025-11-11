//! Match transformation to Facts
//!
//! This module handles the transformation of tree-sitter query matches
//! into IR Facts for integration with the RuleEngine.

use crate::tree_sitter::{QueryCapture, QueryMatch};
use crate::yaml_rule::YamlRule;
use hodei_ir::{
    ColumnNumber, Confidence, ExtractorId, Fact, FactId, FactType, LineNumber, ProjectPath,
    Provenance, SourceLocation,
};

/// Errors during match transformation
#[derive(Debug, thiserror::Error)]
pub enum TransformError {
    #[error("Invalid match: {0}")]
    InvalidMatch(String),

    #[error("Failed to extract capture: {0}")]
    CaptureExtractionFailed(String),

    #[error("Missing required capture: {0}")]
    MissingCapture(String),
}

/// Transform a query match to a Fact
pub fn match_to_fact(
    query_match: &QueryMatch,
    rule: &YamlRule,
    source_code: &str,
) -> Result<Fact, TransformError> {
    // Extract location from match
    let location = create_location_from_match(query_match, source_code)?;

    // Extract relevant captures
    let captures = extract_captures(query_match)?;

    // Create FactType based on rule category
    let fact_type = match rule.category.as_str() {
        "error-handling" => FactType::CodeSmell {
            smell_type: "error-handling".to_string(),
            severity: parse_severity(&rule.severity),
            message: rule.message.clone(),
        },
        "security" | "vulnerability" => FactType::Vulnerability {
            cwe_id: None,
            owasp_category: None,
            description: rule.message.clone(),
            severity: parse_severity(&rule.severity),
            cvss_score: None,
            confidence: Confidence::new(0.9).unwrap(),
        },
        _ => FactType::CodeSmell {
            smell_type: rule.category.clone(),
            severity: parse_severity(&rule.severity),
            message: rule.message.clone(),
        },
    };

    Ok(Fact {
        id: FactId::new(),
        fact_type,
        location,
        provenance: Provenance::new(
            ExtractorId::Custom,
            "yaml-rule-engine".to_string(),
            Confidence::new(0.9).unwrap(),
        ),
    })
}

/// Create a SourceLocation from a QueryMatch
fn create_location_from_match(
    query_match: &QueryMatch,
    source_code: &str,
) -> Result<SourceLocation, TransformError> {
    let start_line = LineNumber::new((query_match.range.start_point.row + 1) as u32)
        .map_err(|_| TransformError::InvalidMatch("Invalid line number".to_string()))?;
    let end_line = LineNumber::new((query_match.range.end_point.row + 1) as u32)
        .map_err(|_| TransformError::InvalidMatch("Invalid line number".to_string()))?;

    Ok(SourceLocation {
        file: ProjectPath::new(std::path::PathBuf::from("unknown")),
        start_line,
        start_column: Some(
            ColumnNumber::new(query_match.range.start_point.column as u32)
                .map_err(|_| TransformError::InvalidMatch("Invalid column".to_string()))?,
        ),
        end_line,
        end_column: Some(
            ColumnNumber::new(query_match.range.end_point.column as u32)
                .map_err(|_| TransformError::InvalidMatch("Invalid column".to_string()))?,
        ),
    })
}

/// Extract captures from query match
fn extract_captures(query_match: &QueryMatch) -> Result<Vec<&QueryCapture>, TransformError> {
    // Allow matches with or without captures
    Ok(query_match.captures.iter().collect())
}

/// Parse severity string to Severity enum
fn parse_severity(severity: &str) -> hodei_ir::Severity {
    match severity.to_lowercase().as_str() {
        "error" => hodei_ir::Severity::Critical,
        "warning" => hodei_ir::Severity::Major,
        "info" => hodei_ir::Severity::Info,
        _ => hodei_ir::Severity::Minor,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree_sitter::{Point, Range};
    use crate::yaml_rule::YamlRule;

    #[test]
    fn test_match_to_fact_code_smell() {
        let query_match = QueryMatch {
            range: Range {
                start_byte: 0,
                end_byte: 10,
                start_point: Point { row: 1, column: 1 },
                end_point: Point { row: 1, column: 11 },
            },
            captures: vec![],
        };

        let rule = YamlRule {
            id: "TEST-001".to_string(),
            language: "python".to_string(),
            message: "Test message".to_string(),
            severity: "warning".to_string(),
            category: "error-handling".to_string(),
            pattern: "test".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        let fact = match_to_fact(&query_match, &rule, "test code").unwrap();

        assert!(matches!(fact.fact_type, FactType::CodeSmell { .. }));
    }

    #[test]
    fn test_match_to_fact_vulnerability() {
        let query_match = QueryMatch {
            range: Range {
                start_byte: 0,
                end_byte: 10,
                start_point: Point { row: 1, column: 1 },
                end_point: Point { row: 1, column: 11 },
            },
            captures: vec![],
        };

        let rule = YamlRule {
            id: "SEC-001".to_string(),
            language: "python".to_string(),
            message: "Security issue".to_string(),
            severity: "error".to_string(),
            category: "security".to_string(),
            pattern: "test".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        let fact = match_to_fact(&query_match, &rule, "test code").unwrap();

        assert!(matches!(fact.fact_type, FactType::Vulnerability { .. }));
    }

    #[test]
    fn test_parse_severity() {
        assert_eq!(parse_severity("error"), hodei_ir::Severity::Critical);
        assert_eq!(parse_severity("warning"), hodei_ir::Severity::Major);
        assert_eq!(parse_severity("info"), hodei_ir::Severity::Info);
        assert_eq!(parse_severity("unknown"), hodei_ir::Severity::Minor);
    }
}
