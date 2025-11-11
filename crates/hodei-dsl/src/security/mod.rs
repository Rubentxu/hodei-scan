//! DSL Security Validation and Sandboxing
//!
//! This module provides security validation for DSL input and runtime sandboxing
//! to prevent malicious code execution and resource exhaustion attacks.

pub mod sandbox;
pub mod validator;

pub use sandbox::*;
pub use validator::*;

#[cfg(test)]
mod security_tests;
