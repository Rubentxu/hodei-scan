//! Confidence type

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Confidence(f64);

#[derive(Debug, thiserror::Error)]
pub enum ConfidenceError {
    #[error("Out of range")]
    OutOfRange,
    #[error("Not finite")]
    NotFinite,
}

impl Confidence {
    pub fn new(value: f64) -> Result<Self, ConfidenceError> {
        if !value.is_finite() {
            return Err(ConfidenceError::NotFinite);
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(ConfidenceError::OutOfRange);
        }
        Ok(Self(value))
    }

    pub fn get(&self) -> f64 {
        self.0
    }
    pub const HIGH: Self = Self(0.9);
    pub const MEDIUM: Self = Self(0.6);
    pub const LOW: Self = Self(0.3);
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl Eq for Confidence {}
impl std::hash::Hash for Confidence {
    fn hash<H: std::hash::Hasher>(&self, s: &mut H) {
        self.0.to_bits().hash(s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_confidence() {
        assert!(Confidence::new(0.5).is_ok());
    }
}
