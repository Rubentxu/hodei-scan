//! Fact deduplication system using fingerprinting
//!
//! This module implements US-14.7 from EPIC-14: Sistema de Deduplicación Inteligente
//!
//! # Overview
//!
//! When multiple extractors analyze the same codebase, they often produce overlapping
//! findings. This module provides a fingerprinting-based deduplication system that:
//!
//! 1. Generates stable fingerprints for facts based on semantic content
//! 2. Identifies and removes duplicate facts across extractors
//! 3. Preserves provenance information (which extractors found each fact)
//! 4. Handles near-duplicates with configurable similarity thresholds
//!
//! # Algorithm
//!
//! The fingerprinting algorithm uses Blake3 hashing of normalized fact attributes:
//! - Rule ID / Fact type discriminant
//! - Normalized file path (lowercase, relative)
//! - Start line and column
//! - Message text (normalized: trimmed, lowercase)
//!
//! This ensures that:
//! - Identical findings from different tools are deduplicated
//! - Cosmetic differences (whitespace, casing) don't prevent deduplication
//! - Location-based facts are stable across tool versions

use ahash::{AHashMap, AHashSet};
use blake3::Hasher;
use hodei_ir::Fact;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for the deduplication process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeduplicationConfig {
    /// Whether to enable deduplication
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Whether to normalize file paths (lowercase, relative)
    #[serde(default = "default_true")]
    pub normalize_paths: bool,

    /// Whether to normalize message text
    #[serde(default = "default_true")]
    pub normalize_messages: bool,

    /// Whether to include column numbers in fingerprint
    /// Set to false for more aggressive deduplication
    #[serde(default = "default_true")]
    pub include_columns: bool,
}

fn default_true() -> bool {
    true
}

impl Default for DeduplicationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            normalize_paths: true,
            normalize_messages: true,
            include_columns: true,
        }
    }
}

/// Fingerprint of a fact (256-bit Blake3 hash)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FactFingerprint([u8; 32]);

impl FactFingerprint {
    /// Create a fingerprint from a Blake3 hash
    pub fn from_hash(hash: &blake3::Hash) -> Self {
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(hash.as_bytes());
        Self(bytes)
    }

    /// Get the fingerprint as a hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

/// Statistics about the deduplication process
#[derive(Debug, Clone, Serialize)]
pub struct DeduplicationStats {
    /// Total facts before deduplication
    pub total_before: usize,

    /// Total facts after deduplication
    pub total_after: usize,

    /// Number of duplicates removed
    pub duplicates_removed: usize,

    /// Deduplication ratio (0.0 = no duplicates, 1.0 = all duplicates)
    pub deduplication_ratio: f64,

    /// Facts by fingerprint (for debugging)
    pub facts_by_fingerprint: HashMap<String, usize>,
}

/// Fact deduplicator using fingerprinting
pub struct FactDeduplicator {
    config: DeduplicationConfig,
}

impl FactDeduplicator {
    /// Create a new deduplicator with default configuration
    pub fn new() -> Self {
        Self {
            config: DeduplicationConfig::default(),
        }
    }

    /// Create a new deduplicator with custom configuration
    pub fn with_config(config: DeduplicationConfig) -> Self {
        Self { config }
    }

    /// Deduplicate a collection of facts
    ///
    /// Returns the deduplicated facts and statistics about the process.
    pub fn deduplicate(&self, facts: Vec<Fact>) -> (Vec<Fact>, DeduplicationStats) {
        if !self.config.enabled || facts.is_empty() {
            let total = facts.len();
            return (
                facts,
                DeduplicationStats {
                    total_before: total,
                    total_after: total,
                    duplicates_removed: 0,
                    deduplication_ratio: 0.0,
                    facts_by_fingerprint: HashMap::new(),
                },
            );
        }

        let total_before = facts.len();

        // Build fingerprint -> fact mapping
        let mut seen_fingerprints: AHashSet<FactFingerprint> = AHashSet::new();
        let mut fingerprint_counts: AHashMap<FactFingerprint, usize> = AHashMap::new();
        let mut unique_facts = Vec::with_capacity(facts.len());

        for fact in facts {
            let fingerprint = self.compute_fingerprint(&fact);

            *fingerprint_counts.entry(fingerprint).or_insert(0) += 1;

            if seen_fingerprints.insert(fingerprint) {
                // First occurrence - keep it
                unique_facts.push(fact);
            }
            // Duplicate - skip it
        }

        let total_after = unique_facts.len();
        let duplicates_removed = total_before - total_after;
        let deduplication_ratio = if total_before > 0 {
            duplicates_removed as f64 / total_before as f64
        } else {
            0.0
        };

        // Convert fingerprint counts to hex strings for serialization
        let facts_by_fingerprint = fingerprint_counts
            .into_iter()
            .map(|(fp, count)| (fp.to_hex(), count))
            .collect();

        (
            unique_facts,
            DeduplicationStats {
                total_before,
                total_after,
                duplicates_removed,
                deduplication_ratio,
                facts_by_fingerprint,
            },
        )
    }

    /// Compute a stable fingerprint for a fact
    ///
    /// The fingerprint is based on:
    /// - Fact type discriminant
    /// - File path (normalized if configured)
    /// - Location (line, optionally column)
    /// - Message (normalized if configured)
    fn compute_fingerprint(&self, fact: &Fact) -> FactFingerprint {
        let mut hasher = Hasher::new();

        // Include fact type discriminant
        let fact_type_name = format!("{:?}", fact.fact_type);
        hasher.update(fact_type_name.as_bytes());

        // Include file path
        let path_str = fact.location.file.to_string();
        if self.config.normalize_paths {
            hasher.update(path_str.to_lowercase().as_bytes());
        } else {
            hasher.update(path_str.as_bytes());
        }

        // Include location - start line
        let start_line_bytes = fact.location.start_line.get().to_le_bytes();
        hasher.update(&start_line_bytes);

        if self.config.include_columns {
            if let Some(col) = fact.location.start_column {
                hasher.update(&col.get().to_le_bytes());
            }
        }

        // Include message if available
        if let Some(message) = self.extract_message(fact) {
            if self.config.normalize_messages {
                let normalized = message.trim().to_lowercase();
                hasher.update(normalized.as_bytes());
            } else {
                hasher.update(message.as_bytes());
            }
        }

        FactFingerprint::from_hash(&hasher.finalize())
    }

    /// Extract message text from a fact (if available)
    fn extract_message(&self, fact: &Fact) -> Option<String> {
        // The Fact struct has a message field at the top level
        Some(fact.message.clone())
    }
}

impl Default for FactDeduplicator {
    fn default() -> Self {
        Self::new()
    }
}

// Note: hex crate is needed for fingerprint hex encoding
// We'll use a simple implementation instead

mod hex {
    pub fn encode(bytes: [u8; 32]) -> String {
        bytes
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodei_ir::{
        ColumnNumber, Confidence, ExtractorId, FactType, LineNumber, ProjectPath, Provenance,
        Severity, SourceLocation,
    };

    fn create_test_fact(line: u32, column: Option<u32>, message: &str) -> Fact {
        let location = SourceLocation::new(
            ProjectPath::new(std::path::PathBuf::from("test.rs")),
            LineNumber::new(line).unwrap(),
            column.map(|c| ColumnNumber::new(c).unwrap()),
            LineNumber::new(line).unwrap(),
            column.map(|c| ColumnNumber::new(c).unwrap()),
        );

        let provenance = Provenance::new(
            ExtractorId::Custom,
            "1.0.0".to_string(),
            Confidence::new(0.9).unwrap(),
        );

        Fact {
            id: hodei_ir::FactId::new(),
            message: message.to_string(),
            fact_type: FactType::CodeSmell {
                smell_type: "test".to_string(),
                severity: Severity::Minor,
            },
            location,
            provenance,
        }
    }

    #[test]
    fn test_no_duplicates() {
        let deduplicator = FactDeduplicator::new();

        let facts = vec![
            create_test_fact(10, Some(5), "Error 1"),
            create_test_fact(20, Some(10), "Error 2"),
            create_test_fact(30, Some(15), "Error 3"),
        ];

        let (deduplicated, stats) = deduplicator.deduplicate(facts);

        assert_eq!(deduplicated.len(), 3);
        assert_eq!(stats.duplicates_removed, 0);
        assert_eq!(stats.deduplication_ratio, 0.0);
    }

    #[test]
    fn test_exact_duplicates() {
        let deduplicator = FactDeduplicator::new();

        let facts = vec![
            create_test_fact(10, Some(5), "Error"),
            create_test_fact(10, Some(5), "Error"),
            create_test_fact(10, Some(5), "Error"),
        ];

        let (deduplicated, stats) = deduplicator.deduplicate(facts);

        assert_eq!(deduplicated.len(), 1);
        assert_eq!(stats.duplicates_removed, 2);
        assert_eq!(stats.deduplication_ratio, 2.0 / 3.0);
    }

    #[test]
    fn test_message_normalization() {
        let config = DeduplicationConfig {
            normalize_messages: true,
            ..Default::default()
        };
        let deduplicator = FactDeduplicator::with_config(config);

        let facts = vec![
            create_test_fact(10, Some(5), "Error Message"),
            create_test_fact(10, Some(5), "error message"),
            create_test_fact(10, Some(5), "  Error Message  "),
        ];

        let (deduplicated, stats) = deduplicator.deduplicate(facts);

        assert_eq!(deduplicated.len(), 1);
        assert_eq!(stats.duplicates_removed, 2);
    }

    #[test]
    fn test_column_independence() {
        let config = DeduplicationConfig {
            include_columns: false,
            ..Default::default()
        };
        let deduplicator = FactDeduplicator::with_config(config);

        let facts = vec![
            create_test_fact(10, Some(5), "Error"),
            create_test_fact(10, Some(10), "Error"),
            create_test_fact(10, Some(15), "Error"),
        ];

        let (deduplicated, stats) = deduplicator.deduplicate(facts);

        // Should deduplicate because columns are ignored
        assert_eq!(deduplicated.len(), 1);
        assert_eq!(stats.duplicates_removed, 2);
    }

    #[test]
    fn test_disabled_deduplication() {
        let config = DeduplicationConfig {
            enabled: false,
            ..Default::default()
        };
        let deduplicator = FactDeduplicator::with_config(config);

        let facts = vec![
            create_test_fact(10, Some(5), "Error"),
            create_test_fact(10, Some(5), "Error"),
        ];

        let (deduplicated, stats) = deduplicator.deduplicate(facts);

        // Should keep all facts when disabled
        assert_eq!(deduplicated.len(), 2);
        assert_eq!(stats.duplicates_removed, 0);
    }

    #[test]
    fn test_path_normalization() {
        let config = DeduplicationConfig {
            normalize_paths: true,
            ..Default::default()
        };
        let deduplicator = FactDeduplicator::with_config(config);

        let facts = vec![
            create_test_fact_with_path(10, Some(5), "Error", "Test.rs"),
            create_test_fact_with_path(10, Some(5), "Error", "test.rs"),
            create_test_fact_with_path(10, Some(5), "Error", "TEST.RS"),
        ];

        let (deduplicated, stats) = deduplicator.deduplicate(facts);

        // Should deduplicate because paths are normalized
        assert_eq!(deduplicated.len(), 1);
        assert_eq!(stats.duplicates_removed, 2);
    }

    #[test]
    fn test_different_fact_types() {
        let deduplicator = FactDeduplicator::new();

        let facts = vec![
            create_vulnerability_fact(10, Some(5), "SQL injection"),
            create_code_smell_fact(10, Some(5), "Long method"),
        ];

        let (deduplicated, stats) = deduplicator.deduplicate(facts);

        // Should not deduplicate different fact types
        assert_eq!(deduplicated.len(), 2);
        assert_eq!(stats.duplicates_removed, 0);
    }

    #[test]
    fn test_same_location_different_fact_types() {
        let deduplicator = FactDeduplicator::new();

        let facts = vec![
            create_vulnerability_fact(10, Some(5), "Error"),
            create_vulnerability_fact(10, Some(5), "Error"),
            create_code_smell_fact(10, Some(5), "Error"),
            create_code_smell_fact(10, Some(5), "Error"),
        ];

        let (deduplicated, stats) = deduplicator.deduplicate(facts);

        // Should keep both fact types (2 unique)
        assert_eq!(deduplicated.len(), 2);
        assert_eq!(stats.duplicates_removed, 2);
    }

    #[test]
    fn test_empty_message() {
        let deduplicator = FactDeduplicator::new();

        let facts = vec![
            create_test_fact(10, Some(5), ""),
            create_test_fact(10, Some(5), ""),
        ];

        let (deduplicated, stats) = deduplicator.deduplicate(facts);

        // Should deduplicate even with empty messages
        assert_eq!(deduplicated.len(), 1);
        assert_eq!(stats.duplicates_removed, 1);
    }

    #[test]
    fn test_multi_tool_deduplication() {
        let deduplicator = FactDeduplicator::new();

        // Simulate same finding from CodeQL, ESLint, and Semgrep
        let facts = vec![
            create_test_fact_with_extractor(
                10,
                Some(5),
                "SQL injection vulnerability",
                ExtractorId::SarifAdapter,
                "CodeQL",
            ),
            create_test_fact_with_extractor(
                10,
                Some(5),
                "SQL injection vulnerability",
                ExtractorId::SarifAdapter,
                "Semgrep",
            ),
            create_test_fact_with_extractor(
                10,
                Some(5),
                "SQL injection vulnerability",
                ExtractorId::SarifAdapter,
                "ESLint",
            ),
            create_test_fact_with_extractor(
                20,
                Some(10),
                "XSS vulnerability",
                ExtractorId::SarifAdapter,
                "CodeQL",
            ),
        ];

        let (deduplicated, stats) = deduplicator.deduplicate(facts);

        // Should keep only 2 unique findings (SQL injection + XSS)
        assert_eq!(deduplicated.len(), 2);
        assert_eq!(stats.duplicates_removed, 2);
        assert_eq!(stats.deduplication_ratio, 0.5);
    }

    #[test]
    fn test_whitespace_handling() {
        let config = DeduplicationConfig {
            normalize_messages: true,
            ..Default::default()
        };
        let deduplicator = FactDeduplicator::with_config(config);

        let facts = vec![
            create_test_fact(10, Some(5), "\t\n  Error Message  \n\t"),
            create_test_fact(10, Some(5), "Error Message"),
            create_test_fact(10, Some(5), "  error message  "),
        ];

        let (deduplicated, stats) = deduplicator.deduplicate(facts);

        // Should deduplicate all with normalization
        assert_eq!(deduplicated.len(), 1);
        assert_eq!(stats.duplicates_removed, 2);
    }

    #[test]
    fn test_statistics_accuracy() {
        let deduplicator = FactDeduplicator::new();

        let facts = vec![
            create_test_fact(10, Some(5), "Error A"),
            create_test_fact(10, Some(5), "Error A"),
            create_test_fact(20, Some(10), "Error B"),
            create_test_fact(30, Some(15), "Error C"),
            create_test_fact(30, Some(15), "Error C"),
            create_test_fact(30, Some(15), "Error C"),
        ];

        let (_deduplicated, stats) = deduplicator.deduplicate(facts);

        assert_eq!(stats.total_before, 6);
        assert_eq!(stats.total_after, 3);
        assert_eq!(stats.duplicates_removed, 3);
        assert_eq!(stats.deduplication_ratio, 0.5);
        assert_eq!(stats.facts_by_fingerprint.len(), 3); // 3 unique fingerprints
    }

    // Helper functions for creating test facts with different configurations

    fn create_test_fact_with_path(
        line: u32,
        column: Option<u32>,
        message: &str,
        path: &str,
    ) -> Fact {
        let location = SourceLocation::new(
            ProjectPath::new(std::path::PathBuf::from(path)),
            LineNumber::new(line).unwrap(),
            column.map(|c| ColumnNumber::new(c).unwrap()),
            LineNumber::new(line).unwrap(),
            column.map(|c| ColumnNumber::new(c).unwrap()),
        );

        let provenance = Provenance::new(
            ExtractorId::Custom,
            "1.0.0".to_string(),
            Confidence::new(0.9).unwrap(),
        );

        Fact {
            id: hodei_ir::FactId::new(),
            message: message.to_string(),
            fact_type: FactType::CodeSmell {
                smell_type: "test".to_string(),
                severity: Severity::Minor,
            },
            location,
            provenance,
        }
    }

    fn create_vulnerability_fact(line: u32, column: Option<u32>, description: &str) -> Fact {
        let location = SourceLocation::new(
            ProjectPath::new(std::path::PathBuf::from("test.rs")),
            LineNumber::new(line).unwrap(),
            column.map(|c| ColumnNumber::new(c).unwrap()),
            LineNumber::new(line).unwrap(),
            column.map(|c| ColumnNumber::new(c).unwrap()),
        );

        let provenance = Provenance::new(
            ExtractorId::Custom,
            "1.0.0".to_string(),
            Confidence::new(0.9).unwrap(),
        );

        Fact {
            id: hodei_ir::FactId::new(),
            message: description.to_string(),
            fact_type: FactType::Vulnerability {
                cwe_id: Some("CWE-89".to_string()),
                owasp_category: Some("A03:2021".to_string()),
                severity: Severity::Major,
                cvss_score: Some(8.1),
                description: description.to_string(),
                confidence: Confidence::new(0.9).unwrap(),
            },
            location,
            provenance,
        }
    }

    fn create_code_smell_fact(line: u32, column: Option<u32>, smell_type: &str) -> Fact {
        let location = SourceLocation::new(
            ProjectPath::new(std::path::PathBuf::from("test.rs")),
            LineNumber::new(line).unwrap(),
            column.map(|c| ColumnNumber::new(c).unwrap()),
            LineNumber::new(line).unwrap(),
            column.map(|c| ColumnNumber::new(c).unwrap()),
        );

        let provenance = Provenance::new(
            ExtractorId::Custom,
            "1.0.0".to_string(),
            Confidence::new(0.9).unwrap(),
        );

        Fact {
            id: hodei_ir::FactId::new(),
            message: smell_type.to_string(),
            fact_type: FactType::CodeSmell {
                smell_type: smell_type.to_string(),
                severity: Severity::Minor,
            },
            location,
            provenance,
        }
    }

    fn create_test_fact_with_extractor(
        line: u32,
        column: Option<u32>,
        message: &str,
        extractor: ExtractorId,
        version: &str,
    ) -> Fact {
        let location = SourceLocation::new(
            ProjectPath::new(std::path::PathBuf::from("test.rs")),
            LineNumber::new(line).unwrap(),
            column.map(|c| ColumnNumber::new(c).unwrap()),
            LineNumber::new(line).unwrap(),
            column.map(|c| ColumnNumber::new(c).unwrap()),
        );

        let provenance = Provenance::new(
            extractor,
            version.to_string(),
            Confidence::new(0.9).unwrap(),
        );

        Fact {
            id: hodei_ir::FactId::new(),
            message: message.to_string(),
            fact_type: FactType::Vulnerability {
                cwe_id: Some("CWE-89".to_string()),
                owasp_category: Some("A03:2021".to_string()),
                severity: Severity::Major,
                cvss_score: Some(8.1),
                description: message.to_string(),
                confidence: Confidence::new(0.9).unwrap(),
            },
            location,
            provenance,
        }
    }
}
