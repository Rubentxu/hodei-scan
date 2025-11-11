//! Type index for fast fact lookup by type
//!
//! This index provides O(1) lookups by FactType using an EnumMap for optimal performance.
//! It stores fact IDs grouped by their type for efficient filtering.

use hodei_ir::{Fact, FactId, FactType};
use std::collections::HashMap;

/// Index by FactType for O(1) lookups using EnumMap
#[derive(Debug, Clone)]
pub struct TypeIndex {
    index: HashMap<FactType, Vec<FactId>>,
}

impl TypeIndex {
    /// Build the index from a collection of facts
    pub fn build(facts: &[&Fact]) -> Self {
        let mut index: HashMap<FactType, Vec<FactId>> = HashMap::new();

        for fact in facts {
            index
                .entry(fact.fact_type.clone())
                .or_insert_with(Vec::new)
                .push(fact.id);
        }

        // Sort fact IDs for better cache locality
        for ids in index.values_mut() {
            ids.sort_unstable();
        }

        Self { index }
    }

    /// Get fact IDs for a specific fact type
    pub fn get(&self, fact_type: &FactType) -> Option<&[FactId]> {
        self.index.get(fact_type).map(|v| v.as_slice())
    }

    /// Get the number of facts for a specific fact type
    pub fn cardinality(&self, fact_type: &FactType) -> usize {
        self.index.get(fact_type).map_or(0, |v| v.len())
    }

    /// Get all fact types that have facts in this index
    pub fn available_types(&self) -> Vec<&FactType> {
        self.index.keys().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_index_creation() {
        // Simple test - just verify the build function works
        let facts: Vec<&hodei_ir::Fact> = vec![];
        let index = TypeIndex::build(&facts);
        // Just verify it builds without panicking
        assert!(index.index.is_empty());
    }
}
