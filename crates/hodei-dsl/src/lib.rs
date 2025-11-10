//! hodei-dsl: DSL parser for hodei-scan rules
//!
//! This crate provides a Cedar-like DSL for defining security and quality rules
//! that can be evaluated against IR facts.

#![warn(missing_docs)]

/// Rule parser
#[derive(Debug)]
pub struct RuleParser;

impl RuleParser {
    /// Parse a rule from string
    pub fn parse_rule(_rule: &str) -> Result<String, String> {
        Ok("Rule parsed".to_string())
    }
}
