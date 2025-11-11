/// Error handling for hodei-server
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

/// Custom error type for hodei-server
#[derive(Debug)]
pub enum ServerError {
    /// Database errors
    Database(sqlx::Error),
    /// Authentication errors
    Authentication(String),
    /// Validation errors
    Validation(String),
    /// Not found errors
    NotFound(String),
    /// Conflict errors
    Conflict(String),
    /// Rate limiting errors
    RateLimit(String),
    /// Configuration errors
    Config(String),
    /// JSON serialization/deserialization errors
    Serialization(serde_json::Error),
    /// JWT errors
    Jwt(String),
    /// Internal server errors
    Internal(String),
}

impl ServerError {
    /// Convert to HTTP status code
    pub fn status_code(&self) -> StatusCode {
        match self {
            ServerError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::Authentication(_) => StatusCode::UNAUTHORIZED,
            ServerError::Validation(_) => StatusCode::BAD_REQUEST,
            ServerError::NotFound(_) => StatusCode::NOT_FOUND,
            ServerError::Conflict(_) => StatusCode::CONFLICT,
            ServerError::RateLimit(_) => StatusCode::TOO_MANY_REQUESTS,
            ServerError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ServerError::Serialization(_) => StatusCode::BAD_REQUEST,
            ServerError::Jwt(_) => StatusCode::UNAUTHORIZED,
            ServerError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get error code for API response
    pub fn code(&self) -> &'static str {
        match self {
            ServerError::Database(_) => "DATABASE_ERROR",
            ServerError::Authentication(_) => "AUTHENTICATION_ERROR",
            ServerError::Validation(_) => "VALIDATION_ERROR",
            ServerError::NotFound(_) => "NOT_FOUND",
            ServerError::Conflict(_) => "CONFLICT_ERROR",
            ServerError::RateLimit(_) => "RATE_LIMIT_EXCEEDED",
            ServerError::Config(_) => "CONFIGURATION_ERROR",
            ServerError::Serialization(_) => "SERIALIZATION_ERROR",
            ServerError::Jwt(_) => "JWT_ERROR",
            ServerError::Internal(_) => "INTERNAL_ERROR",
        }
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerError::Database(e) => write!(f, "Database error: {}", e),
            ServerError::Authentication(e) => write!(f, "Authentication error: {}", e),
            ServerError::Validation(e) => write!(f, "Validation error: {}", e),
            ServerError::NotFound(e) => write!(f, "Not found: {}", e),
            ServerError::Conflict(e) => write!(f, "Conflict: {}", e),
            ServerError::RateLimit(e) => write!(f, "Rate limit: {}", e),
            ServerError::Config(e) => write!(f, "Configuration error: {}", e),
            ServerError::Serialization(e) => write!(f, "Serialization error: {}", e),
            ServerError::Jwt(e) => write!(f, "JWT error: {}", e),
            ServerError::Internal(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl std::error::Error for ServerError {}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        let body = Json(json!({
            "error": self.code(),
            "message": self.to_string(),
            "timestamp": chrono::Utc::now()
        }));

        (status_code, body).into_response()
    }
}

impl From<sqlx::Error> for ServerError {
    fn from(error: sqlx::Error) -> Self {
        ServerError::Database(error)
    }
}

impl From<serde_json::Error> for ServerError {
    fn from(error: serde_json::Error) -> Self {
        ServerError::Serialization(error)
    }
}

impl From<jsonwebtoken::errors::Error> for ServerError {
    fn from(error: jsonwebtoken::errors::Error) -> Self {
        ServerError::Jwt(error.to_string())
    }
}

impl From<anyhow::Error> for ServerError {
    fn from(error: anyhow::Error) -> Self {
        ServerError::Internal(error.to_string())
    }
}

/// Result type alias for hodei-server
pub type Result<T> = std::result::Result<T, ServerError>;
