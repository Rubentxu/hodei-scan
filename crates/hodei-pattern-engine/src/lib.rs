//! # hodei-pattern-engine
//!
//! Declarative Pattern Engine for static code analysis using Tree-sitter and YAML rules.
//!
//! This module provides a framework for defining and executing code analysis rules
//! in a declarative YAML format, enabling rapid rule development without Rust programming.
//!
//! ## Architecture
//!
//! ```text
//! YAML Rule
//!     ↓ (parse)
//! Tree-sitter Query
//!     ↓ (execute)
//! AST Matches
//!     ↓ (transform)
//! Facts (IR)
//!     ↓ (RuleEngine)
//! Findings
//! ```
//!
//! ## Quick Start
//!
//! ```rust
//! use hodei_pattern_engine::{YamlRule, YamlRuleLoader, TreeSitterMatcher};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create YAML rule loader
//! let loader = YamlRuleLoader::new();
//!
//! // Execute rule against source code
//! let mut matcher = TreeSitterMatcher::new();
//! let source_code = "try { } catch (Exception e) { }";
//!
//! let matches = matcher.execute_pattern(
//!     "java",
//!     "try_statement catch_clause: (empty)",
//!     source_code
//! )?;
//!
//! # Ok(())
//! # }
//! ```

/// YAML Rule definitions and parsing
pub mod yaml_rule;

/// Tree-sitter integration and query execution
pub mod tree_sitter;

/// Match transformation to Facts
pub mod match_transform;

/// Parallel batch processing
pub mod batch_processor;

/// Testing framework for YAML rules
pub mod testing;

// Re-export commonly used types
pub use batch_processor::{ProcessingResults, YamlRuleProcessor};
pub use match_transform::{match_to_fact, TransformError};
pub use tree_sitter::{MatcherError, QueryCache, TreeSitterMatcher};
pub use yaml_rule::{YamlError, YamlRule, YamlRuleLoader};

/// Errors that can occur in the pattern engine
#[derive(Debug, thiserror::Error)]
pub enum PatternEngineError {
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] YamlError),

    #[error("Tree-sitter matcher error: {0}")]
    MatcherError(#[from] MatcherError),

    #[error("Transform error: {0}")]
    TransformError(#[from] TransformError),
}
