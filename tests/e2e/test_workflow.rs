//! End-to-End tests for test framework workflow

use hodei_test::domain::models::TestConfig;
use hodei_test::domain::ports::{TestConfigParser, SnapshotRepository};
use hodei_test::infrastructure::yaml_parser::YamlTestConfigParser;
use hodei_test::infrastructure::file_system_snapshot_repo::FileSystemSnapshotRepository;
use std::path::Path;
use tempfile::TempDir;

#[tokio::test]
async fn test_complete_test_workflow() {
    // Complete workflow: create test file, parse, run, verify results
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("complete_test.hodei.test");

    let test_yaml = r#"
rule: "password_strength.hodei"
language: "hodei-dsl"

cases:
  - name: "Strong password"
    code: |
      function validatePassword(pwd: string): boolean {
        return pwd.length >= 12;
      }
    expected_findings: []

  - name: "Weak password"
    code: |
      function validatePassword(pwd: string): boolean {
        return pwd.length >= 6;
      }
    expected_findings:
      - finding_type: "CodeSmell"
        severity: "Major"
        message: "Password too weak"
"#;

    // Write test file
    tokio::fs::write(&test_file, test_yaml).await.unwrap();

    // Parse test file
    let parser = YamlTestConfigParser::new();
    let config = parser.parse_file(&test_file).await.unwrap();

    assert_eq!(config.rule, "password_strength.hodei");
    assert_eq!(config.language, "hodei-dsl");
    assert_eq!(config.cases.len(), 2);

    // Verify test cases
    assert_eq!(config.cases[0].name, "Strong password");
    assert_eq!(config.cases[0].expected_findings.len(), 0);

    assert_eq!(config.cases[1].name, "Weak password");
    assert_eq!(config.cases[1].expected_findings.len(), 1);
}

#[tokio::test]
async fn test_snapshot_testing_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("snapshot_test.hodei.test");
    let snapshot_dir = temp_dir.path().join("snapshots");

    let test_yaml = r#"
rule: "test_rule.hodei"
language: "hodei-dsl"

cases:
  - name: "Test case 1"
    code: "test code 1"
    expected_findings:
      - finding_type: "Vulnerability"
        severity: "Major"
        message: "Finding 1"
"#;

    tokio::fs::write(&test_file, test_yaml).await.unwrap();
    tokio::fs::create_dir_all(&snapshot_dir).await.unwrap();

    // Create snapshot repository
    let repo = FileSystemSnapshotRepository::new(snapshot_dir.clone());

    // Parse test file
    let parser = YamlTestConfigParser::new();
    let config = parser.parse_file(&test_file).await.unwrap();

    // Simulate running tests and getting results
    let mut results = hodei_test::domain::models::TestResults::new();
    for case in &config.cases {
        let result = hodei_test::domain::models::TestCaseResult {
            name: case.name.clone(),
            passed: !case.expected_findings.is_empty(),
            assertions: Vec::new(),
            findings: Vec::new(),
        };
        results.add_result(result);
    }

    // Update snapshots
    let snapshot_manager = hodei_test::application::snapshot::SnapshotManager::new(repo, snapshot_dir.clone());
    snapshot_manager.update_snapshots(&results).await.unwrap();

    // Verify snapshot was created
    let mut dir = tokio::fs::read_dir(&snapshot_dir).await.unwrap();
    let mut snapshot_files: Vec<String> = Vec::new();
    while let Some(entry) = dir.next_entry().await.unwrap() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".snap") {
            snapshot_files.push(name);
        }
    }

    assert!(!snapshot_files.is_empty());

    // Verify snapshot content
    let loaded = FileSystemSnapshotRepository::new(snapshot_dir.clone())
        .load("Test case 1")
        .await
        .unwrap();

    assert!(loaded.is_some());
}

#[tokio::test]
async fn test_multiple_test_files_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let parser = YamlTestConfigParser::new();

    // Create multiple test files
    for i in 0..5 {
        let test_file = temp_dir.path().join(&format!("test{}.hodei.test", i));
        let yaml = format!(
            r#"
rule: "rule{}.hodei"
language: "hodei-dsl"

cases:
  - name: "Test {}"
    code: "test code {}"
    expected_findings: []
"#,
            i, i, i
        );
        tokio::fs::write(&test_file, yaml).await.unwrap();
    }

    // Parse all test files
    for i in 0..5 {
        let test_file = temp_dir.path().join(&format!("test{}.hodei.test", i));
        let config = parser.parse_file(&test_file).await.unwrap();

        assert_eq!(config.rule, format!("rule{}.hodei", i));
        assert_eq!(config.language, "hodei-dsl");
        assert_eq!(config.cases.len(), 1);
    }
}

#[tokio::test]
async fn test_test_results_aggregation() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("aggregation_test.hodei.test");

    let test_yaml = r#"
rule: "multi_case.hodei"
language: "hodei-dsl"

cases:
  - name: "Pass 1"
    code: "code 1"
    expected_findings: []
  
  - name: "Pass 2"
    code: "code 2"
    expected_findings: []
  
  - name: "Fail 1"
    code: "code 3"
    expected_findings:
      - finding_type: "Vulnerability"
        severity: "Critical"
        message: "Test"
  
  - name: "Pass 3"
    code: "code 4"
    expected_findings: []
  
  - name: "Fail 2"
    code: "code 5"
    expected_findings:
      - finding_type: "CodeSmell"
        severity: "Minor"
        message: "Test"
"#;

    tokio::fs::write(&test_file, test_yaml).await.unwrap();

    let parser = YamlTestConfigParser::new();
    let config = parser.parse_file(&test_file).await.unwrap();

    // Aggregate results
    let mut total_tests = 0;
    let mut passed_tests = 0;
    let mut failed_tests = 0;

    for case in &config.cases {
        total_tests += 1;
        if case.expected_findings.is_empty() {
            passed_tests += 1;
        } else {
            failed_tests += 1;
        }
    }

    assert_eq!(total_tests, 5);
    assert_eq!(passed_tests, 3);
    assert_eq!(failed_tests, 2);
}

#[tokio::test]
async fn test_test_with_unicode_content() {
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("unicode_test.hodei.test");

    let unicode_yaml = r#"
rule: "unicode.hodei"
language: "hodei-dsl"

cases:
  - name: "Tëst ñámé"
    code: "Héllo 世界 🌍"
    expected_findings: []
  
  - name: "Тест"  # Russian
    code: "Код"    # Code
    expected_findings: []
"#;

    tokio::fs::write(&test_file, unicode_yaml).await.unwrap();

    let parser = YamlTestConfigParser::new();
    let config = parser.parse_file(&test_file).await.unwrap();

    assert_eq!(config.cases.len(), 2);
    assert!(config.cases[0].name.contains("Tëst"));
    assert!(config.cases[1].name.contains("Тест"));
}
