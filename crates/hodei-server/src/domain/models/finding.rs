/// Domain model for security findings
/// This is pure business logic, no infrastructure dependencies
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Finding severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    Major,
    Minor,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "critical"),
            Severity::Major => write!(f, "major"),
            Severity::Minor => write!(f, "minor"),
            Severity::Info => write!(f, "info"),
        }
    }
}

/// Finding location information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindingLocation {
    pub file: String,
    pub line: Option<u64>,
    pub column: Option<u64>,
}

/// Core security finding entity
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub fact_type: String,
    pub severity: Severity,
    pub fingerprint: String,
    pub location: FindingLocation,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}
