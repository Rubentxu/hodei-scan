//! Benchmarks for Ruff Adapter
//!
//! These benchmarks measure the performance of Ruff JSON parsing and
//! IR conversion across different scenarios and file sizes.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hodei_engine::extractor::ruff_adapter::{RuffAdapter, RuffConfig};
use std::path::PathBuf;

fn generate_ruff_json(num_results: usize) -> String {
    let mut results = Vec::new();

    for i in 0..num_results {
        let file_num = i / 10 + 1;
        let line = (i % 100) + 1;

        results.push(format!(
            r#"{{
    "filename": "src/module{file_num}.py",
    "code": "{code}",
    "rule": "{code}",
    "message": "{message}",
    "severity": "{severity}",
    "line": {line},
    "column": {column}
}}"#,
            file_num = file_num,
            code = if i % 3 == 0 {
                "E501"
            } else if i % 3 == 1 {
                "F401"
            } else {
                "E302"
            },
            message = if i % 3 == 0 {
                "Line too long"
            } else if i % 3 == 1 {
                "Unused import"
            } else {
                "Expected blank lines"
            },
            severity = if i % 5 == 0 {
                "error"
            } else if i % 5 == 1 {
                "warning"
            } else {
                "info"
            },
            line = line,
            column = (i % 80) + 1
        ));
    }

    format!("[{}]", results.join(","))
}

fn bench_parse_small_sarif(c: &mut Criterion) {
    let adapter = RuffAdapter::new();
    let project_root = PathBuf::from("/project");

    c.bench_function("ruff_parse_small", |b| {
        b.iter(|| {
            let ruff_json = generate_ruff_json(black_box(10));
            let result = adapter.parse_str(&ruff_json, &project_root).unwrap();
            black_box(result);
        })
    });
}

fn bench_parse_medium_sarif(c: &mut Criterion) {
    let adapter = RuffAdapter::new();
    let project_root = PathBuf::from("/project");

    c.bench_function("ruff_parse_medium", |b| {
        b.iter(|| {
            let ruff_json = generate_ruff_json(black_box(100));
            let result = adapter.parse_str(&ruff_json, &project_root).unwrap();
            black_box(result);
        })
    });
}

fn bench_parse_large_sarif(c: &mut Criterion) {
    let adapter = RuffAdapter::new();
    let project_root = PathBuf::from("/project");

    c.bench_function("ruff_parse_large", |b| {
        b.iter(|| {
            let ruff_json = generate_ruff_json(black_box(500));
            let result = adapter.parse_str(&ruff_json, &project_root).unwrap();
            black_box(result);
        })
    });
}

fn bench_parse_very_large_sarif(c: &mut Criterion) {
    let adapter = RuffAdapter::new();
    let project_root = PathBuf::from("/project");

    c.bench_function("ruff_parse_very_large", |b| {
        b.iter(|| {
            let ruff_json = generate_ruff_json(black_box(1000));
            let result = adapter.parse_str(&ruff_json, &project_root).unwrap();
            black_box(result);
        })
    });
}

fn bench_varying_sizes(c: &mut Criterion) {
    let adapter = RuffAdapter::new();
    let project_root = PathBuf::from("/project");

    let sizes = vec![10, 50, 100, 250, 500, 1000];

    c.bench_function("ruff_varying_sizes", |b| {
        b.iter(|| {
            for size in &sizes {
                let ruff_json = generate_ruff_json(black_box(*size));
                let result = adapter.parse_str(&ruff_json, &project_root).unwrap();
                black_box(result);
            }
        })
    });
}

fn bench_with_config(c: &mut Criterion) {
    let config = RuffConfig {
        max_parallel: 8,
        include_fixes: true,
    };
    let adapter = RuffAdapter::with_config(config);
    let project_root = PathBuf::from("/project");

    c.bench_function("ruff_parse_with_config", |b| {
        b.iter(|| {
            let ruff_json = generate_ruff_json(black_box(100));
            let result = adapter.parse_str(&ruff_json, &project_root).unwrap();
            black_box(result);
        })
    });
}

fn bench_severity_mapping(c: &mut Criterion) {
    let adapter = RuffAdapter::new();
    let project_root = PathBuf::from("/project");

    // JSON with only errors
    let error_json = generate_ruff_json(100);

    c.bench_function("ruff_severity_mapping", |b| {
        b.iter(|| {
            let result = adapter.parse_str(&error_json, &project_root).unwrap();
            black_box(result);
        })
    });
}

fn bench_path_handling(c: &mut Criterion) {
    let adapter = RuffAdapter::new();

    c.bench_function("ruff_path_handling", |b| {
        b.iter(|| {
            // Test with absolute paths
            let ruff_json = generate_ruff_json(100).replace(
                "\"filename\": \"src/",
                "\"filename\": \"/home/user/project/src/",
            );
            let project_root = PathBuf::from("/home/user/project");
            let result = adapter.parse_str(&ruff_json, &project_root).unwrap();
            black_box(result);
        })
    });
}

fn bench_mixed_severities(c: &mut Criterion) {
    let adapter = RuffAdapter::new();
    let project_root = PathBuf::from("/project");

    c.bench_function("ruff_mixed_severities", |b| {
        b.iter(|| {
            let ruff_json = generate_ruff_json(black_box(200));
            let result = adapter.parse_str(&ruff_json, &project_root).unwrap();
            black_box(result);
        })
    });
}

fn bench_fact_creation(c: &mut Criterion) {
    let adapter = RuffAdapter::new();
    let project_root = PathBuf::from("/project");

    c.bench_function("ruff_fact_creation", |b| {
        b.iter(|| {
            let ruff_json = generate_ruff_json(black_box(150));
            let result = adapter.parse_str(&ruff_json, &project_root).unwrap();
            // Verify facts were created
            assert!(!result.is_empty());
            black_box(result);
        })
    });
}

criterion_group!(
    ruff_benches,
    bench_parse_small_sarif,
    bench_parse_medium_sarif,
    bench_parse_large_sarif,
    bench_parse_very_large_sarif,
    bench_varying_sizes,
    bench_with_config,
    bench_severity_mapping,
    bench_path_handling,
    bench_mixed_severities,
    bench_fact_creation
);

criterion_main!(ruff_benches);
