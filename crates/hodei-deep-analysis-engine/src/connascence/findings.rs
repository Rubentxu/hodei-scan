//! Coupling findings

/// Entity identifier
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityId(pub String);

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Finding from connascence analysis
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CouplingFinding {
    pub entity: EntityId,
    pub connascence_type: super::types::ConnascenceType,
    pub strength: super::types::Strength,
    pub related_entities: Vec<EntityId>,
    pub message: String,
    pub remediation: String,
}
