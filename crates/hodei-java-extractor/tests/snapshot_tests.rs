//! Snapshot Testing for Java Extractor
//!
//! These tests use insta to capture and verify outputs,
//! preventing regressions in JSON structures, ASTs, and analysis results.

#[cfg(test)]
mod snapshot_tests {
    use insta::{assert_debug_snapshot, assert_json_snapshot, assert_snapshot, settings::set_setting};
    use hodei_java_extractor::{ExtractionLevel, JacocoAdapter, JavaSourceId};
    use hodei_ir::ProjectPath;
    use std::path::PathBuf;

    // Configure insta to show full diffs
    fn configure_insta() {
        set_setting("long_click_distance", 100);
        set_setting("line_length", 200);
    }

    #[test]
    fn extraction_level_enum_snapshots() {
        configure_insta();

        let levels = vec![
            ExtractionLevel::Level1,
            ExtractionLevel::Level2,
            ExtractionLevel::Level3,
        ];

        for level in levels {
            // Snapshot of as_str() output
            assert_snapshot!(level.as_str(),
                format!("extraction_level_{}_as_str", level.as_str()));

            // Snapshot of from_str() output
            assert_snapshot!(ExtractionLevel::from_str(level.as_str()),
                format!("extraction_level_{}_from_str", level.as_str()));
        }
    }

    #[test]
    fn coverage_data_structure_snapshot() {
        configure_insta();

        let source_id = JavaSourceId {
            package: "com.example.test".to_string(),
            class_name: "UserController".to_string(),
            file_path: ProjectPath::new(PathBuf::from("src/main/java/com/example/test/UserController.java")),
        };

        let coverage_data = hodei_java_extractor::domain::entities::CoverageData {
            source_id,
            line_number: 42,
            instruction_missed: 5,
            instruction_covered: 95,
            branch_missed: 2,
            branch_covered: 8,
            coverage_percentage: 95.0,
        };

        // Capture full structure
        assert_debug_snapshot!(coverage_data);
    }

    #[test]
    fn jacoco_xml_parsing_output() {
        configure_insta();

        let temp_dir = tempfile::tempdir().unwrap();
        let jacoco_file = temp_dir.path().join("jacoco.xml");

        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<report name="Test JaCoCo Report">
  <sessioninfo id="test-session" start="123456789" dump="123456790"/>
  <package name="com/example/controller">
    <class name="com/example/controller/UserController" sourcefilename="UserController.java">
      <method name="login" desc="(Ljava/lang/String;Ljava/lang/String;)V" line="10">
        <line nr="10" mi="0" ci="5" mb="0" cb="2"/>
        <line nr="11" mi="0" ci="3" mb="0" cb="2"/>
        <line nr="12" mi="5" ci="0" mb="2" cb="0"/>
        <line nr="13" mi="0" ci="0" mb="0" cb="0"/>
      </method>
      <method name="logout" desc="()V" line="20">
        <line nr="20" mi="1" ci="4" mb="0" cb="2"/>
      </method>
    </class>
    <class name="com/example/controller/AuthService" sourcefilename="AuthService.java">
      <method name="authenticate" desc="(Ljava/lang/String;Ljava/lang/String;)Z" line="30">
        <line nr="30" mi="2" ci="8" mb="1" cb="1"/>
      </method>
    </class>
  </package>
</report>
"#;

        std::fs::write(&jacoco_file, xml_content).unwrap();

        let mut adapter = JacocoAdapter::new(jacoco_file);
        let result = adapter.load_coverage_data();

        assert!(result.is_ok());
        let coverage_data = result.unwrap();

        // Snapshot the parsed coverage data
        assert_json_snapshot!(coverage_data,
            { "[].source_id.package" => "package_name", "[].source_id.class_name" => "class_name" });

        drop(temp_dir);
    }

    #[test]
    fn error_message_snapshots() {
        configure_insta();

        let error_cases = vec![
            hodei_java_extractor::domain::entities::DomainError::Io("File not found".to_string()),
            hodei_java_extractor::domain::entities::DomainError::SourceNotFound("TestClass.java".to_string()),
            hodei_java_extractor::domain::entities::DomainError::ValidationError("Invalid XML".to_string()),
            hodei_java_extractor::domain::entities::DomainError::ConfigError("Missing parameter".to_string()),
        ];

        for (i, error) in error_cases.into_iter().enumerate() {
            assert_snapshot!(error.to_string(), format!("error_case_{}", i));
        }
    }

    #[test]
    fn package_name_parsing_snapshot() {
        configure_insta();

        let test_packages = vec![
            "com.example",
            "org.springframework.boot",
            "io.kubernetes.client",
            "javax.servlet",
            "a.b.c.d.e.f", // Deep nesting
            "single", // Single word
            "", // Empty (edge case)
        ];

        for package in test_packages {
            let source_id = JavaSourceId {
                package: package.to_string(),
                class_name: "TestClass".to_string(),
                file_path: ProjectPath::new(PathBuf::from(format!("{}/TestClass.java", package.replace('.', "/")))),
            };

            // Snapshot package parsing
            assert_snapshot!(source_id.package,
                format!("package_{}", package.replace('.", "_").replace('/', "_")));
        }
    }

    #[test]
    fn coverage_calculation_snapshots() {
        configure_insta();

        let test_cases = vec![
            // (mi, ci, mb, cb, expected_coverage)
            (0, 100, 0, 10, 100.0), // 100% coverage
            (100, 0, 10, 0, 0.0), // 0% coverage
            (50, 50, 5, 5, 50.0), // 50% coverage
            (25, 75, 3, 7, 75.0), // 75% coverage
            (0, 0, 0, 0, 0.0), // No coverage data
        ];

        for (mi, ci, mb, cb, expected) in test_cases {
            let total = mi + ci;
            let coverage_pct = if total > 0 {
                (ci as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            assert!((coverage_pct - expected).abs() < 0.01,
                "Coverage mismatch for mi={}, ci={}, mb={}, cb={}", mi, ci, mb, cb);

            // Snapshot calculation result
            let result = format!("mi:{}, ci:{}, mb:{}, cb:{}, coverage:{}%", mi, ci, mb, cb, coverage_pct);
            assert_snapshot!(result,
                format!("coverage_calc_{}_{}_{}_{}_{}", mi, ci, mb, cb, expected as i32));
        }
    }

    #[test]
    fn complex_jacoco_xml_snapshot() {
        configure_insta();

        let temp_dir = tempfile::tempdir().unwrap();
        let jacoco_file = temp_dir.path().join("complex.xml");

        // More complex XML with multiple packages and classes
        let xml_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<report name="Complex JaCoCo Report">
  <sessioninfo id="complex-session" start="123456789" dump="123456790"/>
  <package name="com/example/service">
    <class name="com/example/service/UserService" sourcefilename="UserService.java">
      <method name="createUser" desc="(Lcom/example/model/User;)Lcom/example/model/User;" line="15">
        <line nr="15" mi="0" ci="8" mb="0" cb="2"/>
        <line nr="16" mi="1" ci="7" mb="1" cb="1"/>
        <line nr="17" mi="0" ci="0" mb="0" cb="0"/>
      </method>
      <method name="updateUser" desc="(Ljava/lang/String;Lcom/example/model/User;)Z" line="35">
        <line nr="35" mi="3" ci="12" mb="2" cb="4"/>
      </method>
    </class>
  </package>
  <package name="com/example/repository">
    <class name="com/example/repository/UserRepository" sourcefilename="UserRepository.java">
      <method name="findById" desc="(Ljava/lang/String;)Lcom/example/model/User;" line="10">
        <line nr="10" mi="0" ci="6" mb="0" cb="2"/>
        <line nr="11" mi="2" ci="4" mb="1" cb="1"/>
      </method>
    </class>
  </package>
  <package name="com/example/util">
    <class name="com/example/util/ValidationUtils" sourcefilename="ValidationUtils.java">
      <method name="validateEmail" desc="(Ljava/lang/String;)Z" line="5">
        <line nr="5" mi="1" ci="9" mb="0" cb="2"/>
      </method>
    </class>
  </package>
</report>
"#;

        std::fs::write(&jacoco_file, xml_content).unwrap();

        let mut adapter = JacocoAdapter::new(jacoco_file);
        let result = adapter.load_coverage_data();

        assert!(result.is_ok());
        let coverage_data = result.unwrap();

        // Sort for consistent snapshots
        let mut sorted_data = coverage_data;
        sorted_data.sort_by(|a, b| {
            a.source_id.package.cmp(&b.source_id.package)
                .then_with(|| a.source_id.class_name.cmp(&b.source_id.class_name))
                .then_with(|| a.line_number.cmp(&b.line_number))
        });

        // Snapshot with redacted file paths
        assert_json_snapshot!(sorted_data, {
            "[].source_id.file_path.path" => "file_path_redacted"
        });

        drop(temp_dir);
    }

    #[test]
    fn extraction_level_conversion_snapshots() {
        configure_insta();

        let test_inputs = vec![
            "level1",
            "level2",
            "level3",
            "invalid",
            "LEVEL1",
            "Level1",
            "",
        ];

        for input in test_inputs {
            let result = ExtractionLevel::from_str(input);
            assert_snapshot!(result, format!("from_str_{}", input.replace('-', "_")));
        }
    }

    #[test]
    fn java_source_id_structure_snapshot() {
        configure_insta();

        let test_cases = vec![
            "com/example/controller/UserController",
            "org/springframework/boot/Application",
            "io/kubernetes/client/ApiClient",
            "a/b/c/D", // Shallow
        ];

        for qualified_name in test_cases {
            let parts: Vec<&str> = qualified_name.rsplitn(2, '/').collect();
            let (class_name, package) = if parts.len() == 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                (qualified_name.to_string(), "".to_string())
            };

            let source_id = JavaSourceId {
                package,
                class_name,
                file_path: ProjectPath::new(PathBuf::from(format!("{}.java", qualified_name.replace('/', "/")))),
            };

            assert_debug_snapshot!(source_id,
                format!("java_source_id_{}", qualified_name.replace('/', "_")));
        }
    }
}
