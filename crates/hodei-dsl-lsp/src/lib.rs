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

/// LSP Server entry point
pub use infrastructure::{
    HodeiCompletionProvider, HodeiHoverProvider, HodeiSemanticAnalyzer, InMemoryDocumentRepository,
    server::HodeiDslServer,
};
