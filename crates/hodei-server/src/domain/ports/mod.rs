/// Domain ports - Interfaces that define what the domain needs
///
/// These are trait definitions that express the domain's needs
/// from external systems. Infrastructure will implement these ports.
pub mod repositories;
pub mod services;

pub use repositories::*;
pub use services::*;
