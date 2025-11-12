use crate::core::{Extractor, ExtractorConfig};
use crate::sarif::{SarifConfig, SarifExtractor};
use hodei_ir::{FactType, Severity};
use std::path::PathBuf;

#[cfg(test)]
mod sarif_tests {
    use super::*;

    fn fixture_path(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("sarif")
            .join(name)
    }

    /// Test CA-2.1: Parse valid SARIF 2.1.0 files
    #[tokio::test]
    async fn test_parse_codeql_sarif() {
        let config = SarifConfig {
            sarif_files: vec![
                fixture_path("github-codeql.sarif")
                    .to_string_lossy()
                    .to_string(),
            ],
            exclude_rules: vec![],
            min_severity: None,
            severity_mapping: std::collections::HashMap::new(),
        };

        let extractor = SarifExtractor::new(config);
        let extractor_config = ExtractorConfig {
            project_path: PathBuf::from("/tmp"),
            config: serde_json::json!({}),
            file_filters: Default::default(),
        };

        let ir = extractor.extract(extractor_config).await.unwrap();

        assert_eq!(
            ir.facts.len(),
            2,
            "Should extract 2 facts from CodeQL SARIF"
        );

        // Check first fact (SQL injection - error level)
        let fact = &ir.facts[0];
        assert!(matches!(fact.fact_type, FactType::Vulnerability { .. }));
        assert_eq!(
            fact.message,
            "This use of \"query += ...\" can cause SQL injection because the concatenated string is later passed to a database execution function."
        );
    }

    /// Test CA-2.2: Map SARIF fields correctly to IR
    #[tokio::test]
    async fn test_map_eslint_fields() {
        let config = SarifConfig {
            sarif_files: vec![fixture_path("eslint.sarif").to_string_lossy().to_string()],
            exclude_rules: vec![],
            min_severity: None,
            severity_mapping: std::collections::HashMap::new(),
        };

        let extractor = SarifExtractor::new(config);
        let extractor_config = ExtractorConfig {
            project_path: PathBuf::from("/tmp"),
            config: serde_json::json!({}),
            file_filters: Default::default(),
        };

        let ir = extractor.extract(extractor_config).await.unwrap();

        assert_eq!(
            ir.facts.len(),
            2,
            "Should extract 2 facts from ESLint SARIF"
        );

        // Check severity mapping: error -> Critical, warning -> Major
        if let FactType::CodeSmell { severity, .. } = ir.facts[0].fact_type {
            assert_eq!(severity, Severity::Major); // warning
        }
        if let FactType::Vulnerability { severity, .. } = ir.facts[1].fact_type {
            assert_eq!(severity, Severity::Critical); // error
        }

        // Check location mapping
        let fact = &ir.facts[0];
        assert!(fact.location.file.path.exists());
        assert_eq!(fact.location.start_line.get(), 12);
        assert_eq!(fact.location.start_column.unwrap().get(), 2);
    }

    /// Test CA-2.3: Support multiple runs in a single SARIF file
    #[tokio::test]
    async fn test_semgrep_multiple_rules() {
        let config = SarifConfig {
            sarif_files: vec![fixture_path("semgrep.sarif").to_string_lossy().to_string()],
            exclude_rules: vec![],
            min_severity: None,
            severity_mapping: std::collections::HashMap::new(),
        };

        let extractor = SarifExtractor::new(config);
        let extractor_config = ExtractorConfig {
            project_path: PathBuf::from("/tmp"),
            config: serde_json::json!({}),
            file_filters: Default::default(),
        };

        let ir = extractor.extract(extractor_config).await.unwrap();

        assert_eq!(
            ir.facts.len(),
            2,
            "Should extract 2 facts from Semgrep SARIF"
        );

        // Check that both security and non-security issues are captured
        assert!(matches!(
            ir.facts[0].fact_type,
            FactType::Vulnerability { .. }
        ));
        assert!(matches!(
            ir.facts[1].fact_type,
            FactType::Vulnerability { .. }
        ));

        // Verify provenance metadata
        for fact in &ir.facts {
            assert_eq!(
                fact.provenance.extractor,
                hodei_ir::ExtractorId::SarifAdapter
            );
            assert!(!fact.provenance.version.is_empty());
        }
    }

    /// Test CA-2.4: Extract and normalize security severity
    #[tokio::test]
    async fn test_security_severity_normalization() {
        let config = SarifConfig {
            sarif_files: vec![
                fixture_path("github-codeql.sarif")
                    .to_string_lossy()
                    .to_string(),
            ],
            exclude_rules: vec![],
            min_severity: None,
            severity_mapping: std::collections::HashMap::new(),
        };

        let extractor = SarifExtractor::new(config);
        let extractor_config = ExtractorConfig {
            project_path: PathBuf::from("/tmp"),
            config: serde_json::json!({}),
            file_filters: Default::default(),
        };

        let ir = extractor.extract(extractor_config).await.unwrap();

        // First fact has security-severity: 8.1 -> should normalize to ~0.81
        if let FactType::Vulnerability { ref confidence, .. } = ir.facts[0].fact_type {
            assert!(
                confidence.get() > 0.7,
                "Security severity should be normalized"
            );
            assert!(
                confidence.get() < 0.9,
                "Security severity should be normalized"
            );
        } else {
            panic!("Expected Vulnerability fact type");
        }

        // Second fact has security-severity: 7.2 -> should normalize to ~0.72
        if let FactType::Vulnerability { ref confidence, .. } = ir.facts[1].fact_type {
            assert!(
                confidence.get() > 0.6,
                "Security severity should be normalized"
            );
            assert!(
                confidence.get() < 0.8,
                "Security severity should be normalized"
            );
        } else {
            panic!("Expected Vulnerability fact type");
        }
    }

    /// Test CA-2.4: Extract CWE IDs
    #[tokio::test]
    async fn test_extract_cwe_ids() {
        let config = SarifConfig {
            sarif_files: vec![
                fixture_path("github-codeql.sarif")
                    .to_string_lossy()
                    .to_string(),
            ],
            exclude_rules: vec![],
            min_severity: None,
            severity_mapping: std::collections::HashMap::new(),
        };

        let extractor = SarifExtractor::new(config);
        let extractor_config = ExtractorConfig {
            project_path: PathBuf::from("/tmp"),
            config: serde_json::json!({}),
            file_filters: Default::default(),
        };

        let ir = extractor.extract(extractor_config).await.unwrap();

        // Both facts should have CWE-89
        for fact in &ir.facts {
            if let FactType::Vulnerability { ref cwe_id, .. } = fact.fact_type {
                assert!(cwe_id.is_some(), "Should have CWE ID");
                assert_eq!(cwe_id.as_ref().unwrap(), "CWE-89");
            }
        }
    }

    /// Test CA-2.5: Handle optional fields gracefully
    #[tokio::test]
    async fn test_handle_missing_optional_fields() {
        let config = SarifConfig {
            sarif_files: vec![fixture_path("eslint.sarif").to_string_lossy().to_string()],
            exclude_rules: vec![],
            min_severity: None,
            severity_mapping: std::collections::HashMap::new(),
        };

        let extractor = SarifExtractor::new(config);
        let extractor_config = ExtractorConfig {
            project_path: PathBuf::from("/tmp"),
            config: serde_json::json!({}),
            file_filters: Default::default(),
        };

        // Should not panic even if some properties are missing
        let ir = extractor.extract(extractor_config).await.unwrap();
        assert_eq!(
            ir.facts.len(),
            2,
            "Should handle missing optional fields gracefully"
        );
    }

    /// Test CA-2.6: Severity filtering with min_severity
    #[tokio::test]
    async fn test_severity_filtering() {
        let config = SarifConfig {
            sarif_files: vec![
                fixture_path("github-codeql.sarif")
                    .to_string_lossy()
                    .to_string(),
            ],
            exclude_rules: vec![],
            min_severity: Some("error".to_string()), // Only include error level
            severity_mapping: std::collections::HashMap::new(),
        };

        let extractor = SarifExtractor::new(config);
        let extractor_config = ExtractorConfig {
            project_path: PathBuf::from("/tmp"),
            config: serde_json::json!({}),
            file_filters: Default::default(),
        };

        let ir = extractor.extract(extractor_config).await.unwrap();

        // Should only include the error-level fact, not the warning
        assert_eq!(ir.facts.len(), 1, "Should filter by minimum severity");
        assert_eq!(ir.facts[0].location.start_line.get(), 42);
    }

    /// Test rule exclusion filtering
    #[tokio::test]
    async fn test_exclude_rules_filtering() {
        let config = SarifConfig {
            sarif_files: vec![fixture_path("eslint.sarif").to_string_lossy().to_string()],
            exclude_rules: vec!["no-console".to_string()], // Exclude console rule
            min_severity: None,
            severity_mapping: std::collections::HashMap::new(),
        };

        let extractor = SarifExtractor::new(config);
        let extractor_config = ExtractorConfig {
            project_path: PathBuf::from("/tmp"),
            config: serde_json::json!({}),
            file_filters: Default::default(),
        };

        let ir = extractor.extract(extractor_config).await.unwrap();

        // Should only include the no-eval fact (no-console was excluded)
        assert_eq!(ir.facts.len(), 1, "Should exclude rules by pattern");
        if let FactType::Vulnerability { severity, .. } = ir.facts[0].fact_type {
            assert_eq!(severity, Severity::Critical);
        }
    }

    /// Test OWASP category extraction
    #[tokio::test]
    async fn test_extract_owasp_category() {
        let config = SarifConfig {
            sarif_files: vec![fixture_path("semgrep.sarif").to_string_lossy().to_string()],
            exclude_rules: vec![],
            min_severity: None,
            severity_mapping: std::collections::HashMap::new(),
        };

        let extractor = SarifExtractor::new(config);
        let extractor_config = ExtractorConfig {
            project_path: PathBuf::from("/tmp"),
            config: serde_json::json!({}),
            file_filters: Default::default(),
        };

        let ir = extractor.extract(extractor_config).await.unwrap();

        // Check OWASP category extraction
        for fact in &ir.facts {
            if let FactType::Vulnerability {
                ref owasp_category, ..
            } = fact.fact_type
            {
                assert!(owasp_category.is_some(), "Should have OWASP category");
            }
        }
    }

    /// Test multiple SARIF files with glob pattern
    #[tokio::test]
    async fn test_multiple_files_discovery() {
        let temp_dir = std::env::temp_dir();
        let test_dir = temp_dir.join("sarif_test");
        std::fs::create_dir_all(&test_dir).unwrap();

        // Copy test files to temp directory
        std::fs::copy(
            fixture_path("github-codeql.sarif"),
            test_dir.join("codeql.sarif"),
        )
        .unwrap();
        std::fs::copy(fixture_path("eslint.sarif"), test_dir.join("eslint.sarif")).unwrap();

        let config = SarifConfig {
            sarif_files: vec![format!("{}/**/*.sarif", test_dir.display())],
            exclude_rules: vec![],
            min_severity: None,
            severity_mapping: std::collections::HashMap::new(),
        };

        let extractor = SarifExtractor::new(config);
        let extractor_config = ExtractorConfig {
            project_path: test_dir.clone(),
            config: serde_json::json!({}),
            file_filters: Default::default(),
        };

        let ir = extractor.extract(extractor_config).await.unwrap();

        // Should extract from both files
        assert_eq!(ir.facts.len(), 4, "Should process multiple SARIF files");

        // Cleanup
        std::fs::remove_dir_all(&test_dir).unwrap();
    }

    /// Test metadata extraction
    #[tokio::test]
    async fn test_metadata_extraction() {
        let extractor = SarifExtractor::new(SarifConfig::default());
        let metadata = extractor.metadata();

        assert_eq!(metadata.id, "sarif-universal");
        assert_eq!(metadata.name, "Universal SARIF Extractor");
        assert!(metadata.supported_extensions.contains(&"sarif".to_string()));
        assert!(metadata.supported_extensions.contains(&"json".to_string()));
        assert!(metadata.languages.contains(&"multi-language".to_string()));
    }

    /// Test default configuration
    #[test]
    fn test_default_config() {
        let config = SarifConfig::default();
        assert_eq!(config.sarif_files.len(), 2);
        assert!(
            config
                .sarif_files
                .contains(&"results/**/*.sarif".to_string())
        );
        assert!(
            config
                .sarif_files
                .contains(&".sarif/**/*.sarif".to_string())
        );
        assert!(config.exclude_rules.is_empty());
        assert!(config.min_severity.is_none());
    }
}
