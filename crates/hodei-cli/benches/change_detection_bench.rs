//! Benchmarks for Change Detection System
//!
//! These benchmarks measure the performance of different change detection strategies
//! including Git-based detection and hash-based detection.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hodei_cli::analysis::change_detector::*;
use std::fs;
use std::path::PathBuf;

fn bench_factory_auto_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("ChangeDetectorFactory");

    let temp_dir = tempfile::tempdir().unwrap();
    let project_root = temp_dir.path().to_path_buf();

    group.bench_function("auto_detection", |b| {
        b.iter(|| {
            let detector = ChangeDetectorFactory::create(&project_root).unwrap();
            black_box(detector);
        })
    });

    group.finish();
}

fn bench_hash_based_detector(c: &mut Criterion) {
    let mut group = c.benchmark_group("HashBasedDetector");

    // Create a temporary directory with test files
    let temp_dir = tempfile::tempdir().unwrap();
    let project_root = temp_dir.path().to_path_buf();

    // Create test files
    for i in 0..100 {
        let file_path = project_root.join(format!("file_{}.txt", i));
        fs::write(&file_path, format!("Content of file {}", i)).unwrap();
    }

    let detector = HashBasedDetector::new(&project_root);

    group.bench_function("detect_changed_files_first_scan", |b| {
        b.iter(|| {
            let changes = detector.detect_changed_files(None).unwrap();
            black_box(changes);
        })
    });

    // Modify some files for second scan
    for i in 0..50 {
        let file_path = project_root.join(format!("file_{}.txt", i));
        fs::write(&file_path, format!("Modified content of file {}", i)).unwrap();
    }

    // Add a new file
    fs::write(project_root.join("new_file.txt"), "New file content").unwrap();

    // Remove some files
    fs::remove_file(project_root.join("file_90.txt")).unwrap();
    fs::remove_file(project_root.join("file_91.txt")).unwrap();

    let detector2 = HashBasedDetector::new(&project_root);

    group.bench_function("detect_changed_files_incremental", |b| {
        b.iter(|| {
            let changes = detector2.detect_changed_files(None).unwrap();
            black_box(changes);
        })
    });

    group.finish();
}

fn bench_changed_file_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ChangedFile");

    let path = PathBuf::from("src/main.rs");
    let change_type = ChangeType::Modified;

    group.bench_function("create_changed_file", |b| {
        b.iter(|| {
            let changed_file = ChangedFile::new(path.clone(), change_type);
            black_box(changed_file);
        })
    });

    group.bench_function("create_changed_file_with_change_id", |b| {
        b.iter(|| {
            let changed_file =
                ChangedFile::with_change_id(path.clone(), change_type, "abc123".to_string());
            black_box(changed_file);
        })
    });

    group.bench_function("create_changed_file_with_old_path", |b| {
        let old_path = PathBuf::from("src/old.rs");
        b.iter(|| {
            let changed_file =
                ChangedFile::with_old_path(path.clone(), old_path.clone(), ChangeType::Renamed);
            black_box(changed_file);
        })
    });

    group.finish();
}

fn bench_git_detector_applicability(c: &mut Criterion) {
    let mut group = c.benchmark_group("GitDetector");

    let temp_dir = tempfile::tempdir().unwrap();
    let git_dir = temp_dir.path().join(".git");
    let detector = GitDetector::new(temp_dir.path());

    group.bench_function("is_applicable_git_repo", |b| {
        // Create .git directory
        fs::create_dir(&git_dir).unwrap();
        b.iter(|| {
            let result = detector.is_applicable(temp_dir.path());
            black_box(result);
        })
    });

    group.bench_function("is_applicable_non_git", |b| {
        let temp_dir2 = tempfile::tempdir().unwrap();
        b.iter(|| {
            let result = detector.is_applicable(temp_dir2.path());
            black_box(result);
        })
    });

    group.finish();
}

fn bench_scm_name_retrieval(c: &mut Criterion) {
    let mut group = c.benchmark_group("SCMIdentification");

    let temp_dir = tempfile::tempdir().unwrap();

    group.bench_function("git_detector_name", |b| {
        let detector = GitDetector::new(temp_dir.path());
        b.iter(|| {
            let name = detector.scm_name();
            black_box(name);
        })
    });

    group.bench_function("hash_detector_name", |b| {
        let detector = HashBasedDetector::new(temp_dir.path());
        b.iter(|| {
            let name = detector.scm_name();
            black_box(name);
        })
    });

    group.finish();
}

fn bench_file_hash_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("FileHashing");

    // Create test files of different sizes
    for size in [1024, 10240, 102400] {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let data = vec![0u8; size];
        fs::write(&temp_file, &data).unwrap();

        group.bench_function(format!("hash_{}bytes", size), |b| {
            b.iter(|| {
                let hash = HashBasedDetector::compute_file_hash(temp_file.path()).unwrap();
                black_box(hash);
            })
        });
    }

    group.finish();
}

fn bench_incremental_vs_full_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("IncrementalAnalysis");

    let temp_dir = tempfile::tempdir().unwrap();
    let project_root = temp_dir.path().to_path_buf();

    // Create 1000 files
    for i in 0..1000 {
        let file_path = project_root.join(format!("src/file_{}.rs", i));
        fs::create_dir_all(file_path.parent().unwrap()).unwrap();
        fs::write(&file_path, format!("// File {}", i)).unwrap();
    }

    let detector = HashBasedDetector::new(&project_root);

    group.bench_function("full_scan_1000_files", |b| {
        b.iter(|| {
            let changes = detector.detect_changed_files(None).unwrap();
            black_box(changes.len());
        })
    });

    // Now simulate incremental - only 10 files changed
    drop(detector);
    for i in 0..10 {
        let file_path = project_root.join(format!("src/file_{}.rs", i));
        fs::write(&file_path, format!("// Modified File {}", i)).unwrap();
    }

    let detector2 = HashBasedDetector::new(&project_root);

    group.bench_function("incremental_scan_10_changes", |b| {
        b.iter(|| {
            let changes = detector2.detect_changed_files(None).unwrap();
            black_box(changes.len());
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_factory_auto_detection,
    bench_hash_based_detector,
    bench_changed_file_operations,
    bench_git_detector_applicability,
    bench_scm_name_retrieval,
    bench_file_hash_computation,
    bench_incremental_vs_full_analysis
);
criterion_main!(benches);
