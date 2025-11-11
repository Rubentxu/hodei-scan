//! FactType index using EnumMap for better performance and cache locality

use crate::FactId;
use enum_map::{Enum, EnumMap, enum_map};

/// Discriminant enum for all FactType variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum, Hash, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum FactTypeDiscriminant {
    TaintSource,
    TaintSink,
    Sanitization,
    UnsafeCall,
    CryptographicOperation,
    Vulnerability,
    Function,
    Variable,
    CodeSmell,
    ComplexityViolation,
    Dependency,
    DependencyVulnerability,
    License,
    UncoveredLine,
    LowTestCoverage,
    CoverageStats,
    VulnerableUncovered,
    SecurityTechnicalDebt,
    QualitySecurityCorrelation,
}

/// Index for facts by type using EnumMap for optimal performance
pub struct FactTypeIndex {
    /// Index from FactType to vector of FactId
    index: EnumMap<FactTypeDiscriminant, Vec<FactId>>,
}

impl FactTypeIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        FactTypeIndex {
            index: enum_map! { _ => Vec::new() },
        }
    }

    /// Insert a fact ID by its type
    pub fn insert(&mut self, fact_type: FactTypeDiscriminant, fact_id: FactId) {
        self.index[fact_type].push(fact_id);
    }

    /// Get all fact IDs of a specific type
    pub fn get_by_type(&self, fact_type: FactTypeDiscriminant) -> &[FactId] {
        &self.index[fact_type]
    }

    /// Get the number of facts of a specific type
    pub fn count_by_type(&self, fact_type: FactTypeDiscriminant) -> usize {
        self.index[fact_type].len()
    }

    /// Get the total number of facts in the index
    pub fn total_count(&self) -> usize {
        self.index.values().map(Vec::len).sum()
    }

    /// Clear all facts from the index
    pub fn clear(&mut self) {
        for vec in self.index.values_mut() {
            vec.clear();
        }
    }
}

impl Default for FactTypeIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_fact_type_index_insert() {
        let mut index = FactTypeIndex::new();
        let fact_id = FactId::new();

        index.insert(FactTypeDiscriminant::TaintSource, fact_id);

        let results = index.get_by_type(FactTypeDiscriminant::TaintSource);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], fact_id);
    }

    #[test]
    fn test_fact_type_index_multiple_inserts() {
        let mut index = FactTypeIndex::new();

        for _i in 0..10 {
            let fact_id = FactId::new();
            index.insert(FactTypeDiscriminant::Vulnerability, fact_id);
        }

        let results = index.get_by_type(FactTypeDiscriminant::Vulnerability);
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn test_fact_type_index_different_types() {
        let mut index = FactTypeIndex::new();
        let id1 = FactId::new();
        let id2 = FactId::new();
        let id3 = FactId::new();

        index.insert(FactTypeDiscriminant::TaintSource, id1);
        index.insert(FactTypeDiscriminant::TaintSink, id2);
        index.insert(FactTypeDiscriminant::Vulnerability, id3);

        assert_eq!(
            index.get_by_type(FactTypeDiscriminant::TaintSource).len(),
            1
        );
        assert_eq!(index.get_by_type(FactTypeDiscriminant::TaintSink).len(), 1);
        assert_eq!(
            index.get_by_type(FactTypeDiscriminant::Vulnerability).len(),
            1
        );
        assert_eq!(index.get_by_type(FactTypeDiscriminant::Function).len(), 0);
    }

    #[test]
    fn test_fact_type_index_count() {
        let mut index = FactTypeIndex::new();

        assert_eq!(index.count_by_type(FactTypeDiscriminant::TaintSource), 0);

        index.insert(FactTypeDiscriminant::TaintSource, FactId::new());
        assert_eq!(index.count_by_type(FactTypeDiscriminant::TaintSource), 1);

        index.insert(FactTypeDiscriminant::TaintSource, FactId::new());
        assert_eq!(index.count_by_type(FactTypeDiscriminant::TaintSource), 2);
    }

    #[test]
    fn test_fact_type_index_total_count() {
        let mut index = FactTypeIndex::new();

        assert_eq!(index.total_count(), 0);

        index.insert(FactTypeDiscriminant::TaintSource, FactId::new());
        assert_eq!(index.total_count(), 1);

        index.insert(FactTypeDiscriminant::TaintSink, FactId::new());
        assert_eq!(index.total_count(), 2);

        index.insert(FactTypeDiscriminant::TaintSource, FactId::new());
        assert_eq!(index.total_count(), 3);
    }

    #[test]
    fn test_fact_type_index_clear() {
        let mut index = FactTypeIndex::new();

        index.insert(FactTypeDiscriminant::TaintSource, FactId::new());
        index.insert(FactTypeDiscriminant::TaintSink, FactId::new());
        index.insert(FactTypeDiscriminant::Vulnerability, FactId::new());

        assert_eq!(index.total_count(), 3);

        index.clear();

        assert_eq!(index.total_count(), 0);
        assert_eq!(index.count_by_type(FactTypeDiscriminant::TaintSource), 0);
        assert_eq!(index.count_by_type(FactTypeDiscriminant::TaintSink), 0);
        assert_eq!(index.count_by_type(FactTypeDiscriminant::Vulnerability), 0);
    }

    #[test]
    fn test_enum_map_benchmark() {
        let start = Instant::now();
        let mut index = FactTypeIndex::new();

        for _i in 0..1000 {
            let fact_id = FactId::new();
            index.insert(FactTypeDiscriminant::TaintSource, fact_id);
        }

        let duration = start.elapsed();

        // Should be very fast (less than 10ms for 1000 inserts)
        assert!(
            duration < Duration::from_millis(10),
            "EnumMap operations took too long: {:?}",
            duration
        );
    }
}
