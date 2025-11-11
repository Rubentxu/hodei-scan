//! Parallel batch processing
//!
//! This module provides efficient parallel processing of multiple files
//! against multiple YAML rules using rayon for parallelism.

use crate::match_transform::match_to_fact;
use crate::tree_sitter::TreeSitterMatcher;
use crate::yaml_rule::YamlRule;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

/// Processing results from batch operation
#[derive(Debug, Default, Clone)]
pub struct ProcessingResults {
    /// Number of files processed
    pub processed_files: usize,
    /// Number of matches found
    pub total_matches: usize,
    /// Number of facts generated
    pub total_facts: usize,
}

impl ProcessingResults {
    /// Aggregate another result into this one
    pub fn aggregate(&mut self, other: ProcessingResults) {
        self.processed_files += other.processed_files;
        self.total_matches += other.total_matches;
        self.total_facts += other.total_facts;
    }
}

/// Errors during batch processing
#[derive(Debug, thiserror::Error)]
pub enum ProcessorError {
    #[error("Failed to read file {path}: {source}")]
    FileReadError {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),
}

/// Processor for executing multiple rules against multiple files
pub struct YamlRuleProcessor {
    rules: Arc<Vec<YamlRule>>,
    concurrency: usize,
}

impl YamlRuleProcessor {
    /// Create a new processor
    pub fn new(rules: Vec<YamlRule>) -> Self {
        Self {
            rules: Arc::new(rules),
            concurrency: num_cpus::get(),
        }
    }

    /// Process multiple files in parallel
    pub fn process_files(&self, files: Vec<PathBuf>) -> Result<ProcessingResults, ProcessorError> {
        if files.is_empty() {
            return Ok(ProcessingResults::default());
        }

        // Process files in parallel using rayon
        let results: Vec<Result<ProcessingResults, ProcessorError>> = files
            .par_iter()
            .map(|file_path| self.process_single_file(file_path))
            .collect();

        // Aggregate results
        let mut aggregated = ProcessingResults::default();
        for result in results {
            match result {
                Ok(res) => aggregated.aggregate(res),
                Err(e) => return Err(e),
            }
        }

        Ok(aggregated)
    }

    /// Process a single file
    fn process_single_file(
        &self,
        file_path: &PathBuf,
    ) -> Result<ProcessingResults, ProcessorError> {
        // Read file content
        let source_code =
            fs::read_to_string(file_path).map_err(|source| ProcessorError::FileReadError {
                path: file_path.clone(),
                source,
            })?;

        // Detect language from file extension
        let language = detect_language(file_path).ok_or_else(|| {
            ProcessorError::UnsupportedLanguage(
                file_path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            )
        })?;

        // Process file with all rules
        let mut file_matches = 0;
        let mut file_facts = 0;

        for rule in self.rules.iter() {
            // Skip rules that don't match the language
            if rule.language != language {
                continue;
            }

            // Create matcher for this rule
            let mut matcher = TreeSitterMatcher::new();

            // Execute pattern
            match matcher.execute_pattern(&rule.language, &rule.pattern, &source_code) {
                Ok(matches) => {
                    file_matches += matches.len();

                    // Transform matches to facts
                    for query_match in matches {
                        match match_to_fact(&query_match, rule, &source_code) {
                            Ok(_fact) => {
                                file_facts += 1;
                            }
                            Err(_e) => {
                                // Log error but continue processing
                                // In production, would use proper logging
                            }
                        }
                    }
                }
                Err(_e) => {
                    // Log error but continue with next rule
                    // In production, would use proper logging
                }
            }
        }

        Ok(ProcessingResults {
            processed_files: 1,
            total_matches: file_matches,
            total_facts: file_facts,
        })
    }

    /// Get the concurrency level
    pub fn concurrency(&self) -> usize {
        self.concurrency
    }
}

/// Detect programming language from file extension
fn detect_language(file_path: &PathBuf) -> Option<String> {
    match file_path.extension().and_then(|ext| ext.to_str()) {
        Some("py") => Some("python".to_string()),
        Some("java") => Some("java".to_string()),
        Some("rs") => Some("rust".to_string()),
        Some("js") | Some("jsx") => Some("javascript".to_string()),
        Some("ts") | Some("tsx") => Some("typescript".to_string()),
        Some("go") => Some("go".to_string()),
        Some("php") => Some("php".to_string()),
        Some("rb") => Some("ruby".to_string()),
        Some("cpp") | Some("cc") | Some("cxx") | Some("hpp") | Some("h") => Some("cpp".to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_yaml_rule_processor_creation() {
        let rules = vec![];
        let processor = YamlRuleProcessor::new(rules);
        assert!(processor.concurrency > 0);
        assert_eq!(processor.concurrency, num_cpus::get());
    }

    #[test]
    fn test_processing_results_aggregation() {
        let mut result1 = ProcessingResults {
            processed_files: 5,
            total_matches: 10,
            total_facts: 8,
        };

        let result2 = ProcessingResults {
            processed_files: 3,
            total_matches: 7,
            total_facts: 5,
        };

        result1.aggregate(result2);

        assert_eq!(result1.processed_files, 8);
        assert_eq!(result1.total_matches, 17);
        assert_eq!(result1.total_facts, 13);
    }

    #[test]
    fn test_detect_language_python() {
        let py_file = PathBuf::from("test.py");
        assert_eq!(detect_language(&py_file), Some("python".to_string()));
    }

    #[test]
    fn test_detect_language_java() {
        let java_file = PathBuf::from("Test.java");
        assert_eq!(detect_language(&java_file), Some("java".to_string()));
    }

    #[test]
    fn test_detect_language_rust() {
        let rs_file = PathBuf::from("lib.rs");
        assert_eq!(detect_language(&rs_file), Some("rust".to_string()));
    }

    #[test]
    fn test_detect_language_javascript() {
        let js_file = PathBuf::from("app.js");
        assert_eq!(detect_language(&js_file), Some("javascript".to_string()));

        let jsx_file = PathBuf::from("component.jsx");
        assert_eq!(detect_language(&jsx_file), Some("javascript".to_string()));
    }

    #[test]
    fn test_detect_language_typescript() {
        let ts_file = PathBuf::from("app.ts");
        assert_eq!(detect_language(&ts_file), Some("typescript".to_string()));

        let tsx_file = PathBuf::from("component.tsx");
        assert_eq!(detect_language(&tsx_file), Some("typescript".to_string()));
    }

    #[test]
    fn test_detect_language_go() {
        let go_file = PathBuf::from("main.go");
        assert_eq!(detect_language(&go_file), Some("go".to_string()));
    }

    #[test]
    fn test_detect_language_php() {
        let php_file = PathBuf::from("index.php");
        assert_eq!(detect_language(&php_file), Some("php".to_string()));
    }

    #[test]
    fn test_detect_language_ruby() {
        let rb_file = PathBuf::from("script.rb");
        assert_eq!(detect_language(&rb_file), Some("ruby".to_string()));
    }

    #[test]
    fn test_detect_language_cpp() {
        let cpp_file = PathBuf::from("main.cpp");
        assert_eq!(detect_language(&cpp_file), Some("cpp".to_string()));

        let cc_file = PathBuf::from("test.cc");
        assert_eq!(detect_language(&cc_file), Some("cpp".to_string()));

        let h_file = PathBuf::from("header.h");
        assert_eq!(detect_language(&h_file), Some("cpp".to_string()));
    }

    #[test]
    fn test_detect_language_unknown() {
        let unknown_file = PathBuf::from("file.xyz");
        assert_eq!(detect_language(&unknown_file), None);
    }

    #[test]
    fn test_empty_file_list() {
        let rules = vec![];
        let processor = YamlRuleProcessor::new(rules);

        let result = processor.process_files(vec![]).unwrap();
        assert_eq!(result.processed_files, 0);
        assert_eq!(result.total_matches, 0);
        assert_eq!(result.total_facts, 0);
    }

    #[test]
    fn test_process_files_nonexistent() {
        let rules = vec![];
        let processor = YamlRuleProcessor::new(rules);

        let files = vec![PathBuf::from("/nonexistent/file.py")];
        let result = processor.process_files(files);

        assert!(result.is_err());
    }

    #[test]
    fn test_unsupported_language() {
        let rules = vec![];
        let processor = YamlRuleProcessor::new(rules);

        // Create a temporary file with unsupported extension
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.xyz");
        std::fs::write(&file_path, "test content").unwrap();

        let result = processor.process_files(vec![file_path]);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Language not supported"));
        }
    }
}
