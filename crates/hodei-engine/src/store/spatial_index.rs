//! Spatial index for location-based queries using R-tree
//!
//! This index provides efficient spatial queries using an R-tree data structure
//! from the rstar crate. It allows fast lookups of facts by file and line range.

use hodei_ir::{Fact, FactId, Provenance, SourceLocation};
use rstar::{AABB, RTree, RTreeObject};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Range;

/// Bounding box for spatial queries
#[derive(Clone, Debug)]
pub struct BoundingBox {
    file_hash: u64,
    line_start: u32,
    line_end: u32,
}

impl BoundingBox {
    /// Create a bounding box from file and line range
    pub fn from_file_lines(file: &str, line_range: Range<u32>) -> Self {
        let mut hasher = DefaultHasher::new();
        file.hash(&mut hasher);

        Self {
            file_hash: hasher.finish(),
            line_start: line_range.start,
            line_end: line_range.end,
        }
    }

    /// Get the R-tree envelope
    fn envelope(&self) -> AABB<[f64; 2]> {
        AABB::from_corners(
            [self.file_hash as f64, self.line_start as f64],
            [self.file_hash as f64, self.line_end as f64],
        )
    }
}

/// Entry in the spatial index (R-tree object)
#[derive(Clone, Debug)]
struct SpatialEntry {
    fact_id: FactId,
    bbox: BoundingBox,
}

impl RTreeObject for SpatialEntry {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.bbox.envelope()
    }
}

/// Spatial index using R-tree for efficient spatial queries
#[derive(Debug, Clone)]
pub struct SpatialIndex {
    rtree: RTree<SpatialEntry>,
}

impl SpatialIndex {
    /// Build the spatial index from facts
    pub fn build(facts: &[&Fact]) -> Self {
        let entries: Vec<SpatialEntry> = facts
            .iter()
            .map(|fact| SpatialEntry {
                fact_id: fact.id,
                bbox: BoundingBox::from_file_lines(
                    fact.location.file.as_str(),
                    fact.location.start_line.get()..fact.location.end_line.get(),
                ),
            })
            .collect();

        let rtree = RTree::bulk_load(entries);
        Self { rtree }
    }

    /// Query facts by file and line range
    pub fn query(&self, file: &str, line_start: u32, line_end: u32) -> Vec<FactId> {
        let bbox = BoundingBox::from_file_lines(file, line_start..line_end);
        self.rtree
            .locate_in_envelope_intersecting(&bbox.envelope())
            .map(|entry| entry.fact_id)
            .collect()
    }

    /// Get all facts in a specific file
    pub fn by_file(&self, file: &str) -> Vec<FactId> {
        let bbox = BoundingBox::from_file_lines(file, 0..u32::MAX);
        self.rtree
            .locate_in_envelope_intersecting(&bbox.envelope())
            .map(|entry| entry.fact_id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodei_ir::*;

    #[test]
    fn test_spatial_index_build() {
        let facts: Vec<&Fact> = vec![];
        let index = SpatialIndex::build(&facts);
        assert_eq!(index.rtree.size(), 0);
    }

    #[test]
    fn test_spatial_query() {
        use hodei_ir::{ColumnNumber, LineNumber, ProjectPath};

        // Create a test fact with a source location
        let location = SourceLocation {
            file: ProjectPath::new(std::path::PathBuf::from("test.rs")),
            start_line: LineNumber::new(10).unwrap(),
            end_line: LineNumber::new(20).unwrap(),
            start_column: Some(ColumnNumber::new(0).unwrap()),
            end_column: Some(ColumnNumber::new(100).unwrap()),
        };

        let fact = Fact {
            id: FactId(0),
            fact_type: FactType::Function {
                name: "test".to_string(),
                complexity: 1,
                lines_of_code: 10,
            },
            location,
            provenance: Provenance::default(),
        };

        let index = SpatialIndex::build(&[&fact]);
        let results = index.query("test.rs", 5, 15);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], FactId(0));
    }
}
