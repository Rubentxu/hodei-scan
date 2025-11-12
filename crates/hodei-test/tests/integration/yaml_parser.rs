//! Integration tests for YAML parser

use hodei_test::infrastructure::yaml_parser::YamlTestConfigParser;
use tempfile::TempDir;
use std::path::Path;
use crate::fixtures::SAMPLE_TEST_YAML;

#[tokio::test]
async fn test_parse_simple_yaml() {
    let parser = YamlTestConfigParser::new();
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("simple_test.hodei.test");
    
    let yaml_content = r#"
rule: "test_rule.hodei"
language: "hodei-dsl"

cases:
  - name: "Test case"
    code: "test code"
    expected_findings: []
"#;
    
    tokio::fs::write(&test_file, yaml_content).await.unwrap();
    
    let config = parser.parse_file(&test_file).await.unwrap();
    
    assert_eq!(config.rule, "test_rule.hodei");
    assert_eq!(config.language, "hodei-dsl");
    assert_eq!(config.cases.len(), 1);
    assert_eq!(config.cases[0].name, "Test case");
}

#[tokio::test]
async fn test_parse_yaml_with_findings() {
    let parser = YamlTestConfigParser::new();
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("findings_test.hodei.test");
    
    let yaml_content = r#"
rule: "sql_injection.hodei"
language: "hodei-dsl"

cases:
  - name: "Vulnerable code"
    code: "SELECT * FROM users WHERE id = \" + userInput"
    expected_findings:
      - finding_type: "Vulnerability"
        severity: "Critical"
        message: "SQL injection detected"
"#;
    
    tokio::fs::write(&test_file, yaml_content).await.unwrap();
    
    let config = parser.parse_file(&test_file).await.unwrap();
    
    assert_eq!(config.rule, "sql_injection.hodei");
    assert_eq!(config.cases.len(), 1);
    assert_eq!(config.cases[0].expected_findings.len(), 1);
    
    let finding = &config.cases[0].expected_findings[0];
    assert_eq!(finding.finding_type, "Vulnerability");
    assert_eq!(finding.severity, "Critical");
    assert_eq!(finding.message, "SQL injection detected");
}

#[tokio::test]
async fn test_parse_yaml_multiple_cases() {
    let parser = YamlTestConfigParser::new();
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("multi_case.hodei.test");
    
    let yaml_content = r#"
rule: "auth_rules.hodei"
language: "hodei-dsl"

cases:
  - name: "Case 1"
    code: "code1"
    expected_findings: []
  
  - name: "Case 2"
    code: "code2"
    expected_findings:
      - finding_type: "CodeSmell"
        severity: "Minor"
        message: "Test finding"
  
  - name: "Case 3"
    code: "code3"
    expected_findings: []
"#;
    
    tokio::fs::write(&test_file, yaml_content).await.unwrap();
    
    let config = parser.parse_file(&test_file).await.unwrap();
    
    assert_eq!(config.cases.len(), 3);
    assert_eq!(config.cases[0].name, "Case 1");
    assert_eq!(config.cases[1].name, "Case 2");
    assert_eq!(config.cases[2].name, "Case 3");
}

#[tokio::test]
async fn test_parse_yaml_multiline_code() {
    let parser = YamlTestConfigParser::new();
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("multiline.hodei.test");
    
    let yaml_content = r#"
rule: "password_strength.hodei"
language: "hodei-dsl"

cases:
  - name: "Strong password"
    code: |
      function validatePassword(pwd: string): boolean {
        if (pwd.length >= 12 && pwd.matches(/[A-Z]/) && pwd.matches(/[0-9]/)) {
          return true;
        }
        return false;
      }
    expected_findings: []
"#;
    
    tokio::fs::write(&test_file, yaml_content).await.unwrap();
    
    let config = parser.parse_file(&test_file).await.unwrap();
    
    assert_eq!(config.cases.len(), 1);
    let code = &config.cases[0].code;
    assert!(code.contains("function validatePassword"));
    assert!(code.contains("pwd.length >= 12"));
}

#[tokio::test]
async fn test_parse_yaml_empty_cases() {
    let parser = YamlTestConfigParser::new();
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("empty_cases.hodei.test");
    
    let yaml_content = r#"
rule: "empty.hodei"
language: "hodei-dsl"

cases: []
"#;
    
    tokio::fs::write(&test_file, yaml_content).await.unwrap();
    
    let config = parser.parse_file(&test_file).await.unwrap();
    
    assert!(config.cases.is_empty());
}

#[tokio::test]
async fn test_parse_yaml_special_characters() {
    let parser = YamlTestConfigParser::new();
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("special.hodei.test");
    
    let yaml_content = r#"
rule: "special_chars.hodei"
language: "hodei-dsl"

cases:
  - name: "Test with quotes"
    code: "test \"quoted\" code"
    expected_findings: []
  
  - name: "Test with newline"
    code: "line1\nline2"
    expected_findings: []
"#;
    
    tokio::fs::write(&test_file, yaml_content).await.unwrap();
    
    let config = parser.parse_file(&test_file).await.unwrap();
    
    assert_eq!(config.cases.len(), 2);
    assert!(config.cases[0].code.contains("quoted"));
    assert!(config.cases[1].code.contains("line1"));
}

#[tokio::test]
async fn test_parse_nonexistent_file() {
    let parser = YamlTestConfigParser::new();
    let nonexistent = Path::new("/nonexistent/test.hodei.test");
    
    let result = parser.parse_file(nonexistent).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_parse_invalid_yaml() {
    let parser = YamlTestConfigParser::new();
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("invalid.hodei.test");
    
    let invalid_yaml = r#"
rule: "test.hodei"
language: "hodei-dsl"
invalid yaml syntax
  - indented incorrectly
"#;
    
    tokio::fs::write(&test_file, invalid_yaml).await.unwrap();
    
    let result = parser.parse_file(&test_file).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_parse_sample_yaml() {
    let parser = YamlTestConfigParser::new();
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("sample.hodei.test");
    
    tokio::fs::write(&test_file, SAMPLE_TEST_YAML).await.unwrap();
    
    let config = parser.parse_file(&test_file).await.unwrap();
    
    assert_eq!(config.rule, "password_strength.hodei");
    assert_eq!(config.language, "hodei-dsl");
    assert_eq!(config.cases.len(), 3);
    
    // Verify each case
    assert_eq!(config.cases[0].name, "Strong password");
    assert!(config.cases[0].expected_findings.is_empty());
    
    assert_eq!(config.cases[1].name, "Weak password - too short");
    assert_eq!(config.cases[1].expected_findings.len(), 1);
    
    assert_eq!(config.cases[2].name, "Weak password - no numbers");
    assert_eq!(config.cases[2].expected_findings.len(), 1);
}

#[tokio::test]
async fn test_parse_yaml_with_metadata() {
    let parser = YamlTestConfigParser::new();
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("metadata.hodei.test");
    
    let yaml_content = r#"
rule: "metadata_test.hodei"
language: "hodei-dsl"
metadata:
  version: "1.0"
  author: "test"

cases:
  - name: "Test"
    code: "code"
    expected_findings: []
"#;
    
    tokio::fs::write(&test_file, yaml_content).await.unwrap();
    
    let result = parser.parse_file(&test_file).await;
    
    // Should parse despite extra metadata field
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_parse_yaml_unicode() {
    let parser = YamlTestConfigParser::new();
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("unicode.hodei.test");
    
    let yaml_content = r#"
rule: "unicode.hodei"
language: "hodei-dsl"

cases:
  - name: "Tëst ñámé"
    code: "Héllo 世界 🌍"
    expected_findings: []
"#;
    
    tokio::fs::write(&test_file, yaml_content).await.unwrap();
    
    let config = parser.parse_file(&test_file).await.unwrap();
    
    assert_eq!(config.cases[0].name, "Tëst ñámé");
    assert!(config.cases[0].code.contains("世界"));
    assert!(config.cases[0].code.contains("🌍"));
}
