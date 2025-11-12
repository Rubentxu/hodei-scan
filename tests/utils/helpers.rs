//! Test helpers and assertions
//!
//! This module provides utility functions and custom assertions for testing

use std::path::Path;

/// Assert that a path exists
#[track_caller]
pub fn assert_path_exists(path: &Path) {
    assert!(
        path.exists(),
        "Path should exist: {}",
        path.display()
    );
}

/// Assert that a path does not exist
#[track_caller]
pub fn assert_path_not_exists(path: &Path) {
    assert!(
        !path.exists(),
        "Path should not exist: {}",
        path.display()
    );
}

/// Assert that two FindingSets are equal
#[track_caller]
pub fn assert_findings_equal(
    expected: &hodei_ir::FindingSet,
    actual: &hodei_ir::FindingSet,
) {
    assert_eq!(
        expected.findings.len(),
        actual.findings.len(),
        "Finding sets should have same number of findings"
    );

    for (i, (exp, act)) in expected
        .findings
        .iter()
        .zip(actual.findings.iter())
        .enumerate()
    {
        assert_eq!(
            exp.fact_type, act.fact_type,
            "Finding {} fact_type should match",
            i
        );
        assert_eq!(
            exp.message, act.message,
            "Finding {} message should match",
            i
        );
        assert_eq!(
            exp.location, act.location,
            "Finding {} location should match",
            i
        );
        assert_eq!(
            exp.severity, act.severity,
            "Finding {} severity should match",
            i
        );
    }
}

/// Assert that a FindingSet contains a specific finding
#[track_caller]
pub fn assert_finding_exists(ir: &hodei_ir::FindingSet, message: &str) {
    assert!(
        ir.findings.iter().any(|f| f.message == message),
        "Finding set should contain finding with message: {}",
        message
    );
}

/// Assert that a FindingSet does not contain a specific finding
#[track_caller]
pub fn assert_finding_not_exists(ir: &hodei_ir::FindingSet, message: &str) {
    assert!(
        !ir.findings.iter().any(|f| f.message == message),
        "Finding set should not contain finding with message: {}",
        message
    );
}

/// Assert that a FindingSet contains a finding with specific type
#[track_caller]
pub fn assert_finding_type_exists(ir: &hodei_ir::FindingSet, fact_type: &str) {
    assert!(
        ir.findings.iter().any(|f| f.fact_type == fact_type),
        "Finding set should contain finding with type: {}",
        fact_type
    );
}

/// Assert that a FindingSet contains exactly N findings
#[track_caller]
pub fn assert_finding_count(ir: &hodei_ir::FindingSet, count: usize) {
    assert_eq!(
        ir.findings.len(),
        count,
        "Finding set should contain exactly {} findings",
        count
    );
}

/// Assert that a FindingSet has no findings
#[track_caller]
pub fn assert_no_findings(ir: &hodei_ir::FindingSet) {
    assert!(
        ir.findings.is_empty(),
        "Finding set should be empty"
    );
}

/// Assert that a TestResults has specific pass/fail counts
#[track_caller]
pub fn assert_test_results(
    results: &hodei_test::domain::models::TestResults,
    expected_total: usize,
    expected_passed: usize,
    expected_failed: usize,
) {
    let total = results.total_count();
    let passed = results.passed_count();
    let failed = results.failed_count();

    assert_eq!(
        total, expected_total,
        "Should have {} total tests",
        expected_total
    );
    assert_eq!(
        passed, expected_passed,
        "Should have {} passed tests",
        expected_passed
    );
    assert_eq!(
        failed, expected_failed,
        "Should have {} failed tests",
        expected_failed
    );
}

/// Assert that all tests passed
#[track_caller]
pub fn assert_all_tests_passed(results: &hodei_test::domain::models::TestResults) {
    assert!(
        results.failed_count() == 0,
        "All tests should pass, but {} failed",
        results.failed_count()
    );
}

/// Assert that some tests failed
#[track_caller]
pub fn assert_some_tests_failed(results: &hodei_test::domain::models::TestResults) {
    assert!(
        results.failed_count() > 0,
        "Some tests should fail"
    );
}

/// Wait for a file to exist with timeout
pub async fn wait_for_file(
    path: &Path,
    timeout_millis: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_millis);

    while !path.exists() {
        if start.elapsed() > timeout {
            return Err(format!("File not found after timeout: {}", path.display()).into());
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    Ok(())
}

/// Wait for a condition to be true with timeout
pub async fn wait_for_condition<F>(
    mut condition: F,
    timeout_millis: u64,
    check_interval_millis: u64,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnMut() -> bool,
{
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(timeout_millis);
    let interval = std::time::Duration::from_millis(check_interval_millis);

    while !condition() {
        if start.elapsed() > timeout {
            return Err("Condition not met after timeout".into());
        }
        tokio::time::sleep(interval).await;
    }

    Ok(())
}

/// Create a temporary directory for testing
pub fn temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Create a temporary file for testing
pub fn temp_file() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp_dir = temp_dir();
    let file_path = temp_dir.path().join("test_file.txt");
    (temp_dir, file_path)
}

/// Read file content safely
#[track_caller]
pub fn read_file_content(path: &Path) -> String {
    std::fs::read_to_string(path).expect("Failed to read file")
}

/// Write file content safely
#[track_caller]
pub fn write_file_content(path: &Path, content: &str) {
    std::fs::write(path, content).expect("Failed to write file");
}

/// Compare two files line by line
#[track_caller]
pub fn assert_files_equal(path1: &Path, path2: &Path) {
    let content1 = read_file_content(path1);
    let content2 = read_file_content(path2);
    assert_eq!(content1, content2, "Files should be equal");
}

/// Compare strings line by line
#[track_caller]
pub fn assert_multiline_string_eq(expected: &str, actual: &str) {
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    assert_eq!(
        expected_lines.len(),
        actual_lines.len(),
        "Strings should have same number of lines"
    );

    for (i, (exp, act)) in expected_lines.iter().zip(actual_lines.iter()).enumerate() {
        assert_eq!(
            exp, act,
            "Line {} should match",
            i + 1
        );
    }
}

/// Check if a string contains a substring (case-insensitive)
#[track_caller]
pub fn assert_contains_case_insensitive(haystack: &str, needle: &str) {
    assert!(
        haystack.to_lowercase().contains(&needle.to_lowercase()),
        "String should contain (case-insensitive): {}",
        needle
    );
}

/// Check if a string does not contain a substring (case-insensitive)
#[track_caller]
pub fn assert_not_contains_case_insensitive(haystack: &str, needle: &str) {
    assert!(
        !haystack.to_lowercase().contains(&needle.to_lowercase()),
        "String should not contain (case-insensitive): {}",
        needle
    );
}

/// Measure execution time of an async function
pub async fn measure_time<F, T>(f: F) -> (T, std::time::Duration)
where
    F: std::future::Future<Output = T>,
{
    let start = std::time::Instant::now();
    let result = f.await;
    let elapsed = start.elapsed();
    (result, elapsed)
}

/// Assert that execution time is less than expected
#[track_caller]
pub fn assert_execution_time_less_than(
    elapsed: std::time::Duration,
    max_time: std::time::Duration,
) {
    assert!(
        elapsed < max_time,
        "Execution took {:?}, should be less than {:?}",
        elapsed,
        max_time
    );
}

/// Format finding for display
pub fn format_finding(finding: &hodei_ir::Finding) -> String {
    let mut result = format!("[{}] {}", finding.fact_type, finding.message);
    if let Some(ref location) = finding.location {
        result.push_str(&format!(" at {}", location));
    }
    if let Some(ref severity) = finding.severity {
        result.push_str(&format!(" ({})", severity));
    }
    result
}

/// Print all findings in a FindingSet
pub fn print_findings(ir: &hodei_ir::FindingSet) {
    println!("Total findings: {}", ir.findings.len());
    for (i, finding) in ir.findings.iter().enumerate() {
        println!("  {}. {}", i + 1, format_finding(finding));
    }
}

/// Generate a unique test name based on test function and timestamp
pub fn unique_test_name(test_name: &str) -> String {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    format!("{}_{}", test_name, timestamp)
}

/// Assert that two numbers are approximately equal (with tolerance)
#[track_caller]
pub fn assert_approximately_equal(
    actual: f64,
    expected: f64,
    tolerance: f64,
) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= tolerance,
        "Numbers should be approximately equal: {} vs {} (diff: {}, tolerance: {})",
        actual,
        expected,
        diff,
        tolerance
    );
}

/// Check if a finding matches criteria
pub fn finding_matches(
    finding: &hodei_ir::Finding,
    fact_type: Option<&str>,
    message: Option<&str>,
    severity: Option<&str>,
) -> bool {
    if let Some(ft) = fact_type {
        if finding.fact_type != ft {
            return false;
        }
    }

    if let Some(msg) = message {
        if !finding.message.contains(msg) {
            return false;
        }
    }

    if let Some(sev) = severity {
        if finding.severity.as_ref().map(|s| s.as_str()) != Some(sev) {
            return false;
        }
    }

    true
}

/// Assert that a finding matches criteria
#[track_caller]
pub fn assert_finding_matches(
    finding: &hodei_ir::Finding,
    fact_type: Option<&str>,
    message: Option<&str>,
    severity: Option<&str>,
) {
    assert!(
        finding_matches(finding, fact_type, message, severity),
        "Finding does not match criteria"
    );
}
