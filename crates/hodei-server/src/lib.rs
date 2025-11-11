/// hodei-server: Backend governance server for hodei-scan
pub mod modules;

pub use modules::{
    auth::AuthService,
    config::ServerConfig,
    database::DatabaseConnection,
    server::HodeiServer,
    types::*,
};

use modules::config::ConfigError;

/// Initialize tracing subscriber
pub fn init_tracing(debug: bool) {
    tracing_subscriber::fmt()
        .with_max_level(if debug {
            tracing::Level::DEBUG
        } else {
            tracing::Level::INFO
        })
        .init();
}

/// Load configuration from environment
pub fn load_config() -> Result<ServerConfig, ConfigError> {
    let config = ServerConfig::from_env();
    config.validate()?;
    Ok(config)
}
