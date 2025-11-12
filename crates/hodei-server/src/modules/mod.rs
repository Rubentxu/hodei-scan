/// Module exports for hodei-server
pub mod auth;
pub mod baseline;
pub mod config;
pub mod database;
pub mod diff;
pub mod error;
pub mod policies;
pub mod server;
pub mod types;
pub mod validation;
pub mod websocket;

pub use auth::AuthService;
pub use baseline::{
    BaselineAuditRecord, BaselineManager, BaselineRestoreSummary, BaselineStatusUpdate,
    BaselineUpdateSummary, BulkUpdateSummary,
};
pub use config::ServerConfig;
pub use database::DatabaseConnection;
pub use diff::{DiffEngine, DiffSummary};
pub use error::{Result, ServerError};
pub use policies::{create_analysis_summary, CleanupTask, RateLimiter, RetentionManager};
pub use server::HodeiServer;
pub use types::*;
pub use validation::{validate_project_exists, validate_publish_request, ValidationConfig};
pub use websocket::{DashboardEvent, WebSocketManager};
