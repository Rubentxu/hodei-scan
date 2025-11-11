//! Benchmarks for EnumMap optimization in TypeIndex
//!
//! These benchmarks compare HashMap vs EnumMap performance
//! for fact type indexing.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use enum_map::{EnumMap, enum_map};
use std::collections::HashMap;

use hodei_ir::fact_type_index::FactTypeDiscriminant;

/// Simple benchmark comparing HashMap vs EnumMap
fn bench_hashmap_vs_enummap(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap_vs_enummap");

    // Test with different sizes
    for size in [100, 1000, 10000].iter() {
        // Prepare test data
        let items: Vec<(FactTypeDiscriminant, u32)> = (0..*size)
            .map(|i| {
                let discriminant = match i % 19 {
                    0 => FactTypeDiscriminant::TaintSource,
                    1 => FactTypeDiscriminant::TaintSink,
                    2 => FactTypeDiscriminant::Vulnerability,
                    3 => FactTypeDiscriminant::Function,
                    4 => FactTypeDiscriminant::Variable,
                    5 => FactTypeDiscriminant::Sanitization,
                    6 => FactTypeDiscriminant::UnsafeCall,
                    7 => FactTypeDiscriminant::CryptographicOperation,
                    8 => FactTypeDiscriminant::CodeSmell,
                    9 => FactTypeDiscriminant::ComplexityViolation,
                    10 => FactTypeDiscriminant::Dependency,
                    11 => FactTypeDiscriminant::DependencyVulnerability,
                    12 => FactTypeDiscriminant::License,
                    13 => FactTypeDiscriminant::UncoveredLine,
                    14 => FactTypeDiscriminant::LowTestCoverage,
                    15 => FactTypeDiscriminant::CoverageStats,
                    16 => FactTypeDiscriminant::VulnerableUncovered,
                    17 => FactTypeDiscriminant::SecurityTechnicalDebt,
                    _ => FactTypeDiscriminant::QualitySecurityCorrelation,
                };
                (discriminant, i as u32)
            })
            .collect();

        // HashMap benchmark
        group.bench_with_input(
            BenchmarkId::new("hashmap_insert", size),
            &items,
            |b, items| {
                b.iter(|| {
                    let mut map = HashMap::new();
                    for (k, v) in items {
                        map.insert(*k, *v);
                    }
                    black_box(map)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("hashmap_lookup", size),
            &items,
            |b, items| {
                let mut map = HashMap::new();
                for (k, v) in items {
                    map.insert(*k, *v);
                }

                b.iter(|| {
                    let mut sum = 0;
                    for (k, _) in items {
                        if let Some(v) = map.get(k) {
                            sum += *v;
                        }
                    }
                    black_box(sum);
                })
            },
        );

        // EnumMap benchmark
        group.bench_with_input(
            BenchmarkId::new("enummap_insert", size),
            &items,
            |b, items| {
                b.iter(|| {
                    let mut map: EnumMap<FactTypeDiscriminant, u32> = enum_map! { _ => 0 };
                    for (k, v) in items {
                        map[*k] = *v;
                    }
                    black_box(map)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("enummap_lookup", size),
            &items,
            |b, items| {
                let mut map: EnumMap<FactTypeDiscriminant, u32> = enum_map! { _ => 0 };
                for (k, v) in items {
                    map[*k] = *v;
                }

                b.iter(|| {
                    let mut sum = 0;
                    for (k, _) in items {
                        sum += map[*k];
                    }
                    black_box(sum);
                })
            },
        );
    }

    group.finish();
}

fn bench_iteration_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("iteration_performance");

    // Create a full EnumMap with some non-zero values
    let mut enummap: EnumMap<FactTypeDiscriminant, u32> = enum_map! { _ => 0 };
    for i in 0..19 {
        let discriminant = match i {
            0 => FactTypeDiscriminant::TaintSource,
            1 => FactTypeDiscriminant::TaintSink,
            2 => FactTypeDiscriminant::Vulnerability,
            3 => FactTypeDiscriminant::Function,
            4 => FactTypeDiscriminant::Variable,
            5 => FactTypeDiscriminant::Sanitization,
            6 => FactTypeDiscriminant::UnsafeCall,
            7 => FactTypeDiscriminant::CryptographicOperation,
            8 => FactTypeDiscriminant::CodeSmell,
            9 => FactTypeDiscriminant::ComplexityViolation,
            10 => FactTypeDiscriminant::Dependency,
            11 => FactTypeDiscriminant::DependencyVulnerability,
            12 => FactTypeDiscriminant::License,
            13 => FactTypeDiscriminant::UncoveredLine,
            14 => FactTypeDiscriminant::LowTestCoverage,
            15 => FactTypeDiscriminant::CoverageStats,
            16 => FactTypeDiscriminant::VulnerableUncovered,
            17 => FactTypeDiscriminant::SecurityTechnicalDebt,
            _ => FactTypeDiscriminant::QualitySecurityCorrelation,
        };
        enummap[discriminant] = i as u32;
    }

    group.bench_function("enummap_iter", |b| {
        b.iter(|| {
            let sum: u32 = enummap.values().sum();
            black_box(sum);
        })
    });

    group.bench_function("enummap_iter_keys", |b| {
        b.iter(|| {
            let count = enummap.iter().filter(|(_, &v)| v > 0).count();
            black_box(count);
        })
    });

    group.bench_function("enummap_values", |b| {
        b.iter(|| {
            let non_zero: Vec<_> = enummap.values().filter(|&&v| v > 0).collect();
            black_box(non_zero);
        })
    });

    group.finish();
}

fn bench_cache_locality(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_locality");

    // Simulate sequential access pattern
    group.bench_function("sequential_access_hashmap", |b| {
        let mut map = HashMap::new();
        for i in 0..19 {
            let discriminant = match i {
                0 => FactTypeDiscriminant::TaintSource,
                1 => FactTypeDiscriminant::TaintSink,
                2 => FactTypeDiscriminant::Vulnerability,
                3 => FactTypeDiscriminant::Function,
                4 => FactTypeDiscriminant::Variable,
                5 => FactTypeDiscriminant::Sanitization,
                6 => FactTypeDiscriminant::UnsafeCall,
                7 => FactTypeDiscriminant::CryptographicOperation,
                8 => FactTypeDiscriminant::CodeSmell,
                9 => FactTypeDiscriminant::ComplexityViolation,
                10 => FactTypeDiscriminant::Dependency,
                11 => FactTypeDiscriminant::DependencyVulnerability,
                12 => FactTypeDiscriminant::License,
                13 => FactTypeDiscriminant::UncoveredLine,
                14 => FactTypeDiscriminant::LowTestCoverage,
                15 => FactTypeDiscriminant::CoverageStats,
                16 => FactTypeDiscriminant::VulnerableUncovered,
                17 => FactTypeDiscriminant::SecurityTechnicalDebt,
                _ => FactTypeDiscriminant::QualitySecurityCorrelation,
            };
            map.insert(discriminant, i);
        }

        let order = vec![
            FactTypeDiscriminant::TaintSource,
            FactTypeDiscriminant::TaintSink,
            FactTypeDiscriminant::Vulnerability,
            FactTypeDiscriminant::Function,
            FactTypeDiscriminant::Variable,
        ];

        b.iter(|| {
            let mut sum = 0;
            for discriminant in &order {
                if let Some(&value) = map.get(discriminant) {
                    sum += value;
                }
            }
            black_box(sum);
        })
    });

    group.bench_function("sequential_access_enummap", |b| {
        let mut map: EnumMap<FactTypeDiscriminant, u32> = enum_map! { _ => 0 };
        for i in 0..19 {
            let discriminant = match i {
                0 => FactTypeDiscriminant::TaintSource,
                1 => FactTypeDiscriminant::TaintSink,
                2 => FactTypeDiscriminant::Vulnerability,
                3 => FactTypeDiscriminant::Function,
                4 => FactTypeDiscriminant::Variable,
                5 => FactTypeDiscriminant::Sanitization,
                6 => FactTypeDiscriminant::UnsafeCall,
                7 => FactTypeDiscriminant::CryptographicOperation,
                8 => FactTypeDiscriminant::CodeSmell,
                9 => FactTypeDiscriminant::ComplexityViolation,
                10 => FactTypeDiscriminant::Dependency,
                11 => FactTypeDiscriminant::DependencyVulnerability,
                12 => FactTypeDiscriminant::License,
                13 => FactTypeDiscriminant::UncoveredLine,
                14 => FactTypeDiscriminant::LowTestCoverage,
                15 => FactTypeDiscriminant::CoverageStats,
                16 => FactTypeDiscriminant::VulnerableUncovered,
                17 => FactTypeDiscriminant::SecurityTechnicalDebt,
                _ => FactTypeDiscriminant::QualitySecurityCorrelation,
            };
            map[discriminant] = i;
        }

        let order = vec![
            FactTypeDiscriminant::TaintSource,
            FactTypeDiscriminant::TaintSink,
            FactTypeDiscriminant::Vulnerability,
            FactTypeDiscriminant::Function,
            FactTypeDiscriminant::Variable,
        ];

        b.iter(|| {
            let mut sum = 0;
            for discriminant in &order {
                sum += map[*discriminant];
            }
            black_box(sum);
        })
    });

    group.finish();
}

fn bench_memory_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_overhead");

    // Create 1000 entries
    let items: Vec<(FactTypeDiscriminant, u32)> = (0..1000)
        .map(|i| {
            let discriminant = match i % 19 {
                0 => FactTypeDiscriminant::TaintSource,
                1 => FactTypeDiscriminant::TaintSink,
                2 => FactTypeDiscriminant::Vulnerability,
                3 => FactTypeDiscriminant::Function,
                4 => FactTypeDiscriminant::Variable,
                5 => FactTypeDiscriminant::Sanitization,
                6 => FactTypeDiscriminant::UnsafeCall,
                7 => FactTypeDiscriminant::CryptographicOperation,
                8 => FactTypeDiscriminant::CodeSmell,
                9 => FactTypeDiscriminant::ComplexityViolation,
                10 => FactTypeDiscriminant::Dependency,
                11 => FactTypeDiscriminant::DependencyVulnerability,
                12 => FactTypeDiscriminant::License,
                13 => FactTypeDiscriminant::UncoveredLine,
                14 => FactTypeDiscriminant::LowTestCoverage,
                15 => FactTypeDiscriminant::CoverageStats,
                16 => FactTypeDiscriminant::VulnerableUncovered,
                17 => FactTypeDiscriminant::SecurityTechnicalDebt,
                _ => FactTypeDiscriminant::QualitySecurityCorrelation,
            };
            (discriminant, i as u32)
        })
        .collect();

    group.bench_function("hashmap_memory", |b| {
        b.iter(|| {
            let mut map = HashMap::new();
            for (k, v) in &items {
                map.insert(*k, *v);
            }
            black_box(map)
        })
    });

    group.bench_function("enummap_memory", |b| {
        b.iter(|| {
            let mut map: EnumMap<FactTypeDiscriminant, u32> = enum_map! { _ => 0 };
            for (k, v) in &items {
                map[*k] = *v;
            }
            black_box(map)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_hashmap_vs_enummap,
    bench_iteration_performance,
    bench_cache_locality,
    bench_memory_overhead
);
criterion_main!(benches);
