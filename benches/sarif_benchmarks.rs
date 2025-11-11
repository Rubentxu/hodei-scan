//! Benchmarks for SARIF Adapter performance
//!
//! Run with: cargo bench --bench sarif_benchmarks

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use hodei_engine::extractor::sarif_adapter::{SarifAdapter, SarifConfig};
use std::path::PathBuf;

const SMALL_SARIF: &str = r#"{
  "version": "2.1.0",
  "runs": [{
    "tool": {"driver": {"name": "test"}},
    "results": [{
      "ruleId": "R001",
      "level": "error",
      "message": {"text": "Test"},
      "locations": [{
        "physicalLocation": {
          "artifactLocation": {"uri": "test.py"},
          "region": {"startLine": 10, "startColumn": 5}
        }
      }]
    }]
  }]
}"#;

fn generate_large_sarif(num_results: usize) -> String {
    let mut results = Vec::new();
    for i in 0..num_results {
        results.push(format!(
            r#"{{
              "ruleId": "R{:03}",
              "level": "error",
              "message": {{"text": "Finding {}"}},
              "locations": [{{
                "physicalLocation": {{
                  "artifactLocation": {{"uri": "test{}.py"}},
                  "region": {{"startLine": {}, "startColumn": 5}}
                }}
              }}]
            }}"#,
            i % 100,
            i,
            i,
            10 + (i % 1000)
        ));
    }

    format!(
        r#"{{
  "version": "2.1.0",
  "runs": [{{
    "tool": {{"driver": {{"name": "test"}}}},
    "results": [{}]
  }}]
}}"#,
        results.join(",")
    )
}

fn bench_parse_small_sarif(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let adapter = SarifAdapter::default();

    c.bench_function("parse_small_sarif", |b| {
        b.to_async(&rt).iter(|| async {
            let result = adapter.parse_str(black_box(SMALL_SARIF)).await;
            black_box(result)
        });
    });
}

fn bench_parse_varying_sizes(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let adapter = SarifAdapter::default();

    let mut group = c.benchmark_group("parse_varying_sizes");

    for size in [10, 50, 100, 500, 1000].iter() {
        let sarif_json = generate_large_sarif(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.to_async(&rt).iter(|| async {
                let result = adapter.parse_str(black_box(&sarif_json)).await;
                black_box(result)
            });
        });
    }

    group.finish();
}

fn bench_parse_from_file(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let adapter = SarifAdapter::default();

    // Use test data file
    let test_file =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/sarif/codeql-sample.sarif");

    if test_file.exists() {
        c.bench_function("parse_from_file", |b| {
            b.to_async(&rt).iter(|| async {
                let result = adapter.parse_file(black_box(&test_file)).await;
                black_box(result)
            });
        });
    }
}

fn bench_adapter_creation(c: &mut Criterion) {
    c.bench_function("adapter_creation", |b| {
        b.iter(|| {
            let adapter = SarifAdapter::default();
            black_box(adapter)
        });
    });
}

fn bench_config_with_limits(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let large_sarif = generate_large_sarif(1000);

    let mut group = c.benchmark_group("config_with_limits");

    for max_results in [Some(100), Some(500), Some(10000), None].iter() {
        let config = SarifConfig {
            max_results: *max_results,
        };
        let adapter = SarifAdapter::new(config);

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{:?}", max_results)),
            max_results,
            |b, _| {
                b.to_async(&rt).iter(|| async {
                    let result = adapter.parse_str(black_box(&large_sarif)).await;
                    black_box(result)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_small_sarif,
    bench_parse_varying_sizes,
    bench_parse_from_file,
    bench_adapter_creation,
    bench_config_with_limits
);
criterion_main!(benches);
