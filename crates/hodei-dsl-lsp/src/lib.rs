//! hodei-dsl-lsp
//!
//! Language Server Protocol (LSP) implementation for hodei-scan DSL
//!
//! This crate provides intelligent editing features for hodei-scan rules:
//! - Autocompletion for fact types and fields
//! - Real-time semantic validation
//! - Hover documentation
//! - Error diagnostics

pub mod application;
pub mod domain;
pub mod infrastructure;

// Export main types
pub use infrastructure::adapters::{
    HodeiCompletionProvider, HodeiHoverProvider, HodeiSemanticAnalyzer, InMemoryDocumentRepository,
};

/// Start the LSP server using stdio transport
/// This is the standard entry point for LSP servers
pub async fn run() {
    infrastructure::server::start_stdio().await;
}
