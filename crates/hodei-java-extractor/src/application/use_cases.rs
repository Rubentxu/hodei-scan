//! Use Cases for Java Analysis
//!
//! Following TDD: Write failing tests first (RED), then implement (GREEN)

use crate::domain::entities::{
    DomainError, ExtractionLevel, JavaAnalysisConfig, JavaAnalysisResult, JavaAnalysisService,
    JavaSourceId, JavaSourceRepository,
};
use hodei_ir::types::project_path::ProjectPath;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Request DTO for analyze Java code use case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeJavaCodeRequest {
    pub source_paths: Vec<PathBuf>,
    pub level: ExtractionLevel,
    pub include_packages: Vec<String>,
    pub exclude_packages: Vec<String>,
}

/// Response DTO for analyze Java code use case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzeJavaCodeResponse {
    pub level: ExtractionLevel,
    pub source_count: usize,
    pub fact_count: usize,
    pub execution_time_ms: u64,
}

/// Use case for analyzing Java code at different levels
pub struct AnalyzeJavaCodeUseCase<'a> {
    pub analysis_service: &'a dyn JavaAnalysisService,
    pub source_repository: &'a dyn JavaSourceRepository,
}

impl<'a> AnalyzeJavaCodeUseCase<'a> {
    pub fn new(
        analysis_service: &'a dyn JavaAnalysisService,
        source_repository: &'a dyn JavaSourceRepository,
    ) -> Self {
        Self {
            analysis_service,
            source_repository,
        }
    }

    pub async fn execute(
        &self,
        request: AnalyzeJavaCodeRequest,
    ) -> Result<AnalyzeJavaCodeResponse, DomainError> {
        let config = JavaAnalysisConfig {
            level: request.level,
            source_paths: request.source_paths,
            include_packages: request.include_packages,
            exclude_packages: request.exclude_packages,
            enable_cache: true,
        };

        let start = std::time::Instant::now();

        let result = match config.level {
            ExtractionLevel::Level1 => self.analysis_service.analyze_level1(&config)?,
            ExtractionLevel::Level2 => self.analysis_service.analyze_level2(&config)?,
            ExtractionLevel::Level3 => self.analysis_service.analyze_level3(&config)?,
        };

        let execution_time = start.elapsed().as_millis() as u64;

        Ok(AnalyzeJavaCodeResponse {
            level: result.level,
            source_count: result.source_count,
            fact_count: result.fact_count,
            execution_time_ms: execution_time,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::JavaSourceId;
    use std::path::PathBuf;

    struct MockAnalysisService;
    struct MockSourceRepository;

    impl JavaAnalysisService for MockAnalysisService {
        fn analyze_level1(
            &self,
            _config: &JavaAnalysisConfig,
        ) -> Result<JavaAnalysisResult, DomainError> {
            Ok(JavaAnalysisResult {
                level: ExtractionLevel::Level1,
                facts: vec![],
                source_count: 5,
                fact_count: 10,
                execution_time_ms: 100,
            })
        }

        fn analyze_level2(
            &self,
            _config: &JavaAnalysisConfig,
        ) -> Result<JavaAnalysisResult, DomainError> {
            Ok(JavaAnalysisResult {
                level: ExtractionLevel::Level2,
                facts: vec![],
                source_count: 5,
                fact_count: 20,
                execution_time_ms: 150,
            })
        }

        fn analyze_level3(
            &self,
            _config: &JavaAnalysisConfig,
        ) -> Result<JavaAnalysisResult, DomainError> {
            Ok(JavaAnalysisResult {
                level: ExtractionLevel::Level3,
                facts: vec![],
                source_count: 5,
                fact_count: 30,
                execution_time_ms: 200,
            })
        }
    }

    impl JavaSourceRepository for MockSourceRepository {
        fn find_by_package(
            &self,
            _package: &str,
        ) -> Result<Vec<crate::domain::entities::JavaClass>, DomainError> {
            Ok(vec![])
        }

        fn get_coverage_data(
            &self,
            _source_id: &JavaSourceId,
        ) -> Result<Option<crate::domain::entities::CoverageData>, DomainError> {
            Ok(None)
        }

        fn save_analysis_result(&self, _result: &JavaAnalysisResult) -> Result<(), DomainError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_analyze_level1() {
        // GREEN: Test should pass
        let use_case = AnalyzeJavaCodeUseCase::new(&MockAnalysisService, &MockSourceRepository);

        let request = AnalyzeJavaCodeRequest {
            source_paths: vec![PathBuf::from("src/main/java")],
            level: ExtractionLevel::Level1,
            include_packages: vec![],
            exclude_packages: vec![],
        };

        let response = use_case.execute(request).await.unwrap();

        assert_eq!(response.level, ExtractionLevel::Level1);
        assert_eq!(response.source_count, 5);
        assert_eq!(response.fact_count, 10);
    }

    #[tokio::test]
    async fn test_analyze_level2() {
        // GREEN: Test should pass
        let use_case = AnalyzeJavaCodeUseCase::new(&MockAnalysisService, &MockSourceRepository);

        let request = AnalyzeJavaCodeRequest {
            source_paths: vec![PathBuf::from("src/main/java")],
            level: ExtractionLevel::Level2,
            include_packages: vec!["com.example".to_string()],
            exclude_packages: vec![],
        };

        let response = use_case.execute(request).await.unwrap();

        assert_eq!(response.level, ExtractionLevel::Level2);
        assert_eq!(response.source_count, 5);
        assert_eq!(response.fact_count, 20);
    }

    #[tokio::test]
    async fn test_analyze_level3() {
        // GREEN: Test should pass
        let use_case = AnalyzeJavaCodeUseCase::new(&MockAnalysisService, &MockSourceRepository);

        let request = AnalyzeJavaCodeRequest {
            source_paths: vec![
                PathBuf::from("src/main/java"),
                PathBuf::from("src/test/java"),
            ],
            level: ExtractionLevel::Level3,
            include_packages: vec![],
            exclude_packages: vec!["com.example.test".to_string()],
        };

        let response = use_case.execute(request).await.unwrap();

        assert_eq!(response.level, ExtractionLevel::Level3);
        assert_eq!(response.source_count, 5);
        assert_eq!(response.fact_count, 30);
    }

    #[test]
    fn test_analyze_java_code_request_creation() {
        // GREEN: Test should pass
        let request = AnalyzeJavaCodeRequest {
            source_paths: vec![PathBuf::from("src/main/java")],
            level: ExtractionLevel::Level2,
            include_packages: vec!["com.example".to_string()],
            exclude_packages: vec!["com.example.internal".to_string()],
        };

        assert_eq!(request.level, ExtractionLevel::Level2);
        assert_eq!(request.source_paths.len(), 1);
        assert_eq!(request.include_packages.len(), 1);
        assert_eq!(request.exclude_packages.len(), 1);
    }

    #[test]
    fn test_analyze_java_code_response_creation() {
        // GREEN: Test should pass
        let response = AnalyzeJavaCodeResponse {
            level: ExtractionLevel::Level1,
            source_count: 10,
            fact_count: 25,
            execution_time_ms: 120,
        };

        assert_eq!(response.level, ExtractionLevel::Level1);
        assert_eq!(response.source_count, 10);
        assert_eq!(response.fact_count, 25);
        assert_eq!(response.execution_time_ms, 120);
    }
}
