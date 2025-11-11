//! Comprehensive integration tests for Ruff Adapter
//!
//! These tests verify that the Ruff adapter correctly:
//! - Parses Ruff's JSON output from various scenarios
//! - Converts Ruff diagnostics to hodei-ir Facts
//! - Handles edge cases and error conditions
//! - Integrates properly with the hodei-ir type system

use hodei_engine::extractor::ruff_adapter::{RuffAdapter, RuffConfig};
use hodei_ir::{Confidence, FactType, Severity};
use std::path::PathBuf;

#[tokio::test]
async fn test_parse_ruff_json_error() {
    let adapter = RuffAdapter::new();
    let ruff_json = r#"[
        {
            "filename": "test.py",
            "code": "E501",
            "rule": "E501",
            "message": "Line too long",
            "severity": "error",
            "line": 10,
            "column": 80,
            "endLine": 10,
            "endColumn": 85
        }
    ]"#;

    let project_root = PathBuf::from("/project");
    let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

    assert_eq!(facts.len(), 1);
    let fact = &facts[0];

    assert!(matches!(fact.fact_type, FactType::Vulnerability { .. }));
    assert_eq!(fact.location.start_line.get(), 10);
    assert_eq!(
        fact.location.start_column.as_ref().map(|c| c.get()),
        Some(80)
    );

    if let FactType::Vulnerability { description, .. } = &fact.fact_type {
        assert!(description.contains("Line too long"));
        assert!(description.contains("E501"));
    }
}

#[tokio::test]
async fn test_parse_ruff_json_warning() {
    let adapter = RuffAdapter::new();
    let ruff_json = r#"[
        {
            "filename": "test.py",
            "code": "F401",
            "rule": "F401",
            "message": "Unused import",
            "severity": "warning",
            "line": 5,
            "column": 1
        }
    ]"#;

    let project_root = PathBuf::from("/project");
    let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

    assert_eq!(facts.len(), 1);
    let fact = &facts[0];

    assert!(matches!(fact.fact_type, FactType::Vulnerability { .. }));
    if let FactType::Vulnerability {
        owasp_category,
        severity,
        ..
    } = &fact.fact_type
    {
        assert_eq!(owasp_category.as_ref().unwrap(), "ruff-F401");
        assert_eq!(*severity, Severity::Major);
    }
}

#[tokio::test]
async fn test_parse_ruff_json_multiple() {
    let adapter = RuffAdapter::new();
    let ruff_json = r#"[
        {
            "filename": "test.py",
            "code": "E501",
            "rule": "E501",
            "message": "Line too long",
            "severity": "error",
            "line": 10
        },
        {
            "filename": "test.py",
            "code": "F401",
            "rule": "F401",
            "message": "Unused import",
            "severity": "warning",
            "line": 5
        },
        {
            "filename": "utils.py",
            "code": "E302",
            "rule": "E302",
            "message": "Expected 2 blank lines",
            "severity": "info",
            "line": 1
        }
    ]"#;

    let project_root = PathBuf::from("/project");
    let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

    assert_eq!(facts.len(), 3);

    // Verify error severity
    assert!(matches!(facts[0].fact_type, FactType::Vulnerability { .. }));
    if let FactType::Vulnerability { severity, .. } = &facts[0].fact_type {
        assert_eq!(*severity, Severity::Critical);
    }

    // Verify warning severity
    assert!(matches!(facts[1].fact_type, FactType::Vulnerability { .. }));
    if let FactType::Vulnerability { severity, .. } = &facts[1].fact_type {
        assert_eq!(*severity, Severity::Major);
    }

    // Verify info severity
    assert!(matches!(facts[2].fact_type, FactType::Vulnerability { .. }));
    if let FactType::Vulnerability { severity, .. } = &facts[2].fact_type {
        assert_eq!(*severity, Severity::Minor);
    }
}

#[tokio::test]
async fn test_parse_ruff_json_with_relative_paths() {
    let adapter = RuffAdapter::new();
    let ruff_json = r#"[
        {
            "filename": "src/module.py",
            "code": "F401",
            "rule": "F401",
            "message": "Unused import",
            "severity": "warning",
            "line": 5
        }
    ]"#;

    let project_root = PathBuf::from("/project");
    let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

    assert_eq!(facts.len(), 1);
    let fact = &facts[0];
    assert_eq!(fact.location.file.as_str(), "src/module.py");
}

#[tokio::test]
async fn test_parse_ruff_json_with_absolute_paths() {
    let adapter = RuffAdapter::new();
    let ruff_json = r#"[
        {
            "filename": "/home/user/project/src/module.py",
            "code": "F401",
            "rule": "F401",
            "message": "Unused import",
            "severity": "warning",
            "line": 5
        }
    ]"#;

    let project_root = PathBuf::from("/home/user/project");
    let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

    assert_eq!(facts.len(), 1);
    let fact = &facts[0];
    // Should be relative to project root
    assert_eq!(fact.location.file.as_str(), "src/module.py");
}

#[tokio::test]
async fn test_parse_ruff_json_without_columns() {
    let adapter = RuffAdapter::new();
    let ruff_json = r#"[
        {
            "filename": "test.py",
            "code": "E501",
            "rule": "E501",
            "message": "Line too long",
            "severity": "error",
            "line": 10
        }
    ]"#;

    let project_root = PathBuf::from("/project");
    let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

    assert_eq!(facts.len(), 1);
    let fact = &facts[0];
    // Column should be None when not provided
    assert!(fact.location.start_column.is_none());
    assert!(fact.location.end_column.is_none());
}

#[tokio::test]
async fn test_parse_ruff_json_with_end_positions() {
    let adapter = RuffAdapter::new();
    let ruff_json = r#"[
        {
            "filename": "test.py",
            "code": "E501",
            "rule": "E501",
            "message": "Line too long",
            "severity": "error",
            "line": 10,
            "column": 80,
            "endLine": 10,
            "endColumn": 90
        }
    ]"#;

    let project_root = PathBuf::from("/project");
    let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

    assert_eq!(facts.len(), 1);
    let fact = &facts[0];
    assert_eq!(fact.location.start_line.get(), 10);
    assert_eq!(fact.location.end_line.get(), 10);
    assert_eq!(fact.location.start_column.as_ref().unwrap().get(), 80);
    assert_eq!(fact.location.end_column.as_ref().unwrap().get(), 90);
}

#[tokio::test]
async fn test_parse_ruff_json_empty_array() {
    let adapter = RuffAdapter::new();
    let ruff_json = "[]";

    let project_root = PathBuf::from("/project");
    let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

    assert_eq!(facts.len(), 0);
}

#[tokio::test]
async fn test_parse_ruff_json_provenance() {
    let adapter = RuffAdapter::new();
    let ruff_json = r#"[
        {
            "filename": "test.py",
            "code": "E501",
            "rule": "E501",
            "message": "Line too long",
            "severity": "error",
            "line": 10
        }
    ]"#;

    let project_root = PathBuf::from("/project");
    let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

    assert_eq!(facts.len(), 1);
    let fact = &facts[0];

    // Verify provenance
    assert!(fact.provenance.version.starts_with("ruff-adapter-"));
    assert_eq!(fact.provenance.confidence.get(), 0.9);
}

#[tokio::test]
async fn test_parse_ruff_json_confidence() {
    let adapter = RuffAdapter::new();
    let ruff_json = r#"[
        {
            "filename": "test.py",
            "code": "E501",
            "rule": "E501",
            "message": "Line too long",
            "severity": "error",
            "line": 10
        }
    ]"#;

    let project_root = PathBuf::from("/project");
    let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

    assert_eq!(facts.len(), 1);
    let fact = &facts[0];

    // Ruff diagnostics should have high confidence
    assert!(fact.provenance.confidence.get() >= 0.8);
}

#[tokio::test]
async fn test_parse_ruff_json_rule_metadata() {
    let adapter = RuffAdapter::new();
    let ruff_json = r#"[
        {
            "filename": "test.py",
            "code": "E501",
            "rule": "E501",
            "message": "Line too long",
            "severity": "error",
            "line": 10,
            "url": "https://docs.astral.sh/ruff/rules/line-too-long/"
        }
    ]"#;

    let project_root = PathBuf::from("/project");
    let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

    assert_eq!(facts.len(), 1);
    let fact = &facts[0];

    // Verify that the rule code is embedded in the description
    if let FactType::Vulnerability {
        description,
        owasp_category,
        ..
    } = &fact.fact_type
    {
        assert!(description.contains("E501"));
        assert_eq!(owasp_category.as_ref().unwrap(), "ruff-E501");
    }
}

#[tokio::test]
async fn test_parse_ruff_json_multiline_fact() {
    let adapter = RuffAdapter::new();
    let ruff_json = r#"[
        {
            "filename": "test.py",
            "code": "E501",
            "rule": "E501",
            "message": "Line too long (87 > 79 characters)",
            "severity": "error",
            "line": 10,
            "column": 80
        }
    ]"#;

    let project_root = PathBuf::from("/project");
    let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

    assert_eq!(facts.len(), 1);
    let fact = &facts[0];

    // Verify message is preserved in description
    if let FactType::Vulnerability { description, .. } = &fact.fact_type {
        assert!(description.contains("Line too long"));
        assert!(description.contains("87 > 79 characters"));
    }
}

#[tokio::test]
async fn test_ruff_adapter_with_config() {
    let config = RuffConfig {
        max_parallel: 8,
        include_fixes: true,
    };
    let adapter = RuffAdapter::with_config(config);

    let ruff_json = r#"[
        {
            "filename": "test.py",
            "code": "E501",
            "rule": "E501",
            "message": "Line too long",
            "severity": "error",
            "line": 10
        }
    ]"#;

    let project_root = PathBuf::from("/project");
    let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

    assert_eq!(facts.len(), 1);
}

#[tokio::test]
async fn test_parse_ruff_json_different_severities() {
    let adapter = RuffAdapter::new();

    // Test error (Critical)
    let error_json = r#"[{
        "filename": "test.py",
        "code": "E501",
        "rule": "E501",
        "message": "Error",
        "severity": "error",
        "line": 1
    }]"#;

    let project_root = PathBuf::from("/project");
    let facts = adapter.parse_str(error_json, &project_root).await.unwrap();
    assert_eq!(facts.len(), 1);
    if let FactType::Vulnerability { severity, .. } = &facts[0].fact_type {
        assert_eq!(*severity, Severity::Critical);
    }

    // Test warning (Major)
    let warning_json = r#"[{
        "filename": "test.py",
        "code": "F401",
        "rule": "F401",
        "message": "Warning",
        "severity": "warning",
        "line": 1
    }]"#;

    let facts = adapter
        .parse_str(warning_json, &project_root)
        .await
        .unwrap();
    assert_eq!(facts.len(), 1);
    if let FactType::Vulnerability { severity, .. } = &facts[0].fact_type {
        assert_eq!(*severity, Severity::Major);
    }

    // Test info (Minor)
    let info_json = r#"[{
        "filename": "test.py",
        "code": "E302",
        "rule": "E302",
        "message": "Info",
        "severity": "info",
        "line": 1
    }]"#;

    let facts = adapter.parse_str(info_json, &project_root).await.unwrap();
    assert_eq!(facts.len(), 1);
    if let FactType::Vulnerability { severity, .. } = &facts[0].fact_type {
        assert_eq!(*severity, Severity::Minor);
    }
}

#[tokio::test]
async fn test_parse_invalid_json() {
    let adapter = RuffAdapter::new();
    let invalid_json = "{ this is not valid json }";

    let project_root = PathBuf::from("/project");
    let result = adapter.parse_str(invalid_json, &project_root).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_parse_ruff_json_multiple_files() {
    let adapter = RuffAdapter::new();
    let ruff_json = r#"[
        {
            "filename": "file1.py",
            "code": "E501",
            "rule": "E501",
            "message": "Line too long",
            "severity": "error",
            "line": 1
        },
        {
            "filename": "file2.py",
            "code": "E501",
            "rule": "E501",
            "message": "Line too long",
            "severity": "error",
            "line": 1
        },
        {
            "filename": "file1.py",
            "code": "F401",
            "rule": "F401",
            "message": "Unused import",
            "severity": "warning",
            "line": 2
        }
    ]"#;

    let project_root = PathBuf::from("/project");
    let facts = adapter.parse_str(ruff_json, &project_root).await.unwrap();

    assert_eq!(facts.len(), 3);

    // Count facts per file
    let file1_count = facts
        .iter()
        .filter(|f| f.location.file.as_str() == "file1.py")
        .count();
    let file2_count = facts
        .iter()
        .filter(|f| f.location.file.as_str() == "file2.py")
        .count();

    assert_eq!(file1_count, 2);
    assert_eq!(file2_count, 1);
}
