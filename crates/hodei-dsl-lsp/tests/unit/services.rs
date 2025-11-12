//! Unit tests for domain services

use hodei_dsl_lsp::domain::services::{
    SemanticValidationService, CompletionService, HoverService
};
use hodei_dsl_lsp::domain::models::{Document, CursorPosition, CompletionContext, CompletionTriggerKind};
use crate::fixtures::{create_basic_document, MockDocumentRepository};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_semantic_validation_service_initialization() {
    struct MockParser;
    
    #[async_trait::async_trait]
    impl hodei_dsl_lsp::domain::ports::DslParser for MockParser {
        async fn parse(&self, _content: &str) -> Result<hodei_dsl::ast::RuleFile, String> {
            Ok(hodei_dsl::ast::RuleFile { rules: Vec::new() })
        }
    }
    
    struct MockAnalyzer;
    
    #[async_trait::async_trait]
    impl hodei_dsl_lsp::domain::ports::SemanticAnalyzer for MockAnalyzer {
        async fn analyze(&self, _ast: &hodei_dsl::ast::RuleFile) -> Vec<hodei_dsl_lsp::domain::models::Diagnostic> {
            Vec::new()
        }
    }
    
    let parser = Arc::new(MockParser);
    let analyzer = Arc::new(MockAnalyzer);
    
    let service = SemanticValidationService::new(parser, analyzer);
    
    // Service should initialize successfully
    assert!(true);
}

#[tokio::test]
async fn test_semantic_validation_service_validate_document() {
    struct MockParser;
    
    #[async_trait::async_trait]
    impl hodei_dsl_lsp::domain::ports::DslParser for MockParser {
        async fn parse(&self, _content: &str) -> Result<hodei_dsl::ast::RuleFile, String> {
            Ok(hodei_dsl::ast::RuleFile { rules: Vec::new() })
        }
    }
    
    struct MockAnalyzer;
    
    #[async_trait::async_trait]
    impl hodei_dsl_lsp::domain::ports::SemanticAnalyzer for MockAnalyzer {
        async fn analyze(&self, _ast: &hodei_dsl::ast::RuleFile) -> Vec<hodei_dsl_lsp::domain::models::Diagnostic> {
            Vec::new()
        }
    }
    
    let parser = Arc::new(MockParser);
    let analyzer = Arc::new(MockAnalyzer);
    let service = SemanticValidationService::new(parser, analyzer);
    
    let document = create_basic_document();
    
    let diagnostics = service.validate_document(&document).await;
    
    // Should validate without errors
    assert!(diagnostics.is_empty());
}

#[tokio::test]
async fn test_semantic_validation_service_get_fact_completions() {
    struct MockParser;
    
    #[async_trait::async_trait]
    impl hodei_dsl_lsp::domain::ports::DslParser for MockParser {
        async fn parse(&self, _content: &str) -> Result<hodei_dsl::ast::RuleFile, String> {
            Ok(hodei_dsl::ast::RuleFile { rules: Vec::new() })
        }
    }
    
    struct MockAnalyzer;
    
    #[async_trait::async_trait]
    impl hodei_dsl_lsp::domain::ports::SemanticAnalyzer for MockAnalyzer {
        async fn analyze(&self, _ast: &hodei_dsl::ast::RuleFile) -> Vec<hodei_dsl_lsp::domain::models::Diagnostic> {
            Vec::new()
        }
    }
    
    let parser = Arc::new(MockParser);
    let analyzer = Arc::new(MockAnalyzer);
    let service = SemanticValidationService::new(parser, analyzer);
    
    let completions = service.get_fact_completions().await;
    
    // Should return fact type completions
    assert!(!completions.is_empty());
    
    // Should include built-in fact types
    let labels: Vec<String> = completions.iter().map(|c| c.label.clone()).collect();
    assert!(labels.contains(&"Vulnerability".to_string()));
    assert!(labels.contains(&"CodeSmell".to_string()));
}

#[tokio::test]
async fn test_completion_service() {
    struct MockCompletionProvider;
    
    #[async_trait::async_trait]
    impl hodei_dsl_lsp::domain::ports::CompletionProvider for MockCompletionProvider {
        async fn provide_completions(
            &self,
            _document: &Document,
            _context: &CompletionContext,
        ) -> Result<Vec<hodei_dsl_lsp::domain::models::CompletionItem>, String> {
            Ok(vec![
                hodei_dsl_lsp::domain::models::CompletionItem {
                    label: "test".to_string(),
                    kind: hodei_dsl_lsp::domain::models::CompletionItemKind::Class,
                    detail: None,
                    documentation: None,
                    insert_text: "test".to_string(),
                    additional_text_edits: Vec::new(),
                }
            ])
        }
    }
    
    let provider = Arc::new(MockCompletionProvider);
    let service = CompletionService::new(provider);
    
    let document = create_basic_document();
    let context = CompletionContext {
        position: CursorPosition { line: 0, column: 0 },
        trigger_character: None,
        trigger_kind: CompletionTriggerKind::Invoked,
    };
    
    let completions = service.get_completions(&document, &context)
        .await
        .expect("Should get completions");
    
    assert!(!completions.is_empty());
    assert_eq!(completions[0].label, "test");
}

#[tokio::test]
async fn test_hover_service() {
    struct MockHoverProvider;
    
    #[async_trait::async_trait]
    impl hodei_dsl_lsp::domain::ports::HoverProvider for MockHoverProvider {
        async fn provide_hover(
            &self,
            _document: &Document,
            _position: CursorPosition,
        ) -> Result<Option<hodei_dsl_lsp::domain::models::HoverInfo>, String> {
            Ok(Some(hodei_dsl_lsp::domain::models::HoverInfo {
                contents: "Test hover".to_string(),
                range: None,
            }))
        }
    }
    
    let provider = Arc::new(MockHoverProvider);
    let service = HoverService::new(provider);
    
    let document = create_basic_document();
    let position = CursorPosition { line: 0, column: 0 };
    
    let hover = service.get_hover(&document, position)
        .await
        .expect("Should get hover info");
    
    assert!(hover.is_some());
    assert_eq!(hover.unwrap().contents, "Test hover");
}
