//! Comprehensive test suite for IndexedFactStore
//!
//! Tests cover all three index types and their integration:
//! - TypeIndex tests: creation, lookup, cardinality
//! - SpatialIndex tests: R-tree queries, file lookups
//! - FlowIndex tests: graph traversal, reachability
//! - Integration tests: IndexedFactStore high-level API

#[cfg(test)]
mod tests {
    use super::*;
    use hodei_ir::*;

    mod type_index_tests {
        use super::*;

        #[test]
        fn test_type_index_creation() {
            let facts = vec![];
            let facts_refs: Vec<&Fact> = facts.iter().collect();
            let index = TypeIndex::build(&facts_refs);
            assert!(index.available_types().is_empty());
        }

        #[test]
        fn test_type_index_lookup() {
            let flow_id = FlowId::new_uuid();
            let provenance = Provenance::new(
                ExtractorId::new("test".to_string()),
                "1.0.0".to_string(),
                Confidence::new(0.8).unwrap(),
            );

            let facts = vec![
                Fact {
                    id: FactId::new(),
                    fact_type: FactType::TaintSource {
                        var: VariableName("input".to_string()),
                        flow_id,
                        source_type: "http".to_string(),
                        confidence: Confidence::new(0.8).unwrap(),
                    },
                    location: create_test_location("test.rs", 1, 10, 0, 100),
                    provenance: provenance.clone(),
                },
                Fact {
                    id: FactId::new(),
                    fact_type: FactType::Function {
                        name: "handler".to_string(),
                        complexity: 5,
                        lines_of_code: 20,
                    },
                    location: create_test_location("test.rs", 20, 40, 0, 100),
                    provenance,
                },
            ];
            let facts_refs: Vec<&Fact> = facts.iter().collect();
            let index = TypeIndex::build(&facts_refs);

            let taint_source = FactType::TaintSource {
                var: VariableName("input".to_string()),
                flow_id,
                source_type: "http".to_string(),
                confidence: Confidence::new(0.8).unwrap(),
            };

            let result = index.get(&taint_source);
            assert!(result.is_some());
            assert_eq!(result.unwrap().len(), 1);
        }

        #[test]
        fn test_type_index_cardinality() {
            let facts = generate_test_facts(100, 50); // 50 TaintSource, 50 Function
            let facts_refs: Vec<&Fact> = facts.iter().collect();
            let index = TypeIndex::build(&facts_refs);

            let taint_source = FactType::TaintSource {
                var: VariableName("test".to_string()),
                flow_id: FlowId::new_uuid(),
                source_type: "http".to_string(),
                confidence: Confidence::new(0.8).unwrap(),
            };

            let cardinality = index.cardinality(&taint_source);
            // Cardinality should be 0 since we created with different flow_id
            assert_eq!(cardinality, 0);
        }

        #[test]
        fn test_type_index_multiple_types() {
            let facts = generate_test_facts(10, 10);
            let facts_refs: Vec<&Fact> = facts.iter().collect();
            let index = TypeIndex::build(&facts_refs);

            let types = index.available_types();
            assert!(types.len() >= 1);
        }
    }

    mod spatial_index_tests {
        use super::*;

        #[test]
        fn test_spatial_index_creation() {
            let facts = vec![];
            let facts_refs: Vec<&Fact> = facts.iter().collect();
            let index = SpatialIndex::build(&facts_refs);
            assert_eq!(index.rtree.size(), 0);
        }

        #[test]
        fn test_spatial_query_file_range() {
            let facts = generate_test_facts_with_location(5);
            let facts_refs: Vec<&Fact> = facts.iter().collect();
            let index = SpatialIndex::build(&facts_refs);

            let results = index.query("test.rs", 0, 100);
            // Should find facts in the range
            assert!(!results.is_empty());
        }

        #[test]
        fn test_spatial_by_file() {
            let facts = generate_test_facts_with_location(10);
            let facts_refs: Vec<&Fact> = facts.iter().collect();
            let index = SpatialIndex::build(&facts_refs);

            let results = index.by_file("test.rs");
            assert_eq!(results.len(), 10);
        }

        #[test]
        fn test_spatial_no_results() {
            let facts = generate_test_facts_with_location(5);
            let facts_refs: Vec<&Fact> = facts.iter().collect();
            let index = SpatialIndex::build(&facts_refs);

            let results = index.query("nonexistent.rs", 0, 100);
            assert!(results.is_empty());
        }
    }

    mod flow_index_tests {
        use super::*;

        #[test]
        fn test_flow_index_creation() {
            let facts = vec![];
            let facts_refs: Vec<&Fact> = facts.iter().collect();
            let index = FlowIndex::build(&facts_refs);
            assert_eq!(index.graph.node_count(), 0);
            assert_eq!(index.graph.edge_count(), 0);
        }

        #[test]
        fn test_flow_reachable_from() {
            let flow_id = FlowId::new_uuid();
            let facts = generate_test_facts_with_flow(flow_id, 3);
            let facts_refs: Vec<&Fact> = facts.iter().collect();
            let index = FlowIndex::build(&facts_refs);

            // Find a TaintSource fact
            let source_fact = facts_refs
                .iter()
                .find(|f| matches!(f.fact_type, FactType::TaintSource { .. }))
                .unwrap();

            let reachable = index.reachable_from(source_fact.id);
            // Should be able to reach at least itself
            assert!(!reachable.is_empty());
        }

        #[test]
        fn test_flow_get_facts_for_flow() {
            let flow_id = FlowId::new_uuid();
            let facts = generate_test_facts_with_flow(flow_id, 5);
            let facts_refs: Vec<&Fact> = facts.iter().collect();
            let index = FlowIndex::build(&facts_refs);

            let flow_facts = index.get_facts_for_flow(&flow_id);
            assert!(flow_facts.is_some());
            // Should have facts for this flow
            assert!(!flow_facts.unwrap().is_empty());
        }

        #[test]
        fn test_flow_nodes_and_edges() {
            let flow_id = FlowId::new_uuid();
            let facts = generate_test_facts_with_flow(flow_id, 10);
            let facts_refs: Vec<&Fact> = facts.iter().collect();
            let index = FlowIndex::build(&facts_refs);

            let nodes = index.nodes();
            let edge_count = index.edge_count();

            assert!(nodes.len() > 0);
            assert_eq!(edge_count, 0); // No edges in current implementation
        }
    }

    mod indexed_store_tests {
        use super::*;

        #[test]
        fn test_indexed_store_creation() {
            let facts = vec![];
            let store = IndexedFactStore::new(facts);
            assert_eq!(store.fact_count(), 0);
        }

        #[test]
        fn test_indexed_store_by_type() {
            let facts = generate_test_facts(20, 10);
            let store = IndexedFactStore::new(facts);

            // Query by a TaintSource type (we'll get 0 since flow_id won't match)
            let test_type = FactType::TaintSource {
                var: VariableName("test".to_string()),
                flow_id: FlowId::new_uuid(),
                source_type: "test".to_string(),
                confidence: Confidence::new(0.8).unwrap(),
            };

            let results = store.by_type(&test_type);
            assert_eq!(results.len(), 0);
        }

        #[test]
        fn test_indexed_store_by_location() {
            let facts = generate_test_facts_with_location(15);
            let store = IndexedFactStore::new(facts);

            let results = store.by_location("test.rs", 0, 100);
            assert!(!results.is_empty());
        }

        #[test]
        fn test_indexed_store_by_file() {
            let facts = generate_test_facts_with_location(25);
            let store = IndexedFactStore::new(facts);

            let results = store.by_file("test.rs");
            assert_eq!(results.len(), 25);
        }

        #[test]
        fn test_indexed_store_get_all_facts() {
            let facts = generate_test_facts(10, 10);
            let store = IndexedFactStore::new(facts);

            let all_facts = store.get_all_facts();
            assert_eq!(all_facts.len(), 20);
        }

        #[test]
        fn test_indexed_store_stats() {
            let facts = generate_test_facts(5, 5);
            let store = IndexedFactStore::new(facts);

            let stats = store.stats();
            assert!(stats.total_facts() > 0);
        }
    }

    // Helper functions

    fn create_test_location(
        file: &str,
        start_line: u32,
        end_line: u32,
        start_col: u32,
        end_col: u32,
    ) -> SourceLocation {
        SourceLocation {
            file: ProjectPath::new(std::path::PathBuf::from(file)),
            start_line: LineNumber::new(start_line).unwrap(),
            end_line: LineNumber::new(end_line).unwrap(),
            start_column: Some(ColumnNumber::new(start_col).unwrap()),
            end_column: Some(ColumnNumber::new(end_col).unwrap()),
        }
    }

    fn generate_test_facts(taint_count: usize, func_count: usize) -> Vec<Fact> {
        let mut facts = Vec::new();
        let flow_id = FlowId::new_uuid();

        // Create taint sources
        for i in 0..taint_count {
            let provenance = Provenance::new(
                ExtractorId::new("test".to_string()),
                "1.0.0".to_string(),
                Confidence::new(0.8).unwrap(),
            );

            facts.push(Fact {
                id: FactId::new(),
                fact_type: FactType::TaintSource {
                    var: VariableName(format!("taint{}", i)),
                    flow_id,
                    source_type: "test".to_string(),
                    confidence: Confidence::new(0.8).unwrap(),
                },
                location: create_test_location("test.rs", i as u32, (i + 10) as u32, 0, 100),
                provenance,
            });
        }

        // Create functions
        for i in 0..func_count {
            let provenance = Provenance::new(
                ExtractorId::new("test".to_string()),
                "1.0.0".to_string(),
                Confidence::new(0.8).unwrap(),
            );

            facts.push(Fact {
                id: FactId::new(),
                fact_type: FactType::Function {
                    name: format!("func{}", i),
                    complexity: (i % 5) as u32,
                    lines_of_code: (i as u32 % 50) + 10,
                },
                location: create_test_location(
                    "test.rs",
                    (i + 100) as u32,
                    (i + 120) as u32,
                    0,
                    100,
                ),
                provenance,
            });
        }

        facts
    }

    fn generate_test_facts_with_location(count: usize) -> Vec<Fact> {
        (0..count)
            .map(|i| {
                let provenance = Provenance::new(
                    ExtractorId::new("test".to_string()),
                    "1.0.0".to_string(),
                    Confidence::new(0.8).unwrap(),
                );

                Fact {
                    id: FactId::new(),
                    fact_type: FactType::Function {
                        name: format!("func{}", i),
                        complexity: 1,
                        lines_of_code: 10,
                    },
                    location: create_test_location("test.rs", i as u32, (i + 10) as u32, 0, 100),
                    provenance,
                }
            })
            .collect()
    }

    fn generate_test_facts_with_flow(flow_id: FlowId, count: usize) -> Vec<Fact> {
        (0..count)
            .map(|i| {
                let provenance = Provenance::new(
                    ExtractorId::new("test".to_string()),
                    "1.0.0".to_string(),
                    Confidence::new(0.8).unwrap(),
                );

                let fact_type = if i == 0 {
                    FactType::TaintSource {
                        var: VariableName(format!("source{}", i)),
                        flow_id,
                        source_type: "test".to_string(),
                        confidence: Confidence::new(0.8).unwrap(),
                    }
                } else {
                    FactType::TaintSink {
                        func: FunctionName(format!("sink{}", i)),
                        consumes_flow: flow_id,
                        category: "test".to_string(),
                        severity: Severity::Major,
                    }
                };

                Fact {
                    id: FactId::new(),
                    fact_type,
                    location: create_test_location("test.rs", i as u32, (i + 10) as u32, 0, 100),
                    provenance,
                }
            })
            .collect()
    }
}
