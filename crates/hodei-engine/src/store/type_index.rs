//! Type index for fast fact lookup by type

use hodei_ir::{Fact, FactId, FactType};
use std::collections::HashMap;

/// Index by FactType name for O(1) lookups
#[derive(Debug, Clone)]
pub struct TypeIndex {
    index: HashMap<String, Vec<FactId>>,
}

impl TypeIndex {
    pub fn build(facts: &[&Fact]) -> Self {
        let mut index: HashMap<String, Vec<FactId>> = HashMap::new();

        for fact in facts {
            let type_name = match &fact.fact_type {
                FactType::TaintSource { .. } => "TaintSource",
                FactType::TaintSink { .. } => "TaintSink",
                FactType::Sanitization { .. } => "Sanitization",
                FactType::UnsafeCall { .. } => "UnsafeCall",
                FactType::CryptographicOperation { .. } => "CryptographicOperation",
                FactType::Vulnerability { .. } => "Vulnerability",
                FactType::Function { .. } => "Function",
                FactType::Variable { .. } => "Variable",
                FactType::CodeSmell { .. } => "CodeSmell",
                FactType::ComplexityViolation { .. } => "ComplexityViolation",
                FactType::Dependency { .. } => "Dependency",
                FactType::DependencyVulnerability { .. } => "DependencyVulnerability",
                FactType::License { .. } => "License",
                FactType::UncoveredLine { .. } => "UncoveredLine",
                FactType::LowTestCoverage { .. } => "LowTestCoverage",
                FactType::CoverageStats { .. } => "CoverageStats",
            };

            index
                .entry(type_name.to_string())
                .or_insert_with(Vec::new)
                .push(fact.id);
        }

        // Note: Could sort for cache locality, but FactId doesn't implement Ord
        // for ids in index.values_mut() {
        //     ids.sort_unstable();
        // }

        Self { index }
    }

    /// Get fact IDs for a specific fact type
    pub fn get(&self, fact_type: &FactType) -> Option<&[FactId]> {
        let type_name = match fact_type {
            FactType::TaintSource { .. } => "TaintSource",
            FactType::TaintSink { .. } => "TaintSink",
            FactType::Sanitization { .. } => "Sanitization",
            FactType::UnsafeCall { .. } => "UnsafeCall",
            FactType::CryptographicOperation { .. } => "CryptographicOperation",
            FactType::Vulnerability { .. } => "Vulnerability",
            FactType::Function { .. } => "Function",
            FactType::Variable { .. } => "Variable",
            FactType::CodeSmell { .. } => "CodeSmell",
            FactType::ComplexityViolation { .. } => "ComplexityViolation",
            FactType::Dependency { .. } => "Dependency",
            FactType::DependencyVulnerability { .. } => "DependencyVulnerability",
            FactType::License { .. } => "License",
            FactType::UncoveredLine { .. } => "UncoveredLine",
            FactType::LowTestCoverage { .. } => "LowTestCoverage",
            FactType::CoverageStats { .. } => "CoverageStats",
        };

        self.index.get(type_name).map(|v| v.as_slice())
    }

    /// Get the number of facts for a specific fact type
    pub fn cardinality(&self, fact_type: &FactType) -> usize {
        let type_name = match fact_type {
            FactType::TaintSource { .. } => "TaintSource",
            FactType::TaintSink { .. } => "TaintSink",
            FactType::Sanitization { .. } => "Sanitization",
            FactType::UnsafeCall { .. } => "UnsafeCall",
            FactType::CryptographicOperation { .. } => "CryptographicOperation",
            FactType::Vulnerability { .. } => "Vulnerability",
            FactType::Function { .. } => "Function",
            FactType::Variable { .. } => "Variable",
            FactType::CodeSmell { .. } => "CodeSmell",
            FactType::ComplexityViolation { .. } => "ComplexityViolation",
            FactType::Dependency { .. } => "Dependency",
            FactType::DependencyVulnerability { .. } => "DependencyVulnerability",
            FactType::License { .. } => "License",
            FactType::UncoveredLine { .. } => "UncoveredLine",
            FactType::LowTestCoverage { .. } => "LowTestCoverage",
            FactType::CoverageStats { .. } => "CoverageStats",
        };

        self.index.get(type_name).map_or(0, |v| v.len())
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
