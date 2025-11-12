//! hodei-extractors: Source code analyzers
//!
//! This crate provides extractors that analyze source code and populate
//! the intermediate representation (IR) with facts.
//!
//! # Architecture
//!
//! The extractor system follows a three-tier architecture:
//!
//! ## Tier 1: Adapters (Phase 1 - EPIC-14)
//! - Universal SARIF parser for any SARIF 2.1.0 compatible tool
//! - Tool-specific adapters (Ruff, ESLint, Clippy, staticcheck)
//! - Process orchestration with parallel execution and timeout handling
//!
//! ## Tier 2: Declarative (Phase 2 - EPIC-15)
//! - YAML-based DSL for custom rules
//! - Tree-sitter integration for pattern matching
//!
//! ## Tier 3: Deep Analysis (Phase 3 - EPIC-16)
//! - Taint analysis engine
//! - Control/data flow graph construction
//!
//! # Example
//!
//! ```no_run
//! use hodei_extractors::orchestrator::{ExtractorOrchestrator, OrchestratorConfig};
//! use hodei_extractors::core::ExtractorDefinition;
//! use std::path::Path;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = OrchestratorConfig::default();
//! let extractors = vec![
//!     ExtractorDefinition {
//!         id: "sarif-universal".to_string(),
//!         command: "hodei-extract-sarif".to_string(),
//!         enabled: true,
//!         timeout_seconds: 300,
//!         config: serde_json::json!({
//!             "sarif_files": ["results/**/*.sarif"]
//!         }),
//!     }
//! ];
//!
//! let orchestrator = ExtractorOrchestrator::new(config, extractors);
//! let result = orchestrator.run_all(Path::new("/path/to/project")).await?;
//!
//! println!("Extracted {} facts from {} extractors",
//!     result.facts.len(),
//!     result.metadata.extractor_runs.len());
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

pub mod core;
pub mod deduplication;
pub mod orchestrator;
pub mod sarif;

#[cfg(test)]
mod tests {
    mod orchestrator;
    mod sarif;
}

// Re-export commonly used types
pub use core::{
    Extractor, ExtractorConfig, ExtractorDefinition, ExtractorError, ExtractorMetadata,
    ExtractorRun, FileFilters, IRBuilder,
};
pub use deduplication::FactDeduplicator;
pub use orchestrator::{
    AggregatedIR, AggregationMetadata, ExtractorOrchestrator, OrchestratorConfig,
};
pub use sarif::{SarifConfig, SarifExtractor};
