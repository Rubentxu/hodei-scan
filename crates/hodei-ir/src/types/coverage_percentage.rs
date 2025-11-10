//! Coverage percentage type

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CoveragePercentage {
    value: f32,
}

#[derive(Debug, Error)]
pub enum CoverageError {
    #[error("Coverage value {0} is out of range [0.0, 100.0]")]
    OutOfRange(f32),
}

impl CoveragePercentage {
    pub fn new(value: f32) -> Result<Self, CoverageError> {
        if !(0.0..=100.0).contains(&value) {
            return Err(CoverageError::OutOfRange(value));
        }
        Ok(Self { value })
    }
    
    pub fn value(&self) -> f32 {
        self.value
    }
    
    pub fn is_low(&self) -> bool {
        self.value < 50.0
    }
    
    pub fn is_acceptable(&self) -> bool {
        (50.0..=80.0).contains(&self.value)
    }
    
    pub fn is_excellent(&self) -> bool {
        self.value > 80.0
    }
}

impl fmt::Display for CoveragePercentage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}%", self.value)
    }
}

impl TryFrom<f32> for CoveragePercentage {
    type Error = CoverageError;
    
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
