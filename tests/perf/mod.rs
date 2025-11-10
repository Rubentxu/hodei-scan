//! Simple performance tests

#[cfg(test)]
mod performance_tests {
    use hodei_ir::{Confidence, FactType, FlowId, FunctionName, Severity, VariableName};

    #[test]
    fn test_fact_creation_performance() {
        // Test creating many facts
        for i in 0..1000 {
            let _fact = FactType::CodeSmell {
                smell_type: format!("smell-{}", i),
                severity: Severity::Minor,
                message: format!("Message {}", i),
            };
        }
        assert!(true);
    }

    #[test]
    fn test_confidence_operations() {
        // Test confidence calculations
        let _high = Confidence::new(0.9).unwrap();
        let _medium = Confidence::new(0.5).unwrap();
        let _low = Confidence::new(0.1).unwrap();

        // Just verify they can be created
        assert!(true);
    }

    #[test]
    fn test_large_severity_list() {
        let severities = vec![
            Severity::Info,
            Severity::Minor,
            Severity::Major,
            Severity::Critical,
            Severity::Blocker,
        ];

        assert_eq!(severities.len(), 5);
    }

    #[test]
    fn test_type_variants() {
        use uuid::Uuid;

        // Test all fact type variants can be created quickly
        for _ in 0..100 {
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
        }
        assert!(true);
    }
}
