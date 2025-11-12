//! Semi-automatic translator from Semgrep rules to hodei-scan format
//!
//! US-15.4: Traductor Semi-Automático de Reglas Semgrep

use crate::errors::{DeclarativeExtractorError, Result};
use crate::rules::{Pattern, Rule, RuleMetadata, TestCase};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Semgrep rule structure
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SemgrepRule {
    pub id: String,
    pub pattern: Option<String>,
    pub patterns: Option<Vec<serde_yaml::Value>>,
    pub message: Option<String>,
    pub metadata: Option<serde_yaml::Value>,
    pub languages: Vec<String>,
    pub severity: Option<String>,
    pub fix: Option<String>,
}

/// Translation warning
#[derive(Debug, Clone)]
pub struct TranslationWarning {
    pub rule_id: String,
    pub message: String,
    pub severity: WarningSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningSeverity {
    Info,
    Warning,
    Error,
}

/// Translation result
#[derive(Debug, Clone)]
pub struct TranslationResult {
    pub hodei_rule: Rule,
    pub warnings: Vec<TranslationWarning>,
    pub success: bool,
}

/// Semgrep to hodei-scan rule translator
pub struct SemgrepTranslator {
    /// Mapping of Semgrep patterns to hodei patterns
    pattern_mappings: HashMap<String, String>,
    /// Language mappings (semgrep -> hodei)
    language_mappings: HashMap<String, String>,
}

impl SemgrepTranslator {
    /// Create a new translator
    pub fn new() -> Self {
        let mut pattern_mappings = HashMap::new();
        pattern_mappings.insert(
            "metavariable-pattern".to_string(),
            "metavariable".to_string(),
        );
        pattern_mappings.insert("pattern-either".to_string(), "alternatives".to_string());

        let mut language_mappings = HashMap::new();
        language_mappings.insert("python".to_string(), "python".to_string());
        language_mappings.insert("javascript".to_string(), "javascript".to_string());
        language_mappings.insert("typescript".to_string(), "typescript".to_string());
        language_mappings.insert("go".to_string(), "go".to_string());
        language_mappings.insert("java".to_string(), "java".to_string());
        language_mappings.insert("c".to_string(), "c".to_string());
        language_mappings.insert("cpp".to_string(), "cpp".to_string());

        Self {
            pattern_mappings,
            language_mappings,
        }
    }

    /// Translate a single Semgrep rule to hodei-scan format
    pub fn translate_rule(&self, semgrep_yaml: &str) -> Result<TranslationResult> {
        let semgrep_rule: SemgrepRule = serde_yaml::from_str(semgrep_yaml).map_err(|e| {
            DeclarativeExtractorError::parse(format!("Failed to parse Semgrep rule: {}", e))
        })?;

        let mut warnings = Vec::new();

        // Validate required fields
        if semgrep_rule.id.is_empty() {
            return Err(DeclarativeExtractorError::validation(
                "Semgrep rule must have an ID",
            ));
        }

        // Translate metadata
        let metadata = self.translate_metadata(&semgrep_rule, &mut warnings);

        // Translate patterns
        let patterns = self.translate_patterns(&semgrep_rule, &mut warnings);

        // Translate languages
        let languages = self.translate_languages(&semgrep_rule);

        // Create hodei rule
        let hodei_rule = Rule {
            id: format!("HODEI-TRANSLATED-{}", semgrep_rule.id),
            metadata: Some(metadata),
            languages,
            patterns,
            where_clause: None,
            fix: None,
            tests: None,
        };

        Ok(TranslationResult {
            hodei_rule,
            warnings,
            success: true,
        })
    }

    /// Translate metadata from Semgrep to hodei format
    fn translate_metadata(
        &self,
        semgrep: &SemgrepRule,
        warnings: &mut Vec<TranslationWarning>,
    ) -> RuleMetadata {
        RuleMetadata {
            name: semgrep
                .message
                .clone()
                .unwrap_or_else(|| semgrep.id.clone()),
            description: semgrep.message.clone(),
            severity: self.translate_severity(semgrep.severity.as_deref().unwrap_or("info")),
            confidence: "medium".to_string(), // Default for Semgrep rules
            category: Some("translated".to_string()),
            cwe: None,
            owasp: None,
        }
    }

    /// Translate severity from Semgrep to hodei
    fn translate_severity(&self, severity: &str) -> String {
        match severity.to_lowercase().as_str() {
            "error" | "high" => "critical".to_string(),
            "warning" | "medium" => "major".to_string(),
            "info" | "low" => "minor".to_string(),
            _ => "info".to_string(),
        }
    }

    /// Translate patterns from Semgrep to hodei
    fn translate_patterns(
        &self,
        semgrep: &SemgrepRule,
        warnings: &mut Vec<TranslationWarning>,
    ) -> Vec<Pattern> {
        let mut patterns = Vec::new();

        // Handle simple pattern
        if let Some(pattern) = &semgrep.pattern {
            patterns.push(Pattern {
                pattern: pattern.clone(),
                message: semgrep
                    .message
                    .clone()
                    .unwrap_or_else(|| "Pattern match".to_string()),
            });
            return patterns;
        }

        // Handle patterns array
        if let Some(patterns_yaml) = &semgrep.patterns {
            for pattern_yaml in patterns_yaml {
                // Simple pattern extraction
                if let Ok(pattern_str) = serde_yaml::from_value::<String>(pattern_yaml.clone()) {
                    patterns.push(Pattern {
                        pattern: pattern_str,
                        message: semgrep
                            .message
                            .clone()
                            .unwrap_or_else(|| "Pattern match".to_string()),
                    });
                } else {
                    warnings.push(TranslationWarning {
                        rule_id: semgrep.id.clone(),
                        message: "Complex pattern structure not fully supported".to_string(),
                        severity: WarningSeverity::Warning,
                    });
                }
            }
        }

        if patterns.is_empty() {
            warnings.push(TranslationWarning {
                rule_id: semgrep.id.clone(),
                message: "No patterns found or patterns not in supported format".to_string(),
                severity: WarningSeverity::Error,
            });
        }

        patterns
    }

    /// Translate languages from Semgrep to hodei
    fn translate_languages(&self, semgrep: &SemgrepRule) -> Vec<String> {
        semgrep
            .languages
            .iter()
            .filter_map(|lang| self.language_mappings.get(lang).map(|s| s.to_string()))
            .collect()
    }

    /// Get statistics about translation
    pub fn get_stats(&self, results: &[TranslationResult]) -> TranslationStats {
        let total = results.len();
        let successful = results.iter().filter(|r| r.success).count();
        let total_warnings: usize = results.iter().map(|r| r.warnings.len()).sum();

        let error_count = results
            .iter()
            .flat_map(|r| &r.warnings)
            .filter(|w| w.severity == WarningSeverity::Error)
            .count();

        TranslationStats {
            total_rules: total,
            successful_translations: successful,
            failed_translations: total - successful,
            total_warnings,
            error_warnings: error_count,
            success_rate: if total > 0 {
                successful as f64 / total as f64
            } else {
                0.0
            },
        }
    }
}

impl Default for SemgrepTranslator {
    fn default() -> Self {
        Self::new()
    }
}

/// Translation statistics
#[derive(Debug, Clone)]
pub struct TranslationStats {
    pub total_rules: usize,
    pub successful_translations: usize,
    pub failed_translations: usize,
    pub total_warnings: usize,
    pub error_warnings: usize,
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_simple_rule() {
        let translator = SemgrepTranslator::new();

        let semgrep_yaml = r#"
id: python/sql-injection
message: "Potential SQL injection"
severity: ERROR
languages:
  - python
pattern: |
  $DB.execute(f"... {$VAR} ...")
"#;

        let result = translator.translate_rule(semgrep_yaml).unwrap();

        assert!(result.success);
        assert_eq!(
            result.hodei_rule.id,
            "HODEI-TRANSLATED-python/sql-injection"
        );
        assert_eq!(result.hodei_rule.patterns.len(), 1);
        assert_eq!(result.hodei_rule.languages.len(), 1);
        assert_eq!(result.hodei_rule.languages[0], "python");
    }

    #[test]
    fn test_translate_severity_mapping() {
        let translator = SemgrepTranslator::new();

        let test_cases = vec![
            ("ERROR", "critical"),
            ("WARNING", "major"),
            ("INFO", "minor"),
        ];

        for (input, expected) in test_cases {
            assert_eq!(translator.translate_severity(input), expected);
        }
    }

    #[test]
    fn test_language_mapping() {
        let translator = SemgrepTranslator::new();

        let semgrep_yaml = r#"
id: test
languages:
  - python
  - javascript
  - unknown
pattern: "x = $VAR"
"#;

        let result = translator.translate_rule(semgrep_yaml).unwrap();

        assert_eq!(result.hodei_rule.languages.len(), 2);
        assert!(result.hodei_rule.languages.contains(&"python".to_string()));
        assert!(
            result
                .hodei_rule
                .languages
                .contains(&"javascript".to_string())
        );
    }

    #[test]
    fn test_missing_pattern_warning() {
        let translator = SemgrepTranslator::new();

        let semgrep_yaml = r#"
id: test
languages:
  - python
"#;

        let result = translator.translate_rule(semgrep_yaml).unwrap();

        assert!(!result.warnings.is_empty());
        assert!(
            result
                .warnings
                .iter()
                .any(|w| w.message.contains("No patterns"))
        );
    }

    #[test]
    fn test_get_stats() {
        let translator = SemgrepTranslator::new();

        let results = vec![
            TranslationResult {
                hodei_rule: Rule {
                    id: "RULE-1".to_string(),
                    metadata: None,
                    languages: vec![],
                    patterns: vec![],
                    where_clause: None,
                    fix: None,
                    tests: None,
                },
                warnings: vec![],
                success: true,
            },
            TranslationResult {
                hodei_rule: Rule {
                    id: "RULE-2".to_string(),
                    metadata: None,
                    languages: vec![],
                    patterns: vec![],
                    where_clause: None,
                    fix: None,
                    tests: None,
                },
                warnings: vec![TranslationWarning {
                    rule_id: "RULE-2".to_string(),
                    message: "Test warning".to_string(),
                    severity: WarningSeverity::Warning,
                }],
                success: false,
            },
        ];

        let stats = translator.get_stats(&results);

        assert_eq!(stats.total_rules, 2);
        assert_eq!(stats.successful_translations, 1);
        assert_eq!(stats.failed_translations, 1);
        assert_eq!(stats.total_warnings, 1);
        assert_eq!(stats.success_rate, 0.5);
    }

    #[test]
    fn test_translate_multiple_patterns() {
        let translator = SemgrepTranslator::new();

        let semgrep_yaml = r#"
id: test
message: "Test"
languages:
  - python
patterns:
  - "pattern1"
  - "pattern2"
"#;

        let result = translator.translate_rule(semgrep_yaml).unwrap();

        assert_eq!(result.hodei_rule.patterns.len(), 2);
    }

    #[test]
    fn test_invalid_yaml() {
        let translator = SemgrepTranslator::new();

        let semgrep_yaml = "invalid: yaml: content: [";

        let result = translator.translate_rule(semgrep_yaml);

        assert!(result.is_err());
    }

    #[test]
    fn test_empty_rule_id() {
        let translator = SemgrepTranslator::new();

        let semgrep_yaml = r#"
id: ""
languages:
  - python
pattern: "x = 1"
"#;

        let result = translator.translate_rule(semgrep_yaml);

        assert!(result.is_err());
    }
}
