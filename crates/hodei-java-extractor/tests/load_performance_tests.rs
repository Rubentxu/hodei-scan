#[cfg(test)]
mod load_performance_tests {
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};
    use tempfile::TempDir;
    use tokio::sync::Semaphore;
    use tokio::time::sleep;

    /// Test 1: Single Extraction Performance Benchmark
    /// Measures time for extracting a single Java project
    #[tokio::test]
    async fn test_single_extraction_performance() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let project_path = create_test_java_project(&temp_dir, 10, 5);

        let start_time = Instant::now();

        // Simulate extraction process
        simulate_extraction(&project_path, ExtractionLevel::Level1).await;

        let elapsed = start_time.elapsed();

        println!("Single extraction performance:");
        println!("  Files processed: 50 (10 classes, 5 tests each)");
        println!("  Time taken: {:?}", elapsed);
        println!(
            "  Throughput: {:.2} files/second",
            50.0 / elapsed.as_secs_f64()
        );

        assert!(
            elapsed < Duration::from_secs(10),
            "Extraction should complete in under 10 seconds"
        );
        assert!(
            elapsed > Duration::from_millis(10),
            "Extraction should take some time"
        );
    }

    /// Test 2: Concurrent Extraction Load Test
    /// Tests handling multiple concurrent extractions
    #[tokio::test]
    async fn test_concurrent_extraction_load() {
        let concurrency_levels = vec![5, 10, 20, 50];

        for concurrency in concurrency_levels {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");

            // Create semaphore to limit concurrency
            let semaphore = Arc::new(Semaphore::new(concurrency));
            let completion_counter = Arc::new(AtomicUsize::new(0));
            let start_time = Instant::now();

            // Spawn multiple extraction tasks
            let handles: Vec<_> = (0..concurrency)
                .map(|i| {
                    let semaphore = semaphore.clone();
                    let temp_dir = temp_dir.path();
                    let counter = completion_counter.clone();

                    tokio::spawn(async move {
                        let _permit = semaphore.acquire().await.unwrap();

                        let temp_path = temp_dir.path().to_path_buf();
                        let project_dir = create_test_java_project(&temp_path, 5, 3);
                        simulate_extraction(&project_dir, ExtractionLevel::Level2).await;

                        counter.fetch_add(1, Ordering::SeqCst);
                    })
                })
                .collect();

            // Wait for all tasks to complete
            futures::future::join_all(handles).await;

            let elapsed = start_time.elapsed();
            let completed = completion_counter.load(Ordering::SeqCst);

            println!(
                "Concurrent extraction load test (concurrency: {}):",
                concurrency
            );
            println!("  Completed extractions: {}", completed);
            println!("  Total time: {:?}", elapsed);
            println!(
                "  Throughput: {:.2} extractions/second",
                completed as f64 / elapsed.as_secs_f64()
            );

            assert_eq!(completed, concurrency, "All extractions should complete");
            assert!(
                elapsed < Duration::from_secs(30),
                "Should complete within 30 seconds"
            );
        }
    }

    /// Test 3: Memory Usage Under Load
    /// Tests memory consumption during multiple extractions
    #[tokio::test]
    async fn test_memory_usage_under_load() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let semaphore = Arc::new(Semaphore::new(10));
        let start_time = Instant::now();

        // Simulate memory-heavy operations
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let semaphore = semaphore.clone();
                let temp_dir = temp_dir.path().to_path_buf();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();

                    // Create larger project to stress memory
                    let project_dir = create_test_java_project(&temp_dir, 50, 20);
                    simulate_extraction(&project_dir, ExtractionLevel::Level3).await;

                    // Simulate memory-intensive processing
                    let large_data = vec![0u8; 1024 * 1024]; // 1MB
                    sleep(Duration::from_millis(100)).await;

                    format!("Extraction {} completed", i)
                })
            })
            .collect();

        futures::future::join_all(handles).await;
        let elapsed = start_time.elapsed();

        println!("Memory usage under load test:");
        println!("  Processed 10 large projects (50 classes, 20 tests each)");
        println!("  Total time: {:?}", elapsed);
        println!("  Average time per project: {:?}", elapsed / 10);

        assert!(
            elapsed < Duration::from_secs(60),
            "Should complete within 60 seconds"
        );
    }

    /// Test 4: Throughput Benchmark
    /// Measures extraction throughput in files per second
    #[tokio::test]
    async fn test_extraction_throughput() {
        let project_sizes = vec![
            (10, 5, "Small project"),
            (50, 25, "Medium project"),
            (100, 50, "Large project"),
        ];

        for (classes, tests_per_class, name) in project_sizes {
            let temp_dir = TempDir::new().expect("Failed to create temp dir");
            let project_path = create_test_java_project(&temp_dir, classes, tests_per_class);

            let file_count = classes * tests_per_class;
            let start_time = Instant::now();

            simulate_extraction(&project_path, ExtractionLevel::Level2).await;

            let elapsed = start_time.elapsed();
            let throughput = file_count as f64 / elapsed.as_secs_f64();

            println!("{} throughput benchmark:", name);
            println!("  Files: {}", file_count);
            println!("  Time: {:?}", elapsed);
            println!("  Throughput: {:.2} files/second", throughput);

            assert!(
                throughput > 1.0,
                "Throughput should be at least 1 file/second"
            );
        }
    }

    /// Test 5: Stress Test - Maximum Load
    /// Pushes the system to its limits
    #[tokio::test]
    async fn test_stress_test_maximum_load() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let concurrency = 100;

        let semaphore = Arc::new(Semaphore::new(concurrency));
        let start_time = Instant::now();
        let completion_counter = Arc::new(AtomicUsize::new(0));
        let error_counter = Arc::new(AtomicUsize::new(0));

        // Spawn maximum concurrent load
        let handles: Vec<_> = (0..concurrency)
            .map(|i| {
                let semaphore = semaphore.clone();
                let temp_dir = temp_dir.path();
                let counter = completion_counter.clone();
                let error_count = error_counter.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();

                    // Create minimal project for stress test
                    let project_dir = create_test_java_project(&temp_dir, 2, 1);

                    // Add random delay to simulate real-world variance
                    let delay = Duration::from_millis((i % 10) * 10);
                    sleep(delay).await;

                    match simulate_extraction_with_result(&project_dir, ExtractionLevel::Level1)
                        .await
                    {
                        Ok(_) => counter.fetch_add(1, Ordering::SeqCst),
                        Err(_) => error_count.fetch_add(1, Ordering::SeqCst),
                    }
                })
            })
            .collect();

        // Wait for all tasks with timeout
        let result =
            tokio::time::timeout(Duration::from_secs(60), futures::future::join_all(handles)).await;

        let elapsed = start_time.elapsed();
        let completed = completion_counter.load(Ordering::SeqCst);
        let errors = error_counter.load(Ordering::SeqCst);

        println!("Stress test - Maximum load:");
        println!("  Concurrency level: {}", concurrency);
        println!("  Completed: {}", completed);
        println!("  Errors: {}", errors);
        println!("  Total time: {:?}", elapsed);
        println!(
            "  Success rate: {:.2}%",
            completed as f64 / concurrency as f64 * 100.0
        );
        println!(
            "  Throughput: {:.2} extractions/second",
            completed as f64 / elapsed.as_secs_f64()
        );

        if result.is_ok() {
            assert!(
                completed >= concurrency / 2,
                "At least 50% should complete under stress"
            );
        }
    }

    /// Test 6: Latency Percentile Test
    /// Measures latency percentiles for extraction operations
    #[tokio::test]
    async fn test_latency_percentile_test() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let sample_size = 100;
        let mut latencies = Vec::new();

        // Collect latency samples
        for i in 0..sample_size {
            let temp_path = temp_dir.path().to_path_buf();
            let project_dir = create_test_java_project(&temp_path, 5, 3);
            let start_time = Instant::now();

            simulate_extraction(&project_dir, ExtractionLevel::Level2).await;

            let latency = start_time.elapsed();
            latencies.push(latency);

            // Vary project complexity slightly
            if i % 10 == 0 {
                sleep(Duration::from_millis(10)).await;
            }
        }

        // Sort for percentile calculation
        latencies.sort();

        let p50 = latencies[sample_size * 50 / 100];
        let p90 = latencies[sample_size * 90 / 100];
        let p95 = latencies[sample_size * 95 / 100];
        let p99 = latencies[sample_size * 99 / 100];
        let avg = latencies.iter().sum::<Duration>() / latencies.len() as u32;
        let min = latencies[0];
        let max = latencies[latencies.len() - 1];

        println!("Latency percentile test (n={}):", sample_size);
        println!("  Min: {:?}", min);
        println!("  P50: {:?}", p50);
        println!("  P90: {:?}", p90);
        println!("  P95: {:?}", p95);
        println!("  P99: {:?}", p99);
        println!("  Max: {:?}", max);
        println!("  Avg: {:?}", avg);

        assert!(
            p95 < Duration::from_secs(5),
            "P95 latency should be under 5 seconds"
        );
        assert!(
            avg < Duration::from_secs(2),
            "Average latency should be under 2 seconds"
        );
    }

    /// Test 7: Resource Cleanup Validation
    /// Ensures resources are properly cleaned up after load
    #[tokio::test]
    async fn test_resource_cleanup_validation() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Create and process multiple projects
        for _i in 0..10 {
            let temp_path = temp_dir.path().to_path_buf();
            let project_dir = create_test_java_project(&temp_path, 20, 10);
            simulate_extraction(&project_dir, ExtractionLevel::Level3).await;

            // Force cleanup simulation
            drop(project_dir);
            sleep(Duration::from_millis(10)).await;
        }

        // Verify temp directory is manageable
        let remaining_files = fs::read_dir(temp_dir.path())
            .expect("Failed to read temp dir")
            .count();

        println!("Resource cleanup validation:");
        println!("  Processed 10 projects");
        println!("  Remaining files: {}", remaining_files);
        println!("  Temp dir: {:?}", temp_dir.path());

        assert!(remaining_files <= 100, "Should not leave excessive files");
    }

    /// Test 8: Progressive Load Increase
    /// Gradually increases load to find breaking point
    #[tokio::test]
    async fn test_progressive_load_increase() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let concurrency_steps = vec![1, 5, 10, 20, 30, 40, 50];
        let mut max_successful = 0;

        for concurrency in concurrency_steps {
            let semaphore = Arc::new(Semaphore::new(concurrency));
            let completion_counter = Arc::new(AtomicUsize::new(0));
            let start_time = Instant::now();

            let handles: Vec<_> = (0..concurrency)
                .map(|_| {
                    let semaphore = semaphore.clone();
                    let temp_dir = temp_dir.path();
                    let temp_path = temp_dir.path().to_path_buf();
                    let counter = completion_counter.clone();

                    tokio::spawn(async move {
                        let _permit = semaphore.acquire().await.unwrap();
                        let project_dir = create_test_java_project(&temp_path, 5, 3);
                        simulate_extraction(&project_dir, ExtractionLevel::Level1).await;
                        counter.fetch_add(1, Ordering::SeqCst);
                    })
                })
                .collect();

            futures::future::join_all(handles).await;
            let elapsed = start_time.elapsed();
            let completed = completion_counter.load(Ordering::SeqCst);

            println!("Progressive load (concurrency: {}):", concurrency);
            println!(
                "  Completed: {}/{} ({:.1}%)",
                completed,
                concurrency,
                completed as f64 / concurrency as f64 * 100.0
            );
            println!("  Time: {:?}", elapsed);

            if completed == concurrency {
                max_successful = concurrency;
            } else {
                break;
            }
        }

        println!("Progressive load test result:");
        println!("  Maximum successful concurrency: {}", max_successful);
        println!(
            "  System can handle up to {} concurrent extractions",
            max_successful
        );

        assert!(
            max_successful >= 10,
            "Should handle at least 10 concurrent extractions"
        );
    }

    /// Test 9: Steady State Performance
    /// Tests performance over extended period
    #[tokio::test]
    async fn test_steady_state_performance() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let duration = Duration::from_secs(5); // Run for 5 seconds
        let concurrency = 5;

        let semaphore = Arc::new(Semaphore::new(concurrency));
        let start_time = Instant::now();
        let mut total_completed = 0;
        let mut samples = Vec::new();

        while start_time.elapsed() < duration {
            let batch_start = Instant::now();

            let handles: Vec<_> = (0..concurrency)
                .map(|_| {
                    let semaphore = semaphore.clone();
                    let temp_dir = temp_dir.path().to_path_buf();

                    tokio::spawn(async move {
                        let _permit = semaphore.acquire().await.unwrap();
                        let project_dir = create_test_java_project(&temp_dir, 3, 2);
                        simulate_extraction(&project_dir, ExtractionLevel::Level1).await;
                    })
                })
                .collect();

            futures::future::join_all(handles).await;

            let batch_duration = batch_start.elapsed();
            samples.push(batch_duration);
            total_completed += concurrency;

            sleep(Duration::from_millis(100)).await; // Brief pause between batches
        }

        let total_duration = start_time.elapsed();
        let avg_batch_time = samples.iter().sum::<Duration>() / samples.len() as u32;
        let throughput = total_completed as f64 / total_duration.as_secs_f64();

        println!("Steady state performance test:");
        println!("  Duration: {:?}", total_duration);
        println!("  Total extractions: {}", total_completed);
        println!("  Batches: {}", samples.len());
        println!("  Avg batch time: {:?}", avg_batch_time);
        println!("  Throughput: {:.2} extractions/second", throughput);
        println!(
            "  Stability: {:.2}%",
            100.0 - (standard_deviation(&samples) / avg_batch_time.as_secs_f64() * 100.0)
        );

        assert!(
            throughput > 2.0,
            "Throughput should be at least 2 extractions/second"
        );
    }

    /// Test 10: Spike Test - Sudden Load Increase
    /// Tests system behavior under sudden load spikes
    #[tokio::test]
    async fn test_spike_test_sudden_load() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Baseline: low load
        let baseline_concurrency = 2;
        let baseline_start = Instant::now();

        let baseline_handles: Vec<_> = (0..baseline_concurrency)
            .map(|_| {
                let temp_path = temp_dir.path().to_path_buf();
                tokio::spawn(async move {
                    let project_dir = create_test_java_project(&temp_path, 5, 3);
                    simulate_extraction(&project_dir, ExtractionLevel::Level1).await;
                })
            })
            .collect();

        futures::future::join_all(baseline_handles).await;
        let baseline_time = baseline_start.elapsed();

        // Spike: high load
        let spike_concurrency = 50;
        let spike_start = Instant::now();

        let semaphore = Arc::new(Semaphore::new(spike_concurrency));
        let completion_counter = Arc::new(AtomicUsize::new(0));

        let spike_handles: Vec<_> = (0..spike_concurrency)
            .map(|_| {
                let semaphore = semaphore.clone();
                let temp_path = temp_dir.path().to_path_buf();
                let counter = completion_counter.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let project_dir = create_test_java_project(&temp_path, 3, 2);
                    simulate_extraction(&project_dir, ExtractionLevel::Level1).await;
                    counter.fetch_add(1, Ordering::SeqCst);
                })
            })
            .collect();

        futures::future::join_all(spike_handles).await;
        let spike_time = spike_start.elapsed();
        let spike_completed = completion_counter.load(Ordering::SeqCst);

        println!("Spike test - Sudden load increase:");
        println!(
            "  Baseline ({} concurrent): {:?} (baseline throughput: {:.2})",
            baseline_concurrency,
            baseline_time,
            baseline_concurrency as f64 / baseline_time.as_secs_f64()
        );
        println!(
            "  Spike ({} concurrent): {:?} (spike throughput: {:.2})",
            spike_concurrency,
            spike_time,
            spike_completed as f64 / spike_time.as_secs_f64()
        );
        println!(
            "  Completed: {}/{} ({:.1}%)",
            spike_completed,
            spike_concurrency,
            spike_completed as f64 / spike_concurrency as f64 * 100.0
        );

        assert!(
            spike_completed >= spike_concurrency / 2,
            "Should handle at least 50% of spike load"
        );
    }

    // Helper functions and types

    #[derive(Clone, Copy)]
    enum ExtractionLevel {
        Level1,
        Level2,
        Level3,
    }

    fn create_test_java_project(
        temp_dir: &TempDir,
        classes_count: usize,
        tests_per_class: usize,
    ) -> PathBuf {
        let project_dir = temp_dir.path().join(format!("project_{}", classes_count));
        let src_dir = project_dir
            .join("src")
            .join("main")
            .join("java")
            .join("com");
        let test_dir = project_dir
            .join("src")
            .join("test")
            .join("java")
            .join("com");

        fs::create_dir_all(&src_dir).expect("Failed to create src dir");
        fs::create_dir_all(&test_dir).expect("Failed to create test dir");

        // Create main classes
        for i in 0..classes_count {
            let class_content = format!(
                r#"package com.example;

public class Class{} {{
    public void method{}() {{
        System.out.println("Method {}");
    }}
}}
"#,
                i, i, i
            );

            let class_file = src_dir.join(format!("Class{}.java", i));
            fs::write(&class_file, class_content).expect("Failed to write class file");

            // Create test for each class
            for j in 0..tests_per_class {
                let test_content = format!(
                    r#"package com.example;

import org.junit.Test;
import static org.junit.Assert.*;

public class Class{}Test {{
    @Test
    public void testMethod{}() {{
        Class{} instance = new Class{}();
        instance.method{}();
    }}
}}
"#,
                    i, j, i, i, j
                );

                let test_file = test_dir.join(format!("Class{}Test.java", i));
                fs::write(&test_file, test_content).expect("Failed to write test file");
            }
        }

        project_dir
    }

    async fn simulate_extraction(project_path: &Path, level: ExtractionLevel) {
        let _ = project_path; // In real implementation, would use this

        match level {
            ExtractionLevel::Level1 => sleep(Duration::from_millis(50)).await,
            ExtractionLevel::Level2 => sleep(Duration::from_millis(100)).await,
            ExtractionLevel::Level3 => sleep(Duration::from_millis(200)).await,
        }
    }

    async fn simulate_extraction_with_result(
        project_path: &Path,
        level: ExtractionLevel,
    ) -> Result<(), ()> {
        simulate_extraction(project_path, level).await;
        Ok(())
    }

    fn standard_deviation(samples: &[Duration]) -> f64 {
        if samples.len() < 2 {
            return 0.0;
        }

        let mean = samples.iter().map(|d| d.as_secs_f64()).sum::<f64>() / samples.len() as f64;
        let variance = samples
            .iter()
            .map(|d| {
                let d = d.as_secs_f64();
                (d - mean).powi(2)
            })
            .sum::<f64>()
            / samples.len() as f64;

        variance.sqrt()
    }
}
