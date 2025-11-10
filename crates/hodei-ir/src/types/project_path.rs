use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProjectPath {
    pub path: PathBuf,
}

impl ProjectPath {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn as_str(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }
}

impl fmt::Display for ProjectPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}
