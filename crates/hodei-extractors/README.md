# hodei-extractors

Extractors for various code analysis tools.

## Overview

This crate provides extractors for integrating with various code analysis tools and generating IR facts from different sources.

## Key Components

- **TreeSitterExtractor**: AST-based code parsing
- **OxcExtractor**: High-performance JavaScript/TypeScript parser
- **SemgrepTaint**: Taint analysis integration
- **JaCoCoParser**: Test coverage data parser
- **CargoAudit**: Rust dependency vulnerability scanner

## Usage

```rust
use hodei_extractors::TreeSitterExtractor;

let extractor = TreeSitterExtractor::new();
let facts = extractor.extract(source_code)?;
```

## License

MIT
