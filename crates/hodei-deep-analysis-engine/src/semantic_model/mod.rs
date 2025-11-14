//! Semantic Model - Rich context representation for code analysis
//!
//! This module provides structures for representing code semantics including
//! Control Flow Graphs (CFG), Data Flow Graphs (DFG), Scope Trees, and
//! Coupling Graphs for deep analysis.

pub mod builder;
pub mod cfg;
pub mod coupling_graph;
pub mod dfg;
pub mod fact_extractor;
pub mod scope_tree;

// Re-exports
pub use builder::{SemanticModel, SemanticModelBuilder};
pub use cfg::{BasicBlock, ControlFlowEdge, ControlFlowGraph, ControlFlowGraphBuilder};
pub use coupling_graph::{CodeEntity, ConnascenceEdge, ConnascenceType, CouplingGraph, Strength};
pub use dfg::{DataEdge, DataFlowGraph, DataFlowGraphBuilder, DataNode};
pub use fact_extractor::FactExtractor;
pub use scope_tree::{ScopeId, ScopeNode, ScopeTree};
