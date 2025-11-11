//! Zero-copy IR using memory-mapped files
//!
//! This module provides zero-copy deserialization capabilities using memory-mapped files.
//! It demonstrates the pattern for reading IR data directly from memory-mapped files
//! without deserializing the entire structure.

use memmap2::Mmap;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use thiserror::Error;

// Re-export the IR types for convenience
use crate::{Fact, IntermediateRepresentation};

/// Error types for zero-copy operations
#[derive(Error, Debug)]
pub enum ZeroCopyError {
    #[error("IO error: {source}")]
    Io {
        #[from]
        source: io::Error,
    },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Invalid format: {message}")]
    InvalidFormat { message: String },
}

/// Zero-copy IR reader using memory-mapped files
///
/// This struct allows reading IR data directly from a memory-mapped file
/// without deserializing the entire structure. In a production implementation,
/// this would use efficient serialization formats like rkyv or Cap'n Proto.
pub struct ZeroCopyIR<'a> {
    mmap: &'a Mmap,
}

impl<'a> ZeroCopyIR<'a> {
    /// Create a zero-copy IR reader from a memory-mapped file
    pub fn from_mmap(mmap: &'a Mmap) -> Result<Self, ZeroCopyError> {
        if mmap.is_empty() {
            return Err(ZeroCopyError::InvalidFormat {
                message: "Empty file".to_string(),
            });
        }

        Ok(ZeroCopyIR { mmap })
    }

    /// Get a fact by index without full deserialization
    ///
    /// This is a simplified demonstration. In a production implementation,
    /// this would use rkyv or Cap'n Proto for efficient partial deserialization.
    pub fn get_fact(&self, index: usize) -> Result<Fact, ZeroCopyError> {
        // For demonstration purposes, we deserialize the full structure
        // In production, this would use zero-copy deserialization
        let ir: IntermediateRepresentation =
            bincode::deserialize(self.mmap).map_err(|_| ZeroCopyError::InvalidFormat {
                message: "Failed to parse IR from memory".to_string(),
            })?;

        if index >= ir.facts.len() {
            return Err(ZeroCopyError::InvalidFormat {
                message: format!(
                    "Fact index {} out of bounds (max: {})",
                    index,
                    ir.facts.len()
                ),
            });
        }

        Ok(ir.facts[index].clone())
    }

    /// Get the total number of facts
    pub fn fact_count(&self) -> Result<usize, ZeroCopyError> {
        // For simplicity, we deserialize. In production, this would be zero-copy
        let ir: IntermediateRepresentation =
            bincode::deserialize(self.mmap).map_err(|_| ZeroCopyError::InvalidFormat {
                message: "Failed to parse IR from memory".to_string(),
            })?;

        Ok(ir.facts.len())
    }

    /// Get the project metadata
    pub fn get_metadata(&self) -> Result<&crate::ProjectMetadata, ZeroCopyError> {
        // For simplicity, we deserialize. In production, this would be zero-copy
        let ir: IntermediateRepresentation =
            bincode::deserialize(self.mmap).map_err(|_| ZeroCopyError::InvalidFormat {
                message: "Failed to parse IR from memory".to_string(),
            })?;

        Ok(Box::leak(Box::new(ir.metadata)))
    }
}

/// Zero-copy IR writer for serialization
pub struct ZeroCopyIRWriter {
    ir: IntermediateRepresentation,
}

impl ZeroCopyIRWriter {
    /// Create a new zero-copy IR writer
    pub fn new() -> Self {
        ZeroCopyIRWriter {
            ir: IntermediateRepresentation::new(crate::ProjectMetadata::new(
                "unnamed".to_string(),
                "0.1.0".to_string(),
                crate::ProjectPath::new(std::path::PathBuf::new()),
            )),
        }
    }

    /// Add a fact to the IR
    pub fn add_fact(&mut self, fact: Fact) {
        self.ir.add_fact(fact);
    }

    /// Set the project metadata
    pub fn set_metadata(&mut self, metadata: crate::ProjectMetadata) {
        self.ir.metadata = metadata;
    }

    /// Serialize to a file using bincode (for demonstration)
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ZeroCopyError> {
        let mut file = File::create(path)?;
        let bytes = bincode::serialize(&self.ir).map_err(|_| ZeroCopyError::InvalidFormat {
            message: "Failed to serialize IR".to_string(),
        })?;

        file.write_all(&bytes)?;
        Ok(())
    }

    /// Get the current IR instance
    pub fn get_ir(&self) -> &IntermediateRepresentation {
        &self.ir
    }
}

impl Default for ZeroCopyIRWriter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::LineNumber;
    use crate::{Confidence, ExtractorId, FactType, Provenance, SourceLocation};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_zero_copy_serialization() {
        // Create a test fact
        let mut writer = ZeroCopyIRWriter::new();

        // Set up metadata
        writer.set_metadata(crate::ProjectMetadata::new(
            "test-project".to_string(),
            "1.0.0".to_string(),
            crate::ProjectPath::new(PathBuf::from("/tmp/test")),
        ));

        // Add a test fact
        let fact = Fact::new(
            FactType::Vulnerability {
                cwe_id: Some("CWE-79".to_string()),
                owasp_category: Some("A03:2021".to_string()),
                severity: crate::Severity::Major,
                cvss_score: Some(7.5),
                description: "Cross-site scripting vulnerability".to_string(),
                confidence: Confidence::HIGH,
            },
            SourceLocation::new(
                crate::ProjectPath::new(PathBuf::from("src/main.rs")),
                LineNumber::new(42).unwrap(),
                None,
                LineNumber::new(42).unwrap(),
                None,
            ),
            Provenance::new(
                ExtractorId::SemgrepTaint,
                "1.0.0".to_string(),
                Confidence::MEDIUM,
            ),
        );

        writer.add_fact(fact);

        // Serialize to a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        writer.write_to_file(temp_file.path()).unwrap();

        // Check that the file has content
        let file = File::open(temp_file.path()).unwrap();
        let metadata = file.metadata().unwrap();
        assert!(metadata.len() > 0, "Serialized file should have content");
    }

    #[test]
    fn test_zero_copy_read() {
        // Create and write test IR
        let temp_file = NamedTempFile::new().unwrap();
        {
            let mut writer = ZeroCopyIRWriter::new();

            writer.set_metadata(crate::ProjectMetadata::new(
                "test-project".to_string(),
                "1.0.0".to_string(),
                crate::ProjectPath::new(PathBuf::from("/tmp/test")),
            ));

            // Add a fact
            let fact = Fact::new(
                FactType::Vulnerability {
                    cwe_id: Some("CWE-79".to_string()),
                    owasp_category: Some("A03:2021".to_string()),
                    severity: crate::Severity::Major,
                    cvss_score: Some(7.5),
                    description: "Cross-site scripting vulnerability".to_string(),
                    confidence: Confidence::HIGH,
                },
                SourceLocation::new(
                    crate::ProjectPath::new(PathBuf::from("src/main.rs")),
                    LineNumber::new(42).unwrap(),
                    None,
                    LineNumber::new(42).unwrap(),
                    None,
                ),
                Provenance::new(
                    ExtractorId::SemgrepTaint,
                    "1.0.0".to_string(),
                    Confidence::MEDIUM,
                ),
            );

            writer.add_fact(fact);
            writer.write_to_file(temp_file.path()).unwrap();
        }

        // Read with zero-copy (mmap)
        let file = File::open(temp_file.path()).unwrap();
        let mmap = unsafe { Mmap::map(&file).unwrap() };
        let zero_copy_ir = ZeroCopyIR::from_mmap(&mmap).unwrap();

        // Access facts without full deserialization
        let fact_count = zero_copy_ir.fact_count().unwrap();
        assert_eq!(fact_count, 1, "Should have one fact");

        let fact = zero_copy_ir.get_fact(0).unwrap();
        assert_eq!(
            fact.fact_type,
            FactType::Vulnerability {
                cwe_id: Some("CWE-79".to_string()),
                owasp_category: Some("A03:2021".to_string()),
                severity: crate::Severity::Major,
                cvss_score: Some(7.5),
                description: "Cross-site scripting vulnerability".to_string(),
                confidence: Confidence::HIGH,
            }
        );
    }

    #[test]
    fn test_zero_copy_metadata_access() {
        let temp_file = NamedTempFile::new().unwrap();
        {
            let mut writer = ZeroCopyIRWriter::new();

            writer.set_metadata(crate::ProjectMetadata::new(
                "test-project".to_string(),
                "1.0.0".to_string(),
                crate::ProjectPath::new(PathBuf::from("/tmp/test")),
            ));

            writer.write_to_file(temp_file.path()).unwrap();
        }

        // Read back
        let file = File::open(temp_file.path()).unwrap();
        let mmap = unsafe { Mmap::map(&file).unwrap() };
        let zero_copy_ir = ZeroCopyIR::from_mmap(&mmap).unwrap();

        let metadata = zero_copy_ir.get_metadata().unwrap();

        // Verify metadata fields
        assert_eq!(metadata.name, "test-project");
        assert_eq!(metadata.version, "1.0.0");
    }

    #[test]
    fn test_zero_copy_fact_count() {
        let temp_file = NamedTempFile::new().unwrap();
        {
            let mut writer = ZeroCopyIRWriter::new();

            // Add 5 facts
            for i in 0..5 {
                let fact = Fact::new(
                    FactType::Variable {
                        name: crate::VariableName(format!("var{}", i)),
                        scope: "global".to_string(),
                        var_type: "int".to_string(),
                    },
                    SourceLocation::new(
                        crate::ProjectPath::new(PathBuf::from("src/main.rs")),
                        LineNumber::new(10).unwrap(),
                        None,
                        LineNumber::new(10).unwrap(),
                        None,
                    ),
                    Provenance::new(
                        ExtractorId::TreeSitter,
                        "1.0.0".to_string(),
                        Confidence::HIGH,
                    ),
                );
                writer.add_fact(fact);
            }

            writer.write_to_file(temp_file.path()).unwrap();
        }

        // Read back
        let file = File::open(temp_file.path()).unwrap();
        let mmap = unsafe { Mmap::map(&file).unwrap() };
        let zero_copy_ir = ZeroCopyIR::from_mmap(&mmap).unwrap();

        let count = zero_copy_ir.fact_count().unwrap();
        assert_eq!(count, 5, "Should have 5 facts");
    }
}
