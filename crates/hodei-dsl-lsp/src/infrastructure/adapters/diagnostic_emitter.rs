//! Diagnostic emitter adapter
//!
//! Emits diagnostics (errors, warnings) to the LSP client

use crate::domain::models::Diagnostic;
use crate::domain::ports::DiagnosticEmitter;

/// In-memory diagnostic emitter
pub struct InMemoryDiagnosticEmitter {
    diagnostics: Vec<Diagnostic>,
}

impl InMemoryDiagnosticEmitter {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }
    
    pub fn get_diagnostics(&self) -> &Vec<Diagnostic> {
        &self.diagnostics
    }
    
    pub fn clear(&mut self) {
        self.diagnostics.clear();
    }
}

#[async_trait::async_trait]
impl DiagnosticEmitter for InMemoryDiagnosticEmitter {
    async fn emit(&self, diagnostic: Diagnostic) {
        // In a real implementation, this would send diagnostics to the LSP client
        // via the notification protocol
        eprintln!("[LSP Diagnostic] {:?}", diagnostic);
    }
}
