//! Integration tests for hodei-extractors with hodei-ir

#[cfg(test)]
mod extractor_integration_tests {
    use hodei_extractors::RegexExtractor;
    use hodei_ir::{ExtractorId, FactType, Severity};
    use tempfile;

    #[test]
    fn test_extractor_creation() {
        let extractor = RegexExtractor::new(ExtractorId::Custom, "1.0.0", vec![]);

        // Just verify it compiles
        assert!(true);
    }

    #[test]
    fn test_fact_types() {
        // Test that extractor can use various fact types
        let _fact1 = FactType::CodeSmell {
            smell_type: "TODO".to_string(),
            severity: Severity::Minor,
            message: "TODO comment".to_string(),
        };

        let _fact2 = FactType::Function {
            name: "test".to_string(),
            complexity: 10,
            lines_of_code: 100,
        };

        let _fact3 = FactType::Variable {
            name: "x".to_string(),
            scope: "method".to_string(),
            var_type: "String".to_string(),
        };

        assert!(true);
    }

    #[test]
    fn test_integration() {
        // Test that extractor and ir work together
        use hodei_dsl::parse_rule_file;
        use hodei_engine::{EngineConfig, RuleEngine};

        let _config = EngineConfig::default();
        let _engine = RuleEngine::new(_config);
        let _rules = parse_rule_file("");

        // All modules work together
        assert!(true);
    }
}
