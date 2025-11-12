//! Hodei DSL Language Server
//!
//! Main server implementation using tower-lsp

use crate::domain::models::{CompletionContext, CursorPosition, Document};
use crate::infrastructure::adapters::{
    HodeiCompletionProvider, HodeiHoverProvider, HodeiSemanticAnalyzer, InMemoryDocumentRepository,
};
use lsp_types::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::*;
use tower_lsp::{LanguageServer, LspService};

/// Hodei DSL Language Server implementation
pub struct HodeiDslServer {
    document_repository: Arc<RwLock<InMemoryDocumentRepository>>,
    completion_provider: Arc<HodeiCompletionProvider>,
    hover_provider: Arc<HodeiHoverProvider>,
    semantic_analyzer: Arc<HodeiSemanticAnalyzer>,
}

impl HodeiDslServer {
    /// Create a new server instance
    pub fn new() -> Self {
        Self {
            document_repository: Arc::new(RwLock::new(InMemoryDocumentRepository::new())),
            completion_provider: Arc::new(HodeiCompletionProvider::new()),
            hover_provider: Arc::new(HodeiHoverProvider::new()),
            semantic_analyzer: Arc::new(HodeiSemanticAnalyzer::new()),
        }
    }

    /// Convert LSP Position to CursorPosition
    fn position_from_lsp(pos: Position) -> CursorPosition {
        CursorPosition {
            line: pos.line,
            column: pos.character,
        }
    }

    /// Convert CursorPosition to LSP Position
    fn position_to_lsp(pos: CursorPosition) -> Position {
        Position::new(pos.line, pos.column)
    }

    /// Get document from URI
    async fn get_document(&self, uri: Url) -> Option<Document> {
        let repo = self.document_repository.read().await;
        repo.get(uri.as_str()).await
    }

    /// Store or update document
    async fn store_document(&self, document: Document) {
        let repo = self.document_repository.read().await;
        repo.store(document).await;
    }

    /// Remove document
    async fn remove_document(&self, uri: Url) {
        let repo = self.document_repository.read().await;
        repo.remove(uri.as_str()).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for HodeiDslServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "hodei-dsl-language-server".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string()]),
                    resolve_provider: Some(false),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                diagnostic_provider: Some(DiagnosticRegistrationOptions {
                    identifier: Some("hodei-dsl".to_string()),
                    document_selector: Some(vec![DocumentFilter {
                        language: Some("hodei-dsl".to_string()),
                        pattern: None,
                        scheme: None,
                    }]),
                    inter_file_dependencies: false,
                    workspace_diagnostics: false,
                }),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::Full,
                )),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("hodei-dsl language server initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("hodei-dsl language server shutting down");
        Ok(())
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let position = Self::position_from_lsp(params.text_document_position.position);
        let uri = params.text_document_position.text_document.uri;

        let document = match self.get_document(uri).await {
            Some(doc) => doc,
            None => return Ok(None),
        };

        let context = CompletionContext {
            position,
            trigger_character: params.context.and_then(|c| c.trigger_character),
            trigger_kind: if params.context.is_some() {
                crate::domain::models::CompletionTriggerKind::TriggerForIncompleteCompletions
            } else {
                crate::domain::models::CompletionTriggerKind::Invoked
            },
        };

        match self
            .completion_provider
            .provide_completions(&document, &context)
            .await
        {
            Ok(completions) => {
                let lsp_completions: Vec<CompletionItem> = completions
                    .into_iter()
                    .map(|c| CompletionItem {
                        label: c.label,
                        kind: Some(match c.kind {
                            crate::domain::models::CompletionItemKind::Class => {
                                lsp_types::CompletionItemKind::CLASS
                            }
                            crate::domain::models::CompletionItemKind::Function => {
                                lsp_types::CompletionItemKind::FUNCTION
                            }
                            crate::domain::models::CompletionItemKind::Variable => {
                                lsp_types::CompletionItemKind::VARIABLE
                            }
                            crate::domain::models::CompletionItemKind::Keyword => {
                                lsp_types::CompletionItemKind::KEYWORD
                            }
                            crate::domain::models::CompletionItemKind::Snippet => {
                                lsp_types::CompletionItemKind::SNIPPET
                            }
                        }),
                        detail: c.detail,
                        documentation: c.documentation,
                        insert_text: Some(c.insert_text),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        text_edit: None,
                        additional_text_edits: None,
                        command: None,
                        data: None,
                        sort_text: None,
                        filter_text: None,
                        preselect: None,
                        commit_characters: None,
                        deprecated: None,
                        tags: None,
                        label_details: None,
                        resolve_provider: None,
                    })
                    .collect();

                Ok(Some(CompletionResponse::Array(lsp_completions)))
            }
            Err(e) => {
                tracing::error!("Completion error: {}", e);
                Ok(None)
            }
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<HoverResponse>> {
        let position = Self::position_from_lsp(params.text_document_position.position);
        let uri = params.text_document_position.text_document.uri;

        let document = match self.get_document(uri).await {
            Some(doc) => doc,
            None => return Ok(None),
        };

        match self.hover_provider.provide_hover(&document, position).await {
            Ok(hover_info) => {
                if let Some(hover) = hover_info {
                    Ok(Some(HoverResponse::Scalar(Hover {
                        contents: lsp_types::HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: hover.contents,
                        }),
                        range: hover.range.map(|r| {
                            lsp_types::Range::new(
                                Self::position_to_lsp(r.start),
                                Self::position_to_lsp(r.end),
                            )
                        }),
                    })))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                tracing::error!("Hover error: {}", e);
                Ok(None)
            }
        }
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let document = Document {
            uri: params.text_document.uri.clone(),
            content: params.text_document.text,
            version: params.text_document.version,
        };

        self.store_document(document).await;

        // Trigger semantic validation
        self.did_change(DidChangeTextDocumentParams {
            text_document: VersionedTextDocumentIdentifier {
                uri: params.text_document.uri.to_string(),
                version: params.text_document.version,
            },
            content_changes: vec![],
        })
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // In a full implementation, we would:
        // 1. Update the document
        // 2. Parse and analyze it
        // 3. Publish diagnostics

        // For now, just log the change
        tracing::info!(
            "Document changed: {} (v{})",
            params.text_document.uri.to_string(),
            params.text_document.version
        );
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.remove_document(params.text_document.uri).await;
    }
}

/// Create and run the LSP server
pub async fn run_server() -> Result<()> {
    let service = LspService::new(HodeiDslServer::new());
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

    tracing_subscriber::fmt::init();

    service.new_connection(stdin, stdout).await;

    Ok(())
}
