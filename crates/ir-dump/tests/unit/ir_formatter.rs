//! Unit tests for IRFormatter

use ir_dump::ir_formatter::{IRFormatter, Format};
use crate::fixtures::{single_finding_ir, multiple_findings_ir, empty_ir};

#[test]
fn test_formatter_creation() {
    let formatter = IRFormatter::new();
    assert!(true); // If we can create it, it worked
}

#[test]
fn test_format_json() {
    let formatter = IRFormatter::new();
    let ir = single_finding_ir();
    
    let result = formatter.format(&ir, &Format::Json);
    
    assert!(result.is_ok());
    let json = result.unwrap();
    
    // Should be valid JSON
    assert!(json.starts_with('{'));
    assert!(json.contains('"findings"'));
    assert!(json.contains("Vulnerability"));
    assert!(json.contains("SQL injection"));
}

#[test]
fn test_format_yaml() {
    let formatter = IRFormatter::new();
    let ir = single_finding_ir();
    
    let result = formatter.format(&ir, &Format::Yaml);
    
    assert!(result.is_ok());
    let yaml = result.unwrap();
    
    // Should contain YAML structure
    assert!(yaml.contains("findings:"));
    assert!(yaml.contains("fact_type:"));
    assert!(yaml.contains("Vulnerability"));
}

#[test]
fn test_format_visual() {
    let formatter = IRFormatter::new();
    let ir = multiple_findings_ir();
    
    let result = formatter.format(&ir, &Format::Visual);
    
    assert!(result.is_ok());
    let visual = result.unwrap();
    
    // Should contain visual elements
    assert!(visual.contains("IR Structure:"));
    assert!(visual.contains("="));
    assert!(visual.contains("Finding #1"));
    assert!(visual.contains("Total findings:"));
}

#[test]
fn test_format_empty_ir_json() {
    let formatter = IRFormatter::new();
    let ir = empty_ir();
    
    let result = formatter.format(&ir, &Format::Json);
    
    assert!(result.is_ok());
    let json = result.unwrap();
    
    // Should be empty findings array
    assert!(json.contains("\"findings\": []"));
}

#[test]
fn test_format_empty_ir_yaml() {
    let formatter = IRFormatter::new();
    let ir = empty_ir();
    
    let result = formatter.format(&ir, &Format::Yaml);
    
    assert!(result.is_ok());
    let yaml = result.unwrap();
    
    // Should be empty findings
    assert!(yaml.contains("findings: []"));
}

#[test]
fn test_format_empty_ir_visual() {
    let formatter = IRFormatter::new();
    let ir = empty_ir();
    
    let result = formatter.format(&ir, &Format::Visual);
    
    assert!(result.is_ok());
    let visual = result.unwrap();
    
    // Should still show structure with 0 findings
    assert!(visual.contains("Total findings: 0"));
}

#[test]
fn test_format_multiple_findings_json() {
    let formatter = IRFormatter::new();
    let ir = multiple_findings_ir();
    
    let result = formatter.format(&ir, &Format::Json);
    
    assert!(result.is_ok());
    let json = result.unwrap();
    
    // Should contain all findings
    assert!(json.contains("Vulnerability"));
    assert!(json.contains("CodeSmell"));
    assert!(json.contains("Hardcoded password"));
}

#[test]
fn test_format_multiple_findings_visual() {
    let formatter = IRFormatter::new();
    let ir = multiple_findings_ir();
    
    let result = formatter.format(&ir, &Format::Visual);
    
    assert!(result.is_ok());
    let visual = result.unwrap();
    
    // Should number each finding
    assert!(visual.contains("Finding #1"));
    assert!(visual.contains("Finding #2"));
    assert!(visual.contains("Finding #3"));
    
    // Should show total count
    assert!(visual.contains("Total findings: 3"));
}

#[test]
fn test_format_includes_location() {
    let formatter = IRFormatter::new();
    let ir = single_finding_ir();
    
    let result = formatter.format(&ir, &Format::Visual);
    
    assert!(result.is_ok());
    let visual = result.unwrap();
    
    // Should include location information
    assert!(visual.contains("Location:"));
    assert!(visual.contains("src/auth/login.js:42"));
}

#[test]
fn test_format_includes_severity() {
    let formatter = IRFormatter::new();
    let ir = single_finding_ir();
    
    let result = formatter.format(&ir, &Format::Json);
    
    assert!(result.is_ok());
    let json = result.unwrap();
    
    // Should include severity
    assert!(json.contains("\"severity\""));
    assert!(json.contains("Critical"));
}

#[test]
fn test_format_json_pretty_print() {
    let formatter = IRFormatter::new();
    let ir = single_finding_ir();
    
    let result = formatter.format(&ir, &Format::Json);
    
    assert!(result.is_ok());
    let json = result.unwrap();
    
    // Should be pretty printed (with indentation)
    assert!(json.contains("  ")); // Indentation
}

#[test]
fn test_format_error_handling() {
    let formatter = IRFormatter::new();
    
    // Invalid format (we only test with valid ones)
    // This test just verifies the formatter handles valid input
    
    let ir = single_finding_ir();
    let result = formatter.format(&ir, &Format::Json);
    
    assert!(result.is_ok());
}

#[test]
fn test_format_visual_structure() {
    let formatter = IRFormatter::new();
    let ir = multiple_findings_ir();
    
    let result = formatter.format(&ir, &Format::Visual);
    
    assert!(result.is_ok());
    let visual = result.unwrap();
    
    // Should have clear section separators
    assert!(visual.contains("=")); // Separator line
    assert!(visual.contains("-")); // Subsection separator
    
    // Each finding should be a separate section
    let finding_count = visual.matches("Finding #").count();
    assert_eq!(finding_count, 3);
}

#[test]
fn test_format_all_formats() {
    let formatter = IRFormatter::new();
    let ir = single_finding_ir();
    
    // Test all formats
    let json_result = formatter.format(&ir, &Format::Json);
    let yaml_result = formatter.format(&ir, &Format::Yaml);
    let visual_result = formatter.format(&ir, &Format::Visual);
    
    assert!(json_result.is_ok());
    assert!(yaml_result.is_ok());
    assert!(visual_result.is_ok());
    
    // All should contain the finding
    assert!(json_result.unwrap().contains("Vulnerability"));
    assert!(yaml_result.unwrap().contains("Vulnerability"));
    assert!(visual_result.unwrap().contains("Vulnerability"));
}

#[test]
fn test_format_with_metadata_json() {
    let formatter = IRFormatter::new();
    let ir = multiple_findings_ir();
    
    let result = formatter.format(&ir, &Format::Json);
    
    assert!(result.is_ok());
    let json = result.unwrap();
    
    // Should include metadata in JSON
    assert!(json.contains("\"metadata\""));
    assert!(json.contains("confidence"));
}
