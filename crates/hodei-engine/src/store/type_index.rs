//! Type index for fast fact lookup by type
//!
//! This index provides O(1) lookups by FactType using an EnumMap for optimal performance.
//! It stores fact IDs grouped by their type for efficient filtering and improved cache locality.

use enum_map::{EnumMap, enum_map};
use hodei_ir::fact_type_index::FactTypeDiscriminant;
use hodei_ir::{Fact, FactId};

/// Index by FactType for O(1) lookups using EnumMap for optimal performance
#[derive(Debug, Clone)]
pub struct TypeIndex {
    /// EnumMap provides better cache locality and O(1) lookups
    index: EnumMap<FactTypeDiscriminant, Vec<FactId>>,
}

impl TypeIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self {
            index: enum_map! { _ => Vec::new() },
        }
    }

    /// Build the index from a collection of facts
    pub fn build(facts: &[&Fact]) -> Self {
        let mut index = Self::new();

        for fact in facts {
            // Get the discriminant from the FactType
            let discriminant = fact.fact_type.discriminant();

            // Insert the fact ID into the appropriate slot
            index.index[discriminant].push(fact.id);
        }

        // Sort fact IDs for better cache locality
        for ids in index.index.values_mut() {
            ids.sort_unstable();
        }

        index
    }

    /// Get fact IDs for a specific fact type (returns slice for zero-copy access)
    pub fn get(&self, fact_type: FactTypeDiscriminant) -> Option<&[FactId]> {
        let ids = &self.index[fact_type];
        if ids.is_empty() { None } else { Some(ids) }
    }

    /// Get the number of facts for a specific fact type
    pub fn cardinality(&self, fact_type: FactTypeDiscriminant) -> usize {
        self.index[fact_type].len()
    }

    /// Get all fact types that have facts in this index
    pub fn available_types(&self) -> Vec<FactTypeDiscriminant> {
        self.index
            .iter()
            .filter(|(_, ids)| !ids.is_empty())
            .map(|(fact_type, _)| fact_type)
            .collect()
    }

    /// Check if a specific fact type has any facts
    pub fn has_type(&self, fact_type: FactTypeDiscriminant) -> bool {
        !self.index[fact_type].is_empty()
    }

    /// Get the total number of facts indexed
    pub fn total_count(&self) -> usize {
        self.index.values().map(|v| v.len()).sum()
    }

    /// Clear all facts from the index
    pub fn clear(&mut self) {
        for ids in self.index.values_mut() {
            ids.clear();
        }
    }
}

impl Default for TypeIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_type_index_creation() {
        let index = TypeIndex::new();
        assert_eq!(index.total_count(), 0);
    }

    #[test]
    fn test_enum_map_benchmark() {
        use hodei_ir::{Confidence, ExtractorId, Fact, FactType, Provenance, SourceLocation};

        let start = Instant::now();
        let mut facts = Vec::new();

        // Create 1000 facts of different types
        for i in 0..1000 {
            let (fact_type, message) = if i % 3 == 0 {
                (
                    FactType::TaintSource {
                        var: hodei_ir::VariableName(format!("var{}", i)),
                        flow_id: hodei_ir::FlowId::new_uuid(),
                        source_type: "user_input".to_string(),
                        confidence: Confidence::HIGH,
                    },
                    format!("Taint source {}", i),
                )
            } else if i % 3 == 1 {
                (
                    FactType::Vulnerability {
                        cwe_id: Some(format!("CWE-{}", i)),
                        owasp_category: None,
                        severity: hodei_ir::Severity::Major,
                        cvss_score: Some(8.5),
                        description: format!("Vuln {}", i),
                        confidence: Confidence::MEDIUM,
                    },
                    format!("Vulnerability {}", i),
                )
            } else {
                (
                    FactType::Function {
                        name: hodei_ir::FunctionName(format!("func{}", i)),
                        complexity: 5,
                        lines_of_code: 50,
                    },
                    format!("Function {}", i),
                )
            };

            let fact = Fact::new_with_message(
                fact_type,
                message,
                SourceLocation::new(
                    hodei_ir::ProjectPath::new(std::path::PathBuf::from("test.rs")),
                    hodei_ir::LineNumber::new(1).unwrap(),
                    None,
                    hodei_ir::LineNumber::new(10).unwrap(),
                    None,
                ),
                Provenance::new(
                    ExtractorId::TreeSitter,
                    "1.0.0".to_string(),
                    Confidence::MEDIUM,
                ),
            );
            facts.push(fact);
        }

        let index = TypeIndex::build(&facts.iter().collect::<Vec<_>>());

        let duration = start.elapsed();

        // Should be very fast (less than 10ms for 1000 inserts)
        assert!(
            duration.as_millis() < 10,
            "EnumMap operations took too long: {:?}",
            duration
        );

        // Verify we have all types indexed
        let available_types = index.available_types();
        assert!(!available_types.is_empty());

        println!("Indexed {} facts in {:?}", index.total_count(), duration);
    }
}
