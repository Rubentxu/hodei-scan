//! Zero-copy IR reader implementation
//!
//! This module provides a simplified zero-copy reader for IR files.
//! The current implementation uses standard serialization but provides
//! the API for future zero-copy optimization.

use std::path::Path;
use thiserror::Error;

/// Zero-copy error types
#[derive(Debug, Error)]
pub enum ZeroCopyError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("IO error: {source}")]
    IoError {
        #[source]
        source: std::io::Error,
    },

    #[error("Invalid IR format: {message}")]
    InvalidFormat { message: String },
}

/// Zero-copy IR reader
///
/// This provides a high-level API for IR file access that can be
/// optimized for zero-copy operations in the future.
pub struct ZeroCopyIR {
    bytes: Vec<u8>,
}

impl ZeroCopyIR {
    /// Load IR from file
    pub fn from_file(path: &Path) -> Result<Self, ZeroCopyError> {
        let bytes = std::fs::read(path).map_err(|e| ZeroCopyError::IoError { source: e })?;

        // Basic validation - in future, this could use rkyv
        if bytes.is_empty() {
            return Err(ZeroCopyError::InvalidFormat {
                message: "Empty file".to_string(),
            });
        }

        Ok(Self { bytes })
    }

    /// Get the number of facts
    pub fn fact_count(&self) -> Result<usize, ZeroCopyError> {
        // For now, deserialize to count facts
        // In a real zero-copy implementation, this would access the archived data directly
        match serde_json::from_slice::<super::IntermediateRepresentation>(&self.bytes) {
            Ok(ir) => Ok(ir.fact_count()),
            Err(e) => Err(ZeroCopyError::InvalidFormat {
                message: format!("Failed to parse: {}", e),
            }),
        }
    }

    /// Get facts from the IR
    pub fn facts(&self) -> Result<Vec<super::Fact>, ZeroCopyError> {
        match serde_json::from_slice::<super::IntermediateRepresentation>(&self.bytes) {
            Ok(ir) => Ok(ir.facts),
            Err(e) => Err(ZeroCopyError::InvalidFormat {
                message: format!("Failed to parse: {}", e),
            }),
        }
    }

    /// Serialize IR to bytes for storage
    pub fn to_bytes(ir: &super::IntermediateRepresentation) -> Result<Vec<u8>, ZeroCopyError> {
        match serde_json::to_vec(ir) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(ZeroCopyError::InvalidFormat {
                message: e.to_string(),
            }),
        }
    }

    /// Check if the file is a valid IR format
    pub fn is_valid_ir_file(path: &Path) -> Result<bool, ZeroCopyError> {
        match Self::from_file(path) {
            Ok(_) => Ok(true),
            Err(ZeroCopyError::FileNotFound { .. }) => Ok(false),
            Err(ZeroCopyError::IoError { .. }) => Ok(false),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Confidence, ExtractorId, Fact, FactType, IntermediateRepresentation, ProjectMetadata,
        ProjectPath, Provenance, SourceLocation,
    };

    #[test]
    fn test_zero_copy_serialization() {
        let metadata = ProjectMetadata::new(
            "test-project".to_string(),
            "1.0.0".to_string(),
            ProjectPath::new(std::path::PathBuf::from(".")),
        );

        let mut ir = IntermediateRepresentation::new(metadata);
        ir.add_fact(Fact::new(
            FactType::Vulnerability {
                cwe_id: Some("CWE-79".to_string()),
                owasp_category: Some("A03:2021".to_string()),
                severity: crate::Severity::Critical,
                cvss_score: Some(9.8),
                description: "XSS vulnerability found".to_string(),
                confidence: Confidence::new(0.9).unwrap(),
            },
            SourceLocation::default(),
            Provenance::new(
                ExtractorId::SemgrepTaint,
                "1.0.0".to_string(),
                Confidence::new(0.9).unwrap(),
            ),
        ));

        let bytes = ZeroCopyIR::to_bytes(&ir).unwrap();
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), &bytes).unwrap();

        let zc_ir = ZeroCopyIR::from_file(temp_file.path()).unwrap();
        let fact_count = zc_ir.fact_count().unwrap();
        assert_eq!(fact_count, 1);

        let facts = zc_ir.facts().unwrap();
        assert_eq!(facts.len(), 1);
    }

    #[test]
    fn test_invalid_file() {
        let path = std::path::Path::new("nonexistent.ir");
        let result = ZeroCopyIR::from_file(path);
        // Just verify it returns an error
        assert!(result.is_err());
    }

    #[test]
    fn test_is_valid_ir_file() {
        assert!(!ZeroCopyIR::is_valid_ir_file(std::path::Path::new("nonexistent.ir")).unwrap());

        let metadata = ProjectMetadata::new(
            "test-project".to_string(),
            "1.0.0".to_string(),
            ProjectPath::new(std::path::PathBuf::from(".")),
        );
        let ir = IntermediateRepresentation::new(metadata);
        let bytes = ZeroCopyIR::to_bytes(&ir).unwrap();

        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), &bytes).unwrap();

        assert!(ZeroCopyIR::is_valid_ir_file(temp_file.path()).unwrap());
    }
}
