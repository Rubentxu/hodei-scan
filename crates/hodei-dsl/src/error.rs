//! Error types for DSL parsing and type checking

use thiserror::Error;

/// Result type for DSL operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Main error type for all DSL operations
#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("Pest parsing error")]
    PestError(String),

    #[error("IO error: {message}")]
    IoError { message: String },

    #[error("Type error: {0}")]
    TypeError(#[from] TypeError),

    #[error("Missing field: {field}")]
    MissingField { field: String },

    #[error("Unexpected token: {token}, expected: {expected}")]
    UnexpectedToken { token: String, expected: String },

    #[error("Invalid expression")]
    InvalidExpression,

    #[error("Invalid number")]
    InvalidNumber,

    #[error("Invalid boolean")]
    InvalidBoolean,

    #[error("Invalid severity value")]
    InvalidSeverity,

    #[error("Invalid confidence value")]
    InvalidConfidence,

    #[error("Invalid operator")]
    InvalidOperator,

    #[error("Invalid literal")]
    InvalidLiteral,

    #[error("Expected a single rule")]
    ExpectedSingleRule,

    #[error("Custom error: {0}")]
    Custom(String),
}

impl ParseError {
    pub fn msg(msg: impl Into<String>) -> Self {
        Self::Custom(msg.into())
    }
}

/// Result type for type checking
pub type TypeResult<T> = Result<T, TypeError>;

/// Type checking errors
#[derive(Error, Debug, PartialEq, Clone)]
pub enum TypeError {
    #[error("Unknown fact type: {0:?}")]
    UnknownFactType(String),

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Type mismatch: expected {expected:?}, found {found:?}")]
    TypeMismatch { expected: String, found: String },

    #[error("Expected number type")]
    ExpectedNumber,

    #[error("Expected boolean type")]
    ExpectedBoolean,

    #[error("No such field '{field}' on type '{ty}'")]
    NoSuchField { ty: String, field: String },

    #[error("Cannot access field on type")]
    CannotAccessField { ty: String },

    #[error("Unknown function: {0}")]
    UnknownFunction(String),
}
