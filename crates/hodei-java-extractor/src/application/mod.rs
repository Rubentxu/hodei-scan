//! Application Layer (Hexagonal Architecture)
//!
//! This layer contains use cases, application services, and DTOs.
//! It orchestrates domain objects and coordinates workflows.

pub mod dtos;
pub mod services;
pub mod use_cases;

pub use use_cases::{AnalyzeJavaCodeRequest, AnalyzeJavaCodeResponse, AnalyzeJavaCodeUseCase};
