//! Test utilities module
//!
//! This module provides utilities for writing tests including mocks, builders,
//! and helper functions

mod mocks;
mod builders;
mod helpers;

pub use mocks::*;
pub use builders::*;
pub use helpers::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_builder() {
        let finding = FindingBuilder::new()
            .with_fact_type("TestType")
            .with_message("Test message")
            .with_location("test.java:1")
            .with_severity("Major")
            .build();

        assert_eq!(finding.fact_type, "TestType");
        assert_eq!(finding.message, "Test message");
        assert_eq!(finding.location, Some("test.java:1".to_string()));
        assert_eq!(finding.severity, Some("Major".to_string()));
    }

    #[test]
    fn test_finding_set_builder() {
        let finding_set = FindingSetBuilder::new()
            .with_vulnerability("Vuln 1")
            .with_code_smell("Smell 1")
            .build();

        assert_eq!(finding_set.findings.len(), 2);
        assert_eq!(finding_set.findings[0].fact_type, "Vulnerability");
        assert_eq!(finding_set.findings[1].fact_type, "CodeSmell");
    }

    #[test]
    fn test_document_builder() {
        let doc = DocumentBuilder::new()
            .with_uri("file:///test.hodei")
            .with_content("test content")
            .with_version(2)
            .build();

        assert_eq!(doc.uri, "file:///test.hodei");
        assert_eq!(doc.content, "test content");
        assert_eq!(doc.version, 2);
    }

    #[test]
    fn test_test_case_builder() {
        let test_case = TestCaseBuilder::new()
            .with_name("Test 1")
            .with_code("test code")
            .with_vulnerability_expectation("Expected vulns")
            .build();

        assert_eq!(test_case.name, "Test 1");
        assert_eq!(test_case.code, "test code");
        assert_eq!(test_case.expected_findings.len(), 1);
    }

    #[test]
    fn test_test_config_builder() {
        let config = TestConfigBuilder::new()
            .with_rule("rule.hodei")
            .with_language("hodei-dsl")
            .with_single_case("Case 1", "code", false)
            .with_single_case("Case 2", "code", true)
            .build();

        assert_eq!(config.rule, "rule.hodei");
        assert_eq!(config.language, "hodei-dsl");
        assert_eq!(config.cases.len(), 2);
    }

    #[test]
    fn test_completion_item_builder() {
        let item = CompletionItemBuilder::new()
            .with_label("keyword")
            .with_detail("Keyword detail")
            .build();

        assert_eq!(item.label, "keyword");
        assert_eq!(item.detail, Some("Keyword detail".to_string()));
    }

    #[test]
    fn test_test_results_builder() {
        let results = TestResultsBuilder::new()
            .with_passed_test("Test 1")
            .with_failed_test("Test 2")
            .build();

        assert_eq!(results.total_count(), 2);
        assert_eq!(results.passed_count(), 1);
        assert_eq!(results.failed_count(), 1);
    }

    #[test]
    fn test_test_data_generator() {
        let mut gen = TestDataGenerator::new();
        let finding1 = gen.next_finding();
        let finding2 = gen.next_finding();

        assert_ne!(finding1.message, finding2.message);
    }

    #[test]
    fn test_finding_matches() {
        let finding = hodei_ir::Finding {
            fact_type: "Vulnerability".to_string(),
            message: "Test finding".to_string(),
            location: Some("test.java:1".to_string()),
            severity: Some("Major".to_string()),
            metadata: std::collections::HashMap::new(),
        };

        assert!(finding_matches(&finding, Some("Vulnerability"), None, None));
        assert!(finding_matches(&finding, None, Some("Test"), None));
        assert!(finding_matches(&finding, None, None, Some("Major")));
        assert!(finding_matches(&finding, Some("Vulnerability"), Some("Test"), Some("Major")));
    }

    #[test]
    fn test_assertions() {
        let finding_set = hodei_ir::FindingSet {
            findings: vec![
                hodei_ir::Finding {
                    fact_type: "Vulnerability".to_string(),
                    message: "Finding 1".to_string(),
                    location: Some("file1.java:1".to_string()),
                    severity: Some("Major".to_string()),
                    metadata: std::collections::HashMap::new(),
                },
                hodei_ir::Finding {
                    fact_type: "CodeSmell".to_string(),
                    message: "Finding 2".to_string(),
                    location: Some("file2.java:2".to_string()),
                    severity: Some("Minor".to_string()),
                    metadata: std::collections::HashMap::new(),
                },
            ],
        };

        assert_finding_exists(&finding_set, "Finding 1");
        assert_finding_type_exists(&finding_set, "Vulnerability");
        assert_finding_count(&finding_set, 2);

        let empty_set = hodei_ir::FindingSet {
            findings: Vec::new(),
        };
        assert_no_findings(&empty_set);
    }

    #[test]
    fn test_assert_multiline_string_eq() {
        assert_multiline_string_eq("Line 1\nLine 2", "Line 1\nLine 2");
    }

    #[test]
    fn test_assert_contains_case_insensitive() {
        assert_contains_case_insensitive("Hello World", "hello");
        assert_contains_case_insensitive("Hello World", "WORLD");
    }

    #[test]
    fn test_assert_approximately_equal() {
        assert_approximately_equal(1.0, 1.1, 0.2);
        assert_approximately_equal(1.0, 1.0001, 0.001);
    }
}
