//! Change Detection Module
//!
//! This module provides abstractions and implementations for detecting file changes
//! across different source control systems and scenarios.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Types of file changes that can be detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// File was added to the repository
    Added,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed (movement)
    Renamed,
    /// File was copied from another file
    Copied,
}

/// Represents a single changed file with metadata
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangedFile {
    /// Absolute path to the changed file
    pub path: PathBuf,
    /// Type of change that occurred
    pub change_type: ChangeType,
    /// Timestamp when the change was detected
    pub timestamp: u64,
    /// Optional old path (for renames/copies)
    pub old_path: Option<PathBuf>,
    /// Optional change identifier (commit hash, etc.)
    pub change_id: Option<String>,
}

impl ChangedFile {
    /// Create a new ChangedFile with the given path and change type
    pub fn new(path: PathBuf, change_type: ChangeType) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            path,
            change_type,
            timestamp,
            old_path: None,
            change_id: None,
        }
    }

    /// Create a ChangedFile with an old path (for renames/copies)
    pub fn with_old_path(path: PathBuf, old_path: PathBuf, change_type: ChangeType) -> Self {
        let mut file = Self::new(path, change_type);
        file.old_path = Some(old_path);
        file
    }

    /// Create a ChangedFile with a change ID
    pub fn with_change_id(path: PathBuf, change_type: ChangeType, change_id: String) -> Self {
        let mut file = Self::new(path, change_type);
        file.change_id = Some(change_id);
        file
    }
}

/// Result type for change detection operations
pub type ChangeResult<T> = Result<T, ChangeError>;

/// Error type for change detection operations
#[derive(Debug, thiserror::Error)]
pub enum ChangeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git error: {message}")]
    Git { message: String },

    #[error("Hash computation error: {message}")]
    Hash { message: String },

    #[error("SCM not found: {scm}")]
    ScmNotFound { scm: String },

    #[error("Invalid repository: {path}")]
    InvalidRepository { path: PathBuf },

    #[error("Custom error: {message}")]
    Custom { message: String },
}

impl ChangeError {
    /// Create a new custom error with a message
    pub fn msg(message: impl Into<String>) -> Self {
        Self::Custom {
            message: message.into(),
        }
    }

    /// Create a new Git error
    pub fn git(message: impl Into<String>) -> Self {
        Self::Git {
            message: message.into(),
        }
    }

    /// Create a new Hash error
    pub fn hash(message: impl Into<String>) -> Self {
        Self::Hash {
            message: message.into(),
        }
    }
}

impl From<git2::Error> for ChangeError {
    fn from(error: git2::Error) -> Self {
        Self::git(format!("Git error: {}", error))
    }
}

/// Trait for detecting file changes in various scenarios
pub trait ChangeDetector: Send + Sync {
    /// Detect all changed files since the last scan or a specific reference
    ///
    /// # Arguments
    ///
    /// * `reference` - Optional reference point (commit hash, tag, etc.)
    ///                If None, uses the last scan state
    ///
    /// # Returns
    ///
    /// Returns a vector of ChangedFile structures representing all detected changes
    fn detect_changed_files(&self, reference: Option<&str>) -> ChangeResult<Vec<ChangedFile>>;

    /// Get the name of the SCM system being used (e.g., "git", "hash")
    fn scm_name(&self) -> &'static str;

    /// Check if this detector is applicable to the given path
    ///
    /// # Arguments
    ///
    /// * `project_root` - Path to the project root to check
    ///
    /// # Returns
    ///
    /// Returns true if this detector can detect changes for the given project
    fn is_applicable(&self, project_root: &Path) -> bool;
}

/// Factory for creating ChangeDetector instances with auto-detection
pub struct ChangeDetectorFactory;

impl ChangeDetectorFactory {
    /// Create a ChangeDetector for the given project root
    ///
    /// This method will auto-detect the appropriate detector based on:
    /// 1. Check for .git directory → use GitDetector
    /// 2. Check for other SCM markers
    /// 3. Fall back to HashBasedDetector
    ///
    /// # Arguments
    ///
    /// * `project_root` - Path to the project root
    ///
    /// # Returns
    ///
    /// Returns a boxed ChangeDetector instance
    pub fn create(project_root: &Path) -> ChangeResult<Box<dyn ChangeDetector>> {
        // Try Git detector first
        if GitDetector::is_applicable_internal(project_root) {
            return Ok(Box::new(GitDetector::new(project_root)));
        }

        // Fall back to hash-based detection
        Ok(Box::new(HashBasedDetector::new(project_root)))
    }

    /// Create a specific detector type (for testing)
    pub fn create_git(project_root: &Path) -> ChangeResult<GitDetector> {
        Ok(GitDetector::new(project_root))
    }

    /// Create a hash-based detector
    pub fn create_hash_based(project_root: &Path) -> HashBasedDetector {
        HashBasedDetector::new(project_root)
    }
}

/// Git-based change detector implementation
#[derive(Debug)]
pub struct GitDetector {
    project_root: PathBuf,
}

impl GitDetector {
    /// Create a new GitDetector for the given project root
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
        }
    }

    /// Internal method to check if Git is applicable
    fn is_applicable_internal(project_root: &Path) -> bool {
        project_root.join(".git").exists()
    }
}

impl ChangeDetector for GitDetector {
    fn detect_changed_files(&self, reference: Option<&str>) -> ChangeResult<Vec<ChangedFile>> {
        use git2::{Delta, DiffFindOptions, Repository};

        // Open the Git repository
        let repo = Repository::open(&self.project_root)
            .map_err(|e| ChangeError::git(format!("Failed to open repository: {}", e)))?;

        // Get the current HEAD commit
        let head = repo
            .head()
            .map_err(|e| ChangeError::git(format!("Failed to get HEAD: {}", e)))?;
        let head_commit = head
            .peel_to_commit()
            .map_err(|e| ChangeError::git(format!("Failed to peel to commit: {}", e)))?;

        // Get the commit to compare against
        let compare_commit = if let Some(ref_name) = reference {
            // Try to resolve as reference, branch, or commit OID
            let result = repo
                .resolve_reference_from_short_name(ref_name)
                .or_else(|_| {
                    repo.find_branch(ref_name, git2::BranchType::Local)
                        .map(|branch| branch.into_reference())
                })
                .and_then(|reference| reference.peel_to_commit());

            match result {
                Ok(commit) => commit,
                Err(_) => {
                    // Try as a commit OID
                    let oid = git2::Oid::from_str(ref_name)
                        .map_err(|e| ChangeError::git(format!("Invalid commit OID: {}", e)))?;
                    let commit = repo
                        .find_commit(oid)
                        .map_err(|e| ChangeError::git(format!("Commit not found: {}", e)))?;
                    commit
                }
            }
        } else {
            // Use the parent of HEAD for incremental analysis
            let parent = head_commit
                .parent(0)
                .map_err(|e| ChangeError::git(format!("Failed to get parent commit: {}", e)))?;
            parent
        };

        // Create a diff between the commits
        let mut diff = repo
            .diff_tree_to_tree(
                Some(&compare_commit.tree()?),
                Some(&head_commit.tree()?),
                None,
            )
            .map_err(|e| ChangeError::git(format!("Failed to create diff: {}", e)))?;

        // Find renames and copies
        let mut find_options = DiffFindOptions::new();
        find_options.renames(true).copies(true);
        diff.find_similar(Some(&mut find_options))
            .map_err(|e| ChangeError::git(format!("Failed to find similar files: {}", e)))?;

        // Collect all deltas (changes)
        let mut changes = Vec::new();

        diff.foreach(
            &mut |delta, _progress| {
                match delta.status() {
                    Delta::Added => {
                        if let Some(new_file) = delta.new_file().path() {
                            let mut changed_file =
                                ChangedFile::new(new_file.to_path_buf(), ChangeType::Added);
                            if let Some(old_file) = delta.old_file().path() {
                                // This was a copy
                                changed_file = ChangedFile::with_old_path(
                                    new_file.to_path_buf(),
                                    old_file.to_path_buf(),
                                    ChangeType::Copied,
                                );
                            }
                            changed_file.change_id = Some(head_commit.id().to_string());
                            changes.push(changed_file);
                        }
                    }
                    git2::Delta::Deleted => {
                        if let Some(old_file) = delta.old_file().path() {
                            changes.push(ChangedFile::with_change_id(
                                old_file.to_path_buf(),
                                ChangeType::Deleted,
                                head_commit.id().to_string(),
                            ));
                        }
                    }
                    git2::Delta::Modified => {
                        if let Some(file_path) = delta.new_file().path() {
                            changes.push(ChangedFile::with_change_id(
                                file_path.to_path_buf(),
                                ChangeType::Modified,
                                head_commit.id().to_string(),
                            ));
                        }
                    }
                    git2::Delta::Renamed => {
                        if let Some(new_file) = delta.new_file().path() {
                            if let Some(old_file) = delta.old_file().path() {
                                changes.push(ChangedFile::with_change_id(
                                    new_file.to_path_buf(),
                                    ChangeType::Renamed,
                                    head_commit.id().to_string(),
                                ));
                            }
                        }
                    }
                    git2::Delta::Copied => {
                        if let Some(new_file) = delta.new_file().path() {
                            if let Some(old_file) = delta.old_file().path() {
                                changes.push(ChangedFile::with_change_id(
                                    new_file.to_path_buf(),
                                    ChangeType::Copied,
                                    head_commit.id().to_string(),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
                true
            },
            None,
            None,
            None,
        )
        .map_err(|e| ChangeError::git(format!("Failed to iterate diff: {}", e)))?;

        Ok(changes)
    }

    fn scm_name(&self) -> &'static str {
        "git"
    }

    fn is_applicable(&self, project_root: &Path) -> bool {
        Self::is_applicable_internal(project_root)
    }
}

/// Hash-based change detector (fallback implementation)
#[derive(Debug)]
pub struct HashBasedDetector {
    project_root: PathBuf,
    last_scan_state: Option<std::collections::HashMap<PathBuf, String>>,
}

impl HashBasedDetector {
    /// Create a new HashBasedDetector for the given project root
    pub fn new(project_root: &Path) -> Self {
        Self {
            project_root: project_root.to_path_buf(),
            last_scan_state: None,
        }
    }

    /// Compute SHA-256 hash of a file's contents
    pub fn compute_file_hash(path: &Path) -> ChangeResult<String> {
        use sha2::{Digest, Sha256};
        use std::io::Read;

        let file = std::fs::File::open(path).map_err(|e| ChangeError::Io(e))?;

        let mut reader = std::io::BufReader::new(file);
        let mut hasher = Sha256::new();

        let mut buffer = [0; 8192];
        loop {
            let bytes_read = reader.read(&mut buffer).map_err(|e| ChangeError::Io(e))?;

            if bytes_read == 0 {
                break;
            }

            hasher.update(&buffer[..bytes_read]);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    /// Scan all files in the project directory
    fn scan_all_files(&self) -> ChangeResult<std::collections::HashMap<PathBuf, String>> {
        use std::collections::HashMap;

        let mut file_hashes = HashMap::new();
        let entries = std::fs::read_dir(&self.project_root).map_err(|e| ChangeError::Io(e))?;

        for entry in entries {
            let entry = entry.map_err(|e| ChangeError::Io(e))?;
            let path = entry.path();

            if path.is_file() {
                let hash = Self::compute_file_hash(&path)?;
                file_hashes.insert(path, hash);
            }
        }

        Ok(file_hashes)
    }
}

impl ChangeDetector for HashBasedDetector {
    fn detect_changed_files(&self, _reference: Option<&str>) -> ChangeResult<Vec<ChangedFile>> {
        let current_state = self.scan_all_files()?;

        let mut changes = Vec::new();

        match &self.last_scan_state {
            Some(last_state) => {
                // Find modified and deleted files
                for (path, last_hash) in last_state {
                    match current_state.get(path) {
                        Some(current_hash) if current_hash != last_hash => {
                            changes.push(ChangedFile::new(path.clone(), ChangeType::Modified));
                        }
                        None => {
                            changes.push(ChangedFile::new(path.clone(), ChangeType::Deleted));
                        }
                        _ => {}
                    }
                }

                // Find added files
                for (path, _) in &current_state {
                    if !last_state.contains_key(path) {
                        changes.push(ChangedFile::new(path.clone(), ChangeType::Added));
                    }
                }
            }
            None => {
                // First scan - all files are added
                for (path, _) in &current_state {
                    changes.push(ChangedFile::new(path.clone(), ChangeType::Added));
                }
            }
        }

        Ok(changes)
    }

    fn scm_name(&self) -> &'static str {
        "hash"
    }

    fn is_applicable(&self, _project_root: &Path) -> bool {
        // Hash-based detection is always applicable
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_changed_file_creation() {
        let path = PathBuf::from("src/main.rs");
        let change_type = ChangeType::Added;
        let changed = ChangedFile::new(path.clone(), change_type);

        assert_eq!(changed.path, path);
        assert_eq!(changed.change_type, change_type);
        assert!(changed.timestamp > 0);
        assert!(changed.old_path.is_none());
        assert!(changed.change_id.is_none());
    }

    #[test]
    fn test_changed_file_with_old_path() {
        let old_path = PathBuf::from("src/old.rs");
        let new_path = PathBuf::from("src/new.rs");
        let changed =
            ChangedFile::with_old_path(new_path.clone(), old_path.clone(), ChangeType::Renamed);

        assert_eq!(changed.path, new_path);
        assert_eq!(changed.old_path, Some(old_path));
        assert_eq!(changed.change_type, ChangeType::Renamed);
    }

    #[test]
    fn test_changed_file_with_change_id() {
        let path = PathBuf::from("src/main.rs");
        let change_id = "abc123".to_string();
        let changed =
            ChangedFile::with_change_id(path.clone(), ChangeType::Modified, change_id.clone());

        assert_eq!(changed.path, path);
        assert_eq!(changed.change_id, Some(change_id));
        assert_eq!(changed.change_type, ChangeType::Modified);
    }

    #[test]
    fn test_git_detector_is_applicable() {
        let temp_dir = tempfile::tempdir().unwrap();
        let git_dir = temp_dir.path().join(".git");
        std::fs::create_dir(&git_dir).unwrap();

        let detector = GitDetector::new(temp_dir.path());
        assert!(detector.is_applicable(temp_dir.path()));

        // Should not be applicable to non-git directory
        let temp_dir2 = tempfile::tempdir().unwrap();
        let detector2 = GitDetector::new(temp_dir2.path());
        assert!(!detector2.is_applicable(temp_dir2.path()));
    }

    #[test]
    fn test_hash_based_detector_always_applicable() {
        let temp_dir = tempfile::tempdir().unwrap();
        let detector = HashBasedDetector::new(temp_dir.path());
        assert!(detector.is_applicable(temp_dir.path()));
    }

    #[test]
    fn test_scm_name() {
        let temp_dir = tempfile::tempdir().unwrap();

        let git_detector = GitDetector::new(temp_dir.path());
        assert_eq!(git_detector.scm_name(), "git");

        let hash_detector = HashBasedDetector::new(temp_dir.path());
        assert_eq!(hash_detector.scm_name(), "hash");
    }
}
