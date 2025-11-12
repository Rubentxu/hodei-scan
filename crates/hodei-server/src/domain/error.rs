/// Domain errors - Business rule violations
///
/// These are pure domain errors that express business logic failures.
/// No infrastructure or framework-specific errors.
use thiserror::Error;

/// Result type for domain operations
pub type DomainResult<T> = Result<T, DomainError>;

/// Domain error types
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("Entity not found: {entity} with id {id}")]
    NotFound { entity: String, id: String },

    #[error("Business rule violation: {message}")]
    BusinessRuleViolation { message: String },

    #[error("Invalid input: {reason}")]
    InvalidInput { reason: String },

    #[error("Concurrent modification detected")]
    ConcurrencyError,

    #[error("Domain error: {message}")]
    Internal { message: String },
}

impl DomainError {
    pub fn not_found(entity: &str, id: &str) -> Self {
        Self::NotFound {
            entity: entity.to_string(),
            id: id.to_string(),
        }
    }

    pub fn business_rule(message: &str) -> Self {
        Self::BusinessRuleViolation {
            message: message.to_string(),
        }
    }

    pub fn invalid_input(reason: &str) -> Self {
        Self::InvalidInput {
            reason: reason.to_string(),
        }
    }

    pub fn internal(message: &str) -> Self {
        Self::Internal {
            message: message.to_string(),
        }
    }
}
