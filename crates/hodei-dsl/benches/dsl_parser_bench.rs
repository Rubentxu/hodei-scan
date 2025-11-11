//! Benchmarks for DSL Parser Performance
//!
//! These benchmarks measure the parsing performance of the DSL parser
//! for various rule types and sizes.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hodei_dsl::parser::{parse_file, parse_rule};
use hodei_dsl::type_checker::TypeChecker;
use std::time::Duration;

// Simple rule for basic parsing
const SIMPLE_RULE: &str = r#"rule simple_rule {
    description: "Simple test rule"
    severity: "Medium"
    message: "Simple finding"
    confidence: "High"

    TaintSource: source
}"#;

// Rule with multiple patterns
const MULTI_PATTERN_RULE: &str = r#"rule multi_pattern_rule {
    description: "Rule with multiple patterns"
    severity: "High"
    message: "Multiple pattern finding"
    confidence: "Medium"

    TaintSource: source
    Function: func
    Variable: var
    TaintSink: sink
}"#;

// Complex rule with many patterns
const COMPLEX_RULE: &str = r#"rule complex_rule {
    description: "Complex rule with many patterns"
    severity: "Critical"
    message: "Complex finding: {location}"
    confidence: "High"

    TaintSource: source
    Function: func
    Variable: var
    TaintSink: sink
    Sanitization: sanitize
    CryptographicOperation: crypto
    UnsafeCall: unsafe_call
    Vulnerability: vuln
    CodeSmell: smell
    DependencyVulnerability: dep_vuln
    LowTestCoverage: low_coverage
}"#;

// File with multiple rules
const MULTI_RULE_FILE: &str = r#"
rule rule1 {
    description: "First rule"
    severity: "Low"
    message: "First finding"
    confidence: "Low"

    TaintSource: source
}

rule rule2 {
    description: "Second rule"
    severity: "Medium"
    message: "Second finding"
    confidence: "Medium"

    Vulnerability: vuln
}

rule rule3 {
    description: "Third rule"
    severity: "High"
    message: "Third finding"
    confidence: "High"

    CryptographicOperation: crypto
}

rule rule4 {
    description: "Fourth rule"
    severity: "Critical"
    message: "Fourth finding"
    confidence: "High"

    UnsafeCall: unsafe
}

rule rule5 {
    description: "Fifth rule"
    severity: "Medium"
    message: "Fifth finding"
    confidence: "Low"

    Function: func
}
"#;

// Large file with many rules
const LARGE_RULE_FILE: &str = r#"
rule rule_1 {
    description: "Rule 1"
    severity: "Low"
    message: "Finding 1"
    confidence: "Low"
    TaintSource: src1
}
rule rule_2 {
    description: "Rule 2"
    severity: "Medium"
    message: "Finding 2"
    confidence: "Medium"
    Vulnerability: vuln2
}
rule rule_3 {
    description: "Rule 3"
    severity: "High"
    message: "Finding 3"
    confidence: "High"
    Function: func3
}
rule rule_4 {
    description: "Rule 4"
    severity: "Critical"
    message: "Finding 4"
    confidence: "High"
    UnsafeCall: unsafe4
}
rule rule_5 {
    description: "Rule 5"
    severity: "Low"
    message: "Finding 5"
    confidence: "Low"
    Variable: var5
}
rule rule_6 {
    description: "Rule 6"
    severity: "Medium"
    message: "Finding 6"
    confidence: "Medium"
    TaintSink: sink6
}
rule rule_7 {
    description: "Rule 7"
    severity: "High"
    message: "Finding 7"
    confidence: "High"
    Sanitization: san7
}
rule rule_8 {
    description: "Rule 8"
    severity: "Critical"
    message: "Finding 8"
    confidence: "High"
    CryptographicOperation: crypto8
}
rule rule_9 {
    description: "Rule 9"
    severity: "Low"
    message: "Finding 9"
    confidence: "Low"
    CodeSmell: smell9
}
rule rule_10 {
    description: "Rule 10"
    severity: "Medium"
    message: "Finding 10"
    confidence: "Medium"
    DependencyVulnerability: dep10
}
rule rule_11 {
    description: "Rule 11"
    severity: "High"
    message: "Finding 11"
    confidence: "High"
    LowTestCoverage: cov11
}
rule rule_12 {
    description: "Rule 12"
    severity: "Critical"
    message: "Finding 12"
    confidence: "High"
    UncoveredLine: line12
}
rule rule_13 {
    description: "Rule 13"
    severity: "Low"
    message: "Finding 13"
    confidence: "Low"
    Dependency: dep13
}
rule rule_14 {
    description: "Rule 14"
    severity: "Medium"
    message: "Finding 14"
    confidence: "Medium"
    License: lic14
}
rule rule_15 {
    description: "Rule 15"
    severity: "High"
    message: "Finding 15"
    confidence: "High"
    CoverageStats: stats15
}
"#;

fn bench_parse_simple_rule(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_simple_rule");

    group.bench_function("parse", |b| {
        b.iter(|| black_box(parse_rule(black_box(SIMPLE_RULE))))
    });

    group.bench_function("parse_100_times", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(parse_rule(black_box(SIMPLE_RULE)));
            }
        })
    });

    group.finish();
}

fn bench_parse_multi_pattern_rule(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_multi_pattern_rule");

    group.bench_function("parse", |b| {
        b.iter(|| black_box(parse_rule(black_box(MULTI_PATTERN_RULE))))
    });

    group.bench_function("parse_100_times", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(parse_rule(black_box(MULTI_PATTERN_RULE)));
            }
        })
    });

    group.finish();
}

fn bench_parse_complex_rule(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_complex_rule");

    group.bench_function("parse", |b| {
        b.iter(|| black_box(parse_rule(black_box(COMPLEX_RULE))))
    });

    group.bench_function("parse_100_times", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(parse_rule(black_box(COMPLEX_RULE)));
            }
        })
    });

    group.finish();
}

fn bench_parse_multi_rule_file(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_multi_rule_file");

    group.bench_function("parse_5_rules", |b| {
        b.iter(|| black_box(parse_file(black_box(MULTI_RULE_FILE))))
    });

    group.bench_function("parse_15_rules", |b| {
        b.iter(|| black_box(parse_file(black_box(LARGE_RULE_FILE))))
    });

    group.bench_function("parse_15_rules_10_times", |b| {
        b.iter(|| {
            for _ in 0..10 {
                black_box(parse_file(black_box(LARGE_RULE_FILE)));
            }
        })
    });

    group.finish();
}

fn bench_type_checker(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_checker");

    let checker = TypeChecker::new();
    let simple_rule = parse_rule(SIMPLE_RULE).unwrap();
    let multi_pattern_rule = parse_rule(MULTI_PATTERN_RULE).unwrap();
    let complex_rule = parse_rule(COMPLEX_RULE).unwrap();

    group.bench_function("check_simple_rule", |b| {
        b.iter(|| black_box(checker.check_rule(&simple_rule)))
    });

    group.bench_function("check_multi_pattern_rule", |b| {
        b.iter(|| black_box(checker.check_rule(&multi_pattern_rule)))
    });

    group.bench_function("check_complex_rule", |b| {
        b.iter(|| black_box(checker.check_rule(&complex_rule)))
    });

    group.bench_function("check_100_simple_rules", |b| {
        b.iter(|| {
            for _ in 0..100 {
                black_box(checker.check_rule(&simple_rule));
            }
        })
    });

    group.finish();
}

fn bench_full_workflow(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_workflow");

    let checker = TypeChecker::new();

    group.bench_function("parse_and_check_simple", |b| {
        b.iter(|| {
            let rule = black_box(parse_rule(black_box(SIMPLE_RULE))).unwrap();
            black_box(checker.check_rule(&rule)).unwrap();
        })
    });

    group.bench_function("parse_and_check_multi_pattern", |b| {
        b.iter(|| {
            let rule = black_box(parse_rule(black_box(MULTI_PATTERN_RULE))).unwrap();
            black_box(checker.check_rule(&rule)).unwrap();
        })
    });

    group.bench_function("parse_and_check_complex", |b| {
        b.iter(|| {
            let rule = black_box(parse_rule(black_box(COMPLEX_RULE))).unwrap();
            black_box(checker.check_rule(&rule)).unwrap();
        })
    });

    group.bench_function("parse_and_check_file_5_rules", |b| {
        b.iter(|| {
            let file = black_box(parse_file(black_box(MULTI_RULE_FILE))).unwrap();
            for rule in file.rules {
                black_box(checker.check_rule(&rule)).unwrap();
            }
        })
    });

    group.bench_function("parse_and_check_file_15_rules", |b| {
        b.iter(|| {
            let file = black_box(parse_file(black_box(LARGE_RULE_FILE))).unwrap();
            for rule in file.rules {
                black_box(checker.check_rule(&rule)).unwrap();
            }
        })
    });

    group.finish();
}

fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");

    // Test parsing increasingly large numbers of rules
    for num_rules in [1, 5, 10, 15, 20] {
        let rules: String = (0..num_rules)
            .map(|i| {
                format!(
                    r#"rule rule_{i} {{
    description: "Rule {i}"
    severity: "Medium"
    message: "Finding {i}"
    confidence: "High"
    TaintSource: src{i}
}}"#
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        group.bench_function(format!("parse_{}_rules", num_rules), |b| {
            b.iter(|| {
                black_box(parse_file(black_box(&rules))).unwrap();
            })
        });
    }

    // Test type checking increasingly large numbers of rules
    for num_rules in [1, 5, 10, 15, 20] {
        let mut rules_vec = Vec::new();
        for i in 0..num_rules {
            let rule_text = format!(
                r#"rule rule_{i} {{
    description: "Rule {i}"
    severity: "Medium"
    message: "Finding {i}"
    confidence: "High"
    TaintSource: src{i}
}}"#
            );
            rules_vec.push(parse_rule(&rule_text).unwrap());
        }

        let checker = TypeChecker::new();
        group.bench_function(format!("type_check_{}_rules", num_rules), |b| {
            b.iter(|| {
                for rule in &rules_vec {
                    black_box(checker.check_rule(rule)).unwrap();
                }
            })
        });
    }

    group.finish();
}

fn bench_parsing_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing_speed");

    // Measure how many rules can be parsed per second
    group.bench_function("rules_per_second_simple", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();
            let mut count = 0;
            while start.elapsed() < Duration::from_millis(100) {
                black_box(parse_rule(black_box(SIMPLE_RULE)));
                count += 1;
            }
            black_box(count);
        })
    });

    group.bench_function("rules_per_second_multi_pattern", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();
            let mut count = 0;
            while start.elapsed() < Duration::from_millis(100) {
                black_box(parse_rule(black_box(MULTI_PATTERN_RULE)));
                count += 1;
            }
            black_box(count);
        })
    });

    group.bench_function("rules_per_second_complex", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();
            let mut count = 0;
            while start.elapsed() < Duration::from_millis(100) {
                black_box(parse_rule(black_box(COMPLEX_RULE)));
                count += 1;
            }
            black_box(count);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_simple_rule,
    bench_parse_multi_pattern_rule,
    bench_parse_complex_rule,
    bench_parse_multi_rule_file,
    bench_type_checker,
    bench_full_workflow,
    bench_scalability,
    bench_parsing_speed
);
criterion_main!(benches);
