use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LineNumber(u32);

impl LineNumber {
    pub fn new(line: u32) -> Result<Self, ()> {
        if line == 0 { Err(()) } else { Ok(Self(line)) }
    }
    pub fn get(&self) -> u32 { self.0 }
}

impl fmt::Display for LineNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.0) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ColumnNumber(u32);

impl ColumnNumber {
    pub fn new(col: u32) -> Result<Self, ()> {
        if col == 0 { Err(()) } else { Ok(Self(col)) }
    }
    pub fn get(&self) -> u32 { self.0 }
}
