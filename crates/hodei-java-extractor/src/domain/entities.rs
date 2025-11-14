//! Domain Entities for Java Analysis (Hexagonal Architecture - Domain Layer)
//!
//! This module contains pure business logic entities with no external dependencies.
//! These are the core types that represent the domain concepts of Java analysis.

use hodei_ir::{Fact, ProjectPath};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(test)]
use proptest::prelude::*;

/// Java source file identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JavaSourceId {
    pub package: String,
    pub class_name: String,
    pub file_path: ProjectPath,
}

/// Java package information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JavaPackage {
    pub name: String,
    pub source_root: PathBuf,
}

/// Java class entity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JavaClass {
    pub id: JavaSourceId,
    pub is_public: bool,
    pub is_abstract: bool,
    pub super_class: Option<String>,
    pub interfaces: Vec<String>,
}

/// Java method entity
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JavaMethod {
    pub class_id: JavaSourceId,
    pub name: String,
    pub signature: String,
    pub parameters: Vec<JavaParameter>,
    pub return_type: String,
    pub is_public: bool,
    pub is_static: bool,
}

/// Java parameter
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JavaParameter {
    pub name: String,
    pub type_name: String,
}

/// Coverage data from JaCoCo (Level 1)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoverageData {
    pub source_id: JavaSourceId,
    pub line_number: u32,
    pub instruction_missed: u32,
    pub instruction_covered: u32,
    pub branch_missed: u32,
    pub branch_covered: u32,
    pub coverage_percentage: f64,
}

/// Extraction level enumeration (following EPIC-22 specification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExtractionLevel {
    /// Level 1: Coverage analysis (JaCoCo adapter)
    Level1,
    /// Level 2: Pattern matching (tree-sitter declarative)
    Level2,
    /// Level 3: Deep semantic analysis (Spoon + hodei-deep-analysis-engine)
    Level3,
}

impl ExtractionLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Level1 => "level1",
            Self::Level2 => "level2",
            Self::Level3 => "level3",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "level1" => Some(Self::Level1),
            "level2" => Some(Self::Level2),
            "level3" => Some(Self::Level3),
            _ => None,
        }
    }
}

/// Java analysis configuration (Domain)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JavaAnalysisConfig {
    pub level: ExtractionLevel,
    pub source_paths: Vec<PathBuf>,
    pub include_packages: Vec<String>,
    pub exclude_packages: Vec<String>,
    pub enable_cache: bool,
}

impl Default for JavaAnalysisConfig {
    fn default() -> Self {
        Self {
            level: ExtractionLevel::Level2,
            source_paths: vec!["src/main/java".into()],
            include_packages: vec![],
            exclude_packages: vec![],
            enable_cache: true,
        }
    }
}

/// Result of Java analysis (Domain)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaAnalysisResult {
    pub level: ExtractionLevel,
    pub facts: Vec<Fact>,
    pub source_count: usize,
    pub fact_count: usize,
    pub execution_time_ms: u64,
}

/// Taint source domain entity (Level 3)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaintSource {
    pub source_id: JavaSourceId,
    pub line_number: u32,
    pub parameter_name: String,
    pub annotation: Option<String>,
    pub tags: Vec<String>,
}

/// Taint sink domain entity (Level 3)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaintSink {
    pub sink_id: JavaSourceId,
    pub line_number: u32,
    pub method_name: String,
    pub class_name: String,
    pub category: String,
    pub severity: String,
}

/// Connascence finding (Level 3 - Architectural Analysis)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnascenceFinding {
    pub entity_a: JavaSourceId,
    pub entity_b: JavaSourceId,
    pub connascence_type: ConnascenceType,
    pub strength: Strength,
    pub line_number: u32,
}

/// Types of connascence (from EPIC-20 specification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnascenceType {
    Name,
    Type,
    Meaning,
    Position,
    Algorithm,
}

impl ConnascenceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Type => "type",
            Self::Meaning => "meaning",
            Self::Position => "position",
            Self::Algorithm => "algorithm",
        }
    }
}

/// Strength of connascence coupling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Strength {
    Low,
    Medium,
    High,
}

impl Strength {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// Domain error types (Hexagonal - Domain Layer)
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Invalid source path: {0}")]
    InvalidSourcePath(String),

    #[error("Java source not found: {0}")]
    SourceNotFound(String),

    #[error("Analysis configuration error: {0}")]
    ConfigError(String),

    #[error("Domain validation error: {0}")]
    ValidationError(String),

    #[error("I/O error: {0}")]
    Io(String),
}

/// Result type for domain operations
pub type DomainResult<T> = std::result::Result<T, DomainError>;

/// Domain repository interface (Port)
pub trait JavaSourceRepository: Send + Sync {
    fn find_by_package(&self, package: &str) -> DomainResult<Vec<JavaClass>>;
    fn get_coverage_data(&self, source_id: &JavaSourceId) -> DomainResult<Option<CoverageData>>;
    fn save_analysis_result(&self, result: &JavaAnalysisResult) -> DomainResult<()>;
}

/// Domain service interface (Port)
pub trait JavaAnalysisService: Send + Sync {
    fn analyze_level1(&self, config: &JavaAnalysisConfig) -> DomainResult<JavaAnalysisResult>;
    fn analyze_level2(&self, config: &JavaAnalysisConfig) -> DomainResult<JavaAnalysisResult>;
    fn analyze_level3(&self, config: &JavaAnalysisConfig) -> DomainResult<JavaAnalysisResult>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_java_source_id_creation() {
        let source_id = JavaSourceId {
            package: "com.example".to_string(),
            class_name: "UserService".to_string(),
            file_path: ProjectPath::new(PathBuf::from(
                "src/main/java/com/example/UserService.java",
            )),
        };

        assert_eq!(source_id.package, "com.example");
        assert_eq!(source_id.class_name, "UserService");
    }

    #[test]
    fn test_extraction_level_conversion() {
        let level1 = ExtractionLevel::Level1;
        assert_eq!(level1.as_str(), "level1");
        assert_eq!(
            ExtractionLevel::from_str("level1"),
            Some(ExtractionLevel::Level1)
        );

        let level2 = ExtractionLevel::Level2;
        assert_eq!(level2.as_str(), "level2");
        assert_eq!(
            ExtractionLevel::from_str("level2"),
            Some(ExtractionLevel::Level2)
        );

        let level3 = ExtractionLevel::Level3;
        assert_eq!(level3.as_str(), "level3");
        assert_eq!(
            ExtractionLevel::from_str("level3"),
            Some(ExtractionLevel::Level3)
        );
    }

    #[test]
    fn test_java_analysis_config_default() {
        let config = JavaAnalysisConfig::default();

        assert_eq!(config.level, ExtractionLevel::Level2);
        assert_eq!(
            config.source_paths,
            vec![std::path::PathBuf::from("src/main/java")]
        );
        assert!(config.include_packages.is_empty());
        assert!(config.exclude_packages.is_empty());
        assert!(config.enable_cache);
    }

    #[test]
    fn test_coverage_data_creation() {
        let source_id = JavaSourceId {
            package: "com.example".to_string(),
            class_name: "UserService".to_string(),
            file_path: ProjectPath::new(PathBuf::from(
                "src/main/java/com/example/UserService.java",
            )),
        };

        let coverage = CoverageData {
            source_id,
            line_number: 42,
            instruction_missed: 5,
            instruction_covered: 10,
            branch_missed: 2,
            branch_covered: 0,
            coverage_percentage: 66.67,
        };

        assert_eq!(coverage.line_number, 42);
        assert_eq!(coverage.instruction_missed, 5);
        assert_eq!(coverage.branch_missed, 2);
    }

    #[test]
    fn test_connascence_type_to_str() {
        assert_eq!(ConnascenceType::Name.as_str(), "name");
        assert_eq!(ConnascenceType::Type.as_str(), "type");
        assert_eq!(ConnascenceType::Meaning.as_str(), "meaning");
        assert_eq!(ConnascenceType::Position.as_str(), "position");
        assert_eq!(ConnascenceType::Algorithm.as_str(), "algorithm");
    }

    #[test]
    fn test_strength_to_str() {
        assert_eq!(Strength::Low.as_str(), "low");
        assert_eq!(Strength::Medium.as_str(), "medium");
        assert_eq!(Strength::High.as_str(), "high");
    }

    #[test]
    fn test_taint_source_creation() {
        let source_id = JavaSourceId {
            package: "com.example".to_string(),
            class_name: "UserController".to_string(),
            file_path: ProjectPath::new(PathBuf::from(
                "src/main/java/com/example/UserController.java",
            )),
        };

        let taint_source = TaintSource {
            source_id,
            line_number: 15,
            parameter_name: "username".to_string(),
            annotation: Some("@RequestParam".to_string()),
            tags: vec!["http-input".to_string(), "user-input".to_string()],
        };

        assert_eq!(taint_source.parameter_name, "username");
        assert_eq!(taint_source.tags.len(), 2);
        assert!(taint_source.annotation.is_some());
    }

    #[test]
    fn test_taint_sink_creation() {
        let sink_id = JavaSourceId {
            package: "com.example".to_string(),
            class_name: "UserRepository".to_string(),
            file_path: ProjectPath::new(PathBuf::from(
                "src/main/java/com/example/UserRepository.java",
            )),
        };

        let taint_sink = TaintSink {
            sink_id,
            line_number: 28,
            method_name: "executeQuery".to_string(),
            class_name: "java.sql.Statement".to_string(),
            category: "SqlQuery".to_string(),
            severity: "critical".to_string(),
        };

        assert_eq!(taint_sink.category, "SqlQuery");
        assert_eq!(taint_sink.severity, "critical");
        assert_eq!(taint_sink.method_name, "executeQuery");
    }

    #[test]
    fn test_connascence_finding_creation() {
        let entity_a = JavaSourceId {
            package: "com.example.service".to_string(),
            class_name: "UserService".to_string(),
            file_path: ProjectPath::new(PathBuf::from(
                "src/main/java/com/example/service/UserService.java",
            )),
        };

        let entity_b = JavaSourceId {
            package: "com.example.dto".to_string(),
            class_name: "UserDTO".to_string(),
            file_path: ProjectPath::new(PathBuf::from(
                "src/main/java/com/example/dto/UserDTO.java",
            )),
        };

        let finding = ConnascenceFinding {
            entity_a,
            entity_b,
            connascence_type: ConnascenceType::Position,
            strength: Strength::High,
            line_number: 50,
        };

        assert_eq!(finding.connascence_type, ConnascenceType::Position);
        assert_eq!(finding.strength, Strength::High);
        assert_eq!(finding.line_number, 50);
    }

    #[test]
    fn test_domain_error_messages() {
        let error = DomainError::InvalidSourcePath("invalid path".to_string());
        assert!(error.to_string().contains("Invalid source path"));

        let error = DomainError::SourceNotFound("MissingSource.java".to_string());
        assert!(error.to_string().contains("Java source not found"));

        let error = DomainError::ConfigError("invalid config".to_string());
        assert!(error.to_string().contains("Analysis configuration error"));

        let error = DomainError::ValidationError("validation failed".to_string());
        assert!(error.to_string().contains("Domain validation error"));
    }
}
