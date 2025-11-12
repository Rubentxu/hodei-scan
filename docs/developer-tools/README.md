# Developer Tools Documentation

This directory contains comprehensive documentation for the hodei-scan developer experience tools.

## Tools Overview

### 1. hodei-dsl-lsp
**Language Server Protocol (LSP) implementation**

Provides intelligent editing features for hodei-scan rules:
- Autocompletion for fact types and fields
- Real-time semantic validation
- Hover documentation
- Error diagnostics

**Location**: `crates/hodei-dsl-lsp/`

### 2. hodei-test
**Rule Testing Framework**

YAML-based testing framework for hodei-scan rules:
- Test file format (.hodei.test)
- Rule testing runner
- Snapshot testing
- CI integration

**Location**: `crates/hodei-test/`

### 3. ir-dump
**IR Debug Tool**

Interactive tool for inspecting Intermediate Representation:
- Dump IR in JSON/YAML/Visual format
- Filter IR by fact type
- Interactive REPL explorer
- IR comparison

**Location**: `crates/ir-dump/`

### 4. VS Code Extension
**Complete IDE integration**

Full-featured VS Code extension:
- Syntax highlighting
- LSP integration
- Custom commands
- Keybindings

**Location**: `extensions/vscode-hodei-dsl/`

## Quick Start

### Install LSP Server
```bash
cargo install --path crates/hodei-dsl-lsp
```

### Install Test Framework
```bash
cargo install --path crates/hodei-test
```

### Install IR Dump Tool
```bash
cargo install --path crates/ir-dump
```

### Install VS Code Extension
```bash
cd extensions/vscode-hodei-dsl
npm install
npm run compile
code --install-extension dist/hodei-scan-dsl-support-0.1.0.vsix
```

## Architecture

All tools follow **Hexagonal Architecture** (Ports & Adapters) with three layers:

1. **Domain Layer** - Core business logic and models
2. **Application Layer** - Use cases and application services
3. **Infrastructure Layer** - External integrations and adapters

This ensures:
- Separation of concerns
- Testability
- Maintainability
- Flexibility

## Documentation Structure

- [LSP Server Guide](./lsp-guide.md) - Complete LSP implementation guide
- [Test Framework Guide](./test-framework-guide.md) - Testing rules with hodei-test
- [IR Dump Guide](./ir-dump-guide.md) - Debugging with ir-dump
- [VS Code Extension Guide](./vscode-extension-guide.md) - IDE integration
- [API Reference](./api-reference.md) - Detailed API documentation
- [Examples](./examples/) - Practical examples and tutorials

## Contributing

When adding new features to any tool:

1. Follow the hexagonal architecture pattern
2. Write unit tests for all public APIs
3. Update documentation
4. Add examples if applicable
5. Ensure CI passes

## License

MIT
