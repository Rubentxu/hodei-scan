//! IR Benchmarks
//!
//! This module contains performance benchmarks for IR operations.

use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn create_ir_benchmark(c: &mut Criterion) {
    c.bench_function("create_ir", |b| {
        b.iter(|| {
            let analysis_id = hodei_scan::core::AnalysisId::new();
            let metadata = hodei_scan::ir::AnalysisMetadata {
                language: hodei_scan::core::Language::JavaScript,
                project_root: "/test".to_string(),
                files_analyzed: vec!["test.js".to_string()],
                lines_of_code: 100,
                config: std::collections::HashMap::new(),
            };
            hodei_scan::ir::IntermediateRepresentation::new(analysis_id, metadata);
        })
    });
}

fn serialization_benchmark(c: &mut Criterion) {
    let analysis_id = hodei_scan::core::AnalysisId::new();
    let metadata = hodei_scan::ir::AnalysisMetadata {
        language: hodei_scan::core::Language::JavaScript,
        project_root: "/test".to_string(),
        files_analyzed: vec!["test.js".to_string()],
        lines_of_code: 100,
        config: std::collections::HashMap::new(),
    };
    let ir = hodei_scan::ir::IntermediateRepresentation::new(analysis_id, metadata);

    c.bench_function("json_serialize", |b| {
        b.iter(|| hodei_scan::ir::SerializationFormat::Json.serialize(black_box(&ir)))
    });
}

criterion_group!(benches, create_ir_benchmark, serialization_benchmark);
criterion_main!(benches);
