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
        // Match by fact_type string with support for Custom FactTypes
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
                    // Handle Custom FactTypes with pattern matching
                    hodei_ir::FactType::Custom { discriminant, .. } => {
                        self.match_custom_fact_type(fact_type_str, discriminant)
                    }
                }
            })
            .collect()
    }

    /// Match Custom FactTypes using patterns
    ///
    /// Supports the following patterns:
    /// - "Custom" - matches any custom fact type
    /// - "Custom:discriminant" - matches exact discriminant
    /// - "Custom:prefix*" - matches discriminants starting with prefix
    /// - "Custom:*suffix" - matches discriminants ending with suffix
    /// - "Custom:pattern" - pattern matching with wildcards
    fn match_custom_fact_type(&self, pattern: &str, discriminant: &str) -> bool {
        // Parse pattern: "Custom" or "Custom:pattern"
        let parts: Vec<&str> = pattern.split(':').collect();

        if parts.is_empty() {
            return false;
        }

        // Must start with "Custom"
        if parts[0] != "Custom" {
            return false;
        }

        // If just "Custom", match any custom fact type
        if parts.len() == 1 {
            return true;
        }

        // Extract the pattern part (everything after "Custom:")
        let pattern_part = parts[1..].join(":");

        // Support wildcard patterns
        if pattern_part.contains('*') {
            return self.match_with_wildcard(&pattern_part, discriminant);
        }

        // Exact match
        pattern_part == discriminant
    }

    /// Simple wildcard matching function
    /// Supports:
    /// - "*" wildcard for multiple characters
    /// - "?" wildcard for single character
    fn match_with_wildcard(&self, pattern: &str, text: &str) -> bool {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let text_chars: Vec<char> = text.chars().collect();

        self.match_wildcard_recursive(&pattern_chars, &text_chars, 0, 0)
    }

    fn match_wildcard_recursive(
        &self,
        pattern: &[char],
        text: &[char],
        p_idx: usize,
        t_idx: usize,
    ) -> bool {
        // If we've processed all pattern characters
        if p_idx == pattern.len() {
            return t_idx == text.len();
        }

        // If we've processed all text characters
        if t_idx == text.len() {
            // Check if remaining pattern is all wildcards
            return pattern[p_idx..].iter().all(|&c| c == '*');
        }

        let pattern_char = pattern[p_idx];
        let text_char = text[t_idx];

        if pattern_char == '*' {
            // Wildcard can match zero or more characters
            self.match_wildcard_recursive(pattern, text, p_idx + 1, t_idx)
                || self.match_wildcard_recursive(pattern, text, p_idx + 1, t_idx + 1)
                || self.match_wildcard_recursive(pattern, text, p_idx, t_idx + 1)
        } else if pattern_char == '?' {
            // Wildcard matches exactly one character
            self.match_wildcard_recursive(pattern, text, p_idx + 1, t_idx + 1)
        } else if pattern_char == text_char {
            // Characters match, continue
            self.match_wildcard_recursive(pattern, text, p_idx + 1, t_idx + 1)
        } else {
            // Characters don't match
            false
        }
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

        // Support accessing Custom fact fields with dot notation: fact.field
        if var_name.contains('.') {
            // Split by the last dot to separate field from fact
            if let Some((fact_part, field_name)) = var_name.rsplit_once('.') {
                // Handle "Custom:discriminant.field" format
                if fact_part.starts_with("Custom:") {
                    let discriminant = &fact_part["Custom:".len()..];

                    // Look for a custom fact with this binding
                    let has_fact = facts.iter().any(|fact| {
                        if let hodei_ir::FactType::Custom {
                            discriminant: fact_discriminant,
                            data,
                            ..
                        } = &fact.fact_type
                        {
                            fact_discriminant == discriminant && data.contains_key(field_name)
                        } else {
                            false
                        }
                    });

                    return Ok(has_fact);
                }
            }
        }

        // Look for a fact with this binding
        let has_fact = facts.iter().any(|fact| {
            // Check if fact type name matches or location matches
            let discriminant_str = format!("{:?}", fact.fact_type.discriminant());
            discriminant_str == *var_name ||
            // Also support "Custom" to match any custom fact
            (var_name == "Custom" && matches!(fact.fact_type, hodei_ir::FactType::Custom { .. }))
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
            hodei_dsl::ast::Expr::Literal(lit) => {
                // Extract just the value, not the enum variant
                let result = match lit {
                    hodei_dsl::ast::Literal::String(s) => s.clone(),
                    hodei_dsl::ast::Literal::Number(n) => n.to_string(),
                    hodei_dsl::ast::Literal::Boolean(b) => b.to_string(),
                    hodei_dsl::ast::Literal::Null => "null".to_string(),
                };
                Ok(result)
            }
            hodei_dsl::ast::Expr::Path(path) => {
                let var_name = &path.segments[0];

                // Support accessing Custom fact field values with dot notation
                // Format: "Custom:plugin:type.field" or "fact.field"
                if var_name.contains('.') {
                    // Split by the last dot to separate field from fact
                    if let Some((fact_part, field_part)) = var_name.rsplit_once('.') {
                        // Handle "Custom:discriminant.field" format
                        if fact_part.starts_with("Custom:") {
                            let discriminant = &fact_part["Custom:".len()..];

                            // Find the custom fact with this discriminant
                            for fact in facts {
                                if let hodei_ir::FactType::Custom {
                                    discriminant: fact_discriminant,
                                    data,
                                    ..
                                } = &fact.fact_type
                                {
                                    if fact_discriminant == discriminant {
                                        // Get the field value
                                        if let Some(value) = data.get(field_part) {
                                            return Ok(self.fact_value_to_string(value));
                                        }
                                    }
                                }
                            }
                            return Err(format!(
                                "Field '{}' not found in custom fact 'Custom:{}'",
                                field_part, discriminant
                            ));
                        }
                    }
                }

                // Simplified: return the variable name
                Ok(path.segments[0].clone())
            }
            _ => Err("Cannot convert to value for comparison".to_string()),
        }
    }

    /// Convert a FactValue to a string for comparison
    fn fact_value_to_string(&self, value: &hodei_ir::FactValue) -> String {
        match value {
            hodei_ir::FactValue::String(s) => s.clone(),
            hodei_ir::FactValue::Number(n) => n.to_string(),
            hodei_ir::FactValue::Boolean(b) => b.to_string(),
            hodei_ir::FactValue::Array(arr) => format!(
                "[{}]",
                arr.iter()
                    .map(|v| self.fact_value_to_string(v))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            hodei_ir::FactValue::Object(obj) => format!(
                "{{{}}}",
                obj.iter()
                    .map(|(k, v)| format!("{}: {}", k, self.fact_value_to_string(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodei_dsl::ast::*;
    use hodei_ir::*;
    use std::collections::HashMap;
    use std::time::SystemTime;

    fn create_test_ir() -> IntermediateRepresentation {
        let metadata = ProjectMetadata::new(
            "test".to_string(),
            "1.0".to_string(),
            ProjectPath::new(std::path::PathBuf::from(".")),
        );
        IntermediateRepresentation::new(metadata)
    }

    fn create_custom_fact(discriminant: String, data: HashMap<String, FactValue>) -> Fact {
        let fact_type = FactType::Custom {
            discriminant: discriminant.clone(),
            data,
        };

        Fact::new_with_message(
            fact_type,
            "Test finding".to_string(),
            SourceLocation::default(),
            Provenance::new(
                hodei_ir::ExtractorId::Custom,
                "1.0.0".to_string(),
                hodei_ir::Confidence::MEDIUM,
            ),
        )
    }

    #[test]
    fn test_pattern_matcher_custom_any() {
        let mut data = HashMap::new();
        data.insert(
            "field1".to_string(),
            FactValue::String("value1".to_string()),
        );

        let facts = vec![
            create_custom_fact("plugin1:custom".to_string(), data.clone()),
            create_custom_fact("plugin2:other".to_string(), data.clone()),
        ];

        let store = crate::store::IndexedFactStore::new(facts);
        let matcher = PatternMatcher::new(store);

        // Match any Custom fact type
        let patterns = vec![Pattern {
            binding: "custom".to_string(),
            fact_type: "Custom".to_string(),
            conditions: vec![],
            span: hodei_dsl::ast::Span { start: 0, end: 0 },
        }];

        let results = matcher.match_patterns(&patterns).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_pattern_matcher_custom_exact_discriminant() {
        let mut data = HashMap::new();
        data.insert(
            "field1".to_string(),
            FactValue::String("value1".to_string()),
        );

        let facts = vec![
            create_custom_fact("plugin1:custom".to_string(), data.clone()),
            create_custom_fact("plugin2:other".to_string(), data.clone()),
            create_custom_fact("plugin1:different".to_string(), data.clone()),
        ];

        let store = crate::store::IndexedFactStore::new(facts);
        let matcher = PatternMatcher::new(store);

        // Match exact discriminant
        let patterns = vec![Pattern {
            binding: "custom".to_string(),
            fact_type: "Custom:plugin1:custom".to_string(),
            conditions: vec![],
            span: hodei_dsl::ast::Span { start: 0, end: 0 },
        }];

        let results = matcher.match_patterns(&patterns).unwrap();
        assert_eq!(results.len(), 1);

        if let FactType::Custom { discriminant, .. } = &results[0].fact_type {
            assert_eq!(discriminant, "plugin1:custom");
        } else {
            panic!("Expected Custom fact type");
        }
    }

    #[test]
    fn test_pattern_matcher_custom_wildcard_prefix() {
        let mut data = HashMap::new();
        data.insert(
            "field1".to_string(),
            FactValue::String("value1".to_string()),
        );

        let facts = vec![
            create_custom_fact("plugin1:custom".to_string(), data.clone()),
            create_custom_fact("plugin1:other".to_string(), data.clone()),
            create_custom_fact("plugin2:custom".to_string(), data.clone()),
            create_custom_fact("plugin3:custom".to_string(), data.clone()),
        ];

        let store = crate::store::IndexedFactStore::new(facts);
        let matcher = PatternMatcher::new(store);

        // Match with wildcard prefix
        let patterns = vec![Pattern {
            binding: "custom".to_string(),
            fact_type: "Custom:plugin1:*".to_string(),
            conditions: vec![],
            span: hodei_dsl::ast::Span { start: 0, end: 0 },
        }];

        let results = matcher.match_patterns(&patterns).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_pattern_matcher_custom_wildcard_suffix() {
        let mut data = HashMap::new();
        data.insert(
            "field1".to_string(),
            FactValue::String("value1".to_string()),
        );

        let facts = vec![
            create_custom_fact("plugin:custom".to_string(), data.clone()),
            create_custom_fact("plugin:special".to_string(), data.clone()),
            create_custom_fact("plugin:other".to_string(), data.clone()),
        ];

        let store = crate::store::IndexedFactStore::new(facts);
        let matcher = PatternMatcher::new(store);

        // Match with wildcard suffix
        let patterns = vec![Pattern {
            binding: "custom".to_string(),
            fact_type: "Custom:*:custom".to_string(),
            conditions: vec![],
            span: hodei_dsl::ast::Span { start: 0, end: 0 },
        }];

        let results = matcher.match_patterns(&patterns).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_expr_evaluator_custom_field_access() {
        let mut data1 = HashMap::new();
        data1.insert(
            "severity".to_string(),
            FactValue::String("high".to_string()),
        );
        data1.insert("count".to_string(), FactValue::Number(42.0));

        let mut data2 = HashMap::new();
        data2.insert("severity".to_string(), FactValue::String("low".to_string()));
        data2.insert("count".to_string(), FactValue::Number(10.0));

        let facts = vec![
            create_custom_fact("plugin1:alert".to_string(), data1),
            create_custom_fact("plugin2:info".to_string(), data2),
        ];

        let store = crate::store::IndexedFactStore::new(vec![]);
        let evaluator = ExprEvaluator::new(store);

        // Test accessing custom field
        let path = Path {
            segments: vec!["Custom:plugin1:alert.severity".to_string()],
            span: hodei_dsl::ast::Span { start: 0, end: 0 },
        };
        let result = evaluator.eval_path(&path, &facts).unwrap();
        assert!(result);

        // Test non-existent field
        let path = Path {
            segments: vec!["Custom:plugin1:alert.missing".to_string()],
            span: hodei_dsl::ast::Span { start: 0, end: 0 },
        };
        let result = evaluator.eval_path(&path, &facts).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_expr_evaluator_custom_field_comparison() {
        let mut data = HashMap::new();
        data.insert("count".to_string(), FactValue::Number(42.0));
        data.insert("name".to_string(), FactValue::String("test".to_string()));

        let facts = vec![create_custom_fact("plugin1:alert".to_string(), data)];

        let store = crate::store::IndexedFactStore::new(vec![]);
        let evaluator = ExprEvaluator::new(store);

        // Test comparison with custom field
        let left = Expr::Path(Path {
            segments: vec!["Custom:plugin1:alert.count".to_string()],
            span: hodei_dsl::ast::Span { start: 0, end: 0 },
        });
        let right = Expr::Literal(Literal::Number(42.0));

        let expr = Expr::Binary {
            left: Box::new(left),
            op: BinaryOp::Eq,
            right: Box::new(right),
        };

        let result = evaluator.eval_expr(&expr, &facts, &SourceLocation::default());

        println!("Evaluation result: {:?}", result);
        assert!(result.is_ok(), "Evaluation failed: {:?}", result);
        assert!(result.unwrap());
    }

    #[test]
    fn test_fact_value_to_string() {
        let store = crate::store::IndexedFactStore::new(vec![]);
        let evaluator = ExprEvaluator::new(store);

        // Test String value
        let value = FactValue::String("test".to_string());
        assert_eq!(evaluator.fact_value_to_string(&value), "test");

        // Test Number value
        let value = FactValue::Number(42.5);
        assert_eq!(evaluator.fact_value_to_string(&value), "42.5");

        // Test Boolean value
        let value = FactValue::Boolean(true);
        assert_eq!(evaluator.fact_value_to_string(&value), "true");

        // Test Array value
        let value = FactValue::Array(vec![
            FactValue::String("a".to_string()),
            FactValue::Number(1.0),
        ]);
        let result = evaluator.fact_value_to_string(&value);
        assert!(result.contains("a"));
        assert!(result.contains("1"));

        // Test Object value
        let mut obj = HashMap::new();
        obj.insert("key".to_string(), FactValue::String("value".to_string()));
        let value = FactValue::Object(obj);
        let result = evaluator.fact_value_to_string(&value);
        assert!(result.contains("key"));
        assert!(result.contains("value"));
    }

    #[test]
    fn test_wildcard_matching() {
        let store = crate::store::IndexedFactStore::new(vec![]);
        let matcher = PatternMatcher::new(store);

        // Test exact match
        assert!(matcher.match_with_wildcard("test", "test"));

        // Test wildcard at end
        assert!(matcher.match_with_wildcard("test*", "test123"));
        assert!(matcher.match_with_wildcard("test*", "test"));

        // Test wildcard at start
        assert!(matcher.match_with_wildcard("*test", "123test"));
        assert!(matcher.match_with_wildcard("*test", "test"));

        // Test wildcard in middle
        assert!(matcher.match_with_wildcard("te*st", "test"));
        assert!(matcher.match_with_wildcard("te*st", "te123st"));

        // Test multiple wildcards
        assert!(matcher.match_with_wildcard("*test*", "123test456"));

        // Test question mark wildcard
        assert!(matcher.match_with_wildcard("test?", "test1"));
        assert!(matcher.match_with_wildcard("test??", "test12"));

        // Test no match
        assert!(!matcher.match_with_wildcard("test*", "fail"));
        assert!(!matcher.match_with_wildcard("*test", "fail"));
        assert!(!matcher.match_with_wildcard("test?", "test"));
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
