//! Core types module

pub mod confidence;
pub mod line_number;
pub mod flow_id;
pub mod common;
pub mod project_path;
pub mod metadata;

// Re-exports
pub use confidence::*;
pub use line_number::*;
pub use flow_id::*;
pub use common::*;
pub use project_path::*;
pub use metadata::*;

// ColumnNumber inline
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ColumnNumber(u32);

impl ColumnNumber {
    pub fn new(col: u32) -> Result<Self, ()> {
        if col == 0 { Err(()) } else { Ok(Self(col)) }
    }
    pub fn get(&self) -> u32 { self.0 }
}

impl fmt::Display for ColumnNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}
