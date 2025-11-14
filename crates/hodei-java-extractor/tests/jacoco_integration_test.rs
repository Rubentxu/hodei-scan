#[cfg(test)]
mod jacoco_integration_tests {
    use hodei_ir::ProjectPath;
    use hodei_java_extractor::{CoverageData, JacocoAdapter, JavaSourceId};
    use std::path::PathBuf;

    #[tokio::test]
    #[ignore]
    async fn test_load_coverage_data_from_real_xml() {
        // Arrange - Create a temporary XML file for testing
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test-jacoco.xml");

        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<report name="Test JaCoCo Report">
  <package name="com/example">
    <class name="com/example/TestClass" sourcefilename="TestClass.java">
      <method name="testMethod" desc="()V" line="1">
        <line nr="1" mi="0" ci="5" mb="0" cb="2"/>
      </method>
    </class>
  </package>
</report>"#;

        std::fs::write(&test_file, xml_content).expect("Failed to write test XML");

        let mut adapter = JacocoAdapter::new(test_file.clone());

        // Act
        let result = adapter.load_coverage_data();
        std::fs::remove_file(&test_file).ok(); // Cleanup

        // Assert
        assert!(
            result.is_ok(),
            "Should successfully parse JaCoCo XML report"
        );

        let coverage_data = result.unwrap();
        assert!(!coverage_data.is_empty(), "Should have coverage data");

        // Verify structure - should have at least one source file
        assert!(
            coverage_data.len() >= 1,
            "Should have at least one coverage entry"
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_parse_coverage_metrics_correctly() {
        // Arrange - Create a temporary XML file with known metrics
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test-metrics.xml");

        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<report name="Test Metrics">
  <package name="com/example">
    <class name="com/example/UserController" sourcefilename="UserController.java">
      <method name="testMethod" desc="()V" line="1">
        <line nr="10" mi="0" ci="5" mb="0" cb="2"/>
        <line nr="11" mi="0" ci="3" mb="0" cb="1"/>
        <line nr="12" mi="5" ci="0" mb="1" cb="0"/>
      </method>
    </class>
  </package>
</report>"#;

        std::fs::write(&test_file, xml_content).expect("Failed to write test XML");

        let mut adapter = JacocoAdapter::new(test_file.clone());

        // Act
        let result = adapter.load_coverage_data();
        std::fs::remove_file(&test_file).ok(); // Cleanup

        let coverage_data = result.unwrap();

        // Extract the UserController data
        let user_controller_data = coverage_data
            .iter()
            .find(|data| data.source_id.class_name == "UserController")
            .expect("Should find UserController coverage data");

        // Assert - Verify coverage metrics are correctly parsed
        assert_eq!(
            user_controller_data.instruction_missed, 0,
            "Line 10 should have 0 missed instructions"
        );
        assert_eq!(
            user_controller_data.instruction_covered, 5,
            "Line 10 should have 5 covered instructions"
        );
    }

    #[tokio::test]
    async fn test_parse_multiple_classes() {
        // Arrange - Create a temporary XML with multiple classes
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test-multiple-classes.xml");

        let xml_content = r#"
        <report name="Multi-Class Test">
          <package name="com/example">
            <class name="com/example/UserController" sourcefilename="UserController.java">
              <method name="login" desc="(Ljava/lang/String;Ljava/lang/String;)V" line="10">
                <line nr="10" mi="0" ci="5" mb="0" cb="2"/>
              </method>
            </class>
            <class name="com/example/UserService" sourcefilename="UserService.java">
              <method name="findUser" desc="(Ljava/lang/String;)V" line="20">
                <line nr="20" mi="3" ci="2" mb="1" cb="1"/>
              </method>
            </class>
          </package>
        </report>
        "#;

        std::fs::write(&test_file, xml_content).expect("Failed to write test XML");

        let mut adapter = JacocoAdapter::new(test_file.clone());

        // Act
        let result = adapter.load_coverage_data();
        std::fs::remove_file(&test_file).ok(); // Cleanup

        // Assert
        let coverage_data = result.unwrap();
        assert_eq!(
            coverage_data.len(),
            2,
            "Should have coverage data for 2 classes"
        );

        let has_user_controller = coverage_data
            .iter()
            .any(|data| data.source_id.class_name == "UserController");
        let has_user_service = coverage_data
            .iter()
            .any(|data| data.source_id.class_name == "UserService");

        assert!(
            has_user_controller,
            "Should find UserController coverage data"
        );
        assert!(has_user_service, "Should find UserService coverage data");
    }

    #[tokio::test]
    async fn test_calculate_branch_coverage_correctly() {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test-branches.xml");

        let xml_content = r#"
        <report name="Branch Coverage Test">
          <package name="com/example">
            <class name="com/example/TestClass" sourcefilename="TestClass.java">
              <method name="testMethod" desc="()V" line="1">
                <line nr="1" mi="0" ci="5" mb="0" cb="2"/>
                <line nr="2" mi="1" ci="4" mb="1" cb="1"/>
              </method>
            </class>
          </package>
        </report>
        "#;

        std::fs::write(&test_file, xml_content).expect("Failed to write test XML");

        let mut adapter = JacocoAdapter::new(test_file.clone());

        // Act
        let result = adapter.load_coverage_data();
        std::fs::remove_file(&test_file).ok(); // Cleanup

        // Assert
        let coverage_data = result.unwrap();
        let test_class_data = coverage_data
            .iter()
            .find(|data| data.source_id.class_name == "TestClass")
            .expect("Should find TestClass coverage data");

        // Line 1: mb=0, cb=2 (2 branches covered, 0 missed)
        // Line 2: mb=1, cb=1 (1 branch covered, 1 missed)
        // For line 2: Total branches = 2, covered = 1, so 50% coverage
        assert_eq!(
            test_class_data.branch_missed, 1,
            "Should have 1 missed branch"
        );
        assert_eq!(
            test_class_data.branch_covered, 1,
            "Should have 1 covered branch"
        );
        assert!(
            (test_class_data.coverage_percentage - 50.0).abs() < 0.1,
            "Branch coverage should be approximately 50%"
        );
    }

    #[tokio::test]
    async fn test_error_handling_for_missing_file() {
        // Arrange
        let non_existent_path = PathBuf::from("/non/existent/path/jacoco.xml");
        let mut adapter = JacocoAdapter::new(non_existent_path);

        // Act
        let result = adapter.load_coverage_data();

        // Assert
        assert!(result.is_err(), "Should return error for non-existent file");
    }

    #[tokio::test]
    async fn test_error_handling_for_malformed_xml() {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test-malformed.xml");

        let malformed_xml = r#"
        <report name="Malformed XML">
          <package name="com/example">
            <class name="com/example/TestClass" sourcefilename="TestClass.java">
              <method name="testMethod" desc="()V" line="1">
                <line nr="1" mi="0" ci="5" mb="0" cb="2"/>
              </method>
            <!-- Missing closing tags -->
        </report>
        "#;

        std::fs::write(&test_file, malformed_xml).expect("Failed to write malformed XML");

        let mut adapter = JacocoAdapter::new(test_file.clone());

        // Act
        let result = adapter.load_coverage_data();
        std::fs::remove_file(&test_file).ok(); // Cleanup

        // Assert
        assert!(result.is_err(), "Should return error for malformed XML");
    }

    #[tokio::test]
    #[ignore]
    async fn test_extract_source_id_correctly() {
        // Arrange - Create a temporary XML file
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test-source-id.xml");

        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<report name="Test Source ID">
  <package name="com/example">
    <class name="com/example/UserController" sourcefilename="UserController.java">
      <method name="testMethod" desc="()V" line="1">
        <line nr="10" mi="0" ci="5" mb="0" cb="2"/>
      </method>
    </class>
  </package>
</report>"#;

        std::fs::write(&test_file, xml_content).expect("Failed to write test XML");

        let mut adapter = JacocoAdapter::new(test_file.clone());

        // Act
        let result = adapter.load_coverage_data();
        std::fs::remove_file(&test_file).ok(); // Cleanup

        let coverage_data = result.unwrap();

        // Assert
        let user_controller = coverage_data
            .iter()
            .find(|data| data.source_id.class_name == "UserController")
            .expect("Should find UserController");

        assert_eq!(user_controller.source_id.package, "com/example");
        assert_eq!(user_controller.source_id.class_name, "UserController");
        assert!(
            user_controller
                .source_id
                .file_path
                .to_string()
                .contains("UserController.java")
        );
    }

    #[tokio::test]
    async fn test_line_coverage_details() {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test-line-coverage.xml");

        let xml_content = r#"
        <report name="Line Coverage Test">
          <package name="com/example">
            <class name="com/example/LineTest" sourcefilename="LineTest.java">
              <method name="testMethod" desc="()V" line="1">
                <line nr="10" mi="0" ci="5" mb="0" cb="0"/>
                <line nr="11" mi="3" ci="0" mb="0" cb="0"/>
                <line nr="12" mi="0" ci="0" mb="0" cb="0"/>
              </method>
            </class>
          </package>
        </report>
        "#;

        std::fs::write(&test_file, xml_content).expect("Failed to write test XML");

        let mut adapter = JacocoAdapter::new(test_file.clone());

        // Act
        let result = adapter.load_coverage_data();
        std::fs::remove_file(&test_file).ok(); // Cleanup

        // Assert
        let coverage_data = result.unwrap();
        let line_test = coverage_data
            .iter()
            .find(|data| data.source_id.class_name == "LineTest")
            .expect("Should find LineTest");

        // Each CoverageData represents a single line
        // We should have multiple entries for different lines
        assert!(
            coverage_data.iter().any(|d| d.line_number == 10),
            "Should have coverage data for line 10"
        );
        assert!(
            coverage_data.iter().any(|d| d.line_number == 11),
            "Should have coverage data for line 11"
        );

        let line_10_data = coverage_data
            .iter()
            .find(|d| d.line_number == 10)
            .expect("Should find line 10 data");
        assert_eq!(
            line_10_data.instruction_missed, 0,
            "Line 10 should have 0 missed instructions"
        );
        assert_eq!(
            line_10_data.instruction_covered, 5,
            "Line 10 should have 5 covered instructions"
        );

        let line_11_data = coverage_data
            .iter()
            .find(|d| d.line_number == 11)
            .expect("Should find line 11 data");
        assert_eq!(
            line_11_data.instruction_missed, 3,
            "Line 11 should have 3 missed instructions"
        );
        assert_eq!(
            line_11_data.instruction_covered, 0,
            "Line 11 should have 0 covered instructions"
        );
    }
}
