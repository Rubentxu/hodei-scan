//! Test file for hodei-ir with minimal dependencies

#[cfg(test)]
mod hodei_ir_tests {
    use std::path::PathBuf;

    #[test]
    fn test_fact_type_creation() {
        // Basic test that the module compiles and types are accessible
        use hodei_ir::{FactType, FactTypeDiscriminant};

        let _discriminant = FactTypeDiscriminant::TaintSource;
        assert!(true);
    }

    #[test]
    fn test_fact_creation() {
        use hodei_ir::{
            ColumnNumber, Confidence, ExtractorId, Fact, FactId, FactType, LineNumber, ProjectPath,
            Provenance, SourceLocation,
        };

        // Create a minimal fact to verify compilation
        let location = SourceLocation::new(
            ProjectPath::new(PathBuf::from("test.rs")).unwrap(),
            LineNumber::new(1).unwrap(),
            Some(ColumnNumber::new(1).unwrap()),
            LineNumber::new(1).unwrap(),
            Some(ColumnNumber::new(10).unwrap()),
        );

        let provenance =
            Provenance::new(ExtractorId::TreeSitter, "1.0".to_string(), Confidence::HIGH);

        let fact_type = FactType::Function {
            name: hodei_ir::FunctionName("test".to_string()),
            complexity: 1,
            lines_of_code: 10,
        };

        let _fact =
            Fact::new_with_message(fact_type, "Test function".to_string(), location, provenance);
        assert!(true);
    }

    #[test]
    fn test_ir_creation() {
        use hodei_ir::{IntermediateRepresentation, ProjectMetadata, ProjectPath};
        use std::path::PathBuf;

        let metadata = ProjectMetadata::new(
            "test-project".to_string(),
            "1.0.0".to_string(),
            ProjectPath::new(PathBuf::from("/tmp/test")).unwrap(),
        );

        let _ir = IntermediateRepresentation::new(metadata);
        assert!(true);
    }

    #[test]
    fn test_confidence_levels() {
        use hodei_ir::Confidence;

        let high = Confidence::HIGH;
        let medium = Confidence::MEDIUM;
        let low = Confidence::LOW;

        assert_eq!(high.get(), 0.9);
        assert_eq!(medium.get(), 0.6);
        assert_eq!(low.get(), 0.3);
    }
}
