//! Mock implementations for testing

use hodei_test::domain::models::{TestCase, TestCaseResult, ExpectedFinding};
use hodei_test::domain::ports::{TestCaseRunner, ResultComparator};
use hodei_ir::FindingSet;

/// Mock test case runner
pub struct MockTestCaseRunner {
    pub findings_by_case: std::collections::HashMap<String, hodei_ir::FindingSet>,
}

impl MockTestCaseRunner {
    pub fn new() -> Self {
        Self {
            findings_by_case: std::collections::HashMap::new(),
        }
    }
    
    pub fn with_findings(mut self, case_name: &str, findings: hodei_ir::FindingSet) -> Self {
        self.findings_by_case.insert(case_name.to_string(), findings);
        self
    }
}

#[async_trait::async_trait]
impl TestCaseRunner for MockTestCaseRunner {
    async fn run_case(&self, test_case: &TestCase, _rule_path: &str) -> Result<TestCaseResult, anyhow::Error> {
        // Return mock findings if configured
        let findings = self.findings_by_case
            .get(&test_case.name)
            .cloned()
            .unwrap_or_else(|| FindingSet { findings: Vec::new() });
        
        Ok(TestCaseResult {
            name: test_case.name.clone(),
            passed: test_case.expected_findings.is_empty() == findings.findings.is_empty(),
            assertions: Vec::new(), // Simplified for mock
        })
    }
}

/// Mock result comparator
pub struct MockResultComparator {
    pub pass_all: bool,
}

impl MockResultComparator {
    pub fn new(pass_all: bool) -> Self {
        Self { pass_all }
    }
}

#[async_trait::async_trait]
impl ResultComparator for MockResultComparator {
    async fn compare(
        &self,
        _actual: &[hodei_ir::Finding],
        _expected: &[ExpectedFinding],
    ) -> Vec<hodei_test::domain::models::AssertionResult> {
        if self.pass_all {
            Vec::new() // All assertions pass
        } else {
            vec![hodei_test::domain::models::AssertionResult {
                expected: ExpectedFinding {
                    finding_type: "Vulnerability".to_string(),
                    severity: "Major".to_string(),
                    message: "Test".to_string(),
                },
                actual: None,
                passed: false,
            }]
        }
    }
}
