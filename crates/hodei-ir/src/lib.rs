//! hodei-ir: Intermediate Representation core types
//!
//! This crate provides the core data structures and types for representing
//! code analysis facts in a type-safe, language-agnostic format.

#![warn(missing_docs)]

/// Core fact types
pub mod types {
    /// Fact identifier
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct FactId {
        /// Unique identifier
        pub id: String,
    }

    impl FactId {
        /// Create a new fact ID
        pub fn new(id: String) -> Self {
            Self { id }
        }
    }

    /// Source code location
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SourceLocation {
        /// File path
        pub file: String,
        /// Line number (1-based)
        pub line: u32,
        /// Column number (1-based)
        pub column: Option<u32>,
    }

    impl SourceLocation {
        /// Create a new source location
        pub fn new(file: String, line: u32) -> Self {
            Self {
                file,
                line,
                column: None,
            }
        }
    }

    /// Fact type enum
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum FactType {
        /// Taint source (where data enters the system)
        TaintSource {
            /// Variable name
            var: String,
            /// Source type
            source_type: String,
        },
        /// Taint sink (where data is used dangerously)
        TaintSink {
            /// Function name
            func: String,
            /// Sink category
            category: String,
        },
    }
}

/// A single fact in the IR
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fact {
    /// Unique fact identifier
    pub id: types::FactId,
    /// Type of fact
    pub fact_type: types::FactType,
    /// Source location
    pub location: types::SourceLocation,
    /// When the fact was extracted
    pub extracted_at: chrono::DateTime<chrono::Utc>,
}

impl Fact {
    /// Create a new fact
    pub fn new(
        id: types::FactId,
        fact_type: types::FactType,
        location: types::SourceLocation,
    ) -> Self {
        Self {
            id,
            fact_type,
            location,
            extracted_at: chrono::Utc::now(),
        }
    }
}
