use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use hodei_dsl::ast::*;
use hodei_engine::*;
use hodei_ir::*;
use std::path::PathBuf;

/// Benchmark the rule engine with different numbers of facts
fn bench_evaluate_rules(c: &mut Criterion) {
    let mut group = c.benchmark_group("RuleEngine");

    // Create test data with varying sizes
    let test_sizes = [100, 1000, 10_000];

    for size in test_sizes {
        let facts = create_test_facts(size);
        let ir = create_test_ir(facts);
        let rules = create_test_rules(10); // 10 rules

        group.bench_with_input(BenchmarkId::new("evaluate", size), &size, |b, _| {
            let engine = RuleEngine::default();
            b.iter(|| {
                let result = engine.evaluate(&rules, &ir).unwrap();
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark pattern matching performance
fn bench_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("PatternMatcher");

    let sizes = [100, 1000, 5000];

    for size in sizes {
        let facts = create_test_facts(size);
        let store = IndexedFactStore::new(facts);
        let matcher = PatternMatcher::new(store);

        let patterns = vec![Pattern {
            binding: "func".to_string(),
            fact_type: "Function".to_string(),
            conditions: vec![],
            span: Span { start: 0, end: 0 },
        }];

        group.bench_with_input(BenchmarkId::new("match_patterns", size), &size, |b, _| {
            b.iter(|| {
                let result = matcher.match_patterns(&patterns).unwrap();
                black_box(result)
            });
        });
    }

    group.finish();
}

/// Benchmark expression evaluation
fn bench_expression_evaluation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ExprEvaluator");

    let store = IndexedFactStore::new(vec![]);
    let evaluator = ExprEvaluator::new(store);

    // Test simple boolean expression
    let simple_expr = Expr::Literal(Literal::Boolean(true));

    group.bench_function("eval_literal", |b| {
        b.iter(|| {
            let result = evaluator.eval_expr(&simple_expr, &[], &SourceLocation::default());
            black_box(result)
        });
    });

    // Test binary expression
    let binary_expr = Expr::Binary {
        left: Box::new(Expr::Literal(Literal::Boolean(true))),
        op: BinaryOp::And,
        right: Box::new(Expr::Literal(Literal::Boolean(true))),
    };

    group.bench_function("eval_binary", |b| {
        b.iter(|| {
            let result = evaluator.eval_expr(&binary_expr, &[], &SourceLocation::default());
            black_box(result)
        });
    });

    group.finish();
}

/// Benchmark finding generation with template interpolation
fn bench_finding_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("FindingBuilder");

    let template = "Found {fact.type} in {fact.location.file}:{fact.location.start_line} with confidence {fact.confidence}";

    // Create a test fact
    let fact = create_single_fact();

    group.bench_function("build_with_template", |b| {
        b.iter(|| {
            let finding = FindingBuilder::new()
                .rule_name("test-rule")
                .message(template)
                .location(fact.location.clone())
                .confidence(fact.provenance.confidence)
                .severity("high")
                .with_fact(fact.clone())
                .build()
                .unwrap();
            black_box(finding)
        });
    });

    group.bench_function("build_without_template", |b| {
        b.iter(|| {
            let finding = FindingBuilder::new()
                .rule_name("test-rule")
                .message("Simple message")
                .location(fact.location.clone())
                .confidence(fact.provenance.confidence)
                .severity("high")
                .build()
                .unwrap();
            black_box(finding)
        });
    });

    group.finish();
}

/// Helper function to create test facts
fn create_test_facts(count: usize) -> Vec<Fact> {
    (0..count)
        .map(|i| create_fact_with_complexity(i as u32))
        .collect()
}

/// Create a single test fact
fn create_single_fact() -> Fact {
    create_fact_with_complexity(0)
}

/// Create a test fact with varying complexity
fn create_fact_with_complexity(complexity: u32) -> Fact {
    let location = SourceLocation::new(
        ProjectPath::new(PathBuf::from(format!("test{}.rs", complexity % 10))).unwrap(),
        LineNumber::new(1).unwrap(),
        None,
        LineNumber::new(10).unwrap(),
        None,
    );

    Fact::new_with_message(
        FactType::Function {
            name: VariableName::new(format!("func_{}", complexity)),
            complexity,
            lines_of_code: complexity * 2,
        },
        location,
        Provenance::new(
            ExtractorId::TreeSitter,
            version_without_semver(), // Use simplified version
            Confidence::MEDIUM,
        ),
    )
}

/// Create test rules
fn create_test_rules(count: usize) -> Vec<RuleDef> {
    (0..count)
        .map(|i| RuleDef {
            name: format!("rule_{}", i),
            span: Span { start: 0, end: 0 },
            match_block: MatchBlock {
                patterns: vec![Pattern {
                    binding: "func".to_string(),
                    fact_type: "Function".to_string(),
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
        })
        .collect()
}

/// Create test IR
fn create_test_ir(facts: Vec<Fact>) -> IntermediateRepresentation {
    let metadata = ProjectMetadata::new(
        "bench".to_string(),
        None,
        PathBuf::from("."),
        None,
        None,
        None,
    );
    IntermediateRepresentation::new(metadata, facts, AnalysisStats::default())
}

/// Simplified version without SemanticVersion
fn version_without_semver() -> hodei_ir::Confidence {
    Confidence::MEDIUM
}

criterion_group!(
    benches,
    bench_evaluate_rules,
    bench_pattern_matching,
    bench_expression_evaluation,
    bench_finding_generation
);
criterion_main!(benches);
