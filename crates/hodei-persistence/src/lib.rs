//! hodei-persistence: Persistence layer
//!
//! This crate provides persistence capabilities for IR and findings.

use hodei_ir::IntermediateRepresentation;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// Persistence errors
#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Save IR to JSON file
pub fn save_ir_to_json(
    ir: &IntermediateRepresentation,
    path: &Path,
) -> Result<(), PersistenceError> {
    let json = serde_json::to_string_pretty(ir)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load IR from JSON file
pub fn load_ir_from_json(path: &Path) -> Result<IntermediateRepresentation, PersistenceError> {
    let content = std::fs::read_to_string(path)?;
    let ir = serde_json::from_str(&content)?;
    Ok(ir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_load_ir() {
        let metadata = hodei_ir::ProjectMetadata::new(
            "test".to_string(),
            "1.0".to_string(),
            hodei_ir::ProjectPath::new(std::path::PathBuf::from(".")),
        );
        let ir = hodei_ir::IntermediateRepresentation::new(metadata);

        let path = std::path::PathBuf::from("/tmp/test-ir.json");
        save_ir_to_json(&ir, &path).unwrap();
        let loaded_ir = load_ir_from_json(&path).unwrap();

        assert_eq!(loaded_ir.fact_count(), 0);

        std::fs::remove_file(&path).ok();
    }
}
