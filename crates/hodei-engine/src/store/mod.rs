//! Indexed fact store with multiple specialized indexes
//!
//! This module provides high-performance fact storage and querying
//! using multiple indexes (type, spatial, flow).

mod flow_index;
mod planner;
mod spatial_index;
mod type_index;

pub use flow_index::*;
pub use planner::*;
pub use spatial_index::*;
pub use type_index::*;

use hodei_ir::{Fact, FactId};
use std::collections::HashMap;

/// Main indexed fact store
#[derive(Debug, Clone)]
pub struct IndexedFactStore {
    facts: HashMap<FactId, Fact>,
    type_index: TypeIndex,
    spatial_index: SpatialIndex,
    flow_index: FlowIndex,
    stats: IndexStats,
}

impl IndexedFactStore {
    /// Build store from facts
    pub fn new(mut facts: Vec<Fact>) -> Self {
        // Convert Vec<Fact> to HashMap<FactId, Fact>
        let facts_map: HashMap<FactId, Fact> =
            facts.drain(..).map(|fact| (fact.id, fact)).collect();

        let facts_slice: Vec<&Fact> = facts_map.values().collect();
        let type_index = TypeIndex::build(&facts_slice);
        let spatial_index = SpatialIndex::build(&facts_slice);
        let flow_index = FlowIndex::build(&facts_slice);
        let stats = IndexStats::compute(&facts_slice);

        Self {
            facts: facts_map,
            type_index,
            spatial_index,
            flow_index,
            stats,
        }
    }

    /// Get facts by type
    pub fn by_type(&self, fact_type: &hodei_ir::FactType) -> Vec<&Fact> {
        if let Some(fact_ids) = self.type_index.get(fact_type) {
            fact_ids
                .iter()
                .filter_map(|id| self.facts.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get facts by location (file + line range)
    pub fn by_location(&self, file: &str, line_start: u32, line_end: u32) -> Vec<&Fact> {
        let fact_ids = self.spatial_index.query(file, line_start, line_end);
        fact_ids
            .iter()
            .filter_map(|id| self.facts.get(id))
            .collect()
    }

    /// Get facts in a flow
    pub fn by_flow(&self, flow_id: &hodei_ir::FlowId) -> Vec<&Fact> {
        if let Some(fact_ids) = self.flow_index.get_facts_for_flow(flow_id) {
            fact_ids
                .iter()
                .filter_map(|id| self.facts.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get facts by file
    pub fn by_file(&self, file: &str) -> Vec<&Fact> {
        self.spatial_index
            .file_lines
            .get(file)
            .map(|lines| {
                lines
                    .values()
                    .flatten()
                    .filter_map(|id| self.facts.get(id))
                    .collect()
            })
            .unwrap_or_else(Vec::new)
    }

    /// Get all facts
    pub fn get_all_facts(&self) -> Vec<Fact> {
        self.facts.values().cloned().collect()
    }

    /// Total number of facts
    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }

    /// Get statistics
    pub fn stats(&self) -> &IndexStats {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodei_ir::*;

    #[test]
    fn test_indexed_store() {
        let facts = vec![];
        let store = IndexedFactStore::new(facts);
        assert_eq!(store.fact_count(), 0);
    }
}
