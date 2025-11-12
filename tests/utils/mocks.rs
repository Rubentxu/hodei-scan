//! Mock implementations for testing
//!
//! This module provides mock implementations of various services and adapters
//! to facilitate unit and integration testing

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock document repository for LSP testing
#[derive(Clone, Default)]
pub struct MockDocumentRepository {
    documents: Arc<RwLock<HashMap<String, String>>>,
}

impl MockDocumentRepository {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn store(&self, uri: String, content: String) {
        let mut docs = self.documents.write().await;
        docs.insert(uri, content);
    }

    pub async fn get(&self, uri: &str) -> Option<String> {
        let docs = self.documents.read().await;
        docs.get(uri).cloned()
    }

    pub async fn remove(&self, uri: &str) {
        let mut docs = self.documents.write().await;
        docs.remove(uri);
    }

    pub async fn clear(&self) {
        let mut docs = self.documents.write().await;
        docs.clear();
    }

    pub async fn count(&self) -> usize {
        let docs = self.documents.read().await;
        docs.len()
    }
}

/// Mock test runner for testing
#[derive(Default)]
pub struct MockTestRunner {
    pub last_config: Option<hodei_test::domain::models::TestConfig>,
    pub execution_count: usize,
    pub should_fail: bool,
}

impl MockTestRunner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_should_fail(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }

    pub async fn run(&mut self, config: hodei_test::domain::models::TestConfig) -> hodei_test::domain::models::TestResults {
        self.last_config = Some(config.clone());
        self.execution_count += 1;

        let mut results = hodei_test::domain::models::TestResults::new();

        for case in &config.cases {
            let passed = !self.should_fail && case.expected_findings.is_empty();
            let result = hodei_test::domain::models::TestCaseResult {
                name: case.name.clone(),
                passed,
                assertions: Vec::new(),
            };
            results.add_result(result);
        }

        results
    }

    pub fn was_executed(&self) -> bool {
        self.execution_count > 0
    }

    pub fn execution_count(&self) -> usize {
        self.execution_count
    }
}

/// Mock snapshot repository for testing
#[derive(Default)]
pub struct MockSnapshotRepository {
    pub snapshots: HashMap<String, String>,
    pub save_count: usize,
    pub load_count: usize,
}

impl MockSnapshotRepository {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn save(&mut self, test_name: String, snapshot: String) {
        self.snapshots.insert(test_name, snapshot);
        self.save_count += 1;
    }

    pub async fn load(&mut self, test_name: &str) -> Option<String> {
        self.load_count += 1;
        self.snapshots.get(test_name).cloned()
    }

    pub async fn delete(&mut self, test_name: &str) {
        self.snapshots.remove(test_name);
    }

    pub async fn clear(&mut self) {
        self.snapshots.clear();
    }

    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }
}

/// Mock IR reader for testing
#[derive(Default)]
pub struct MockIRReader {
    pub read_count: usize,
    pub should_fail: bool,
    pub last_path: Option<std::path::PathBuf>,
}

impl MockIRReader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_should_fail(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }

    pub async fn read(&mut self, path: &std::path::Path) -> Result<hodei_ir::FindingSet, Box<dyn std::error::Error>> {
        self.read_count += 1;
        self.last_path = Some(path.to_path_buf());

        if self.should_fail {
            return Err("Mock error".into());
        }

        // Return sample IR
        Ok(hodei_ir::FindingSet {
            findings: vec![
                hodei_ir::Finding {
                    fact_type: "Vulnerability".to_string(),
                    message: "Mock finding".to_string(),
                    location: Some("test.java:1".to_string()),
                    severity: Some("Major".to_string()),
                    metadata: HashMap::new(),
                }
            ],
        })
    }

    pub fn was_called(&self) -> bool {
        self.read_count > 0
    }
}

/// Mock IR formatter for testing
#[derive(Default)]
pub struct MockIRFormatter {
    pub format_count: usize,
    pub last_format: Option<hodei_ir::Format>,
    pub should_fail: bool,
}

impl MockIRFormatter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_should_fail(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }

    pub fn format(&mut self, ir: &hodei_ir::FindingSet, format: &hodei_ir::Format) -> Result<String, Box<dyn std::error::Error>> {
        self.format_count += 1;
        self.last_format = Some(format.clone());

        if self.should_fail {
            return Err("Mock error".into());
        }

        // Return formatted output based on format type
        match format {
            hodei_ir::Format::Json => Ok(r#"{"findings":[]}"#.to_string()),
            hodei_ir::Format::Yaml => Ok("findings: []".to_string()),
            hodei_ir::Format::Visual => Ok("IR Structure:\nTotal findings: 0".to_string()),
        }
    }

    pub fn was_called(&self) -> bool {
        self.format_count > 0
    }
}

/// Mock LSP server for testing
#[derive(Default)]
pub struct MockLspServer {
    pub initialize_count: usize,
    pub documents: HashMap<lsp_types::Url, String>,
    pub should_fail: bool,
}

impl MockLspServer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_should_fail(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }

    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.initialize_count += 1;

        if self.should_fail {
            return Err("Mock error".into());
        }

        Ok(())
    }

    pub async fn open_document(&mut self, uri: lsp_types::Url, content: String) {
        self.documents.insert(uri, content);
    }

    pub async fn close_document(&mut self, uri: &lsp_types::Url) {
        self.documents.remove(uri);
    }

    pub fn document_count(&self) -> usize {
        self.documents.len()
    }

    pub fn get_document(&self, uri: &lsp_types::Url) -> Option<&String> {
        self.documents.get(uri)
    }
}

/// Mock completion provider for testing
#[derive(Default)]
pub struct MockCompletionProvider {
    pub completion_count: usize,
    pub completions_to_return: Vec<hodei_dsl_lsp::domain::models::CompletionItem>,
}

impl MockCompletionProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_completions(mut self, completions: Vec<hodei_dsl_lsp::domain::models::CompletionItem>) -> Self {
        self.completions_to_return = completions;
        self
    }

    pub async fn provide_completions(
        &mut self,
        _document: &hodei_dsl_lsp::domain::models::Document,
        _context: &hodei_dsl_lsp::domain::models::CompletionContext,
    ) -> Result<Vec<hodei_dsl_lsp::domain::models::CompletionItem>, Box<dyn std::error::Error>> {
        self.completion_count += 1;
        Ok(self.completions_to_return.clone())
    }

    pub fn was_called(&self) -> bool {
        self.completion_count > 0
    }
}

/// Mock hover provider for testing
#[derive(Default)]
pub struct MockHoverProvider {
    pub hover_count: usize,
    pub hover_to_return: Option<hodei_dsl_lsp::domain::models::HoverInfo>,
}

impl MockHoverProvider {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_hover(mut self, hover: hodei_dsl_lsp::domain::models::HoverInfo) -> Self {
        self.hover_to_return = Some(hover);
        self
    }

    pub async fn provide_hover(
        &mut self,
        _document: &hodei_dsl_lsp::domain::models::Document,
        _position: hodei_dsl_lsp::domain::models::CursorPosition,
    ) -> Result<Option<hodei_dsl_lsp::domain::models::HoverInfo>, Box<dyn std::error::Error>> {
        self.hover_count += 1;
        Ok(self.hover_to_return.clone())
    }

    pub fn was_called(&self) -> bool {
        self.hover_count > 0
    }
}

/// Mock file system for testing
#[derive(Default)]
pub struct MockFileSystem {
    pub files: HashMap<String, String>,
    pub read_count: usize,
    pub write_count: usize,
    pub should_fail: bool,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_file(mut self, path: &str, content: &str) -> Self {
        self.files.insert(path.to_string(), content.to_string());
        self
    }

    pub fn with_should_fail(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }

    pub async fn read_file(&mut self, path: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.read_count += 1;

        if self.should_fail {
            return Err("Mock error".into());
        }

        self.files.get(path)
            .cloned()
            .ok_or_else(|| format!("File not found: {}", path).into())
    }

    pub async fn write_file(&mut self, path: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.write_count += 1;

        if self.should_fail {
            return Err("Mock error".into());
        }

        self.files.insert(path.to_string(), content.to_string());
        Ok(())
    }

    pub async fn exists(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

/// Mock HTTP client for testing
#[derive(Default)]
pub struct MockHttpClient {
    pub requests: Vec<MockRequest>,
    pub responses: Vec<Result<String, Box<dyn std::error::Error>>>,
    pub response_index: usize,
}

#[derive(Debug, Clone)]
pub struct MockRequest {
    pub method: String,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl MockHttpClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_response(mut self, response: Result<String, Box<dyn std::error::Error>>) -> Self {
        self.responses.push(response);
        self
    }

    pub async fn get(&mut self, url: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.requests.push(MockRequest {
            method: "GET".to_string(),
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
        });

        self.get_response()
    }

    pub async fn post(&mut self, url: &str, body: &str) -> Result<String, Box<dyn std::error::Error>> {
        self.requests.push(MockRequest {
            method: "POST".to_string(),
            url: url.to_string(),
            headers: HashMap::new(),
            body: Some(body.to_string()),
        });

        self.get_response()
    }

    fn get_response(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        if self.response_index < self.responses.len() {
            let response = self.responses[self.response_index].clone();
            self.response_index += 1;
            response
        } else {
            Ok("default response".to_string())
        }
    }

    pub fn request_count(&self) -> usize {
        self.requests.len()
    }

    pub fn get_request(&self, index: usize) -> Option<&MockRequest> {
        self.requests.get(index)
    }
}
