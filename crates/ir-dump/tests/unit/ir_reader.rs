//! Unit tests for IRReader

use ir_dump::ir_reader::IRReader;
use tempfile::TempDir;
use std::path::Path;
use crate::fixtures::SAMPLE_JSON, SAMPLE_YAML;

#[tokio::test]
async fn test_reader_creation() {
    let reader = IRReader::new();
    assert!(true); // If we can create it, it worked
}

#[tokio::test]
async fn test_read_json_file() {
    let reader = IRReader::new();
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("test.json");
    
    tokio::fs::write(&json_file, SAMPLE_JSON).await.unwrap();
    
    let result = reader.read(&json_file).await;
    
    assert!(result.is_ok());
    let ir = result.unwrap();
    
    assert_eq!(ir.findings.len(), 2);
    assert_eq!(ir.findings[0].fact_type, "Vulnerability");
    assert_eq!(ir.findings[1].fact_type, "CodeSmell");
}

#[tokio::test]
async fn test_read_yaml_file() {
    let reader = IRReader::new();
    let temp_dir = TempDir::new().unwrap();
    let yaml_file = temp_dir.path().join("test.yaml");
    
    tokio::fs::write(&yaml_file, SAMPLE_YAML).await.unwrap();
    
    let result = reader.read(&yaml_file).await;
    
    assert!(result.is_ok());
    let ir = result.unwrap();
    
    assert_eq!(ir.findings.len(), 2);
    assert_eq!(ir.findings[0].fact_type, "Vulnerability");
    assert_eq!(ir.findings[1].fact_type, "CodeSmell");
}

#[tokio::test]
async fn test_read_json_with_extension() {
    let reader = IRReader::new();
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("data.json");
    
    tokio::fs::write(&json_file, SAMPLE_JSON).await.unwrap();
    
    let result = reader.read(&json_file).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap().findings.len(), 2);
}

#[tokio::test]
async fn test_read_yaml_with_yml_extension() {
    let reader = IRReader::new();
    let temp_dir = TempDir::new().unwrap();
    let yaml_file = temp_dir.path().join("data.yml");
    
    tokio::fs::write(&yaml_file, SAMPLE_YAML).await.unwrap();
    
    let result = reader.read(&yaml_file).await;
    
    assert!(result.is_ok());
    assert_eq!(result.unwrap().findings.len(), 2);
}

#[tokio::test]
async fn test_read_unsupported_format() {
    let reader = IRReader::new();
    let temp_dir = TempDir::new().unwrap();
    let txt_file = temp_dir.path().join("test.txt");
    
    tokio::fs::write(&txt_file, "unsupported format").await.unwrap();
    
    let result = reader.read(&txt_file).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_read_nonexistent_file() {
    let reader = IRReader::new();
    let nonexistent = Path::new("/nonexistent/file.json");
    
    let result = reader.read(nonexistent).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_read_capnp_not_implemented() {
    let reader = IRReader::new();
    let temp_dir = TempDir::new().unwrap();
    let capnp_file = temp_dir.path().join("data.capnp");
    
    tokio::fs::write(&capnp_file, "placeholder").await.unwrap();
    
    let result = reader.read(&capnp_file).await;
    
    assert!(result.is_err());
    let error = result.unwrap_err().to_string();
    assert!(error.contains("Cap'n Proto"));
    assert!(error.contains("not yet implemented"));
}

#[tokio::test]
async fn test_read_invalid_json() {
    let reader = IRReader::new();
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("invalid.json");
    
    let invalid_json = "{ invalid json syntax ";
    tokio::fs::write(&json_file, invalid_json).await.unwrap();
    
    let result = reader.read(&json_file).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_read_invalid_yaml() {
    let reader = IRReader::new();
    let temp_dir = TempDir::new().unwrap();
    let yaml_file = temp_dir.path().join("invalid.yaml");
    
    let invalid_yaml = "invalid: yaml: syntax: [ unclosed";
    tokio::fs::write(&yaml_file, invalid_yaml).await.unwrap();
    
    let result = reader.read(&yaml_file).await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_read_empty_json() {
    let reader = IRReader::new();
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("empty.json");
    
    let empty_json = r#"{"findings": []}"#;
    tokio::fs::write(&json_file, empty_json).await.unwrap();
    
    let result = reader.read(&json_file).await;
    
    assert!(result.is_ok());
    let ir = result.unwrap();
    assert!(ir.findings.is_empty());
}

#[tokio::test]
async fn test_read_empty_yaml() {
    let reader = IRReader::new();
    let temp_dir = TempDir::new().unwrap();
    let yaml_file = temp_dir.path().join("empty.yaml");
    
    let empty_yaml = "findings: []";
    tokio::fs::write(&yaml_file, empty_yaml).await.unwrap();
    
    let result = reader.read(&yaml_file).await;
    
    assert!(result.is_ok());
    let ir = result.unwrap();
    assert!(ir.findings.is_empty());
}

#[tokio::test]
async fn test_read_finds_all_fields() {
    let reader = IRReader::new();
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("complete.json");
    
    let complete_json = r#"{
  "findings": [
    {
      "fact_type": "Vulnerability",
      "message": "Test message",
      "location": "file.js:10",
      "severity": "Critical",
      "metadata": {
        "key": "value"
      }
    }
  ]
}"#;
    
    tokio::fs::write(&json_file, complete_json).await.unwrap();
    
    let result = reader.read(&json_file).await;
    
    assert!(result.is_ok());
    let ir = result.unwrap();
    
    assert_eq!(ir.findings.len(), 1);
    let finding = &ir.findings[0];
    
    assert_eq!(finding.fact_type, "Vulnerability");
    assert_eq!(finding.message, "Test message");
    assert_eq!(finding.location, Some("file.js:10".to_string()));
    assert_eq!(finding.severity, Some("Critical".to_string()));
    assert_eq!(finding.metadata.get("key"), Some(&"value".to_string()));
}

#[tokio::test]
async fn test_read_with_missing_optional_fields() {
    let reader = IRReader::new();
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("minimal.json");
    
    let minimal_json = r#"{
  "findings": [
    {
      "fact_type": "Vulnerability",
      "message": "Minimal finding"
    }
  ]
}"#;
    
    tokio::fs::write(&json_file, minimal_json).await.unwrap();
    
    let result = reader.read(&json_file).await;
    
    assert!(result.is_ok());
    let ir = result.unwrap();
    
    assert_eq!(ir.findings.len(), 1);
    let finding = &ir.findings[0];
    
    assert_eq!(finding.fact_type, "Vulnerability");
    assert_eq!(finding.message, "Minimal finding");
    assert_eq!(finding.location, None);
    assert_eq!(finding.severity, None);
    assert!(finding.metadata.is_empty());
}

#[tokio::test]
async fn test_read_preserves_json_structure() {
    let reader = IRReader::new();
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("structured.json");
    
    tokio::fs::write(&json_file, SAMPLE_JSON).await.unwrap();
    
    let result = reader.read(&json_file).await;
    
    assert!(result.is_ok());
    let ir = result.unwrap();
    
    // Verify structure is preserved
    assert!(!ir.findings.is_empty());
    for finding in &ir.findings {
        assert!(!finding.fact_type.is_empty());
        assert!(!finding.message.is_empty());
    }
}

#[tokio::test]
async fn test_read_unicode_content() {
    let reader = IRReader::new();
    let temp_dir = TempDir::new().unwrap();
    let json_file = temp_dir.path().join("unicode.json");
    
    let unicode_json = r#"{
  "findings": [
    {
      "fact_type": "Vulnerability",
      "message": "安全漏洞检测到",
      "location": "文件.js:42"
    }
  ]
}"#;
    
    tokio::fs::write(&json_file, unicode_json).await.unwrap();
    
    let result = reader.read(&json_file).await;
    
    assert!(result.is_ok());
    let ir = result.unwrap();
    
    assert_eq!(ir.findings.len(), 1);
    assert!(ir.findings[0].message.contains("安全漏洞"));
}
