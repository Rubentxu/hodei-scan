//! Language Server Protocol implementation for Hodei DSL
//!
//! This module provides LSP server capabilities for editing Hodei rule files,
//! including code completion, diagnostics, hover help, and document navigation.

pub mod handlers;
pub mod server;
pub mod utils;

pub use handlers::{
    CodeActionHandler, CompletionHandler, DiagnosticHandler, HoverHandler, SymbolHandler,
    TemplateHandler,
};
pub use server::HodeiLspServer;
