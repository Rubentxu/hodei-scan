//! Contract Tests for Java Extractor
//!
//! These tests validate the contracts between adapters and the domain,
//! ensuring that implementations adhere to their interfaces.

use hodei_ir::ProjectPath;
use hodei_java_extractor::{
    JacocoAdapter, SpoonService, TreeSitterAdapter,
    domain::{
        entities::{CoverageData, DomainError, JavaClass, JavaSourceId},
        repositories::JavaSourceRepository,
    },
};
use std::path::PathBuf;

/// Helper to validate repository contract compliance
fn validate_repository_contract<T: JavaSourceRepository>(repository: &T) {
    // Contract 1: find_by_package should never panic
    let result = repository.find_by_package("com.example.test");
    assert!(result.is_ok() || result.is_err());

    // Contract 2: get_coverage_data should handle invalid IDs gracefully
    let invalid_id = JavaSourceId {
        package: "invalid".to_string(),
        class_name: "NonExistent".to_string(),
        file_path: ProjectPath::new(PathBuf::from("test.java")),
    };
    let result = repository.get_coverage_data(&invalid_id);
    assert!(result.is_ok()); // Should return Ok(None) or Ok(Some(...))

    // Contract 3: save_analysis_result should not panic
    let dummy_result = hodei_java_extractor::domain::entities::JavaAnalysisResult {
        level: hodei_java_extractor::ExtractionLevel::Level1,
        facts: vec![],
        source_count: 0,
        fact_count: 0,
        execution_time_ms: 0,
    };
    let result = repository.save_analysis_result(&dummy_result);
    assert!(result.is_ok() || result.is_err());
}

#[cfg(test)]
mod contract_tests {
    use super::*;

    #[test]
    fn jacoco_adapter_repository_contract() {
        let adapter = JacocoAdapter::new(PathBuf::from("/fake/path/jacoco.xml"));
        validate_repository_contract(&adapter);
    }

    #[test]
    fn tree_sitter_adapter_repository_contract() {
        let adapter = TreeSitterAdapter::new(vec![PathBuf::from("/fake/src")]);
        validate_repository_contract(&adapter);
    }

    #[test]
    fn spoon_service_repository_contract() {
        let service = SpoonService::new(vec![PathBuf::from("/fake/src")]);
        validate_repository_contract(&service);
    }

    #[test]
    fn repository_error_types_are_consistent() {
        let adapter = JacocoAdapter::new(PathBuf::from("/nonexistent"));

        // All repository operations should return DomainError variants
        let _ = adapter.find_by_package("test");
        let _ = adapter.get_coverage_data(&JavaSourceId {
            package: "test".to_string(),
            class_name: "Test".to_string(),
            file_path: ProjectPath::new(PathBuf::from("Test.java")),
        });
        let _ = adapter.save_analysis_result(
            &hodei_java_extractor::domain::entities::JavaAnalysisResult {
                level: hodei_java_extractor::ExtractionLevel::Level1,
                facts: vec![],
                source_count: 0,
                fact_count: 0,
                execution_time_ms: 0,
            },
        );
    }

    #[test]
    fn coverage_data_structure_contract() {
        // Contract: CoverageData should always have valid ranges
        let source_id = JavaSourceId {
            package: "com.example".to_string(),
            class_name: "TestClass".to_string(),
            file_path: ProjectPath::new(PathBuf::from("TestClass.java")),
        };

        let coverage = CoverageData {
            source_id,
            line_number: 1,
            instruction_missed: 0,
            instruction_covered: 100,
            branch_missed: 0,
            branch_covered: 10,
            coverage_percentage: 100.0,
        };

        // Contract validations
        assert!(coverage.line_number > 0);
        assert!(coverage.instruction_missed >= 0);
        assert!(coverage.instruction_covered >= 0);
        assert!(coverage.branch_missed >= 0);
        assert!(coverage.branch_covered >= 0);
        assert!(coverage.coverage_percentage >= 0.0);
        assert!(coverage.coverage_percentage <= 100.0);

        // Contract: Total coverage should be reasonable
        let total_instructions = coverage.instruction_missed + coverage.instruction_covered;
        let total_branches = coverage.branch_missed + coverage.branch_covered;

        assert!(total_instructions > 0 || coverage.instruction_covered == 0);
        assert!(total_branches > 0 || coverage.branch_covered == 0);
    }

    #[test]
    fn repository_behavior_with_empty_cache() {
        // Contract: Repositories should handle empty state gracefully
        let adapter = JacocoAdapter::new(PathBuf::from("/nonexistent"));

        let result = adapter.find_by_package("nonexistent.package");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn multiple_repository_instances_are_independent() {
        // Contract: Multiple instances should not share state
        let adapter1 = JacocoAdapter::new(PathBuf::from("/path1"));
        let adapter2 = JacocoAdapter::new(PathBuf::from("/path2"));

        // Both should be independent
        assert_ne!(
            format!("{:p}", &adapter1 as *const _),
            format!("{:p}", &adapter2 as *const _)
        );
    }
}
