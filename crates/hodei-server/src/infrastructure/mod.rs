/// Infrastructure Layer - Database adapters and external services
///
/// This layer implements the ports defined in the domain layer.
/// PostgreSQL is the primary database adapter.
pub mod database;
pub mod handlers;

pub use database::*;
