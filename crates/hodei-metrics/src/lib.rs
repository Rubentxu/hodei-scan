//! hodei-metrics: Metrics and analytics
//!
//! This crate provides metrics collection and analytics capabilities.

use hodei_ir::{Fact, IntermediateRepresentation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metrics for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Total number of facts
    pub fact_count: usize,

    /// Facts by type
    pub facts_by_type: HashMap<String, usize>,

    /// Facts by severity
    pub facts_by_severity: HashMap<String, usize>,

    /// Quality score (0-100)
    pub quality_score: f64,
}

/// Compute metrics from IR
pub fn compute_metrics(ir: &IntermediateRepresentation) -> Metrics {
    let mut facts_by_type = HashMap::new();
    let mut facts_by_severity = HashMap::new();

    for fact in &ir.facts {
        // Count by type
        let type_name = match &fact.fact_type {
            hodei_ir::FactType::TaintSource { .. } => "TaintSource",
            hodei_ir::FactType::TaintSink { .. } => "TaintSink",
            hodei_ir::FactType::Vulnerability { .. } => "Vulnerability",
            hodei_ir::FactType::CodeSmell { .. } => "CodeSmell",
            hodei_ir::FactType::Function { .. } => "Function",
            _ => "Other",
        };
        *facts_by_type.entry(type_name.to_string()).or_insert(0) += 1;

        // Count by severity
        let severity = match &fact.fact_type {
            hodei_ir::FactType::Vulnerability { severity, .. } => match severity {
                hodei_ir::Severity::Critical => "Critical",
                hodei_ir::Severity::Major => "Major",
                hodei_ir::Severity::Minor => "Minor",
                hodei_ir::Severity::Info => "Info",
                hodei_ir::Severity::Blocker => "Blocker",
            },
            hodei_ir::FactType::CodeSmell { severity, .. } => match severity {
                hodei_ir::Severity::Critical => "Critical",
                hodei_ir::Severity::Major => "Major",
                hodei_ir::Severity::Minor => "Minor",
                hodei_ir::Severity::Info => "Info",
                hodei_ir::Severity::Blocker => "Blocker",
            },
            _ => "N/A",
        };
        *facts_by_severity.entry(severity.to_string()).or_insert(0) += 1;
    }

    // Compute quality score
    let mut score = 100.0;
    if let Some(&critical_count) = facts_by_severity.get("Critical") {
        score -= critical_count as f64 * 10.0;
    }
    if let Some(&major_count) = facts_by_severity.get("Major") {
        score -= major_count as f64 * 5.0;
    }
    if let Some(&minor_count) = facts_by_severity.get("Minor") {
        score -= minor_count as f64 * 1.0;
    }

    Metrics {
        fact_count: ir.fact_count(),
        facts_by_type,
        facts_by_severity,
        quality_score: score.max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_metrics() {
        let metadata = hodei_ir::ProjectMetadata::new(
            "test".to_string(),
            "1.0".to_string(),
            hodei_ir::ProjectPath::new(std::path::PathBuf::from(".")),
        );
        let ir = hodei_ir::IntermediateRepresentation::new(metadata);

        let metrics = compute_metrics(&ir);
        assert_eq!(metrics.fact_count, 0);
        assert!(metrics.quality_score >= 0.0);
    }
}
