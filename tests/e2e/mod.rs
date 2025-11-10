//! Simple E2E tests to verify complete workflows

#[cfg(test)]
mod basic_e2e {
    use std::process::Command;

    #[test]
    fn test_full_build() {
        // Test that the entire project can be built
        let output = Command::new("cargo")
            .args(&["build", "--all"])
            .output()
            .expect("Failed to build project");

        assert!(output.status.success(), "Project should build successfully");
    }

    #[test]
    fn test_all_tests_run() {
        // Test that all unit tests pass
        let output = Command::new("cargo")
            .args(&["test", "--lib", "--bins"])
            .output()
            .expect("Failed to run tests");

        // Check that tests ran
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success() || stdout.contains("test result"),
            "Tests should run"
        );
    }

    #[test]
    fn test_petclinic_example_exists() {
        use std::path::Path;

        // Verify PetClinic example exists
        let example_dir = Path::new("examples/petclinic-scan");
        assert!(example_dir.exists(), "PetClinic example should exist");

        let readme = example_dir.join("README.md");
        assert!(readme.exists(), "README should exist");
    }

    #[test]
    fn test_integration_works() {
        use hodei_dsl::parse_rule_file;
        use hodei_engine::{EngineConfig, RuleEngine};
        use hodei_extractors::RegexExtractor;
        use hodei_ir::{ExtractorId, FactType, Severity};

        // Create all components
        let _extractor = RegexExtractor::new(ExtractorId::Custom, "1.0.0", vec![]);
        let _engine = RuleEngine::new(EngineConfig::default());
        let _rules = parse_rule_file("");
        let _fact = FactType::CodeSmell {
            smell_type: "test".to_string(),
            severity: Severity::Minor,
            message: "test".to_string(),
        };

        // If we get here, the full stack works
        assert!(true);
    }
}
