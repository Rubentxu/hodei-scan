//! Core types module

pub mod common;
pub mod confidence;
pub mod extractor_id;
pub mod fact_id;
pub mod flow_id;
pub mod line_number;
pub mod project_path;
pub mod provenance;
pub mod source_location;

// Re-exports
pub use common::*;
pub use confidence::*;
pub use extractor_id::*;
pub use fact_id::*;
pub use flow_id::*;
pub use line_number::*;
pub use project_path::*;
pub use provenance::*;
pub use source_location::*;

// ColumnNumber inline
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ColumnNumber(u32);

impl ColumnNumber {
    pub fn new(col: u32) -> Result<Self, ()> {
        if col == 0 { Err(()) } else { Ok(Self(col)) }
    }
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for ColumnNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
