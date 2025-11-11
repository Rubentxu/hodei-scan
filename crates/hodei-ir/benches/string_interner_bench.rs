//! Benchmarks for String Interner optimization
//!
//! These benchmarks measure the performance and memory improvements
//! from using string interning in hodei-ir.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use hodei_ir::interning::{Interner, ProjectPathInterner};

fn bench_string_interner_basic(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_interner");

    // Test with different numbers of strings
    for size in [100, 1000, 10000, 100000].iter() {
        let strings: Vec<String> = (0..*size)
            .map(|i| format!("path/to/file{}.rs", i % 100))
            .collect();

        group.bench_with_input(BenchmarkId::new("intern", size), &strings, |b, strings| {
            b.iter(|| {
                let mut interner = Interner::new();
                for s in strings {
                    black_box(interner.intern(s));
                }
                black_box(interner)
            })
        });

        group.bench_with_input(BenchmarkId::new("resolve", size), &strings, |b, strings| {
            let mut interner = Interner::new();
            let symbols: Vec<_> = strings.iter().map(|s| interner.intern(s)).collect();

            b.iter(|| {
                for sym in &symbols {
                    black_box(interner.resolve(*sym));
                }
            })
        });
    }

    group.finish();
}

fn bench_string_interner_deduplication(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_interner_deduplication");

    // Create 10,000 strings with heavy duplication (90% duplicates)
    let mut strings = Vec::new();
    for i in 0..10000 {
        if i % 10 < 9 {
            strings.push("src/main.rs".to_string());
        } else {
            strings.push(format!("src/file{}.rs", i % 10));
        }
    }

    group.bench_function("without_interning", |b| {
        b.iter(|| {
            // Simulate storing without deduplication
            let _unique: Vec<_> = strings.clone();
            black_box(_unique);
        })
    });

    group.bench_function("with_interning", |b| {
        b.iter(|| {
            let mut interner = Interner::new();
            let symbols: Vec<_> = strings.iter().map(|s| interner.intern(s)).collect();
            black_box((symbols, interner.len()));
        })
    });

    group.finish();
}

fn bench_string_interner_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_interner_comparison");

    // Generate test data
    let strings: Vec<String> = (0..1000)
        .map(|i| format!("src/module{}/file{}.rs", i % 20, i))
        .collect();

    group.bench_function("string_equality", |b| {
        b.iter(|| {
            let mut count = 0;
            for i in 0..strings.len() {
                for j in (i + 1)..strings.len() {
                    if strings[i] == strings[j] {
                        count += 1;
                    }
                }
            }
            black_box(count);
        })
    });

    group.bench_function("interned_equality", |b| {
        let mut interner = Interner::new();
        let symbols: Vec<_> = strings.iter().map(|s| interner.intern(s)).collect();

        b.iter(|| {
            let mut count = 0;
            for i in 0..symbols.len() {
                for j in (i + 1)..symbols.len() {
                    if symbols[i] == symbols[j] {
                        count += 1;
                    }
                }
            }
            black_box(count);
        })
    });

    group.finish();
}

fn bench_project_path_interner(c: &mut Criterion) {
    let mut group = c.benchmark_group("project_path_interner");

    // Test with various path complexities
    let paths = vec![
        "src/main.rs",
        "src/lib.rs",
        "tests/integration/test_main.rs",
        "src/../src/main.rs",
        "./src/main.rs",
        "src/utils/mod.rs",
        "Cargo.toml",
        "README.md",
    ];

    group.bench_function("intern_simple_paths", |b| {
        b.iter(|| {
            let mut interner = ProjectPathInterner::new();
            for path in &paths {
                black_box(interner.intern_path(path));
            }
            black_box(interner.len());
        })
    });

    group.bench_function("intern_complex_paths", |b| {
        let complex_paths: Vec<String> = (0..1000)
            .map(|i| {
                if i % 4 == 0 {
                    format!("src/../src/module{}/file.rs", i % 10)
                } else if i % 4 == 1 {
                    format!("./src/file{}", i)
                } else if i % 4 == 2 {
                    format!("src/module{}/file.rs", i % 20)
                } else {
                    "src/file.rs".to_string()
                }
            })
            .collect();

        b.iter(|| {
            let mut interner = ProjectPathInterner::new();
            for path in &complex_paths {
                black_box(interner.intern_path(path));
            }
            black_box(interner.len());
        })
    });

    group.bench_function("resolve_paths", |b| {
        let mut interner = ProjectPathInterner::new();
        let symbols: Vec<_> = paths.iter().map(|p| interner.intern_path(p)).collect();

        b.iter(|| {
            for sym in &symbols {
                black_box(interner.resolve_path(*sym));
            }
        })
    });

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_interner_memory");

    // Measure memory usage with and without interning
    let paths: Vec<String> = vec!["src/main.rs".to_string(); 50000];

    group.bench_function("baseline_storage", |b| {
        b.iter(|| {
            // Simulate storing without deduplication
            let _storage: Vec<String> = paths.clone();
            black_box(_storage)
        })
    });

    group.bench_function("interned_storage", |b| {
        b.iter(|| {
            let mut interner = Interner::new();
            let _symbols: Vec<usize> = paths.iter().map(|p| interner.intern(p)).collect();
            black_box((_symbols, interner))
        })
    });

    group.finish();
}

fn bench_concurrent_interning(c: &mut Criterion) {
    let mut group = c.benchmark_group("string_interner_concurrent");

    let strings: Vec<String> = (0..10000)
        .map(|i| format!("path/to/file{}.rs", i % 500))
        .collect();

    group.bench_function("single_thread", |b| {
        b.iter(|| {
            let mut interner = Interner::new();
            for s in &strings {
                black_box(interner.intern(s));
            }
            black_box(interner.len());
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_string_interner_basic,
    bench_string_interner_deduplication,
    bench_string_interner_comparison,
    bench_project_path_interner,
    bench_memory_usage,
    bench_concurrent_interning
);
criterion_main!(benches);
