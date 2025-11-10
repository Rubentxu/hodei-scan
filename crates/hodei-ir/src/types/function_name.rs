//! Function name type

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FunctionName(String);

impl FunctionName {
    pub fn new(name: String) -> Self {
        Self(name)
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for FunctionName {
    fn from(name: String) -> Self {
        Self::new(name)
    }
}

impl fmt::Display for FunctionName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
