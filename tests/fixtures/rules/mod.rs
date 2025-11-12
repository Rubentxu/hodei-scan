//! Rule fixtures for testing
//!
//! These fixtures contain sample Hodei DSL rules for testing

/// Simple rule that detects TODO comments
pub const TODO_DETECTOR_RULE: &str = r#"
rule "TODO Comment Detector" {
    description: "Detects TODO comments in code"
    severity: "Minor"
    tags: ["maintenance", "documentation"]

    match {
        pattern: Comment {
            text contains "TODO"
        }
    }

    emit CodeSmell {
        type: "todo_comment",
        message: "TODO comment found - needs attention",
        severity: "Minor"
    }
}
"#;

/// Simple rule that detects FIXME comments
pub const FIXME_DETECTOR_RULE: &str = r#"
rule "FIXME Comment Detector" {
    description: "Detects FIXME comments in code"
    severity: "Major"
    tags: ["maintenance", "bug"]

    match {
        pattern: Comment {
            text contains "FIXME"
        }
    }

    emit Vulnerability {
        type: "fixme_comment",
        message: "FIXME comment found - known issue",
        severity: "Major"
    }
}
"#;

/// Rule that detects System.out.println usage
pub const SYSOUT_DETECTOR_RULE: &str = r#"
rule "System.out.println Detector" {
    description: "Detects System.out.println usage in production code"
    severity: "Major"
    tags: ["logging", "production"]

    match {
        pattern: MethodCall {
            target contains "System.out.println"
        }
    }

    emit CodeSmell {
        type: "print_statement",
        message: "System.out.println should not be in production code",
        severity: "Major"
    }
}
"#;

/// Rule that detects hardcoded credentials
pub const HARDCODED_CREDENTIALS_RULE: &str = r#"
rule "Hardcoded Credentials Detector" {
    description: "Detects hardcoded passwords and API keys"
    severity: "Critical"
    tags: ["security", "credentials"]

    match {
        pattern: FieldDeclaration {
            type contains "String"
            modifiers contains "private static final"
            name matches "(PASSWORD|API_KEY|SECRET|TOKEN)"
        }
    }

    emit Vulnerability {
        type: "hardcoded_credentials",
        message: "Hardcoded credentials detected",
        severity: "Critical"
    }
}
"#;

/// Rule that detects SQL injection vulnerabilities
pub const SQL_INJECTION_RULE: &str = r#"
rule "SQL Injection Detector" {
    description: "Detects SQL injection vulnerabilities"
    severity: "Critical"
    tags: ["security", "injection"]

    match {
        pattern: MethodCall {
            target contains "createStatement"
            or {
                pattern: StringConcatenation {
                    contains "SELECT"
                    contains "FROM"
                    contains variable
                }
            }
        }
    }

    emit Vulnerability {
        type: "sql_injection",
        message: "Potential SQL injection vulnerability",
        severity: "Critical"
    }
}
"#;

/// Rule that detects XSS vulnerabilities
pub const XSS_RULE: &str = r#"
rule "XSS Detector" {
    description: "Detects potential XSS vulnerabilities"
    severity: "Major"
    tags: ["security", "xss"]

    match {
        pattern: MethodCall {
            target contains "println"
            or {
                pattern: StringConcatenation {
                    contains "<"
                    contains ">"
                }
            }
        }
    }

    emit Vulnerability {
        type: "xss",
        message: "Potential XSS vulnerability",
        severity: "Major"
    }
}
"#;

/// Rule that detects long methods
pub const LONG_METHOD_RULE: &str = r#"
rule "Long Method Detector" {
    description: "Detects methods that are too long"
    severity: "Minor"
    tags: ["code-quality", "maintainability"]

    match {
        pattern: MethodDeclaration {
            statements count > 15
        }
    }

    emit CodeSmell {
        type: "long_method",
        message: "Method is too long - consider refactoring",
        severity: "Minor"
    }
}
"#;

/// Rule that detects deprecated API usage
pub const DEPRECATED_API_RULE: &str = r#"
rule "Deprecated API Detector" {
    description: "Detects usage of deprecated APIs"
    severity: "Major"
    tags: ["compatibility", "maintenance"]

    match {
        pattern: MethodCall {
            modifiers contains "@Deprecated"
        }
    }

    emit CodeSmell {
        type: "deprecated_api",
        message: "Using deprecated API",
        severity: "Major"
    }
}
"#;

/// Composite rule file with multiple rules
pub const COMPLETE_RULESET: &str = r#"
rule "TODO Comment Detector" {
    description: "Detects TODO comments in code"
    severity: "Minor"
    tags: ["maintenance", "documentation"]

    match {
        pattern: Comment {
            text contains "TODO"
        }
    }

    emit CodeSmell {
        type: "todo_comment",
        message: "TODO comment found - needs attention",
        severity: "Minor"
    }
}

rule "FIXME Comment Detector" {
    description: "Detects FIXME comments in code"
    severity: "Major"
    tags: ["maintenance", "bug"]

    match {
        pattern: Comment {
            text contains "FIXME"
        }
    }

    emit Vulnerability {
        type: "fixme_comment",
        message: "FIXME comment found - known issue",
        severity: "Major"
    }
}

rule "System.out.println Detector" {
    description: "Detects System.out.println usage in production code"
    severity: "Major"
    tags: ["logging", "production"]

    match {
        pattern: MethodCall {
            target contains "System.out.println"
        }
    }

    emit CodeSmell {
        type: "print_statement",
        message: "System.out.println should not be in production code",
        severity: "Major"
    }
}

rule "Hardcoded Credentials Detector" {
    description: "Detects hardcoded passwords and API keys"
    severity: "Critical"
    tags: ["security", "credentials"]

    match {
        pattern: FieldDeclaration {
            type contains "String"
            modifiers contains "private static final"
            name matches "(PASSWORD|API_KEY|SECRET|TOKEN)"
        }
    }

    emit Vulnerability {
        type: "hardcoded_credentials",
        message: "Hardcoded credentials detected",
        severity: "Critical"
    }
}

rule "SQL Injection Detector" {
    description: "Detects SQL injection vulnerabilities"
    severity: "Critical"
    tags: ["security", "injection"]

    match {
        pattern: MethodCall {
            target contains "createStatement"
        }
    }

    emit Vulnerability {
        type: "sql_injection",
        message: "Potential SQL injection vulnerability",
        severity: "Critical"
    }
}
"#;

/// Simple rule for testing
pub const SIMPLE_TEST_RULE: &str = r#"
rule "Simple Test Rule" {
    description: "Simple rule for testing"
    severity: "Minor"

    match {
        pattern: Comment {
            text contains "TEST"
        }
    }

    emit CodeSmell {
        type: "test_finding",
        message: "Test finding",
        severity: "Minor"
    }
}
"#;

/// Rule with complex pattern matching
pub const COMPLEX_PATTERN_RULE: &str = r#"
rule "Complex Pattern Rule" {
    description: "Rule with complex pattern matching"
    severity: "Major"
    tags: ["testing", "complex"]

    match {
        or: [
            {
                pattern: MethodCall {
                    target contains "System.out"
                }
            },
            {
                pattern: Comment {
                    text contains "TODO"
                }
            },
            {
                pattern: Comment {
                    text contains "FIXME"
                }
            }
        ]
    }

    emit CodeSmell {
        type: "complex_pattern",
        message: "Complex pattern matched",
        severity: "Major"
    }
}
"#;

/// Rule with multiple emit statements
pub const MULTI_EMIT_RULE: &str = r#"
rule "Multi Emit Rule" {
    description: "Rule that can emit multiple findings"
    severity: "Major"

    match {
        pattern: ClassDeclaration {
            name contains "Test"
        }
    }

    emit CodeSmell {
        type: "test_class",
        message: "Test class found",
        severity: "Minor"
    }

    emit Vulnerability {
        type: "test_vulnerability",
        message: "Potential issue in test class",
        severity: "Major"
    }
}
"#;

/// Rule with metadata
pub const METADATA_RULE: &str = r#"
rule "Metadata Rule" {
    description: "Rule with metadata"
    severity: "Major"
    tags: ["testing", "metadata"]
    metadata: {
        author: "test",
        version: "1.0",
        category: "testing"
    }

    match {
        pattern: Comment {
            text contains "METADATA"
        }
    }

    emit CodeSmell {
        type: "metadata_finding",
        message: "Finding with metadata",
        severity: "Major"
        metadata: {
            confidence: "0.95",
            false_positive_rate: "0.05"
        }
    }
}
"#;
