//! Error types for declarative extractors

use thiserror::Error;

/// Result type with custom error
pub type Result<T> = std::result::Result<T, DeclarativeExtractorError>;

/// Main error type for declarative extractors
#[derive(Error, Debug)]
pub enum DeclarativeExtractorError {
    #[error("Parse error: {message}")]
    Parse {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("Validation error: {message}")]
    Validation {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("IO error: {message}")]
    Io {
        message: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Language not supported: {language}")]
    LanguageNotSupported { language: String },

    #[error("Rule error: {message}")]
    Rule {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("Matcher error: {message}")]
    Matcher {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },

    #[error("Tree-sitter error: {message}")]
    TreeSitter {
        message: String,
        #[source]
        source: Option<anyhow::Error>,
    },
}

impl DeclarativeExtractorError {
    /// Create a parse error
    pub fn parse<T: std::fmt::Display>(msg: T) -> Self {
        Self::Parse {
            message: msg.to_string(),
            source: None,
        }
    }

    /// Create a validation error
    pub fn validation<T: std::fmt::Display>(msg: T) -> Self {
        Self::Validation {
            message: msg.to_string(),
            source: None,
        }
    }

    /// Create a language not supported error
    pub fn language_not_supported<T: std::fmt::Display>(lang: T) -> Self {
        Self::LanguageNotSupported {
            language: lang.to_string(),
        }
    }
}
