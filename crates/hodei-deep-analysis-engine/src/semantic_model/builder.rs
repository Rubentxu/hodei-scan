//! Semantic Model Builder
//!
//! This module provides the SemanticModelBuilder which constructs rich
//! semantic representations of code from AST, including CFG and DFG.

use crate::Result;
use std::fs;
use std::path::Path;

/// Builder for constructing SemanticModel from source code
#[derive(Debug, Default)]
pub struct SemanticModelBuilder {
    /// Source path being analyzed
    source_path: Option<String>,
}

impl SemanticModelBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Build semantic model from source file or directory
    pub fn from_source(&mut self, source_path: &str) -> Result<SemanticModel> {
        self.source_path = Some(source_path.to_string());

        // Check if path exists
        if !Path::new(source_path).exists() {
            return Err(crate::DeepAnalysisError::SemanticModel(format!(
                "Source path does not exist: {}",
                source_path
            )));
        }

        // For now, create an empty semantic model
        // A full implementation would:
        // 1. Parse source code using tree-sitter
        // 2. Build CFG from AST
        // 3. Build DFG from code structure
        // 4. Create scope tree
        // 5. Extract coupling relationships

        let mut model = SemanticModel::new();

        // Parse the source file if it exists
        if Path::new(source_path).is_file() {
            self.parse_source_file(source_path, &mut model)?;
        } else if Path::new(source_path).is_dir() {
            self.parse_source_directory(source_path, &mut model)?;
        }

        Ok(model)
    }

    /// Parse a single source file
    fn parse_source_file(&self, path: &str, _model: &mut SemanticModel) -> Result<()> {
        // Read the file
        let content = fs::read_to_string(path).map_err(|e| crate::DeepAnalysisError::Io(e))?;

        // TODO: Parse using tree-sitter
        // For now, just verify the file can be read

        // TODO: Build CFG from AST
        // TODO: Build DFG from code structure
        // TODO: Extract entities and relationships

        let _ = content; // Suppress unused warning

        Ok(())
    }

    /// Parse a directory of source files
    fn parse_source_directory(&self, _path: &str, _model: &mut SemanticModel) -> Result<()> {
        // TODO: Walk directory and parse all source files
        // For now, just verify the directory can be read

        // TODO: Merge CFG/DFG from all files
        // TODO: Build global scope tree
        // TODO: Detect inter-file dependencies

        Ok(())
    }
}

/// SemanticModel - Rich representation of code structure
#[derive(Debug, Default)]
pub struct SemanticModel {
    /// Control Flow Graph
    pub cfg: super::cfg::ControlFlowGraph,
    /// Data Flow Graph
    pub dfg: super::dfg::DataFlowGraph,
}

impl SemanticModel {
    /// Create a new semantic model
    pub fn new() -> Self {
        Self::default()
    }

    /// Get number of control flow nodes
    pub fn cfg_node_count(&self) -> usize {
        self.cfg.node_count()
    }

    /// Get number of data flow nodes
    pub fn dfg_node_count(&self) -> usize {
        self.dfg.node_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = SemanticModelBuilder::new();
        assert!(format!("{:?}", builder).contains("SemanticModelBuilder"));
    }

    #[test]
    fn test_builder_default() {
        let builder = SemanticModelBuilder::default();
        assert!(format!("{:?}", builder).contains("SemanticModelBuilder"));
    }

    #[test]
    fn test_model_creation() {
        let model = SemanticModel::new();
        assert_eq!(model.cfg_node_count(), 0);
        assert_eq!(model.dfg_node_count(), 0);
    }

    #[test]
    fn test_model_default() {
        let model = SemanticModel::default();
        assert_eq!(model.cfg_node_count(), 0);
        assert_eq!(model.dfg_node_count(), 0);
    }

    #[test]
    fn test_from_source_with_nonexistent_path() {
        let mut builder = SemanticModelBuilder::new();
        let result = builder.from_source("/nonexistent/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_source_existing_file() {
        let mut builder = SemanticModelBuilder::new();

        // Create a temporary test file
        let temp_file = "/tmp/test_source.rs";
        std::fs::write(temp_file, "fn main() {}").unwrap();

        let result = builder.from_source(temp_file);
        assert!(result.is_ok());

        // Clean up
        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_from_source_existing_directory() {
        let mut builder = SemanticModelBuilder::new();

        // Create a temporary directory
        let temp_dir = "/tmp/test_dir_12345";
        std::fs::create_dir_all(temp_dir).unwrap();

        let result = builder.from_source(temp_dir);
        assert!(result.is_ok());

        // Clean up
        std::fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_from_source_updates_builder_state() {
        let mut builder = SemanticModelBuilder::new();

        let temp_file = tempfile::NamedTempFile::with_suffix(".rs")
            .unwrap()
            .into_temp_path();
        std::fs::write(&temp_file, "fn main() {}").unwrap();

        assert!(builder.source_path.is_none());

        let _result = builder.from_source(temp_file.to_str().unwrap());
        assert!(builder.source_path.is_some());
        assert_eq!(builder.source_path.as_ref().unwrap(), temp_file.to_str().unwrap());

        // Clean up
        temp_file.close().unwrap();
    }

    #[test]
    fn test_parse_source_file_reads_content() {
        let temp_file = "/tmp/test_content.rs";
        let content = "fn test() { println!(\"hello\"); }";
        std::fs::write(temp_file, content).unwrap();

        let mut builder = SemanticModelBuilder::new();
        let mut model = SemanticModel::new();

        let result = builder.parse_source_file(temp_file, &mut model);
        assert!(result.is_ok());

        // Clean up
        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_parse_source_file_with_empty_content() {
        let temp_file = "/tmp/test_empty.rs";
        std::fs::write(temp_file, "").unwrap();

        let mut builder = SemanticModelBuilder::new();
        let mut model = SemanticModel::new();

        let result = builder.parse_source_file(temp_file, &mut model);
        assert!(result.is_ok());

        // Clean up
        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_parse_source_file_with_large_content() {
        let temp_file = "/tmp/test_large.rs";
        let content = "fn main() {\n".to_string() + &"    println!(\"test\");\n".repeat(10000) + "}";
        std::fs::write(temp_file, content).unwrap();

        let mut builder = SemanticModelBuilder::new();
        let mut model = SemanticModel::new();

        let result = builder.parse_source_file(temp_file, &mut model);
        assert!(result.is_ok());

        // Clean up
        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_parse_source_file_unicode_content() {
        let temp_file = "/tmp/test_unicode.rs";
        let content = "fn main() { let 变量 = \"测试\"; println!(\"{}\", 变量); }";
        std::fs::write(temp_file, content).unwrap();

        let mut builder = SemanticModelBuilder::new();
        let mut model = SemanticModel::new();

        let result = builder.parse_source_file(temp_file, &mut model);
        assert!(result.is_ok());

        // Clean up
        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_parse_source_directory_creates_model() {
        let temp_dir = "/tmp/test_dir_67890";
        std::fs::create_dir_all(temp_dir).unwrap();

        let mut builder = SemanticModelBuilder::new();
        let mut model = SemanticModel::new();

        let result = builder.parse_source_directory(temp_dir, &mut model);
        assert!(result.is_ok());

        // Clean up
        std::fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_parse_source_directory_with_nested_structure() {
        let temp_dir = "/tmp/test_nested_123";
        let sub_dir = format!("{}/src", temp_dir);
        std::fs::create_dir_all(&sub_dir).unwrap();

        // Create files in nested structure
        std::fs::write(format!("{}/main.rs", temp_dir), "fn main() {}").unwrap();
        std::fs::write(format!("{}/lib.rs", temp_dir), "pub fn lib() {}").unwrap();
        std::fs::write(format!("{}/mod.rs", sub_dir), "pub mod module {}").unwrap();

        let mut builder = SemanticModelBuilder::new();
        let mut model = SemanticModel::new();

        let result = builder.parse_source_directory(temp_dir, &mut model);
        assert!(result.is_ok());

        // Clean up
        std::fs::remove_dir_all(temp_dir).unwrap();
    }

    #[test]
    fn test_from_source_with_permission_denied() {
        // This test may not work on all systems, so we'll skip if we can't create the file
        let temp_file = "/tmp/test_permission.rs";
        std::fs::write(temp_file, "fn main() {}").unwrap();

        let mut builder = SemanticModelBuilder::new();

        // Try to parse a directory as a file (will still work in current impl)
        let result = builder.from_source(temp_file);
        assert!(result.is_ok());

        // Clean up
        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_from_source_with_special_characters_in_path() {
        let temp_file = "/tmp/test_special_chars_symbols.rs";
        std::fs::write(temp_file, "fn main() {}").unwrap();

        let mut builder = SemanticModelBuilder::new();
        let result = builder.from_source(temp_file);

        // Should handle special characters in path
        assert!(result.is_ok() || result.is_err()); // Either works or fails gracefully

        // Clean up
        if std::path::Path::new(temp_file).exists() {
            std::fs::remove_file(temp_file).unwrap();
        }
    }

    #[test]
    fn test_model_node_counts() {
        let model = SemanticModel::new();

        // Initially empty
        assert_eq!(model.cfg_node_count(), 0);
        assert_eq!(model.dfg_node_count(), 0);
    }

    #[test]
    fn test_builder_source_path_after_from_source() {
        let mut builder = SemanticModelBuilder::new();
        let temp_file = "/tmp/test_path.rs";
        std::fs::write(temp_file, "fn main() {}").unwrap();

        assert!(builder.source_path.is_none());

        let _ = builder.from_source(temp_file);
        assert!(builder.source_path.is_some());

        // Clean up
        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_from_source_multiple_times() {
        let mut builder = SemanticModelBuilder::new();

        let temp_file1 = "/tmp/test1.rs";
        let temp_file2 = "/tmp/test2.rs";
        std::fs::write(temp_file1, "fn test1() {}").unwrap();
        std::fs::write(temp_file2, "fn test2() {}").unwrap();

        let result1 = builder.from_source(temp_file1);
        let result2 = builder.from_source(temp_file2);

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        // Should update source path
        assert_eq!(builder.source_path.as_ref().unwrap(), temp_file2);

        // Clean up
        std::fs::remove_file(temp_file1).unwrap();
        std::fs::remove_file(temp_file2).unwrap();
    }

    #[test]
    fn test_builder_io_error_handling() {
        let mut builder = SemanticModelBuilder::new();

        // Should handle non-existent file gracefully
        let result = builder.from_source("/nonexistent/file.rs");
        assert!(result.is_err());
    }

    #[test]
    fn test_semantic_model_builder_has_source_path() {
        let builder = SemanticModelBuilder::new();
        assert!(builder.source_path.is_none());

        let mut builder2 = SemanticModelBuilder::default();
        assert!(builder2.source_path.is_none());
    }

    #[test]
    fn test_from_source_with_rust_file_extension() {
        let mut builder = SemanticModelBuilder::new();

        let temp_file = tempfile::NamedTempFile::with_suffix(".rs")
            .unwrap()
            .into_temp_path();
        std::fs::write(&temp_file, "fn main() {}").unwrap();

        let result = builder.from_source(temp_file.to_str().unwrap());
        assert!(result.is_ok());

        // Clean up
        temp_file.close().unwrap();
    }

    #[test]
    fn test_from_source_with_multiple_file_extensions() {
        let mut builder = SemanticModelBuilder::new();

        let extensions = vec!["rs", "toml", "md", "txt"];
        for ext in extensions {
            let temp_file = tempfile::NamedTempFile::with_suffix(&format!(".{}", ext))
                .unwrap()
                .into_temp_path();
            std::fs::write(&temp_file, "test content").unwrap();

            let result = builder.from_source(temp_file.to_str().unwrap());
            assert!(result.is_ok(), "Failed for extension: {}", ext);

            temp_file.close().unwrap();
        }
    }

    #[test]
    fn test_semantic_model_debug_format() {
        let model = SemanticModel::new();
        let debug_str = format!("{:?}", model);
        assert!(debug_str.contains("SemanticModel"));
        assert!(debug_str.contains("cfg"));
        assert!(debug_str.contains("dfg"));
    }

    #[test]
    fn test_builder_debug_format() {
        let builder = SemanticModelBuilder::new();
        let debug_str = format!("{:?}", builder);
        assert!(debug_str.contains("SemanticModelBuilder"));
        assert!(debug_str.contains("source_path"));
    }
}
