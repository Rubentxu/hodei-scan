//! Builders for test data generation
//!
//! This module provides builder patterns for creating test objects easily

use std::collections::HashMap;

/// Builder for Finding objects
pub struct FindingBuilder {
    fact_type: String,
    message: String,
    location: Option<String>,
    severity: Option<String>,
    metadata: HashMap<String, String>,
}

impl Default for FindingBuilder {
    fn default() -> Self {
        Self {
            fact_type: "Vulnerability".to_string(),
            message: "Test finding".to_string(),
            location: None,
            severity: Some("Major".to_string()),
            metadata: HashMap::new(),
        }
    }
}

impl FindingBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fact_type(mut self, fact_type: &str) -> Self {
        self.fact_type = fact_type.to_string();
        self
    }

    pub fn with_message(mut self, message: &str) -> Self {
        self.message = message.to_string();
        self
    }

    pub fn with_location(mut self, location: &str) -> Self {
        self.location = Some(location.to_string());
        self
    }

    pub fn with_severity(mut self, severity: &str) -> Self {
        self.severity = Some(severity.to_string());
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_metadata_map(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn with_no_location(mut self) -> Self {
        self.location = None;
        self
    }

    pub fn with_no_severity(mut self) -> Self {
        self.severity = None;
        self
    }

    pub fn build(self) -> hodei_ir::Finding {
        hodei_ir::Finding {
            fact_type: self.fact_type,
            message: self.message,
            location: self.location,
            severity: self.severity,
            metadata: self.metadata,
        }
    }

    pub fn build_vec(self, count: usize) -> Vec<hodei_ir::Finding> {
        vec![self.build(); count]
    }
}

/// Builder for FindingSet objects
pub struct FindingSetBuilder {
    findings: Vec<hodei_ir::Finding>,
}

impl Default for FindingSetBuilder {
    fn default() -> Self {
        Self {
            findings: Vec::new(),
        }
    }
}

impl FindingSetBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_finding(mut self, finding: hodei_ir::Finding) -> Self {
        self.findings.push(finding);
        self
    }

    pub fn with_findings(mut self, findings: Vec<hodei_ir::Finding>) -> Self {
        self.findings.extend(findings);
        self
    }

    pub fn with_vulnerability(mut self, message: &str) -> Self {
        self.findings.push(
            FindingBuilder::new()
                .with_fact_type("Vulnerability")
                .with_message(message)
                .build()
        );
        self
    }

    pub fn with_code_smell(mut self, message: &str) -> Self {
        self.findings.push(
            FindingBuilder::new()
                .with_fact_type("CodeSmell")
                .with_message(message)
                .build()
        );
        self
    }

    pub fn with_critical_vulnerability(mut self, message: &str, location: &str) -> Self {
        self.findings.push(
            FindingBuilder::new()
                .with_fact_type("Vulnerability")
                .with_message(message)
                .with_location(location)
                .with_severity("Critical")
                .build()
        );
        self
    }

    pub fn with_multiple(mut self, count: usize, fact_type: &str) -> Self {
        for i in 0..count {
            self.findings.push(
                FindingBuilder::new()
                    .with_fact_type(fact_type)
                    .with_message(&format!("Finding {}", i))
                    .build()
            );
        }
        self
    }

    pub fn build(self) -> hodei_ir::FindingSet {
        hodei_ir::FindingSet {
            findings: self.findings,
        }
    }
}

/// Builder for Document objects (LSP)
pub struct DocumentBuilder {
    uri: String,
    content: String,
    version: i32,
}

impl Default for DocumentBuilder {
    fn default() -> Self {
        Self {
            uri: "file:///test.hodei".to_string(),
            content: String::new(),
            version: 1,
        }
    }
}

impl DocumentBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_uri(mut self, uri: &str) -> Self {
        self.uri = uri.to_string();
        self
    }

    pub fn with_content(mut self, content: &str) -> Self {
        self.content = content.to_string();
        self
    }

    pub fn with_version(mut self, version: i32) -> Self {
        self.version = version;
        self
    }

    pub fn with_rule_content(mut self) -> Self {
        self.content = r#"
rule "Test Rule" {
    description: "Test rule"
    severity: "Major"

    match {
        pattern: Comment {
            text contains "TODO"
        }
    }

    emit CodeSmell {
        type: "todo"
        message: "TODO found"
    }
}
"#.to_string();
        self
    }

    pub fn build(self) -> hodei_dsl_lsp::domain::models::Document {
        hodei_dsl_lsp::domain::models::Document {
            uri: self.uri,
            content: self.content,
            version: self.version,
        }
    }
}

/// Builder for TestCase objects
pub struct TestCaseBuilder {
    name: String,
    code: String,
    expected_findings: Vec<hodei_ir::Finding>,
}

impl Default for TestCaseBuilder {
    fn default() -> Self {
        Self {
            name: "Test Case".to_string(),
            code: "test code".to_string(),
            expected_findings: Vec::new(),
        }
    }
}

impl TestCaseBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_code(mut self, code: &str) -> Self {
        self.code = code.to_string();
        self
    }

    pub fn with_expected_finding(mut self, finding: hodei_ir::Finding) -> Self {
        self.expected_findings.push(finding);
        self
    }

    pub fn with_expected_findings(mut self, findings: Vec<hodei_ir::Finding>) -> Self {
        self.expected_findings.extend(findings);
        self
    }

    pub fn with_no_findings(mut self) -> Self {
        self.expected_findings.clear();
        self
    }

    pub fn with_vulnerability_expectation(mut self, message: &str) -> Self {
        self.expected_findings.push(
            FindingBuilder::new()
                .with_fact_type("Vulnerability")
                .with_message(message)
                .build()
        );
        self
    }

    pub fn build(self) -> hodei_test::domain::models::TestCase {
        hodei_test::domain::models::TestCase {
            name: self.name,
            code: self.code,
            expected_findings: self.expected_findings,
        }
    }
}

/// Builder for TestConfig objects
pub struct TestConfigBuilder {
    rule: String,
    language: String,
    cases: Vec<hodei_test::domain::models::TestCase>,
}

impl Default for TestConfigBuilder {
    fn default() -> Self {
        Self {
            rule: "test_rule.hodei".to_string(),
            language: "hodei-dsl".to_string(),
            cases: Vec::new(),
        }
    }
}

impl TestConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rule(mut self, rule: &str) -> Self {
        self.rule = rule.to_string();
        self
    }

    pub fn with_language(mut self, language: &str) -> Self {
        self.language = language.to_string();
        self
    }

    pub fn with_case(mut self, test_case: hodei_test::domain::models::TestCase) -> Self {
        self.cases.push(test_case);
        self
    }

    pub fn with_cases(mut self, cases: Vec<hodei_test::domain::models::TestCase>) -> Self {
        self.cases.extend(cases);
        self
    }

    pub fn with_single_case(mut self, name: &str, code: &str, has_findings: bool) -> Self {
        let finding = if has_findings {
            vec![FindingBuilder::new()
                .with_fact_type("Vulnerability")
                .with_message("Expected finding")
                .build()]
        } else {
            Vec::new()
        };

        self.cases.push(
            TestCaseBuilder::new()
                .with_name(name)
                .with_code(code)
                .with_expected_findings(finding)
                .build()
        );
        self
    }

    pub fn build(self) -> hodei_test::domain::models::TestConfig {
        hodei_test::domain::models::TestConfig {
            rule: self.rule,
            language: self.language,
            cases: self.cases,
        }
    }
}

/// Builder for CompletionItem objects
pub struct CompletionItemBuilder {
    label: String,
    kind: Option<hodei_dsl_lsp::domain::models::CompletionItemKind>,
    detail: Option<String>,
    documentation: Option<String>,
}

impl Default for CompletionItemBuilder {
    fn default() -> Self {
        Self {
            label: "completion".to_string(),
            kind: Some(hodei_dsl_lsp::domain::models::CompletionItemKind::Keyword),
            detail: None,
            documentation: None,
        }
    }
}

impl CompletionItemBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.label = label.to_string();
        self
    }

    pub fn with_kind(mut self, kind: hodei_dsl_lsp::domain::models::CompletionItemKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn with_detail(mut self, detail: &str) -> Self {
        self.detail = Some(detail.to_string());
        self
    }

    pub fn with_documentation(mut self, documentation: &str) -> Self {
        self.documentation = Some(documentation.to_string());
        self
    }

    pub fn build(self) -> hodei_dsl_lsp::domain::models::CompletionItem {
        hodei_dsl_lsp::domain::models::CompletionItem {
            label: self.label,
            kind: self.kind,
            detail: self.detail,
            documentation: self.documentation,
        }
    }

    pub fn build_vec(self, count: usize) -> Vec<hodei_dsl_lsp::domain::models::CompletionItem> {
        vec![self.build(); count]
    }
}

/// Builder for TestResults objects
pub struct TestResultsBuilder {
    results: Vec<hodei_test::domain::models::TestCaseResult>,
}

impl Default for TestResultsBuilder {
    fn default() -> Self {
        Self {
            results: Vec::new(),
        }
    }
}

impl TestResultsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_result(mut self, result: hodei_test::domain::models::TestCaseResult) -> Self {
        self.results.push(result);
        self
    }

    pub fn with_passed_test(mut self, name: &str) -> Self {
        self.results.push(
            hodei_test::domain::models::TestCaseResult {
                name: name.to_string(),
                passed: true,
                assertions: Vec::new(),
            }
        );
        self
    }

    pub fn with_failed_test(mut self, name: &str) -> Self {
        self.results.push(
            hodei_test::domain::models::TestCaseResult {
                name: name.to_string(),
                passed: false,
                assertions: Vec::new(),
            }
        );
        self
    }

    pub fn with_multiple_passed(mut self, count: usize) -> Self {
        for i in 0..count {
            self.results.push(
                hodei_test::domain::models::TestCaseResult {
                    name: format!("Test {}", i),
                    passed: true,
                    assertions: Vec::new(),
                }
            );
        }
        self
    }

    pub fn build(self) -> hodei_test::domain::models::TestResults {
        let mut results = hodei_test::domain::models::TestResults::new();
        for result in self.results {
            results.add_result(result);
        }
        results
    }
}

/// Helper to generate random-looking test data
pub struct TestDataGenerator {
    counter: usize,
}

impl Default for TestDataGenerator {
    fn default() -> Self {
        Self { counter: 0 }
    }
}

impl TestDataGenerator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_finding(&mut self) -> hodei_ir::Finding {
        self.counter += 1;
        FindingBuilder::new()
            .with_fact_type(&format!("Type{}", self.counter % 3))
            .with_message(&format!("Finding {}", self.counter))
            .with_location(&format!("file{}.java:{}", self.counter, self.counter))
            .with_severity(&match self.counter % 4 {
                0 => "Critical",
                1 => "Major",
                2 => "Minor",
                _ => "Info",
            })
            .build()
    }

    pub fn next_document(&mut self) -> hodei_dsl_lsp::domain::models::Document {
        self.counter += 1;
        DocumentBuilder::new()
            .with_uri(&format!("file:///test{}.hodei", self.counter))
            .with_content(&format!("rule test{} {{ }}", self.counter))
            .build()
    }

    pub fn next_test_case(&mut self) -> hodei_test::domain::models::TestCase {
        self.counter += 1;
        TestCaseBuilder::new()
            .with_name(&format!("Test Case {}", self.counter))
            .with_code(&format!("// Test code {}", self.counter))
            .build()
    }
}
