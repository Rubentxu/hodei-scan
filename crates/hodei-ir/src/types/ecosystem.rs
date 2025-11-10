//! Ecosystem types for dependency management

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Ecosystem {
    Npm,
    Cargo,
    Maven,
    Gradle,
    PyPI,
    NuGet,
    Go,
    RubyGems,
    Composer,
}

impl Ecosystem {
    pub fn package_manager(&self) -> &'static str {
        match self {
            Self::Npm => "npm",
            Self::Cargo => "cargo",
            Self::Maven => "mvn",
            Self::Gradle => "gradle",
            Self::PyPI => "pip",
            Self::NuGet => "nuget",
            Self::Go => "go",
            Self::RubyGems => "gem",
            Self::Composer => "composer",
        }
    }
}

impl fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.package_manager())
    }
}
