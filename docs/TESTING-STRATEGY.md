# Testing Strategy

## Overview

hodei-scan follows a comprehensive testing strategy to ensure reliability and quality.

## Test Types

### 1. Unit Tests
- **Scope**: Individual functions and modules
- **Coverage**: All public APIs and critical logic paths
- **Location**: Inline tests with `#[cfg(test)]` modules

### 2. Integration Tests
- **Scope**: Cross-crate interactions
- **Tools**: Embedded gRPC servers with mock services
- **Pattern**: Test entire workflows (extract -> evaluate -> report)

### 3. Property Tests
- **Framework**: Proptest for generative testing
- **Scope**: Serialization, parsing, and transformation logic
- **Coverage**: Invariant preservation and edge cases

## Coverage Goals

- **Unit Test Coverage**: >80%
- **Integration Test Coverage**: All major workflows
- **Critical Path Coverage**: 100%

## Running Tests

```bash
# All tests
cargo test

# Specific crate
cargo test -p hodei-engine

# With coverage
cargo install cargo-tarpaulin
cargo tarpaulin --out xml
```

## Best Practices

1. **TDD**: Write tests first, then implementation
2. **Test Data**: Use factories and builders for test data
3. **Mocking**: Prefer embedded servers over Testcontainers
4. **CI Integration**: All tests must pass before merge
5. **Performance**: Benchmark critical paths with Criterion

## Test Organization

```
crate/
├── src/
│   ├── module.rs          # Implementation
│   └── tests.rs           # Unit tests
├── tests/
│   ├── integration/       # Integration tests
│   └── fixtures/          # Test fixtures
└── benches/               # Benchmarks
```
