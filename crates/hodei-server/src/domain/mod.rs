pub mod error;
/// Domain Layer - Core business logic independent of infrastructure
///
/// This layer contains:
/// - Domain models (Finding, Baseline, Analysis)
/// - Domain events
/// - Business rules and invariants
/// - Value objects
///
/// NO dependencies on external frameworks or databases
pub mod models;
pub mod ports;
