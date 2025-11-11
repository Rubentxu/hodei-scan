//! Cap'n Proto serialization for IR types (optional feature)
//!
//! This module provides serialization and deserialization capabilities for the IR
//! using Cap'n Proto format, with specific support for Custom FactTypes.

#[cfg(feature = "capnp")]
pub mod capnp_impl;

#[cfg(not(feature = "capnp"))]
pub mod capnp_stub {
    /// Stub error type for when Cap'n Proto is not available
    #[derive(thiserror::Error, Debug)]
    pub enum CapnpError {
        #[error("Cap'n Proto support not enabled. Build with --features capnp")]
        NotEnabled,
    }

    /// Stub function that returns an error when Cap'n Proto is not enabled
    pub fn serialize_ir_to_bytes<T>(_ir: &T) -> Result<Vec<u8>, CapnpError> {
        Err(CapnpError::NotEnabled)
    }

    /// Stub function that returns an error when Cap'n Proto is not enabled
    pub fn deserialize_ir_from_bytes<T>(_bytes: &[u8]) -> Result<T, CapnpError> {
        Err(CapnpError::NotEnabled)
    }
}

#[cfg(feature = "capnp")]
pub use capnp_impl::*;

#[cfg(not(feature = "capnp"))]
pub use capnp_stub::*;

/// Feature check for Cap'n Proto support
pub fn is_capnp_enabled() -> bool {
    cfg!(feature = "capnp")
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_capnp_feature_check() {
        #[cfg(feature = "capnp")]
        {
            println!("Cap'n Proto support is ENABLED");
        }

        #[cfg(not(feature = "capnp"))]
        {
            println!("Cap'n Proto support is DISABLED (using stubs)");
        }
    }
}
