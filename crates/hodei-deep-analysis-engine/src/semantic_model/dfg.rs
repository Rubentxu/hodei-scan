//! Data Flow Graph structures

use petgraph::graph::Graph;

/// Data Flow Graph
pub type DataFlowGraph = Graph<DataNode, DataEdge>;

/// Edge in data flow graph
#[derive(Debug, Clone)]
pub struct DataEdge {
    // TODO: Add edge metadata
}

/// Node in data flow graph
#[derive(Debug, Clone)]
pub enum DataNode {
    /// Variable
    Variable {
        name: String,
        var_type: Option<String>,
        scope: u32,
    },
    /// Constant value
    Constant { value: String, var_type: String },
    /// Expression
    Expression {
        operation: String,
        operands: Vec<u32>,
    },
}

/// Builder for DFG
#[derive(Debug, Default)]
pub struct DataFlowGraphBuilder {}

impl DataFlowGraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}
