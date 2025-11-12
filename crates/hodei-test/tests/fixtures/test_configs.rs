//! Test configuration fixtures

use hodei_test::domain::models::{TestConfig, TestCase, ExpectedFinding};

/// Simple test configuration
pub fn simple_test_config() -> TestConfig {
    TestConfig {
        rule: "password_strength.hodei".to_string(),
        language: "hodei-dsl".to_string(),
        cases: vec![TestCase {
            name: "Strong password".to_string(),
            code: "function validate(pwd) { return pwd.length >= 12; }".to_string(),
            expected_findings: vec![],
        }],
    }
}

/// Test configuration with expected findings
pub fn test_config_with_findings() -> TestConfig {
    TestConfig {
        rule: "sql_injection.hodei".to_string(),
        language: "hodei-dsl".to_string(),
        cases: vec![
            TestCase {
                name: "Vulnerable code".to_string(),
                code: "SELECT * FROM users WHERE id = \" + userInput".to_string(),
                expected_findings: vec![ExpectedFinding {
                    finding_type: "Vulnerability".to_string(),
                    severity: "Critical".to_string(),
                    message: "SQL injection detected".to_string(),
                }],
            },
            TestCase {
                name: "Safe code".to_string(),
                code: "SELECT * FROM users WHERE id = ?".to_string(),
                expected_findings: vec![],
            },
        ],
    }
}

/// Multiple test cases configuration
pub fn multi_case_config() -> TestConfig {
    TestConfig {
        rule: "auth_rules.hodei".to_string(),
        language: "hodei-dsl".to_string(),
        cases: vec![
            TestCase {
                name: "Case 1".to_string(),
                code: "code1".to_string(),
                expected_findings: vec![],
            },
            TestCase {
                name: "Case 2".to_string(),
                code: "code2".to_string(),
                expected_findings: vec![
                    ExpectedFinding {
                        finding_type: "CodeSmell".to_string(),
                        severity: "Minor".to_string(),
                        message: "Test finding".to_string(),
                    }
                ],
            },
            TestCase {
                name: "Case 3".to_string(),
                code: "code3".to_string(),
                expected_findings: vec![],
            },
        ],
    }
}

/// Empty test configuration
pub fn empty_test_config() -> TestConfig {
    TestConfig {
        rule: "empty.hodei".to_string(),
        language: "hodei-dsl".to_string(),
        cases: vec![],
    }
}

/// YAML content for testing
pub const SAMPLE_TEST_YAML: &str = r#"
rule: "password_strength.hodei"
language: "hodei-dsl"

cases:
  - name: "Strong password"
    code: |
      function validatePassword(pwd: string): boolean {
        if (pwd.length >= 12 && pwd.matches(/[A-Z]/) && pwd.matches(/[0-9]/)) {
          return true;
        }
        return false;
      }
    expected_findings: []

  - name: "Weak password - too short"
    code: |
      function validatePassword(pwd: string): boolean {
        return pwd.length >= 8;  // Too short!
      }
    expected_findings:
      - finding_type: "CodeSmell"
        severity: "Major"
        message: "Password too weak"

  - name: "Weak password - no numbers"
    code: |
      function validatePassword(pwd: string): boolean {
        return pwd.length >= 12;  // Missing numbers!
      }
    expected_findings:
      - finding_type: "CodeSmell"
        severity: "Major"
        message: "Password missing numbers"
"#;

/// Creates a test config with specific number of cases
pub fn create_test_config_with_cases(num_cases: usize) -> TestConfig {
    let cases: Vec<TestCase> = (0..num_cases)
        .map(|i| TestCase {
            name: format!("Test case {}", i),
            code: format!("code {}", i),
            expected_findings: if i % 2 == 0 {
                vec![ExpectedFinding {
                    finding_type: "Vulnerability".to_string(),
                    severity: "Major".to_string(),
                    message: format!("Finding {}", i),
                }]
            } else {
                vec![]
            },
        })
        .collect();
    
    TestConfig {
        rule: "test_rule.hodei".to_string(),
        language: "hodei-dsl".to_string(),
        cases,
    }
}
