//! Benchmarks for IndexedFactStore
//!
//! These benchmarks test the performance of different index types according to EPIC-04:
//! - TypeIndex: O(1) lookup should be <10μs for 1M facts
//! - SpatialIndex: R-tree query should be <500μs for 100k facts
//! - FlowIndex: Reachability should be <2ms for 10k flows

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hodei_engine::store::{FlowIndex, IndexedFactStore, SpatialIndex, TypeIndex};
use hodei_ir::*;

fn generate_facts(count: usize) -> Vec<Fact> {
    (0..count)
        .map(|i| {
            let flow_id = FlowId::new_uuid();
            let provenance = Provenance::new(
                ExtractorId::new("test".to_string()),
                "1.0.0".to_string(),
                Confidence::new(0.8).unwrap(),
            );

            Fact {
                id: FactId::new(),
                fact_type: if i % 10 == 0 {
                    FactType::TaintSource {
                        var: VariableName(format!("var{}", i)),
                        flow_id,
                        source_type: "http".to_string(),
                        confidence: Confidence::new(0.8).unwrap(),
                    }
                } else {
                    FactType::Function {
                        name: format!("func{}", i),
                        complexity: i as u32 % 10,
                        lines_of_code: (i as u32 % 100) + 1,
                    }
                },
                location: SourceLocation {
                    file: ProjectPath::new(std::path::PathBuf::from(format!("file{}.rs", i % 10))),
                    start_line: LineNumber::new((i % 1000) as u32).unwrap(),
                    end_line: LineNumber::new((i % 1000 + 10) as u32).unwrap(),
                    start_column: Some(ColumnNumber::new(0).unwrap()),
                    end_column: Some(ColumnNumber::new(100).unwrap()),
                },
                provenance,
            }
        })
        .collect()
}

fn bench_type_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("TypeIndex");

    for size in [1_000, 10_000, 100_000, 1_000_000] {
        let facts = generate_facts(size);
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let index = TypeIndex::build(&facts_refs);

        let taint_source = FactType::TaintSource {
            var: VariableName("test".to_string()),
            flow_id: FlowId::new_uuid(),
            source_type: "http".to_string(),
            confidence: Confidence::new(0.8).unwrap(),
        };

        group.bench_with_input(
            criterion::BenchmarkId::new("lookup", size),
            &(&index, &taint_source),
            |b, (index, fact_type)| b.iter(|| black_box(index.get(fact_type))),
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("cardinality", size),
            &(&index, &taint_source),
            |b, (index, fact_type)| b.iter(|| black_box(index.cardinality(fact_type))),
        );
    }
    group.finish();
}

fn bench_spatial_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("SpatialIndex");

    for size in [1_000, 10_000, 50_000, 100_000] {
        let facts = generate_facts(size);
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let index = SpatialIndex::build(&facts_refs);

        group.bench_with_input(
            criterion::BenchmarkId::new("query_file_range", size),
            &index,
            |b, index| b.iter(|| black_box(index.query("file0.rs", 0, 100))),
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("by_file", size),
            &index,
            |b, index| b.iter(|| black_box(index.by_file("file0.rs"))),
        );
    }
    group.finish();
}

fn bench_flow_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("FlowIndex");

    for size in [1_000, 5_000, 10_000] {
        let facts = generate_facts(size);
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let index = FlowIndex::build(&facts_refs);

        // Create a test fact with a flow
        let first_taint = facts_refs
            .iter()
            .find(|f| matches!(f.fact_type, FactType::TaintSource { .. }))
            .unwrap();

        group.bench_with_input(
            criterion::BenchmarkId::new("reachable_from", size),
            &(index.clone(), first_taint.id),
            |b, (index, fact_id)| b.iter(|| black_box(index.reachable_from(*fact_id))),
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("nodes", size),
            &index,
            |b, index| b.iter(|| black_box(index.nodes())),
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("edge_count", size),
            &index,
            |b, index| b.iter(|| black_box(index.edge_count())),
        );
    }
    group.finish();
}

fn bench_indexed_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("IndexedFactStore");

    for size in [1_000, 10_000, 100_000] {
        let facts = generate_facts(size);
        let store = IndexedFactStore::new(facts);

        let taint_source = FactType::TaintSource {
            var: VariableName("test".to_string()),
            flow_id: FlowId::new_uuid(),
            source_type: "http".to_string(),
            confidence: Confidence::new(0.8).unwrap(),
        };

        group.bench_with_input(
            criterion::BenchmarkId::new("by_type", size),
            &(&store, &taint_source),
            |b, (store, fact_type)| b.iter(|| black_box(store.by_type(fact_type))),
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("by_location", size),
            &store,
            |b, store| b.iter(|| black_box(store.by_location("file0.rs", 0, 100))),
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("by_file", size),
            &store,
            |b, store| b.iter(|| black_box(store.by_file("file0.rs"))),
        );

        group.bench_with_input(
            criterion::BenchmarkId::new("fact_count", size),
            &store,
            |b, store| b.iter(|| black_box(store.fact_count())),
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_type_index,
    bench_spatial_index,
    bench_flow_index,
    bench_indexed_store
);
criterion_main!(benches);
