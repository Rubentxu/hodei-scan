//! Scope Tree structures

/// Scope ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub u32);

/// Scope node in tree
#[derive(Debug, Clone)]
pub struct ScopeNode {
    pub id: ScopeId,
    pub parent: Option<ScopeId>,
    pub name: String,
    pub depth: u32,
}

/// Scope tree
#[derive(Debug, Default)]
pub struct ScopeTree {
    // TODO: Implement tree structure
}

impl ScopeTree {
    pub fn new() -> Self {
        Self::default()
    }
}
