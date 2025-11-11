use hodei_pattern_engine::yaml_rule::{parse_yaml_rule, YamlRule, YamlRuleLoader};

#[test]
fn test_load_yaml_rule() {
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
    assert_eq!(rule.severity, "warning");
    assert_eq!(rule.category, "error-handling");
}

#[test]
fn test_validate_required_fields() {
    let yaml = r#"
id: TEST-001
language: python
message: "Test message"
severity: info
category: testing
pattern: "(identifier) @id"
"#;

    let rule = parse_yaml_rule(yaml).unwrap();
    assert!(rule.validate().is_ok());
}

#[test]
fn test_validate_missing_id() {
    let yaml = r#"
language: java
message: "Test"
severity: warning
pattern: "test"
"#;

    let result = parse_yaml_rule(yaml);
    assert!(result.is_err());
}

#[test]
fn test_validate_missing_pattern() {
    let yaml = r#"
id: TEST-002
language: java
message: "Test"
severity: warning
"#;

    let result = parse_yaml_rule(yaml);
    assert!(result.is_err());
}

#[test]
fn test_rule_loader_new() {
    let loader = YamlRuleLoader::new();
    // Just ensure it can be created
    assert!(true);
}
