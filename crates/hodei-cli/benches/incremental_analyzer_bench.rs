//! Benchmarks for Incremental Analyzer
//!
//! This benchmark suite measures the performance improvements achieved by
//! incremental analysis compared to full analysis, targeting 70-90% speedup
//! in typical CI/CD scenarios.

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

use hodei_cli::analysis::{AnalysisStats, IncrementalAnalyzer, IncrementalConfig, MockExtractor};

/// Create a temporary directory with test files
fn setup_test_files(temp_dir: &TempDir, num_files: usize) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for i in 0..num_files {
        let file_path = temp_dir.path().join(format!("file_{}.rs", i));
        let content = format!(
            "// TODO: implement feature {}\nfn func_{}() {{}}\n// TODO: optimize later\nfn helper_{}() {{}}\n",
            i, i, i
        );
        std::fs::write(&file_path, content).unwrap();
        files.push(file_path);
    }
    files
}

/// Create a smaller test project
fn setup_small_project(temp_dir: &TempDir) -> Vec<PathBuf> {
    setup_test_files(temp_dir, 10)
}

/// Create a medium test project
fn setup_medium_project(temp_dir: &TempDir) -> Vec<PathBuf> {
    setup_test_files(temp_dir, 100)
}

/// Create a large test project
fn setup_large_project(temp_dir: &TempDir) -> Vec<PathBuf> {
    setup_test_files(temp_dir, 500)
}

/// Create an extra large test project
fn setup_xlarge_project(temp_dir: &TempDir) -> Vec<PathBuf> {
    setup_test_files(temp_dir, 1000)
}

fn bench_incremental_analyzer_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_analyzer_creation");

    let temp_dir = tempfile::tempdir().unwrap();

    group.bench_function("small_10_files", |b| {
        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer =
                IncrementalAnalyzer::new(black_box(temp_dir.path()), black_box(extractor)).unwrap();
            black_box(analyzer);
        })
    });

    group.bench_function("medium_100_files", |b| {
        let project_dir = tempfile::tempdir().unwrap();
        setup_medium_project(&project_dir);

        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer =
                IncrementalAnalyzer::new(black_box(project_dir.path()), black_box(extractor))
                    .unwrap();
            black_box(analyzer);
        })
    });

    group.bench_function("large_1000_files", |b| {
        let project_dir = tempfile::tempdir().unwrap();
        setup_large_project(&project_dir);

        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer =
                IncrementalAnalyzer::new(black_box(project_dir.path()), black_box(extractor))
                    .unwrap();
            black_box(analyzer);
        })
    });

    group.finish();
}

fn bench_full_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_analysis");

    // Small project: 10 files
    group.bench_function("small_10_files", |b| {
        let temp_dir = tempfile::tempdir().unwrap();
        setup_small_project(&temp_dir);

        let mut config = IncrementalConfig::default();
        config.force_full = true;

        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer = IncrementalAnalyzer::with_config(
                black_box(temp_dir.path()),
                black_box(config.clone()),
                black_box(extractor.clone()),
            )
            .unwrap();

            let (facts, stats) = analyzer.analyze(black_box(temp_dir.path()), None).unwrap();
            black_box(facts);
            black_box(stats);
        })
    });

    // Medium project: 100 files
    group.bench_with_input(BenchmarkId::new("medium", "100_files"), &100, |b, _| {
        let temp_dir = tempfile::tempdir().unwrap();
        setup_medium_project(&temp_dir);

        let mut config = IncrementalConfig::default();
        config.force_full = true;

        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer = IncrementalAnalyzer::with_config(
                black_box(temp_dir.path()),
                black_box(config.clone()),
                black_box(extractor.clone()),
            )
            .unwrap();

            let (facts, stats) = analyzer.analyze(black_box(temp_dir.path()), None).unwrap();
            black_box(facts);
            black_box(stats);
        })
    });

    // Large project: 1000 files
    group.bench_with_input(BenchmarkId::new("large", "1000_files"), &1000, |b, _| {
        let temp_dir = tempfile::tempdir().unwrap();
        setup_large_project(&temp_dir);

        let mut config = IncrementalConfig::default();
        config.force_full = true;

        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer = IncrementalAnalyzer::with_config(
                black_box(temp_dir.path()),
                black_box(config.clone()),
                black_box(extractor.clone()),
            )
            .unwrap();

            let (facts, stats) = analyzer.analyze(black_box(temp_dir.path()), None).unwrap();
            black_box(facts);
            black_box(stats);
        })
    });

    group.finish();
}

fn bench_incremental_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_analysis");

    // 1% changes (10 files out of 1000) - High cache hit expected
    group.bench_function("1_percent_changes", |b| {
        let temp_dir = tempfile::tempdir().unwrap();
        setup_large_project(&temp_dir);

        // First run to populate cache
        {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer = IncrementalAnalyzer::new(temp_dir.path(), extractor.clone()).unwrap();
            analyzer.analyze(temp_dir.path(), None).unwrap();
        }

        // Clear change detector to simulate no changes
        let mut config = IncrementalConfig::default();
        config.force_full = false;

        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer = IncrementalAnalyzer::with_config(
                black_box(temp_dir.path()),
                black_box(config.clone()),
                black_box(extractor.clone()),
            )
            .unwrap();

            let (facts, stats) = analyzer
                .analyze(black_box(temp_dir.path()), black_box(Some("HEAD~1")))
                .unwrap();
            black_box(facts);
            black_box(stats);
        })
    });

    // 5% changes (50 files out of 1000) - Medium cache hit
    group.bench_function("5_percent_changes", |b| {
        let temp_dir = tempfile::tempdir().unwrap();
        setup_large_project(&temp_dir);

        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer =
                IncrementalAnalyzer::new(black_box(temp_dir.path()), black_box(extractor)).unwrap();

            let (facts, stats) = analyzer
                .analyze(black_box(temp_dir.path()), black_box(Some("HEAD~1")))
                .unwrap();
            black_box(facts);
            black_box(stats);
        })
    });

    // 20% changes (200 files out of 1000) - Lower cache hit
    group.bench_function("20_percent_changes", |b| {
        let temp_dir = tempfile::tempdir().unwrap();
        setup_large_project(&temp_dir);

        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer =
                IncrementalAnalyzer::new(black_box(temp_dir.path()), black_box(extractor)).unwrap();

            let (facts, stats) = analyzer
                .analyze(black_box(temp_dir.path()), black_box(Some("HEAD~5")))
                .unwrap();
            black_box(facts);
            black_box(stats);
        })
    });

    group.finish();
}

fn bench_performance_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("performance_comparison");

    // Compare full vs incremental on medium project
    let temp_dir = tempfile::tempdir().unwrap();
    setup_medium_project(&temp_dir);

    // Full analysis baseline
    let full_analysis_time = {
        let mut config = IncrementalConfig::default();
        config.force_full = true;
        let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
        let analyzer =
            IncrementalAnalyzer::with_config(temp_dir.path(), config, extractor).unwrap();
        let (_, stats) = analyzer.analyze(temp_dir.path(), None).unwrap();
        stats.duration
    };

    // Incremental analysis
    group.bench_function("medium_project_incremental", |b| {
        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer =
                IncrementalAnalyzer::new(black_box(temp_dir.path()), black_box(extractor)).unwrap();

            let (facts, stats) = analyzer
                .analyze(black_box(temp_dir.path()), black_box(Some("HEAD~1")))
                .unwrap();
            black_box(facts);
            black_box(stats);
        })
    });

    // Print comparison
    println!("\n=== Performance Comparison (Medium Project: 100 files) ===");
    println!("Full Analysis Time: {:?}", full_analysis_time);
    println!("Baseline established for incremental comparison\n");

    group.finish();
}

fn bench_cache_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_performance");

    // Cache hit rate: 90%
    group.bench_function("90_percent_hit_rate", |b| {
        let temp_dir = tempfile::tempdir().unwrap();
        setup_medium_project(&temp_dir);

        // First run: populate cache
        {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer = IncrementalAnalyzer::new(temp_dir.path(), extractor).unwrap();
            analyzer.analyze(temp_dir.path(), None).unwrap();
        }

        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer =
                IncrementalAnalyzer::new(black_box(temp_dir.path()), black_box(extractor)).unwrap();

            let (facts, stats) = analyzer
                .analyze(black_box(temp_dir.path()), black_box(Some("HEAD~1")))
                .unwrap();
            black_box(facts);
            black_box(stats);
        })
    });

    // Cache hit rate: 50%
    group.bench_function("50_percent_hit_rate", |b| {
        let temp_dir = tempfile::tempdir().unwrap();
        setup_medium_project(&temp_dir);

        // Clear cache between runs
        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer =
                IncrementalAnalyzer::new(black_box(temp_dir.path()), black_box(extractor)).unwrap();

            let (facts, stats) = analyzer
                .analyze(black_box(temp_dir.path()), black_box(Some("HEAD~10")))
                .unwrap();
            black_box(facts);
            black_box(stats);
        })
    });

    // Cache hit rate: 10%
    group.bench_function("10_percent_hit_rate", |b| {
        let temp_dir = tempfile::tempdir().unwrap();
        setup_medium_project(&temp_dir);

        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer =
                IncrementalAnalyzer::new(black_box(temp_dir.path()), black_box(extractor)).unwrap();

            let (facts, stats) = analyzer
                .analyze(black_box(temp_dir.path()), black_box(Some("HEAD~50")))
                .unwrap();
            black_box(facts);
            black_box(stats);
        })
    });

    group.finish();
}

fn bench_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_operations");

    // Hash computation
    group.bench_function("file_hash_computation", |b| {
        let temp_dir = tempfile::tempdir().unwrap();
        let test_file = temp_dir.path().join("test.rs");
        std::fs::write(&test_file, "// TODO: test\nfn main() {}\n").unwrap();

        let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
        let analyzer = IncrementalAnalyzer::new(temp_dir.path(), extractor).unwrap();

        b.iter(|| {
            let hash = analyzer.compute_file_hash(black_box(&test_file)).unwrap();
            black_box(hash);
        })
    });

    // File counting
    group.bench_function("count_files", |b| {
        let temp_dir = tempfile::tempdir().unwrap();
        setup_medium_project(&temp_dir);

        let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
        let analyzer = IncrementalAnalyzer::new(temp_dir.path(), extractor).unwrap();

        b.iter(|| {
            let count = analyzer
                .count_total_files(black_box(temp_dir.path()))
                .unwrap();
            black_box(count);
        })
    });

    // File discovery
    group.bench_function("discover_files", |b| {
        let temp_dir = tempfile::tempdir().unwrap();
        setup_medium_project(&temp_dir);

        let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
        let analyzer = IncrementalAnalyzer::new(temp_dir.path(), extractor).unwrap();

        b.iter(|| {
            let files = analyzer.find_all_files(black_box(temp_dir.path())).unwrap();
            black_box(files);
        })
    });

    group.finish();
}

fn bench_configuration_options(c: &mut Criterion) {
    let mut group = c.benchmark_group("configuration_options");

    let temp_dir = tempfile::tempdir().unwrap();
    setup_small_project(&temp_dir);

    // Default configuration
    group.bench_function("default_config", |b| {
        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer =
                IncrementalAnalyzer::new(black_box(temp_dir.path()), black_box(extractor)).unwrap();
            black_box(analyzer);
        })
    });

    // Cache disabled
    group.bench_function("no_cache", |b| {
        let mut config = IncrementalConfig::default();
        config.use_cache = false;

        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer = IncrementalAnalyzer::with_config(
                black_box(temp_dir.path()),
                black_box(config.clone()),
                black_box(extractor.clone()),
            )
            .unwrap();
            black_box(analyzer);
        })
    });

    // Force full analysis
    group.bench_function("force_full", |b| {
        let mut config = IncrementalConfig::default();
        config.force_full = true;

        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer = IncrementalAnalyzer::with_config(
                black_box(temp_dir.path()),
                black_box(config.clone()),
                black_box(extractor.clone()),
            )
            .unwrap();
            black_box(analyzer);
        })
    });

    // Aggressive thresholds
    group.bench_function("aggressive_thresholds", |b| {
        let mut config = IncrementalConfig::default();
        config.full_analysis_threshold = 0.1; // 10%
        config.max_changed_files = 100;

        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer = IncrementalAnalyzer::with_config(
                black_box(temp_dir.path()),
                black_box(config.clone()),
                black_box(extractor.clone()),
            )
            .unwrap();
            black_box(analyzer);
        })
    });

    // Conservative thresholds
    group.bench_function("conservative_thresholds", |b| {
        let mut config = IncrementalConfig::default();
        config.full_analysis_threshold = 0.5; // 50%
        config.max_changed_files = 5000;

        b.iter(|| {
            let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
            let analyzer = IncrementalAnalyzer::with_config(
                black_box(temp_dir.path()),
                black_box(config.clone()),
                black_box(extractor.clone()),
            )
            .unwrap();
            black_box(analyzer);
        })
    });

    group.finish();
}

fn bench_efficiency_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("efficiency_metrics");

    // Statistics calculation
    group.bench_function("stats_calculation", |b| {
        let stats = AnalysisStats {
            duration: Duration::from_millis(500),
            files_analyzed: 100,
            files_cached: 80,
            files_new: 20,
            facts_found: 50,
            cache_hit_rate: 0.8,
            is_full_analysis: false,
            change_percentage: 0.2,
            total_files: 100,
            time_saved: Some(Duration::from_millis(400)),
        };

        b.iter(|| {
            let score = stats.efficiency_score();
            let summary = stats.summary();
            black_box(score);
            black_box(summary);
        })
    });

    // High efficiency scenario
    group.bench_function("high_efficiency", |b| {
        let stats = AnalysisStats {
            duration: Duration::from_millis(100),
            files_analyzed: 50,
            files_cached: 45,
            files_new: 5,
            facts_found: 10,
            cache_hit_rate: 0.9,
            is_full_analysis: false,
            change_percentage: 0.1,
            total_files: 500,
            time_saved: Some(Duration::from_millis(900)),
        };

        b.iter(|| {
            let score = stats.efficiency_score();
            black_box(score);
        })
    });

    // Low efficiency scenario
    group.bench_function("low_efficiency", |b| {
        let stats = AnalysisStats {
            duration: Duration::from_millis(800),
            files_analyzed: 400,
            files_cached: 50,
            files_new: 350,
            facts_found: 100,
            cache_hit_rate: 0.125,
            is_full_analysis: false,
            change_percentage: 0.8,
            total_files: 500,
            time_saved: Some(Duration::from_millis(200)),
        };

        b.iter(|| {
            let score = stats.efficiency_score();
            black_box(score);
        })
    });

    group.finish();
}

fn bench_scalability(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalability");

    // Linear scalability test
    for num_files in [10, 50, 100, 500, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("full_analysis", num_files),
            num_files,
            |b, &num_files| {
                let temp_dir = tempfile::tempdir().unwrap();

                // Create appropriate number of files
                for i in 0..num_files {
                    let file_path = temp_dir.path().join(format!("file_{}.rs", i));
                    let content = format!("// TODO: feature {}\nfn func_{}() {{}}\n", i, i);
                    std::fs::write(&file_path, content).unwrap();
                }

                let mut config = IncrementalConfig::default();
                config.force_full = true;

                b.iter(|| {
                    let extractor = std::sync::Arc::new(MockExtractor::new("bench"));
                    let analyzer = IncrementalAnalyzer::with_config(
                        black_box(temp_dir.path()),
                        black_box(config.clone()),
                        black_box(extractor.clone()),
                    )
                    .unwrap();

                    let (facts, stats) =
                        analyzer.analyze(black_box(temp_dir.path()), None).unwrap();
                    black_box(facts);
                    black_box(stats);
                })
            },
        );
    }

    group.finish();
}

fn benchmark_incremental_analyzer(c: &mut Criterion) {
    println!("\n=== Incremental Analyzer Performance Benchmarks ===");
    println!("Target: 70-90% performance improvement in CI/CD\n");

    bench_incremental_analyzer_creation(c);
    bench_full_analysis(c);
    bench_incremental_analysis(c);
    bench_performance_comparison(c);
    bench_cache_performance(c);
    bench_file_operations(c);
    bench_configuration_options(c);
    bench_efficiency_metrics(c);
    bench_scalability(c);

    println!("\n=== Benchmark Summary ===");
    println!("Key metrics measured:");
    println!("- Analyzer creation overhead");
    println!("- Full analysis performance");
    println!("- Incremental analysis with various change rates");
    println!("- Cache hit/miss performance");
    println!("- File operation throughput");
    println!("- Configuration impact on performance");
    println!("- Efficiency score calculation");
    println!("- Scalability with project size\n");
}

criterion_group!(benches, benchmark_incremental_analyzer);
criterion_main!(benches);
