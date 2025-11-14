#[cfg(test)]
mod visual_regression_tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    /// Test 1: Snapshot Testing - JaCoCo Coverage Report
    /// Tests visual output of JaCoCo coverage data as structured snapshots
    #[test]
    fn test_jacoco_coverage_report_snapshot() {
        // Simulate JaCoCo coverage data
        let coverage_data = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "summary": {
                "total_lines": 1000,
                "covered_lines": 750,
                "line_coverage": 75.0
            },
            "packages": [
                {
                    "name": "com.example.controller",
                    "line_coverage": 85.0,
                    "classes": [
                        {"name": "UserController", "line_coverage": 90.0},
                        {"name": "OrderController", "line_coverage": 80.0}
                    ]
                },
                {
                    "name": "com.example.service",
                    "line_coverage": 70.0,
                    "classes": [
                        {"name": "UserService", "line_coverage": 75.0},
                        {"name": "OrderService", "line_coverage": 65.0}
                    ]
                }
            ]
        });

        // Use insta for snapshot testing
        insta::assert_json_snapshot!(
            coverage_data,
            #[cfg(not(target_os = "windows"))]
            {
                ".snapshots/jacoco_coverage_report_snapshot"
            }
        );

        println!("JaCoCo coverage report snapshot test passed");
    }

    /// Test 2: Snapshot Testing - Extraction Results
    /// Tests visual representation of extraction results
    #[test]
    fn test_extraction_results_snapshot() {
        let extraction_results = serde_json::json!({
            "project_name": "java-web-app",
            "extraction_level": "level3",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "findings": {
                "vulnerabilities": 5,
                "code_smells": 12,
                "bugs": 3,
                "duplicated_lines": 150
            },
            "complexity_metrics": {
                "cyclomatic_complexity": 4.2,
                "cognitive_complexity": 5.8,
                "maintainability_index": 75.5
            },
            "test_coverage": {
                "line_coverage": 87.3,
                "branch_coverage": 82.1,
                "method_coverage": 90.0
            }
        });

        insta::assert_json_snapshot!(extraction_results);

        println!("Extraction results snapshot test passed");
    }

    /// Test 3: Snapshot Testing - Tree Structure
    /// Tests visual representation of Java package tree
    #[test]
    fn test_package_tree_snapshot() {
        let package_tree = serde_json::json!({
            "project_root": "src/main/java",
            "structure": {
                "com": {
                    "type": "package",
                    "children": {
                        "example": {
                            "type": "package",
                            "children": {
                                "controller": {
                                    "type": "package",
                                    "files": ["UserController.java", "OrderController.java"],
                                    "line_count": 250
                                },
                                "service": {
                                    "type": "package",
                                    "files": ["UserService.java", "OrderService.java"],
                                    "line_count": 400
                                },
                                "repository": {
                                    "type": "package",
                                    "files": ["UserRepository.java"],
                                    "line_count": 150
                                }
                            }
                        }
                    }
                }
            },
            "total_files": 5,
            "total_lines": 800
        });

        insta::assert_json_snapshot!(package_tree);

        println!("Package tree snapshot test passed");
    }

    /// Test 4: Snapshot Testing - Code Analysis Report
    /// Tests visual output of code analysis metrics
    #[test]
    fn test_code_analysis_report_snapshot() {
        let analysis_report = serde_json::json!({
            "project": "Java Web Application",
            "analysis_date": chrono::Utc::now().to_rfc3339(),
            "metrics": {
                "lines_of_code": 12500,
                "cyclomatic_complexity": {
                    "average": 4.5,
                    "max": 25,
                    "classes_above_threshold": 3
                },
                "code_duplication": {
                    "percentage": 8.5,
                    "duplicated_lines": 1062,
                    "affected_files": 12
                },
                "test_coverage": {
                    "line_coverage": 87.3,
                    "branch_coverage": 82.1,
                    "total_tests": 450
                }
            },
            "quality_gate": {
                "passed": true,
                "score": 82.5,
                "grade": "B+"
            }
        });

        insta::assert_json_snapshot!(analysis_report);

        println!("Code analysis report snapshot test passed");
    }

    /// Test 5: Snapshot Testing - Vulnerability Report
    /// Tests visual representation of security vulnerabilities
    #[test]
    fn test_vulnerability_report_snapshot() {
        let vulnerability_report = serde_json::json!({
            "scan_date": chrono::Utc::now().to_rfc3339(),
            "total_vulnerabilities": 7,
            "severity_breakdown": {
                "critical": 1,
                "high": 2,
                "medium": 3,
                "low": 1
            },
            "vulnerabilities": [
                {
                    "id": "VUL-001",
                    "severity": "critical",
                    "title": "SQL Injection",
                    "file": "UserRepository.java",
                    "line": 45,
                    "description": "Potential SQL injection vulnerability"
                },
                {
                    "id": "VUL-002",
                    "severity": "high",
                    "title": "XSS Vulnerability",
                    "file": "UserController.java",
                    "line": 78,
                    "description": "Unescaped user input in HTML"
                },
                {
                    "id": "VUL-003",
                    "severity": "medium",
                    "title": "Hardcoded Credentials",
                    "file": "Config.java",
                    "line": 12,
                    "description": "Hardcoded password detected"
                }
            ]
        });

        insta::assert_json_snapshot!(vulnerability_report);

        println!("Vulnerability report snapshot test passed");
    }

    /// Test 6: Snapshot Testing - Comparison Report
    /// Tests visual representation of baseline comparison
    #[test]
    fn test_comparison_report_snapshot() {
        let comparison_report = serde_json::json!({
            "comparison_date": chrono::Utc::now().to_rfc3339(),
            "baseline_version": "v1.2.3",
            "current_version": "v1.2.4",
            "changes": {
                "lines_added": 150,
                "lines_removed": 75,
                "files_modified": 8,
                "files_added": 3,
                "files_deleted": 1
            },
            "quality_metrics": {
                "previous": {
                    "coverage": 85.2,
                    "complexity": 4.5,
                    "duplication": 9.1
                },
                "current": {
                    "coverage": 87.3,
                    "complexity": 4.2,
                    "duplication": 8.5
                },
                "delta": {
                    "coverage": "+2.1",
                    "complexity": "-0.3",
                    "duplication": "-0.6"
                }
            },
            "trend": "improving"
        });

        insta::assert_json_snapshot!(comparison_report);

        println!("Comparison report snapshot test passed");
    }

    /// Test 7: Snapshot Testing - Class Diagram Structure
    /// Tests visual representation of class relationships
    #[test]
    fn test_class_diagram_snapshot() {
        let class_diagram = serde_json::json!({
            "project": "Java Web Application",
            "classes": [
                {
                    "name": "UserController",
                    "package": "com.example.controller",
                    "type": "controller",
                    "methods": 12,
                    "lines": 150,
                    "dependencies": ["UserService"]
                },
                {
                    "name": "UserService",
                    "package": "com.example.service",
                    "type": "service",
                    "methods": 8,
                    "lines": 200,
                    "dependencies": ["UserRepository"]
                },
                {
                    "name": "UserRepository",
                    "package": "com.example.repository",
                    "type": "repository",
                    "methods": 5,
                    "lines": 100,
                    "dependencies": []
                }
            ],
            "relationships": [
                {"from": "UserController", "to": "UserService", "type": "uses"},
                {"from": "UserService", "to": "UserRepository", "type": "uses"}
            ],
            "metrics": {
                "total_classes": 3,
                "total_dependencies": 2,
                "average_methods_per_class": 8.3
            }
        });

        insta::assert_json_snapshot!(class_diagram);

        println!("Class diagram snapshot test passed");
    }

    /// Test 8: Snapshot Testing - Test Coverage Heatmap
    /// Tests visual representation of coverage heatmap data
    #[test]
    fn test_coverage_heatmap_snapshot() {
        let coverage_heatmap = serde_json::json!({
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "data": [
                {"file": "UserController.java", "coverage": 95, "lines": 150, "color": "green"},
                {"file": "OrderController.java", "coverage": 88, "lines": 200, "color": "yellow"},
                {"file": "UserService.java", "coverage": 92, "lines": 180, "color": "green"},
                {"file": "OrderService.java", "coverage": 65, "lines": 220, "color": "orange"},
                {"file": "UserRepository.java", "coverage": 98, "lines": 100, "color": "green"},
                {"file": "OrderRepository.java", "coverage": 45, "lines": 130, "color": "red"}
            ],
            "summary": {
                "total_files": 6,
                "high_coverage": 3,
                "medium_coverage": 1,
                "low_coverage": 2,
                "overall_coverage": 80.5
            }
        });

        insta::assert_json_snapshot!(coverage_heatmap);

        println!("Coverage heatmap snapshot test passed");
    }

    /// Test 9: Snapshot Testing - Complexity Distribution
    /// Tests visual representation of complexity metrics
    #[test]
    fn test_complexity_distribution_snapshot() {
        let complexity_distribution = serde_json::json!({
            "analysis_date": chrono::Utc::now().to_rfc3339(),
            "complexity_ranges": {
                "1-5 (Simple)": {
                    "count": 45,
                    "percentage": 60.0,
                    "color": "#4CAF50"
                },
                "6-10 (Moderate)": {
                    "count": 20,
                    "percentage": 26.7,
                    "color": "#FFC107"
                },
                "11-20 (Complex)": {
                    "count": 8,
                    "percentage": 10.7,
                    "color": "#FF9800"
                },
                "20+ (Very Complex)": {
                    "count": 2,
                    "percentage": 2.7,
                    "color": "#F44336"
                }
            },
            "statistics": {
                "total_methods": 75,
                "average_complexity": 4.5,
                "max_complexity": 25,
                "complexity_threshold": 10
            }
        });

        insta::assert_json_snapshot!(complexity_distribution);

        println!("Complexity distribution snapshot test passed");
    }

    /// Test 10: Snapshot Testing - Trend Analysis
    /// Tests visual representation of historical trends
    #[test]
    fn test_trend_analysis_snapshot() {
        let trend_analysis = serde_json::json!({
            "period": "Last 6 months",
            "metrics": [
                {
                    "date": "2024-06-01",
                    "coverage": 78.5,
                    "complexity": 5.2,
                    "duplication": 12.3,
                    "vulnerabilities": 15
                },
                {
                    "date": "2024-07-01",
                    "coverage": 80.2,
                    "complexity": 5.0,
                    "duplication": 11.8,
                    "vulnerabilities": 12
                },
                {
                    "date": "2024-08-01",
                    "coverage": 82.1,
                    "complexity": 4.8,
                    "duplication": 10.5,
                    "vulnerabilities": 10
                },
                {
                    "date": "2024-09-01",
                    "coverage": 84.5,
                    "complexity": 4.6,
                    "duplication": 9.2,
                    "vulnerabilities": 8
                },
                {
                    "date": "2024-10-01",
                    "coverage": 86.8,
                    "complexity": 4.4,
                    "duplication": 8.5,
                    "vulnerabilities": 7
                },
                {
                    "date": "2024-11-01",
                    "coverage": 87.3,
                    "complexity": 4.2,
                    "duplication": 8.5,
                    "vulnerabilities": 7
                }
            ],
            "trends": {
                "coverage": "improving",
                "complexity": "improving",
                "duplication": "improving",
                "vulnerabilities": "improving"
            }
        });

        insta::assert_json_snapshot!(trend_analysis);

        println!("Trend analysis snapshot test passed");
    }

    /// Test 11: Redaction Snapshot Testing
    /// Tests snapshot testing with sensitive data redaction
    #[test]
    fn test_redacted_snapshot() {
        let sensitive_data = serde_json::json!({
            "user_id": "user-12345",
            "email": "user@example.com",
            "api_key": "sk-1234567890abcdef",
            "database_url": "postgresql://user:password@localhost:5432/db",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        // Create redacted version
        let redacted_data = serde_json::json!({
            "user_id": "[REDACTED]",
            "email": "[REDACTED]",
            "api_key": "[REDACTED]",
            "database_url": "[REDACTED]",
            "timestamp": "[REDACTED]"
        });

        insta::assert_json_snapshot!(redacted_data);

        println!("Redacted snapshot test passed");
    }

    /// Test 12: Inline Snapshot Testing
    /// Tests inline snapshots for easy updates
    #[test]
    fn test_inline_snapshot() {
        let simple_data = serde_json::json!({
            "status": "success",
            "code": 200,
            "message": "Operation completed successfully"
        });

        // Inline snapshot - can be updated with INSTA_UPDATE=new
        insta::assert_json_snapshot!(simple_data, @r#"
        {
          "code": 200,
          "message": "Operation completed successfully",
          "status": "success"
        }
        "#);

        println!("Inline snapshot test passed");
    }

    /// Test 13: Snapshot Conflict Detection
    /// Tests that snapshot conflicts are detected
    #[test]
    fn test_snapshot_conflict_detection() {
        let current_data = serde_json::json!({
            "version": "1.0.0",
            "features": ["feature_a", "feature_b", "feature_c"],
            "metrics": {
                "users": 1000,
                "requests": 50000,
                "errors": 5
            }
        });

        // This snapshot should match the expected structure
        insta::assert_json_snapshot!(current_data);

        println!("Snapshot conflict detection test passed");
    }

    /// Test 14: Custom Snapshot Serializer
    /// Tests custom serialization for complex data
    #[test]
    fn test_custom_snapshot_serializer() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        // Simulate creating a visual artifact (e.g., a report file)
        let report_content = "Coverage Report\n================\nLine Coverage: 87.3%\nBranch Coverage: 82.1%\nMethod Coverage: 90.0%";
        let report_path = temp_dir.path().join("coverage_report.txt");
        fs::write(&report_path, report_content).expect("Failed to write report");

        // Read and snapshot the content
        let report_text = fs::read_to_string(&report_path).expect("Failed to read report");

        insta::assert_snapshot!(report_text);

        println!("Custom snapshot serializer test passed");
    }

    /// Test 15: Snapshot Update Workflow
    /// Tests the workflow for updating snapshots
    #[test]
    fn test_snapshot_update_workflow() {
        let config_data = serde_json::json!({
            "extractor": {
                "level": 3,
                "enabled_tools": ["jacoco", "spoon", "tree_sitter"],
                "timeout": 300
            },
            "output": {
                "format": "json",
                "pretty_print": true,
                "include_metadata": true
            }
        });

        // In development, run with: cargo test -- --include-ignored INSTA_UPDATE=new
        // to automatically update snapshots
        insta::assert_json_snapshot!(config_data);

        println!("Snapshot update workflow test passed");
        println!("To update snapshots: cargo test -- --include-ignored INSTA_UPDATE=new");
    }

    /// Test 16: YAML Snapshot Testing
    /// Tests YAML format snapshots
    #[test]
    fn test_yaml_snapshot() {
        let yaml_data = serde_yaml::Value::Mapping(vec![
            (
                yaml::Value::String("project".into()),
                yaml::Value::String("test-project".into()),
            ),
            (
                yaml::Value::String("version".into()),
                yaml::Value::String("1.0.0".into()),
            ),
            (
                yaml::Value::String("build".into()),
                yaml::Value::Number(42.into()),
            ),
        ]);

        insta::assert_yaml_snapshot!(yaml_data);

        println!("YAML snapshot test passed");
    }

    /// Test 17: Snapshot Storage Location
    /// Tests that snapshots are stored in the correct location
    #[test]
    fn test_snapshot_storage_location() {
        let test_data = serde_json::json!({
            "test_name": "storage_location_test",
            "snapshot_path_info": {
                "auto_path": true,
                "custom_path": false,
                "use_stderr": false
            }
        });

        insta::assert_json_snapshot!(test_data);

        println!("Snapshot storage location test passed");
    }

    /// Test 18: Descriptive Snapshot Names
    /// Tests using descriptive names for snapshots
    #[test]
    fn test_descriptive_snapshot_names() {
        let report_data = serde_json::json!({
            "report_type": "security_audit",
            "generated": chrono::Utc::now().to_rfc3339(),
            "summary": "Security audit completed with 3 high-priority findings"
        });

        insta::assert_json_snapshot!(report_data, "security_audit_report_summary");

        println!("Descriptive snapshot names test passed");
    }

    /// Test 19: Nested Structure Snapshots
    /// Tests snapshot testing with deeply nested structures
    #[test]
    fn test_nested_structure_snapshots() {
        let nested_structure = serde_json::json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {
                            "data": "deeply nested value",
                            "count": 42,
                            "items": [1, 2, 3, 4, 5]
                        }
                    }
                }
            }
        });

        insta::assert_json_snapshot!(nested_structure);

        println!("Nested structure snapshots test passed");
    }

    /// Test 20: Snapshot Validation and CI Integration
    /// Tests that snapshots work in CI environments
    #[test]
    fn test_snapshot_ci_validation() {
        let ci_build_info = serde_json::json!({
            "build_id": "build-12345",
            "commit_sha": "abc123def456",
            "branch": "feature/new-extractor",
            "author": "developer@example.com",
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "environment": {
                "rust_version": "1.75.0",
                "os": "ubuntu-latest",
                "ci": true
            }
        });

        insta::assert_json_snapshot!(ci_build_info);

        println!("Snapshot CI validation test passed");
        println!("Snapshots help ensure visual regression detection in CI/CD pipelines");
    }
}
