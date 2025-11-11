//! YAML Rule definitions and parsing
//!
//! This module provides the core data structures and parsing logic for YAML-based
//! pattern rules that can be executed by the Tree-sitter matcher.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Errors that can occur during YAML rule parsing
#[derive(Debug, thiserror::Error)]
pub enum YamlError {
    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid field value: {0}")]
    InvalidField(String),

    #[error("YAML parsing error: {0}")]
    ParseError(String),
}

impl From<serde_yaml::Error> for YamlError {
    fn from(err: serde_yaml::Error) -> Self {
        YamlError::ParseError(err.to_string())
    }
}

/// Represents a single YAML rule for pattern matching
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct YamlRule {
    /// Unique identifier for the rule
    pub id: String,

    /// Target programming language
    pub language: String,

    /// Human-readable message describing the finding
    pub message: String,

    /// Severity level (error, warning, info)
    pub severity: String,

    /// Category classification
    pub category: String,

    /// Tree-sitter query pattern
    pub pattern: String,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl YamlRule {
    /// Validate required fields
    pub fn validate(&self) -> Result<(), YamlError> {
        if self.id.trim().is_empty() {
            return Err(YamlError::MissingField("id".to_string()));
        }

        if self.language.trim().is_empty() {
            return Err(YamlError::MissingField("language".to_string()));
        }

        if self.pattern.trim().is_empty() {
            return Err(YamlError::MissingField("pattern".to_string()));
        }

        if self.message.trim().is_empty() {
            return Err(YamlError::MissingField("message".to_string()));
        }

        Ok(())
    }
}

/// Loader for YAML rules from files
pub struct YamlRuleLoader {
    rules: HashMap<String, YamlRule>,
}

impl YamlRuleLoader {
    /// Create a new rule loader
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }
}

/// Load a single YAML rule from string
pub fn parse_yaml_rule(yaml: &str) -> Result<YamlRule, YamlError> {
    let rule: YamlRule = serde_yaml::from_str(yaml)?;
    rule.validate()?;
    Ok(rule)
}

impl Default for YamlRuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_yaml_rule() {
        let yaml = r#"
id: JAVA-EMPTY-CATCH
language: java
message: "Empty catch block"
severity: warning
category: error-handling
pattern: |
  try {
    $STMT
  } catch ($EXCEPTION $VAR) {
    // $COMMENT
  }
"#;

        let rule = parse_yaml_rule(yaml).unwrap();
        assert_eq!(rule.id, "JAVA-EMPTY-CATCH");
        assert_eq!(rule.language, "java");
        assert!(rule.pattern.contains("try"));
    }

    #[test]
    fn validate_required_fields() {
        let rule = YamlRule {
            id: "TEST-001".to_string(),
            language: "java".to_string(),
            message: "Test message".to_string(),
            severity: "warning".to_string(),
            category: "test".to_string(),
            pattern: "test pattern".to_string(),
            metadata: HashMap::new(),
        };

        assert!(rule.validate().is_ok());
    }

    #[test]
    fn validate_missing_fields() {
        let rule = YamlRule {
            id: "".to_string(),
            language: "java".to_string(),
            message: "Test message".to_string(),
            severity: "warning".to_string(),
            category: "test".to_string(),
            pattern: "test pattern".to_string(),
            metadata: HashMap::new(),
        };

        assert!(rule.validate().is_err());
    }
}
