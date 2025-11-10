use hodei_ir::{Confidence, IntermediateRepresentation, SourceLocation};
use std::collections::HashMap;

/// PatternMatcher performs pattern matching on facts
#[derive(Debug, Clone)]
pub struct PatternMatcher {
    store: crate::store::IndexedFactStore,
}

impl PatternMatcher {
    pub fn new(store: crate::store::IndexedFactStore) -> Self {
        Self { store }
    }

    /// Find all facts matching patterns
    pub fn match_patterns(&self, patterns: &[hodei_dsl::ast::Pattern]) -> Vec<hodei_ir::Fact> {
        let mut results = Vec::new();

        for pattern in patterns {
            let facts = self.match_single_pattern(pattern);
            results.extend(facts);
        }

        // Remove duplicates while preserving order
        results.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
        results.dedup_by(|a, b| a.id == b.id);

        results
    }

    fn match_single_pattern(&self, pattern: &hodei_dsl::ast::Pattern) -> Vec<hodei_ir::Fact> {
        // Simplified: match by fact_type string
        let fact_type_str = &pattern.fact_type;
        let matched_facts: Vec<_> = self.store.get_all_facts();

        matched_facts
            .into_iter()
            .filter(|fact| {
                // Check if fact type matches
                match &fact.fact_type {
                    hodei_ir::FactType::TaintSource { .. } => fact_type_str == "TaintSource",
                    hodei_ir::FactType::TaintSink { .. } => fact_type_str == "TaintSink",
                    hodei_ir::FactType::Sanitization { .. } => fact_type_str == "Sanitization",
                    hodei_ir::FactType::UnsafeCall { .. } => fact_type_str == "UnsafeCall",
                    hodei_ir::FactType::CryptographicOperation { .. } => {
                        fact_type_str == "CryptographicOperation"
                    }
                    hodei_ir::FactType::Vulnerability { .. } => fact_type_str == "Vulnerability",
                    hodei_ir::FactType::Function { .. } => fact_type_str == "Function",
                    hodei_ir::FactType::Variable { .. } => fact_type_str == "Variable",
                    hodei_ir::FactType::CodeSmell { .. } => fact_type_str == "CodeSmell",
                    hodei_ir::FactType::ComplexityViolation { .. } => {
                        fact_type_str == "ComplexityViolation"
                    }
                    hodei_ir::FactType::Dependency { .. } => fact_type_str == "Dependency",
                    hodei_ir::FactType::DependencyVulnerability { .. } => {
                        fact_type_str == "DependencyVulnerability"
                    }
                    hodei_ir::FactType::License { .. } => fact_type_str == "License",
                    hodei_ir::FactType::UncoveredLine { .. } => fact_type_str == "UncoveredLine",
                    hodei_ir::FactType::LowTestCoverage { .. } => {
                        fact_type_str == "LowTestCoverage"
                    }
                    hodei_ir::FactType::CoverageStats { .. } => fact_type_str == "CoverageStats",
                }
            })
            .collect()
    }
}

/// Expression evaluator for WHERE clauses
#[derive(Debug, Clone)]
pub struct ExprEvaluator {
    context: HashMap<String, String>,
}

impl ExprEvaluator {
    pub fn new() -> Self {
        Self {
            context: HashMap::new(),
        }
    }

    /// Evaluate an expression against the given facts and context
    pub fn eval_expr(
        &self,
        expr: &hodei_dsl::ast::Expr,
        facts: &[hodei_ir::Fact],
        source_location: &SourceLocation,
    ) -> bool {
        match expr {
            hodei_dsl::ast::Expr::Literal(literal) => match literal {
                hodei_dsl::ast::Literal::Boolean(b) => *b,
                hodei_dsl::ast::Literal::String(s) => s.parse::<bool>().unwrap_or(false),
                hodei_dsl::ast::Literal::Number(n) => *n > 0.0,
                hodei_dsl::ast::Literal::Null => false,
            },
            hodei_dsl::ast::Expr::Path(path) => {
                // Simple path evaluation
                self.context
                    .get(&path.segments.join("."))
                    .and_then(|v| v.parse::<bool>().ok())
                    .unwrap_or(false)
            }
            hodei_dsl::ast::Expr::FunctionCall { name, args } => {
                // Simple function call handling
                match name.as_str() {
                    "count" => args.len() > 0,
                    "exists" => !args.is_empty(),
                    _ => false,
                }
            }
            hodei_dsl::ast::Expr::Binary { left, op, right } => {
                let left_val = self.eval_expr(left, facts, source_location);
                let right_val = self.eval_expr(right, facts, source_location);

                match op {
                    hodei_dsl::BinaryOp::And => left_val && right_val,
                    hodei_dsl::BinaryOp::Or => left_val || right_val,
                    hodei_dsl::BinaryOp::Eq => left_val == right_val,
                    hodei_dsl::BinaryOp::Ne => left_val != right_val,
                    hodei_dsl::BinaryOp::Gt => false,
                    hodei_dsl::BinaryOp::Lt => false,
                    hodei_dsl::BinaryOp::Ge => false,
                    hodei_dsl::BinaryOp::Le => false,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodei_dsl::ast::*;
    use hodei_ir::*;
    use std::time::SystemTime;

    fn create_test_ir() -> IntermediateRepresentation {
        let metadata = ProjectMetadata::new(
            "test".to_string(),
            "1.0".to_string(),
            ProjectPath::new(std::path::PathBuf::from(".")),
        );
        IntermediateRepresentation::new(metadata)
    }

    #[test]
    fn test_pattern_matcher_basic() {
        let ir = create_test_ir();
        let store = crate::store::IndexedFactStore::new(ir.facts);
        let matcher = PatternMatcher::new(store);

        let patterns = vec![Pattern {
            binding: "test".to_string(),
            fact_type: "Vulnerability".to_string(),
            conditions: vec![],
            span: hodei_dsl::ast::Span { start: 0, end: 0 },
        }];

        let results = matcher.match_patterns(&patterns);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_expr_evaluator_literal() {
        let evaluator = ExprEvaluator::new();

        let expr = Expr::Literal(Literal::Boolean(true));
        let result = evaluator.eval_expr(&expr, &[], &SourceLocation::default());
        assert!(result);
    }

    #[test]
    fn test_expr_evaluator_binary_and() {
        let evaluator = ExprEvaluator::new();

        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Boolean(true))),
            op: BinaryOp::And,
            right: Box::new(Expr::Literal(Literal::Boolean(true))),
        };
        let result = evaluator.eval_expr(&expr, &[], &SourceLocation::default());
        assert!(result);
    }
}
