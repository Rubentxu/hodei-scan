//! Variable name type

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct VariableName(String);

impl VariableName {
    pub fn new(name: String) -> Self {
        Self(name)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for VariableName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl fmt::Display for VariableName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
