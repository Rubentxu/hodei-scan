//! End-to-End tests for IR dump workflow

use ir_dump::{IRFormatter, IRReader};
use ir_dump::ir_formatter::Format;
use tempfile::TempDir;

// Simple sample IR JSON for testing
const SAMPLE_JSON: &str = r#"{
  "schema_version": "3.3.0",
  "metadata": {
    "name": "Test Project",
    "version": "1.0.0",
    "root_path": "/test"
  },
  "facts": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "fact_type": {
        "Vulnerability": {
          "cwe_id": null,
          "owasp_category": null,
          "severity": "Critical",
          "cvss_score": null,
          "description": "Test vulnerability",
          "confidence": 0.9
        }
      },
      "message": "Test vulnerability found",
      "location": {
        "file": {
          "path": "/test/file1.js"
        },
        "start_line": 1,
        "start_column": 1,
        "end_line": 1,
        "end_column": 10
      },
      "provenance": {
        "extractor": "Custom",
        "version": "1.0.0",
        "confidence": 0.9,
        "extracted_at": "2025-01-15T10:30:00Z"
      }
    },
    {
      "id": "550e8400-e29b-41d4-b716-446655440002",
      "fact_type": {
        "CodeSmell": {
          "smell_type": "test_smell",
          "severity": "Minor"
        }
      },
      "message": "Test code smell found",
      "location": {
        "file": {
          "path": "/test/file2.js"
        },
        "start_line": 5,
        "start_column": 1,
        "end_line": 5,
        "end_column": 20
      },
      "provenance": {
        "extractor": "Custom",
        "version": "1.0.0",
        "confidence": 0.8,
        "extracted_at": "2025-01-15T10:31:00Z"
      }
    }
  ]
}"#;

#[tokio::test]
async fn test_complete_ir_inspection_workflow() {
    // Complete workflow: create IR file, read it, format it in different ways
    let temp_dir = TempDir::new().unwrap();
    let ir_file = temp_dir.path().join("findings.json");

    // Create IR file
    tokio::fs::write(&ir_file, SAMPLE_JSON).await.unwrap();

    // Read IR
    let reader = IRReader::new();
    let ir = reader.read(&ir_file).await.unwrap();

    assert_eq!(ir.len(), 2);
    assert_eq!(ir[0].fact_type.discriminant(), hodei_ir::FactTypeDiscriminant::Vulnerability);
    assert_eq!(ir[1].fact_type.discriminant(), hodei_ir::FactTypeDiscriminant::CodeSmell);

    // Format as JSON
    let formatter = IRFormatter::new();
    let json_output = formatter.format(&ir, &Format::Json).unwrap();
    assert!(json_output.contains("Vulnerability"));

    // Format as YAML
    let yaml_output = formatter.format(&ir, &Format::Yaml).unwrap();
    assert!(yaml_output.contains("fact_type:"));

    // Format as Visual
    let visual_output = formatter.format(&ir, &Format::Visual).unwrap();
    assert!(visual_output.contains("Finding #1"));
    assert!(visual_output.contains("Total findings: 2"));
}

#[tokio::test]
async fn test_ir_comparison_workflow() {
    let temp_dir = TempDir::new().unwrap();

    // Create two IR files
    let ir_v1 = temp_dir.path().join("ir_v1.json");
    let ir_v2 = temp_dir.path().join("ir_v2.json");

    let v1_content = r#"{
  "schema_version": "3.3.0",
  "metadata": {
    "name": "Test Project",
    "version": "1.0.0",
    "root_path": "/test"
  },
  "facts": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "fact_type": {
        "Vulnerability": {
          "cwe_id": null,
          "owasp_category": null,
          "severity": "Critical",
          "cvss_score": null,
          "description": "Test vulnerability",
          "confidence": 0.9
        }
      },
      "message": "Issue 1",
      "location": {
        "file": {
          "path": "/test/file1.js"
        },
        "start_line": 1,
        "start_column": 1,
        "end_line": 1,
        "end_column": 10
      },
      "provenance": {
        "extractor": "Custom",
        "version": "1.0.0",
        "confidence": 0.9,
        "extracted_at": "2025-01-15T10:30:00Z"
      }
    }
  ]
}"#;

    let v2_content = r#"{
  "schema_version": "3.3.0",
  "metadata": {
    "name": "Test Project",
    "version": "1.0.0",
    "root_path": "/test"
  },
  "facts": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "fact_type": {
        "Vulnerability": {
          "cwe_id": null,
          "owasp_category": null,
          "severity": "Critical",
          "cvss_score": null,
          "description": "Test vulnerability",
          "confidence": 0.9
        }
      },
      "message": "Issue 1",
      "location": {
        "file": {
          "path": "/test/file1.js"
        },
        "start_line": 1,
        "start_column": 1,
        "end_line": 1,
        "end_column": 10
      },
      "provenance": {
        "extractor": "Custom",
        "version": "1.0.0",
        "confidence": 0.9,
        "extracted_at": "2025-01-15T10:30:00Z"
      }
    },
    {
      "id": "550e8400-e29b-41d4-b716-446655440002",
      "fact_type": {
        "CodeSmell": {
          "smell_type": "test_smell",
          "severity": "Minor"
        }
      },
      "message": "Issue 2",
      "location": {
        "file": {
          "path": "/test/file2.js"
        },
        "start_line": 5,
        "start_column": 1,
        "end_line": 5,
        "end_column": 20
      },
      "provenance": {
        "extractor": "Custom",
        "version": "1.0.0",
        "confidence": 0.8,
        "extracted_at": "2025-01-15T10:31:00Z"
      }
    }
  ]
}"#;

    tokio::fs::write(&ir_v1, v1_content).await.unwrap();
    tokio::fs::write(&ir_v2, v2_content).await.unwrap();

    // Read both files
    let reader = IRReader::new();
    let ir1 = reader.read(&ir_v1).await.unwrap();
    let ir2 = reader.read(&ir_v2).await.unwrap();

    // Compare
    assert_eq!(ir1.len(), 1);
    assert_eq!(ir2.len(), 2);
    assert_ne!(ir1.len(), ir2.len());
}

#[tokio::test]
async fn test_ir_format_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let original_file = temp_dir.path().join("original.json");
    let formatted_file = temp_dir.path().join("formatted.json");

    // Read and format
    let reader = IRReader::new();
    let formatter = IRFormatter::new();

    tokio::fs::write(&original_file, SAMPLE_JSON).await.unwrap();

    let ir = reader.read(&original_file).await.unwrap();
    let formatted = formatter.format(&ir, &Format::Json).unwrap();

    // Write formatted output
    tokio::fs::write(&formatted_file, &formatted).await.unwrap();

    // Read formatted output
    let read_formatted = reader.read(&formatted_file).await.unwrap();

    // Should be the same
    assert_eq!(ir.len(), read_formatted.len());
    for (orig, read) in ir.iter().zip(read_formatted.iter()) {
        assert_eq!(orig.fact_type.discriminant(), read.fact_type.discriminant());
        assert_eq!(orig.message, read.message);
    }
}

#[tokio::test]
async fn test_multiple_ir_formats() {
    let temp_dir = TempDir::new().unwrap();
    let reader = IRReader::new();
    let formatter = IRFormatter::new();

    // Create IR in different formats
    let json_file = temp_dir.path().join("ir.json");
    let yaml_file = temp_dir.path().join("ir.yaml");

    // Write JSON
    tokio::fs::write(&json_file, SAMPLE_JSON).await.unwrap();

    // Read JSON and write YAML
    let ir = reader.read(&json_file).await.unwrap();
    let yaml_output = formatter.format(&ir, &Format::Yaml).unwrap();
    tokio::fs::write(&yaml_file, &yaml_output).await.unwrap();

    // Read YAML back
    let ir_from_yaml = reader.read(&yaml_file).await.unwrap();

    // Should be equivalent
    assert_eq!(ir.len(), ir_from_yaml.len());
    for (json_finding, yaml_finding) in ir.iter().zip(ir_from_yaml.iter()) {
        assert_eq!(json_finding.fact_type.discriminant(), yaml_finding.fact_type.discriminant());
        assert_eq!(json_finding.message, yaml_finding.message);
    }
}

#[tokio::test]
async fn test_ir_statistics_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let ir_file = temp_dir.path().join("stats.json");

    let stats_json = r#"{
  "schema_version": "3.3.0",
  "metadata": {
    "name": "Test Project",
    "version": "1.0.0",
    "root_path": "/test"
  },
  "facts": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "fact_type": {
        "Vulnerability": {
          "cwe_id": null,
          "owasp_category": null,
          "severity": "Critical",
          "cvss_score": null,
          "description": "SQL injection vulnerability",
          "confidence": 0.9
        }
      },
      "message": "SQL injection",
      "location": {
        "file": {
          "path": "/test/sql_file.js"
        },
        "start_line": 10,
        "start_column": 1,
        "end_line": 10,
        "end_column": 50
      },
      "provenance": {
        "extractor": "Custom",
        "version": "1.0.0",
        "confidence": 0.9,
        "extracted_at": "2025-01-15T10:30:00Z"
      }
    },
    {
      "id": "550e8400-e29b-41d4-b716-446655440002",
      "fact_type": {
        "Vulnerability": {
          "cwe_id": null,
          "owasp_category": null,
          "severity": "Major",
          "cvss_score": null,
          "description": "XSS vulnerability",
          "confidence": 0.8
        }
      },
      "message": "XSS",
      "location": {
        "file": {
          "path": "/test/xss_file.js"
        },
        "start_line": 20,
        "start_column": 1,
        "end_line": 20,
        "end_column": 30
      },
      "provenance": {
        "extractor": "Custom",
        "version": "1.0.0",
        "confidence": 0.8,
        "extracted_at": "2025-01-15T10:31:00Z"
      }
    },
    {
      "id": "550e8400-e29b-41d4-c716-446655440003",
      "fact_type": {
        "CodeSmell": {
          "smell_type": "unused_variable",
          "severity": "Minor"
        }
      },
      "message": "Unused variable",
      "location": {
        "file": {
          "path": "/test/smell_file.js"
        },
        "start_line": 5,
        "start_column": 1,
        "end_line": 5,
        "end_column": 15
      },
      "provenance": {
        "extractor": "Custom",
        "version": "1.0.0",
        "confidence": 0.7,
        "extracted_at": "2025-01-15T10:32:00Z"
      }
    },
    {
      "id": "550e8400-e29b-41d4-d716-446655440004",
      "fact_type": {
        "Vulnerability": {
          "cwe_id": null,
          "owasp_category": null,
          "severity": "Critical",
          "cvss_score": null,
          "description": "CSRF vulnerability",
          "confidence": 0.9
        }
      },
      "message": "CSRF",
      "location": {
        "file": {
          "path": "/test/csrf_file.js"
        },
        "start_line": 15,
        "start_column": 1,
        "end_line": 15,
        "end_column": 40
      },
      "provenance": {
        "extractor": "Custom",
        "version": "1.0.0",
        "confidence": 0.9,
        "extracted_at": "2025-01-15T10:33:00Z"
      }
    }
  ]
}"#;

    tokio::fs::write(&ir_file, stats_json).await.unwrap();

    let reader = IRReader::new();
    let ir = reader.read(&ir_file).await.unwrap();

    // Calculate statistics
    let mut by_type = std::collections::HashMap::new();
    let mut by_severity = std::collections::HashMap::new();

    for finding in &ir {
        *by_type.entry(format!("{:?}", finding.fact_type.discriminant())).or_insert(0) += 1;
        // Extract severity from fact_type if available
        let severity = match &finding.fact_type {
            hodei_ir::FactType::Vulnerability { severity, .. } => Some(format!("{:?}", severity)),
            hodei_ir::FactType::CodeSmell { severity, .. } => Some(format!("{:?}", severity)),
            _ => None,
        };
        if let Some(severity_str) = severity {
            *by_severity.entry(severity_str).or_insert(0) += 1;
        }
    }

    // Verify statistics - need to use the discriminant strings
    assert_eq!(by_type.get("Vulnerability"), Some(&3));
    assert_eq!(by_type.get("CodeSmell"), Some(&1));
    assert_eq!(by_severity.get("Critical"), Some(&2));
    assert_eq!(by_severity.get("Major"), Some(&1));
    assert_eq!(by_severity.get("Minor"), Some(&1));
}

#[tokio::test]
async fn test_ir_visualization_output() {
    let temp_dir = TempDir::new().unwrap();
    let ir_file = temp_dir.path().join("visual.json");

    let visual_json = r#"{
  "schema_version": "3.3.0",
  "metadata": {
    "name": "Test Project",
    "version": "1.0.0",
    "root_path": "/test"
  },
  "facts": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "fact_type": {
        "Vulnerability": {
          "cwe_id": null,
          "owasp_category": null,
          "severity": "Critical",
          "cvss_score": null,
          "description": "SQL injection detected",
          "confidence": 0.9
        }
      },
      "message": "SQL injection detected",
      "location": {
        "file": {
          "path": "src/auth/login.js"
        },
        "start_line": 42,
        "start_column": 1,
        "end_line": 42,
        "end_column": 50
      },
      "provenance": {
        "extractor": "Custom",
        "version": "1.0.0",
        "confidence": 0.9,
        "extracted_at": "2025-01-15T10:30:00Z"
      }
    },
    {
      "id": "550e8400-e29b-41d4-b716-446655440002",
      "fact_type": {
        "CodeSmell": {
          "smell_type": "unused_variable",
          "severity": "Minor"
        }
      },
      "message": "Unused variable 'x'",
      "location": {
        "file": {
          "path": "src/utils/helpers.js"
        },
        "start_line": 15,
        "start_column": 1,
        "end_line": 15,
        "end_column": 20
      },
      "provenance": {
        "extractor": "Custom",
        "version": "1.0.0",
        "confidence": 0.8,
        "extracted_at": "2025-01-15T10:31:00Z"
      }
    },
    {
      "id": "550e8400-e29b-41d4-c716-446655440003",
      "fact_type": {
        "Vulnerability": {
          "cwe_id": null,
          "owasp_category": null,
          "severity": "Major",
          "cvss_score": null,
          "description": "Missing authentication",
          "confidence": 0.8
        }
      },
      "message": "Missing authentication",
      "location": {
        "file": {
          "path": "src/api/users.js"
        },
        "start_line": 8,
        "start_column": 1,
        "end_line": 8,
        "end_column": 30
      },
      "provenance": {
        "extractor": "Custom",
        "version": "1.0.0",
        "confidence": 0.8,
        "extracted_at": "2025-01-15T10:32:00Z"
      }
    }
  ]
}"#;

    tokio::fs::write(&ir_file, visual_json).await.unwrap();

    let reader = IRReader::new();
    let formatter = IRFormatter::new();

    let ir = reader.read(&ir_file).await.unwrap();
    let visual = formatter.format(&ir, &Format::Visual).unwrap();

    // Verify visual format contains all expected elements
    assert!(visual.contains("IR Structure:"));
    assert!(visual.contains("Finding #1"));
    assert!(visual.contains("Finding #2"));
    assert!(visual.contains("Finding #3"));
    assert!(visual.contains("SQL injection detected"));
    assert!(visual.contains("Unused variable"));
    assert!(visual.contains("Missing authentication"));
    assert!(visual.contains("Total findings: 3"));

    // Should include locations
    assert!(visual.contains("src/auth/login.js:42"));
    assert!(visual.contains("src/utils/helpers.js:15"));
}

#[tokio::test]
async fn test_ir_round_trip_preservation() {
    let temp_dir = TempDir::new().unwrap();
    let original = temp_dir.path().join("original.json");
    let intermediate = temp_dir.path().join("intermediate.yaml");
    let final_output = temp_dir.path().join("final.json");

    let reader = IRReader::new();
    let formatter = IRFormatter::new();

    // Start with JSON
    tokio::fs::write(&original, SAMPLE_JSON).await.unwrap();

    // Read JSON -> Convert to YAML
    let ir1 = reader.read(&original).await.unwrap();
    let yaml = formatter.format(&ir1, &Format::Yaml).unwrap();
    tokio::fs::write(&intermediate, &yaml).await.unwrap();

    // Read YAML -> Convert back to JSON
    let ir2 = reader.read(&intermediate).await.unwrap();
    let json = formatter.format(&ir2, &Format::Json).unwrap();
    tokio::fs::write(&final_output, &json).await.unwrap();

    // Read final JSON
    let ir3 = reader.read(&final_output).await.unwrap();

    // All should be equivalent
    assert_eq!(ir1.len(), ir2.len());
    assert_eq!(ir2.len(), ir3.len());

    for (f1, f3) in ir1.iter().zip(ir3.iter()) {
        assert_eq!(f1.fact_type.discriminant(), f3.fact_type.discriminant());
        assert_eq!(f1.message, f3.message);
        assert_eq!(f1.location.file.as_str(), f3.location.file.as_str());
    }
}
