//! Infrastructure Layer (Hexagonal Architecture)
//!
//! This layer contains adapters for external tools and systems:
//! - JaCoCo adapter for coverage analysis
//! - tree-sitter adapter for pattern matching
//! - Spoon service for semantic analysis
//! - CLI adapter for end-user interaction

pub mod adapters;
pub mod cli;

pub use adapters::{
    JavaAnalysisServiceImpl, jacoco::JacocoAdapter, spoon::SpoonService,
    tree_sitter::TreeSitterAdapter,
};
