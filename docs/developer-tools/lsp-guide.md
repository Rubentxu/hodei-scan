# LSP Server Implementation Guide

The hodei-dsl-lsp crate provides a complete Language Server Protocol (LSP) implementation for the hodei-scan DSL.

## Architecture

### Domain Layer
Located in `src/domain/`:

- **models.rs** - Core data structures (Document, CompletionItem, HoverInfo, etc.)
- **ports.rs** - Interfaces (DocumentRepository, CompletionProvider, etc.)
- **services.rs** - Business logic services (SemanticValidationService, CompletionService, etc.)

### Application Layer
Located in `src/application/`:

- **use_cases.rs** - Application use cases (GetCompletionsUseCase, GetHoverInfoUseCase, etc.)

### Infrastructure Layer
Located in `src/infrastructure/`:

- **adapters/** - Concrete implementations of domain ports
  - `document_repository.rs` - In-memory document storage
  - `completion_provider.rs` - Intelligent autocompletion
  - `hover_provider.rs` - Hover documentation
  - `semantic_analyzer.rs` - Semantic validation
  - `diagnostic_emitter.rs` - Diagnostic message emitter

- **server/** - LSP server implementation
  - `hodei_server.rs` - Main server using tower-lsp

## Key Components

### HodeiDslServer
The main LSP server that implements the `LanguageServer` trait from tower-lsp.

```rust
pub struct HodeiDslServer {
    document_repository: Arc<RwLock<InMemoryDocumentRepository>>,
    completion_provider: Arc<HodeiCompletionProvider>,
    hover_provider: Arc<HodeiHoverProvider>,
    semantic_analyzer: Arc<HodeiSemanticAnalyzer>,
}
```

### CompletionProvider
Provides intelligent autocompletion based on context:

- After `fact.`: Suggests fact types (Vulnerability, CodeSmell, SecurityIssue)
- After fact type name: Suggests fields
- Keyword autocompletion: `rule`, `when`, etc.
- Function autocompletion: `matches()`, `contains()`, etc.

### HoverProvider
Shows documentation on hover:

- Fact types: Description and fields
- Functions: Usage, parameters, description
- Keywords: Brief explanation

### SemanticAnalyzer
Validates DSL code:

- Checks for unknown fact types
- Validates function calls
- Detects syntax errors

## Running the Server

### As a standalone binary
```bash
cargo run --package hodei-dsl-lsp
```

### With a Language Client
```rust
use tower_lsp::{LspService, LanguageServer};

#[tokio::main]
async fn main() {
    let service = LspService::new(HodeiDslServer::new());
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    service.new_connection(stdin, stdout).await;
}
```

## Testing

```bash
# Run all tests
cargo test --package hodei-dsl-lsp

# Run specific test
cargo test --package hodei-dsl-lsp test_completion_provides_fact_types
```

## Configuration

The server accepts configuration via LSP initialization:

```json
{
  "initializationOptions": {
    "factTypes": ["Vulnerability", "CodeSmell"],
    "functions": ["matches", "contains"]
  }
}
```

## Extending the LSP

### Adding a new completion provider
1. Implement `CompletionProvider` trait in `src/domain/ports.rs`
2. Create adapter in `src/infrastructure/adapters/`
3. Register in `HodeiDslServer::new()`

### Adding a new feature
1. Define domain model in `src/domain/models.rs`
2. Create port interface in `src/domain/ports.rs`
3. Implement service in `src/domain/services.rs`
4. Create use case in `src/application/use_cases.rs`
5. Implement adapter in `src/infrastructure/adapters/`
6. Integrate into server in `src/infrastructure/server/hodei_server.rs`

## Best Practices

1. **Keep domain pure** - No dependencies on external crates
2. **Use traits for ports** - Enables mocking and testing
3. **Prefer async/await** - For non-blocking operations
4. **Log everything** - Use tracing for debugging
5. **Handle errors gracefully** - Return meaningful error messages
