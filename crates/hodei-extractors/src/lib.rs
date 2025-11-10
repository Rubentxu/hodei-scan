//! hodei-extractors: Source code analyzers
//!
//! This crate provides extractors that analyze source code and populate
//! the intermediate representation (IR) with facts.

#![warn(missing_docs)]

use hodei_ir::{
    ColumnNumber, Confidence, ExtractorId, Fact, FactType, FlowId, FunctionName,
    IntermediateRepresentation, LineNumber, ProjectMetadata, ProjectPath, Provenance, Severity,
    SourceLocation, VariableName,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during extraction
#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}

/// Trait for source code extractors
pub trait Extractor: Send + Sync {
    /// Extract facts from source code
    fn extract(&self, file_path: &Path) -> Result<Vec<Fact>, ExtractError>;

    /// Get the extractor ID
    fn id(&self) -> ExtractorId;

    /// Get the extractor version
    fn version(&self) -> &'static str;
}

/// Base extractor implementation
pub struct BaseExtractor {
    id: ExtractorId,
    version: &'static str,
}

impl BaseExtractor {
    pub fn new(id: ExtractorId, version: &'static str) -> Self {
        Self { id, version }
    }
}

/// Simple regex-based extractor for demonstration
pub struct RegexExtractor {
    base: BaseExtractor,
    patterns: Vec<(String, FactType)>,
}

impl RegexExtractor {
    pub fn new(id: ExtractorId, version: &'static str, patterns: Vec<(String, FactType)>) -> Self {
        Self {
            base: BaseExtractor::new(id, version),
            patterns,
        }
    }

    /// Extract facts by matching patterns in file content
    fn extract_patterns(&self, content: &str, file_path: &Path) -> Vec<Fact> {
        let mut facts = Vec::new();
        let path = ProjectPath::new(PathBuf::from(file_path));
        let provenance = Provenance::new(
            self.base.id,
            self.base.version.to_string(),
            Confidence::MEDIUM,
        );

        for (line_num, line) in content.lines().enumerate() {
            let line_number = LineNumber::new((line_num as u32) + 1).unwrap();

            for (pattern, fact_type) in &self.patterns {
                if line.contains(pattern) {
                    let location =
                        SourceLocation::new(path.clone(), line_number, None, line_number, None);

                    let fact = match fact_type {
                        FactType::CodeSmell { .. } => Fact::new(
                            FactType::CodeSmell {
                                smell_type: "TODO".to_string(),
                                severity: Severity::Minor,
                                message: format!("Found pattern: {}", pattern),
                            },
                            location,
                            provenance.clone(),
                        ),
                        _ => Fact::new(fact_type.clone(), location, provenance.clone()),
                    };

                    facts.push(fact);
                }
            }
        }

        facts
    }
}

impl Extractor for RegexExtractor {
    fn extract(&self, file_path: &Path) -> Result<Vec<Fact>, ExtractError> {
        let content = std::fs::read_to_string(file_path)
            .map_err(|e| ExtractError::FileNotFound(file_path.to_path_buf()))?;

        Ok(self.extract_patterns(&content, file_path))
    }

    fn id(&self) -> ExtractorId {
        self.base.id
    }

    fn version(&self) -> &'static str {
        self.base.version
    }
}

/// Extract all facts from a directory
pub async fn extract_from_directory(
    dir: &Path,
    extractors: &[Box<dyn Extractor>],
) -> Result<IntermediateRepresentation, ExtractError> {
    let metadata = ProjectMetadata::new(
        "extraction".to_string(),
        "1.0".to_string(),
        ProjectPath::new(PathBuf::from(dir)),
    );

    let mut ir = IntermediateRepresentation::new(metadata);
    let mut all_facts = Vec::new();

    // Collect all Rust files
    let files: Vec<PathBuf> = walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        .map(|e| e.path().to_path_buf())
        .collect();

    // Process files
    for file_path in files {
        for extractor in extractors {
            match extractor.extract(&file_path) {
                Ok(facts) => all_facts.extend(facts),
                Err(e) => eprintln!("Extraction error for {:?}: {}", file_path, e),
            }
        }
    }

    // Add facts to IR
    for fact in all_facts {
        ir.add_fact(fact);
    }

    Ok(ir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_regex_extractor() {
        let extractor = RegexExtractor::new(
            ExtractorId::Custom,
            "1.0.0",
            vec![
                (
                    "TODO".to_string(),
                    FactType::CodeSmell {
                        smell_type: "TODO".to_string(),
                        severity: Severity::Minor,
                        message: "TODO comment".to_string(),
                    },
                ),
                (
                    "FIXME".to_string(),
                    FactType::CodeSmell {
                        smell_type: "FIXME".to_string(),
                        severity: Severity::Major,
                        message: "FIXME comment".to_string(),
                    },
                ),
            ],
        );

        // Create a temporary test file
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        fs::write(&test_file, "// TODO: implement this\nfn main() {}").unwrap();

        let facts = extractor.extract(&test_file).unwrap();

        assert_eq!(facts.len(), 1);
        assert_eq!(
            facts[0].fact_type,
            FactType::CodeSmell {
                smell_type: "TODO".to_string(),
                severity: Severity::Minor,
                message: "Found pattern: TODO".to_string(),
            }
        );
    }
}
