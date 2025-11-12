//! Cap'n Proto serialization for IR types (optional feature)
//!
//! This module provides serialization and deserialization capabilities for the IR
//! using Cap'n Proto format, with specific support for Custom FactTypes.

use crate::capnp_impl;

/// Error type from capnp_impl
pub use capnp_impl::CapnpError;

/// Serialize IR to bytes
pub fn serialize_ir_to_bytes<T>(ir: &T) -> Result<Vec<u8>, CapnpError> {
    crate::capnp_impl::serialize_ir_to_bytes(ir)
}

/// Deserialize IR from bytes
pub fn deserialize_ir_from_bytes<T>(bytes: &[u8]) -> Result<T, CapnpError> {
    crate::capnp_impl::deserialize_ir_from_bytes(bytes)
}

/// Feature check for Cap'n Proto support
pub fn is_capnp_enabled() -> bool {
    cfg!(feature = "capnp")
}

#[cfg(test)]
mod tests {
    use super::*;

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
