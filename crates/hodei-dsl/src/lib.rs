//! hodei-dsl: Parser for Hodei Scan rule DSL
//!
//! This crate provides a formal parser for the Hodei Scan rule language,
//! using PEG grammar with pest. The DSL is type-safe and inspired by Cedar.

#![warn(missing_docs)]

pub mod ast;
pub mod error;
pub mod parser;
pub mod security;
pub mod type_checker;

#[cfg(test)]
mod tests;

pub use ast::*;
pub use error::*;
pub use parser::parse_file;
pub use security::*;
pub use type_checker::TypeChecker;

/// Parse a rule file and return the AST
pub fn parse_rule_file(input: &str) -> ParseResult<RuleFile> {
    parse_file(input)
}
