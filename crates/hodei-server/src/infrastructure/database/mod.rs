/// Infrastructure Layer - Database adapters
///
/// This layer implements the ports defined in the domain layer.
/// PostgreSQL is the primary database adapter.
pub mod postgres;

pub mod factory;

pub use factory::{create_analysis_repository, create_baseline_repository};
pub use postgres::{PostgresAnalysisRepository, PostgresBaselineRepository};
