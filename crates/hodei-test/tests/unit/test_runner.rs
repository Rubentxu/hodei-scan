//! Unit tests for test runner

use hodei_test::application::test_runner::HodeiTestRunner;
use hodei_test::domain::models::{TestResults, TestCaseResult};
use hodei_test::domain::ports::{TestConfigParser, TestCaseRunner, ResultComparator};
use crate::fixtures::{simple_test_config, multi_case_config, MockTestCaseRunner, MockResultComparator};
use tempfile::TempDir;
use std::path::Path;

struct MockParser;
struct MockRunner;
struct MockComparator;

#[async_trait::async_trait]
impl TestConfigParser for MockParser {
    async fn parse_file(&self, _path: &Path) -> Result<hodei_test::domain::models::TestConfig, anyhow::Error> {
        Ok(simple_test_config())
    }
}

#[async_trait::async_trait]
impl TestCaseRunner for MockRunner {
    async fn run_case(&self, test_case: &crate::fixtures::TestCase, _rule_path: &str) -> Result<TestCaseResult, anyhow::Error> {
        Ok(TestCaseResult {
            name: test_case.name.clone(),
            passed: true,
            assertions: Vec::new(),
        })
    }
}

#[async_trait::async_trait]
impl ResultComparator for MockComparator {
    async fn compare(
        &self,
        _actual: &[hodei_ir::Finding],
        _expected: &[crate::fixtures::ExpectedFinding],
    ) -> Vec<hodei_test::domain::models::AssertionResult> {
        Vec::new()
    }
}

#[tokio::test]
async fn test_test_runner_creation() {
    let parser = MockParser;
    let runner = MockRunner;
    let comparator = MockComparator;
    
    let test_runner = HodeiTestRunner::new(parser, runner, comparator);
    
    assert!(true); // If we can create it, it worked
}

#[tokio::test]
async fn test_run_single_test_file() {
    let parser = MockParser;
    let runner = MockRunner;
    let comparator = MockComparator;
    
    let test_runner = HodeiTestRunner::new(parser, runner, comparator);
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.hodei.test");
    
    let results = test_runner
        .run_test_file(&test_file, "rule.hodei")
        .await
        .expect("Should run tests");
    
    assert_eq!(results.total_tests, 1);
    assert_eq!(results.passed_tests, 1);
    assert_eq!(results.failed_tests, 0);
    assert!(results.all_passed());
}

#[tokio::test]
async fn test_run_multiple_test_cases() {
    let parser = MockParser;
    let runner = MockRunner;
    let comparator = MockComparator;
    
    let test_runner = HodeiTestRunner::new(parser, runner, comparator);
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.hodei.test");
    
    let results = test_runner
        .run_test_file(&test_file, "rule.hodei")
        .await
        .expect("Should run tests");
    
    // Should have test results for all cases
    assert_eq!(results.case_results.len(), 1);
}

#[tokio::test]
async fn test_test_results_add_result() {
    let mut results = TestResults::new();
    
    // Initially empty
    assert_eq!(results.total_tests, 0);
    assert_eq!(results.passed_tests, 0);
    assert_eq!(results.failed_tests, 0);
    assert!(results.all_passed());
    
    // Add a passing test
    let passing_result = TestCaseResult {
        name: "passing_test".to_string(),
        passed: true,
        assertions: Vec::new(),
    };
    results.add_result(passing_result);
    
    assert_eq!(results.total_tests, 1);
    assert_eq!(results.passed_tests, 1);
    assert_eq!(results.failed_tests, 0);
    assert!(results.all_passed());
    
    // Add a failing test
    let failing_result = TestCaseResult {
        name: "failing_test".to_string(),
        passed: false,
        assertions: Vec::new(),
    };
    results.add_result(failing_result);
    
    assert_eq!(results.total_tests, 2);
    assert_eq!(results.passed_tests, 1);
    assert_eq!(results.failed_tests, 1);
    assert!(!results.all_passed());
}

#[tokio::test]
async fn test_test_results_all_passed() {
    let mut results = TestResults::new();
    
    // All passing
    for i in 0..5 {
        let result = TestCaseResult {
            name: format!("test_{}", i),
            passed: true,
            assertions: Vec::new(),
        };
        results.add_result(result);
    }
    
    assert!(results.all_passed());
    
    // Mix of passing and failing
    let failing_result = TestCaseResult {
        name: "failing_test".to_string(),
        passed: false,
        assertions: Vec::new(),
    };
    results.add_result(failing_result);
    
    assert!(!results.all_passed());
}

#[tokio::test]
async fn test_test_results_empty() {
    let results = TestResults::new();
    
    assert_eq!(results.total_tests, 0);
    assert_eq!(results.passed_tests, 0);
    assert_eq!(results.failed_tests, 0);
    assert!(results.all_passed());
}

#[tokio::test]
async fn test_run_all_tests_directory() {
    let parser = MockParser;
    let runner = MockRunner;
    let comparator = MockComparator;
    
    let test_runner = HodeiTestRunner::new(parser, runner, comparator);
    
    let temp_dir = TempDir::new().unwrap();
    
    let results = test_runner
        .run_all_tests(temp_dir.path(), "rule.hodei")
        .await
        .expect("Should run all tests");
    
    // Results will be empty if no test files
    assert_eq!(results.total_tests, 0);
}

#[tokio::test]
async fn test_test_runner_with_mock_runner() {
    let parser = MockParser;
    let runner = MockTestCaseRunner::new()
        .with_findings("Strong password", hodei_ir::FindingSet { findings: Vec::new() });
    let comparator = MockComparator;
    
    let test_runner = HodeiTestRunner::new(parser, runner, comparator);
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.hodei.test");
    
    let results = test_runner
        .run_test_file(&test_file, "rule.hodei")
        .await
        .expect("Should run tests");
    
    assert_eq!(results.total_tests, 1);
}

#[tokio::test]
async fn test_test_results_ordering() {
    let mut results = TestResults::new();
    
    // Add tests in specific order
    let test_names = vec!["zebra", "apple", "banana"];
    
    for name in test_names {
        let result = TestCaseResult {
            name: name.to_string(),
            passed: true,
            assertions: Vec::new(),
        };
        results.add_result(result);
    }
    
    // Should maintain insertion order
    assert_eq!(results.case_results.len(), 3);
    assert_eq!(results.case_results[0].name, "zebra");
    assert_eq!(results.case_results[1].name, "apple");
    assert_eq!(results.case_results[2].name, "banana");
}
