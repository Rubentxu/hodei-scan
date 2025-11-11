//! hodei-engine: Rule evaluation engine
//!
//! This crate provides the high-performance rule evaluation engine that
//! processes IR facts and generates findings.

#![warn(missing_docs)]

pub mod engine;
pub mod extractor;
pub mod gates;
pub mod store;

// Include Cap'n Proto generated modules (or manual implementation)
#[cfg(feature = "capnp")]
pub mod extractor_protocol_capnp {
    include!(concat!(env!("OUT_DIR"), "/extractor_protocol_capnp.rs"));
}

#[cfg(not(feature = "capnp"))]
pub mod extractor_protocol_capnp {
    include!("extractor_protocol_capnp.rs");
}

pub use engine::*;
pub use extractor::*;
pub use gates::*;
pub use store::*;
