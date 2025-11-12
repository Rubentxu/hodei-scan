# API Reference

This document provides detailed API documentation for all developer tools.

## hodei-dsl-lsp

### Core Types

#### HodeiDslServer
```rust
pub struct HodeiDslServer {
    document_repository: Arc<RwLock<InMemoryDocumentRepository>>,
    completion_provider: Arc<HodeiCompletionProvider>,
    hover_provider: Arc<HodeiHoverProvider>,
    semantic_analyzer: Arc<HodeiSemanticAnalyzer>,
}

impl HodeiDslServer {
    pub fn new() -> Self
    
    async fn get_document(&self, uri: Url) -> Option<Document>
    async fn store_document(&self, document: Document)
    async fn remove_document(&self, uri: Url)
}
```

Main LSP server implementation. Use with tower-lsp:

```rust
use tower_lsp::LspService;

let service = LspService::new(HodeiDslServer::new());
service.new_connection(stdin, stdout).await;
```

#### Document
```rust
pub struct Document {
    pub uri: String,
    pub content: String,
    pub version: i32,
}
```

Represents an open document in the LSP server.

#### CompletionItem
```rust
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: String,
    pub additional_text_edits: Vec<TextEdit>,
}

pub enum CompletionItemKind {
    Class,      // Fact types
    Function,   // Functions
    Variable,   // Fields
    Keyword,    // Keywords
    Snippet,    // Code snippets
}
```

### Domain Ports

#### CompletionProvider
```rust
#[async_trait::async_trait]
pub trait CompletionProvider: Send + Sync {
    async fn provide_completions(
        &self,
        document: &Document,
        context: &CompletionContext,
    ) -> Result<Vec<CompletionItem>, String>;
}
```

Provides autocompletion suggestions.

#### HoverProvider
```rust
#[async_trait::async_trait]
pub trait HoverProvider: Send + Sync {
    async fn provide_hover(
        &self,
        document: &Document,
        position: CursorPosition,
    ) -> Result<Option<HoverInfo>, String>;
}
```

Provides hover documentation.

#### SemanticAnalyzer
```rust
#[async_trait::async_trait]
pub trait SemanticAnalyzer: Send + Sync {
    async fn analyze(&self, ast: &hodei_dsl::ast::RuleFile) -> Vec<Diagnostic>;
}
```

Analyzes DSL code for semantic errors.

### Services

#### SemanticValidationService
```rust
pub struct SemanticValidationService<Parser, Analyzer> {
    parser: Arc<Parser>,
    analyzer: Arc<Analyzer>,
    fact_cache: Arc<RwLock<HashMap<String, FactDocumentation>>>,
}

impl<Parser, Analyzer> SemanticValidationService<Parser, Analyzer> {
    pub fn new(parser: Arc<Parser>, analyzer: Arc<Analyzer>) -> Self
    pub async fn validate_document(&self, document: &Document) -> Vec<Diagnostic>
    pub async fn get_fact_completions(&self) -> Vec<CompletionItem>
}
```

Validates documents and provides fact completions.

## hodei-test

### Core Types

#### TestConfig
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub rule: String,
    pub language: String,
    pub cases: Vec<TestCase>,
}
```

Configuration for a test file.

#### TestCase
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    pub name: String,
    pub code: String,
    pub expected_findings: Vec<ExpectedFinding>,
}
```

A single test case.

#### ExpectedFinding
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExpectedFinding {
    pub finding_type: String,
    pub severity: String,
    pub message: String,
}
```

Expected finding from a test.

#### TestResults
```rust
pub struct TestResults {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub case_results: Vec<TestCaseResult>,
}

impl TestResults {
    pub fn new() -> Self
    pub fn add_result(&mut self, result: TestCaseResult)
    pub fn all_passed(&self) -> bool
}
```

Results from running test suite.

### Test Runner

#### HodeiTestRunner
```rust
pub struct HodeiTestRunner<P, R, C> {
    parser: P,
    runner: R,
    comparator: C,
}

impl<P, R, C> HodeiTestRunner<P, R, C>
where
    P: TestConfigParser,
    R: TestCaseRunner,
    C: ResultComparator,
{
    pub fn new(parser: P, runner: R, comparator: C) -> Self
    pub async fn run_test_file(&self, test_file: &Path, rule_path: &str) -> Result<TestResults>
    pub async fn run_all_tests(&self, test_dir: &Path, rule_path: &str) -> Result<TestResults>
}
```

Main test runner orchestration.

### Snapshot Testing

#### SnapshotManager
```rust
pub struct SnapshotManager<R> {
    repository: R,
    snapshot_dir: PathBuf,
}

impl<R> SnapshotManager<R>
where
    R: SnapshotRepository,
{
    pub fn new(repository: R, snapshot_dir: PathBuf) -> Self
    pub async fn update_snapshots(&self, results: &TestResults) -> Result<()>
    pub async fn verify_snapshots(&self, results: &TestResults) -> Result<Vec<SnapshotDiff>>
}
```

Manages test snapshots.

#### SnapshotDiff
```rust
pub struct SnapshotDiff {
    pub test_name: String,
    pub changes: Vec<String>,
}
```

Represents a difference between snapshots.

### Domain Ports

#### TestConfigParser
```rust
#[async_trait::async_trait]
pub trait TestConfigParser: Send + Sync {
    async fn parse_file(&self, path: &Path) -> Result<TestConfig>;
}
```

Parses test configuration files.

#### TestCaseRunner
```rust
#[async_trait::async_trait]
pub trait TestCaseRunner: Send + Sync {
    async fn run_case(&self, test_case: &TestCase, rule_path: &str) -> Result<TestCaseResult>;
}
```

Runs individual test cases.

#### ResultComparator
```rust
#[async_trait::async_trait]
pub trait ResultComparator: Send + Sync {
    async fn compare(
        &self,
        actual: &[hodei_ir::Finding],
        expected: &[ExpectedFinding],
    ) -> Vec<AssertionResult>;
}
```

Compares actual vs expected results.

#### SnapshotRepository
```rust
#[async_trait::async_trait]
pub trait SnapshotRepository: Send + Sync {
    async fn save(&self, snapshot: &TestSnapshot) -> Result<()>
    async fn load(&self, name: &str) -> Result<Option<TestSnapshot>>
    async fn list(&self) -> Result<Vec<String>>
}
```

Repository for storing/retrieving snapshots.

## ir-dump

### Core Types

#### Format
```rust
#[derive(Clone)]
pub enum Format {
    Json,
    Yaml,
    Visual,
}
```

Output format options.

#### IRReader
```rust
pub struct IRReader;

impl IRReader {
    pub fn new() -> Self
    pub async fn read(&self, path: &Path) -> Result<FindingSet>
}
```

Reads IR from various formats.

#### IRFormatter
```rust
pub struct IRFormatter;

impl IRFormatter {
    pub fn new() -> Self
    pub fn format(&self, ir: &FindingSet, format: &Format) -> Result<String, String>
}
```

Formats IR to output.

#### InteractiveExplorer
```rust
pub struct InteractiveExplorer {
    ir: Arc<FindingSet>,
    current_index: usize,
    reedline: Reedline,
}

impl InteractiveExplorer {
    pub fn new(ir: FindingSet) -> Self
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>
}
```

Interactive REPL for exploring IR.

### CLI

#### Cli
```rust
#[derive(Parser)]
#[command(name = "ir-dump")]
struct Cli {
    #[arg(short, long)]
    input: PathBuf,
    
    #[arg(short, long, default_value = "visual")]
    format: Format,
    
    #[arg(short, long)]
    filter: Option<String>,
    
    #[arg(short, long)]
    interactive: bool,
    
    #[arg(short = '1', long)]
    input1: Option<PathBuf>,
    
    #[arg(short = '2', long)]
    input2: Option<PathBuf>,
}

pub async fn run_cli() -> Result<(), Box<dyn std::error::Error>>
```

Main CLI entry point.

## Usage Examples

### Complete LSP Setup
```rust
use hodei_dsl_lsp::{HodeiDslServer, Infrastructure};
use tower_lsp::LspService;

#[tokio::main]
async fn main() {
    let server = HodeiDslServer::new();
    let service = LspService::new(server);
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    service.new_connection(stdin, stdout).await;
}
```

### Running Tests
```rust
use hodei_test::{HodeiTestRunner, YamlTestConfigParser, FileSystemSnapshotRepository};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = YamlTestConfigParser::new();
    let runner = /* create runner */;
    let comparator = /* create comparator */;
    
    let test_runner = HodeiTestRunner::new(parser, runner, comparator);
    let results = test_runner
        .run_test_file("tests/my_rule.hodei.test", "rules/my_rule.hodei")
        .await?;
    
    println!("Passed: {}/{}", results.passed_tests, results.total_tests);
    Ok(())
}
```

### Dumping IR
```rust
use ir_dump::{IRReader, IRFormatter, Format};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reader = IRReader::new();
    let formatter = IRFormatter::new();
    
    let ir = reader.read(Path::new("facts.json")).await?;
    let output = formatter.format(&ir, &Format::Visual)?;
    
    println!("{}", output);
    Ok(())
}
```

## Error Handling

All public APIs return `Result<T, E>` where:
- `T` is the success type
- `E` is typically `Box<dyn std::error::Error>` or `anyhow::Error`

### Example Error Handling
```rust
match reader.read(path).await {
    Ok(ir) => {
        let output = formatter.format(&ir, &Format::Json)?;
        println!("{}", output);
    }
    Err(e) => {
        eprintln!("Error reading IR: {}", e);
        std::process::exit(1);
    }
}
```

## Thread Safety

All types are designed to be thread-safe:
- Use `Arc<T>` for shared ownership
- Use `RwLock<T>` for concurrent access
- Traits use `Send + Sync` bounds

## Async/Await

All I/O operations are async:
- Use `tokio` runtime
- All async functions return `impl Future<Output = Result<T, E>>`
- Enable `tokio` features: `features = ["full"]`

## Performance Considerations

### LSP Server
- Documents are cached in memory
- Use `RwLock` for concurrent access
- Parse documents lazily

### Test Runner
- Run tests in parallel using `tokio::spawn`
- Cache compiled rules
- Use incremental compilation

### IR Dump
- Stream large IR files
- Use buffered I/O
- Support filtering to reduce memory usage
