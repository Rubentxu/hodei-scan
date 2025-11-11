//! Extractor Orchestrator - Multi-process architecture for external tools
//!
//! This module implements the core infrastructure for running external extractors
//! as separate processes, enabling hodei-scan to integrate with any tool without
//! recompiling the core.

pub mod error;
pub mod orchestrator;
pub mod protocol;

// Re-export public types
pub use error::{OrchestratorError, Result};
pub use orchestrator::ExtractorOrchestrator;
pub use protocol::{
    AggregatedIR, ExtractorConfig, ExtractorDef, ExtractorRequest, ExtractorResponse,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_module_compiles() {
        // Smoke test to ensure module compiles
        assert!(true);
    }
}
