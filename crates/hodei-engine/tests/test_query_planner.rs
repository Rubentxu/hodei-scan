//! Tests for Query Planner with Cost-Based Optimization

use chrono::Utc;
use hodei_engine::store::planner::*;
use hodei_ir::*;
use std::collections::HashMap;
use std::path::PathBuf;

#[cfg(test)]
mod query_planner_tests {
    use super::*;

    /// Helper to create test facts
    fn create_test_facts() -> Vec<Fact> {
        let mut facts = Vec::new();

        // TaintSource facts
        for i in 0..10 {
            let flow_id = FlowId::new_uuid();
            let fact = Fact {
                id: FactId::new(),
                fact_type: FactType::TaintSource {
                    var: VariableName(("user_input_".to_string() + &i.to_string()).into()),
                    flow_id,
                    source_type: "http_request".to_string(),
                    confidence: Confidence::new(0.9).unwrap(),
                },
                location: SourceLocation {
                    file: ProjectPath::new(PathBuf::from("src/main.rs")),
                    start_line: LineNumber::new(10 + i as u32).unwrap(),
                    start_column: Some(ColumnNumber::new(5).unwrap()),
                    end_line: LineNumber::new(10 + i as u32).unwrap(),
                    end_column: Some(ColumnNumber::new(20).unwrap()),
                },
                provenance: Provenance {
                    extractor: ExtractorId::TreeSitter,
                    version: "1.0.0".to_string(),
                    confidence: Confidence::new(0.9).unwrap(),
                    extracted_at: Utc::now(),
                },
            };
            facts.push(fact);
        }

        // TaintSink facts
        for i in 0..5 {
            let fact = Fact {
                id: FactId::new(),
                fact_type: FactType::TaintSink {
                    func: FunctionName(("process_".to_string() + &i.to_string()).into()),
                    consumes_flow: FlowId::new_uuid(),
                    category: "SQL_INJECTION".to_string(),
                    severity: Severity::Critical,
                },
                location: SourceLocation {
                    file: ProjectPath::new(PathBuf::from("src/main.rs")),
                    start_line: LineNumber::new(100 + i as u32).unwrap(),
                    start_column: Some(ColumnNumber::new(5).unwrap()),
                    end_line: LineNumber::new(100 + i as u32).unwrap(),
                    end_column: Some(ColumnNumber::new(20).unwrap()),
                },
                provenance: Provenance {
                    extractor: ExtractorId::TreeSitter,
                    version: "1.0.0".to_string(),
                    confidence: Confidence::new(0.9).unwrap(),
                    extracted_at: Utc::now(),
                },
            };
            facts.push(fact);
        }

        // Vulnerability facts
        for i in 0..3 {
            let fact = Fact {
                id: FactId::new(),
                fact_type: FactType::Vulnerability {
                    cwe_id: Some("CWE-79".to_string()),
                    owasp_category: Some("A03:2021".to_string()),
                    severity: Severity::Critical,
                    cvss_score: Some(9.8),
                    description: format!("Cross-site scripting vulnerability {}", i),
                    confidence: Confidence::new(0.9).unwrap(),
                },
                location: SourceLocation {
                    file: ProjectPath::new(PathBuf::from("src/vuln.rs")),
                    start_line: LineNumber::new(50 + i as u32).unwrap(),
                    start_column: Some(ColumnNumber::new(1).unwrap()),
                    end_line: LineNumber::new(50 + i as u32).unwrap(),
                    end_column: Some(ColumnNumber::new(30).unwrap()),
                },
                provenance: Provenance {
                    extractor: ExtractorId::TreeSitter,
                    version: "1.0.0".to_string(),
                    confidence: Confidence::new(0.9).unwrap(),
                    extracted_at: Utc::now(),
                },
            };
            facts.push(fact);
        }

        // Function facts
        for i in 0..20 {
            let fact = Fact {
                id: FactId::new(),
                fact_type: FactType::Function {
                    name: FunctionName(("function_".to_string() + &i.to_string()).into()),
                    complexity: 5 + i as u32,
                    lines_of_code: 20 + i as u32,
                },
                location: SourceLocation {
                    file: ProjectPath::new(PathBuf::from("src/main.rs")),
                    start_line: LineNumber::new(200 + i as u32).unwrap(),
                    start_column: Some(ColumnNumber::new(1).unwrap()),
                    end_line: LineNumber::new(200 + i as u32).unwrap(),
                    end_column: Some(ColumnNumber::new(10).unwrap()),
                },
                provenance: Provenance {
                    extractor: ExtractorId::TreeSitter,
                    version: "1.0.0".to_string(),
                    confidence: Confidence::new(0.9).unwrap(),
                    extracted_at: Utc::now(),
                },
            };
            facts.push(fact);
        }

        facts
    }

    #[test]
    fn test_cost_estimate_creation() {
        let cost = CostEstimate::new(100.0, 50.0, 1024, 10);
        assert_eq!(cost.io_cost, 100.0);
        assert_eq!(cost.cpu_cost, 50.0);
        assert_eq!(cost.memory_cost, 1024);
        assert_eq!(cost.result_size, 10);
    }

    #[test]
    fn test_cost_estimate_total_cost() {
        let cost = CostEstimate::new(100.0, 50.0, 1024, 10);
        assert_eq!(cost.total_cost(), 150.0);
    }

    #[test]
    fn test_cost_estimate_comparison() {
        let cost1 = CostEstimate::new(100.0, 50.0, 1024, 10);
        let cost2 = CostEstimate::new(120.0, 40.0, 1024, 10);
        let cost3 = CostEstimate::new(90.0, 80.0, 1024, 10);

        assert!(cost1.is_better_than(&cost2));
        assert!(cost2.is_better_than(&cost3));
        assert!(cost1.is_better_than(&cost3));
    }

    #[test]
    fn test_index_statistics_creation() {
        let facts = create_test_facts();
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);

        assert_eq!(stats.total_facts, 38);
        assert!(stats.type_stats.contains_key("TaintSource"));
        assert!(stats.type_stats.contains_key("TaintSink"));
        assert!(stats.type_stats.contains_key("Vulnerability"));
        assert!(stats.type_stats.contains_key("Function"));

        // Check TaintSource count
        let taint_source_stat = stats.type_stats.get("TaintSource").unwrap();
        assert_eq!(taint_source_stat.count, 10);
    }

    #[test]
    fn test_index_statistics_spatial() {
        let facts = create_test_facts();
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);

        assert!(stats.spatial_stats.contains_key("src/main.rs"));
        assert!(stats.spatial_stats.contains_key("src/vuln.rs"));

        // main.rs should have 35 facts (10 taint + 5 taint_sink + 20 functions)
        let main_stats = stats.spatial_stats.get("src/main.rs").unwrap();
        assert_eq!(main_stats.fact_count, 35);
    }

    #[test]
    fn test_index_statistics_flow() {
        let facts = create_test_facts();
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);

        // Should have flow statistics for 10 TaintSource facts
        assert!(!stats.flow_stats.is_empty());
        assert_eq!(stats.flow_stats.len(), 10);
    }

    #[test]
    fn test_query_planner_creation() {
        let facts = create_test_facts();
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        assert_eq!(planner.stats().total_facts, 38);
    }

    #[test]
    fn test_query_planner_by_type() {
        let facts = create_test_facts();
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        // Query for vulnerabilities
        let query = Query::ByType(FactType::Vulnerability {
            cwe_id: Some("CWE-79".to_string()),
            owasp_category: Some("A03:2021".to_string()),
            severity: Severity::Critical,
            cvss_score: Some(9.8),
            description: "Test vulnerability".into(),
            confidence: Confidence::new(0.9).unwrap(),
        });

        let plan = planner.plan(&query);

        assert_eq!(plan.query, query);
        assert!(matches!(plan.strategy, ExecutionStrategy::TypeIndex { .. }));
        assert!(plan.estimated_cost.result_size > 0);
        assert!(!plan.explanation.is_empty());
    }

    #[test]
    fn test_query_planner_by_file() {
        let facts = create_test_facts();
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        let file_path = ProjectPath::new(PathBuf::from("src/main.rs"));
        let query = Query::ByFile(file_path.clone());

        let plan = planner.plan(&query);

        assert_eq!(plan.query, query);
        assert!(matches!(
            plan.strategy,
            ExecutionStrategy::SpatialIndex { .. }
        ));
    }

    #[test]
    fn test_query_planner_by_flow() {
        let facts = create_test_facts();
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        // Use a flow ID from one of the taint sources
        let flow_id = FlowId::new_uuid();
        let query = Query::ByFlow(flow_id);

        let plan = planner.plan(&query);

        assert_eq!(plan.query, query);
        assert!(matches!(plan.strategy, ExecutionStrategy::FlowIndex { .. }));
    }

    #[test]
    fn test_query_planner_all_facts() {
        let facts = create_test_facts();
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        let query = Query::All;
        let plan = planner.plan(&query);

        assert_eq!(plan.query, query);
        assert!(matches!(plan.strategy, ExecutionStrategy::FullScan));
    }

    #[test]
    fn test_execution_strategy_selection() {
        let facts = create_test_facts();
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        // Type-based query should prefer TypeIndex
        let taint_query = Query::ByType(FactType::TaintSource {
            var: VariableName("test".into()),
            flow_id: FlowId::new_uuid(),
            source_type: "test".to_string(),
            confidence: Confidence::new(0.9).unwrap(),
        });
        let taint_plan = planner.plan(&taint_query);
        assert!(matches!(
            taint_plan.strategy,
            ExecutionStrategy::TypeIndex { .. }
        ));

        // File-based query should prefer SpatialIndex
        let file_path = ProjectPath::new(PathBuf::from("src/main.rs"));
        let file_query = Query::ByFile(file_path);
        let file_plan = planner.plan(&file_query);
        assert!(matches!(
            file_plan.strategy,
            ExecutionStrategy::SpatialIndex { .. }
        ));
    }

    #[test]
    fn test_cost_based_optimization() {
        let facts = create_test_facts();
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        // Query for rare type (should be more selective)
        let vuln_query = Query::ByType(FactType::Vulnerability {
            cwe_id: Some("CWE-79".to_string()),
            owasp_category: Some("A03:2021".to_string()),
            severity: Severity::Critical,
            cvss_score: Some(9.8),
            description: "Test".into(),
            confidence: Confidence::new(0.9).unwrap(),
        });

        let plan = planner.plan(&vuln_query);

        // Should have reasonable cost estimate
        assert!(plan.estimated_cost.total_cost() > 0.0);
        assert!(plan.estimated_cost.result_size > 0);
    }

    #[test]
    fn test_planner_config() {
        let config = PlannerConfig {
            selective_threshold: 0.05,
            parallel_threshold: 5000,
            max_cache_entries: 500,
        };

        assert_eq!(config.selective_threshold, 0.05);
        assert_eq!(config.parallel_threshold, 5000);
        assert_eq!(config.max_cache_entries, 500);
    }

    #[test]
    fn test_default_planner_config() {
        let config = PlannerConfig::default();

        assert!(config.selective_threshold > 0.0);
        assert!(config.parallel_threshold > 0);
        assert!(config.max_cache_entries > 0);
    }

    #[test]
    fn test_complex_query() {
        let facts = create_test_facts();
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        let mut predicates = HashMap::new();
        predicates.insert("severity".to_string(), "Critical".to_string());

        let query = Query::Complex {
            type_discriminant: FactTypeDiscriminant::Vulnerability,
            predicates,
        };

        let plan = planner.plan(&query);

        assert!(matches!(plan.query, Query::Complex { .. }));
        assert!(!plan.explanation.is_empty());
    }

    #[test]
    fn test_line_range_query() {
        let facts = create_test_facts();
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        let file_path = ProjectPath::new(PathBuf::from("src/main.rs"));
        let start = LineNumber::new(10).unwrap();
        let end = LineNumber::new(20).unwrap();

        let query = Query::ByLineRange {
            file: file_path.clone(),
            start,
            end,
        };

        let plan = planner.plan(&query);

        assert!(matches!(
            plan.strategy,
            ExecutionStrategy::SpatialIndex { .. }
        ));
    }

    #[test]
    fn test_query_plan_explanation() {
        let facts = create_test_facts();
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        let query = Query::ByType(FactType::Function {
            name: FunctionName("test".into()),
            complexity: 10,
            lines_of_code: 50,
        });

        let plan = planner.plan(&query);

        // Explanation should contain useful information
        assert!(!plan.explanation.is_empty());
        assert!(plan.explanation.len() > 10);
    }

    #[test]
    fn test_multiple_query_types() {
        let facts = create_test_facts();
        let facts_refs: Vec<&Fact> = facts.iter().collect();
        let stats = IndexStatistics::compute(&facts_refs);
        let mut planner = QueryPlanner::with_config(stats, PlannerConfig::default());

        // Test different query types
        let queries = vec![
            Query::All,
            Query::ByFile(ProjectPath::new(PathBuf::from("src/main.rs"))),
            Query::ByFlow(FlowId::new_uuid()),
        ];

        for query in queries {
            let plan = planner.plan(&query);
            // Query planning should not fail
            assert!(!plan.explanation.is_empty());
        }
    }
}
