//! Unit tests for SnapshotManager

use hodei_test::application::snapshot::{SnapshotManager, SnapshotDiff};
use hodei_test::domain::models::{TestResults, TestCaseResult};
use tempfile::TempDir;
use std::path::PathBuf;

#[tokio::test]
async fn test_snapshot_manager_creation() {
    struct MockRepository;
    
    #[async_trait::async_trait]
    impl hodei_test::domain::ports::SnapshotRepository for MockRepository {
        async fn save(&self, _snapshot: &hodei_test::domain::models::TestSnapshot) -> Result<(), anyhow::Error> {
            Ok(())
        }
        
        async fn load(&self, _name: &str) -> Result<Option<hodei_test::domain::models::TestSnapshot>, anyhow::Error> {
            Ok(None)
        }
        
        async fn list(&self) -> Result<Vec<String>, anyhow::Error> {
            Ok(Vec::new())
        }
    }
    
    let temp_dir = TempDir::new().unwrap();
    let repo = MockRepository;
    let manager = SnapshotManager::new(repo, temp_dir.path().to_path_buf());
    
    assert!(true); // If we can create it, it worked
}

#[tokio::test]
async fn test_snapshot_manager_new() {
    struct MockRepository;
    
    #[async_trait::async_trait]
    impl hodei_test::domain::ports::SnapshotRepository for MockRepository {
        async fn save(&self, _snapshot: &hodei_test::domain::models::TestSnapshot) -> Result<(), anyhow::Error> {
            Ok(())
        }
        
        async fn load(&self, _name: &str) -> Result<Option<hodei_test::domain::models::TestSnapshot>, anyhow::Error> {
            Ok(None)
        }
        
        async fn list(&self) -> Result<Vec<String>, anyhow::Error> {
            Ok(Vec::new())
        }
    }
    
    let temp_dir = TempDir::new().unwrap();
    let snapshot_dir = temp_dir.path().to_path_buf();
    let repo = MockRepository;
    
    let manager = SnapshotManager::new(repo, snapshot_dir.clone());
    
    // Manager should be created successfully
    assert!(true);
}

#[tokio::test]
async fn test_snapshot_diff_creation() {
    let diff = SnapshotDiff {
        test_name: "test_snapshot".to_string(),
        changes: vec!["change1".to_string(), "change2".to_string()],
    };
    
    assert_eq!(diff.test_name, "test_snapshot");
    assert_eq!(diff.changes.len(), 2);
    assert_eq!(diff.changes[0], "change1");
    assert_eq!(diff.changes[1], "change2");
}

#[tokio::test]
async fn test_snapshot_diff_empty() {
    let diff = SnapshotDiff {
        test_name: "empty_test".to_string(),
        changes: Vec::new(),
    };
    
    assert_eq!(diff.test_name, "empty_test");
    assert!(diff.changes.is_empty());
}

#[tokio::test]
async fn test_snapshot_diff_is_empty() {
    let diff_with_changes = SnapshotDiff {
        test_name: "test".to_string(),
        changes: vec!["change".to_string()],
    };
    assert!(!diff_with_changes.changes.is_empty());
    
    let diff_without_changes = SnapshotDiff {
        test_name: "test".to_string(),
        changes: Vec::new(),
    };
    assert!(diff_without_changes.changes.is_empty());
}

#[tokio::test]
async fn test_create_test_snapshot() {
    struct MockRepository;
    
    #[async_trait::async_trait]
    impl hodei_test::domain::ports::SnapshotRepository for MockRepository {
        async fn save(&self, _snapshot: &hodei_test::domain::models::TestSnapshot) -> Result<(), anyhow::Error> {
            Ok(())
        }
        
        async fn load(&self, _name: &str) -> Result<Option<hodei_test::domain::models::TestSnapshot>, anyhow::Error> {
            Ok(None)
        }
        
        async fn list(&self) -> Result<Vec<String>, anyhow::Error> {
            Ok(Vec::new())
        }
    }
    
    let temp_dir = TempDir::new().unwrap();
    let repo = MockRepository;
    let manager = SnapshotManager::new(repo, temp_dir.path().to_path_buf());
    
    let mut results = TestResults::new();
    let case_result = TestCaseResult {
        name: "test_case".to_string(),
        passed: true,
        assertions: vec![hodei_test::domain::models::AssertionResult {
            expected: hodei_test::domain::models::ExpectedFinding {
                finding_type: "Vulnerability".to_string(),
                severity: "Major".to_string(),
                message: "Test finding".to_string(),
            },
            actual: None,
            passed: true,
        }],
    };
    results.add_result(case_result);
    
    // update_snapshots should succeed
    manager.update_snapshots(&results).await.unwrap();
    
    assert!(true);
}

#[tokio::test]
async fn test_verify_snapshots_new() {
    struct MockRepository;
    
    #[async_trait::async_trait]
    impl hodei_test::domain::ports::SnapshotRepository for MockRepository {
        async fn save(&self, _snapshot: &hodei_test::domain::models::TestSnapshot) -> Result<(), anyhow::Error> {
            Ok(())
        }
        
        async fn load(&self, _name: &str) -> Result<Option<hodei_test::domain::models::TestSnapshot>, anyhow::Error> {
            // No snapshot exists
            Ok(None)
        }
        
        async fn list(&self) -> Result<Vec<String>, anyhow::Error> {
            Ok(Vec::new())
        }
    }
    
    let temp_dir = TempDir::new().unwrap();
    let repo = MockRepository;
    let manager = SnapshotManager::new(repo, temp_dir.path().to_path_buf());
    
    let mut results = TestResults::new();
    let case_result = TestCaseResult {
        name: "new_test".to_string(),
        passed: true,
        assertions: Vec::new(),
    };
    results.add_result(case_result);
    
    // Should create snapshot for new test
    let diffs = manager.verify_snapshots(&results).await.unwrap();
    
    // No diffs (snapshot will be created)
    assert!(diffs.is_empty());
}

#[tokio::test]
async fn test_verify_snapshots_matching() {
    struct MockRepository;
    
    #[async_trait::async_trait]
    impl hodei_test::domain::ports::SnapshotRepository for MockRepository {
        async fn save(&self, _snapshot: &hodei_test::domain::models::TestSnapshot) -> Result<(), anyhow::Error> {
            Ok(())
        }
        
        async fn load(&self, name: &str) -> Result<Option<hodei_test::domain::models::TestSnapshot>, anyhow::Error> {
            // Return matching snapshot
            if name == "matching_test" {
                Ok(Some(hodei_test::domain::models::TestSnapshot {
                    test_name: name.to_string(),
                    findings: vec![hodei_test::domain::models::ExpectedFinding {
                        finding_type: "Vulnerability".to_string(),
                        severity: "Major".to_string(),
                        message: "Test".to_string(),
                    }],
                    metadata: std::collections::HashMap::new(),
                }))
            } else {
                Ok(None)
            }
        }
        
        async fn list(&self) -> Result<Vec<String>, anyhow::Error> {
            Ok(Vec::new())
        }
    }
    
    let temp_dir = TempDir::new().unwrap();
    let repo = MockRepository;
    let manager = SnapshotManager::new(repo, temp_dir.path().to_path_buf());
    
    let mut results = TestResults::new();
    let case_result = TestCaseResult {
        name: "matching_test".to_string(),
        passed: true,
        assertions: vec![hodei_test::domain::models::AssertionResult {
            expected: hodei_test::domain::models::ExpectedFinding {
                finding_type: "Vulnerability".to_string(),
                severity: "Major".to_string(),
                message: "Test".to_string(),
            },
            actual: None,
            passed: true,
        }],
    };
    results.add_result(case_result);
    
    // Should match (no diffs)
    let diffs = manager.verify_snapshots(&results).await.unwrap();
    assert!(diffs.is_empty());
}

#[tokio::test]
async fn test_verify_snapshots_different() {
    struct MockRepository;
    
    #[async_trait::async_trait]
    impl hodei_test::domain::ports::SnapshotRepository for MockRepository {
        async fn save(&self, _snapshot: &hodei_test::domain::models::TestSnapshot) -> Result<(), anyhow::Error> {
            Ok(())
        }
        
        async fn load(&self, _name: &str) -> Result<Option<hodei_test::domain::models::TestSnapshot>, anyhow::Error> {
            // Return snapshot with different findings
            Ok(Some(hodei_test::domain::models::TestSnapshot {
                test_name: "test".to_string(),
                findings: vec![hodei_test::domain::models::ExpectedFinding {
                    finding_type: "Vulnerability".to_string(),
                    severity: "Critical".to_string(),
                    message: "Different".to_string(),
                }],
                metadata: std::collections::HashMap::new(),
            }))
        }
        
        async fn list(&self) -> Result<Vec<String>, anyhow::Error> {
            Ok(Vec::new())
        }
    }
    
    let temp_dir = TempDir::new().unwrap();
    let repo = MockRepository;
    let manager = SnapshotManager::new(repo, temp_dir.path().to_path_buf());
    
    let mut results = TestResults::new();
    let case_result = TestCaseResult {
        name: "test".to_string(),
        passed: false,
        assertions: vec![hodei_test::domain::models::AssertionResult {
            expected: hodei_test::domain::models::ExpectedFinding {
                finding_type: "Vulnerability".to_string(),
                severity: "Major".to_string(),
                message: "Test".to_string(),
            },
            actual: None,
            passed: false,
        }],
    };
    results.add_result(case_result);
    
    // Should have diffs
    let diffs = manager.verify_snapshots(&results).await.unwrap();
    assert!(!diffs.is_empty());
    assert_eq!(diffs[0].test_name, "test");
}
