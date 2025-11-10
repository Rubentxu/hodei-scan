//! Severity level for findings and issues

use serde::{Deserialize, Serialize};
use std::fmt;

/// Severity level for security vulnerabilities and code issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Minor,
    Major,
    Critical,
    Blocker,
}

impl Severity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Minor => "Minor",
            Self::Major => "Major",
            Self::Critical => "Critical",
            Self::Blocker => "Blocker",
        }
    }
    
    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Info => 0,
            Self::Minor => 0,
            Self::Major => 1,
            Self::Critical => 2,
            Self::Blocker => 3,
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Minor);
        assert!(Severity::Minor < Severity::Major);
        assert!(Severity::Major < Severity::Critical);
        assert!(Severity::Critical < Severity::Blocker);
    }
    
    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Critical), "Critical");
    }
}
