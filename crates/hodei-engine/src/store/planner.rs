//! Query Planner and Optimizer for IndexedFactStore
//!
//! This module implements a cost-based query planner that selects optimal
//! execution plans based on index statistics and query characteristics.
//!
//! The planner supports multiple query types and optimization strategies:
//! - Type-based queries (using TypeIndex)
//! - Spatial queries (using SpatialIndex with R-tree)
//! - Flow-based queries (using FlowIndex with graph traversal)
//! - Full scans (as fallback)
//!
//! Optimization strategies:
//! - Index selection based on selectivity estimates
//! - Cost-based plan comparison
//! - Query result caching
//! - Parallel execution planning

use hodei_ir::{Fact, FactType, FactTypeDiscriminant, FlowId, LineNumber, ProjectPath};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

/// Query representation with optional predicates
#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    /// Select all facts of a specific type
    ByType(FactType),

    /// Select facts by file location
    ByFile(ProjectPath),

    /// Select facts by line range
    ByLineRange {
        file: ProjectPath,
        start: LineNumber,
        end: LineNumber,
    },

    /// Select facts by flow
    ByFlow(FlowId),

    /// Select facts by type and additional predicates
    Complex {
        type_discriminant: FactTypeDiscriminant,
        predicates: HashMap<String, String>,
    },

    /// Select all facts (fallback)
    All,
}

/// Cost estimate for query execution
#[derive(Debug, Clone, Copy)]
pub struct CostEstimate {
    /// Estimated I/O operations
    pub io_cost: f64,
    /// Estimated CPU operations
    pub cpu_cost: f64,
    /// Estimated memory usage (bytes)
    pub memory_cost: usize,
    /// Estimated result size (number of facts)
    pub result_size: usize,
}

impl CostEstimate {
    /// Create a new cost estimate
    pub fn new(io_cost: f64, cpu_cost: f64, memory_cost: usize, result_size: usize) -> Self {
        Self {
            io_cost,
            cpu_cost,
            memory_cost,
            result_size,
        }
    }

    /// Total cost (weighted sum of I/O and CPU)
    pub fn total_cost(&self) -> f64 {
        self.io_cost + self.cpu_cost
    }

    /// Check if this cost is better than another
    pub fn is_better_than(&self, other: &Self) -> bool {
        self.total_cost() < other.total_cost()
    }
}

/// Query execution plan
#[derive(Debug, Clone)]
pub struct QueryPlan {
    /// The query being executed
    pub query: Query,
    /// Selected index(es) for execution
    pub strategy: ExecutionStrategy,
    /// Cost estimate for this plan
    pub estimated_cost: CostEstimate,
    /// Whether this plan uses parallel execution
    pub parallel: bool,
    /// Explanation of why this plan was chosen
    pub explanation: String,
}

/// Execution strategies ordered by preference
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutionStrategy {
    /// Use TypeIndex for O(1) lookup by fact type
    TypeIndex { fact_type: FactType },

    /// Use SpatialIndex for range queries
    SpatialIndex {
        file: ProjectPath,
        line_range: Option<(LineNumber, LineNumber)>,
    },

    /// Use FlowIndex for graph traversal
    FlowIndex { flow_id: FlowId },

    /// Use multiple indexes and merge results
    IndexIntersection { strategies: Vec<ExecutionStrategy> },

    /// Use multiple indexes and union results
    IndexUnion { strategies: Vec<ExecutionStrategy> },

    /// Full scan of all facts
    FullScan,
}

impl ExecutionStrategy {
    /// Check if this strategy is optimal for the query
    pub fn is_optimal(&self) -> bool {
        !matches!(self, Self::FullScan)
    }
}

/// Statistics about index usage and selectivity
#[derive(Debug, Clone)]
pub struct IndexStatistics {
    /// Type index statistics
    pub type_stats: HashMap<String, TypeStat>,
    /// Spatial statistics by file
    pub spatial_stats: HashMap<String, SpatialStat>,
    /// Flow statistics
    pub flow_stats: HashMap<String, usize>,
    /// Total facts in store
    pub total_facts: usize,
}

/// Type statistics for query planning
#[derive(Debug, Clone)]
pub struct TypeStat {
    /// Number of facts of this type
    pub count: usize,
    /// Average size per fact in bytes
    pub avg_size_per_fact: usize,
}

/// Spatial statistics for query planning
#[derive(Debug, Clone)]
pub struct SpatialStat {
    /// File path
    pub file: ProjectPath,
    /// Number of facts in this file
    pub fact_count: usize,
    /// Line range occupied by facts
    pub line_range: (LineNumber, LineNumber),
    /// Facts per line density
    pub density: f64,
}

impl IndexStatistics {
    /// Compute statistics from facts
    pub fn compute(facts: &[&Fact]) -> Self {
        let mut type_stats = HashMap::new();
        let mut spatial_stats = HashMap::new();
        let mut flow_stats = HashMap::new();

        for fact in facts {
            // Type statistics
            let type_name = Self::type_name(&fact.fact_type);
            let type_stat = type_stats.entry(type_name).or_insert(TypeStat {
                count: 0,
                avg_size_per_fact: 64,
            });
            type_stat.count += 1;

            // Spatial statistics
            let location = &fact.location;
            let file_str = location.file.as_str();
            let spatial_stat = spatial_stats
                .entry(file_str.to_string())
                .or_insert(SpatialStat {
                    file: location.file.clone(),
                    fact_count: 0,
                    line_range: (location.start_line, location.end_line),
                    density: 0.0,
                });
            spatial_stat.fact_count += 1;

            // Flow statistics
            if let FactType::TaintSource { flow_id, .. } = &fact.fact_type {
                *flow_stats.entry(flow_id.as_str().to_string()).or_insert(0) += 1;
            }
        }

        Self {
            type_stats,
            spatial_stats,
            flow_stats,
            total_facts: facts.len(),
        }
    }

    fn type_name(fact_type: &FactType) -> String {
        match fact_type {
            FactType::TaintSource { .. } => "TaintSource".to_string(),
            FactType::TaintSink { .. } => "TaintSink".to_string(),
            FactType::Vulnerability { .. } => "Vulnerability".to_string(),
            FactType::Function { .. } => "Function".to_string(),
            FactType::Variable { .. } => "Variable".to_string(),
            FactType::CodeSmell { .. } => "CodeSmell".to_string(),
            FactType::ComplexityViolation { .. } => "ComplexityViolation".to_string(),
            FactType::Dependency { .. } => "Dependency".to_string(),
            FactType::DependencyVulnerability { .. } => "DependencyVulnerability".to_string(),
            FactType::License { .. } => "License".to_string(),
            FactType::UncoveredLine { .. } => "UncoveredLine".to_string(),
            FactType::LowTestCoverage { .. } => "LowTestCoverage".to_string(),
            FactType::CoverageStats { .. } => "CoverageStats".to_string(),
            FactType::Sanitization { .. } => "Sanitization".to_string(),
            FactType::UnsafeCall { .. } => "UnsafeCall".to_string(),
            FactType::CryptographicOperation { .. } => "CryptographicOperation".to_string(),
            FactType::Custom { discriminant, .. } => {
                format!("Custom:{}", discriminant)
            }
        }
    }
}

/// Query planner with cost-based optimization
pub struct QueryPlanner {
    stats: IndexStatistics,
    /// Cache for recent query plans
    plan_cache: HashMap<String, QueryPlan>,
    /// Configuration for optimization
    config: PlannerConfig,
}

/// Planner configuration
#[derive(Debug, Clone)]
pub struct PlannerConfig {
    /// When to consider a type selective (threshold as % of total)
    pub selective_threshold: f64,
    /// When to use parallel execution
    pub parallel_threshold: usize,
    /// Cache size limit
    pub max_cache_entries: usize,
}

impl Default for PlannerConfig {
    fn default() -> Self {
        Self {
            selective_threshold: 0.1, // 10% of facts
            parallel_threshold: 10_000,
            max_cache_entries: 1000,
        }
    }
}

impl QueryPlanner {
    /// Create a new query planner with statistics
    pub fn new(stats: IndexStatistics) -> Self {
        Self {
            stats,
            plan_cache: HashMap::new(),
            config: PlannerConfig::default(),
        }
    }

    /// Create a planner with custom configuration
    pub fn with_config(stats: IndexStatistics, config: PlannerConfig) -> Self {
        Self {
            stats,
            plan_cache: HashMap::new(),
            config,
        }
    }

    /// Plan the optimal execution for a query
    pub fn plan(&mut self, query: &Query) -> QueryPlan {
        // Check cache first
        let cache_key = self.cache_key(query);
        if let Some(plan) = self.plan_cache.get(&cache_key) {
            return plan.clone();
        }

        // Generate all possible plans
        let plans = self.generate_plans(query);

        // Select the best plan
        let best_plan = self.select_best_plan(query, &plans);

        // Cache the result
        self.cache_plan(cache_key, best_plan.clone());

        best_plan
    }

    /// Generate all possible execution plans for a query
    fn generate_plans(&self, query: &Query) -> Vec<QueryPlan> {
        let mut plans = Vec::new();

        match query {
            Query::ByType(fact_type) => {
                // TypeIndex is optimal for type queries
                let cost = self.estimate_type_index_cost(fact_type);
                plans.push(QueryPlan {
                    query: query.clone(),
                    strategy: ExecutionStrategy::TypeIndex {
                        fact_type: fact_type.clone(),
                    },
                    estimated_cost: cost,
                    parallel: false,
                    explanation: "TypeIndex provides O(1) lookup for type queries".to_string(),
                });

                // Full scan as fallback
                let full_scan_cost = self.estimate_full_scan_cost();
                plans.push(QueryPlan {
                    query: query.clone(),
                    strategy: ExecutionStrategy::FullScan,
                    estimated_cost: full_scan_cost,
                    parallel: true,
                    explanation: "Full scan as fallback option".to_string(),
                });
            }

            Query::ByFile(file) => {
                // SpatialIndex for file queries
                let cost = self.estimate_spatial_cost(file, None);
                plans.push(QueryPlan {
                    query: query.clone(),
                    strategy: ExecutionStrategy::SpatialIndex {
                        file: file.clone(),
                        line_range: None,
                    },
                    estimated_cost: cost,
                    parallel: false,
                    explanation: "SpatialIndex efficient for file-scoped queries".to_string(),
                });

                // Full scan
                let full_scan_cost = self.estimate_full_scan_cost();
                plans.push(QueryPlan {
                    query: query.clone(),
                    strategy: ExecutionStrategy::FullScan,
                    estimated_cost: full_scan_cost,
                    parallel: true,
                    explanation: "Full scan fallback for file query".to_string(),
                });
            }

            Query::ByLineRange { file, start, end } => {
                // SpatialIndex is optimal for line range queries
                let line_range = Some((*start, *end));
                let cost = self.estimate_spatial_cost(file, line_range);
                plans.push(QueryPlan {
                    query: query.clone(),
                    strategy: ExecutionStrategy::SpatialIndex {
                        file: file.clone(),
                        line_range,
                    },
                    estimated_cost: cost,
                    parallel: false,
                    explanation: "SpatialIndex R-tree optimized for line range queries".to_string(),
                });

                // Full scan
                let full_scan_cost = self.estimate_full_scan_cost();
                plans.push(QueryPlan {
                    query: query.clone(),
                    strategy: ExecutionStrategy::FullScan,
                    estimated_cost: full_scan_cost,
                    parallel: true,
                    explanation: "Full scan fallback for line range query".to_string(),
                });
            }

            Query::ByFlow(flow_id) => {
                // FlowIndex for flow queries
                let cost = self.estimate_flow_cost(flow_id);
                plans.push(QueryPlan {
                    query: query.clone(),
                    strategy: ExecutionStrategy::FlowIndex {
                        flow_id: flow_id.clone(),
                    },
                    estimated_cost: cost,
                    parallel: false,
                    explanation: "FlowIndex graph traversal for flow queries".to_string(),
                });

                // Full scan
                let full_scan_cost = self.estimate_full_scan_cost();
                plans.push(QueryPlan {
                    query: query.clone(),
                    strategy: ExecutionStrategy::FullScan,
                    estimated_cost: full_scan_cost,
                    parallel: true,
                    explanation: "Full scan fallback for flow query".to_string(),
                });
            }

            Query::Complex { .. } => {
                // Complex queries might benefit from index intersection
                // For now, use full scan
                let cost = self.estimate_full_scan_cost();
                plans.push(QueryPlan {
                    query: query.clone(),
                    strategy: ExecutionStrategy::FullScan,
                    estimated_cost: cost,
                    parallel: true,
                    explanation: "Complex query using full scan (index intersection TBD)"
                        .to_string(),
                });
            }

            Query::All => {
                // Full scan is the only option
                let cost = self.estimate_full_scan_cost();
                plans.push(QueryPlan {
                    query: query.clone(),
                    strategy: ExecutionStrategy::FullScan,
                    estimated_cost: cost,
                    parallel: true,
                    explanation: "Select all requires full scan".to_string(),
                });
            }
        }

        plans
    }

    /// Select the best plan from generated options
    fn select_best_plan(&self, query: &Query, plans: &[QueryPlan]) -> QueryPlan {
        if plans.is_empty() {
            // Fallback to full scan
            return QueryPlan {
                query: query.clone(),
                strategy: ExecutionStrategy::FullScan,
                estimated_cost: self.estimate_full_scan_cost(),
                parallel: true,
                explanation: "Emergency fallback - no plans available".to_string(),
            };
        }

        // Find the plan with minimum cost
        let mut best_plan = &plans[0];
        for plan in plans.iter().skip(1) {
            if plan
                .estimated_cost
                .is_better_than(&best_plan.estimated_cost)
            {
                best_plan = plan;
            }
        }

        best_plan.clone()
    }

    /// Estimate cost for TypeIndex query
    fn estimate_type_index_cost(&self, fact_type: &FactType) -> CostEstimate {
        let type_name = self.type_name(fact_type);
        let count = self
            .stats
            .type_stats
            .get(&type_name)
            .map(|s| s.count)
            .unwrap_or(0);

        let io_cost = 1.0; // O(1) hash lookup
        let cpu_cost = 1.0; // Minimal processing
        let memory_cost = count * 64; // Estimate 64 bytes per fact
        let result_size = count;

        CostEstimate::new(io_cost, cpu_cost, memory_cost, result_size)
    }

    /// Estimate cost for SpatialIndex query
    fn estimate_spatial_cost(
        &self,
        file: &ProjectPath,
        line_range: Option<(LineNumber, LineNumber)>,
    ) -> CostEstimate {
        let file_str = file.as_str();
        let file_count = self
            .stats
            .spatial_stats
            .get(file_str)
            .map(|s| s.fact_count)
            .unwrap_or(0);

        let (result_size, cpu_cost) = if let Some((start, end)) = line_range {
            // R-tree range query
            let line_span = end.get() - start.get() + 1;
            let selectivity = (line_span as f64) / 1000.0; // Assume 1000 lines per file
            let estimated = (file_count as f64 * selectivity).round() as usize;

            (estimated, 2.0 + selectivity) // R-tree has small overhead
        } else {
            // Full file query
            (file_count, 1.0)
        };

        let io_cost = 1.0; // R-tree lookup
        let memory_cost = result_size * 64;

        CostEstimate::new(io_cost, cpu_cost, memory_cost, result_size)
    }

    /// Estimate cost for FlowIndex query
    fn estimate_flow_cost(&self, flow_id: &FlowId) -> CostEstimate {
        let flow_str = flow_id.as_str();
        let related_count = self.stats.flow_stats.get(&flow_str).copied().unwrap_or(0);

        let io_cost = 1.0; // Graph node lookup
        let cpu_cost = 2.0; // Graph traversal
        let memory_cost = related_count * 64;
        let result_size = related_count;

        CostEstimate::new(io_cost, cpu_cost, memory_cost, result_size)
    }

    /// Estimate cost for full scan
    fn estimate_full_scan_cost(&self) -> CostEstimate {
        let io_cost = self.stats.total_facts as f64; // Scan all facts
        let cpu_cost = self.stats.total_facts as f64; // Check each fact
        let memory_cost = 0; // Streaming, no memory overhead
        let result_size = self.stats.total_facts;

        CostEstimate::new(io_cost, cpu_cost, memory_cost, result_size)
    }

    /// Get type name from FactType
    fn type_name(&self, fact_type: &FactType) -> String {
        match fact_type {
            FactType::TaintSource { .. } => "TaintSource".to_string(),
            FactType::TaintSink { .. } => "TaintSink".to_string(),
            FactType::Vulnerability { .. } => "Vulnerability".to_string(),
            FactType::Function { .. } => "Function".to_string(),
            FactType::Variable { .. } => "Variable".to_string(),
            FactType::CodeSmell { .. } => "CodeSmell".to_string(),
            FactType::ComplexityViolation { .. } => "ComplexityViolation".to_string(),
            FactType::Dependency { .. } => "Dependency".to_string(),
            FactType::DependencyVulnerability { .. } => "DependencyVulnerability".to_string(),
            FactType::License { .. } => "License".to_string(),
            FactType::UncoveredLine { .. } => "UncoveredLine".to_string(),
            FactType::LowTestCoverage { .. } => "LowTestCoverage".to_string(),
            FactType::CoverageStats { .. } => "CoverageStats".to_string(),
            FactType::Sanitization { .. } => "Sanitization".to_string(),
            FactType::UnsafeCall { .. } => "UnsafeCall".to_string(),
            FactType::CryptographicOperation { .. } => "CryptographicOperation".to_string(),
            FactType::Custom { discriminant, .. } => {
                format!("Custom:{}", discriminant)
            }
        }
    }

    /// Generate cache key for query
    fn cache_key(&self, query: &Query) -> String {
        match query {
            Query::ByType(t) => format!("type:{:?}", t.discriminant()),
            Query::ByFile(p) => format!("file:{}", p.as_str()),
            Query::ByLineRange { file, start, end } => {
                format!("range:{}:{}:{}", file.as_str(), start.get(), end.get())
            }
            Query::ByFlow(f) => format!("flow:{}", f.as_str()),
            Query::Complex {
                type_discriminant, ..
            } => {
                format!("complex:{:?}", type_discriminant)
            }
            Query::All => "all".to_string(),
        }
    }

    /// Cache a query plan
    fn cache_plan(&mut self, key: String, plan: QueryPlan) {
        if self.plan_cache.len() >= self.config.max_cache_entries {
            // Simple LRU: remove first entry
            let first_key = self.plan_cache.keys().next().unwrap().clone();
            self.plan_cache.remove(&first_key);
        }
        self.plan_cache.insert(key, plan);
    }

    /// Clear the plan cache
    pub fn clear_cache(&mut self) {
        self.plan_cache.clear();
    }

    /// Get planner statistics
    pub fn stats(&self) -> &IndexStatistics {
        &self.stats
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodei_ir::{Confidence, FlowId, LineNumber, ProjectPath};

    fn create_test_stats() -> IndexStatistics {
        let mut type_stats = HashMap::new();
        type_stats.insert(
            "TaintSource".to_string(),
            TypeStat {
                count: 100,
                avg_size_per_fact: 64,
            },
        );
        type_stats.insert(
            "Vulnerability".to_string(),
            TypeStat {
                count: 10,
                avg_size_per_fact: 64,
            },
        );

        let mut spatial_stats = HashMap::new();
        let path = hodei_ir::ProjectPath::new(std::path::PathBuf::from("src/main.rs"));
        spatial_stats.insert(
            "src/main.rs".to_string(),
            SpatialStat {
                file: path.clone(),
                fact_count: 50,
                line_range: (LineNumber::new(1).unwrap(), LineNumber::new(500).unwrap()),
                density: 0.1,
            },
        );

        let mut flow_stats = HashMap::new();
        flow_stats.insert("flow-123".to_string(), 5);

        IndexStatistics {
            type_stats,
            spatial_stats,
            flow_stats,
            total_facts: 200,
        }
    }

    #[test]
    fn test_type_query_planning() {
        let stats = create_test_stats();
        let mut planner = QueryPlanner::new(stats);

        let query = Query::ByType(FactType::TaintSource {
            var: hodei_ir::VariableName("test".to_string()),
            flow_id: FlowId::new_uuid(),
            source_type: "test".to_string(),
            confidence: Confidence::MEDIUM,
        });

        let plan = planner.plan(&query);

        match plan.strategy {
            ExecutionStrategy::TypeIndex { .. } => {
                // TypeIndex is optimal for type queries
            }
            _ => panic!("Expected TypeIndex strategy for type query"),
        }
    }

    #[test]
    fn test_spatial_query_planning() {
        let stats = create_test_stats();
        let mut planner = QueryPlanner::new(stats);

        let path = hodei_ir::ProjectPath::new(std::path::PathBuf::from("src/main.rs"));
        let start = LineNumber::new(10).unwrap();
        let end = LineNumber::new(50).unwrap();
        let query = Query::ByLineRange {
            file: path,
            start,
            end,
        };

        let plan = planner.plan(&query);

        match plan.strategy {
            ExecutionStrategy::SpatialIndex { .. } => {
                // SpatialIndex is optimal for line range queries
            }
            _ => panic!("Expected SpatialIndex strategy for line range query"),
        }
    }

    #[test]
    fn test_flow_query_planning() {
        let stats = create_test_stats();
        let mut planner = QueryPlanner::new(stats);

        let flow_id = FlowId::new_uuid();
        let query = Query::ByFlow(flow_id);

        let plan = planner.plan(&query);

        match plan.strategy {
            ExecutionStrategy::FlowIndex { .. } => {
                // FlowIndex is optimal for flow queries
            }
            _ => panic!("Expected FlowIndex strategy for flow query"),
        }
    }

    #[test]
    fn test_plan_cache() {
        let stats = create_test_stats();
        let mut planner = QueryPlanner::new(stats);

        let query = Query::All;
        let plan1 = planner.plan(&query);
        let plan2 = planner.plan(&query);

        assert_eq!(plan1.strategy, plan2.strategy);
        assert_eq!(planner.plan_cache.len(), 1);
    }

    #[test]
    fn test_cost_comparison() {
        let cheap = CostEstimate::new(1.0, 1.0, 100, 10);
        let expensive = CostEstimate::new(10.0, 10.0, 1000, 100);

        assert!(cheap.is_better_than(&expensive));
        assert!(!expensive.is_better_than(&cheap));
    }

    #[test]
    fn test_selective_threshold() {
        let stats = create_test_stats();
        let mut planner = QueryPlanner::with_config(
            stats,
            PlannerConfig {
                selective_threshold: 0.05, // 5%
                ..Default::default()
            },
        );

        // TaintSource is 50% of facts (not selective)
        let query = Query::ByType(FactType::TaintSource {
            var: hodei_ir::VariableName("test".to_string()),
            flow_id: FlowId::new_uuid(),
            source_type: "test".to_string(),
            confidence: Confidence::MEDIUM,
        });

        let plan = planner.plan(&query);
        // Should still use TypeIndex as it's the optimal strategy
        assert!(matches!(plan.strategy, ExecutionStrategy::TypeIndex { .. }));
    }
}
