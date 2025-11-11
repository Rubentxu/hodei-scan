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
    pub fn match_patterns(
        &self,
        patterns: &[hodei_dsl::ast::Pattern],
    ) -> Result<Vec<hodei_ir::Fact>, String> {
        let mut results = Vec::new();

        for pattern in patterns {
            let facts = self.match_single_pattern(pattern);
            results.extend(facts);
        }

        // Remove duplicates while preserving order
        results.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
        results.dedup_by(|a, b| a.id == b.id);

        Ok(results)
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
    store: crate::store::IndexedFactStore,
}

impl ExprEvaluator {
    pub fn new(store: crate::store::IndexedFactStore) -> Self {
        Self { store }
    }

    /// Evaluate an expression against the given facts and context
    pub fn eval_expr(
        &self,
        expr: &hodei_dsl::ast::Expr,
        facts: &[hodei_ir::Fact],
        source_location: &SourceLocation,
    ) -> Result<bool, String> {
        match expr {
            hodei_dsl::ast::Expr::Literal(literal) => match literal {
                hodei_dsl::ast::Literal::Boolean(b) => Ok(*b),
                hodei_dsl::ast::Literal::String(s) => Ok(s.parse::<bool>().unwrap_or(false)),
                hodei_dsl::ast::Literal::Number(n) => Ok(*n > 0.0),
                hodei_dsl::ast::Literal::Null => Ok(false),
            },
            hodei_dsl::ast::Expr::Path(path) => {
                // Path evaluation: look for variables in facts
                self.eval_path(path, facts)
            }
            hodei_dsl::ast::Expr::FunctionCall { name, args } => {
                self.eval_function_call(name, args, facts)
            }
            hodei_dsl::ast::Expr::Binary { left, op, right } => {
                self.eval_binary_op(op, left, right, facts, source_location)
            }
        }
    }

    /// Evaluate a path expression
    fn eval_path(
        &self,
        path: &hodei_dsl::ast::Path,
        facts: &[hodei_ir::Fact],
    ) -> Result<bool, String> {
        // Check if it's a variable reference in the current facts
        let var_name = &path.segments[0];

        // Look for a fact with this binding
        let has_fact = facts.iter().any(|fact| {
            // Simplified: check if fact type name matches or location matches
            let discriminant_str = format!("{:?}", fact.fact_type.discriminant());
            discriminant_str == *var_name
        });

        Ok(has_fact)
    }

    /// Evaluate built-in functions
    fn eval_function_call(
        &self,
        name: &str,
        args: &[hodei_dsl::ast::Expr],
        facts: &[hodei_ir::Fact],
    ) -> Result<bool, String> {
        match name {
            "count" => {
                // count() returns true if there are facts
                Ok(!facts.is_empty())
            }
            "exists" => {
                // exists() checks if any fact satisfies a condition
                Ok(!facts.is_empty())
            }
            "reachable" => {
                // reachable(source_fact, sink_fact) - simplified implementation
                if args.len() != 2 {
                    return Err("reachable() requires 2 arguments".to_string());
                }

                // Simplified: just check if we have both source and sink facts
                let has_source = facts
                    .iter()
                    .any(|f| matches!(f.fact_type, hodei_ir::FactType::TaintSource { .. }));
                let has_sink = facts
                    .iter()
                    .any(|f| matches!(f.fact_type, hodei_ir::FactType::TaintSink { .. }));

                Ok(has_source && has_sink)
            }
            _ => Err(format!("Unknown function: {}", name)),
        }
    }

    /// Evaluate binary operations
    fn eval_binary_op(
        &self,
        op: &hodei_dsl::BinaryOp,
        left: &hodei_dsl::ast::Expr,
        right: &hodei_dsl::ast::Expr,
        facts: &[hodei_ir::Fact],
        source_location: &SourceLocation,
    ) -> Result<bool, String> {
        match op {
            hodei_dsl::BinaryOp::And => {
                // Short-circuit evaluation
                let left_val = self.eval_expr(left, facts, source_location)?;
                if !left_val {
                    return Ok(false);
                }
                self.eval_expr(right, facts, source_location)
            }
            hodei_dsl::BinaryOp::Or => {
                // Short-circuit evaluation
                let left_val = self.eval_expr(left, facts, source_location)?;
                if left_val {
                    return Ok(true);
                }
                self.eval_expr(right, facts, source_location)
            }
            hodei_dsl::BinaryOp::Eq
            | hodei_dsl::BinaryOp::Ne
            | hodei_dsl::BinaryOp::Gt
            | hodei_dsl::BinaryOp::Lt
            | hodei_dsl::BinaryOp::Ge
            | hodei_dsl::BinaryOp::Le => {
                // For comparison operations, evaluate both sides
                let left_val = self.eval_value_expr(left, facts, source_location)?;
                let right_val = self.eval_value_expr(right, facts, source_location)?;

                match op {
                    hodei_dsl::BinaryOp::Eq => Ok(left_val == right_val),
                    hodei_dsl::BinaryOp::Ne => Ok(left_val != right_val),
                    hodei_dsl::BinaryOp::Gt => Ok(left_val > right_val),
                    hodei_dsl::BinaryOp::Lt => Ok(left_val < right_val),
                    hodei_dsl::BinaryOp::Ge => Ok(left_val >= right_val),
                    hodei_dsl::BinaryOp::Le => Ok(left_val <= right_val),
                    _ => Err("Invalid comparison operator".to_string()),
                }
            }
        }
    }

    /// Evaluate an expression that returns a value (for comparisons)
    fn eval_value_expr(
        &self,
        expr: &hodei_dsl::ast::Expr,
        facts: &[hodei_ir::Fact],
        source_location: &SourceLocation,
    ) -> Result<String, String> {
        match expr {
            hodei_dsl::ast::Expr::Literal(lit) => Ok(format!("{:?}", lit)),
            hodei_dsl::ast::Expr::Path(path) => {
                // Simplified: return the variable name
                Ok(path.segments[0].clone())
            }
            _ => Err("Cannot convert to value for comparison".to_string()),
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

        let results = matcher.match_patterns(&patterns).unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_expr_evaluator_literal() {
        let store = crate::store::IndexedFactStore::new(vec![]);
        let evaluator = ExprEvaluator::new(store);

        let expr = Expr::Literal(Literal::Boolean(true));
        let result = evaluator
            .eval_expr(&expr, &[], &SourceLocation::default())
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_expr_evaluator_binary_and() {
        let store = crate::store::IndexedFactStore::new(vec![]);
        let evaluator = ExprEvaluator::new(store);

        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Boolean(true))),
            op: BinaryOp::And,
            right: Box::new(Expr::Literal(Literal::Boolean(true))),
        };
        let result = evaluator
            .eval_expr(&expr, &[], &SourceLocation::default())
            .unwrap();
        assert!(result);
    }
}
