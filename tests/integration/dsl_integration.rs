//! Integration tests for hodei-dsl with hodei-ir

use hodei_dsl::parse_rule_file;
use hodei_ir::{Confidence, FactType, Severity};

#[cfg(test)]
mod dsl_integration_tests {
    use super::*;

    #[test]
    fn test_parse_simple_rule() {
        let rule_content = r#"
        rule "TestRule" {
            description: "Test rule"
            severity: "High"
            tags: ["test", "security"]

            match {
                pattern: Function {
                    name == "test"
                }
            }

            emit Finding {
                message: "Test finding"
                confidence: "High"
            }
        }
        "#;

        let result = parse_rule_file(rule_content);
        assert!(result.is_ok(), "Should parse simple rule successfully");

        let rule_file = result.unwrap();
        assert_eq!(rule_file.rules.len(), 1, "Should have one rule");

        let rule = &rule_file.rules[0];
        assert_eq!(rule.name, "TestRule");
        assert_eq!(rule.metadata.severity, Severity::High);
        assert!(rule.metadata.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_parse_multiple_rules() {
        let rule_content = r#"
        rule "Rule1" {
            description: "First rule"
            severity: "Medium"
            tags: ["test"]

            match {
                pattern: Function {
                    name == "func1"
                }
            }

            emit Finding {
                message: "Finding 1"
                confidence: "Medium"
            }
        }

        rule "Rule2" {
            description: "Second rule"
            severity: "Low"
            tags: ["test"]

            match {
                pattern: Variable {
                    name == "var1"
                }
            }

            emit Finding {
                message: "Finding 2"
                confidence: "Low"
            }
        }
        "#;

        let result = parse_rule_file(rule_content);
        assert!(result.is_ok(), "Should parse multiple rules successfully");

        let rule_file = result.unwrap();
        assert_eq!(rule_file.rules.len(), 2, "Should have two rules");
    }

    #[test]
    fn test_parse_rule_with_conditions() {
        let rule_content = r#"
        rule "ComplexRule" {
            description: "Rule with conditions"
            severity: "Critical"
            tags: ["security", "sql"]

            match {
                pattern: Function {
                    name == "query"
                    source_code contains "SELECT"
                }
                where: pattern.confidence == "High"
            }

            emit Finding {
                message: "SQL query detected"
                confidence: "High"
                metadata: {
                    category: "sql_injection"
                }
            }
        }
        "#;

        let result = parse_rule_file(rule_content);
        assert!(result.is_ok(), "Should parse rule with conditions");

        let rule_file = result.unwrap();
        let rule = &rule_file.rules[0];
        assert_eq!(rule.metadata.severity, Severity::Critical);
        assert!(rule.match_block.patterns.len() == 1);
        assert!(rule.match_block.where_clause.is_some());
    }

    #[test]
    fn test_empty_rule_file() {
        let rule_content = "";
        let result = parse_rule_file(rule_content);
        assert!(result.is_ok(), "Should parse empty file");
        assert_eq!(result.unwrap().rules.len(), 0);
    }

    #[test]
    fn test_parse_rule_with_all_metadata() {
        let rule_content = r#"
        rule "FullMetadata" {
            description: "Rule with all metadata fields"
            severity: "High"
            tags: ["tag1", "tag2", "tag3"]

            match {
                pattern: Function {
                    name == "test"
                }
            }

            emit Finding {
                message: "Full metadata test"
                confidence: "Medium"
                metadata: {
                    key1 = "value1"
                    key2 = 42
                    key3 = true
                }
            }
        }
        "#;

        let result = parse_rule_file(rule_content);
        assert!(result.is_ok(), "Should parse rule with full metadata");

        let rule_file = result.unwrap();
        let rule = &rule_file.rules[0];
        assert_eq!(rule.metadata.tags.len(), 3);
        assert_eq!(rule.emit_block.metadata.len(), 3);
    }

    #[test]
    fn test_rule_fact_types() {
        let test_cases = vec![
            (
                "TaintSink",
                FactType::TaintSink {
                    func: "test".to_string(),
                    consumes_flow: "flow1".to_string(),
                    category: "sink".to_string(),
                    severity: Severity::High,
                },
            ),
            (
                "TaintSource",
                FactType::TaintSource {
                    var: "var1".to_string(),
                    flow_id: "flow1".to_string(),
                    source_type: "input".to_string(),
                    confidence: Confidence::High,
                },
            ),
            (
                "Vulnerability",
                FactType::Vulnerability {
                    vuln_type: "sql_injection".to_string(),
                    severity: Severity::Critical,
                    location: "test".to_string(),
                    confidence: Confidence::High,
                },
            ),
            (
                "CodeSmell",
                FactType::CodeSmell {
                    smell_type: "long_method".to_string(),
                    severity: Severity::Medium,
                    message: "Method too long".to_string(),
                },
            ),
        ];

        for (fact_type_name, expected_fact) in test_cases {
            let rule_content = format!(
                r#"
                rule "Test{}" {{
                    description: "Test"
                    severity: "Medium"
                    tags: ["test"]

                    match {{
                        pattern: {} {{
                            name == "test"
                        }}
                    }}

                    emit Finding {{
                        message: "Test"
                        confidence: "Medium"
                    }}
                }}
                "#,
                fact_type_name, fact_type_name
            );

            let result = parse_rule_file(&rule_content);
            assert!(
                result.is_ok(),
                "Should parse rule with fact type: {}",
                fact_type_name
            );
        }
    }
}
