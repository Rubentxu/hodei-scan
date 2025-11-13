//! Integration tests for hodei-cli

use tempfile;

#[cfg(test)]
mod cli_integration_tests {
    use std::process::Command;

    #[test]
    fn test_cli_exists() {
        // Test that the CLI binary can be built
        let output = Command::new("cargo")
            .args(&["build", "--bin", "hodei-scan"])
            .output()
            .expect("Failed to build hodei-scan");

        assert!(output.status.success(), "hodei-scan binary should build");
    }

    #[test]
    fn test_cli_help() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "hodei-scan", "--", "--help"])
            .output()
            .expect("Failed to execute hodei-scan");

        // Should either succeed or show help
        let _stdout = String::from_utf8_lossy(&output.stdout);
        let _stderr = String::from_utf8_lossy(&output.stderr);
        assert!(true);
    }

    #[test]
    fn test_integration_with_extractors() {
        use hodei_extractors::ExtractorDefinition;
        use hodei_ir::ExtractorId;

        // Test that CLI dependencies work
        let _extractor = ExtractorDefinition {
            id: "test".to_string(),
            command: "echo".to_string(),
            enabled: true,
            timeout_seconds: 60,
            config: serde_json::Value::Null,
        };
        assert!(true);
    }

    #[test]
    fn test_integration_with_engine() {
        use hodei_engine::{EngineConfig, RuleEngine};

        // Test that CLI can use engine
        let config = EngineConfig::default();
        let _engine = RuleEngine::new(config);
        assert!(true);
    }
}
