//! Benchmarks for Query Planner with Cost-Based Optimization

use chrono::Utc;
use hodei_engine::store::planner::*;
use hodei_ir::*;
use std::collections::HashMap;
use std::path::PathBuf;

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

/// Generate a large dataset for benchmarking
fn generate_large_fact_dataset(size: usize) -> Vec<Fact> {
    let mut facts = Vec::with_capacity(size);

    for i in 0..size {
        // Create different types of facts to test different strategies
        let fact_type = match i % 4 {
            0 => FactType::TaintSource {
                var: VariableName(("var_".to_string() + &i.to_string()).into()),
                flow_id: FlowId::new_uuid(),
                source_type: "http_request".to_string(),
                confidence: Confidence::new(0.9).unwrap(),
            },
            1 => FactType::Vulnerability {
                cwe_id: Some(format!("CWE-{}", 79 + (i % 10))),
                owasp_category: Some("A03:2021".to_string()),
                severity: Severity::Critical,
                cvss_score: Some(9.0),
                description: format!("Vulnerability {}", i),
                confidence: Confidence::new(0.9).unwrap(),
            },
            2 => FactType::Function {
                name: FunctionName(("func_".to_string() + &i.to_string()).into()),
                complexity: 5 + (i % 10) as u32,
                lines_of_code: 20 + (i % 50) as u32,
            },
            _ => FactType::TaintSink {
                func: FunctionName(("sink_".to_string() + &i.to_string()).into()),
                consumes_flow: FlowId::new_uuid(),
                category: "SQL_INJECTION".to_string(),
                severity: Severity::Major,
            },
        };

        let file = if i % 3 == 0 {
            "src/main.rs"
        } else if i % 3 == 1 {
            "src/vuln.rs"
        } else {
            "src/utils.rs"
        };

        let fact = Fact {
            id: FactId::new(),
            fact_type,
            location: SourceLocation {
                file: ProjectPath::new(PathBuf::from(file)),
                start_line: LineNumber::new(1 + (i % 1000) as u32).unwrap(),
                start_column: Some(ColumnNumber::new(1).unwrap()),
                end_line: LineNumber::new(1 + (i % 1000) as u32).unwrap(),
                end_column: Some(ColumnNumber::new(30).unwrap()),
            },
            provenance: Provenance {
                extractor: ExtractorId::TreeSitter,
                version: "1.0.0".to_string(),
                confidence: Confidence::new(0.9).unwrap(),
                extracted_at: Utc::now(),
            },
        };

        facts.push(fact);
    }

    facts
}

fn benchmark_query_planner_creation(c: &mut Criterion) {
    let sizes = [100, 1000, 10000];

    let mut group = c.benchmark_group("query_planner_creation");

    for size in sizes {
        let facts = generate_large_fact_dataset(size);
        let facts_refs: Vec<&Fact> = facts.iter().collect();

        group.bench_with_input(BenchmarkId::new("create_planner", size), &size, |b, _| {
            b.iter(|| {
                let stats = IndexStatistics::compute(black_box(&facts_refs));
                let planner = QueryPlanner::with_config(black_box(stats), PlannerConfig::default());
                black_box(planner);
            });
        });
    }

    group.finish();
}

fn benchmark_type_index_queries(c: &mut Criterion) {
    let sizes = [100, 1000, 10000];

    let mut group = c.benchmark_group("type_index_queries");

    for size in sizes {
        let facts = generate_large_fact_dataset(size);
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        // Create query for taint sources
        let query = Query::ByType(FactType::TaintSource {
            var: VariableName("test".into()),
            flow_id: FlowId::new_uuid(),
            source_type: "test".to_string(),
            confidence: Confidence::new(0.9).unwrap(),
        });

        group.bench_with_input(BenchmarkId::new("by_type", size), &size, |b, _| {
            b.iter(|| {
                let plan = planner.plan(black_box(&query));
                black_box(plan);
            });
        });
    }

    group.finish();
}

fn benchmark_spatial_queries(c: &mut Criterion) {
    let sizes = [100, 1000, 10000];

    let mut group = c.benchmark_group("spatial_queries");

    for size in sizes {
        let facts = generate_large_fact_dataset(size);
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        // Create query for specific file
        let query = Query::ByFile(ProjectPath::new(PathBuf::from("src/main.rs")));

        group.bench_with_input(BenchmarkId::new("by_file", size), &size, |b, _| {
            b.iter(|| {
                let plan = planner.plan(black_box(&query));
                black_box(plan);
            });
        });

        // Create line range query
        let line_query = Query::ByLineRange {
            file: ProjectPath::new(PathBuf::from("src/main.rs")),
            start: LineNumber::new(10).unwrap(),
            end: LineNumber::new(100).unwrap(),
        };

        group.bench_with_input(BenchmarkId::new("by_line_range", size), &size, |b, _| {
            b.iter(|| {
                let plan = planner.plan(black_box(&line_query));
                black_box(plan);
            });
        });
    }

    group.finish();
}

fn benchmark_flow_queries(c: &mut Criterion) {
    let sizes = [100, 1000, 10000];

    let mut group = c.benchmark_group("flow_queries");

    for size in sizes {
        let facts = generate_large_fact_dataset(size);
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        // Create query for flow
        let flow_id = FlowId::new_uuid();
        let query = Query::ByFlow(flow_id);

        group.bench_with_input(BenchmarkId::new("by_flow", size), &size, |b, _| {
            b.iter(|| {
                let plan = planner.plan(black_box(&query));
                black_box(plan);
            });
        });
    }

    group.finish();
}

fn benchmark_all_facts_query(c: &mut Criterion) {
    let sizes = [100, 1000, 10000];

    let mut group = c.benchmark_group("all_facts_query");

    for size in sizes {
        let facts = generate_large_fact_dataset(size);
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        let query = Query::All;

        group.bench_with_input(BenchmarkId::new("all_facts", size), &size, |b, _| {
            b.iter(|| {
                let plan = planner.plan(black_box(&query));
                black_box(plan);
            });
        });
    }

    group.finish();
}

fn benchmark_complex_queries(c: &mut Criterion) {
    let sizes = [100, 1000, 10000];

    let mut group = c.benchmark_group("complex_queries");

    for size in sizes {
        let facts = generate_large_fact_dataset(size);
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        // Create complex query with predicates
        let mut predicates = HashMap::new();
        predicates.insert("severity".to_string(), "Critical".to_string());
        predicates.insert("category".to_string(), "SQL_INJECTION".to_string());

        let query = Query::Complex {
            type_discriminant: FactTypeDiscriminant::Vulnerability,
            predicates,
        };

        group.bench_with_input(
            BenchmarkId::new("complex_with_predicates", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let plan = planner.plan(black_box(&query));
                    black_box(plan);
                });
            },
        );
    }

    group.finish();
}

fn benchmark_cache_performance(c: &mut Criterion) {
    let size = 1000;
    let facts = generate_large_fact_dataset(size);
    let facts_refs: Vec<&Fact> = facts.iter().collect();
    let stats = IndexStatistics::compute(&facts_refs);

    let mut group = c.benchmark_group("cache_performance");

    // Without cache
    let mut planner_no_cache = QueryPlanner::with_config(
        stats.clone(),
        PlannerConfig {
            selective_threshold: 0.1,
            parallel_threshold: 10000,
            max_cache_entries: 0, // No cache
        },
    );

    let query = Query::ByType(FactType::TaintSource {
        var: VariableName("test".into()),
        flow_id: FlowId::new_uuid(),
        source_type: "test".to_string(),
        confidence: Confidence::new(0.9).unwrap(),
    });

    group.bench_function("without_cache", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let plan = planner_no_cache.plan(black_box(&query));
                black_box(plan);
            }
        });
    });

    // With cache
    let mut planner_with_cache = QueryPlanner::with_config(
        stats,
        PlannerConfig {
            selective_threshold: 0.1,
            parallel_threshold: 10000,
            max_cache_entries: 1000,
        },
    );

    group.bench_function("with_cache", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let plan = planner_with_cache.plan(black_box(&query));
                black_box(plan);
            }
        });
    });

    group.finish();
}

fn benchmark_cost_estimation(c: &mut Criterion) {
    let size = 10000;
    let facts = generate_large_fact_dataset(size);
    let facts_refs: Vec<&Fact> = facts.iter().collect();
    let stats = IndexStatistics::compute(&facts_refs);

    let mut group = c.benchmark_group("cost_estimation");

    // Test cost estimation for different types
    let test_types = vec![
        FactType::TaintSource {
            var: VariableName("test".into()),
            flow_id: FlowId::new_uuid(),
            source_type: "test".to_string(),
            confidence: Confidence::new(0.9).unwrap(),
        },
        FactType::Vulnerability {
            cwe_id: Some("CWE-79".to_string()),
            owasp_category: Some("A03:2021".to_string()),
            severity: Severity::Critical,
            cvss_score: Some(9.8),
            description: "Test".into(),
            confidence: Confidence::new(0.9).unwrap(),
        },
        FactType::Function {
            name: FunctionName("test".into()),
            complexity: 10,
            lines_of_code: 50,
        },
    ];

    for (i, fact_type) in test_types.into_iter().enumerate() {
        group.bench_with_input(BenchmarkId::new("estimate_cost", i), &i, |b, _| {
            b.iter(|| {
                // Simulate cost estimation
                let _cost = black_box(fact_type.discriminant());
            });
        });
    }

    group.finish();
}

fn benchmark_execution_strategy_selection(c: &mut Criterion) {
    let size = 10000;
    let facts = generate_large_fact_dataset(size);
    let facts_refs: Vec<&Fact> = facts.iter().collect();
    let stats = IndexStatistics::compute(&facts_refs);
    let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

    let mut group = c.benchmark_group("execution_strategy_selection");

    // Different query types that should select different strategies
    let queries = vec![
        (
            "TypeIndex",
            Query::ByType(FactType::TaintSource {
                var: VariableName("test".into()),
                flow_id: FlowId::new_uuid(),
                source_type: "test".to_string(),
                confidence: Confidence::new(0.9).unwrap(),
            }),
        ),
        (
            "SpatialIndex",
            Query::ByFile(ProjectPath::new(PathBuf::from("src/main.rs"))),
        ),
        ("FlowIndex", Query::ByFlow(FlowId::new_uuid())),
        ("FullScan", Query::All),
    ];

    for (name, query) in queries {
        group.bench_function(name, |b| {
            b.iter(|| {
                let plan = planner.plan(black_box(&query));
                black_box(plan);
            });
        });
    }

    group.finish();
}

fn benchmark_statistics_computation(c: &mut Criterion) {
    let sizes = [100, 1000, 10000];

    let mut group = c.benchmark_group("statistics_computation");

    for size in sizes {
        let facts = generate_large_fact_dataset(size);
        let facts_refs: Vec<&Fact> = facts.iter().collect();

        group.bench_with_input(BenchmarkId::new("compute_stats", size), &size, |b, _| {
            b.iter(|| {
                let stats = IndexStatistics::compute(black_box(&facts_refs));
                black_box(stats);
            });
        });
    }

    group.finish();
}

fn benchmark_query_planning_throughput(c: &mut Criterion) {
    let size = 10000;
    let facts = generate_large_fact_dataset(size);
    let facts_refs: Vec<&Fact> = facts.iter().collect();
    let stats = IndexStatistics::compute(&facts_refs);
    let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

    let mut group = c.benchmark_group("query_planning_throughput");

    // Create a mix of different query types
    let mut queries = Vec::new();
    for i in 0..100 {
        queries.push(match i % 4 {
            0 => Query::ByType(FactType::TaintSource {
                var: VariableName(format!("var_{}", i).into()),
                flow_id: FlowId::new_uuid(),
                source_type: "test".to_string(),
                confidence: Confidence::new(0.9).unwrap(),
            }),
            1 => Query::ByType(FactType::Vulnerability {
                cwe_id: Some(format!("CWE-{}", 79 + (i % 10))),
                owasp_category: Some("A03:2021".to_string()),
                severity: Severity::Critical,
                cvss_score: Some(9.0),
                description: format!("Vuln {}", i),
                confidence: Confidence::new(0.9).unwrap(),
            }),
            2 => Query::ByFile(ProjectPath::new(PathBuf::from("src/main.rs"))),
            _ => Query::All,
        });
    }

    group.bench_function("mixed_queries", |b| {
        b.iter(|| {
            for query in &queries {
                let plan = planner.plan(black_box(query));
                black_box(plan);
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_query_planner_creation,
    benchmark_type_index_queries,
    benchmark_spatial_queries,
    benchmark_flow_queries,
    benchmark_all_facts_query,
    benchmark_complex_queries,
    benchmark_cache_performance,
    benchmark_cost_estimation,
    benchmark_execution_strategy_selection,
    benchmark_statistics_computation,
    benchmark_query_planning_throughput
);

criterion_main!(benches);
