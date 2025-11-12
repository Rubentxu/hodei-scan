//! File system snapshot repository
//!
//! Stores snapshots on the local file system

use crate::domain::models::TestSnapshot;
use crate::domain::ports::SnapshotRepository;
use std::path::{Path, PathBuf};
use anyhow::Result;
use tokio::fs;

/// File system based snapshot repository
pub struct FileSystemSnapshotRepository {
    snapshot_dir: PathBuf,
}

impl FileSystemSnapshotRepository {
    pub fn new(snapshot_dir: PathBuf) -> Self {
        Self { snapshot_dir }
    }
    
    fn snapshot_path(&self, name: &str) -> PathBuf {
        self.snapshot_dir.join(format!("{}.snap", name))
    }
}

#[async_trait::async_trait]
impl SnapshotRepository for FileSystemSnapshotRepository {
    async fn save(&self, snapshot: &TestSnapshot) -> Result<()> {
        let path = self.snapshot_path(&snapshot.test_name);
        let content = serde_json::to_string_pretty(snapshot)?;
        fs::write(&path, content).await?;
        Ok(())
    }
    
    async fn load(&self, name: &str) -> Result<Option<TestSnapshot>> {
        let path = self.snapshot_path(name);
        if !path.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&path).await?;
        let snapshot: TestSnapshot = serde_json::from_str(&content)?;
        Ok(Some(snapshot))
    }
    
    async fn list(&self) -> Result<Vec<String>> {
        let mut entries = fs::read_dir(&self.snapshot_dir).await?;
        let mut snapshots = Vec::new();
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "snap") {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    snapshots.push(name.to_string());
                }
            }
        }
        
        Ok(snapshots)
    }
}
