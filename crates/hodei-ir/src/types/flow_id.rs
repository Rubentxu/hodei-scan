use serde::{Deserialize, Serialize};
use std::fmt;
use std::str;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FlowId(pub uuid::Uuid);

impl FlowId {
    pub fn new_uuid() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// Get string representation of the flow ID
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl fmt::Display for FlowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
