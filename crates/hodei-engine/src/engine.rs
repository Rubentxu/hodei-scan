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

    /// Evaluate a single rule
    fn evaluate_rule(
        &self,
        rule: &RuleDef,
        store: &IndexedFactStore,
    ) -> Result<(Vec<Finding>, RuleStats), RuleEvaluationError> {
        let start = std::time::Instant::now();

        // Simple timeout check
        if start.elapsed() > self.config.per_rule_timeout {
            return Err(RuleEvaluationError {
                rule_name: rule.name.clone(),
                kind: EvaluationErrorKind::Timeout,
            });
        }

        // 1. Pattern matching
        let matcher = PatternMatcher::new(store.clone());
        let matching_facts = matcher.match_patterns(&rule.match_block.patterns);

        // 2. Filter by where clause
        let evaluator = ExprEvaluator::new();
        let filtered_facts: Vec<_> = if let Some(where_clause) = &rule.match_block.where_clause {
            matching_facts
                .into_iter()
                .filter(|fact| evaluator.eval_expr(where_clause, &[fact.clone()], &fact.location))
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

            let finding = FindingBuilder::new()
                .rule_name(&rule.name)
                .message(&format!("Found fact: {:?} at location", fact.fact_type))
                .location(fact.location.clone())
                .confidence(fact.provenance.confidence)
                .severity("medium")
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
