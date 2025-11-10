//! Column number type for source code locations
//!
//! Ensures column numbers are always >= 1 (column 0 is invalid).

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u32")]
pub struct ColumnNumber(u32);

impl ColumnNumber {
    pub fn new(col: u32) -> Result<Self, super::line_number::LineNumberError> {
        if col == 0 {
            return Err(super::line_number::LineNumberError::ZeroLine);
        }
        Ok(Self(col))
    }
    
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for ColumnNumber {
    type Error = super::line_number::LineNumberError;
    
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Display for ColumnNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_column_number_rejects_zero() {
        assert!(ColumnNumber::new(0).is_err());
    }
    
    #[test]
    fn test_column_number_accepts_positive() {
        assert!(ColumnNumber::new(1).is_ok());
        assert!(ColumnNumber::new(50).is_ok());
    }
}
