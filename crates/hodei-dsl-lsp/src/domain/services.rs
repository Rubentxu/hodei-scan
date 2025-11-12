//! Domain services
//!
//! Core business logic for LSP features

use crate::domain::models::{
    CompletionContext, CompletionItem, CursorPosition, Diagnostic, Document, 
    FactDocumentation, HoverInfo
};
use crate::domain::ports::{
    FactRepository, FunctionRepository, SemanticAnalyzer, DslParser,
};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::sync::Arc;

/// Service for semantic validation of DSL code
pub struct SemanticValidationService<Parser, Analyzer> {
    parser: Arc<Parser>,
    analyzer: Arc<Analyzer>,
    fact_cache: Arc<RwLock<HashMap<String, FactDocumentation>>>,
}

impl<Parser, Analyzer> SemanticValidationService<Parser, Analyzer>
where
    Parser: DslParser,
    Analyzer: SemanticAnalyzer,
{
    pub fn new(parser: Arc<Parser>, analyzer: Arc<Analyzer>) -> Self {
        let fact_cache = Arc::new(RwLock::new(HashMap::new()));
        
        // Pre-populate cache with built-in facts
        let mut cache = HashMap::new();
        cache.insert(
            "Vulnerability".to_string(),
            FactDocumentation {
                name: "Vulnerability".to_string(),
                description: "Represents a security vulnerability detected in the code".to_string(),
                fields: {
                    let mut fields = HashMap::new();
                    fields.insert(
                        "severity".to_string(),
                        crate::domain::models::FieldDocumentation {
                            name: "severity".to_string(),
                            field_type: "Severity".to_string(),
                            description: "Severity level of the vulnerability (Critical, Major, Minor)".to_string(),
                        }
                    );
                    fields.insert(
                        "message".to_string(),
                        crate::domain::models::FieldDocumentation {
                            name: "message".to_string(),
                            field_type: "String".to_string(),
                            description: "Human-readable description of the vulnerability".to_string(),
                        }
                    );
                    fields
                },
            },
        );
        
        SemanticValidationService {
            parser,
            analyzer,
            fact_cache: Arc::new(RwLock::new(cache)),
        }
    }
    
    pub async fn validate_document(&self, document: &Document) -> Vec<Diagnostic> {
        match self.parser.parse(&document.content).await {
            Ok(ast) => self.analyzer.analyze(&ast).await,
            Err(error) => vec![Diagnostic {
                range: crate::domain::models::Range {
                    start: CursorPosition { line: 0, column: 0 },
                    end: CursorPosition { line: 0, column: 0 },
                },
                severity: crate::domain::models::DiagnosticSeverity::Error,
                message: format!("Parse error: {}", error),
                source: "hodei-dsl".to_string(),
            }],
        }
    }
}

impl<Parser, Analyzer> SemanticValidationService<Parser, Analyzer> {
    pub async fn get_fact_completions(&self) -> Vec<CompletionItem> {
        let cache = self.fact_cache.read().await;
        
        cache.values()
            .map(|fact| CompletionItem {
                label: fact.name.clone(),
                kind: crate::domain::models::CompletionItemKind::Class,
                detail: Some(fact.description.clone()),
                documentation: Some(format!(
                    "# {}\n\n{}\n\n## Fields\n{}",
                    fact.name,
                    fact.description,
                    fact.fields.values()
                        .map(|f| format!("- `{}` ({}): {}", f.name, f.field_type, f.description))
                        .collect::<Vec<_>>()
                        .join("\n")
                )),
                insert_text: format!("{} {{", fact.name),
                additional_text_edits: Vec::new(),
            })
            .collect()
    }
}

/// Service for providing completion suggestions
pub struct CompletionService<Provider> {
    provider: Arc<Provider>,
}

impl<Provider> CompletionService<Provider>
where
    Provider: crate::domain::ports::CompletionProvider,
{
    pub fn new(provider: Arc<Provider>) -> Self {
        CompletionService { provider }
    }
    
    pub async fn get_completions(
        &self,
        document: &Document,
        context: &CompletionContext,
    ) -> Result<Vec<CompletionItem>, String> {
        self.provider.provide_completions(document, context).await
    }
}

/// Service for providing hover information
pub struct HoverService<Provider> {
    provider: Arc<Provider>,
}

impl<Provider> HoverService<Provider>
where
    Provider: crate::domain::ports::HoverProvider,
{
    pub fn new(provider: Arc<Provider>) -> Self {
        HoverService { provider }
    }
    
    pub async fn get_hover(
        &self,
        document: &Document,
        position: CursorPosition,
    ) -> Result<Option<HoverInfo>, String> {
        self.provider.provide_hover(document, position).await
    }
}
