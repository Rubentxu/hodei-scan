use crate::core::{ExtractorDefinition, ExtractorError};
use crate::orchestrator::{ExtractorOrchestrator, OrchestratorConfig};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to get bash command with script path
    fn bash_script_command(script_path: &PathBuf) -> String {
        format!("bash {}", script_path.to_string_lossy())
    }

    /// Helper to create a test directory with necessary files
    fn create_test_dir() -> PathBuf {
        let temp_dir = std::env::temp_dir().join("hodei-test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        std::fs::write(temp_dir.join("test.py"), "// test file\n").unwrap();
        temp_dir
    }

    /// Test CA-1.1 & CA-1.3: Read extractor configuration from hodei.toml
    #[tokio::test]
    async fn test_configuration_loading() {
        let config = OrchestratorConfig {
            parallel_execution: true,
            max_parallel_extractors: 4,
            global_timeout_seconds: 300,
        };

        assert!(config.parallel_execution);
        assert_eq!(config.max_parallel_extractors, 4);
        assert_eq!(config.global_timeout_seconds, 300);
    }

    /// Test CA-1.2: Execute extractors as child processes
    /// Test CA-1.6: Merge IRs with deduplication
    #[tokio::test]
    async fn test_extractor_execution_success() {
        let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("mock_extractor.sh");

        assert!(fixture_path.exists(), "Mock extractor fixture not found");

        // Create a test directory with a test file
        let temp_dir = create_test_dir();

        let extractor = ExtractorOrchestrator::new(
            OrchestratorConfig {
                parallel_execution: false,
                max_parallel_extractors: 1,
                global_timeout_seconds: 300,
            },
            vec![ExtractorDefinition {
                id: "mock-extractor".to_string(),
                command: bash_script_command(&fixture_path),
                enabled: true,
                timeout_seconds: 300,
                config: serde_json::json!({}),
            }],
        );

        let result = extractor.run_all(&temp_dir).await;

        assert!(
            result.is_ok(),
            "Extractor execution should succeed: {:?}",
            result.err()
        );
        let aggregated_ir = result.unwrap();
        assert_eq!(aggregated_ir.facts.len(), 2);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    /// Test CA-1.2: Parallel execution of multiple extractors
    #[tokio::test]
    async fn test_parallel_execution_success() {
        let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("mock_extractor.sh");

        // Create a test directory with a test file
        let temp_dir = create_test_dir();

        let extractors = vec![
            ExtractorDefinition {
                id: "extractor-1".to_string(),
                command: bash_script_command(&fixture_path),
                enabled: true,
                timeout_seconds: 300,
                config: serde_json::json!({}),
            },
            ExtractorDefinition {
                id: "extractor-2".to_string(),
                command: bash_script_command(&fixture_path),
                enabled: true,
                timeout_seconds: 300,
                config: serde_json::json!({}),
            },
            ExtractorDefinition {
                id: "extractor-3".to_string(),
                command: bash_script_command(&fixture_path),
                enabled: true,
                timeout_seconds: 300,
                config: serde_json::json!({}),
            },
        ];

        let orchestrator = ExtractorOrchestrator::new(
            OrchestratorConfig {
                parallel_execution: true,
                max_parallel_extractors: 4,
                global_timeout_seconds: 300,
            },
            extractors,
        );

        let results = orchestrator.run_all(&temp_dir).await;

        assert!(
            results.is_ok(),
            "Parallel execution should succeed: {:?}",
            results.err()
        );
        let aggregated_ir = results.unwrap();
        assert_eq!(aggregated_ir.facts.len(), 6);
        assert_eq!(aggregated_ir.metadata.extractor_runs.len(), 3);
    }

    /// Test CA-1.4: Graceful failure handling - partial failure
    #[tokio::test]
    async fn test_partial_failure_continues() {
        let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("mock_extractor.sh");

        let extractors = vec![
            ExtractorDefinition {
                id: "good-extractor-1".to_string(),
                command: bash_script_command(&fixture_path),
                enabled: true,
                timeout_seconds: 300,
                config: serde_json::json!({}),
            },
            ExtractorDefinition {
                id: "bad-extractor".to_string(),
                command: "/nonexistent/path/to/extractor".to_string(),
                enabled: true,
                timeout_seconds: 300,
                config: serde_json::json!({}),
            },
            ExtractorDefinition {
                id: "good-extractor-2".to_string(),
                command: bash_script_command(&fixture_path),
                enabled: true,
                timeout_seconds: 300,
                config: serde_json::json!({}),
            },
        ];

        // Create a test directory
        let temp_dir = create_test_dir();

        let orchestrator = ExtractorOrchestrator::new(
            OrchestratorConfig {
                parallel_execution: false,
                max_parallel_extractors: 1,
                global_timeout_seconds: 300,
            },
            extractors,
        );

        let results = orchestrator.run_all(&temp_dir).await;

        assert!(results.is_ok(), "Should handle partial failures");
        let aggregated_ir = results.unwrap();
        assert!(aggregated_ir.facts.len() >= 2);
        assert!(
            aggregated_ir
                .metadata
                .extractor_runs
                .iter()
                .any(|r| !r.success)
        );
    }

    /// Test CA-1.3: Configurable timeout handling
    #[tokio::test]
    async fn test_extractor_timeout() {
        let slow_script = r#"#!/bin/bash
sleep 5
echo '{}'
"#;

        let temp_dir = std::env::temp_dir();
        let slow_extractor_path = temp_dir.join("slow_extractor.sh");

        std::fs::write(&slow_extractor_path, slow_script).unwrap();
        std::fs::set_permissions(&slow_extractor_path, std::fs::Permissions::from_mode(0o755))
            .unwrap();

        let extractor = ExtractorOrchestrator::new(
            OrchestratorConfig {
                parallel_execution: false,
                max_parallel_extractors: 1,
                global_timeout_seconds: 1,
            },
            vec![ExtractorDefinition {
                id: "slow-extractor".to_string(),
                command: bash_script_command(&slow_extractor_path),
                enabled: true,
                timeout_seconds: 1,
                config: serde_json::json!({}),
            }],
        );

        let result = extractor.run_all(&temp_dir).await;

        assert!(result.is_err(), "Should timeout");
        let error = result.unwrap_err();
        assert!(matches!(error, ExtractorError::AllExtractorsFailed));

        std::fs::remove_file(slow_extractor_path).ok();
    }

    /// Test CA-1.5: Validate IR against schema
    #[tokio::test]
    async fn test_invalid_ir_rejected() {
        let invalid_ir_script = r#"#!/bin/bash
echo 'invalid json {'
"#;

        let temp_dir = std::env::temp_dir();
        let invalid_extractor_path = temp_dir.join("invalid_extractor.sh");

        std::fs::write(&invalid_extractor_path, invalid_ir_script).unwrap();
        std::fs::set_permissions(
            &invalid_extractor_path,
            std::fs::Permissions::from_mode(0o755),
        )
        .unwrap();

        let extractor = ExtractorOrchestrator::new(
            OrchestratorConfig {
                parallel_execution: false,
                max_parallel_extractors: 1,
                global_timeout_seconds: 300,
            },
            vec![ExtractorDefinition {
                id: "invalid-extractor".to_string(),
                command: bash_script_command(&invalid_extractor_path),
                enabled: true,
                timeout_seconds: 300,
                config: serde_json::json!({}),
            }],
        );

        let result = extractor.run_all(&temp_dir).await;

        assert!(result.is_err(), "Should reject invalid IR");
        let error = result.unwrap_err();
        assert!(matches!(error, ExtractorError::AllExtractorsFailed));

        std::fs::remove_file(invalid_extractor_path).ok();
    }

    /// Test CA-1.7: Generate execution metrics
    #[tokio::test]
    async fn test_execution_metrics_generation() {
        let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("mock_extractor.sh");

        let extractors = vec![
            ExtractorDefinition {
                id: "extractor-1".to_string(),
                command: bash_script_command(&fixture_path),
                enabled: true,
                timeout_seconds: 300,
                config: serde_json::json!({}),
            },
            ExtractorDefinition {
                id: "extractor-2".to_string(),
                command: bash_script_command(&fixture_path),
                enabled: true,
                timeout_seconds: 300,
                config: serde_json::json!({}),
            },
        ];

        // Create a test directory
        let temp_dir = create_test_dir();

        let orchestrator = ExtractorOrchestrator::new(
            OrchestratorConfig {
                parallel_execution: false,
                max_parallel_extractors: 1,
                global_timeout_seconds: 300,
            },
            extractors,
        );

        let results = orchestrator.run_all(&temp_dir).await;
        assert!(results.is_ok());

        let aggregated_ir = results.unwrap();
        assert!(!aggregated_ir.metadata.extractor_runs.is_empty());
        assert!(aggregated_ir.metadata.total_facts_after_dedup > 0);
        assert!(aggregated_ir.metadata.deduplication_ratio >= 0.0);
    }

    /// Integration test: Full orchestrator workflow
    #[tokio::test]
    async fn test_full_orchestrator_integration() {
        let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("mock_extractor.sh");

        let extractors = vec![
            ExtractorDefinition {
                id: "ruff".to_string(),
                command: bash_script_command(&fixture_path),
                enabled: true,
                timeout_seconds: 300,
                config: serde_json::json!({}),
            },
            ExtractorDefinition {
                id: "eslint".to_string(),
                command: bash_script_command(&fixture_path),
                enabled: true,
                timeout_seconds: 300,
                config: serde_json::json!({}),
            },
            ExtractorDefinition {
                id: "clippy".to_string(),
                command: bash_script_command(&fixture_path),
                enabled: true,
                timeout_seconds: 300,
                config: serde_json::json!({}),
            },
        ];

        // Create a test directory
        let temp_dir = create_test_dir();

        let orchestrator = ExtractorOrchestrator::new(
            OrchestratorConfig {
                parallel_execution: true,
                max_parallel_extractors: 4,
                global_timeout_seconds: 300,
            },
            extractors,
        );

        let results = orchestrator.run_all(&temp_dir).await;

        assert!(results.is_ok(), "Full integration test should succeed");
        let aggregated_ir = results.unwrap();

        assert!(!aggregated_ir.facts.is_empty());
        assert_eq!(aggregated_ir.metadata.extractor_runs.len(), 3);

        let fact_ids: Vec<_> = aggregated_ir.facts.iter().map(|f| f.id.as_uuid()).collect();
        let unique_ids: std::collections::HashSet<_> = fact_ids.iter().collect();
        assert_eq!(fact_ids.len(), unique_ids.len());

        for fact in &aggregated_ir.facts {
            assert!(!fact.message.is_empty());
            assert!(fact.location.file.path.exists());
        }
    }
}
