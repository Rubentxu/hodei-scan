//! IR Schema Migration Tools

use thiserror::Error;

/// Errors for schema migration
#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("Unsupported IR schema version: {0}")]
    UnsupportedVersion(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),
}

/// Schema version information
#[derive(Debug, Clone, PartialEq)]
pub enum SchemaVersion {
    V32,
    V33,
    Unknown(String),
}

impl SchemaVersion {
    /// Parse schema version string
    pub fn parse(version: &str) -> Self {
        match version {
            "3.2.0" | "3.2" => SchemaVersion::V32,
            "3.3.0" | "3.3" => SchemaVersion::V33,
            _ => SchemaVersion::Unknown(version.to_string()),
        }
    }

    /// Convert back to string
    pub fn to_string(&self) -> String {
        match self {
            SchemaVersion::V32 => "3.2.0".to_string(),
            SchemaVersion::V33 => "3.3.0".to_string(),
            SchemaVersion::Unknown(v) => v.clone(),
        }
    }
}

/// Migrate IR from v3.2 to v3.3
pub fn migrate_ir_version(
    ir: &crate::IntermediateRepresentation,
) -> Result<crate::IntermediateRepresentation, MigrationError> {
    let version = SchemaVersion::parse(&ir.schema_version);

    match version {
        SchemaVersion::V32 => {
            // v3.2 to v3.3 migration: Add Custom FactType support
            // All existing facts remain unchanged, just bump version
            Ok(crate::IntermediateRepresentation {
                facts: ir.facts.clone(),
                metadata: ir.metadata.clone(),
                schema_version: "3.3.0".to_string(),
            })
        }
        SchemaVersion::V33 => {
            // Already v3.3, just return as-is
            Ok(ir.clone())
        }
        SchemaVersion::Unknown(v) => Err(MigrationError::UnsupportedVersion(v)),
    }
}

/// Check if an IR needs migration
pub fn needs_migration(ir: &crate::IntermediateRepresentation) -> bool {
    SchemaVersion::parse(&ir.schema_version) == SchemaVersion::V32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProjectMetadata, ProjectPath};

    #[test]
    fn test_migrate_ir_v32_to_v33() {
        let metadata = ProjectMetadata::new(
            "test-project".to_string(),
            "1.0.0".to_string(),
            ProjectPath::new(std::path::PathBuf::from("/test")),
        );

        let ir_v32 = crate::IntermediateRepresentation {
            facts: Vec::new(),
            metadata: metadata.clone(),
            schema_version: "3.2.0".to_string(),
        };

        let ir_v33 = migrate_ir_version(&ir_v32).unwrap();

        assert_eq!(ir_v33.schema_version, "3.3.0");
        assert_eq!(ir_v33.facts.len(), ir_v32.facts.len());
        assert_eq!(ir_v33.metadata, metadata);
    }

    #[test]
    fn test_migrate_ir_v33_to_v33() {
        let metadata = ProjectMetadata::new(
            "test-project".to_string(),
            "1.0.0".to_string(),
            ProjectPath::new(std::path::PathBuf::from("/test")),
        );

        let ir_v33 = crate::IntermediateRepresentation {
            facts: Vec::new(),
            metadata: metadata.clone(),
            schema_version: "3.3.0".to_string(),
        };

        let migrated = migrate_ir_version(&ir_v33).unwrap();

        assert_eq!(migrated.schema_version, "3.3.0");
        assert_eq!(migrated.facts.len(), ir_v33.facts.len());
        assert_eq!(migrated.metadata, metadata);
    }

    #[test]
    fn test_migrate_with_facts() {
        let metadata = ProjectMetadata::new(
            "test-project".to_string(),
            "1.0.0".to_string(),
            ProjectPath::new(std::path::PathBuf::from("/test")),
        );

        let location = crate::SourceLocation::new(
            crate::ProjectPath::new(std::path::PathBuf::from("/test/file.rs")),
            crate::LineNumber::new(1).unwrap(),
            Some(crate::ColumnNumber::new(1).unwrap()),
            crate::LineNumber::new(1).unwrap(),
            Some(crate::ColumnNumber::new(1).unwrap()),
        );

        let provenance = crate::Provenance::new(
            crate::ExtractorId::Custom,
            "test".to_string(),
            crate::Confidence::HIGH,
        );

        let fact = crate::Fact::new(
            crate::FactType::CodeSmell {
                smell_type: "test_smell".to_string(),
                severity: crate::Severity::Minor,
                message: "test message".to_string(),
            },
            location,
            provenance,
        );

        let ir_v32 = crate::IntermediateRepresentation {
            facts: vec![fact.clone()],
            metadata,
            schema_version: "3.2.0".to_string(),
        };

        let ir_v33 = migrate_ir_version(&ir_v32).unwrap();

        assert_eq!(ir_v33.schema_version, "3.3.0");
        assert_eq!(ir_v33.facts.len(), 1);
        assert_eq!(ir_v33.facts[0], fact);
    }

    #[test]
    fn test_needs_migration() {
        let metadata = ProjectMetadata::new(
            "test-project".to_string(),
            "1.0.0".to_string(),
            ProjectPath::new(std::path::PathBuf::from("/test")),
        );

        let ir_v32 = crate::IntermediateRepresentation {
            facts: Vec::new(),
            metadata: metadata.clone(),
            schema_version: "3.2.0".to_string(),
        };

        let ir_v33 = crate::IntermediateRepresentation {
            facts: Vec::new(),
            metadata,
            schema_version: "3.3.0".to_string(),
        };

        assert!(needs_migration(&ir_v32));
        assert!(!needs_migration(&ir_v33));
    }

    #[test]
    fn test_parse_schema_version() {
        assert_eq!(SchemaVersion::parse("3.2.0"), SchemaVersion::V32);
        assert_eq!(SchemaVersion::parse("3.2"), SchemaVersion::V32);
        assert_eq!(SchemaVersion::parse("3.3.0"), SchemaVersion::V33);
        assert_eq!(SchemaVersion::parse("3.3"), SchemaVersion::V33);
        assert_eq!(
            SchemaVersion::parse("4.0.0"),
            SchemaVersion::Unknown("4.0.0".to_string())
        );
    }

    #[test]
    fn test_schema_version_to_string() {
        assert_eq!(SchemaVersion::V32.to_string(), "3.2.0");
        assert_eq!(SchemaVersion::V33.to_string(), "3.3.0");
        assert_eq!(
            SchemaVersion::Unknown("4.0.0".to_string()).to_string(),
            "4.0.0"
        );
    }

    #[test]
    fn test_migrate_unknown_version() {
        let metadata = ProjectMetadata::new(
            "test-project".to_string(),
            "1.0.0".to_string(),
            ProjectPath::new(std::path::PathBuf::from("/test")),
        );

        let ir_unknown = crate::IntermediateRepresentation {
            facts: Vec::new(),
            metadata,
            schema_version: "4.0.0".to_string(),
        };

        match migrate_ir_version(&ir_unknown) {
            Err(MigrationError::UnsupportedVersion(v)) => {
                assert_eq!(v, "4.0.0");
            }
            _ => panic!("Expected UnsupportedVersion error"),
        }
    }
}
