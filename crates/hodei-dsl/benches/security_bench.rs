//! Benchmarks for DSL Security Features
//!
//! These benchmarks measure the performance overhead of security validation
//! and sandboxing features.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hodei_dsl::ast::*;
use hodei_dsl::security::*;
use std::thread;
use std::time::Duration;

fn bench_validator(b: &mut Criterion) {
    let mut group = b.benchmark_group("DSLValidator");

    let validator = DSLValidator::new();

    let valid_input = r#"
        rule test_rule {
            match {
                TaintSource
            }
            emit {
                severity: High
                message: "Test rule"
            }
        }
    "#;

    group.bench_function("validate_valid_input", |b| {
        b.iter(|| black_box(validator.validate_raw_input(valid_input)))
    });

    // Test with malicious input
    let malicious_inputs = vec![
        "<script>alert('xss')</script>",
        "../../../etc/passwd",
        "javascript:alert('xss')",
        "test\0null",
    ];

    for input in malicious_inputs {
        group.bench_function(
            format!(
                "validate_malicious_{}",
                input.split(':').next().unwrap_or("input")
            ),
            |b| b.iter(|| black_box(validator.validate_raw_input(input))),
        );
    }

    group.finish();
}

fn bench_sandbox(b: &mut Criterion) {
    let mut group = b.benchmark_group("ExecutionSandbox");

    // Test with time limit
    for time_limit in [100, 500, 1000] {
        let mut sandbox = ExecutionSandbox::with_config(SandboxConfig {
            time_limit_ms: time_limit,
            ..Default::default()
        });

        group.bench_function(format!("execute_with_timeout_{}ms", time_limit), |b| {
            b.iter(|| {
                black_box(sandbox.execute(|| {
                    thread::sleep(Duration::from_millis(10));
                    Ok(42)
                }))
            })
        });
    }

    // Test complex execution
    let mut complex_sandbox = ExecutionSandbox::new();
    group.bench_function("execute_complex_calculation", |b| {
        b.iter(|| {
            black_box(complex_sandbox.execute(|| {
                // Simulate some computation
                let mut sum = 0;
                for i in 0..1000 {
                    sum += i * i;
                }
                Ok(sum)
            }))
        })
    });

    group.finish();
}

fn bench_complexity_analyzer(b: &mut Criterion) {
    let mut group = b.benchmark_group("RuleComplexityAnalyzer");

    let analyzer = RuleComplexityAnalyzer::new();

    // Simple rule
    let simple_rule = RuleDef {
        name: "simple".to_string(),
        metadata: Default::default(),
        match_block: MatchBlock {
            patterns: vec![Pattern {
                binding: "source".to_string(),
                fact_type: "TaintSource".to_string(),
                conditions: vec![],
                span: Default::default(),
            }],
            where_clause: None,
        },
        emit_block: EmitBlock {
            message_template: "Test".to_string(),
            confidence: Confidence::Medium,
            metadata: std::collections::HashMap::new(),
        },
        span: Default::default(),
    };

    group.bench_function("analyze_simple_rule", |b| {
        b.iter(|| black_box(analyzer.estimate_complexity(&simple_rule)))
    });

    // Complex rule
    let complex_patterns = (0..10)
        .map(|i| Pattern {
            binding: format!("var{}", i),
            fact_type: format!("Type{}", i),
            conditions: vec![],
            span: Default::default(),
        })
        .collect();

    let complex_rule = RuleDef {
        name: "complex".to_string(),
        metadata: Default::default(),
        match_block: MatchBlock {
            patterns: complex_patterns,
            where_clause: Some(Expr::Binary {
                op: BinaryOp::And,
                left: Box::new(Expr::Literal(Literal::Boolean(true))),
                right: Box::new(Expr::Literal(Literal::Boolean(true))),
            }),
        },
        emit_block: EmitBlock {
            message_template: "Test".to_string(),
            confidence: Confidence::Medium,
            metadata: std::collections::HashMap::new(),
        },
        span: Default::default(),
    };

    group.bench_function("analyze_complex_rule", |b| {
        b.iter(|| black_box(analyzer.estimate_complexity(&complex_rule)))
    });

    // Very complex rule
    let very_complex_patterns = (0..20)
        .map(|i| Pattern {
            binding: format!("var{}", i),
            fact_type: format!("Type{}", i),
            conditions: vec![],
            span: Default::default(),
        })
        .collect();

    let very_complex_rule = RuleDef {
        name: "very_complex".to_string(),
        metadata: Default::default(),
        match_block: MatchBlock {
            patterns: very_complex_patterns,
            where_clause: Some(Expr::Binary {
                op: BinaryOp::And,
                left: Box::new(Expr::Binary {
                    op: BinaryOp::Or,
                    left: Box::new(Expr::Literal(Literal::Boolean(true))),
                    right: Box::new(Expr::Literal(Literal::Boolean(false))),
                }),
                right: Box::new(Expr::Binary {
                    op: BinaryOp::And,
                    left: Box::new(Expr::Literal(Literal::Boolean(true))),
                    right: Box::new(Expr::Literal(Literal::Boolean(true))),
                }),
            }),
        },
        emit_block: EmitBlock {
            message_template: "Test".to_string(),
            confidence: Confidence::Medium,
            metadata: std::collections::HashMap::new(),
        },
        span: Default::default(),
    };

    group.bench_function("analyze_very_complex_rule", |b| {
        b.iter(|| black_box(analyzer.estimate_complexity(&very_complex_rule)))
    });

    group.finish();
}

fn bench_security_overhead(b: &mut Criterion) {
    let mut group = b.benchmark_group("SecurityOverhead");

    let validator = DSLValidator::new();
    let mut sandbox = ExecutionSandbox::new();
    let analyzer = RuleComplexityAnalyzer::new();

    let valid_rule = r#"
        rule test {
            match {
                TaintSource
            }
            emit {
                severity: High
            }
        }
    "#;

    // Benchmark validation + sandbox execution + complexity analysis
    group.bench_function("full_security_pipeline", |b| {
        b.iter(|| {
            // 1. Validate input
            black_box(validator.validate_raw_input(valid_rule)).unwrap();

            // 2. Parse rule (simplified)
            let rule = RuleDef {
                name: "test".to_string(),
                metadata: Default::default(),
                match_block: MatchBlock {
                    patterns: vec![Pattern {
                        binding: "source".to_string(),
                        fact_type: "TaintSource".to_string(),
                        conditions: vec![],
                        span: Default::default(),
                    }],
                    where_clause: None,
                },
                emit_block: EmitBlock {
                    message_template: "Test".to_string(),
                    confidence: Confidence::Medium,
                    metadata: std::collections::HashMap::new(),
                },
                span: Default::default(),
            };

            // 3. Analyze complexity
            black_box(analyzer.estimate_complexity(&rule)).unwrap();

            // 4. Execute in sandbox
            black_box(sandbox.execute(|| Ok(42))).result.unwrap();
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_validator,
    bench_sandbox,
    bench_complexity_analyzer,
    bench_security_overhead
);
criterion_main!(benches);
