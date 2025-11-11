//! Rule Engine - orchestrates rule evaluation and finding generation

mod evaluator;
mod finding;

pub use evaluator::*;
pub use finding::*;

use hodei_dsl::ast::RuleDef;
use hodei_ir::IntermediateRepresentation;
use rayon::prelude::*;
use std::time::Duration;
use thiserror::Error;

use crate::store::IndexedFactStore;

/// Main Rule Engine for evaluating rules over IR
pub struct RuleEngine {
    config: EngineConfig,
}

/// Configuration for Rule Engine
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Timeout per rule (default: 1s)
    pub per_rule_timeout: Duration,

    /// Maximum findings per rule (prevents memory exhaustion)
    pub max_findings_per_rule: usize,

    /// Parallelism level (default: number of CPU threads)
    pub parallelism: usize,

    /// Enable telemetry
    pub enable_telemetry: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            per_rule_timeout: Duration::from_secs(1),
            max_findings_per_rule: 10_000,
            parallelism: rayon::current_num_threads(),
            enable_telemetry: true,
        }
    }
}

/// Result of rule evaluation
pub struct EvaluationResult {
    pub findings: Vec<Finding>,
    pub stats: EvaluationStats,
}

/// Statistics from evaluation
#[derive(Debug, Default)]
pub struct EvaluationStats {
    pub total_rules: usize,
    pub successful_rules: usize,
    pub failed_rules: usize,
    pub total_findings: usize,
    pub total_duration: Duration,
    pub per_rule_stats: Vec<RuleStats>,
}

/// Statistics per rule
#[derive(Debug)]
pub struct RuleStats {
    pub rule_name: String,
    pub duration: Duration,
    pub findings_count: usize,
    pub error: Option<String>,
}

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Rule evaluation error: {0}")]
    RuleEvaluationError(#[from] RuleEvaluationError),
}

#[derive(Debug)]
pub struct RuleEvaluationError {
    pub rule_name: String,
    pub kind: EvaluationErrorKind,
}

impl std::fmt::Display for RuleEvaluationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.rule_name, self.kind)
    }
}

impl std::error::Error for RuleEvaluationError {}

#[derive(Error, Debug)]
pub enum EvaluationErrorKind {
    #[error("Rule evaluation timed out")]
    Timeout,

    #[error("Too many findings")]
    TooManyFindings,

    #[error("Pattern matching failed: {0}")]
    PatternMatchError(String),

    #[error("Expression evaluation failed: {0}")]
    ExprEvalError(String),
}

impl RuleEngine {
    /// Create a new Rule Engine with configuration
    pub fn new(config: EngineConfig) -> Self {
        Self { config }
    }

    /// Evaluate all rules over the IR
    pub fn evaluate(
        &self,
        rules: &[RuleDef],
        ir: &IntermediateRepresentation,
    ) -> Result<EvaluationResult, EngineError> {
        let start = std::time::Instant::now();

        // 1. Build fact store with indexes
        let store = IndexedFactStore::new(ir.facts.clone());

        // 2. Evaluate rules in parallel (simplified - no timeout for now)
        let rule_results: Vec<_> = rules
            .par_iter()
            .map(|rule| self.evaluate_rule(rule, &store))
            .collect();

        // 3. Aggregate results
        let mut findings = Vec::new();
        let mut stats = EvaluationStats {
            total_rules: rules.len(),
            ..Default::default()
        };

        for result in rule_results {
            match result {
                Ok((rule_findings, rule_stats)) => {
                    findings.extend(rule_findings);
                    stats.successful_rules += 1;
                    stats.per_rule_stats.push(rule_stats);
                }
                Err(err) => {
                    stats.failed_rules += 1;
                    stats.per_rule_stats.push(RuleStats {
                        rule_name: err.rule_name.clone(),
                        duration: Duration::ZERO,
                        findings_count: 0,
                        error: Some(err.kind.to_string()),
                    });
                }
            }
        }

        stats.total_findings = findings.len();
        stats.total_duration = start.elapsed();

        Ok(EvaluationResult { findings, stats })
    }

    /// Evaluate a single rule with timeout protection
    fn evaluate_rule(
        &self,
        rule: &RuleDef,
        store: &IndexedFactStore,
    ) -> Result<(Vec<Finding>, RuleStats), RuleEvaluationError> {
        let start = std::time::Instant::now();

        // Setup timeout using crossbeam channels
        let (tx, rx) = crossbeam::channel::bounded::<
            Result<(Vec<Finding>, RuleStats), RuleEvaluationError>,
        >(1);

        // Spawn evaluation in a separate thread
        std::thread::scope(|s| {
            s.spawn(|| {
                let result = self.evaluate_rule_impl(rule, store);
                let _ = tx.send(result);
            });

            // Wait for result with timeout
            match rx.recv_timeout(self.config.per_rule_timeout) {
                Ok(result) => {
                    let duration = start.elapsed();
                    match result {
                        Ok((findings, _stats)) => {
                            let findings_count = findings.len();
                            Ok((
                                findings,
                                RuleStats {
                                    rule_name: rule.name.clone(),
                                    duration,
                                    findings_count,
                                    error: None,
                                },
                            ))
                        }
                        Err(err) => Err(err),
                    }
                }
                Err(_timeout) => Err(RuleEvaluationError {
                    rule_name: rule.name.clone(),
                    kind: EvaluationErrorKind::Timeout,
                }),
            }
        })
    }

    /// Internal rule evaluation implementation (without timeout handling)
    fn evaluate_rule_impl(
        &self,
        rule: &RuleDef,
        store: &IndexedFactStore,
    ) -> Result<(Vec<Finding>, RuleStats), RuleEvaluationError> {
        let start = std::time::Instant::now();

        // 1. Pattern matching
        let matcher = PatternMatcher::new(store.clone());
        let matching_facts = matcher
            .match_patterns(&rule.match_block.patterns)
            .map_err(|e| RuleEvaluationError {
                rule_name: rule.name.clone(),
                kind: EvaluationErrorKind::PatternMatchError(e),
            })?;

        // 2. Filter by where clause
        let evaluator = ExprEvaluator::new(store.clone());
        let filtered_facts: Vec<_> = if let Some(where_clause) = &rule.match_block.where_clause {
            matching_facts
                .into_iter()
                .filter(|fact| {
                    evaluator
                        .eval_expr(where_clause, &[fact.clone()], &fact.location)
                        .unwrap_or(false)
                })
                .collect()
        } else {
            matching_facts
        };

        // 3. Generate findings
        let mut findings = Vec::new();

        for fact in filtered_facts {
            if findings.len() >= self.config.max_findings_per_rule {
                return Err(RuleEvaluationError {
                    rule_name: rule.name.clone(),
                    kind: EvaluationErrorKind::TooManyFindings,
                });
            }

            // Use a default message template that includes fact information
            let message_template = "Found {fact.type} at {fact.location.file}:{fact.location.start_line} with confidence {fact.confidence}";

            let finding = FindingBuilder::new()
                .rule_name(&rule.name)
                .message(message_template)
                .location(fact.location.clone())
                .confidence(fact.provenance.confidence)
                .severity("medium")
                .with_fact(fact.clone())
                .build()
                .unwrap_or_else(|_| Finding {
                    id: format!("{}-error", rule.name),
                    rule_name: rule.name.clone(),
                    message: "Error building finding".to_string(),
                    location: fact.location,
                    confidence: fact.provenance.confidence,
                    severity: "error".to_string(),
                    metadata: std::collections::HashMap::new(),
                    created_at: std::time::SystemTime::now(),
                });
            findings.push(finding);
        }

        let duration = start.elapsed();
        let stats = RuleStats {
            rule_name: rule.name.clone(),
            duration,
            findings_count: findings.len(),
            error: None,
        };

        Ok((findings, stats))
    }
}

impl Default for RuleEngine {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodei_dsl::ast::*;
    use hodei_ir::*;
    use std::path::PathBuf;

    fn create_test_source_location() -> SourceLocation {
        SourceLocation::new(
            ProjectPath::new(PathBuf::from("test.rs")).unwrap(),
            LineNumber::new(1).unwrap(),
            None,
            LineNumber::new(10).unwrap(),
            None,
        )
    }

    fn create_test_fact(fact_type: FactType) -> Fact {
        Fact::new(
            fact_type,
            create_test_source_location(),
            Provenance::new(
                ExtractorId::TreeSitter,
                SemanticVersion::new(1, 0, 0),
                Confidence::MEDIUM,
            ),
        )
    }

    fn create_test_ir_with_facts(facts: Vec<Fact>) -> IntermediateRepresentation {
        let metadata = ProjectMetadata::new(
            "test".to_string(),
            Some(SemanticVersion::new(1, 0, 0)),
            PathBuf::from("."),
            None,
            None,
            None,
        );
        IntermediateRepresentation::new(metadata, facts, AnalysisStats::default())
    }

    fn create_simple_rule(name: &str, fact_type: &str) -> RuleDef {
        RuleDef {
            name: name.to_string(),
            span: Span { start: 0, end: 0 },
            match_block: MatchBlock {
                patterns: vec![Pattern {
                    binding: "test".to_string(),
                    fact_type: fact_type.to_string(),
                    conditions: vec![],
                    span: Span { start: 0, end: 0 },
                }],
                where_clause: None,
            },
            emit_block: EmitBlock {
                message_template: "Found {fact.type}".to_string(),
                confidence: hodei_dsl::ast::Confidence::High,
                metadata: vec![],
            },
            metadata: RuleMetadata {
                severity: hodei_dsl::ast::Severity::High,
                category: "test".to_string(),
                description: "Test rule".to_string(),
            },
        }
    }

    #[test]
    fn test_evaluate_simple_rule() {
        let facts = vec![create_test_fact(FactType::Function {
            name: VariableName::new("test_func".to_string()),
            complexity: 5,
            lines_of_code: 10,
        })];
        let ir = create_test_ir_with_facts(facts);

        let rules = vec![create_simple_rule("test-func", "Function")];
        let engine = RuleEngine::default();

        let result = engine.evaluate(&rules, &ir).unwrap();

        assert_eq!(result.stats.total_rules, 1);
        assert_eq!(result.stats.successful_rules, 1);
        assert_eq!(result.stats.failed_rules, 0);
        assert_eq!(result.findings.len(), 1);
        assert!(result.findings[0].message.contains("Function"));
    }

    #[test]
    fn test_evaluate_rule_with_where_clause() {
        let facts = vec![create_test_fact(FactType::Function {
            name: VariableName::new("test_func".to_string()),
            complexity: 5,
            lines_of_code: 10,
        })];
        let ir = create_test_ir_with_facts(facts);

        // Rule with where clause that filters by confidence
        let rule = RuleDef {
            name: "complex-function".to_string(),
            span: Span { start: 0, end: 0 },
            match_block: MatchBlock {
                patterns: vec![Pattern {
                    binding: "func".to_string(),
                    fact_type: "Function".to_string(),
                    conditions: vec![],
                    span: Span { start: 0, end: 0 },
                }],
                where_clause: Some(Expr::Literal(Literal::Boolean(true))),
            },
            emit_block: EmitBlock {
                message_template: "Complex function found".to_string(),
                confidence: hodei_dsl::ast::Confidence::High,
                metadata: vec![],
            },
            metadata: RuleMetadata {
                severity: hodei_dsl::ast::Severity::High,
                category: "test".to_string(),
                description: "Test rule with where clause".to_string(),
            },
        };

        let rules = vec![rule];
        let engine = RuleEngine::default();

        let result = engine.evaluate(&rules, &ir).unwrap();

        assert_eq!(result.stats.total_rules, 1);
        assert_eq!(result.stats.successful_rules, 1);
        assert_eq!(result.findings.len(), 1);
    }

    #[test]
    fn test_evaluate_rule_timeout() {
        let facts = vec![];
        let ir = create_test_ir_with_facts(facts);

        let rule = create_simple_rule("slow-rule", "Function");
        let config = EngineConfig {
            per_rule_timeout: std::time::Duration::from_millis(1),
            max_findings_per_rule: 1000,
            parallelism: 1,
            enable_telemetry: false,
        };

        let engine = RuleEngine::new(config);
        let result = engine.evaluate(&[rule], &ir).unwrap();

        assert_eq!(result.stats.total_rules, 1);
        assert_eq!(result.stats.failed_rules, 1);
        assert_eq!(result.stats.successful_rules, 0);
        assert!(result.stats.per_rule_stats[0].error.is_some());
    }

    #[test]
    fn test_template_interpolation() {
        let fact = create_test_fact(FactType::Function {
            name: VariableName::new("test_func".to_string()),
            complexity: 5,
            lines_of_code: 10,
        });

        let finding = FindingBuilder::new()
            .rule_name("test-template")
            .message("Found {fact.type} in {fact.location.file} at line {fact.location.start_line} with confidence {fact.confidence}")
            .location(fact.location.clone())
            .confidence(fact.provenance.confidence)
            .severity("high")
            .with_fact(fact)
            .build()
            .unwrap();

        assert!(finding.message.contains("Function"));
        assert!(finding.message.contains("test.rs"));
        assert!(finding.message.contains("1"));
        assert!(finding.message.contains("0.60"));
    }

    #[test]
    fn test_max_findings_limit() {
        // Create 100 facts
        let facts: Vec<_> = (0..100)
            .map(|_| {
                create_test_fact(FactType::Function {
                    name: VariableName::new("test_func".to_string()),
                    complexity: 5,
                    lines_of_code: 10,
                })
            })
            .collect();
        let ir = create_test_ir_with_facts(facts);

        let rules = vec![create_simple_rule("test-funcs", "Function")];
        let config = EngineConfig {
            per_rule_timeout: std::time::Duration::from_secs(10),
            max_findings_per_rule: 5,
            parallelism: 1,
            enable_telemetry: false,
        };

        let engine = RuleEngine::new(config);
        let result = engine.evaluate(&rules, &ir).unwrap();

        // Should be limited to 5 findings despite having 100 matching facts
        assert_eq!(result.findings.len(), 5);
        assert_eq!(result.stats.total_rules, 1);
    }

    #[test]
    fn test_evaluator_binary_operations() {
        let store = IndexedFactStore::new(vec![]);
        let evaluator = ExprEvaluator::new(store);

        // Test AND operator with short-circuit
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Boolean(false))),
            op: BinaryOp::And,
            right: Box::new(Expr::Literal(Literal::Boolean(true))),
        };

        let result = evaluator
            .eval_expr(&expr, &[], &create_test_source_location())
            .unwrap();
        assert!(!result); // Short-circuit should return false

        // Test OR operator
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Boolean(false))),
            op: BinaryOp::Or,
            right: Box::new(Expr::Literal(Literal::Boolean(true))),
        };

        let result = evaluator
            .eval_expr(&expr, &[], &create_test_source_location())
            .unwrap();
        assert!(result);

        // Test EQ operator
        let expr = Expr::Binary {
            left: Box::new(Expr::Literal(Literal::Number(5.0))),
            op: BinaryOp::Eq,
            right: Box::new(Expr::Literal(Literal::Number(5.0))),
        };

        let result = evaluator
            .eval_expr(&expr, &[], &create_test_source_location())
            .unwrap();
        assert!(result);
    }

    #[test]
    fn test_pattern_matcher() {
        let facts = vec![
            create_test_fact(FactType::Function {
                name: VariableName::new("test_func".to_string()),
                complexity: 5,
                lines_of_code: 10,
            }),
            create_test_fact(FactType::Function {
                name: VariableName::new("another_func".to_string()),
                complexity: 10,
                lines_of_code: 20,
            }),
        ];

        let store = IndexedFactStore::new(facts);
        let matcher = PatternMatcher::new(store);

        let patterns = vec![Pattern {
            binding: "func".to_string(),
            fact_type: "Function".to_string(),
            conditions: vec![],
            span: Span { start: 0, end: 0 },
        }];

        let results = matcher.match_patterns(&patterns).unwrap();
        assert_eq!(results.len(), 2);
        assert!(matches!(results[0].fact_type, FactType::Function { .. }));
    }
}
