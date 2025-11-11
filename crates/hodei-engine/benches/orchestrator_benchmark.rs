//! Benchmarks for ExtractorOrchestrator performance
//!
//! These benchmarks measure:
//! - Concurrent execution throughput
//! - Timeout handling overhead
//! - Resource tracking performance
//! - Message serialization performance

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hodei_engine::extractor::{
    ExtractorConfig, ExtractorDef, ExtractorOrchestrator, OrchestratorError,
};
use std::time::Duration;
use tokio::runtime::Runtime;

fn bench_orchestrator_creation(c: &mut Criterion) {
    c.bench_function("orchestrator_creation", |b| {
        b.iter(|| {
            let config = ExtractorConfig {
                extractors: vec![ExtractorDef {
                    name: "test".to_string(),
                    command: "echo".to_string(),
                    args: vec!["test".to_string()],
                    timeout: None,
                    env: None,
                }],
                max_concurrent: Some(4),
                default_timeout: Some(Duration::from_secs(30)),
            };
            black_box(ExtractorOrchestrator::new(config));
        });
    });
}

fn bench_request_serialization(c: &mut Criterion) {
    use hodei_engine::extractor::protocol::{ExtractorMessage, ExtractorRequest, PROTOCOL_VERSION};

    c.bench_function("request_serialization", |b| {
        b.iter(|| {
            let request = ExtractorRequest {
                request_id: 12345,
                project_path: "/path/to/project".to_string(),
                language: "rust".to_string(),
                config: r#"{"rule": "test"}"#.to_string(),
                timeout_ms: 30000,
                version: PROTOCOL_VERSION.to_string(),
            };

            let message = ExtractorMessage::Request(request);
            black_box(message.serialize().unwrap());
        });
    });
}

fn bench_response_deserialization(c: &mut Criterion) {
    use hodei_engine::extractor::protocol::{ExtractorMessage, ExtractorResponse};

    let rt = Runtime::new().unwrap();

    let response = ExtractorResponse {
        request_id: 12345,
        success: true,
        ir: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        metadata: r#"{"version": "1.0", "stats": {"files": 100}}"#.to_string(),
        processing_time_ms: 150,
    };

    let message = ExtractorMessage::Response(response);
    let serialized = message.serialize().unwrap();

    c.bench_function("response_deserialization", |b| {
        b.iter(|| {
            black_box(ExtractorMessage::deserialize(&serialized).unwrap());
        });
    });

    // Also benchmark the runtime performance
    let mut group = c.benchmark_group("async_execution");
    group.bench_function("empty_execution", |b| {
        let rt = Runtime::new().unwrap();
        b.to_async(&rt).iter(|| async {
            let config = ExtractorConfig {
                extractors: vec![],
                max_concurrent: Some(4),
                default_timeout: Some(Duration::from_secs(30)),
            };
            let orchestrator = ExtractorOrchestrator::new(config);
            black_box(orchestrator.execute_all("/tmp", "rust").await.unwrap());
        });
    });
    group.finish();
}

fn bench_concurrency_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrency_scaling");
    for num_extractors in [1, 5, 10, 20] {
        group.bench_with_input(
            criterion::BenchmarkId::new("execute_all", format!("{} extractors", num_extractors)),
            &num_extractors,
            |b, &num_extractors| {
                b.to_async(&rt).iter(|| async {
                    let extractors: Vec<ExtractorDef> = (0..num_extractors)
                        .map(|i| ExtractorDef {
                            name: format!("extractor-{}", i),
                            command: "echo".to_string(),
                            args: vec!["test".to_string()],
                            timeout: Some(Duration::from_secs(1)),
                            env: None,
                        })
                        .collect();

                    let config = ExtractorConfig {
                        extractors,
                        max_concurrent: Some(4),
                        default_timeout: Some(Duration::from_secs(5)),
                    };

                    let orchestrator = ExtractorOrchestrator::new(config);
                    black_box(orchestrator.execute_all("/tmp", "rust").await);
                });
            },
        );
    }
    group.finish();
}

fn bench_timeout_handling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("timeout_handling");
    group.bench_function("timeout_execution", |b| {
        b.to_async(&rt).iter(|| async {
            let extractor = ExtractorDef {
                name: "sleep".to_string(),
                command: "sleep".to_string(),
                args: vec!["0.1".to_string()],
                timeout: Some(Duration::from_millis(50)), // Will timeout
                env: None,
            };

            let orchestrator = ExtractorOrchestrator::default();
            black_box(
                orchestrator
                    .execute_with_timeout(&extractor, Duration::from_millis(50))
                    .await,
            );
        });
    });
    group.finish();
}

fn bench_resource_tracking(c: &mut Criterion) {
    c.bench_function("resource_stats", |b| {
        let rt = Runtime::new().unwrap();
        let config = ExtractorConfig {
            extractors: vec![ExtractorDef {
                name: "test".to_string(),
                command: "echo".to_string(),
                args: vec!["test".to_string()],
                timeout: Some(Duration::from_secs(1)),
                env: None,
            }],
            max_concurrent: Some(1),
            default_timeout: None,
        };
        let orchestrator = ExtractorOrchestrator::new(config);

        b.to_async(&rt).iter(|| async {
            black_box(orchestrator.get_resource_stats().await);
        });
    });
}

fn bench_large_message_handling(c: &mut Criterion) {
    use hodei_engine::extractor::protocol::{ExtractorMessage, ExtractorResponse};

    let large_ir_data = vec![42u8; 10000]; // 10KB of data
    let response = ExtractorResponse {
        request_id: 12345,
        success: true,
        ir: large_ir_data,
        metadata: r#"{"version": "1.0", "large_field": "value"}"#.repeat(100),
        processing_time_ms: 150,
    };

    let message = ExtractorMessage::Response(response);
    let serialized = message.serialize().unwrap();

    c.bench_function("large_message_serialization", |b| {
        b.iter(|| {
            let message = ExtractorMessage::Response(ExtractorResponse {
                request_id: 12345,
                success: true,
                ir: vec![42u8; 10000],
                metadata: r#"{"version": "1.0"}"#.repeat(100),
                processing_time_ms: 150,
            });
            black_box(message.serialize().unwrap());
        });
    });

    c.bench_function("large_message_deserialization", |b| {
        b.iter(|| {
            black_box(ExtractorMessage::deserialize(&serialized).unwrap());
        });
    });
}

fn bench_id_generation(c: &mut Criterion) {
    c.bench_function("request_id_generation", |b| {
        b.iter(|| {
            black_box(super::super::extractor::orchestrator::generate_request_id());
        });
    });
}

criterion_group!(
    benches,
    bench_orchestrator_creation,
    bench_request_serialization,
    bench_response_deserialization,
    bench_concurrency_scaling,
    bench_timeout_handling,
    bench_resource_tracking,
    bench_large_message_handling,
    bench_id_generation
);

criterion_main!(benches);
