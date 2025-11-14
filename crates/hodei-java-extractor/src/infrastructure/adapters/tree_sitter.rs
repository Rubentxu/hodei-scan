//! tree-sitter Adapter (Infrastructure Layer)
//!
//! This adapter provides pattern matching capabilities using tree-sitter.
//! It implements Level 2 (pattern-based) analysis for Java code.
//!
//! **Note**: Currently using stub implementation due to tree-sitter-java version compatibility issues.
//! Real integration will be implemented once dependency versions are resolved.

use crate::domain::{
    entities::{DomainError, JavaClass},
    repositories::JavaSourceRepository,
};
use hodei_ir::types::project_path::ProjectPath;
use std::path::{Path, PathBuf};

/// tree-sitter Adapter Implementation
pub struct TreeSitterAdapter {
    source_paths: Vec<PathBuf>,
    cache: Vec<JavaClass>,
}

impl TreeSitterAdapter {
    pub fn new(source_paths: Vec<PathBuf>) -> Self {
        Self {
            source_paths,
            cache: vec![],
        }
    }

    /// Parse Java files and extract classes (stub implementation)
    pub fn parse_java_files(&mut self) -> Result<Vec<JavaClass>, DomainError> {
        // TODO: Implement real tree-sitter-java parsing
        // For now, return empty cache - using stub to allow compilation
        self.cache = Vec::new();
        Ok(Vec::new())
    }
}

impl JavaSourceRepository for TreeSitterAdapter {
    fn find_by_package(&self, package: &str) -> Result<Vec<JavaClass>, DomainError> {
        let filtered: Vec<JavaClass> = self
            .cache
            .iter()
            .filter(|cls| cls.id.package == package)
            .cloned()
            .collect();
        Ok(filtered)
    }

    fn get_coverage_data(
        &self,
        _source_id: &crate::domain::entities::JavaSourceId,
    ) -> Result<Option<crate::domain::entities::CoverageData>, DomainError> {
        // tree-sitter adapter doesn't provide coverage data
        Ok(None)
    }

    fn save_analysis_result(
        &self,
        _result: &crate::domain::entities::JavaAnalysisResult,
    ) -> Result<(), DomainError> {
        // tree-sitter adapter doesn't save analysis results
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_sitter_adapter_creation() {
        let adapter = TreeSitterAdapter::new(vec![PathBuf::from("src/main/java")]);
        assert_eq!(adapter.source_paths.len(), 1);
    }

    #[test]
    fn test_find_package_empty() {
        let adapter = TreeSitterAdapter::new(vec![PathBuf::from("src/main/java")]);
        let result = adapter.find_by_package("com.example");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_parse_java_files_stub() {
        let mut adapter = TreeSitterAdapter::new(vec![PathBuf::from("src/main/java")]);
        let result = adapter.parse_java_files();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
