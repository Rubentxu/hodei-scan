//! hodei-server - Backend governance server with hexagonal architecture
//!
//! This crate implements a hexagonal architecture with clear separation of concerns:
//!
//! - Domain layer: Pure business logic
//! - Application layer: Use cases and services
//! - Infrastructure layer: Database adapters and external services
//!
//! # Architecture
//!
//! The server follows the hexagonal (ports & adapters) architecture pattern:
//!
//! ```text
//! +------------------+
//! |  REST API Layer  |
//! |   (handlers)     |
//! +--------+---------+
//!          |
//! +--------v----------+
//! | Application      |  <-- Use cases (business workflows)
//! |   Layer          |
//! +--------+---------+
//!          |
//! +--------v----------+
//! | Domain Layer     |  <-- Core business logic (domain models)
//! |                  |
//! +--------+---------+
//!          |
//! +--------v----------+
//! | Ports (Traits)   |  <-- Interfaces the domain needs
//! +--------+---------+
//!          |
//! +--------v----------+
//! | Infrastructure   |  <-- Database adapters (PostgreSQL)
//! |   Layer          |
//! +------------------+
//! ```

/// Domain Layer - Pure business logic
pub mod domain;

/// Application Layer - Use cases and services
pub mod application;

/// Infrastructure Layer - Database adapters and external services
pub mod infrastructure;

/// Module re-exports - Public API for tests and external usage
pub mod modules;

pub use domain::error::*;
/// Re-export commonly used types
pub use domain::models::*;

/// Convenience re-exports for repositories
pub use infrastructure::database::factory;

/// The server application
pub mod server {
    use axum::Router;

    /// Create the server application with default configuration
    pub fn create_app() -> Router {
        // This is a placeholder - would implement the actual server setup
        Router::new()
    }
}
