//! Domain Layer (Hexagonal Architecture)
//!
//! This layer contains pure business logic with no external dependencies.
//! It defines the domain entities, value objects, and domain services.

pub mod entities;
pub mod repositories;
pub mod services;

pub use entities::{
    ConnascenceFinding, ConnascenceType, CoverageData, DomainError, DomainResult, ExtractionLevel,
    JavaAnalysisConfig, JavaAnalysisResult, JavaAnalysisService, JavaClass, JavaMethod,
    JavaPackage, JavaSourceId, JavaSourceRepository, Strength, TaintSink, TaintSource,
};
