//! Cap'n Proto serialization (simplified for compilation)
//!
//! This module provides serialization capabilities for IR using Cap'n Proto.
//! Currently simplified to avoid compilation errors in base code.

use thiserror::Error;

/// Error types for Cap'n Proto operations
#[derive(Error, Debug)]
pub enum CapnpError {
    #[error("Not implemented: {message}")]
    NotImplemented { message: String },
}

/// Serialize IR to bytes (stub)
pub fn serialize_ir_to_bytes<T>(_ir: &T) -> Result<Vec<u8>, CapnpError> {
    Err(CapnpError::NotImplemented {
        message: "Cap'n Proto serialization is temporarily disabled".to_string(),
    })
}

/// Deserialize IR from bytes (stub)
pub fn deserialize_ir_from_bytes<T>(_bytes: &[u8]) -> Result<T, CapnpError> {
    Err(CapnpError::NotImplemented {
        message: "Cap'n Proto deserialization is temporarily disabled".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_functions() {
        // These are just stubs to satisfy the module interface
        assert!(serialize_ir_to_bytes(&"test".to_string()).is_err());
        assert!(deserialize_ir_from_bytes::<&str>(b"test").is_err());
    }
}
