//! LSP server implementation for Hodei DSL

use lsp_types::*;

/// Main LSP server for Hodei DSL
#[derive(Debug, Clone)]
pub struct HodeiLspServer {
    capabilities: ClientCapabilities,
}

impl HodeiLspServer {
    pub fn new() -> Self {
        Self {
            capabilities: ClientCapabilities::default(),
        }
    }

    pub fn get_server_capabilities(&self) -> ServerCapabilities {
        ServerCapabilities::default()
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

impl Default for HodeiLspServer {
    fn default() -> Self {
        Self::new()
    }
}
