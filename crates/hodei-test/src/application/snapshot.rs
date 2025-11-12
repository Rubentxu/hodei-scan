//! Snapshot testing
//!
//! Manages test snapshots for regression testing

use crate::domain::models::{TestResults, TestSnapshot};
use crate::domain::ports::SnapshotRepository;
use std::path::{Path, PathBuf};
use anyhow::Result;
use tokio::fs;
use serde_json;

/// Snapshot manager
pub struct SnapshotManager<R> {
    repository: R,
    snapshot_dir: PathBuf,
}

impl<R> SnapshotManager<R>
where
    R: SnapshotRepository,
{
    pub fn new(repository: R, snapshot_dir: PathBuf) -> Self {
        Self {
            repository,
            snapshot_dir,
        }
    }
    
    /// Update snapshots from test results
    pub async fn update_snapshots(&self, results: &TestResults) -> Result<()> {
        for case_result in &results.case_results {
            let snapshot = self.create_snapshot(case_result)?;
            
            self.repository.save(&snapshot).await?;
        }
        
        Ok(())
    }
    
    /// Verify snapshots against current results
    pub async fn verify_snapshots(&self, results: &TestResults) -> Result<Vec<SnapshotDiff>> {
        let mut diffs = Vec::new();
        
        for case_result in &results.case_results {
            if let Some(existing) = self.repository.load(&case_result.name).await? {
                let diff = self.compare_snapshots(&existing, case_result)?;
                if !diff.changes.is_empty() {
                    diffs.push(diff);
                }
            } else {
                // No snapshot exists - create one
                let snapshot = self.create_snapshot(case_result)?;
                self.repository.save(&snapshot).await?;
            }
        }
        
        Ok(diffs)
    }
    
    /// Create a snapshot from test case result
    fn create_snapshot(&self, case_result: &crate::domain::models::TestCaseResult) -> Result<TestSnapshot> {
        Ok(TestSnapshot {
            test_name: case_result.name.clone(),
            findings: case_result.assertions
                .iter()
                .map(|a| a.expected.clone())
                .collect(),
            metadata: std::collections::HashMap::new(),
        })
    }
    
    /// Compare snapshots
    fn compare_snapshots(
        &self,
        expected: &TestSnapshot,
        actual: &crate::domain::models::TestCaseResult,
    ) -> Result<SnapshotDiff> {
        let mut diff = SnapshotDiff {
            test_name: actual.name.clone(),
            changes: Vec::new(),
        };
        
        // Check if findings match
        if expected.findings.len() != actual.assertions.len() {
            diff.changes.push(format!(
                "Expected {} findings, got {}",
                expected.findings.len(),
                actual.assertions.len()
            ));
        }
        
        for (i, (expected_finding, assertion)) in expected.findings.iter()
            .zip(actual.assertions.iter())
            .enumerate()
        {
            if !assertion.passed {
                diff.changes.push(format!(
                    "Assertion {} failed: expected {:?} but got {:?}",
                    i, expected_finding, assertion.actual
                ));
            }
        }
        
        Ok(diff)
    }
}

/// Represents a difference between snapshots
pub struct SnapshotDiff {
    pub test_name: String,
    pub changes: Vec<String>,
}
