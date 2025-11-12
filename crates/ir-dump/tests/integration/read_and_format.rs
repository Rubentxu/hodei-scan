//! Integration tests for read and format workflow

use ir_dump::{IRReader, IRFormatter, Format};
use tempfile::TempDir;
use crate::fixtures::SAMPLE_JSON, SAMPLE_YAML, multiple_findings_ir;

#[tokio::test]
async fn test_read_json_format_json() {
    let reader = IRReader::new();
    let formatter = IRFormatter::new();
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    
    tokio::fs::write(&json_file, SAMPLE_JSON).await.unwrap();
    
    // Read and format
    let ir = reader.read(&json_file).await.unwrap();
    let output = formatter.format(&ir, &Format::Json).unwrap();
    
    // Should be valid JSON with findings
    assert!(output.starts_with('{'));
    assert!(output.contains("Vulnerability"));
    assert!(output.contains("CodeSmell"));
}

#[tokio::test]
async fn test_read_json_format_yaml() {
    let reader = IRReader::new();
    let formatter = IRFormatter::new();
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    
    tokio::fs::write(&json_file, SAMPLE_JSON).await.unwrap();
    
    // Read from JSON, output as YAML
    let ir = reader.read(&json_file).await.unwrap();
    let output = formatter.format(&ir, &Format::Yaml).unwrap();
    
    // Should be YAML format
    assert!(output.contains("findings:"));
    assert!(output.contains("fact_type:"));
    assert!(output.contains("Vulnerability"));
}

#[tokio::test]
async fn test_read_yaml_format_visual() {
    let reader = IRReader::new();
    let formatter = IRFormatter::new();
    let temp_dir = TempDir::new().unwrap();
    let yaml_file = temp_dir.path().join("test.yaml");
    
    tokio::fs::write(&yaml_file, SAMPLE_YAML).await.unwrap();
    
    // Read from YAML, output as visual
    let ir = reader.read(&yaml_file).await.unwrap();
    let output = formatter.format(&ir, &Format::Visual).unwrap();
    
    // Should be visual format
    assert!(output.contains("IR Structure:"));
    assert!(output.contains("Finding #1"));
    assert!(output.contains("Vulnerability"));
}

#[tokio::test]
async fn test_round_trip_json() {
    let reader = IRReader::new();
    let formatter = IRFormatter::new();
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    
    // Write original IR
    let original_ir = multiple_findings_ir();
    let json = formatter.format(&original_ir, &Format::Json).unwrap();
    tokio::fs::write(&json_file, json).await.unwrap();
    
    // Read back
    let read_ir = reader.read(&json_file).await.unwrap();
    
    // Should match original
    assert_eq!(read_ir.findings.len(), original_ir.findings.len());
    for (original, read) in original_ir.findings.iter().zip(read_ir.findings.iter()) {
        assert_eq!(original.fact_type, read.fact_type);
        assert_eq!(original.message, read.message);
        assert_eq!(original.location, read.location);
        assert_eq!(original.severity, read.severity);
    }
}

#[tokio::test]
async fn test_all_format_combinations() {
    let reader = IRReader::new();
    let formatter = IRFormatter::new();
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    
    tokio::fs::write(&json_file, SAMPLE_JSON).await.unwrap();
    
    // Read once
    let ir = reader.read(&json_file).await.unwrap();
    
    // Test all format combinations
    let json_output = formatter.format(&ir, &Format::Json).unwrap();
    let yaml_output = formatter.format(&ir, &Format::Yaml).unwrap();
    let visual_output = formatter.format(&ir, &Format::Visual).unwrap();
    
    // All should succeed and contain the same data
    assert!(json_output.contains("Vulnerability"));
    assert!(yaml_output.contains("Vulnerability"));
    assert!(visual_output.contains("Vulnerability"));
    
    // All should be different formats
    assert!(json_output.starts_with('{'));
    assert!(yaml_output.starts_with("findings:"));
    assert!(visual_output.contains("="));
}

#[tokio::test]
async fn test_empty_ir_all_formats() {
    let formatter = IRFormatter::new();
    
    let empty_ir = ir_dump::tests::fixtures::empty_ir();
    
    // Test all formats with empty IR
    let json = formatter.format(&empty_ir, &Format::Json).unwrap();
    let yaml = formatter.format(&empty_ir, &Format::Yaml).unwrap();
    let visual = formatter.format(&empty_ir, &Format::Visual).unwrap();
    
    // All should handle empty case
    assert!(json.contains("\"findings\": []"));
    assert!(yaml.contains("findings: []"));
    assert!(visual.contains("Total findings: 0"));
}

#[tokio::test]
async fn test_large_ir_formats() {
    let formatter = IRFormatter::new();
    
    // Create IR with 1000 findings
    let mut findings = Vec::new();
    for i in 0..1000 {
        findings.push(hodei_ir::Finding {
            fact_type: format!("Type{}", i % 3),
            message: format!("Finding {}", i),
            location: Some(format!("file{}.js:{}", i, i)),
            severity: Some("Major".to_string()),
            metadata: std::collections::HashMap::new(),
        });
    }
    
    let large_ir = hodei_ir::FindingSet { findings };
    
    // All formats should handle large IR
    let json = formatter.format(&large_ir, &Format::Json).unwrap();
    let yaml = formatter.format(&large_ir, &Format::Yaml).unwrap();
    let visual = formatter.format(&large_ir, &Format::Visual).unwrap();
    
    assert!(json.contains("\"findings\": ["));
    assert!(yaml.contains("findings:"));
    assert!(visual.contains("Total findings: 1000"));
}

#[tokio::test]
async fn test_preserve_metadata() {
    let reader = IRReader::new();
    let formatter = IRFormatter::new();
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("metadata.json");
    
    let json_with_metadata = r#"{
  "findings": [
    {
      "fact_type": "Vulnerability",
      "message": "Test",
      "metadata": {
        "confidence": "0.95",
        "cwe": "CWE-79",
        "severity_score": "9.8"
      }
    }
  ]
}"#;
    
    tokio::fs::write(&json_file, json_with_metadata).await.unwrap();
    
    let ir = reader.read(&json_file).await.unwrap();
    let output = formatter.format(&ir, &Format::Json).unwrap();
    
    // Should preserve metadata
    assert!(output.contains("\"metadata\""));
    assert!(output.contains("confidence"));
    assert!(output.contains("0.95"));
    assert!(output.contains("cwe"));
    assert!(output.contains("CWE-79"));
}

#[tokio::test]
async fn test_multiple_reads() {
    let reader = IRReader::new();
    let formatter = IRFormatter::new();
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    
    tokio::fs::write(&json_file, SAMPLE_JSON).await.unwrap();
    
    // Read and format multiple times
    for _ in 0..5 {
        let ir = reader.read(&json_file).await.unwrap();
        let output = formatter.format(&ir, &Format::Json).unwrap();
        
        assert!(output.contains("Vulnerability"));
    }
}

#[tokio::test]
async fn test_concurrent_reads() {
    let reader = IRReader::new();
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("concurrent.json");
    
    tokio::fs::write(&json_file, SAMPLE_JSON).await.unwrap();
    
    // Spawn multiple concurrent reads
    let mut handles = Vec::new();
    for _ in 0..10 {
        let reader_clone = &reader;
        let file_path = json_file.clone();
        let handle = tokio::spawn(async move {
            reader_clone.read(&file_path).await
        });
        handles.push(handle);
    }
    
    // All should succeed
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
        let ir = result.unwrap();
        assert_eq!(ir.findings.len(), 2);
    }
}
