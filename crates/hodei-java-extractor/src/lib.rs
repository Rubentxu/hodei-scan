//! hodei-java-extractor
//!
//! Java Extractor following Hexagonal Architecture with TDD methodology
//!
//! ## Architecture Layers (Hexagonal)
//!
//! - **Domain Layer** (`domain/`): Pure business logic, entities, domain services
//! - **Application Layer** (`application/`): Use cases and application services
//! - **Infrastructure Layer** (`infrastructure/`): External adapters (Spoon, tree-sitter, JaCoCo)
//!
//! ## Extraction Levels (EPIC-22)
//!
//! - **Level 1**: Coverage analysis using JaCoCo adapter
//! - **Level 2**: Pattern matching using tree-sitter-java
//! - **Level 3**: Deep semantic analysis using Spoon + hodei-deep-analysis-engine
//!
//! # TDD Methodology
//!
//! This crate follows strict Test-Driven Development:
//! 1. Write failing tests first (RED)
//! 2. Implement minimal code to pass tests (GREEN)
//! 3. Refactor while keeping tests green (REFACTOR)

#![warn(missing_docs)]

pub mod application;
pub mod domain;
pub mod infrastructure;

pub use domain::entities::{
    ConnascenceFinding, ConnascenceType, CoverageData, DomainError, DomainResult, ExtractionLevel,
    JavaAnalysisConfig, JavaAnalysisResult, JavaAnalysisService, JavaClass, JavaMethod,
    JavaPackage, JavaSourceId, JavaSourceRepository, Strength, TaintSink, TaintSource,
};

pub use application::use_cases::{
    AnalyzeJavaCodeRequest, AnalyzeJavaCodeResponse, AnalyzeJavaCodeUseCase,
};

pub use infrastructure::adapters::{
    JacocoAdapter, JavaAnalysisServiceImpl, SpoonService, TreeSitterAdapter,
};
