//! Unit tests for use cases

use hodei_dsl_lsp::application::use_cases::{
    GetCompletionsUseCase, GetHoverInfoUseCase, ValidateDocumentUseCase
};
use hodei_dsl_lsp::domain::models::{CursorPosition, Document, CompletionTriggerKind};
use hodei_dsl_lsp::domain::ports::{
    CompletionProvider as CompletionProviderPort,
    HoverProvider as HoverProviderPort,
    SemanticAnalyzer
};
use crate::fixtures::{create_basic_document, MockCompletionProvider, MockHoverProvider, MockDocumentRepository};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_get_completions_use_case() {
    let document = create_basic_document();
    let doc_repo = Arc::new(RwLock::new(MockDocumentRepository::new()));
    doc_repo.write().await.store(document.clone()).await;
    
    let mock_provider = MockCompletionProvider::new()
        .with_completion(hodei_dsl_lsp::domain::models::CompletionItem {
            label: "Vulnerability".to_string(),
            kind: hodei_dsl_lsp::domain::models::CompletionItemKind::Class,
            detail: Some("Security vulnerability".to_string()),
            documentation: None,
            insert_text: "Vulnerability".to_string(),
            additional_text_edits: Vec::new(),
        });
    
    let use_case = GetCompletionsUseCase::new(
        Arc::new(mock_provider),
        doc_repo,
    );
    
    let completions = use_case.execute(
        &document.uri,
        CursorPosition { line: 0, column: 0 },
        None,
    )
    .await
    .expect("Should get completions");
    
    assert!(!completions.is_empty());
}

#[tokio::test]
async fn test_get_completions_nonexistent_document() {
    let doc_repo = Arc::new(RwLock::new(MockDocumentRepository::new()));
    
    let mock_provider = MockCompletionProvider::new();
    
    let use_case = GetCompletionsUseCase::new(
        Arc::new(mock_provider),
        doc_repo,
    );
    
    let result = use_case.execute(
        "file:///nonexistent.hodei",
        CursorPosition { line: 0, column: 0 },
        None,
    )
    .await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_hover_info_use_case() {
    let document = create_basic_document();
    let doc_repo = Arc::new(RwLock::new(MockDocumentRepository::new()));
    doc_repo.write().await.store(document.clone()).await;
    
    let hover_info = hodei_dsl_lsp::domain::models::HoverInfo {
        contents: "Test hover".to_string(),
        range: None,
    };
    
    let mock_provider = MockHoverProvider::new()
        .with_hover_info(hover_info);
    
    let use_case = GetHoverInfoUseCase::new(
        Arc::new(mock_provider),
        doc_repo,
    );
    
    let hover = use_case.execute(
        &document.uri,
        CursorPosition { line: 0, column: 0 },
    )
    .await
    .expect("Should get hover info");
    
    assert!(hover.is_some());
}

#[tokio::test]
async fn test_get_hover_info_nonexistent_document() {
    let doc_repo = Arc::new(RwLock::new(MockDocumentRepository::new()));
    
    let mock_provider = MockHoverProvider::new();
    
    let use_case = GetHoverInfoUseCase::new(
        Arc::new(mock_provider),
        doc_repo,
    );
    
    let result = use_case.execute(
        "file:///nonexistent.hodei",
        CursorPosition { line: 0, column: 0 },
    )
    .await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_validate_document_use_case() {
    struct MockSemanticValidator {
        diagnostics: Vec<hodei_dsl_lsp::domain::models::Diagnostic>,
    }
    
    impl MockSemanticValidator {
        fn new() -> Self {
            Self {
                diagnostics: Vec::new(),
            }
        }
    }
    
    impl MockSemanticValidator {
        async fn validate_document(&self, _document: &Document) -> Vec<hodei_dsl_lsp::domain::models::Diagnostic> {
            self.diagnostics.clone()
        }
    }
    
    let document = create_basic_document();
    let doc_repo = Arc::new(RwLock::new(MockDocumentRepository::new()));
    doc_repo.write().await.store(document.clone()).await;
    
    let validator = Arc::new(MockSemanticValidator::new());
    
    let use_case = ValidateDocumentUseCase::new(
        validator,
        doc_repo,
    );
    
    let diagnostics = use_case.execute(&document.uri)
        .await
        .expect("Should validate document");
    
    assert!(diagnostics.is_empty());
}

#[tokio::test]
async fn test_validate_document_nonexistent() {
    struct MockSemanticValidator;
    
    impl MockSemanticValidator {
        async fn validate_document(&self, _document: &Document) -> Vec<hodei_dsl_lsp::domain::models::Diagnostic> {
            Vec::new()
        }
    }
    
    let doc_repo = Arc::new(RwLock::new(MockDocumentRepository::new()));
    
    let validator = Arc::new(MockSemanticValidator);
    
    let use_case = ValidateDocumentUseCase::new(
        validator,
        doc_repo,
    );
    
    let result = use_case.execute("file:///nonexistent.hodei")
        .await;
    
    assert!(result.is_err());
}

#[tokio::test]
async fn test_use_case_with_trigger_character() {
    let document = create_basic_document();
    let doc_repo = Arc::new(RwLock::new(MockDocumentRepository::new()));
    doc_repo.write().await.store(document.clone()).await;
    
    let mock_provider = MockCompletionProvider::new()
        .with_completion(hodei_dsl_lsp::domain::models::CompletionItem {
            label: "test".to_string(),
            kind: hodei_dsl_lsp::domain::models::CompletionItemKind::Class,
            detail: None,
            documentation: None,
            insert_text: "test".to_string(),
            additional_text_edits: Vec::new(),
        });
    
    let use_case = GetCompletionsUseCase::new(
        Arc::new(mock_provider),
        doc_repo,
    );
    
    let completions = use_case.execute(
        &document.uri,
        CursorPosition { line: 0, column: 0 },
        Some('.'),
    )
    .await
    .expect("Should get completions");
    
    assert!(!completions.is_empty());
}
