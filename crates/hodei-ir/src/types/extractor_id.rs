//! Extractor identifier for provenance tracking

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExtractorId {
    TreeSitter,
    OxcParser,
    SemgrepTaint,
    DataFlowAnalyzer,
    SymbolicExecutor,
    CargoAudit,
    NpmAudit,
    TrivyScanner,
    JaCoCoParser,
    LcovParser,
    CoberturaParser,
    Custom,
}

impl ExtractorId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::TreeSitter => "tree-sitter",
            Self::OxcParser => "oxc",
            Self::SemgrepTaint => "semgrep-taint",
            Self::DataFlowAnalyzer => "dataflow",
            Self::SymbolicExecutor => "symbolic",
            Self::CargoAudit => "cargo-audit",
            Self::NpmAudit => "npm-audit",
            Self::TrivyScanner => "trivy",
            Self::JaCoCoParser => "jacoco",
            Self::LcovParser => "lcov",
            Self::CoberturaParser => "cobertura",
            Self::Custom => "custom",
        }
    }
}

impl fmt::Display for ExtractorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
