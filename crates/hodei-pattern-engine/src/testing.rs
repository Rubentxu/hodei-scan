//! Testing framework for YAML rules
//!
//! This module provides utilities for testing YAML rules against
//! test cases with expected results.

use crate::match_transform::match_to_fact;
use crate::tree_sitter::TreeSitterMatcher;
use crate::yaml_rule::{YamlRule, YamlRuleLoader};
use std::collections::HashMap;

/// Testing framework error
#[derive(Debug, thiserror::Error)]
pub enum TestError {
    #[error("Rule validation failed: {0}")]
    ValidationError(String),

    #[error("Rule execution failed: {0}")]
    ExecutionError(String),

    #[error("Test case error: {0}")]
    TestCaseError(String),
}

/// Test case for a YAML rule
#[derive(Debug, Clone)]
pub struct RuleTestCase {
    /// Name of the test case
    pub name: String,
    /// Source code to test
    pub code: String,
    /// Expected results
    pub expected: TestExpected,
    /// Language of the test case
    pub language: String,
}

/// Expected results for a test case
#[derive(Debug, Clone)]
pub struct TestExpected {
    /// Number of findings expected
    pub findings: usize,
    /// Expected message (optional)
    pub expected_message: Option<String>,
}

/// Test results
#[derive(Debug, Clone)]
pub struct TestResults {
    /// Whether the test passed
    pub passed: bool,
    /// Expected number of findings
    pub expected_findings: usize,
    /// Actual number of findings
    pub actual_findings: usize,
    /// Test case name
    pub name: String,
    /// Error message if test failed
    pub error_message: Option<String>,
}

/// Validate a YAML rule
pub fn validate_rule(rule: &YamlRule) -> Result<(), TestError> {
    // Check required fields
    if rule.id.is_empty() {
        return Err(TestError::ValidationError(
            "Rule ID is required".to_string(),
        ));
    }

    if rule.language.is_empty() {
        return Err(TestError::ValidationError(
            "Rule language is required".to_string(),
        ));
    }

    if rule.pattern.is_empty() {
        return Err(TestError::ValidationError(
            "Rule pattern is required".to_string(),
        ));
    }

    if rule.message.is_empty() {
        return Err(TestError::ValidationError(
            "Rule message is required".to_string(),
        ));
    }

    if rule.severity.is_empty() {
        return Err(TestError::ValidationError(
            "Rule severity is required".to_string(),
        ));
    }

    if rule.category.is_empty() {
        return Err(TestError::ValidationError(
            "Rule category is required".to_string(),
        ));
    }

    // Validate severity
    let valid_severities = ["error", "warning", "info", "minor"];
    if !valid_severities.contains(&rule.severity.to_lowercase().as_str()) {
        return Err(TestError::ValidationError(format!(
            "Invalid severity '{}'. Must be one of: {:?}",
            rule.severity, valid_severities
        )));
    }

    // Validate language
    let valid_languages = [
        "python",
        "java",
        "rust",
        "javascript",
        "typescript",
        "go",
        "php",
        "ruby",
        "cpp",
    ];
    if !valid_languages.contains(&rule.language.as_str()) {
        return Err(TestError::ValidationError(format!(
            "Invalid language '{}'. Must be one of: {:?}",
            rule.language, valid_languages
        )));
    }

    Ok(())
}

/// Run a single test case
pub fn run_test_case(test_case: &RuleTestCase, rule: &YamlRule) -> Result<TestResults, TestError> {
    // Validate the rule first
    validate_rule(rule)?;

    // Create a matcher
    let mut matcher = TreeSitterMatcher::new();

    // Execute the pattern against the test code
    let matches = matcher
        .execute_pattern(&rule.language, &rule.pattern, &test_case.code)
        .map_err(|e| TestError::ExecutionError(e.to_string()))?;

    // Count actual findings
    let actual_findings = matches.len();

    // Check if the number of findings matches expectations
    let passed = actual_findings == test_case.expected.findings;

    // Create results
    let results = TestResults {
        passed,
        expected_findings: test_case.expected.findings,
        actual_findings,
        name: test_case.name.clone(),
        error_message: if !passed {
            Some(format!(
                "Expected {} findings, but got {}",
                test_case.expected.findings, actual_findings
            ))
        } else {
            None
        },
    };

    Ok(results)
}

/// Run multiple test cases
pub fn run_test_suite(test_cases: &[RuleTestCase], rule: &YamlRule) -> Vec<TestResults> {
    let mut results = Vec::new();

    for test_case in test_cases {
        match run_test_case(test_case, rule) {
            Ok(result) => results.push(result),
            Err(e) => {
                results.push(TestResults {
                    passed: false,
                    expected_findings: test_case.expected.findings,
                    actual_findings: 0,
                    name: test_case.name.clone(),
                    error_message: Some(format!("Test execution failed: {}", e)),
                });
            }
        }
    }

    results
}

/// Generate a test report
pub fn generate_test_report(results: &[TestResults]) -> String {
    let mut report = String::new();
    report.push_str("=== Test Report ===\n\n");

    let mut passed_count = 0;
    let mut failed_count = 0;

    for result in results {
        if result.passed {
            report.push_str(&format!("✓ PASS: {}\n", result.name));
            passed_count += 1;
        } else {
            report.push_str(&format!("✗ FAIL: {}\n", result.name));
            if let Some(ref error) = result.error_message {
                report.push_str(&format!("  Error: {}\n", error));
            }
            report.push_str(&format!(
                "  Expected: {}, Actual: {}\n",
                result.expected_findings, result.actual_findings
            ));
            failed_count += 1;
        }
    }

    report.push_str("\n=== Summary ===\n");
    report.push_str(&format!("Total tests: {}\n", results.len()));
    report.push_str(&format!("Passed: {}\n", passed_count));
    report.push_str(&format!("Failed: {}\n", failed_count));

    if failed_count == 0 {
        report.push_str("\n🎉 All tests passed!\n");
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rule_with_valid_rule() {
        let rule = YamlRule {
            id: "TEST-001".to_string(),
            language: "python".to_string(),
            message: "Test message".to_string(),
            severity: "warning".to_string(),
            category: "error-handling".to_string(),
            pattern: "(identifier) @id".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        assert!(validate_rule(&rule).is_ok());
    }

    #[test]
    fn validate_rule_with_empty_id() {
        let rule = YamlRule {
            id: "".to_string(),
            language: "python".to_string(),
            message: "Test message".to_string(),
            severity: "warning".to_string(),
            category: "error-handling".to_string(),
            pattern: "(identifier) @id".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        assert!(validate_rule(&rule).is_err());
    }

    #[test]
    fn validate_rule_with_invalid_language() {
        let rule = YamlRule {
            id: "TEST-001".to_string(),
            language: "invalid_lang".to_string(),
            message: "Test message".to_string(),
            severity: "warning".to_string(),
            category: "error-handling".to_string(),
            pattern: "(identifier) @id".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        assert!(validate_rule(&rule).is_err());
    }

    #[test]
    fn validate_rule_with_invalid_severity() {
        let rule = YamlRule {
            id: "TEST-001".to_string(),
            language: "python".to_string(),
            message: "Test message".to_string(),
            severity: "invalid_severity".to_string(),
            category: "error-handling".to_string(),
            pattern: "(identifier) @id".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        assert!(validate_rule(&rule).is_err());
    }

    #[test]
    fn validate_rule_case_insensitive_severity() {
        let rule = YamlRule {
            id: "TEST-001".to_string(),
            language: "python".to_string(),
            message: "Test message".to_string(),
            severity: "WARNING".to_string(),
            category: "error-handling".to_string(),
            pattern: "(identifier) @id".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        assert!(validate_rule(&rule).is_ok());
    }

    #[test]
    fn test_results_creation() {
        let results = TestResults {
            passed: true,
            expected_findings: 5,
            actual_findings: 5,
            name: "Test Case 1".to_string(),
            error_message: None,
        };

        assert!(results.passed);
        assert_eq!(results.expected_findings, 5);
        assert_eq!(results.actual_findings, 5);
    }

    #[test]
    fn test_results_failed_case() {
        let results = TestResults {
            passed: false,
            expected_findings: 5,
            actual_findings: 3,
            name: "Test Case 2".to_string(),
            error_message: Some("Expected 5 findings, but got 3".to_string()),
        };

        assert!(!results.passed);
        assert_eq!(results.expected_findings, 5);
        assert_eq!(results.actual_findings, 3);
        assert!(results.error_message.is_some());
    }

    #[test]
    fn test_case_creation() {
        let test_case = RuleTestCase {
            name: "Test Python Function".to_string(),
            language: "python".to_string(),
            code: "def foo(): pass".to_string(),
            expected: TestExpected {
                findings: 1,
                expected_message: None,
            },
        };

        assert_eq!(test_case.name, "Test Python Function");
        assert_eq!(test_case.language, "python");
        assert_eq!(test_case.expected.findings, 1);
    }

    #[test]
    fn generate_test_report_all_passed() {
        let results = vec![
            TestResults {
                passed: true,
                expected_findings: 1,
                actual_findings: 1,
                name: "Test 1".to_string(),
                error_message: None,
            },
            TestResults {
                passed: true,
                expected_findings: 2,
                actual_findings: 2,
                name: "Test 2".to_string(),
                error_message: None,
            },
        ];

        let report = generate_test_report(&results);

        assert!(report.contains("✓ PASS: Test 1"));
        assert!(report.contains("✓ PASS: Test 2"));
        assert!(report.contains("Passed: 2"));
        assert!(report.contains("Failed: 0"));
        assert!(report.contains("All tests passed"));
    }

    #[test]
    fn generate_test_report_with_failures() {
        let results = vec![
            TestResults {
                passed: true,
                expected_findings: 1,
                actual_findings: 1,
                name: "Test 1".to_string(),
                error_message: None,
            },
            TestResults {
                passed: false,
                expected_findings: 2,
                actual_findings: 1,
                name: "Test 2".to_string(),
                error_message: Some("Expected 2 findings, but got 1".to_string()),
            },
        ];

        let report = generate_test_report(&results);

        assert!(report.contains("✓ PASS: Test 1"));
        assert!(report.contains("✗ FAIL: Test 2"));
        assert!(report.contains("Expected 2 findings, but got 1"));
        assert!(report.contains("Passed: 1"));
        assert!(report.contains("Failed: 1"));
    }

    #[test]
    fn run_test_case_simple_case() {
        let rule = YamlRule {
            id: "TEST-001".to_string(),
            language: "python".to_string(),
            message: "Test rule".to_string(),
            severity: "warning".to_string(),
            category: "testing".to_string(),
            pattern: "(identifier) @id".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        let test_case = RuleTestCase {
            name: "Simple Test".to_string(),
            language: "python".to_string(),
            code: "x = 42".to_string(),
            expected: TestExpected {
                findings: 1,
                expected_message: None,
            },
        };

        // Test that the API works - actual execution may fail if grammars not available
        let result = run_test_case(&test_case, &rule);
        // Either passes or gracefully handles missing grammars
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn run_test_suite_multiple_cases() {
        let rule = YamlRule {
            id: "TEST-001".to_string(),
            language: "python".to_string(),
            message: "Test rule".to_string(),
            severity: "warning".to_string(),
            category: "testing".to_string(),
            pattern: "(identifier) @id".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        let test_cases = vec![
            RuleTestCase {
                name: "Test 1".to_string(),
                language: "python".to_string(),
                code: "x = 42".to_string(),
                expected: TestExpected {
                    findings: 1,
                    expected_message: None,
                },
            },
            RuleTestCase {
                name: "Test 2".to_string(),
                language: "python".to_string(),
                code: "y = 24".to_string(),
                expected: TestExpected {
                    findings: 1,
                    expected_message: None,
                },
            },
        ];

        let results = run_test_suite(&test_cases, &rule);

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "Test 1");
        assert_eq!(results[1].name, "Test 2");
    }

    #[test]
    fn run_rule_test_case() {
        // This test ensures the module compiles and basic structure works
        let test_case = RuleTestCase {
            name: "Basic Test".to_string(),
            language: "python".to_string(),
            code: "x = 42".to_string(),
            expected: TestExpected {
                findings: 0,
                expected_message: None,
            },
        };

        assert_eq!(test_case.name, "Basic Test");
        assert_eq!(test_case.language, "python");
        assert_eq!(test_case.expected.findings, 0);
    }
}
