//! Multi-language parser using tree-sitter
//!
//! US-15.1: Motor Tree-sitter Multi-Lenguaje

use crate::errors::{DeclarativeExtractorError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Supported programming languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Python,
    JavaScript,
    TypeScript,
    Rust,
    Go,
    Java,
    C,
    Cpp,
}

impl Language {
    /// Get all supported languages
    pub fn all_languages() -> Vec<Self> {
        vec![
            Self::Python,
            Self::JavaScript,
            Self::TypeScript,
            Self::Rust,
            Self::Go,
            Self::Java,
            Self::C,
            Self::Cpp,
        ]
    }

    /// Get language name as string
    pub fn name(&self) -> &'static str {
        match self {
            Self::Python => "python",
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Rust => "rust",
            Self::Go => "go",
            Self::Java => "java",
            Self::C => "c",
            Self::Cpp => "cpp",
        }
    }

    /// Parse language from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "python" => Some(Self::Python),
            "javascript" | "js" => Some(Self::JavaScript),
            "typescript" | "ts" => Some(Self::TypeScript),
            "rust" => Some(Self::Rust),
            "go" | "golang" => Some(Self::Go),
            "java" => Some(Self::Java),
            "c" => Some(Self::C),
            "cpp" | "c++" => Some(Self::Cpp),
            _ => None,
        }
    }
}

/// Parse error with location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let (Some(line), Some(col)) = (self.line, self.column) {
            write!(f, "Parse error at {}:{} - {}", line, col, self.message)
        } else {
            write!(f, "Parse error: {}", self.message)
        }
    }
}

/// AST node representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ASTNode {
    pub node_type: String,
    pub text: String,
    pub start_position: usize,
    pub end_position: usize,
    pub children: Vec<ASTNode>,
}

impl ASTNode {
    /// Create a new leaf node
    pub fn new_leaf(node_type: String, text: String, start: usize, end: usize) -> Self {
        Self {
            node_type,
            text,
            start_position: start,
            end_position: end,
            children: Vec::new(),
        }
    }

    /// Create a new internal node
    pub fn new_internal(node_type: String, children: Vec<ASTNode>) -> Self {
        let (start, end) = if !children.is_empty() {
            let start = children.first().unwrap().start_position;
            let end = children.last().unwrap().end_position;
            (start, end)
        } else {
            (0, 0)
        };

        Self {
            node_type,
            text: String::new(),
            start_position: start,
            end_position: end,
            children,
        }
    }

    /// Check if this is a leaf node
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// Get the text of this node
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the node type
    pub fn node_type(&self) -> &str {
        &self.node_type
    }

    /// Get children
    pub fn children(&self) -> &[ASTNode] {
        &self.children
    }
}

/// Parse result with AST and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub ast: ASTNode,
    pub language: Language,
    pub line_count: usize,
    pub character_count: usize,
}

impl ParseResult {
    /// Create a new parse result
    pub fn new(
        ast: ASTNode,
        language: Language,
        line_count: usize,
        character_count: usize,
    ) -> Self {
        Self {
            ast,
            language,
            line_count,
            character_count,
        }
    }

    /// Get the AST
    pub fn ast(&self) -> &ASTNode {
        &self.ast
    }

    /// Get the language
    pub fn language(&self) -> Language {
        self.language
    }

    /// Get line count
    pub fn line_count(&self) -> usize {
        self.line_count
    }

    /// Get character count
    pub fn character_count(&self) -> usize {
        self.character_count
    }
}

/// Multi-language parser using tree-sitter
pub struct MultiLanguageParser {
    parsers: Arc<RwLock<HashMap<Language, String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LanguageConfig {
    name: &'static str,
}

/// Performance metrics
#[derive(Debug, Clone)]
pub struct ParseMetrics {
    pub parse_time_ms: u64,
    pub ast_nodes: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

impl Default for ParseMetrics {
    fn default() -> Self {
        Self {
            parse_time_ms: 0,
            ast_nodes: 0,
            cache_hits: 0,
            cache_misses: 0,
        }
    }
}

impl MultiLanguageParser {
    /// Create a new multi-language parser
    pub fn new() -> Self {
        Self {
            parsers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Parse code in the specified language
    pub async fn parse(&self, language: Language, code: &str) -> Result<ParseResult> {
        let start_time = std::time::Instant::now();

        // Get or initialize parser for this language
        {
            let parsers = self.parsers.read().await;
            if !parsers.contains_key(&language) {
                drop(parsers);
                let mut parsers = self.parsers.write().await;
                parsers.insert(language, language.name().to_string());
            }
        }

        let parse_time = start_time.elapsed();
        let parse_time_ms = parse_time.as_millis() as u64;

        // Simple AST generation for now
        // In real implementation, would use tree-sitter
        let ast = self.generate_simple_ast(code, language);

        let line_count = code.lines().count();
        let character_count = code.len();

        Ok(ParseResult::new(ast, language, line_count, character_count))
    }

    /// Generate a simple AST for demonstration
    fn generate_simple_ast(&self, code: &str, language: Language) -> ASTNode {
        let mut children = Vec::new();

        for (i, line) in code.lines().enumerate() {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                let node = ASTNode::new_leaf(
                    "statement".to_string(),
                    trimmed.to_string(),
                    i * 100,
                    i * 100 + line.len(),
                );
                children.push(node);
            }
        }

        ASTNode::new_internal(format!("{}_module", language.name()), children)
    }

    /// Pre-warm parser for specific language
    pub async fn warm_up(&self, language: Language) -> Result<()> {
        let test_code = match language {
            Language::Python => "def test():\n    pass\n",
            Language::JavaScript => "function test() {\n    return;\n}",
            Language::TypeScript => "function test(): void {\n    return;\n}",
            Language::Rust => "fn test() {\n    let _ = 42;\n}",
            Language::Go => "func test() {\n    _ = 42\n}",
            Language::Java => "void test() {\n    int x = 42;\n}",
            Language::C => "void test() {\n    int x = 42;\n}",
            Language::Cpp => "void test() {\n    int x = 42;\n}",
        };

        self.parse(language, test_code).await.map(|_| ())
    }

    /// Get parse metrics
    pub async fn get_metrics(&self) -> ParseMetrics {
        // For now, return default metrics
        // In real implementation, would track actual metrics
        ParseMetrics::default()
    }
}

impl Default for MultiLanguageParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_python_code() {
        let parser = MultiLanguageParser::new();

        let code = r#"
def hello_world(name: str) -> str:
    return f"Hello, {name}!"
"#;

        let result = parser.parse(Language::Python, code).await;

        assert!(result.is_ok(), "Should parse Python code successfully");
        let parse_result = result.unwrap();

        assert_eq!(parse_result.language(), Language::Python);
        assert!(parse_result.line_count() >= 3);
        assert!(parse_result.character_count() > 0);
        assert!(!parse_result.ast().is_leaf()); // Should have children
    }

    #[tokio::test]
    async fn test_parse_javascript_code() {
        let parser = MultiLanguageParser::new();

        let code = r#"
function greet(name: string): string {
    return `Hello, ${name}!`;
}
const result = greet("World");
"#;

        let result = parser.parse(Language::JavaScript, code).await;

        assert!(result.is_ok(), "Should parse JavaScript code successfully");
        let parse_result = result.unwrap();

        assert_eq!(parse_result.language(), Language::JavaScript);
        assert!(parse_result.line_count() >= 3);
        assert!(!parse_result.ast().is_leaf());
    }

    #[tokio::test]
    async fn test_parse_rust_code() {
        let parser = MultiLanguageParser::new();

        let code = r#"
fn main() {
    println!("Hello, World!");
}
"#;

        let result = parser.parse(Language::Rust, code).await;

        assert!(result.is_ok(), "Should parse Rust code successfully");
        let parse_result = result.unwrap();

        assert_eq!(parse_result.language(), Language::Rust);
        assert!(parse_result.line_count() >= 3);
    }

    #[tokio::test]
    async fn test_language_from_str() {
        assert_eq!(Language::from_str("python"), Some(Language::Python));
        assert_eq!(Language::from_str("javascript"), Some(Language::JavaScript));
        assert_eq!(Language::from_str("js"), Some(Language::JavaScript));
        assert_eq!(Language::from_str("rust"), Some(Language::Rust));
        assert_eq!(Language::from_str("go"), Some(Language::Go));
        assert_eq!(Language::from_str("java"), Some(Language::Java));
        assert_eq!(Language::from_str("unknown"), None);
    }

    #[tokio::test]
    async fn test_language_name() {
        assert_eq!(Language::Python.name(), "python");
        assert_eq!(Language::JavaScript.name(), "javascript");
        assert_eq!(Language::Rust.name(), "rust");
    }

    #[tokio::test]
    async fn test_all_languages() {
        let languages = Language::all_languages();
        assert!(languages.len() >= 8, "Should support 8+ languages");

        // Check that all expected languages are present
        assert!(languages.contains(&Language::Python));
        assert!(languages.contains(&Language::JavaScript));
        assert!(languages.contains(&Language::TypeScript));
        assert!(languages.contains(&Language::Rust));
        assert!(languages.contains(&Language::Go));
        assert!(languages.contains(&Language::Java));
    }

    #[tokio::test]
    async fn test_warm_up() {
        let parser = MultiLanguageParser::new();

        let result = parser.warm_up(Language::Python).await;
        assert!(result.is_ok(), "Warm-up should succeed");
    }

    #[tokio::test]
    async fn test_empty_code() {
        let parser = MultiLanguageParser::new();

        let result = parser.parse(Language::Python, "").await;
        assert!(result.is_ok(), "Should parse empty code");
        assert_eq!(result.unwrap().line_count(), 0);
    }

    #[tokio::test]
    async fn test_single_line_code() {
        let parser = MultiLanguageParser::new();

        let result = parser.parse(Language::Python, "x = 42").await;
        assert!(result.is_ok(), "Should parse single line");
        assert_eq!(result.unwrap().line_count(), 1);
    }

    #[test]
    fn test_ast_node_creation() {
        let node = ASTNode::new_leaf("identifier".to_string(), "x".to_string(), 0, 1);

        assert_eq!(node.node_type(), "identifier");
        assert_eq!(node.text(), "x");
        assert!(node.is_leaf());
        assert_eq!(node.children().len(), 0);
    }

    #[test]
    fn test_ast_internal_node() {
        let children = vec![ASTNode::new_leaf(
            "number".to_string(),
            "42".to_string(),
            2,
            4,
        )];

        let node = ASTNode::new_internal("assignment".to_string(), children);

        assert_eq!(node.node_type(), "assignment");
        assert!(!node.is_leaf());
        assert_eq!(node.children().len(), 1);
    }
}
