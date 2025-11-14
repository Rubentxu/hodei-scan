#[cfg(test)]
mod integration_tests {
    use hodei_ir::ProjectPath;
    use hodei_java_extractor::{ExtractionLevel, JacocoAdapter, JavaSourceId};
    use std::path::PathBuf;

    #[tokio::test]
    #[ignore]
    async fn test_jacoco_adapter_integration() {
        // Arrange
        let mut adapter = JacocoAdapter::new(PathBuf::from(
            "/home/rubentxu/Proyectos/rust/hodei-scan/test-java-project/jacoco.xml",
        ));

        // Act
        let result = adapter.load_coverage_data();

        // Assert
        assert!(
            result.is_ok(),
            "Should successfully parse JaCoCo XML report"
        );

        let coverage_data = result.unwrap();
        assert!(!coverage_data.is_empty(), "Should have coverage data");

        // Verify structure
        assert!(
            coverage_data.len() >= 1,
            "Should have at least one coverage entry"
        );

        // Verify UserController exists
        let has_user_controller = coverage_data
            .iter()
            .any(|data| data.source_id.class_name == "UserController");

        assert!(
            has_user_controller,
            "Should find UserController coverage data"
        );
    }

    #[tokio::test]
    async fn test_java_source_id_creation() {
        // Test JavaSourceId creation and usage
        let source_id = JavaSourceId {
            package: "com.example".to_string(),
            class_name: "UserController".to_string(),
            file_path: ProjectPath::new(PathBuf::from(
                "src/main/java/com/example/UserController.java",
            )),
        };

        assert_eq!(source_id.package, "com.example");
        assert_eq!(source_id.class_name, "UserController");
        assert!(
            source_id
                .file_path
                .to_string()
                .contains("UserController.java")
        );
    }

    #[tokio::test]
    async fn test_extraction_level_enum() {
        // Test ExtractionLevel enum functionality
        assert_eq!(ExtractionLevel::Level1.as_str(), "level1");
        assert_eq!(ExtractionLevel::Level2.as_str(), "level2");
        assert_eq!(ExtractionLevel::Level3.as_str(), "level3");

        assert_eq!(
            ExtractionLevel::from_str("level1"),
            Some(ExtractionLevel::Level1)
        );
        assert_eq!(
            ExtractionLevel::from_str("level2"),
            Some(ExtractionLevel::Level2)
        );
        assert_eq!(
            ExtractionLevel::from_str("level3"),
            Some(ExtractionLevel::Level3)
        );
        assert_eq!(ExtractionLevel::from_str("invalid"), None);
    }

    #[tokio::test]
    async fn test_error_handling_invalid_path() {
        // Test error handling for non-existent JaCoCo report
        let mut adapter = JacocoAdapter::new(PathBuf::from("/non/existent/path/jacoco.xml"));

        let result = adapter.load_coverage_data();

        assert!(result.is_err(), "Should return error for non-existent file");
    }
}
