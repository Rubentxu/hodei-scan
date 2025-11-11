//! Benchmarks for Cache Manager
//!
//! These benchmarks measure the performance of the cache system including
//! store, retrieve, and batch operations.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hodei_cli::analysis::cache_manager::*;
use hodei_ir::{Fact, FactId, FactType, Provenance, Severity};
use std::path::PathBuf;

fn create_test_fact() -> Fact {
    use hodei_ir::{Confidence, ExtractorId, LineNumber, ProjectPath, SourceLocation};

    let provenance = Provenance::new(
        ExtractorId::Custom,
        "1.0".to_string(),
        Confidence::new(0.5).unwrap(),
    );

    let location = SourceLocation::new(
        ProjectPath::new(PathBuf::from("test.rs")),
        LineNumber::new(1).unwrap(),
        None,
        LineNumber::new(1).unwrap(),
        None,
    );

    Fact::new(
        FactType::CodeSmell {
            smell_type: "TODO".to_string(),
            severity: Severity::Minor,
            message: "Test fact".to_string(),
        },
        location,
        provenance,
    )
}

fn create_test_facts(count: usize) -> Vec<Fact> {
    use hodei_ir::{Confidence, ExtractorId, LineNumber, ProjectPath, SourceLocation};

    let provenance = Provenance::new(
        ExtractorId::Custom,
        "1.0".to_string(),
        Confidence::new(0.5).unwrap(),
    );

    let location = SourceLocation::new(
        ProjectPath::new(PathBuf::from("test.rs")),
        LineNumber::new(1).unwrap(),
        None,
        LineNumber::new(1).unwrap(),
        None,
    );

    (0..count)
        .map(|i| {
            Fact::new(
                FactType::CodeSmell {
                    smell_type: format!("CodeSmell{}", i),
                    severity: Severity::Minor,
                    message: format!("Test fact {}", i),
                },
                location.clone(),
                provenance.clone(),
            )
        })
        .collect()
}

fn bench_cache_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("CacheStore");

    let cache = CacheManager::new_temp().unwrap();
    let facts = create_test_facts(10);

    for file_count in [1, 10, 100] {
        group.bench_function(format!("store_{}_files", file_count), |b| {
            b.iter(|| {
                for i in 0..file_count {
                    let path = PathBuf::from(format!("src/file_{}.rs", i));
                    let hash = format!("hash_{}", i);
                    let key = CacheKey::from_file(&path, hash);

                    cache.store_facts(&key, &facts).unwrap();
                    black_box(&cache);
                }
            })
        });
    }

    group.finish();
}

fn bench_cache_retrieve(c: &mut Criterion) {
    let mut group = c.benchmark_group("CacheRetrieve");

    let mut cache = CacheManager::new_temp().unwrap();
    let facts = create_test_facts(10);

    // Pre-populate cache
    for i in 0..100 {
        let path = PathBuf::from(format!("src/file_{}.rs", i));
        let hash = format!("hash_{}", i);
        let key = CacheKey::from_file(&path, hash);
        cache.store_facts(&key, &facts).unwrap();
    }

    for file_count in [1, 10, 100] {
        group.bench_function(format!("retrieve_{}_files", file_count), |b| {
            b.iter(|| {
                for i in 0..file_count {
                    let path = PathBuf::from(format!("src/file_{}.rs", i));
                    let hash = format!("hash_{}", i);
                    let key = CacheKey::from_file(&path, hash);

                    let (retrieved_facts, is_hit) = cache.get_facts(&key).unwrap();
                    black_box((retrieved_facts, is_hit));
                }
            })
        });
    }

    group.finish();
}

fn bench_cache_hit_rate(c: &mut Criterion) {
    let mut group = c.benchmark_group("CacheHitRate");

    let mut cache = CacheManager::new_temp().unwrap();
    let facts = create_test_facts(10);

    // Pre-populate cache with 100 entries
    for i in 0..100 {
        let path = PathBuf::from(format!("src/file_{}.rs", i));
        let hash = format!("hash_{}", i);
        let key = CacheKey::from_file(&path, hash);
        cache.store_facts(&key, &facts).unwrap();
    }

    // Test 90% hit rate (90 hits, 10 misses)
    group.bench_function("hit_rate_90_percent", |b| {
        b.iter(|| {
            for i in 0..90 {
                let path = PathBuf::from(format!("src/file_{}.rs", i));
                let hash = format!("hash_{}", i);
                let key = CacheKey::from_file(&path, hash);
                let (_, is_hit) = cache.get_facts(&key).unwrap();
                assert!(is_hit);
            }

            for i in 100..110 {
                let path = PathBuf::from(format!("src/new_{}.rs", i));
                let hash = format!("hash_{}", i);
                let key = CacheKey::from_file(&path, hash);
                let (_, is_hit) = cache.get_facts(&key).unwrap();
                assert!(!is_hit);
            }
        })
    });

    // Test 10% hit rate (10 hits, 90 misses)
    group.bench_function("hit_rate_10_percent", |b| {
        b.iter(|| {
            for i in 0..10 {
                let path = PathBuf::from(format!("src/file_{}.rs", i));
                let hash = format!("hash_{}", i);
                let key = CacheKey::from_file(&path, hash);
                let (_, is_hit) = cache.get_facts(&key).unwrap();
                assert!(is_hit);
            }

            for i in 100..190 {
                let path = PathBuf::from(format!("src/new_{}.rs", i));
                let hash = format!("hash_{}", i);
                let key = CacheKey::from_file(&path, hash);
                let (_, is_hit) = cache.get_facts(&key).unwrap();
                assert!(!is_hit);
            }
        })
    });

    group.finish();
}

fn bench_cache_key_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("CacheKeySerialization");

    let keys: Vec<CacheKey> = (0..1000)
        .map(|i| CacheKey {
            path: PathBuf::from(format!("src/file_{}.rs", i)),
            file_hash: format!("hash_{}", i),
            modified_at: 1234567890 + i as u64,
        })
        .collect();

    group.bench_function("serialize_keys", |b| {
        b.iter(|| {
            for key in &keys {
                let bytes = key.to_bytes().unwrap();
                black_box(&bytes);
            }
        })
    });

    group.bench_function("deserialize_keys", |b| {
        // Pre-serialize all keys
        let serialized: Vec<Vec<u8>> = keys.iter().map(|k| k.to_bytes().unwrap()).collect();

        b.iter(|| {
            for bytes in &serialized {
                let key = CacheKey::from_bytes(&bytes).unwrap();
                black_box(&key);
            }
        })
    });

    group.finish();
}

fn bench_cache_value_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("CacheValueSerialization");

    for fact_count in [1, 10, 100] {
        let values: Vec<CacheValue> = (0..100)
            .map(|i| {
                let facts = create_test_facts(fact_count);
                CacheValue::new(facts, Some(3600))
            })
            .collect();

        group.bench_function(format!("serialize_values_{}_facts", fact_count), |b| {
            b.iter(|| {
                for value in &values {
                    let bytes = value.to_bytes().unwrap();
                    black_box(&bytes);
                }
            })
        });

        group.bench_function(format!("deserialize_values_{}_facts", fact_count), |b| {
            // Pre-serialize all values
            let serialized: Vec<Vec<u8>> = values.iter().map(|v| v.to_bytes().unwrap()).collect();

            b.iter(|| {
                for bytes in &serialized {
                    let value = CacheValue::from_bytes(&bytes).unwrap();
                    black_box(&value);
                }
            })
        });
    }

    group.finish();
}

fn bench_cache_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("CacheStats");

    let cache = CacheManager::new_temp().unwrap();
    let facts = create_test_facts(10);

    group.bench_function("update_stats", |b| {
        b.iter(|| {
            for i in 0..100 {
                let path = PathBuf::from(format!("src/file_{}.rs", i));
                let hash = format!("hash_{}", i);
                let key = CacheKey::from_file(&path, hash);

                cache.store_facts(&key, &facts).unwrap();

                // Get stats
                let stats = cache.get_stats();
                black_box(&stats);
            }
        })
    });

    group.finish();
}

fn bench_cache_cleanup(c: &mut Criterion) {
    let mut group = c.benchmark_group("CacheCleanup");

    // Create cache with 1000 entries
    let mut cache = CacheManager::new_temp().unwrap();
    let facts = create_test_facts(10);

    for i in 0..1000 {
        let path = PathBuf::from(format!("src/file_{}.rs", i));
        let hash = format!("hash_{}", i);
        let key = CacheKey::from_file(&path, hash);
        cache.store_facts(&key, &facts).unwrap();
    }

    // Create a new cache with short TTL
    let mut short_cache = CacheManager::with_config(CacheConfig {
        ttl_seconds: Some(0), // All entries expire immediately
        max_entries: None,
        write_buffer_size_mb: 64,
        max_write_buffers: 3,
        compression: true,
        cache_dir: tempfile::tempdir().unwrap().path().to_path_buf(),
    })
    .unwrap();

    for i in 0..1000 {
        let path = PathBuf::from(format!("src/file_{}.rs", i));
        let hash = format!("hash_{}", i);
        let key = CacheKey::from_file(&path, hash);
        short_cache.store_facts(&key, &facts).unwrap();
    }

    group.bench_function("cleanup_expired_entries", |b| {
        b.iter(|| {
            let removed = short_cache.cleanup_expired().unwrap();
            black_box(removed);
        })
    });

    group.bench_function("clear_all_entries", |b| {
        b.iter(|| {
            cache.clear().unwrap();
        })
    });

    group.finish();
}

fn bench_cache_configuration(c: &mut Criterion) {
    let mut group = c.benchmark_group("CacheConfiguration");

    let temp_dir = tempfile::tempdir().unwrap();

    // Test different cache configurations
    for (name, config) in [
        ("default", CacheConfig::default()),
        (
            "no_compression",
            CacheConfig {
                compression: false,
                ..Default::default()
            },
        ),
        (
            "no_ttl",
            CacheConfig {
                ttl_seconds: None,
                ..Default::default()
            },
        ),
    ] {
        group.bench_function(format!("create_cache_{}", name), |b| {
            b.iter(|| {
                let config = CacheConfig {
                    cache_dir: temp_dir.path().join(name).to_path_buf(),
                    ..config.clone()
                };
                let cache = CacheManager::with_config(config).unwrap();
                black_box(cache);
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_cache_store,
    bench_cache_retrieve,
    bench_cache_hit_rate,
    bench_cache_key_serialization,
    bench_cache_value_serialization,
    bench_cache_stats,
    bench_cache_cleanup,
    bench_cache_configuration
);
criterion_main!(benches);
