//! Integration tests for file system snapshot repository

use hodei_test::infrastructure::file_system_snapshot_repo::FileSystemSnapshotRepository;
use hodei_test::domain::models::TestSnapshot;
use tempfile::TempDir;
use std::collections::HashMap;

#[tokio::test]
async fn test_save_and_load_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let snapshot_dir = temp_dir.path().join("snapshots");
    tokio::fs::create_dir_all(&snapshot_dir).await.unwrap();
    
    let repo = FileSystemSnapshotRepository::new(snapshot_dir);
    
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "1.0".to_string());
    
    let snapshot = TestSnapshot {
        test_name: "test_snapshot".to_string(),
        findings: vec![hodei_test::domain::models::ExpectedFinding {
            finding_type: "Vulnerability".to_string(),
            severity: "Critical".to_string(),
            message: "Test finding".to_string(),
        }],
        metadata,
    };
    
    // Save snapshot
    repo.save(&snapshot).await.unwrap();
    
    // Load snapshot
    let loaded = repo.load("test_snapshot").await.unwrap().unwrap();
    
    assert_eq!(loaded.test_name, "test_snapshot");
    assert_eq!(loaded.findings.len(), 1);
    assert_eq!(loaded.findings[0].finding_type, "Vulnerability");
}

#[tokio::test]
async fn test_load_nonexistent_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let snapshot_dir = temp_dir.path().join("snapshots");
    tokio::fs::create_dir_all(&snapshot_dir).await.unwrap();
    
    let repo = FileSystemSnapshotRepository::new(snapshot_dir);
    
    let loaded = repo.load("nonexistent").await.unwrap();
    
    assert!(loaded.is_none());
}

#[tokio::test]
async fn test_list_snapshots() {
    let temp_dir = TempDir::new().unwrap();
    let snapshot_dir = temp_dir.path().join("snapshots");
    tokio::fs::create_dir_all(&snapshot_dir).await.unwrap();
    
    let repo = FileSystemSnapshotRepository::new(snapshot_dir);
    
    // Save multiple snapshots
    for i in 0..5 {
        let snapshot = TestSnapshot {
            test_name: format!("test_{}", i),
            findings: Vec::new(),
            metadata: HashMap::new(),
        };
        repo.save(&snapshot).await.unwrap();
    }
    
    let snapshots = repo.list().await.unwrap();
    
    assert_eq!(snapshots.len(), 5);
    assert!(snapshots.contains(&"test_0".to_string()));
    assert!(snapshots.contains(&"test_4".to_string()));
}

#[tokio::test]
async fn test_update_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let snapshot_dir = temp_dir.path().join("snapshots");
    tokio::fs::create_dir_all(&snapshot_dir).await.unwrap();
    
    let repo = FileSystemSnapshotRepository::new(snapshot_dir);
    
    // Save initial snapshot
    let initial_snapshot = TestSnapshot {
        test_name: "updatable".to_string(),
        findings: vec![hodei_test::domain::models::ExpectedFinding {
            finding_type: "Vulnerability".to_string(),
            severity: "Major".to_string(),
            message: "Initial".to_string(),
        }],
        metadata: HashMap::new(),
    };
    repo.save(&initial_snapshot).await.unwrap();
    
    // Update snapshot
    let updated_snapshot = TestSnapshot {
        test_name: "updatable".to_string(),
        findings: vec![hodei_test::domain::models::ExpectedFinding {
            finding_type: "CodeSmell".to_string(),
            severity: "Minor".to_string(),
            message: "Updated".to_string(),
        }],
        metadata: HashMap::new(),
    };
    repo.save(&updated_snapshot).await.unwrap();
    
    // Verify update
    let loaded = repo.load("updatable").await.unwrap().unwrap();
    assert_eq!(loaded.findings.len(), 1);
    assert_eq!(loaded.findings[0].finding_type, "CodeSmell");
    assert_eq!(loaded.findings[0].message, "Updated");
}

#[tokio::test]
async fn test_save_multiple_snapshots() {
    let temp_dir = TempDir::new().unwrap();
    let snapshot_dir = temp_dir.path().join("snapshots");
    tokio::fs::create_dir_all(&snapshot_dir).await.unwrap();
    
    let repo = FileSystemSnapshotRepository::new(snapshot_dir);
    
    // Save snapshots with different names
    let snapshots = vec![
        "snapshot_a",
        "snapshot_b",
        "snapshot_c",
    ];
    
    for name in &snapshots {
        let snapshot = TestSnapshot {
            test_name: name.to_string(),
            findings: Vec::new(),
            metadata: HashMap::new(),
        };
        repo.save(&snapshot).await.unwrap();
    }
    
    // Verify all can be loaded
    for name in snapshots {
        let loaded = repo.load(name).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().test_name, name);
    }
}

#[tokio::test]
async fn test_save_snapshot_with_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let snapshot_dir = temp_dir.path().join("snapshots");
    tokio::fs::create_dir_all(&snapshot_dir).await.unwrap();
    
    let repo = FileSystemSnapshotRepository::new(snapshot_dir);
    
    let mut metadata = HashMap::new();
    metadata.insert("version".to_string(), "1.0".to_string());
    metadata.insert("author".to_string(), "test".to_string());
    metadata.insert("timestamp".to_string(), "2025-01-01".to_string());
    
    let snapshot = TestSnapshot {
        test_name: "metadata_test".to_string(),
        findings: Vec::new(),
        metadata,
    };
    
    repo.save(&snapshot).await.unwrap();
    
    let loaded = repo.load("metadata_test").await.unwrap().unwrap();
    
    assert_eq!(loaded.metadata.get("version"), Some(&"1.0".to_string()));
    assert_eq!(loaded.metadata.get("author"), Some(&"test".to_string()));
    assert_eq!(loaded.metadata.get("timestamp"), Some(&"2025-01-01".to_string()));
}

#[tokio::test]
async fn test_save_empty_snapshot() {
    let temp_dir = TempDir::new().unwrap();
    let snapshot_dir = temp_dir.path().join("snapshots");
    tokio::fs::create_dir_all(&snapshot_dir).await.unwrap();
    
    let repo = FileSystemSnapshotRepository::new(snapshot_dir);
    
    let snapshot = TestSnapshot {
        test_name: "empty".to_string(),
        findings: Vec::new(),
        metadata: HashMap::new(),
    };
    
    repo.save(&snapshot).await.unwrap();
    
    let loaded = repo.load("empty").await.unwrap().unwrap();
    
    assert_eq!(loaded.test_name, "empty");
    assert!(loaded.findings.is_empty());
    assert!(loaded.metadata.is_empty());
}

#[tokio::test]
async fn test_save_snapshot_with_empty_name() {
    let temp_dir = TempDir::new().unwrap();
    let snapshot_dir = temp_dir.path().join("snapshots");
    tokio::fs::create_dir_all(&snapshot_dir).await.unwrap();
    
    let repo = FileSystemSnapshotRepository::new(snapshot_dir);
    
    let snapshot = TestSnapshot {
        test_name: "".to_string(),
        findings: Vec::new(),
        metadata: HashMap::new(),
    };
    
    // Should still save (even with empty name)
    repo.save(&snapshot).await.unwrap();
    
    let loaded = repo.load("").await.unwrap();
    assert!(loaded.is_some());
    assert_eq!(loaded.unwrap().test_name, "");
}

#[tokio::test]
async fn test_list_empty_directory() {
    let temp_dir = TempDir::new().unwrap();
    let snapshot_dir = temp_dir.path().join("snapshots");
    tokio::fs::create_dir_all(&snapshot_dir).await.unwrap();
    
    let repo = FileSystemSnapshotRepository::new(snapshot_dir);
    
    let snapshots = repo.list().await.unwrap();
    
    assert!(snapshots.is_empty());
}

#[tokio::test]
async fn test_snapshot_file_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let snapshot_dir = temp_dir.path().join("snapshots");
    tokio::fs::create_dir_all(&snapshot_dir).await.unwrap();
    
    let repo = FileSystemSnapshotRepository::new(snapshot_dir.clone());
    
    // Save snapshot
    let snapshot = TestSnapshot {
        test_name: "persistent".to_string(),
        findings: vec![hodei_test::domain::models::ExpectedFinding {
            finding_type: "Vulnerability".to_string(),
            severity: "Critical".to_string(),
            message: "Persistent finding".to_string(),
        }],
        metadata: HashMap::new(),
    };
    repo.save(&snapshot).await.unwrap();
    
    // Verify file exists
    let snapshot_file = snapshot_dir.join("persistent.snap");
    assert!(snapshot_file.exists());
    
    // Create new repository instance (simulates restart)
    let repo2 = FileSystemSnapshotRepository::new(snapshot_dir);
    
    // Load with new instance
    let loaded = repo2.load("persistent").await.unwrap().unwrap();
    
    assert_eq!(loaded.test_name, "persistent");
    assert_eq!(loaded.findings[0].message, "Persistent finding");
}

#[tokio::test]
async fn test_list_snapshots_with_non_snap_files() {
    let temp_dir = TempDir::new().unwrap();
    let snapshot_dir = temp_dir.path().join("snapshots");
    tokio::fs::create_dir_all(&snapshot_dir).await.unwrap();
    
    let repo = FileSystemSnapshotRepository::new(snapshot_dir);
    
    // Save legitimate snapshot
    let snapshot = TestSnapshot {
        test_name: "legitimate".to_string(),
        findings: Vec::new(),
        metadata: HashMap::new(),
    };
    repo.save(&snapshot).await.unwrap();
    
    // Create non-snap files
    let other_file = snapshot_dir.join("other.txt");
    let config_file = snapshot_dir.join("config.json");
    tokio::fs::write(&other_file, "other").await.unwrap();
    tokio::fs::write(&config_file, "{}").await.unwrap();
    
    let snapshots = repo.list().await.unwrap();
    
    // Should only list .snap files
    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0], "legitimate");
}
