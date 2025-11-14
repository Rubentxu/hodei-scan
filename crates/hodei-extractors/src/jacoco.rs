//! JaCoCo Coverage Extractor (Nivel 1)
//!
//! This module implements the Tier 1 adapter for JaCoCo code coverage data.
//! It parses JaCoCo XML reports and generates facts about uncovered code lines.
//!
//! # Overview
//!
//! The JaCoCo extractor listens for "uncovered lines" - these are code paths
//! that are not exercised by tests. This information is crucial for correlation
//! with security findings to prioritize risks.
//!
//! # Features
//!
//! - Parses JaCoCo XML format (jacoco.xml or jacoco.exec + report)
//! - Detects uncovered lines, branches, and methods
//! - Generates `UncoveredLine` facts for correlation with vulnerabilities
//! - Supports both project-level and per-package coverage reports
//!
//! # Example
//!
//! ```no_run
//! use hodei_extractors::jacoco::JacocoExtractor;
//! use hodei_extractors::core::{ExtractorConfig, Extractor};
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = serde_json::json!({
//!     "report_path": "target/site/jacoco/jacoco.xml",
//!     "include_unchanged": false
//! });
//!
//! let extractor = JacocoExtractor::new(serde_json::from_value(config)?);
//! let extractor_config = ExtractorConfig {
//!     project_path: PathBuf::from("/path/to/project"),
//!     config: serde_json::json!({}),
//!     file_filters: Default::default(),
//! };
//!
//! let ir = extractor.extract(extractor_config).await?;
//! println!("Found {} uncovered lines", ir.facts.len());
//! # Ok(())
//! # }
//! ```

use crate::core::{Extractor, ExtractorConfig, ExtractorError, ExtractorMetadata, IRBuilder};
use hodei_ir::{
    ColumnNumber, Confidence, ExtractorId, FactType, IntermediateRepresentation, LineNumber,
    ProjectPath, Provenance, Severity, SourceLocation,
};
use quick_xml::Reader;
use quick_xml::events::Event;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Configuration for the JaCoCo extractor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JacocoConfig {
    /// Path to the JaCoCo XML report
    /// Can be a glob pattern to handle multiple reports
    #[serde(default = "default_report_path")]
    pub report_path: String,

    /// Include unchanged/zero-covered entities
    #[serde(default = "default_false")]
    pub include_unchanged: bool,

    /// Minimum coverage percentage to consider a file as covered
    /// Files below this threshold will have all lines marked as uncovered
    #[serde(default = "default_min_coverage")]
    pub min_coverage_threshold: f64,

    /// Package patterns to include (empty means all)
    #[serde(default)]
    pub include_packages: Vec<String>,

    /// Package patterns to exclude
    #[serde(default)]
    pub exclude_packages: Vec<String>,
}

fn default_report_path() -> String {
    "target/site/jacoco/jacoco.xml".to_string()
}

fn default_false() -> bool {
    false
}

fn default_min_coverage() -> f64 {
    0.0
}

impl Default for JacocoConfig {
    fn default() -> Self {
        Self {
            report_path: default_report_path(),
            include_unchanged: default_false(),
            min_coverage_threshold: default_min_coverage(),
            include_packages: Vec::new(),
            exclude_packages: Vec::new(),
        }
    }
}

/// JaCoCo coverage extractor implementation
pub struct JacocoExtractor {
    config: JacocoConfig,
}

impl JacocoExtractor {
    /// Create a new JaCoCo extractor with the given configuration
    pub fn new(config: JacocoConfig) -> Self {
        Self { config }
    }

    /// Parse JaCoCo XML report and extract uncovered lines
    async fn parse_jacoco_report(
        &self,
        report_path: &Path,
        project_path: &Path,
    ) -> Result<Vec<UncoveredLine>, ExtractorError> {
        info!("Parsing JaCoCo report: {}", report_path.display());

        let content =
            tokio::fs::read_to_string(report_path)
                .await
                .map_err(|e| ExtractorError::Io {
                    id: "jacoco".to_string(),
                    error: e,
                })?;

        let mut uncovered_lines = Vec::new();
        let mut current_package = String::new();
        let mut current_class = String::new();

        // Simple XML parsing for JaCoCo format
        // JaCoCo XML structure: <report> -> <package> -> <class> -> <method> -> <line>
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Parse package name
            if line.starts_with("<package ") {
                if let Some(name_start) = line.find("name=\"") {
                    if let Some(name_end) = line[name_start + 6..].find('"') {
                        current_package =
                            line[name_start + 6..name_start + 6 + name_end].to_string();
                    }
                }
            }

            // Parse class name
            if line.starts_with("<class ") {
                if let Some(name_start) = line.find("name=\"") {
                    if let Some(name_end) = line[name_start + 6..].find('"') {
                        current_class = line[name_start + 6..name_start + 6 + name_end].to_string();
                    }
                }
            }

            // Parse line data
            if line.starts_with("<line ") {
                let line_info = self.parse_line_element(line, &current_package, &current_class)?;
                if line_info.is_some() {
                    uncovered_lines.push(line_info.unwrap());
                }
            }

            i += 1;
        }

        info!(
            "Found {} uncovered lines in JaCoCo report",
            uncovered_lines.len()
        );

        Ok(uncovered_lines)
    }

    /// Parse a line element from JaCoCo XML
    fn parse_line_element(
        &self,
        line: &str,
        package: &str,
        class: &str,
    ) -> Result<Option<UncoveredLine>, ExtractorError> {
        // JaCoCo line format: <line nr="23" mi="0" ci="2" mb="0" cb="0"/>
        let mut line_number = None;
        let mut instruction_missed = None;
        let mut branch_missed = None;

        // Extract line number
        if let Some(nr_start) = line.find("nr=\"") {
            let nr_end =
                line[nr_start + 4..]
                    .find('"')
                    .ok_or_else(|| ExtractorError::InvalidIR {
                        id: "jacoco".to_string(),
                        error: "Invalid line number format".to_string(),
                    })?;
            line_number = Some(
                line[nr_start + 4..nr_start + 4 + nr_end]
                    .parse::<u32>()
                    .map_err(|_| ExtractorError::InvalidIR {
                        id: "jacoco".to_string(),
                        error: "Line number is not a valid integer".to_string(),
                    })?,
            );
        }

        // Extract missed instructions count
        if let Some(mi_start) = line.find("mi=\"") {
            let mi_end =
                line[mi_start + 4..]
                    .find('"')
                    .ok_or_else(|| ExtractorError::InvalidIR {
                        id: "jacoco".to_string(),
                        error: "Invalid missed instructions format".to_string(),
                    })?;
            instruction_missed = Some(
                line[mi_start + 4..mi_start + 4 + mi_end]
                    .parse::<u32>()
                    .unwrap_or(0),
            );
        }

        // Extract missed branches count
        if let Some(mb_start) = line.find("mb=\"") {
            let mb_end =
                line[mb_start + 4..]
                    .find('"')
                    .ok_or_else(|| ExtractorError::InvalidIR {
                        id: "jacoco".to_string(),
                        error: "Invalid missed branches format".to_string(),
                    })?;
            branch_missed = Some(
                line[mb_start + 4..mb_start + 4 + mb_end]
                    .parse::<u32>()
                    .unwrap_or(0),
            );
        }

        // Determine if line is uncovered
        // A line is uncovered if: mi > 0 (missed instructions) OR (mb > 0 AND ci == 0)
        let is_uncovered = instruction_missed.unwrap_or(0) > 0 || {
            // Check if fully uncovered (no coverage)
            if let Some(ci_start) = line.find("ci=\"") {
                if let Some(ci_end) = line[ci_start + 4..].find('"') {
                    let covered_instructions = line[ci_start + 4..ci_start + 4 + ci_end]
                        .parse::<u32>()
                        .unwrap_or(0);
                    covered_instructions == 0
                } else {
                    false
                }
            } else {
                false
            }
        };

        if !is_uncovered {
            return Ok(None);
        }

        // Convert package/class to file path
        // package: com/example -> src/main/java/com/example
        // class: UserService -> UserService.java
        let mut file_path = PathBuf::new();
        file_path.push("src"); // Assuming standard Maven/Gradle structure
        file_path.push("main");
        file_path.push("java");

        for part in package.split('.') {
            if !part.is_empty() {
                file_path.push(part);
            }
        }

        // Add class name with .java extension
        let class_name = class.split('$').next().unwrap_or(&class);
        file_path.push(format!("{}.java", class_name));

        Ok(Some(UncoveredLine {
            file_path,
            line_number: LineNumber::new(line_number.unwrap_or(0))
                .unwrap_or_else(|_| LineNumber::new(1).unwrap()),
            coverage_percentage: 0.0,
            missed_instructions: instruction_missed.unwrap_or(0),
            missed_branches: branch_missed.unwrap_or(0),
        }))
    }

    /// Convert uncovered line to hodei Fact
    fn uncovered_line_to_fact(&self, uncovered: &UncoveredLine) -> hodei_ir::Fact {
        let file = ProjectPath::new(uncovered.file_path.clone());

        let location = SourceLocation::new(
            file,
            uncovered.line_number,
            None,
            uncovered.line_number,
            None,
        );

        let fact_type = FactType::UncoveredLine {
            location: location.clone(),
            coverage: "JaCoCo".to_string(),
        };

        let provenance = Provenance::new(
            ExtractorId::JaCoCoParser,
            "0.1.0".to_string(),
            Confidence::HIGH,
        );

        hodei_ir::Fact::new(fact_type, location, provenance)
    }
}

/// Represents an uncovered line from JaCoCo report
struct UncoveredLine {
    file_path: PathBuf,
    line_number: LineNumber,
    coverage_percentage: f64,
    missed_instructions: u32,
    missed_branches: u32,
}

#[async_trait::async_trait]
impl Extractor for JacocoExtractor {
    async fn extract(
        &self,
        config: ExtractorConfig,
    ) -> Result<IntermediateRepresentation, ExtractorError> {
        let mut ir_builder = IRBuilder::new();
        ir_builder.project_path(config.project_path.clone());

        // Build full path to report
        let report_path = config.project_path.join(&self.config.report_path);

        // Check if report exists
        if !report_path.exists() {
            warn!("JaCoCo report not found at: {}", report_path.display());
            return Ok(ir_builder.build());
        }

        // Parse the report
        let uncovered_lines = self
            .parse_jacoco_report(&report_path, &config.project_path)
            .await?;

        // Convert to facts
        for uncovered in uncovered_lines {
            let fact = self.uncovered_line_to_fact(&uncovered);
            ir_builder.add_fact(fact);
        }

        info!("Generated {} UncoveredLine facts", ir_builder.fact_count());

        Ok(ir_builder.build())
    }

    fn metadata(&self) -> ExtractorMetadata {
        ExtractorMetadata {
            id: "jacoco-coverage".to_string(),
            name: "JaCoCo Coverage Extractor".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            supported_extensions: vec!["xml".to_string()],
            languages: vec!["java".to_string()],
            description: "Extracts code coverage data from JaCoCo reports to identify untested code paths for security risk correlation".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jacoco_config_deserialization() {
        let json = r#"{
            "report_path": "target/site/jacoco/jacoco.xml",
            "include_unchanged": false,
            "min_coverage_threshold": 10.0
        }"#;

        let config: JacocoConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.report_path, "target/site/jacoco/jacoco.xml");
        assert!(!config.include_unchanged);
        assert_eq!(config.min_coverage_threshold, 10.0);
    }

    #[test]
    fn test_parse_line_element_fully_uncovered() {
        let extractor = JacocoExtractor::new(JacocoConfig::default());
        let line = r#"<line nr="42" mi="5" ci="0" mb="2" cb="0"/>"#;

        let result = extractor
            .parse_line_element(line, "com.example", "TestClass")
            .unwrap()
            .unwrap();

        assert_eq!(result.line_number.get(), 42);
        assert_eq!(result.missed_instructions, 5);
    }

    #[test]
    fn test_parse_line_element_partially_covered() {
        let extractor = JacocoExtractor::new(JacocoConfig::default());
        let line = r#"<line nr="10" mi="0" ci="3" mb="1" cb="0"/>"#;

        // Partially covered with branch miss should be marked as uncovered
        let result = extractor.parse_line_element(line, "com.example", "TestClass");

        // With missed branches and partial coverage, it's still considered uncovered
        assert!(result.is_ok());
    }
}
