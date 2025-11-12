/// Server configuration management
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// Main server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server bind address
    pub bind_address: SocketAddr,
    /// Database connection string
    pub database_url: String,
    /// JWT secret for authentication
    pub jwt_secret: String,
    /// JWT expiration in hours
    pub jwt_expiration_hours: u64,
    /// Maximum request size in bytes
    pub max_request_size: usize,
    /// Database connection pool size
    pub db_pool_size: u32,
    /// Rate limiting: requests per minute
    pub rate_limit_rpm: u64,
    /// Enable debug logging
    pub debug: bool,
    /// CORS allowed origins
    pub cors_origins: Vec<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            bind_address: "0.0.0.0:8080".parse().unwrap(),
            database_url: "postgres://hodei:password@localhost:5432/hodei_db".to_string(),
            jwt_secret: "your-secret-key-change-in-production".to_string(),
            jwt_expiration_hours: 24,
            max_request_size: 10 * 1024 * 1024, // 10MB
            db_pool_size: 10,
            rate_limit_rpm: 1000,
            debug: false,
            cors_origins: vec!["http://localhost:3000".to_string()],
        }
    }
}

impl ServerConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            bind_address: std::env::var("HODEI_BIND_ADDRESS")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string())
                .parse()
                .unwrap_or_default(),
            database_url: std::env::var("HODEI_DATABASE_URL").unwrap_or_else(|_| {
                "postgres://hodei:password@localhost:5432/hodei_db".to_string()
            }),
            jwt_secret: std::env::var("HODEI_JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key-change-in-production".to_string()),
            jwt_expiration_hours: std::env::var("HODEI_JWT_EXPIRATION")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .unwrap_or(24),
            max_request_size: std::env::var("HODEI_MAX_REQUEST_SIZE")
                .unwrap_or_else(|_| "10485760".to_string()) // 10MB
                .parse()
                .unwrap_or(10 * 1024 * 1024),
            db_pool_size: std::env::var("HODEI_DB_POOL_SIZE")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
            rate_limit_rpm: std::env::var("HODEI_RATE_LIMIT_RPM")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            debug: std::env::var("HODEI_DEBUG")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            cors_origins: std::env::var("HODEI_CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.database_url.is_empty() {
            return Err(ConfigError::MissingField("database_url".to_string()));
        }
        if self.jwt_secret.len() < 32 {
            return Err(ConfigError::InvalidValue(
                "jwt_secret".to_string(),
                "must be at least 32 characters".to_string(),
            ));
        }
        if self.db_pool_size == 0 {
            return Err(ConfigError::InvalidValue(
                "db_pool_size".to_string(),
                "must be greater than 0".to_string(),
            ));
        }
        Ok(())
    }
}

/// Configuration error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid value for {0}: {1}")]
    InvalidValue(String, String),
    #[error("Failed to parse configuration: {0}")]
    ParseError(#[from] std::net::AddrParseError),
}
