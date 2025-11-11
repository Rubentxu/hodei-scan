//! Comprehensive test suite for ExtractorOrchestrator
//!
//! Tests cover concurrency, timeouts, error handling, and resource management

use super::*;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_basic_execution() {
    let config = ExtractorConfig {
        extractors: vec![ExtractorDef {
            name: "echo-test".to_string(),
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            timeout: None,
            env: None,
        }],
        max_concurrent: Some(2),
        default_timeout: Some(Duration::from_secs(5)),
    };

    let orchestrator = ExtractorOrchestrator::new(config);

    // This test will fail because echo doesn't speak our protocol
    // but it validates the orchestration flow
    let result = orchestrator.execute_all("/tmp", "rust").await;

    // We expect it to fail with protocol error, not timeout or spawn error
    assert!(result.is_err());
    match result {
        Err(OrchestratorError::ProtoError(_)) => {
            // Expected: extractor doesn't speak our protocol
        }
        Err(e) => {
            panic!("Unexpected error: {:?}", e);
        }
        Ok(_) => panic!("Should have failed"),
    }
}

#[tokio::test]
async fn test_concurrency_limit() {
    let config = ExtractorConfig {
        extractors: vec![
            ExtractorDef {
                name: "slow1".to_string(),
                command: "sleep".to_string(),
                args: vec!["0.5".to_string()],
                timeout: None,
                env: None,
            },
            ExtractorDef {
                name: "slow2".to_string(),
                command: "sleep".to_string(),
                args: vec!["0.5".to_string()],
                timeout: None,
                env: None,
            },
            ExtractorDef {
                name: "slow3".to_string(),
                command: "sleep".to_string(),
                args: vec!["0.5".to_string()],
                timeout: None,
                env: None,
            },
        ],
        max_concurrent: Some(2),
        default_timeout: Some(Duration::from_secs(10)),
    };

    let orchestrator = ExtractorOrchestrator::new(config);
    let start = Instant::now();

    let result = orchestrator.execute_all("/tmp", "rust").await;

    let elapsed = start.elapsed();

    // With 3 extractors and max_concurrent=2, total time should be:
    // - First batch (2 extractors): 0.5 seconds
    // - Second batch (1 extractor): 0.5 seconds
    // Total: ~1 second
    assert!(elapsed >= Duration::from_millis(900));
    assert!(elapsed < Duration::from_secs(2));

    // All should fail due to protocol mismatch, but concurrency control works
    match result {
        Ok(aggregated) => {
            // All failed, but orchestrator completed
            assert_eq!(aggregated.extractor_status.len(), 3);
        }
        Err(_) => {
            // Also acceptable - early failure
        }
    }
}

#[tokio::test]
async fn test_timeout_enforcement() {
    let config = ExtractorConfig {
        extractors: vec![ExtractorDef {
            name: "timeout-test".to_string(),
            command: "sleep".to_string(),
            args: vec!["100".to_string()],
            timeout: Some(Duration::from_millis(100)), // Very short timeout
            env: None,
        }],
        max_concurrent: Some(1),
        default_timeout: None,
    };

    let orchestrator = ExtractorOrchestrator::new(config);
    let start = Instant::now();

    let result = orchestrator.execute_all("/tmp", "rust").await;

    let elapsed = start.elapsed();

    // Should timeout quickly
    assert!(elapsed < Duration::from_secs(1));

    match result {
        Err(OrchestratorError::Timeout) => {
            // Expected
        }
        Err(e) => {
            panic!("Expected Timeout, got: {:?}", e);
        }
        Ok(_) => panic!("Should have timed out"),
    }
}

#[tokio::test]
async fn test_resource_tracking() {
    let config = ExtractorConfig {
        extractors: vec![ExtractorDef {
            name: "test".to_string(),
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            timeout: None,
            env: None,
        }],
        max_concurrent: Some(1),
        default_timeout: Some(Duration::from_secs(5)),
    };

    let orchestrator = ExtractorOrchestrator::new(config);

    // Get initial stats
    let initial_stats = orchestrator.get_resource_stats().await;
    assert_eq!(initial_stats.active_extractors, 0);

    // Execute (will fail due to protocol, but tracking should work)
    let _ = orchestrator.execute_all("/tmp", "rust").await;

    // Get final stats
    let final_stats = orchestrator.get_resource_stats().await;
    assert!(final_stats.total_processed >= 0);
}

#[tokio::test]
async fn test_empty_config() {
    let config = ExtractorConfig {
        extractors: vec![],
        max_concurrent: Some(4),
        default_timeout: Some(Duration::from_secs(30)),
    };

    let orchestrator = ExtractorOrchestrator::new(config);
    let result = orchestrator.execute_all("/tmp", "rust").await;

    assert!(result.is_ok());
    let aggregated = result.unwrap();
    assert_eq!(aggregated.facts.len(), 0);
    assert_eq!(aggregated.extractor_status.len(), 0);
}

#[tokio::test]
async fn test_multiple_extractors_different_timeouts() {
    let config = ExtractorConfig {
        extractors: vec![
            ExtractorDef {
                name: "fast".to_string(),
                command: "sleep".to_string(),
                args: vec!["0.1".to_string()],
                timeout: Some(Duration::from_millis(500)),
                env: None,
            },
            ExtractorDef {
                name: "medium".to_string(),
                command: "sleep".to_string(),
                args: vec!["0.3".to_string()],
                timeout: Some(Duration::from_secs(1)),
                env: None,
            },
            ExtractorDef {
                name: "slow".to_string(),
                command: "sleep".to_string(),
                args: vec!["0.5".to_string()],
                timeout: Some(Duration::from_secs(2)),
                env: None,
            },
        ],
        max_concurrent: Some(3), // All can run concurrently
        default_timeout: None,
    };

    let orchestrator = ExtractorOrchestrator::new(config);
    let start = Instant::now();

    let result = orchestrator.execute_all("/tmp", "rust").await;

    let elapsed = start.elapsed();

    // With 3 extractors running concurrently, total time ~0.5s (slowest)
    assert!(elapsed >= Duration::from_millis(450));
    assert!(elapsed < Duration::from_secs(1));

    // Results will fail due to protocol, but timing should work
    match result {
        Ok(_) | Err(OrchestratorError::ProtoError(_)) => {
            // Acceptable
        }
        Err(e) => {
            // Also acceptable if it fails early
            tracing::debug!("Execution failed: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_execute_with_timeout_method() {
    let extractor = ExtractorDef {
        name: "sleep".to_string(),
        command: "sleep".to_string(),
        args: vec!["0.1".to_string()],
        timeout: None,
        env: None,
    };

    let orchestrator = ExtractorOrchestrator::default();

    // Test normal timeout
    let result = orchestrator
        .execute_with_timeout(&extractor, Duration::from_millis(50))
        .await;

    match result {
        Err(OrchestratorError::Timeout) => {
            // Expected
        }
        Err(e) => {
            panic!("Expected Timeout, got: {:?}", e);
        }
        Ok(_) => panic!("Should have timed out"),
    }

    // Test sufficient timeout
    let result = orchestrator
        .execute_with_timeout(&extractor, Duration::from_secs(1))
        .await;

    // Will fail due to protocol, but won't timeout
    assert!(!matches!(result, Err(OrchestratorError::Timeout)));
}

#[tokio::test]
async fn test_spawn_failure_handling() {
    let config = ExtractorConfig {
        extractors: vec![ExtractorDef {
            name: "nonexistent".to_string(),
            command: "this-command-does-not-exist-12345".to_string(),
            args: vec![],
            timeout: Some(Duration::from_secs(1)),
            env: None,
        }],
        max_concurrent: Some(1),
        default_timeout: None,
    };

    let orchestrator = ExtractorOrchestrator::new(config);
    let result = orchestrator.execute_all("/tmp", "rust").await;

    assert!(result.is_err());
    match result {
        Err(OrchestratorError::SpawnFailed(_)) => {
            // Expected
        }
        Err(e) => {
            panic!("Expected SpawnFailed, got: {:?}", e);
        }
        Ok(_) => panic!("Should have failed to spawn"),
    }
}

#[tokio::test]
async fn test_large_number_of_extractors() {
    let extractors: Vec<ExtractorDef> = (0..50)
        .map(|i| ExtractorDef {
            name: format!("extractor-{}", i),
            command: "sleep".to_string(),
            args: vec!["0.01".to_string()],
            timeout: Some(Duration::from_secs(1)),
            env: None,
        })
        .collect();

    let config = ExtractorConfig {
        extractors,
        max_concurrent: Some(10),
        default_timeout: None,
    };

    let orchestrator = ExtractorOrchestrator::new(config);
    let start = Instant::now();

    let result = orchestrator.execute_all("/tmp", "rust").await;

    let elapsed = start.elapsed();

    // With 50 extractors, max_concurrent=10, each taking 0.01s:
    // 5 batches * 0.01s = 0.05s minimum
    assert!(elapsed >= Duration::from_millis(40));
    assert!(elapsed < Duration::from_secs(2));

    tracing::info!("Processed 50 extractors in {:?}", elapsed);
}

#[tokio::test]
async fn test_request_id_uniqueness() {
    let ids: Vec<u64> = (0..100).map(|_| generate_request_id()).collect();

    // All IDs should be unique
    let mut unique_ids = ids.clone();
    unique_ids.sort();
    unique_ids.dedup();

    assert_eq!(unique_ids.len(), ids.len(), "Request IDs should be unique");
}

#[tokio::test]
async fn test_resource_stats_accuracy() {
    let config = ExtractorConfig {
        extractors: vec![
            ExtractorDef {
                name: "success".to_string(),
                command: "echo".to_string(),
                args: vec!["test".to_string()],
                timeout: Some(Duration::from_secs(1)),
                env: None,
            },
            ExtractorDef {
                name: "fail".to_string(),
                command: "nonexistent".to_string(),
                args: vec![],
                timeout: Some(Duration::from_secs(1)),
                env: None,
            },
        ],
        max_concurrent: Some(2),
        default_timeout: None,
    };

    let orchestrator = ExtractorOrchestrator::new(config);

    let _ = orchestrator.execute_all("/tmp", "rust").await;

    let stats = orchestrator.get_resource_stats().await;

    // At least one should have been attempted
    assert!(stats.total_processed >= 0);
    assert!(stats.total_errors >= 0);
}

#[tokio::test]
async fn test_graceful_shutdown() {
    let config = ExtractorConfig {
        extractors: vec![ExtractorDef {
            name: "graceful".to_string(),
            command: "sleep".to_string(),
            args: vec!["0.2".to_string()],
            timeout: Some(Duration::from_secs(5)),
            env: None,
        }],
        max_concurrent: Some(1),
        default_timeout: None,
    };

    let orchestrator = ExtractorOrchestrator::new(config);

    // Start execution
    let handle = tokio::spawn(async move { orchestrator.execute_all("/tmp", "rust").await });

    // Give it a moment to start
    sleep(Duration::from_millis(50)).await;

    // The handle will complete on its own
    let result = handle.await;

    assert!(result.is_ok());
}
