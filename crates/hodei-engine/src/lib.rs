//! hodei-engine: Rule evaluation engine
//!
//! This crate provides the high-performance rule evaluation engine that
//! processes IR facts and generates findings.

#![warn(missing_docs)]

/// Rule evaluation engine
#[derive(Debug)]
pub struct RuleEngine {
    /// Maximum rules to evaluate
    pub max_rules: usize,
}

impl RuleEngine {
    /// Create a new rule engine
    pub fn new() -> Self {
        Self {
            max_rules: 1000,
        }
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new()
    }
}
