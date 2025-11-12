//! Unit tests for domain models

use hodei_test::domain::models::{
    TestConfig, TestCase, ExpectedFinding, TestResults, TestCaseResult, 
    AssertionResult, ActualFinding, TestSnapshot
};

#[test]
fn test_test_config_creation() {
    let config = TestConfig {
        rule: "test_rule.hodei".to_string(),
        language: "hodei-dsl".to_string(),
        cases: Vec::new(),
    };
    
    assert_eq!(config.rule, "test_rule.hodei");
    assert_eq!(config.language, "hodei-dsl");
    assert!(config.cases.is_empty());
}

#[test]
fn test_test_case_creation() {
    let test_case = TestCase {
        name: "Test case".to_string(),
        code: "test code".to_string(),
        expected_findings: Vec::new(),
    };
    
    assert_eq!(test_case.name, "Test case");
    assert_eq!(test_case.code, "test code");
    assert!(test_case.expected_findings.is_empty());
}

#[test]
fn test_expected_finding_creation() {
    let finding = ExpectedFinding {
        finding_type: "Vulnerability".to_string(),
        severity: "Critical".to_string(),
        message: "Security issue".to_string(),
    };
    
    assert_eq!(finding.finding_type, "Vulnerability");
    assert_eq!(finding.severity, "Critical");
    assert_eq!(finding.message, "Security issue");
}

#[test]
fn test_expected_finding_equality() {
    let finding1 = ExpectedFinding {
        finding_type: "Vulnerability".to_string(),
        severity: "Critical".to_string(),
        message: "Security issue".to_string(),
    };
    
    let finding2 = ExpectedFinding {
        finding_type: "Vulnerability".to_string(),
        severity: "Critical".to_string(),
        message: "Security issue".to_string(),
    };
    
    assert_eq!(finding1, finding2);
}

#[test]
fn test_expected_finding_inequality() {
    let finding1 = ExpectedFinding {
        finding_type: "Vulnerability".to_string(),
        severity: "Critical".to_string(),
        message: "Security issue".to_string(),
    };
    
    let finding2 = ExpectedFinding {
        finding_type: "Vulnerability".to_string(),
        severity: "Major".to_string(),
        message: "Security issue".to_string(),
    };
    
    assert_ne!(finding1, finding2);
}

#[test]
fn test_test_results_new() {
    let results = TestResults::new();
    
    assert_eq!(results.total_tests, 0);
    assert_eq!(results.passed_tests, 0);
    assert_eq!(results.failed_tests, 0);
    assert!(results.case_results.is_empty());
}

#[test]
fn test_test_results_all_passed() {
    let mut results = TestResults::new();
    
    // Empty results should pass
    assert!(results.all_passed());
    
    // Add passing tests
    let passing_result = TestCaseResult {
        name: "pass1".to_string(),
        passed: true,
        assertions: Vec::new(),
    };
    results.add_result(passing_result);
    
    assert!(results.all_passed());
    
    // Add failing test
    let failing_result = TestCaseResult {
        name: "fail1".to_string(),
        passed: false,
        assertions: Vec::new(),
    };
    results.add_result(failing_result);
    
    assert!(!results.all_passed());
}

#[test]
fn test_test_case_result_creation() {
    let result = TestCaseResult {
        name: "Test result".to_string(),
        passed: true,
        assertions: Vec::new(),
    };
    
    assert_eq!(result.name, "Test result");
    assert!(result.passed);
    assert!(result.assertions.is_empty());
}

#[test]
fn test_assertion_result_creation() {
    let expected = ExpectedFinding {
        finding_type: "Vulnerability".to_string(),
        severity: "Critical".to_string(),
        message: "Test".to_string(),
    };
    
    let actual = ActualFinding {
        finding_type: "Vulnerability".to_string(),
        severity: "Critical".to_string(),
        message: "Test".to_string(),
    };
    
    let assertion = AssertionResult {
        expected: expected.clone(),
        actual: Some(actual.clone()),
        passed: true,
    };
    
    assert_eq!(assertion.expected, expected);
    assert_eq!(assertion.actual, Some(actual));
    assert!(assertion.passed);
}

#[test]
fn test_assertion_result_no_actual() {
    let expected = ExpectedFinding {
        finding_type: "Vulnerability".to_string(),
        severity: "Critical".to_string(),
        message: "Test".to_string(),
    };
    
    let assertion = AssertionResult {
        expected: expected.clone(),
        actual: None,
        passed: false,
    };
    
    assert_eq!(assertion.expected, expected);
    assert!(assertion.actual.is_none());
    assert!(!assertion.passed);
}

#[test]
fn test_actual_finding_creation() {
    let actual = ActualFinding {
        finding_type: "Vulnerability".to_string(),
        severity: "Critical".to_string(),
        message: "Found vulnerability".to_string(),
    };
    
    assert_eq!(actual.finding_type, "Vulnerability");
    assert_eq!(actual.severity, "Critical");
    assert_eq!(actual.message, "Found vulnerability");
}

#[test]
fn test_actual_finding_equality() {
    let actual1 = ActualFinding {
        finding_type: "Vulnerability".to_string(),
        severity: "Critical".to_string(),
        message: "Test".to_string(),
    };
    
    let actual2 = ActualFinding {
        finding_type: "Vulnerability".to_string(),
        severity: "Critical".to_string(),
        message: "Test".to_string(),
    };
    
    assert_eq!(actual1, actual2);
}

#[test]
fn test_actual_finding_inequality() {
    let actual1 = ActualFinding {
        finding_type: "Vulnerability".to_string(),
        severity: "Critical".to_string(),
        message: "Test1".to_string(),
    };
    
    let actual2 = ActualFinding {
        finding_type: "Vulnerability".to_string(),
        severity: "Critical".to_string(),
        message: "Test2".to_string(),
    };
    
    assert_ne!(actual1, actual2);
}

#[test]
fn test_test_snapshot_creation() {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("key".to_string(), "value".to_string());
    
    let snapshot = TestSnapshot {
        test_name: "test_snapshot".to_string(),
        findings: vec![
            ExpectedFinding {
                finding_type: "Vulnerability".to_string(),
                severity: "Critical".to_string(),
                message: "Test".to_string(),
            }
        ],
        metadata: metadata.clone(),
    };
    
    assert_eq!(snapshot.test_name, "test_snapshot");
    assert_eq!(snapshot.findings.len(), 1);
    assert_eq!(snapshot.metadata, metadata);
}

#[test]
fn test_test_snapshot_empty_findings() {
    let snapshot = TestSnapshot {
        test_name: "empty_snapshot".to_string(),
        findings: Vec::new(),
        metadata: std::collections::HashMap::new(),
    };
    
    assert_eq!(snapshot.test_name, "empty_snapshot");
    assert!(snapshot.findings.is_empty());
    assert!(snapshot.metadata.is_empty());
}

#[test]
fn test_test_case_with_multiple_expected_findings() {
    let finding1 = ExpectedFinding {
        finding_type: "Vulnerability".to_string(),
        severity: "Critical".to_string(),
        message: "Issue 1".to_string(),
    };
    
    let finding2 = ExpectedFinding {
        finding_type: "CodeSmell".to_string(),
        severity: "Minor".to_string(),
        message: "Issue 2".to_string(),
    };
    
    let test_case = TestCase {
        name: "Multiple findings".to_string(),
        code: "code with issues".to_string(),
        expected_findings: vec![finding1, finding2],
    };
    
    assert_eq!(test_case.expected_findings.len(), 2);
    assert_eq!(test_case.expected_findings[0].finding_type, "Vulnerability");
    assert_eq!(test_case.expected_findings[1].finding_type, "CodeSmell");
}

#[test]
fn test_test_results_multiple_cases() {
    let mut results = TestResults::new();
    
    // Add multiple test cases
    for i in 0..10 {
        let result = TestCaseResult {
            name: format!("test_{}", i),
            passed: i % 2 == 0, // Even indices pass, odd fail
            assertions: Vec::new(),
        };
        results.add_result(result);
    }
    
    assert_eq!(results.total_tests, 10);
    assert_eq!(results.passed_tests, 5); // 0, 2, 4, 6, 8
    assert_eq!(results.failed_tests, 5); // 1, 3, 5, 7, 9
    assert!(!results.all_passed());
}

#[test]
fn test_assertion_result_passed_state() {
    let expected = ExpectedFinding {
        finding_type: "Vulnerability".to_string(),
        severity: "Critical".to_string(),
        message: "Test".to_string(),
    };
    
    // Passed assertion
    let passed_assertion = AssertionResult {
        expected: expected.clone(),
        actual: Some(ActualFinding {
            finding_type: "Vulnerability".to_string(),
            severity: "Critical".to_string(),
            message: "Test".to_string(),
        }),
        passed: true,
    };
    assert!(passed_assertion.passed);
    
    // Failed assertion - actual doesn't match expected
    let failed_assertion = AssertionResult {
        expected: expected.clone(),
        actual: Some(ActualFinding {
            finding_type: "CodeSmell".to_string(),
            severity: "Minor".to_string(),
            message: "Different".to_string(),
        }),
        passed: false,
    };
    assert!(!failed_assertion.passed);
    
    // Failed assertion - no actual finding
    let no_actual_assertion = AssertionResult {
        expected: expected.clone(),
        actual: None,
        passed: false,
    };
    assert!(!no_actual_assertion.passed);
}
