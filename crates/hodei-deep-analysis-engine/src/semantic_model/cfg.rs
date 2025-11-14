//! Control Flow Graph structures

use petgraph::graph::Graph;

/// Control Flow Graph
pub type ControlFlowGraph = Graph<BasicBlock, ControlFlowEdge>;

/// Edge in control flow graph
#[derive(Debug, Clone)]
pub struct ControlFlowEdge {
    // TODO: Add edge metadata
}

/// Basic block in CFG
#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: u32,
    pub start_line: u32,
    pub end_line: u32,
}

/// Builder for CFG
#[derive(Debug, Default)]
pub struct ControlFlowGraphBuilder {}

impl ControlFlowGraphBuilder {
    pub fn new() -> Self {
        Self::default()
    }
}
