//! hodei-engine: Rule evaluation engine
//!
//! This crate provides the high-performance rule evaluation engine that
//! processes IR facts and generates findings.

#![warn(missing_docs)]

pub mod engine;
pub mod gates;
pub mod store;

pub use engine::*;
pub use gates::*;
pub use store::*;
