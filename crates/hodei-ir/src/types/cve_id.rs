//! CVE (Common Vulnerabilities and Exposures) ID type

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// CVE identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CveId {
    pub year: u16,
    pub identifier: u32,
}

#[derive(Debug, Error)]
pub enum CveIdError {
    #[error("Invalid CVE prefix: expected 'CVE-'")]
    InvalidPrefix,
    #[error("Invalid CVE format")]
    InvalidFormat,
}

impl CveId {
    pub fn new(year: u16, identifier: u32) -> Self {
        Self { year, identifier }
    }
    
    pub fn as_str(&self) -> String {
        format!("CVE-{}-{:07}", self.year, self.identifier)
    }
}

impl TryFrom<&str> for CveId {
    type Error = CveIdError;
    
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if !value.starts_with("CVE-") {
            return Err(CveIdError::InvalidPrefix);
        }
        
        let parts: Vec<&str> = value[4..].split('-').collect();
        if parts.len() != 2 {
            return Err(CveIdError::InvalidFormat);
        }
        
        let year = parts[0].parse().map_err(|_| CveIdError::InvalidFormat)?;
        let identifier = parts[1].parse().map_err(|_| CveIdError::InvalidFormat)?;
        
        Ok(Self { year, identifier })
    }
}

impl fmt::Display for CveId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
