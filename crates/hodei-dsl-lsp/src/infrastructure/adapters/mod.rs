//! Adapters
//!
//! Implementations of domain ports for external systems

pub mod completion_provider;
pub mod diagnostic_emitter;
pub mod document_repository;
pub mod hover_provider;
pub mod semantic_analyzer;

// Re-exports
pub use completion_provider::HodeiCompletionProvider;
pub use document_repository::InMemoryDocumentRepository;
pub use hover_provider::HodeiHoverProvider;
pub use semantic_analyzer::HodeiSemanticAnalyzer;
