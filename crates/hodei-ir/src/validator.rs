//! IRValidator for checking integrity of intermediate representation

use thiserror::Error;

/// Error types for validation failures
#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Duplicate FactId detected: {0}")]
    DuplicateFactId(String),
    
    #[error("Schema version mismatch: expected {expected}, got {actual}")]
    SchemaVersionMismatch { expected: String, actual: String },
    
    #[error("Invalid fact type: {0}")]
    InvalidFactType(String),
}

/// IR validator for checking integrity
pub struct IRValidator {
    expected_schema: String,
}

impl IRValidator {
    pub fn new(expected_schema: String) -> Self {
        Self { expected_schema }
    }
    
    /// Validate an intermediate representation
    pub fn validate(&self, ir: &super::IntermediateRepresentation) -> Result<(), ValidationError> {
        // Check schema version
        if ir.schema_version != self.expected_schema {
            return Err(ValidationError::SchemaVersionMismatch {
                expected: self.expected_schema.clone(),
                actual: ir.schema_version.clone(),
            });
        }
        
        // Check for duplicate FactIds
        let mut seen_ids = std::collections::HashSet::new();
        for fact in &ir.facts {
            if !seen_ids.insert(fact.id) {
                return Err(ValidationError::DuplicateFactId(format!("{}", fact.id.0)));
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validator_accepts_valid_ir() {
        use crate::*;
        let metadata = ProjectMetadata::new(
            "test".to_string(),
            "1.0".to_string(),
            ProjectPath::new(std::path::PathBuf::new())
        );
        let ir = IntermediateRepresentation::new(metadata);
        let validator = IRValidator::new("3.2.0".to_string());
        assert!(validator.validate(&ir).is_ok());
    }
}
