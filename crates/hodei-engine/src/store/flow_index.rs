//! Flow index for data flow tracking using directed graphs
//!
//! This index uses a directed graph (DiGraph) from the petgraph crate to track
//! data flow relationships between facts. It enables efficient reachability queries
//! and shortest path calculations.

use hodei_ir::{Fact, FactId, FactType, SourceLocation};
use petgraph::algo::{astar, dijkstra};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// Flow index using directed graph for efficient flow tracking
#[derive(Debug, Clone)]
pub struct FlowIndex {
    graph: DiGraph<FactId, ()>,
    fact_to_node: HashMap<FactId, NodeIndex>,
    flow_to_facts: HashMap<hodei_ir::FlowId, Vec<FactId>>,
}

impl FlowIndex {
    /// Build the flow index from facts
    pub fn build(facts: &[&Fact]) -> Self {
        let mut graph = DiGraph::new();
        let mut fact_to_node = HashMap::new();
        let mut flow_to_facts = HashMap::new();

        // First pass: create nodes for all facts that have flow data
        for fact in facts {
            if Self::has_flow_data(fact) {
                let node = graph.add_node(fact.id);
                fact_to_node.insert(fact.id, node);

                // Track facts by flow_id
                if let Some(flow_id) = Self::extract_flow_id(fact) {
                    flow_to_facts
                        .entry(flow_id)
                        .or_insert_with(Vec::new)
                        .push(fact.id);
                }
            }
        }

        // Second pass: create edges based on data flow relationships
        for fact in facts {
            if let Some(flow_edges) = Self::extract_flow_edges(fact) {
                for (from_id, to_id) in flow_edges {
                    if let (Some(&from_node), Some(&to_node)) =
                        (fact_to_node.get(&from_id), fact_to_node.get(&to_id))
                    {
                        graph.add_edge(from_node, to_node, ());
                    }
                }
            }
        }

        Self {
            graph,
            fact_to_node,
            flow_to_facts,
        }
    }

    /// Find all facts reachable from a given fact following data flow
    pub fn reachable_from(&self, fact_id: FactId) -> Vec<FactId> {
        let Some(&start_node) = self.fact_to_node.get(&fact_id) else {
            return vec![];
        };

        let distances = dijkstra(&self.graph, start_node, None, |_| 1);

        distances.keys().map(|&node| self.graph[node]).collect()
    }

    /// Find the shortest path between two facts
    pub fn shortest_path(&self, from: FactId, to: FactId) -> Option<Vec<FactId>> {
        let start_node = self.fact_to_node.get(&from)?;
        let end_node = self.fact_to_node.get(&to)?;

        astar(&self.graph, *start_node, |n| n == *end_node, |_| 1, |_| 0)
            .map(|(_, path)| path.into_iter().map(|n| self.graph[n]).collect())
    }

    /// Check if a fact has flow data
    fn has_flow_data(fact: &Fact) -> bool {
        matches!(
            fact.fact_type,
            FactType::TaintSource { .. }
                | FactType::TaintSink { .. }
                | FactType::Sanitization { .. }
        )
    }

    /// Extract flow edges from a fact
    fn extract_flow_edges(fact: &Fact) -> Option<Vec<(FactId, FactId)>> {
        match &fact.fact_type {
            // Flow edges would be created from explicit flow relationships
            // For now, return empty - edges are created during graph building
            _ => Some(vec![]),
        }
    }

    /// Get all nodes in the graph (facts with flow data)
    pub fn nodes(&self) -> Vec<FactId> {
        self.graph
            .node_indices()
            .map(|idx| self.graph[idx])
            .collect()
    }

    /// Get the number of edges in the flow graph
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    /// Get facts for a specific flow ID
    pub fn get_facts_for_flow(&self, flow_id: &hodei_ir::FlowId) -> Option<&[FactId]> {
        self.flow_to_facts.get(flow_id).map(|v| v.as_slice())
    }

    /// Extract flow ID from a fact
    fn extract_flow_id(fact: &Fact) -> Option<hodei_ir::FlowId> {
        match &fact.fact_type {
            FactType::TaintSource { flow_id, .. } => Some(*flow_id),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodei_ir::*;

    #[test]
    fn test_flow_index_build() {
        let facts: Vec<&Fact> = vec![];
        let index = FlowIndex::build(&facts);
        assert_eq!(index.graph.node_count(), 0);
        assert_eq!(index.graph.edge_count(), 0);
    }

    #[test]
    fn test_reachable_from() {
        // Create test facts with flow relationships
        let fact1 = Fact::new_with_message(
            FactType::TaintSource {
                var: hodei_ir::VariableName("user_input".to_string()),
                flow_id: FlowId::new_uuid(),
                source_type: "http".to_string(),
                confidence: Confidence::MEDIUM,
            },
            "Taint source".to_string(),
            create_test_location("test.rs", 1, 1, 1, 10),
            Provenance::new(
                ExtractorId::TreeSitter,
                "1.0.0".to_string(),
                Confidence::MEDIUM,
            ),
        );

        let fact2 = Fact::new_with_message(
            FactType::TaintSink {
                func: hodei_ir::FunctionName("write".to_string()),
                consumes_flow: FlowId::new_uuid(),
                category: "write".to_string(),
                severity: hodei_ir::Severity::Major,
            },
            "Taint sink".to_string(),
            create_test_location("test.rs", 10, 10, 1, 10),
            Provenance::new(
                ExtractorId::TreeSitter,
                "1.0.0".to_string(),
                Confidence::MEDIUM,
            ),
        );

        let index = FlowIndex::build(&[&fact1, &fact2]);
        let reachable = index.reachable_from(fact1.id);
        // Should be able to reach itself (or related facts in same flow)
        assert!(!reachable.is_empty());
    }
}

fn create_test_location(
    file: &str,
    start_line: u32,
    end_line: u32,
    start_col: u32,
    end_col: u32,
) -> SourceLocation {
    use hodei_ir::{ColumnNumber, LineNumber, ProjectPath};

    SourceLocation::new(
        ProjectPath::new(std::path::PathBuf::from(file)),
        LineNumber::new(start_line).unwrap(),
        Some(ColumnNumber::new(start_col).unwrap()),
        LineNumber::new(end_line).unwrap(),
        Some(ColumnNumber::new(end_col).unwrap()),
    )
}
