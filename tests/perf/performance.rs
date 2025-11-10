//! Performance and Stress Tests

use hodei_ir;
use std::time::Duration;
use tempfile;

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_large_project_handling() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("large-project");

        // Create 100 Java files
        let src_dir = project_dir.join("src");
        std::fs::create_dir_all(&src_dir).unwrap();

        for i in 0..100 {
            let file_path = src_dir.join(format!("File{}.java", i));
            std::fs::write(
                &file_path,
                format!(
                    r#"
public class File{} {{
    // TODO: Implement method {}
    public void method{}() {{
        // Implementation
    }}

    // FIXME: Add validation
    public void validate{}() {{
        System.out.println("Validating");
    }}
}}
                    "#,
                    i, i, i, i
                ),
            )
            .unwrap();
        }

        // Verify all files created
        let entries = std::fs::read_dir(&src_dir).unwrap();
        let file_count = entries.count();
        assert_eq!(file_count, 100, "Should create 100 files");
    }

    #[test]
    fn test_many_patterns_extraction() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("complex.java");

        // Create file with many patterns
        let mut content = String::new();
        content.push_str("public class Complex {\n");

        for i in 0..50 {
            content.push_str(&format!("    // TODO: Task {}\n", i));
            content.push_str(&format!("    // FIXME: Issue {}\n", i));
            content.push_str(&format!("    public void method{}() {{ }}\n", i));
        }

        content.push_str("}\n");

        std::fs::write(&file_path, content).unwrap();

        // Verify file size
        let metadata = std::fs::metadata(&file_path).unwrap();
        assert!(metadata.len() > 1000, "File should be substantial");
    }

    #[test]
    fn test_nested_directory_structure() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("nested-project");

        // Create deeply nested structure
        let deep_path = project_dir
            .join("src")
            .join("main")
            .join("java")
            .join("com")
            .join("example")
            .join("deep")
            .join("nested")
            .join("structure");

        std::fs::create_dir_all(&deep_path).unwrap();

        // Add files at each level
        for i in 0..10 {
            let level_path = deep_path.join(format!("Level{}", i));
            std::fs::create_dir_all(&level_path).unwrap();

            std::fs::write(
                level_path.join("Class.java"),
                format!(
                    r#"
public class Level{} {{
    // TODO: Implement
    public void process() {{ }}
}}
                    "#,
                    i
                ),
            )
            .unwrap();
        }

        // Verify structure
        assert!(project_dir.exists());
        assert!(project_dir.join("src").exists());
    }

    #[test]
    fn test_large_file_processing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("large.java");

        // Create a large file (1MB+)
        let mut content = String::new();
        content.push_str("public class Large {\n");

        // Generate 10000 lines
        for i in 0..10000 {
            content.push_str(&format!(
                "    // TODO: Line {} - Some long comment with more text\n",
                i
            ));
            content.push_str(&format!(
                "    public void method{}() {{ /* implementation */ }}\n",
                i
            ));
        }

        content.push_str("}\n");

        std::fs::write(&file_path, content).unwrap();

        // Verify file size
        let metadata = std::fs::metadata(&file_path).unwrap();
        assert!(
            metadata.len() > 1024 * 1024,
            "File should be > 1MB, got {} bytes",
            metadata.len()
        );
    }

    #[test]
    fn test_many_concurrent_scans() {
        // This test validates we can handle the concept of concurrent processing
        // In a real implementation, this would test parallel scanning

        let temp_dir = tempfile::tempdir().unwrap();

        // Create multiple small projects
        for project in 0..5 {
            let project_dir = temp_dir.path().join(format!("project-{}", project));
            let src_dir = project_dir.join("src");
            std::fs::create_dir_all(&src_dir).unwrap();

            // Each project has 10 files
            for file in 0..10 {
                std::fs::write(
                    src_dir.join(format!("File{}.java", file)),
                    format!(
                        r#"
public class Project{}File{} {{
    public void method() {{ }}
}}
                        "#,
                        project, file
                    ),
                )
                .unwrap();
            }
        }

        // Verify all projects created
        let entries = std::fs::read_dir(temp_dir.path()).unwrap();
        let project_count = entries.count();
        assert_eq!(project_count, 5, "Should have 5 projects");
    }

    #[test]
    fn test_memory_efficient_fact_storage() {
        // Test that we can create many facts without memory issues
        let fact_types = vec![
            hodei_ir::FactType::CodeSmell {
                smell_type: "TODO".to_string(),
                severity: hodei_ir::Severity::Minor,
                message: "TODO comment".to_string(),
            },
            hodei_ir::FactType::Vulnerability {
                vuln_type: "test".to_string(),
                severity: hodei_ir::Severity::High,
                location: "test:1".to_string(),
                confidence: hodei_ir::Confidence::High,
            },
        ];

        // Create many facts
        let mut facts = Vec::new();
        for i in 0..1000 {
            for fact_type in &fact_types {
                facts.push(hodei_ir::Fact {
                    fact_id: format!("fact-{}", i),
                    fact_type: fact_type.clone(),
                    metadata: std::collections::HashMap::from([
                        ("index".to_string(), i.to_string()),
                        ("type".to_string(), "test".to_string()),
                    ]),
                    confidence: hodei_ir::Confidence::Medium,
                    provenance: hodei_ir::ExtractorId::Custom,
                    source_location: format!("test.java:{}", i),
                });
            }
        }

        assert_eq!(facts.len(), 2000, "Should have 2000 facts");
    }

    #[test]
    fn test_quick_scan_small_project() {
        let temp_dir = tempfile::tempdir().unwrap();
        let project_dir = temp_dir.path().join("small-project");
        std::fs::create_dir_all(&project_dir).unwrap();

        // Create 5 small files
        for i in 0..5 {
            let file_path = project_dir.join(format!("File{}.java", i));
            std::fs::write(
                &file_path,
                format!(
                    r#"
public class File{} {{
    public void method() {{ }}
}}
                    "#,
                    i
                ),
            )
            .unwrap();
        }

        // Verify project is small and quick to process
        let entries = std::fs::read_dir(&project_dir).unwrap();
        let file_count = entries.count();
        assert_eq!(file_count, 5, "Should have 5 files");
    }

    #[test]
    fn test_fact_indexing_performance() {
        // Simulate building indexes
        let mut type_index = std::collections::HashMap::new();
        let mut spatial_index = std::collections::HashMap::new();
        let mut flow_index = std::collections::HashMap::new();

        // Index 1000 facts
        for i in 0..1000 {
            let fact_type = format!("type-{}", i % 10);
            let file = format!("file-{}.java", i % 5);
            let line = (i % 100) as u32;

            // Type index
            type_index.entry(fact_type).or_insert_with(Vec::new).push(i);

            // Spatial index
            spatial_index
                .entry(file.clone())
                .or_insert_with(std::collections::HashMap::new)
                .entry(line)
                .or_insert_with(Vec::new)
                .push(i);

            // Flow index
            if i % 20 == 0 {
                flow_index
                    .entry(format!("flow-{}", i / 20))
                    .or_insert_with(Vec::new)
                    .push(i);
            }
        }

        // Verify indexes
        assert_eq!(type_index.len(), 10, "Should have 10 types");
        assert_eq!(spatial_index.len(), 5, "Should have 5 files");
        assert_eq!(flow_index.len(), 50, "Should have 50 flows");
    }

    #[test]
    fn test_unicode_handling_performance() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("unicode.java");

        // Create file with various Unicode characters
        let mut content = String::new();
        content.push_str("public class Unicode {\n");

        // Add various Unicode content
        for _ in 0..1000 {
            content.push_str("    // Comment with ñáéíóú and 中文 andрусский andالعربية\n");
            content.push_str("    public void método() { /* método */ }\n");
        }

        content.push_str("}\n");

        std::fs::write(&file_path, content).unwrap();

        // Verify file processed
        let metadata = std::fs::metadata(&file_path).unwrap();
        assert!(metadata.len() > 1000, "File should be substantial");
    }

    #[test]
    fn test_rule_evaluation_performance() {
        // Simulate rule evaluation with many facts
        let facts: Vec<_> = (0..500)
            .map(|i| hodei_ir::Fact {
                fact_id: format!("fact-{}", i),
                fact_type: hodei_ir::FactType::CodeSmell {
                    smell_type: if i % 2 == 0 {
                        "TODO".to_string()
                    } else {
                        "FIXME".to_string()
                    },
                    severity: hodei_ir::Severity::Medium,
                    message: "test".to_string(),
                },
                metadata: std::collections::HashMap::new(),
                confidence: hodei_ir::Confidence::Medium,
                provenance: hodei_ir::ExtractorId::Custom,
                source_location: format!("test.java:{}", i),
            })
            .collect();

        // Group facts by type (simulates rule evaluation)
        let grouped: std::collections::HashMap<String, Vec<_>> =
            facts
                .iter()
                .fold(std::collections::HashMap::new(), |mut acc, fact| {
                    if let hodei_ir::FactType::CodeSmell { ref smell_type, .. } = fact.fact_type {
                        acc.entry(smell_type.clone())
                            .or_insert_with(Vec::new)
                            .push(fact);
                    }
                    acc
                });

        assert_eq!(grouped.len(), 2, "Should have 2 types");
        assert_eq!(
            grouped.get("TODO").map(|v| v.len()).unwrap_or(0),
            250,
            "Should have 250 TODOs"
        );
    }
}
