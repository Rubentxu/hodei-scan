//! End-to-End tests for IR dump workflow

use crate::fixtures::SAMPLE_JSON;
use ir_dump::{Format, IRFormatter, IRReader};
use tempfile::TempDir;

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

    assert_eq!(ir.findings.len(), 2);
    assert_eq!(ir.findings[0].fact_type, "Vulnerability");
    assert_eq!(ir.findings[1].fact_type, "CodeSmell");

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
  "findings": [
    {
      "fact_type": "Vulnerability",
      "message": "Issue 1",
      "location": "file1.js:1"
    }
  ]
}"#;

    let v2_content = r#"{
  "findings": [
    {
      "fact_type": "Vulnerability",
      "message": "Issue 1",
      "location": "file1.js:1"
    },
    {
      "fact_type": "CodeSmell",
      "message": "Issue 2",
      "location": "file2.js:5"
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
    assert_eq!(ir1.findings.len(), 1);
    assert_eq!(ir2.findings.len(), 2);
    assert_ne!(ir1.findings.len(), ir2.findings.len());
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
    assert_eq!(ir.findings.len(), read_formatted.findings.len());
    for (orig, read) in ir.findings.iter().zip(read_formatted.findings.iter()) {
        assert_eq!(orig.fact_type, read.fact_type);
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
    assert_eq!(ir.findings.len(), ir_from_yaml.findings.len());
    for (json_finding, yaml_finding) in ir.findings.iter().zip(ir_from_yaml.findings.iter()) {
        assert_eq!(json_finding.fact_type, yaml_finding.fact_type);
        assert_eq!(json_finding.message, yaml_finding.message);
    }
}

#[tokio::test]
async fn test_ir_statistics_workflow() {
    let temp_dir = TempDir::new().unwrap();
    let ir_file = temp_dir.path().join("stats.json");

    let stats_json = r#"{
  "findings": [
    {
      "fact_type": "Vulnerability",
      "message": "SQL injection",
      "severity": "Critical"
    },
    {
      "fact_type": "Vulnerability",
      "message": "XSS",
      "severity": "Major"
    },
    {
      "fact_type": "CodeSmell",
      "message": "Unused variable",
      "severity": "Minor"
    },
    {
      "fact_type": "Vulnerability",
      "message": "CSRF",
      "severity": "Critical"
    }
  ]
}"#;

    tokio::fs::write(&ir_file, stats_json).await.unwrap();

    let reader = IRReader::new();
    let ir = reader.read(&ir_file).await.unwrap();

    // Calculate statistics
    let mut by_type = std::collections::HashMap::new();
    let mut by_severity = std::collections::HashMap::new();

    for finding in &ir.findings {
        *by_type.entry(&finding.fact_type).or_insert(0) += 1;
        if let Some(ref severity) = finding.severity {
            *by_severity.entry(severity).or_insert(0) += 1;
        }
    }

    // Verify statistics
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
  "findings": [
    {
      "fact_type": "Vulnerability",
      "message": "SQL injection detected",
      "location": "src/auth/login.js:42",
      "severity": "Critical"
    },
    {
      "fact_type": "CodeSmell",
      "message": "Unused variable 'x'",
      "location": "src/utils/helpers.js:15",
      "severity": "Minor"
    },
    {
      "fact_type": "SecurityIssue",
      "message": "Missing authentication",
      "location": "src/api/users.js:8",
      "severity": "Major"
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
    assert_eq!(ir1.findings.len(), ir2.findings.len());
    assert_eq!(ir2.findings.len(), ir3.findings.len());

    for (f1, f3) in ir1.findings.iter().zip(ir3.findings.iter()) {
        assert_eq!(f1.fact_type, f3.fact_type);
        assert_eq!(f1.message, f3.message);
        assert_eq!(f1.location, f3.location);
        assert_eq!(f1.severity, f3.severity);
    }
}
