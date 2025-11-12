//! Test runner application service
//!
//! Orchestrates test execution

use crate::domain::models::{
    AssertionResult, ExpectedFinding, TestCase, TestCaseResult, TestResults,
};
use crate::domain::ports::{ResultComparator, TestCaseRunner, TestConfigParser};
use anyhow::Result;
use std::path::Path;
use tokio::fs;

/// Test runner implementation
pub struct HodeiTestRunner<P, R, C> {
    parser: P,
    runner: R,
    comparator: C,
}

impl<P, R, C> HodeiTestRunner<P, R, C>
where
    P: TestConfigParser,
    R: TestCaseRunner,
    C: ResultComparator,
{
    pub fn new(parser: P, runner: R, comparator: C) -> Self {
        Self {
            parser,
            runner,
            comparator,
        }
    }

    /// Run all tests in a test file
    pub async fn run_test_file(&self, test_file: &Path, rule_path: &str) -> Result<TestResults> {
        let config = self.parser.parse_file(test_file).await?;

        let mut results = TestResults::new();

        for test_case in &config.cases {
            let result = self.run_single_test(test_case, rule_path).await?;
            results.add_result(result);
        }

        Ok(results)
    }

    /// Run a single test case
    async fn run_single_test(
        &self,
        test_case: &TestCase,
        rule_path: &str,
    ) -> Result<TestCaseResult> {
        let actual_findings = self.runner.run_case(test_case, rule_path).await?;

        let assertions = self
            .comparator
            .compare(&actual_findings.findings, &test_case.expected_findings)
            .await;

        let passed = assertions.iter().all(|a| a.passed);

        Ok(TestCaseResult {
            name: test_case.name.clone(),
            passed,
            assertions,
            findings: actual_findings.findings,
        })
    }

    /// Run all test files in a directory
    pub async fn run_all_tests(&self, test_dir: &Path, rule_path: &str) -> Result<TestResults> {
        let mut all_results = TestResults::new();

        let mut entries = fs::read_dir(test_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| {
                ext == "hodei.test" || ext == "yaml" || ext == "yml"
            }) {
                let results = self.run_test_file(&path, rule_path).await?;
                all_results.total_tests += results.total_tests;
                all_results.passed_tests += results.passed_tests;
                all_results.failed_tests += results.failed_tests;
                all_results.case_results.extend(results.case_results);
            }
        }

        Ok(all_results)
    }
}
