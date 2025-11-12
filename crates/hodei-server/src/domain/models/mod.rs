use serde::{Deserialize, Serialize};

pub mod analysis;
pub mod baseline;
/// Domain models - Pure business entities
///
/// These are simple data structures that contain business logic.
/// They have no dependencies on external systems or frameworks.
pub mod finding;

pub use analysis::*;
pub use baseline::*;
/// Re-export all models
pub use finding::*;

use uuid::Uuid;

/// Project identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectId(pub String);

/// Analysis identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AnalysisId(pub Uuid);

/// User identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub Uuid);

impl ProjectId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AnalysisId {
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }
}

impl UserId {
    pub fn new_v4() -> Self {
        Self(Uuid::new_v4())
    }
}
