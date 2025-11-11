//! Extractor Orchestrator - Multi-process architecture for external tools
//!
//! This module implements the core infrastructure for running external extractors
//! as separate processes, enabling hodei-scan to integrate with any tool without
//! recompiling the core.

pub mod error;
pub mod orchestrator;
pub mod protocol;
pub mod sarif_adapter;
pub mod sarif_extractor;

// Re-export public types
pub use error::{OrchestratorError, Result};
pub use orchestrator::{ExtractorOrchestrator, ResourceStats};
pub use protocol::{
    AggregatedIR, ErrorResponse, ExtractorConfig, ExtractorDef, ExtractorMessage, ExtractorRequest,
    ExtractorResponse, Heartbeat,
};
pub use sarif_adapter::{SarifAdapter, SarifConfig, SarifError};
pub use sarif_extractor::SarifExtractor;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_module_compiles() {
        // Smoke test to ensure module compiles
        assert!(true);
    }
}
