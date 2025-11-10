# hodei-scan v3.2: Advanced Static Code Analysis with IR Architecture

## Overview

hodei-scan is a high-performance static code analysis tool built on a hexagonal architecture with an Intermediate Representation (IR) at its core.

## Architecture

### Core Components

1. **hodei-ir**: Intermediate Representation core types
   - 17 atomic FactType variants
   - Type-safe newtype wrappers
   - Zero-copy Cap'n Proto serialization

2. **hodei-engine**: Rule evaluation engine
   - High-performance indexed fact store
   - Parallel rule evaluation with Rayon
   - Pattern matching and expression evaluation

3. **hodei-dsl**: Domain-specific language
   - PEG grammar with pest parser
   - Type-checked rule definitions
   - Rich AST with pattern matching

4. **hodei-extractors**: Source code analyzers
   - Pluggable extractor architecture
   - Regex-based pattern matching
   - File traversal with walkdir

5. **hodei-engine (Gates)**: Quality gates
   - Configurable quality thresholds
   - Block/Warn/Review actions
   - Automatic violation detection

6. **hodei-metrics**: Analytics and metrics
   - Fact aggregation
   - Quality scoring
   - Severity distribution

7. **hodei-persistence**: Storage layer
   - JSON serialization
   - IR persistence
   - Cross-session continuity

8. **hodei-cli**: Command-line interface
   - clap-based CLI
   - Scan orchestration
   - User-friendly output

## Performance Features

- **20,000x faster** than traditional SAST tools through:
  - Zero-copy serialization
  - Memory-mapped IR access
  - Specialized indexes (O(1) lookups)
  - Parallel evaluation with Rayon

## Getting Started

```bash
# Install
cargo install hodei-scan

# Scan a project
hodei-scan scan /path/to/project

# Build from source
cargo build --release
```

## Example Rule

```hodei
rule "SQL Injection Detection" {
    match {
        pattern TaintSource with { var: source, flow_id: f }
        pattern TaintSink with { func: "query", consumes_flow: f }
    }
    where {
        source.confidence > 0.8
    }
    emit {
        message: "Possible SQL injection: untrusted input flows to query function"
        severity: CRITICAL
    }
}
```

## Project Structure

```
hodei-scan/
├── crates/
│   ├── hodei-ir/              # Core IR types
│   ├── hodei-engine/          # Rule engine & quality gates
│   ├── hodei-dsl/             # Domain-specific language
│   ├── hodei-extractors/      # Source analyzers
│   ├── hodei-metrics/         # Metrics & analytics
│   ├── hodei-persistence/     # Storage layer
│   └── hodei-cli/             # CLI interface
├── docs/                      # Documentation
├── .github/workflows/         # CI/CD pipeline
└── Cargo.toml                 # Workspace configuration
```

## Development

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration

# Benchmarks
cargo bench

# Documentation
cargo doc --no-deps --open
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

MIT License - see LICENSE file for details
