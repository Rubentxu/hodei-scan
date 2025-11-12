//! Infrastructure layer
//!
//! External integrations and adapters for LSP

pub mod adapters;
pub mod server;

pub use adapters::{
    HodeiCompletionProvider, HodeiHoverProvider, HodeiSemanticAnalyzer, InMemoryDocumentRepository,
};
