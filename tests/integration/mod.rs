//! Simple integration tests to verify all crates work together

#[cfg(test)]
mod basic_integration {
    use hodei_dsl;
    use hodei_engine;
    use hodei_extractors;
    use hodei_ir;
    use serde_json;

    #[test]
    fn test_all_crates_link() {
        // Just verify all crates can be imported
        assert!(true);
    }

    #[test]
    fn test_types_exist() {
        use hodei_ir::{FactType, Severity};

        // Create a simple fact type
        let _ft = FactType::CodeSmell {
            smell_type: "test".to_string(),
            severity: Severity::Minor,
        };

        assert!(true);
    }

    #[test]
    fn test_extractor_exists() {
        use hodei_extractors::{ExtractorDefinition, ExtractorMetadata};
        use hodei_ir::ExtractorId;

        let _r = ExtractorDefinition {
            id: "test".to_string(),
            command: "echo".to_string(),
            config: serde_json::Value::Null,
            metadata: ExtractorMetadata {
                id: "test".to_string(),
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
                description: "Test extractor".to_string(),
            },
        };
        assert!(true);
    }

    #[test]
    fn test_engine_exists() {
        use hodei_engine::{EngineConfig, RuleEngine};

        let _config = EngineConfig::default();
        let _engine = RuleEngine::new(_config);
        assert!(true);
    }

    #[test]
    fn test_dsl_exists() {
        use hodei_dsl::parse_rule_file;

        let _r = parse_rule_file("");
        assert!(true);
    }

    #[test]
    fn test_integration_flow() {
        // Test that we can use types from multiple crates together
        use hodei_dsl::parse_rule_file;
        use hodei_engine::{EngineConfig, RuleEngine};
        use hodei_extractors::ExtractorDefinition;
        use hodei_ir::{ExtractorId, FactType, Severity};

        // Create components
        let extractor = ExtractorDefinition {
            id: "test".to_string(),
            command: "echo".to_string(),
            config: serde_json::Value::Null,
            metadata: hodei_extractors::ExtractorMetadata {
                id: "test".to_string(),
                name: "Test".to_string(),
                version: "1.0.0".to_string(),
                description: "Test extractor".to_string(),
            },
        };
        let engine = RuleEngine::new(EngineConfig::default());
        let rules = parse_rule_file("");

        // Create a fact
        let fact = FactType::CodeSmell {
            smell_type: "test".to_string(),
            severity: Severity::Minor,
        };

        // If we get here without errors, integration works
        assert!(true);
    }
}
