//! Error types for DSL parsing and type checking

use thiserror::Error;

/// Result type for DSL operations
pub type ParseResult<T> = Result<T, ParseError>;

/// Main error type for all DSL operations
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Parse error: {message}")]
    PestError { message: String },

    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("Type error: {0}")]
    TypeError(#[from] TypeError),

    #[error("Missing match block in rule '{0}'")]
    MissingMatchBlock(String),

    #[error("Missing emit block in rule '{0}'")]
    MissingEmitBlock(String),

    #[error("Unknown fact type: {0}")]
    UnknownFactType(String),

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch { expected: String, found: String },

    #[error("Expected number, found {0}")]
    ExpectedNumber(String),

    #[error("Expected boolean, found {0}")]
    ExpectedBoolean(String),

    #[error("No such field '{field}' on type '{ty}'")]
    NoSuchField { ty: String, field: String },

    #[error("Cannot access field on type: {0}")]
    CannotAccessField(String),

    #[error("Unknown function: {0}")]
    UnknownFunction(String),

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
#[derive(Error, Debug, PartialEq)]
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
