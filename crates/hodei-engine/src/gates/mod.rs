//! Quality gates for enforcing quality thresholds
//!
//! This module provides quality gates that evaluate IR facts against
//! defined thresholds and can block deployments or flag violations.

use hodei_ir::{Fact, IntermediateRepresentation, Severity};
use std::collections::HashMap;
use thiserror::Error;

/// Quality gate errors
#[derive(Error, Debug)]
pub enum GateError {
    #[error("Quality gate violation: {0}")]
    Violation(String),

    #[error("Gate evaluation failed: {0}")]
    EvaluationFailed(String),
}

/// A quality gate definition
#[derive(Debug, Clone)]
pub struct QualityGate {
    /// Gate name
    pub name: String,

    /// Metric to evaluate
    pub metric: QualityMetric,

    /// Threshold value
    pub threshold: f64,

    /// Comparison operator
    pub operator: ComparisonOp,

    /// Action to take on violation
    pub action: GateAction,
}

/// Quality metrics that can be checked
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QualityMetric {
    /// Total number of facts
    FactCount,

    /// Number of high/critical severity issues
    CriticalIssueCount,

    /// Number of code smells
    CodeSmellCount,

    /// Number of vulnerabilities
    VulnerabilityCount,

    /// Test coverage percentage
    TestCoverage,

    /// Custom metric from metadata
    Custom(String),
}

/// Comparison operators for thresholds
#[derive(Debug, Clone, Copy)]
pub enum ComparisonOp {
    LessThan,
    LessThanOrEqual,
    Equal,
    GreaterThan,
    GreaterThanOrEqual,
}

/// Actions when gate fails
#[derive(Debug, Clone, Copy)]
pub enum GateAction {
    /// Log warning only
    Warn,

    /// Block the operation
    Block,

    /// Mark as review required
    Review,
}

/// Quality gate evaluator
pub struct GateEvaluator {
    gates: Vec<QualityGate>,
}

impl GateEvaluator {
    /// Create evaluator with given gates
    pub fn new(gates: Vec<QualityGate>) -> Self {
        Self { gates }
    }

    /// Evaluate all gates against IR
    pub fn evaluate(&self, ir: &IntermediateRepresentation) -> Result<(), GateError> {
        for gate in &self.gates {
            self.evaluate_gate(gate, ir)?;
        }
        Ok(())
    }

    /// Evaluate a single gate
    fn evaluate_gate(
        &self,
        gate: &QualityGate,
        ir: &IntermediateRepresentation,
    ) -> Result<(), GateError> {
        let value = self.compute_metric(&gate.metric, ir);

        let violated = match gate.operator {
            ComparisonOp::LessThan => value < gate.threshold,
            ComparisonOp::LessThanOrEqual => value <= gate.threshold,
            ComparisonOp::Equal => (value - gate.threshold).abs() > f64::EPSILON,
            ComparisonOp::GreaterThan => value > gate.threshold,
            ComparisonOp::GreaterThanOrEqual => value >= gate.threshold,
        };

        if violated {
            match gate.action {
                GateAction::Warn => {
                    eprintln!(
                        "⚠️  Quality gate '{}' violated: {} = {} (threshold: {})",
                        gate.name,
                        gate.metric.to_string(),
                        value,
                        gate.threshold
                    );
                }
                GateAction::Block => {
                    return Err(GateError::Violation(format!(
                        "Quality gate '{}' blocked: {} = {} (threshold: {})",
                        gate.name,
                        gate.metric.to_string(),
                        value,
                        gate.threshold
                    )));
                }
                GateAction::Review => {
                    println!(
                        "🔍 Quality gate '{}' requires review: {} = {} (threshold: {})",
                        gate.name,
                        gate.metric.to_string(),
                        value,
                        gate.threshold
                    );
                }
            }
        }

        Ok(())
    }

    /// Compute a metric value
    fn compute_metric(&self, metric: &QualityMetric, ir: &IntermediateRepresentation) -> f64 {
        match metric {
            QualityMetric::FactCount => ir.fact_count() as f64,
            QualityMetric::CriticalIssueCount => ir
                .facts
                .iter()
                .filter(|f| {
                    matches!(
                        f.fact_type,
                        hodei_ir::FactType::Vulnerability {
                            severity: Severity::Critical,
                            ..
                        }
                    )
                })
                .count() as f64,
            QualityMetric::CodeSmellCount => ir
                .facts
                .iter()
                .filter(|f| matches!(f.fact_type, hodei_ir::FactType::CodeSmell { .. }))
                .count() as f64,
            QualityMetric::VulnerabilityCount => ir
                .facts
                .iter()
                .filter(|f| matches!(f.fact_type, hodei_ir::FactType::Vulnerability { .. }))
                .count() as f64,
            QualityMetric::TestCoverage => {
                // For now, return 0.0 - would be calculated from coverage facts
                0.0
            }
            QualityMetric::Custom(_) => 0.0,
        }
    }
}

/// Default quality gates for common use cases
pub fn default_gates() -> Vec<QualityGate> {
    vec![
        // No critical vulnerabilities allowed
        QualityGate {
            name: "No Critical Vulnerabilities".to_string(),
            metric: QualityMetric::CriticalIssueCount,
            threshold: 0.0,
            operator: ComparisonOp::GreaterThan,
            action: GateAction::Block,
        },
        // Limit code smells
        QualityGate {
            name: "Max Code Smells".to_string(),
            metric: QualityMetric::CodeSmellCount,
            threshold: 100.0,
            operator: ComparisonOp::GreaterThan,
            action: GateAction::Review,
        },
        // Require minimum test coverage (when available)
        QualityGate {
            name: "Test Coverage".to_string(),
            metric: QualityMetric::TestCoverage,
            threshold: 80.0,
            operator: ComparisonOp::LessThan,
            action: GateAction::Warn,
        },
    ]
}

impl QualityMetric {
    fn to_string(&self) -> String {
        match self {
            QualityMetric::FactCount => "Fact Count".to_string(),
            QualityMetric::CriticalIssueCount => "Critical Issues".to_string(),
            QualityMetric::CodeSmellCount => "Code Smells".to_string(),
            QualityMetric::VulnerabilityCount => "Vulnerabilities".to_string(),
            QualityMetric::TestCoverage => "Test Coverage".to_string(),
            QualityMetric::Custom(s) => s.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodei_ir::*;

    #[test]
    fn test_gate_evaluator() {
        let gates = vec![QualityGate {
            name: "No Facts".to_string(),
            metric: QualityMetric::FactCount,
            threshold: 0.0,
            operator: ComparisonOp::LessThan,
            action: GateAction::Block,
        }];

        let evaluator = GateEvaluator::new(gates);

        let metadata = ProjectMetadata::new(
            "test".to_string(),
            "1.0".to_string(),
            ProjectPath::new(std::path::PathBuf::from(".")),
        );
        let ir = IntermediateRepresentation::new(metadata);

        // Should pass - no facts
        assert!(evaluator.evaluate(&ir).is_ok());
    }

    #[test]
    fn test_gate_block_action() {
        let gates = vec![QualityGate {
            name: "Block All".to_string(),
            metric: QualityMetric::FactCount,
            threshold: 0.0,
            operator: ComparisonOp::GreaterThan,
            action: GateAction::Block,
        }];

        let evaluator = GateEvaluator::new(gates);

        // Add a fact
        let mut ir = IntermediateRepresentation::new(ProjectMetadata::new(
            "test".to_string(),
            "1.0".to_string(),
            ProjectPath::new(std::path::PathBuf::from(".")),
        ));

        // Verify fact count before adding
        assert_eq!(ir.fact_count(), 0);

        ir.add_fact(Fact::new(
            FactType::CodeSmell {
                smell_type: "test".to_string(),
                severity: Severity::Minor,
                message: "test".to_string(),
            },
            SourceLocation::default(),
            Provenance::new(ExtractorId::Custom, "1.0".to_string(), Confidence::MEDIUM),
        ));

        // Verify fact count after adding
        assert_eq!(ir.fact_count(), 1);

        // Should fail - fact count > 0
        assert!(evaluator.evaluate(&ir).is_err());
    }
}
