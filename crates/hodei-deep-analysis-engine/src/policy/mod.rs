//! Policy definitions for taint analysis and connascence detection
//!
//! This module provides configuration parsing and policy management.

use hodei_ir::types::Severity;
use serde::{Deserialize, Serialize};

/// Taint policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaintPolicy {
    pub sources: Vec<SourceDefinition>,
    pub sinks: Vec<SinkDefinition>,
    pub sanitizers: Vec<SanitizerDefinition>,
}

impl Default for TaintPolicy {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            sinks: Vec::new(),
            sanitizers: Vec::new(),
        }
    }
}

/// Source definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceDefinition {
    pub pattern: String,
    pub source_type: String,
    pub tags: Vec<DataTag>,
}

/// Sink definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SinkDefinition {
    pub pattern: String,
    pub category: String,
    pub severity: Severity,
}

/// Sanitizer definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizerDefinition {
    pub pattern: String,
    pub method: Option<String>,
}

/// Data tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataTag {
    PII,
    Finance,
    Credentials,
    UserInput,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_loading() {
        // TODO: Implement tests
    }
}
