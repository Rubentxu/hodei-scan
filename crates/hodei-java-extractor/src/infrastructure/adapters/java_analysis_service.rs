//! Java Analysis Service Implementation
//!
//! This is the main service implementation that orchestrates all adapters
//! (JaCoCo, tree-sitter, Spoon) based on the requested analysis level.

use crate::domain::entities::{
    DomainError, ExtractionLevel, JavaAnalysisConfig, JavaAnalysisResult, JavaAnalysisService,
};
use crate::infrastructure::adapters::{
    jacoco::JacocoAdapter, spoon::SpoonService, tree_sitter::TreeSitterAdapter,
};

/// Composite Java Analysis Service
pub struct JavaAnalysisServiceImpl {
    jacoco_adapter: JacocoAdapter,
    tree_sitter_adapter: TreeSitterAdapter,
    spoon_service: SpoonService,
}

impl JavaAnalysisServiceImpl {
    pub fn new(
        jacoco_report_path: std::path::PathBuf,
        source_paths: Vec<std::path::PathBuf>,
    ) -> Self {
        Self {
            jacoco_adapter: JacocoAdapter::new(jacoco_report_path),
            tree_sitter_adapter: TreeSitterAdapter::new(source_paths.clone()),
            spoon_service: SpoonService::new(source_paths),
        }
    }

    /// Get the appropriate repository adapter for the analysis level
    fn get_repository_for_level(
        &self,
        level: ExtractionLevel,
    ) -> &dyn crate::domain::repositories::JavaSourceRepository {
        match level {
            ExtractionLevel::Level1 => &self.jacoco_adapter,
            ExtractionLevel::Level2 => &self.tree_sitter_adapter,
            ExtractionLevel::Level3 => &self.spoon_service,
        }
    }
}

impl JavaAnalysisService for JavaAnalysisServiceImpl {
    fn analyze_level1(
        &self,
        config: &JavaAnalysisConfig,
    ) -> Result<JavaAnalysisResult, DomainError> {
        // Level 1: Use JaCoCo adapter for coverage analysis
        let repository = self.get_repository_for_level(ExtractionLevel::Level1);
        let classes = repository.find_by_package("")?;

        Ok(JavaAnalysisResult {
            level: ExtractionLevel::Level1,
            facts: vec![], // TODO: Convert coverage data to facts
            source_count: classes.len(),
            fact_count: 0, // TODO: Count actual coverage facts
            execution_time_ms: 0,
        })
    }

    fn analyze_level2(
        &self,
        config: &JavaAnalysisConfig,
    ) -> Result<JavaAnalysisResult, DomainError> {
        // Level 2: Use tree-sitter adapter for pattern matching
        let repository = self.get_repository_for_level(ExtractionLevel::Level2);
        let classes = repository.find_by_package("")?;

        Ok(JavaAnalysisResult {
            level: ExtractionLevel::Level2,
            facts: vec![], // TODO: Convert pattern matches to facts
            source_count: classes.len(),
            fact_count: 0, // TODO: Count actual pattern matching facts
            execution_time_ms: 0,
        })
    }

    fn analyze_level3(
        &self,
        config: &JavaAnalysisConfig,
    ) -> Result<JavaAnalysisResult, DomainError> {
        // Level 3: Use Spoon service for deep semantic analysis
        let repository = self.get_repository_for_level(ExtractionLevel::Level3);
        let classes = repository.find_by_package("")?;

        Ok(JavaAnalysisResult {
            level: ExtractionLevel::Level3,
            facts: vec![], // TODO: Convert taint flows and connascence to facts
            source_count: classes.len(),
            fact_count: 0, // TODO: Count actual semantic facts
            execution_time_ms: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_service_creation() {
        // GREEN: Test should pass
        let _service = JavaAnalysisServiceImpl::new(
            PathBuf::from("target/site/jacoco/jacoco.xml"),
            vec![PathBuf::from("src/main/java")],
        );
        // Service created successfully - no assertion needed on private fields
    }

    #[test]
    fn test_analyze_level1() {
        // RED: Test should fail until implementation
        let service = JavaAnalysisServiceImpl::new(
            PathBuf::from("target/site/jacoco/jacoco.xml"),
            vec![PathBuf::from("src/main/java")],
        );

        let config = crate::domain::entities::JavaAnalysisConfig {
            level: ExtractionLevel::Level1,
            source_paths: vec![PathBuf::from("src/main/java")],
            include_packages: vec![],
            exclude_packages: vec![],
            enable_cache: true,
        };

        let result = service.analyze_level1(&config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().level, ExtractionLevel::Level1);
    }
}
