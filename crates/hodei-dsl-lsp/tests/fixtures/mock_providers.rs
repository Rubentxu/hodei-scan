//! Mock providers for testing
//!
//! Provides mock implementations of domain ports for testing

use hodei_dsl_lsp::domain::models::{
    Document, CompletionItem, CursorPosition, HoverInfo, Diagnostic, 
    CompletionContext, ExpectedFinding
};
use hodei_dsl_lsp::domain::ports::{
    DocumentRepository, CompletionProvider, HoverProvider, 
    SemanticAnalyzer, FactRepository, FunctionRepository
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock document repository for testing
#[derive(Default)]
pub struct MockDocumentRepository {
    documents: Arc<RwLock<HashMap<String, Document>>>,
}

impl MockDocumentRepository {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn with_document(mut self, doc: Document) -> Self {
        let uri = doc.uri.clone();
        let mut docs = HashMap::new();
        docs.insert(uri, doc);
        self.documents = Arc::new(RwLock::new(docs));
        self
    }
}

#[async_trait::async_trait]
impl DocumentRepository for MockDocumentRepository {
    async fn get(&self, uri: &str) -> Option<Document> {
        let documents = self.documents.read().await;
        documents.get(uri).cloned()
    }
    
    async fn store(&self, document: Document) {
        let mut documents = self.documents.write().await;
        documents.insert(document.uri.clone(), document);
    }
    
    async fn remove(&self, uri: &str) {
        let mut documents = self.documents.write().await;
        documents.remove(uri);
    }
    
    async fn get_all(&self) -> Vec<Document> {
        let documents = self.documents.read().await;
        documents.values().cloned().collect()
    }
}

/// Mock completion provider that returns predefined completions
pub struct MockCompletionProvider {
    completions: Vec<CompletionItem>,
}

impl MockCompletionProvider {
    pub fn new() -> Self {
        Self {
            completions: Vec::new(),
        }
    }
    
    pub fn with_completion(mut self, completion: CompletionItem) -> Self {
        self.completions.push(completion);
        self
    }
    
    pub fn with_completions(mut self, completions: Vec<CompletionItem>) -> Self {
        self.completions = completions;
        self
    }
}

#[async_trait::async_trait]
impl CompletionProvider for MockCompletionProvider {
    async fn provide_completions(
        &self,
        _document: &Document,
        _context: &CompletionContext,
    ) -> Result<Vec<CompletionItem>, String> {
        Ok(self.completions.clone())
    }
}

/// Mock hover provider that returns predefined hover info
pub struct MockHoverProvider {
    hover_info: Option<HoverInfo>,
}

impl MockHoverProvider {
    pub fn new() -> Self {
        Self { hover_info: None }
    }
    
    pub fn with_hover_info(mut self, hover_info: HoverInfo) -> Self {
        self.hover_info = Some(hover_info);
        self
    }
}

#[async_trait::async_trait]
impl HoverProvider for MockHoverProvider {
    async fn provide_hover(
        &self,
        _document: &Document,
        _position: CursorPosition,
    ) -> Result<Option<HoverInfo>, String> {
        Ok(self.hover_info.clone())
    }
}

/// Mock semantic analyzer that returns predefined diagnostics
pub struct MockSemanticAnalyzer {
    diagnostics: Vec<Diagnostic>,
}

impl MockSemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
        }
    }
    
    pub fn with_diagnostic(mut self, diagnostic: Diagnostic) -> Self {
        self.diagnostics.push(diagnostic);
        self
    }
    
    pub fn with_diagnostics(mut self, diagnostics: Vec<Diagnostic>) -> Self {
        self.diagnostics = diagnostics;
        self
    }
}

#[async_trait::async_trait]
impl SemanticAnalyzer for MockSemanticAnalyzer {
    async fn analyze(&self, _ast: &hodei_dsl::ast::RuleFile) -> Vec<Diagnostic> {
        self.diagnostics.clone()
    }
}

/// Mock fact repository
pub struct MockFactRepository {
    facts: HashMap<String, String>,
}

impl MockFactRepository {
    pub fn new() -> Self {
        let mut facts = HashMap::new();
        facts.insert("Vulnerability".to_string(), "Security vulnerability".to_string());
        facts.insert("CodeSmell".to_string(), "Code quality issue".to_string());
        facts.insert("SecurityIssue".to_string(), "Security issue".to_string());
        Self { facts }
    }
    
    pub fn with_fact(mut self, name: &str, description: &str) -> Self {
        self.facts.insert(name.to_string(), description.to_string());
        self
    }
}

#[async_trait::async_trait]
impl FactRepository for MockFactRepository {
    async fn get_fact_doc(&self, name: &str) -> Option<hodei_dsl_lsp::domain::models::FactDocumentation> {
        self.facts.get(name).map(|desc| hodei_dsl_lsp::domain::models::FactDocumentation {
            name: name.to_string(),
            description: desc.clone(),
            fields: HashMap::new(),
        })
    }
    
    async fn get_all_facts(&self) -> Vec<String> {
        self.facts.keys().cloned().collect()
    }
    
    async fn fact_exists(&self, name: &str) -> bool {
        self.facts.contains_key(name)
    }
}

/// Mock function repository
pub struct MockFunctionRepository {
    functions: HashMap<String, String>,
}

impl MockFunctionRepository {
    pub fn new() -> Self {
        let mut functions = HashMap::new();
        functions.insert("matches".to_string(), "Pattern matching".to_string());
        functions.insert("contains".to_string(), "Substring check".to_string());
        functions.insert("length_gt".to_string(), "Length greater than".to_string());
        Self { functions }
    }
}

#[async_trait::async_trait]
impl FunctionRepository for MockFunctionRepository {
    async fn get_function_doc(&self, name: &str) -> Option<hodei_dsl_lsp::domain::models::FunctionDocumentation> {
        self.functions.get(name).map(|desc| hodei_dsl_lsp::domain::models::FunctionDocumentation {
            name: name.to_string(),
            description: desc.clone(),
            usage: format!("{}(...)", name),
            parameters: Vec::new(),
        })
    }
    
    async fn get_all_functions(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }
    
    async fn function_exists(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
}
