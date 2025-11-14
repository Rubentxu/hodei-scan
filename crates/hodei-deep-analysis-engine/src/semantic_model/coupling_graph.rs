//! Coupling Graph structures

use petgraph::graph::Graph;

/// Coupling Graph
pub type CouplingGraph = Graph<CodeEntity, ConnascenceEdge>;

/// Code entity (node in coupling graph)
#[derive(Debug, Clone)]
pub struct CodeEntity {
    pub id: String,
    pub name: String,
    pub entity_type: String,
}

/// Edge representing connascence
#[derive(Debug, Clone)]
pub struct ConnascenceEdge {
    pub connascence_type: ConnascenceType,
    pub strength: Strength,
}

/// Type of connascence
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnascenceType {
    Name,
    Type,
    Meaning,
    Position,
    Algorithm,
}

/// Strength of connascence (1-5)
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Strength {
    VeryLow = 1,
    Low = 2,
    Medium = 3,
    High = 4,
    VeryHigh = 5,
}
