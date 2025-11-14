//! hodei-deep-analysis-engine - Deep Analysis Engine
//!
//! This crate provides the core functionality for deep code analysis including:
//! - Taint Analysis using datafrog Datalog engine
//! - Connascence Analysis for architectural coupling detection
//! - Semantic Model construction from AST
//! - Policy-based configuration
//!

#![warn(missing_docs)]

pub mod analysis_cache;
pub mod connascence;
pub mod policy;
pub mod semantic_model;
pub mod taint_analysis;

// Re-export commonly used types
pub use analysis_cache::{CouplingCache, SemanticModelCache, TaintFlowCache};
pub use connascence::{ConnascenceAnalyzer, CouplingFinding};
pub use semantic_model::{FactExtractor, SemanticModel, SemanticModelBuilder};
pub use taint_analysis::{TaintAnalysisError, TaintFlow, TaintPropagator};

/// Error types for the deep analysis engine
#[derive(Debug, thiserror::Error)]
pub enum DeepAnalysisError {
    #[error("Taint analysis error: {0}")]
    TaintAnalysis(#[from] TaintAnalysisError),

    #[error("Semantic model error: {0}")]
    SemanticModel(String),

    #[error("Connascence analysis error: {0}")]
    ConnascenceAnalysis(String),

    #[error("Policy error: {0}")]
    Policy(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Result type for deep analysis operations
pub type Result<T> = std::result::Result<T, DeepAnalysisError>;

/// Deep Analysis Engine - Main entry point
///
/// This struct orchestrates the various analysis components.
pub struct DeepAnalysisEngine {
    semantic_builder: SemanticModelBuilder,
    taint_propagator: TaintPropagator,
    connascence_analyzer: ConnascenceAnalyzer,
}

impl DeepAnalysisEngine {
    /// Create a new deep analysis engine
    pub fn new() -> Self {
        Self {
            semantic_builder: SemanticModelBuilder::new(),
            taint_propagator: TaintPropagator::new(),
            connascence_analyzer: ConnascenceAnalyzer::new(),
        }
    }

    /// Run complete analysis on source code
    pub async fn analyze_source(&mut self, source_path: &str) -> Result<AnalysisResult> {
        // Build semantic model
        let model = self.semantic_builder.from_source(source_path)?;

        // Run taint analysis
        // TODO: Load policy from file
        let taint_flows = self
            .taint_propagator
            .run_analysis(&model, &Default::default())?;

        // Run connascence analysis
        let couplings = self.connascence_analyzer.analyze(&model)?;

        Ok(AnalysisResult {
            taint_flows,
            coupling_findings: couplings,
        })
    }
}

impl Default for DeepAnalysisEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a complete deep analysis
pub struct AnalysisResult {
    /// Taint flows detected
    pub taint_flows: Vec<TaintFlow>,

    /// Coupling findings
    pub coupling_findings: Vec<CouplingFinding>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        // Test basic construction of components
        let _engine = DeepAnalysisEngine::new();
        // Components created successfully
        assert!(true);
    }
}
