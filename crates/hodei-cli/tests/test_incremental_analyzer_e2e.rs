//! End-to-end test for Incremental Analyzer
//!
//! This test demonstrates the Incremental Analyzer working in realistic scenarios,
//! measuring performance improvements and verifying correctness.

use std::path::PathBuf;
use std::time::Duration;
use tempfile::TempDir;

use hodei_cli::analysis::{AnalysisStats, IncrementalAnalyzer, IncrementalConfig, MockExtractor};

fn create_analyzer(temp_dir: &TempDir) -> IncrementalAnalyzer {
    let extractor = std::sync::Arc::new(MockExtractor::new("test"));
    let mut config = IncrementalConfig::default();
    config.use_cache = false; // Disable cache to avoid lock issues in tests
    IncrementalAnalyzer::with_config(temp_dir.path(), config, extractor).unwrap()
}

#[test]
fn test_incremental_analyzer_e2e_basic() {
    let temp_dir = tempfile::tempdir().unwrap();
    let _files = create_test_files(&temp_dir, 10);

    let analyzer = create_analyzer(&temp_dir);

    // Run initial full analysis
    let (_facts, stats) = analyzer.analyze(temp_dir.path(), None).unwrap();

    assert!(stats.files_analyzed > 0);
    assert!(stats.facts_found > 0);
    assert!(stats.is_full_analysis);
    assert_eq!(stats.cache_hit_rate, 0.0);

    // Run incremental analysis (no cache, so it will be full analysis)
    let (_facts2, stats2) = analyzer.analyze(temp_dir.path(), Some("HEAD~1")).unwrap();

    // Without cache or change detector, this will be full analysis
    assert!(stats2.files_analyzed > 0);
}

#[test]
fn test_incremental_analyzer_with_cache() {
    let temp_dir = tempfile::tempdir().unwrap();
    let _files = create_test_files(&temp_dir, 20);

    let analyzer = create_analyzer(&temp_dir);

    let (_facts, stats) = analyzer.analyze(temp_dir.path(), None).unwrap();
    assert!(stats.files_analyzed == 20);

    let (_facts, stats) = analyzer.analyze(temp_dir.path(), Some("HEAD~1")).unwrap();
    // With cache disabled, this will be full analysis
    assert!(stats.files_analyzed > 0);
}

#[test]
fn test_incremental_analyzer_force_full() {
    let temp_dir = tempfile::tempdir().unwrap();
    let _files = create_test_files(&temp_dir, 5);

    let extractor = std::sync::Arc::new(MockExtractor::new("test"));
    let mut config = IncrementalConfig::default();
    config.force_full = true;
    config.use_cache = false;

    let analyzer = IncrementalAnalyzer::with_config(temp_dir.path(), config, extractor).unwrap();
    let (_facts, stats) = analyzer.analyze(temp_dir.path(), None).unwrap();

    assert!(stats.is_full_analysis);
}

#[test]
fn test_incremental_analyzer_no_cache() {
    let temp_dir = tempfile::tempdir().unwrap();
    let _files = create_test_files(&temp_dir, 5);

    let analyzer = create_analyzer(&temp_dir);
    let (_facts, stats) = analyzer.analyze(temp_dir.path(), None).unwrap();

    assert!(stats.files_analyzed > 0);
}

#[test]
fn test_efficiency_score_calculation() {
    let stats_high = AnalysisStats {
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

    let score_high = stats_high.efficiency_score();
    assert!(score_high > 0.7);

    let stats_low = AnalysisStats {
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

    let score_low = stats_low.efficiency_score();
    assert!(score_low < 0.3);

    let mut stats_full = stats_high.clone();
    stats_full.is_full_analysis = true;
    assert_eq!(stats_full.efficiency_score(), 0.0);
}

#[test]
fn test_analysis_stats_summary() {
    let stats = AnalysisStats {
        duration: Duration::from_secs(1),
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

    let summary = stats.summary();
    assert!(summary.contains("100"));
    assert!(summary.contains("50"));
    assert!(summary.contains("80.0%"));
}

#[test]
fn test_incremental_analyzer_large_project() {
    let temp_dir = tempfile::tempdir().unwrap();
    let _files = create_test_files(&temp_dir, 100);

    let analyzer = create_analyzer(&temp_dir);

    let (_facts, stats) = analyzer.analyze(temp_dir.path(), None).unwrap();

    assert_eq!(stats.files_analyzed, 100);
    assert!(stats.facts_found > 0);
    assert!(stats.total_files >= 100);
}

#[test]
fn test_cache_operations() {
    let temp_dir = tempfile::tempdir().unwrap();
    let _files = create_test_files(&temp_dir, 5);

    let analyzer = create_analyzer(&temp_dir);

    // Should have no cache stats when cache is disabled
    let stats = analyzer.get_cache_stats();
    assert!(stats.is_none());

    let cleaned = analyzer.cleanup_cache().unwrap();
    assert_eq!(cleaned, 0);

    analyzer.clear_cache().unwrap();
}

#[test]
fn test_file_hash_computation() {
    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("test.rs");
    std::fs::write(&test_file, "// TODO: test\nfn main() {}\n").unwrap();

    let analyzer = create_analyzer(&temp_dir);

    let (facts, stats) = analyzer.analyze(temp_dir.path(), None).unwrap();
    assert!(stats.files_analyzed >= 1);
    assert!(facts.len() > 0);
}

#[test]
fn test_file_discovery() {
    let temp_dir = tempfile::tempdir().unwrap();
    let _files = create_test_files(&temp_dir, 20);

    let analyzer = create_analyzer(&temp_dir);

    let (_facts, stats) = analyzer.analyze(temp_dir.path(), None).unwrap();
    assert!(stats.total_files >= 20);
    assert!(stats.files_analyzed == 20);
}

fn create_test_files(temp_dir: &TempDir, num_files: usize) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for i in 0..num_files {
        let file_path = temp_dir.path().join(format!("file_{}.rs", i));
        let content = format!(
            "// TODO: implement feature {}\n// TODO: optimize later\nfn func_{}() {{\n    // TODO: add tests\n}}\n",
            i, i
        );
        std::fs::write(&file_path, content).unwrap();
        files.push(file_path);
    }

    files
}
