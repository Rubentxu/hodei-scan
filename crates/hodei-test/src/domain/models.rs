//! Domain models
//!
//! Core data structures for rule testing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for a test file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    /// Path to the rule file to test
    pub rule: String,

    /// Programming language of the code snippets
    pub language: String,

    /// Test cases to run
    pub cases: Vec<TestCase>,
}

/// A single test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Name of the test case
    pub name: String,

    /// Code snippet to analyze
    pub code: String,

    /// Expected findings from the rule
    pub expected_findings: Vec<ExpectedFinding>,
}

/// An expected finding from a test
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpectedFinding {
    /// Type of finding
    pub finding_type: String,

    /// Severity level
    pub severity: String,

    /// Error message
    pub message: String,
}

/// Results from running a test suite
#[derive(Debug, Clone)]
pub struct TestResults {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub case_results: Vec<TestCaseResult>,
}

impl TestResults {
    pub fn new() -> Self {
        Self {
            total_tests: 0,
            passed_tests: 0,
            failed_tests: 0,
            case_results: Vec::new(),
        }
    }

    pub fn add_result(&mut self, result: TestCaseResult) {
        self.total_tests += 1;
        if result.passed {
            self.passed_tests += 1;
        } else {
            self.failed_tests += 1;
        }
        self.case_results.push(result);
    }

    pub fn all_passed(&self) -> bool {
        self.failed_tests == 0
    }
}

/// Result of a single test case
#[derive(Debug, Clone)]
pub struct TestCaseResult {
    pub name: String,
    pub passed: bool,
    pub assertions: Vec<AssertionResult>,
    pub findings: Vec<hodei_ir::Fact>,
}

/// Result of a single assertion
#[derive(Debug, Clone)]
pub struct AssertionResult {
    pub expected: ExpectedFinding,
    pub actual: Option<ActualFinding>,
    pub passed: bool,
}

/// Actual finding from running a rule
#[derive(Debug, Clone, PartialEq)]
pub struct ActualFinding {
    pub finding_type: String,
    pub severity: String,
    pub message: String,
}

/// A snapshot of test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSnapshot {
    pub test_name: String,
    pub findings: Vec<ExpectedFinding>,
    pub metadata: HashMap<String, String>,
}
