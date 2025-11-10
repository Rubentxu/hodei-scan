//! Query planner for optimal index selection

use hodei_ir::FactType;
use std::collections::HashMap;

/// Query execution plan
#[derive(Debug, Clone)]
pub enum QueryPlan {
    /// Full scan (slowest)
    FullScan,

    /// Type index lookup (fastest for type queries)
    TypeIndexScan { fact_type: FactType },

    /// Spatial query (by file/line range)
    SpatialQuery {
        file: String,
        line_start: u32,
        line_end: u32,
    },

    /// Flow query (by flow_id)
    FlowQuery { flow_id: String },
}

/// Statistics for query planning
#[derive(Debug, Clone)]
pub struct IndexStats {
    type_cardinality: HashMap<String, usize>,
    total_facts: usize,
}

impl IndexStats {
    pub fn compute(facts: &[&hodei_ir::Fact]) -> Self {
        let mut type_cardinality = HashMap::new();

        for fact in facts {
            let type_str = match fact.fact_type {
                FactType::TaintSource { .. } => "TaintSource",
                FactType::TaintSink { .. } => "TaintSink",
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
                _ => "Other",
            };

            *type_cardinality.entry(type_str.to_string()).or_insert(0) += 1;
        }

        Self {
            type_cardinality,
            total_facts: facts.len(),
        }
    }

    pub fn cardinality(&self, fact_type: &str) -> usize {
        self.type_cardinality.get(fact_type).copied().unwrap_or(0)
    }
}

/// Simple query planner
pub struct QueryPlanner {
    stats: IndexStats,
}

impl QueryPlanner {
    pub fn new(stats: IndexStats) -> Self {
        Self { stats }
    }

    /// Plan the optimal query execution
    pub fn plan(&self, _query_type: &str) -> QueryPlan {
        // Simplified - always return TypeIndexScan for now
        let flow_id = hodei_ir::FlowId::new_uuid();
        let confidence = hodei_ir::Confidence::new(0.5).unwrap();
        QueryPlan::TypeIndexScan {
            fact_type: FactType::TaintSource {
                var: hodei_ir::VariableName("dummy".to_string()),
                flow_id,
                source_type: "http".to_string(),
                confidence,
            },
        }
    }
}
