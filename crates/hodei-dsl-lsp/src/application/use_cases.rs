//! Use cases
//!
//! Application use cases that orchestrate domain services

use crate::domain::models::{
    CompletionContext, CompletionItem, CursorPosition, Diagnostic, Document, HoverInfo,
};
use crate::domain::ports::{
    CompletionProvider as CompletionProviderPort, DocumentRepository,
    HoverProvider as HoverProviderPort,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Use case: Get autocompletion suggestions
pub struct GetCompletionsUseCase {
    completion_provider: Arc<dyn CompletionProviderPort>,
    document_repository: Arc<RwLock<HashMap<String, Document>>>,
}

impl GetCompletionsUseCase {
    pub fn new(
        completion_provider: Arc<dyn CompletionProviderPort>,
        document_repository: Arc<RwLock<HashMap<String, Document>>>,
    ) -> Self {
        Self {
            completion_provider,
            document_repository,
        }
    }

    pub async fn execute(
        &self,
        uri: &str,
        position: CursorPosition,
        trigger_character: Option<char>,
    ) -> Result<Vec<CompletionItem>, String> {
        let document_repo = self.document_repository.read().await;
        let document = document_repo
            .get(uri)
            .ok_or_else(|| format!("Document not found: {}", uri))?;

        let context = CompletionContext {
            position,
            trigger_character,
            trigger_kind: if trigger_character.is_some() {
                crate::domain::models::CompletionTriggerKind::TriggerCharacter
            } else {
                crate::domain::models::CompletionTriggerKind::Invoked
            },
        };

        self.completion_provider
            .provide_completions(&document, &context)
            .await
    }
}

/// Use case: Get hover information
pub struct GetHoverInfoUseCase {
    hover_provider: Arc<dyn HoverProviderPort>,
    document_repository: Arc<RwLock<HashMap<String, Document>>>,
}

impl GetHoverInfoUseCase {
    pub fn new(
        hover_provider: Arc<dyn HoverProviderPort>,
        document_repository: Arc<RwLock<HashMap<String, Document>>>,
    ) -> Self {
        Self {
            hover_provider,
            document_repository,
        }
    }

    pub async fn execute(
        &self,
        uri: &str,
        position: CursorPosition,
    ) -> Result<Option<HoverInfo>, String> {
        let document_repo = self.document_repository.read().await;
        let document = document_repo
            .get(uri)
            .ok_or_else(|| format!("Document not found: {}", uri))?;

        self.hover_provider.provide_hover(&document, position).await
    }
}

/// Use case: Validate document
pub struct ValidateDocumentUseCase<SemanticAnalyzer> {
    analyzer: Arc<SemanticAnalyzer>,
    document_repository: Arc<RwLock<HashMap<String, Document>>>,
}

impl<SemanticAnalyzer: crate::domain::ports::SemanticAnalyzer>
    ValidateDocumentUseCase<SemanticAnalyzer>
{
    pub fn new(
        analyzer: Arc<SemanticAnalyzer>,
        document_repository: Arc<RwLock<HashMap<String, Document>>>,
    ) -> Self {
        Self {
            analyzer,
            document_repository,
        }
    }

    pub async fn execute(&self, uri: &str) -> Result<Vec<Diagnostic>, String> {
        let document_repo = self.document_repository.read().await;
        let document = document_repo
            .get(uri)
            .ok_or_else(|| format!("Document not found: {}", uri))?;

        // Parse the document first
        let ast = match hodei_dsl::parser::parse_file(&document.content) {
            Ok(ast) => ast,
            Err(e) => return Err(format!("Parse error: {:?}", e)),
        };

        // Then analyze it
        Ok(self.analyzer.analyze(&ast).await)
    }
}
