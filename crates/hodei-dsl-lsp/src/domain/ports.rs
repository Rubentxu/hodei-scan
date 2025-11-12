//! Domain ports (interfaces)
//!
//! Define the contracts that the domain layer expects from external systems

use crate::domain::models::{
    CompletionContext, CompletionItem, CursorPosition, Diagnostic, Document, 
    FactDocumentation, FunctionDocumentation, HoverInfo
};
use std::result::Result;

/// Port: Repository for storing and retrieving open documents
#[async_trait::async_trait]
pub trait DocumentRepository: Send + Sync {
    /// Get a document by URI
    async fn get(&self, uri: &str) -> Option<Document>;
    
    /// Store or update a document
    async fn store(&self, document: Document);
    
    /// Remove a document
    async fn remove(&self, uri: &str);
    
    /// Get all open documents
    async fn get_all(&self) -> Vec<Document>;
}

/// Port: Repository for fact types and their documentation
#[async_trait::async_trait]
pub trait FactRepository: Send + Sync {
    /// Get documentation for a fact type
    async fn get_fact_doc(&self, name: &str) -> Option<FactDocumentation>;
    
    /// Get all available fact types
    async fn get_all_facts(&self) -> Vec<String>;
    
    /// Check if a fact type exists
    async fn fact_exists(&self, name: &str) -> bool;
}

/// Port: Repository for functions and their documentation
#[async_trait::async_trait]
pub trait FunctionRepository: Send + Sync {
    /// Get documentation for a function
    async fn get_function_doc(&self, name: &str) -> Option<FunctionDocumentation>;
    
    /// Get all available functions
    async fn get_all_functions(&self) -> Vec<String>;
    
    /// Check if a function exists
    async fn function_exists(&self, name: &str) -> bool;
}

/// Port: Emitter for diagnostics (errors, warnings, etc.)
#[async_trait::async_trait]
pub trait DiagnosticEmitter: Send + Sync {
    /// Emit a diagnostic message
    async fn emit(&self, diagnostic: Diagnostic);
}

/// Port: Parser for parsing DSL documents into AST
#[async_trait::async_trait]
pub trait DslParser: Send + Sync {
    /// Parse a document into an AST
    async fn parse(&self, content: &str) -> Result<hodei_dsl::ast::RuleFile, String>;
}

/// Port: Semantic analyzer for validating DSL code
#[async_trait::async_trait]
pub trait SemanticAnalyzer: Send + Sync {
    /// Analyze a parsed document and return diagnostics
    async fn analyze(&self, ast: &hodei_dsl::ast::RuleFile) -> Vec<Diagnostic>;
}

/// Port: Provider for completion suggestions
#[async_trait::async_trait]
pub trait CompletionProvider: Send + Sync {
    /// Get completions at a specific position in a document
    async fn provide_completions(
        &self,
        document: &Document,
        context: &CompletionContext,
    ) -> Result<Vec<CompletionItem>, String>;
}

/// Port: Provider for hover information
#[async_trait::async_trait]
pub trait HoverProvider: Send + Sync {
    /// Get hover information at a specific position
    async fn provide_hover(
        &self,
        document: &Document,
        position: CursorPosition,
    ) -> Result<Option<HoverInfo>, String>;
}
