//! Adapters for External Tools
//!
//! These adapters implement the domain ports using external tools:
//! - JaCoCo for coverage data
//! - tree-sitter for pattern matching
//! - Spoon for semantic analysis

pub mod jacoco;
pub mod spoon;
pub mod tree_sitter;

pub use jacoco::JacocoAdapter;
pub use java_analysis_service::JavaAnalysisServiceImpl;
pub use spoon::SpoonService;
pub use tree_sitter::TreeSitterAdapter;

mod java_analysis_service;
