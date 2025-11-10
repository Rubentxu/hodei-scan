//! Spatial index for location-based queries

use hodei_ir::{Fact, FactId, SourceLocation};
use std::collections::HashMap;

/// Simple spatial index using HashMap (R-tree would be better but simpler for demo)
#[derive(Debug, Clone)]
pub struct SpatialIndex {
    pub file_lines: HashMap<String, HashMap<u32, Vec<FactId>>>, // file -> line -> fact_ids
}

impl SpatialIndex {
    pub fn build(facts: &[&Fact]) -> Self {
        let mut file_lines = HashMap::new();

        for fact in facts {
            let file_str = fact.location.file.as_str().to_string();
            let line = fact.location.start_line.get();

            file_lines
                .entry(file_str)
                .or_insert_with(HashMap::new)
                .entry(line)
                .or_insert_with(Vec::new)
                .push(fact.id);
        }

        Self { file_lines }
    }

    pub fn query(&self, file: &str, line_start: u32, line_end: u32) -> Vec<FactId> {
        let mut results = Vec::new();

        if let Some(lines) = self.file_lines.get(file) {
            for line in line_start..line_end {
                if let Some(fact_ids) = lines.get(&line) {
                    results.extend(fact_ids);
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spatial_index() {
        let facts = vec![];
        let index = SpatialIndex::build(&facts);
        let results = index.query("test.rs", 0, 100);
        assert!(results.is_empty());
    }
}
