//! Rule definitions and loading
//!
//! US-15.2: Cargador y Validador de Reglas YAML

use crate::errors::{DeclarativeExtractorError, Result};
use crate::tree_sitter::Language;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Rule metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMetadata {
    pub name: String,
    pub description: Option<String>,
    pub severity: String,
    pub confidence: String,
    pub category: Option<String>,
    pub cwe: Option<Vec<String>>,
    pub owasp: Option<Vec<String>>,
}

/// Pattern definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub pattern: String,
    pub message: String,
}

/// Where clause for advanced matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhereClause {
    pub metavariable: Option<String>,
    pub pattern: Option<String>,
    pub not: Option<Box<WhereClause>>,
}

/// Fix suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fix {
    pub template: String,
    pub message: String,
}

/// Test case for rule validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub code: String,
    pub should_match: bool,
}

/// Rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub metadata: Option<RuleMetadata>,
    pub languages: Vec<String>,
    pub patterns: Vec<Pattern>,
    pub where_clause: Option<Vec<WhereClause>>,
    pub fix: Option<Fix>,
    pub tests: Option<Vec<TestCase>>,
}

/// Collection of rules
#[derive(Debug, Clone)]
pub struct RuleSet {
    pub rules: Vec<Rule>,
    pub source_dir: Option<String>,
}

impl RuleSet {
    /// Create a new empty rule set
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            source_dir: None,
        }
    }

    /// Add a rule to the set
    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }

    /// Get all rules
    pub fn rules(&self) -> &[Rule] {
        &self.rules
    }

    /// Get rules for a specific language
    pub fn rules_for_language(&self, language: Language) -> Vec<&Rule> {
        let lang_name = language.name();
        self.rules
            .iter()
            .filter(|rule| rule.languages.contains(&lang_name.to_string()))
            .collect()
    }

    /// Get rule by ID
    pub fn get_rule(&self, id: &str) -> Option<&Rule> {
        self.rules.iter().find(|r| r.id == id)
    }
}

impl Default for RuleSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Rule loader and validator
pub struct RuleLoader {
    validator: RuleValidator,
}

#[derive(Debug, Clone)]
struct RuleValidator;

impl RuleValidator {
    /// Validate a rule
    pub fn validate(&self, rule: &Rule) -> Result<()> {
        // Validate required fields
        if rule.id.is_empty() {
            return Err(DeclarativeExtractorError::validation(
                "Rule ID cannot be empty",
            ));
        }

        if rule.patterns.is_empty() {
            return Err(DeclarativeExtractorError::validation(
                "Rule must have at least one pattern",
            ));
        }

        if rule.languages.is_empty() {
            return Err(DeclarativeExtractorError::validation(
                "Rule must specify at least one language",
            ));
        }

        // Validate pattern format
        for pattern in &rule.patterns {
            if pattern.pattern.is_empty() {
                return Err(DeclarativeExtractorError::validation(
                    "Pattern cannot be empty",
                ));
            }
        }

        Ok(())
    }
}

impl RuleLoader {
    /// Create a new rule loader
    pub fn new() -> Self {
        Self {
            validator: RuleValidator,
        }
    }

    /// Load a single rule from YAML
    pub fn load_rule_from_yaml(&self, yaml: &str) -> Result<Rule> {
        let rule: Rule = serde_yaml::from_str(yaml).map_err(|e| {
            DeclarativeExtractorError::parse(format!("Failed to parse rule YAML: {}", e))
        })?;

        // Validate the rule
        self.validator.validate(&rule)?;

        Ok(rule)
    }

    /// Load rules from a directory
    pub fn load_rules_from_dir(&self, dir: &Path) -> Result<RuleSet> {
        let mut rule_set = RuleSet::new();
        rule_set.source_dir = Some(dir.to_string_lossy().to_string());

        // For now, just return empty set
        // TODO: Implement directory scanning and YAML loading

        Ok(rule_set)
    }
}

impl Default for RuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rule_set_creation() {
        let rule_set = RuleSet::new();
        assert_eq!(rule_set.rules().len(), 0);
    }

    #[test]
    fn test_rule_loader_basic() {
        let loader = RuleLoader::new();

        let yaml = r#"
id: TEST-001
languages:
  - python
patterns:
  - pattern: "x = $VAR"
    message: "Test pattern"
"#;

        let result = loader.load_rule_from_yaml(yaml);
        assert!(result.is_ok(), "Should parse valid YAML");

        let rule = result.unwrap();
        assert_eq!(rule.id, "TEST-001");
        assert_eq!(rule.languages.len(), 1);
    }

    #[test]
    fn test_validate_rule_empty_id() {
        let loader = RuleLoader::new();

        let yaml = r#"
id: ""
languages:
  - python
patterns:
  - pattern: "x = $VAR"
    message: "Test"
"#;

        let result = loader.load_rule_from_yaml(yaml);
        assert!(result.is_err(), "Should reject rule with empty ID");
    }

    #[test]
    fn test_validate_rule_no_patterns() {
        let loader = RuleLoader::new();

        let yaml = r#"
id: TEST-001
languages:
  - python
patterns: []
"#;

        let result = loader.load_rule_from_yaml(yaml);
        assert!(result.is_err(), "Should reject rule with no patterns");
    }

    #[test]
    fn test_rules_for_language() {
        let mut rule_set = RuleSet::new();

        let rule1 = Rule {
            id: "RULE-1".to_string(),
            metadata: None,
            languages: vec!["python".to_string()],
            patterns: vec![],
            where_clause: None,
            fix: None,
            tests: None,
        };

        let rule2 = Rule {
            id: "RULE-2".to_string(),
            metadata: None,
            languages: vec!["javascript".to_string()],
            patterns: vec![],
            where_clause: None,
            fix: None,
            tests: None,
        };

        rule_set.add_rule(rule1);
        rule_set.add_rule(rule2);

        let python_rules = rule_set.rules_for_language(Language::Python);
        assert_eq!(python_rules.len(), 1);
        assert_eq!(python_rules[0].id, "RULE-1");
    }

    #[test]
    fn test_complete_rule_yaml() {
        let loader = RuleLoader::new();

        let yaml = r#"
id: HODEI-SEC-001
metadata:
  name: "SQL Injection Detection"
  description: "Detects potential SQL injection vulnerabilities"
  severity: critical
  confidence: high
  category: security
  cwe: ["89"]
  owasp: ["A03:2021"]
languages:
  - python
  - javascript
patterns:
  - pattern: |
      $DB.execute(f"... {$VAR} ...")
    message: "SQL query with f-string formatting"
  - pattern: |
      $SQL = "... %s ..."
      $DB.execute($SQL % $VAR)
    message: "SQL query with % formatting"
fix:
  template: |
    $DB.execute("... WHERE id = ?", ($VAR,))
  message: "Use parameterized queries"
tests:
  - name: "Detects f-string SQL"
    code: "db.execute(f'SELECT * FROM users WHERE id = {user_id}')"
    should_match: true
  - name: "No false positive for params"
    code: "db.execute('SELECT * FROM users WHERE id = ?', (user_id,))"
    should_match: false
"#;

        let result = loader.load_rule_from_yaml(yaml);
        assert!(result.is_ok(), "Should parse complete YAML rule");

        let rule = result.unwrap();
        assert_eq!(rule.id, "HODEI-SEC-001");
        assert_eq!(rule.languages.len(), 2);
        assert_eq!(rule.patterns.len(), 2);
        assert!(rule.metadata.is_some());
        assert!(rule.fix.is_some());
        assert!(rule.tests.is_some());

        let metadata = rule.metadata.unwrap();
        assert_eq!(metadata.name, "SQL Injection Detection");
        assert_eq!(metadata.severity, "critical");
        assert_eq!(metadata.confidence, "high");
    }

    #[test]
    fn test_minimal_rule_yaml() {
        let loader = RuleLoader::new();

        let yaml = r#"
id: TEST-001
languages:
  - python
patterns:
  - pattern: "x = $VAR"
    message: "Test pattern"
"#;

        let result = loader.load_rule_from_yaml(yaml);
        assert!(result.is_ok(), "Should parse minimal YAML rule");
    }

    #[test]
    fn test_validate_rule_no_languages() {
        let loader = RuleLoader::new();

        let yaml = r#"
id: TEST-001
languages: []
patterns:
  - pattern: "x = $VAR"
    message: "Test"
"#;

        let result = loader.load_rule_from_yaml(yaml);
        assert!(result.is_err(), "Should reject rule with no languages");
    }

    #[test]
    fn test_rule_get_by_id() {
        let mut rule_set = RuleSet::new();

        let rule1 = Rule {
            id: "RULE-1".to_string(),
            metadata: None,
            languages: vec!["python".to_string()],
            patterns: vec![],
            where_clause: None,
            fix: None,
            tests: None,
        };

        let rule2 = Rule {
            id: "RULE-2".to_string(),
            metadata: None,
            languages: vec!["javascript".to_string()],
            patterns: vec![],
            where_clause: None,
            fix: None,
            tests: None,
        };

        rule_set.add_rule(rule1.clone());
        rule_set.add_rule(rule2.clone());

        assert_eq!(rule_set.get_rule("RULE-1").unwrap().id, "RULE-1");
        assert_eq!(rule_set.get_rule("RULE-2").unwrap().id, "RULE-2");
        assert!(rule_set.get_rule("NONEXISTENT").is_none());
    }
}
