//! hodei-extractors: Extractors for various code analysis tools
//!
//! This crate provides extractors for integrating with various code analysis
//! tools and generating IR facts from different sources.

#![warn(missing_docs)]

/// Base extractor trait
pub trait Extractor {
    /// Extract facts from source code
    fn extract(&self, _source: &str) -> Result<Vec<hodei_ir::Fact>, String> {
        Ok(Vec::new())
    }
}

/// TreeSitter extractor
#[derive(Debug)]
pub struct TreeSitterExtractor;

impl TreeSitterExtractor {
    /// Create a new TreeSitter extractor
    pub fn new() -> Self {
        Self
    }
}

impl Extractor for TreeSitterExtractor {
    fn extract(&self, source: &str) -> Result<Vec<hodei_ir::Fact>, String> {
        // Placeholder implementation
        println!("Extracting from: {} bytes of source", source.len());
        Ok(Vec::new())
    }
}
