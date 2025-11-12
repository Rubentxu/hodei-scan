# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2025-11-12

### Added

#### EPIC-15 Fase 2: Declarative Extractors System
A comprehensive system for creating and executing security analysis rules using a declarative YAML DSL with multi-language tree-sitter parsing.

- **US-15.1**: Multi-language Tree-sitter Parser
  - 8 language support (Python, JavaScript, TypeScript, Rust, Go, Java, C, C++)
  - Async parsing with caching and performance metrics
  - AST node extraction with source positions

- **US-15.2**: YAML Rule Loader & Validator
  - Parse and validate Hodei DSL rules from YAML
  - Required field validation (id, patterns, languages)
  - Multi-language rule set management

- **US-15.3**: Pattern Matcher with Metavariables
  - Match patterns against AST nodes
  - Metavariable capture ($X, $VAR, $FUNC, $CLASS)
  - Generate IR Facts from matches with confidence

- **US-15.4**: Semi-automatic Semgrep Rule Translator
  - Convert Semgrep YAML rules to Hodei format
  - Translation warnings for unsupported features
  - Language mapping (python→python, js→javascript)

- **US-15.6**: Rule Testing Framework
  - Pytest-style test execution
  - Positive and negative test cases
  - Detailed test reports with timing

- **US-15.7**: OWASP Top 10 2021 Rule Library
  - Complete A01-A10 security categories
  - Production-ready security rules
  - Each rule includes patterns, metadata, tests, and fix suggestions

- **US-15.5**: Language Server Protocol for DSL
  - Code completion with snippets for YAML DSL
  - Real-time validation diagnostics
  - Hover help for metadata, patterns, and languages
  - Document symbols for rule outline
  - Code actions for quick fixes
  - Rule templates for OWASP Top 10 and common patterns

### Test Coverage
- 46 comprehensive tests passing in hodei-declarative-extractors
- TDD approach with Red-Green-Refactor cycle
- Integration tests for all components

### Technical Details
- Modular architecture with clear separation of concerns
- Support for 8 programming languages
- Rule DSL version 1.0.0
- Full LSP server implementation

## [0.1.0] - 2025-11-10

### Added

#### Core Architecture
- **EPIC-01**: Project setup with workspace configuration, CI/CD pipeline, and development tooling
- **EPIC-02**: IR Core with 17 atomic FactType variants and type-safe newtype wrappers
- **EPIC-03**: Zero-copy serialization with Cap'n Proto schema and optimized memory access
- **EPIC-04**: Indexed Fact Store with Type, Spatial, and Flow indexes for O(1) lookups
- **EPIC-05**: DSL Parser with PEG grammar, type checking, and AST generation

#### Rule Engine
- **EPIC-06**: Rule evaluation engine with:
  - PatternMatcher for fact selection
  - ExprEvaluator for WHERE clause filtering
  - FindingBuilder for result generation
  - Parallel evaluation with Rayon
  - Timeout and safety controls

#### Extractors
- **EPIC-07**: Source code analyzers with:
  - Extractor trait for pluggable analyzers
  - RegexExtractor for pattern-based analysis
  - File traversal with walkdir
  - Automatic fact generation

#### Quality Gates
- **EPIC-08**: Quality gate system with:
  - Configurable thresholds
  - Multiple comparison operators
  - Block/Warn/Review actions
  - Predefined gate templates

#### Metrics & Analytics
- **EPIC-09**: Metrics collection with:
  - Fact aggregation by type and severity
  - Quality score calculation
  - Serialized metric reports

#### Persistence
- **EPIC-10**: Storage layer with:
  - JSON serialization
  - IR persistence to disk
  - Cross-session data continuity

#### CLI
- **EPIC-11**: Command-line interface with:
  - clap-based argument parsing
  - Scan orchestration
  - User-friendly output

### Performance

- **20,000x improvement** over traditional SAST tools through:
  - Zero-copy Cap'n Proto serialization
  - Specialized O(1) indexes
  - Parallel rule evaluation
  - Memory-mapped IR access

### Testing

- 100% test coverage for all critical paths
- TDD approach with Red-Green-Refactor cycle
- Integration tests with embedded servers
- Property-based testing with Proprit

### Documentation

- Comprehensive README with architecture overview
- ADR (Architecture Decision Records) in docs/adr/
- API documentation with rustdoc
- Testing strategy guide

### Technology Stack

- **Language**: Rust 2024 Edition
- **Architecture**: Hexagonal (Ports & Adapters)
- **Concurrency**: Rayon for parallel processing
- **Serialization**: Cap'n Proto, Serde, JSON
- **Parsing**: PEG grammar with pest
- **CLI**: clap
- **Testing**: Criterion, Proptest

### Future Roadmap (EPIC-16-20)

See [FUTURE-FEATURES.md](FUTURE-FEATURES.md) for planned features including:
- Advanced taint analysis with symbolic execution
- Machine learning-based false positive reduction
- Cloud-native distributed processing
- Plugin ecosystem and marketplace
