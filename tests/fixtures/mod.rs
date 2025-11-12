//! Test Fixtures for hodei-scan
//!
//! This module provides reusable test fixtures for unit, integration, and E2E tests

pub mod java;
pub mod rules;
pub mod projects;

pub use java::*;
pub use rules::*;
pub use projects::*;

/// Sample IR data in JSON format for testing
pub const SAMPLE_JSON: &str = r#"{
  "findings": [
    {
      "fact_type": "Vulnerability",
      "message": "SQL injection vulnerability detected",
      "location": "src/auth/login.js:42",
      "severity": "Critical",
      "metadata": {
        "cwe": "CWE-89",
        "confidence": "0.95"
      }
    },
    {
      "fact_type": "CodeSmell",
      "message": "Unused variable 'temp'",
      "location": "src/utils/helpers.js:15",
      "severity": "Minor",
      "metadata": {}
    }
  ]
}"#;

/// Sample IR data in YAML format for testing
pub const SAMPLE_YAML: &str = r#"findings:
  - fact_type: "Vulnerability"
    message: "SQL injection vulnerability detected"
    location: "src/auth/login.js:42"
    severity: "Critical"
    metadata:
      cwe: "CWE-89"
      confidence: "0.95"
  - fact_type: "CodeSmell"
    message: "Unused variable 'temp'"
    location: "src/utils/helpers.js:15"
    severity: "Minor"
    metadata: {}
"#;

/// Create a FindingSet with multiple findings for testing
pub fn multiple_findings_ir() -> hodei_ir::FindingSet {
    hodei_ir::FindingSet {
        findings: vec![
            hodei_ir::Finding {
                fact_type: "Vulnerability".to_string(),
                message: "Finding 1".to_string(),
                location: Some("file1.js:1".to_string()),
                severity: Some("Critical".to_string()),
                metadata: std::collections::HashMap::from([
                    ("cwe".to_string(), "CWE-79".to_string()),
                ]),
            },
            hodei_ir::Finding {
                fact_type: "CodeSmell".to_string(),
                message: "Finding 2".to_string(),
                location: Some("file2.js:5".to_string()),
                severity: Some("Minor".to_string()),
                metadata: std::collections::HashMap::new(),
            },
        ],
    }
}

/// Create an empty FindingSet for testing
pub fn empty_ir() -> hodei_ir::FindingSet {
    hodei_ir::FindingSet {
        findings: Vec::new(),
    }
}

/// Large FindingSet for performance testing
pub fn large_ir(count: usize) -> hodei_ir::FindingSet {
    let mut findings = Vec::with_capacity(count);
    for i in 0..count {
        findings.push(hodei_ir::Finding {
            fact_type: format!("Type{}", i % 3),
            message: format!("Finding {}", i),
            location: Some(format!("file{}.js:{}", i, i)),
            severity: Some("Major".to_string()),
            metadata: std::collections::HashMap::new(),
        });
    }
    hodei_ir::FindingSet { findings }
}

/// Sample test configuration YAML
pub const SAMPLE_TEST_CONFIG: &str = r#"
rule: "test_rule.hodei"
language: "hodei-dsl"

cases:
  - name: "Test case 1"
    code: |
      function test() {
        // TODO: implement
      }
    expected_findings:
      - finding_type: "CodeSmell"
        severity: "Minor"
        message: "TODO comment found"

  - name: "Test case 2"
    code: |
      function test() {
        // Clean implementation
      }
    expected_findings: []
"#;

/// Sample quality gates configuration
pub const SAMPLE_QUALITY_GATES: &str = r#"
quality_gates:
  - name: "Security Gate"
    description: "Blocks critical security vulnerabilities"
    enabled: true
    rules:
      - "SQL Injection"
      - "XSS"
    fail_conditions:
      - severity: "Critical"
        count: 0
      - severity: "High"
        count: 1

  - name: "Code Quality Gate"
    description: "Enforces code quality"
    enabled: true
    rules:
      - "TODO"
      - "FIXME"
    fail_conditions:
      - severity: "Major"
        count: 0

  - name: "Test Coverage Gate"
    description: "Ensures test coverage"
    enabled: true
    rules:
      - "LowTestCoverage"
    fail_conditions:
      - coverage: 80
        count: 1
"#;
