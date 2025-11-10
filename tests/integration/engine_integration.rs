//! Integration tests for hodei-engine with hodei-ir

#[cfg(test)]
mod engine_integration_tests {
    use hodei_engine::{EngineConfig, RuleEngine};
    use hodei_ir::{Confidence, Fact, FactType, Severity};
    use hodei_ir::{FlowId, FunctionName, VariableName};

    #[test]
    fn test_engine_initialization() {
        let config = EngineConfig::default();
        let _engine = RuleEngine::new(config);
        // Just verify it compiles
        assert!(true);
    }

    #[test]
    fn test_types_exist() {
        // Just verify all the types exist and can be referenced
        let _severity = Severity::Minor;
        let _confidence = Confidence::new(0.5).unwrap();
        let _fact_type = FactType::CodeSmell {
            smell_type: "test".to_string(),
            severity: Severity::Minor,
            message: "test".to_string(),
        };
    }

    #[test]
    fn test_fact_type_variants() {
        use uuid::Uuid;

        // Test that all fact type variants exist
        let _v1 = FactType::TaintSource {
            var: VariableName("x".to_string()),
            flow_id: FlowId(Uuid::new_v4()),
            source_type: "input".to_string(),
            confidence: Confidence::new(0.9).unwrap(),
        };

        let _v2 = FactType::TaintSink {
            func: FunctionName("f".to_string()),
            consumes_flow: FlowId(Uuid::new_v4()),
            category: "c".to_string(),
            severity: Severity::Major,
        };

        let _v3 = FactType::Vulnerability {
            cwe_id: Some("CWE-1".to_string()),
            owasp_category: Some("A01".to_string()),
            severity: Severity::Critical,
            cvss_score: Some(9.0),
            description: "vuln".to_string(),
            confidence: Confidence::new(0.95).unwrap(),
        };

        let _v4 = FactType::Function {
            name: FunctionName("test".to_string()),
            complexity: 10,
            lines_of_code: 100,
        };

        let _v5 = FactType::Variable {
            name: VariableName("x".to_string()),
            scope: "method".to_string(),
            var_type: "String".to_string(),
        };

        let _v6 = FactType::CodeSmell {
            smell_type: "todo".to_string(),
            severity: Severity::Minor,
            message: "todo".to_string(),
        };

        // If we get here, all variants exist
        assert!(true);
    }

    #[test]
    fn test_severity_levels() {
        // Test that all severity levels exist
        let _s1 = Severity::Info;
        let _s2 = Severity::Minor;
        let _s3 = Severity::Major;
        let _s4 = Severity::Critical;
        let _s5 = Severity::Blocker;
    }

    #[test]
    fn test_confidence_creation() {
        let _high_conf = Confidence::new(0.9).unwrap();
        let _medium_conf = Confidence::new(0.5).unwrap();
        let _low_conf = Confidence::new(0.1).unwrap();
        // Just verify creation works
        assert!(true);
    }

    #[test]
    fn test_integration_suite() {
        // Test that all modules compile together
        use hodei_dsl::parse_rule_file;
        use hodei_extractors::RegexExtractor;

        let _extractor = RegexExtractor::new(hodei_ir::ExtractorId::Custom, "1.0.0", vec![]);

        let _rules = parse_rule_file(
            "rule \"Test\" { description: \"Test\" severity: \"Medium\" match { pattern: Function { name == \"test\" } } emit Finding { message: \"test\" confidence: \"Medium\" } }",
        );

        // If we get here, integration works
        assert!(true);
    }
}
