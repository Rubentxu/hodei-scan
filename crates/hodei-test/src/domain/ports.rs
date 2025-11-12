//! Domain ports
//!
//! Interfaces for external dependencies

use crate::domain::models::{TestCase, TestCaseResult, TestConfig, TestResults};
use anyhow::Result;
use std::path::Path;

/// Port: Parser for test configuration files
#[async_trait::async_trait]
pub trait TestConfigParser: Send + Sync {
    /// Parse a test file into TestConfig
    async fn parse_file(&self, path: &Path) -> Result<TestConfig>;
}

/// Port: Runner for executing test cases
#[async_trait::async_trait]
pub trait TestCaseRunner: Send + Sync {
    /// Run a single test case
    async fn run_case(&self, test_case: &TestCase, rule_path: &str) -> Result<TestCaseResult>;
}

/// Port: Comparator for comparing actual vs expected results
#[async_trait::async_trait]
pub trait ResultComparator: Send + Sync {
    /// Compare actual findings with expected ones
    async fn compare(
        &self,
        actual: &[hodei_ir::Fact],
        expected: &[crate::domain::models::ExpectedFinding],
    ) -> Vec<crate::domain::models::AssertionResult>;
}

/// Port: Repository for storing and retrieving snapshots
#[async_trait::async_trait]
pub trait SnapshotRepository: Send + Sync {
    /// Save a snapshot
    async fn save(&self, snapshot: &crate::domain::models::TestSnapshot) -> Result<()>;

    /// Load a snapshot by name
    async fn load(&self, name: &str) -> Result<Option<crate::domain::models::TestSnapshot>>;

    /// List all snapshots
    async fn list(&self) -> Result<Vec<String>>;
}
