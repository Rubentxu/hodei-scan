# Integration & End-to-End Testing Strategy

## Overview
Comprehensive testing strategy for hodei-scan to validate all functionality, integration points, and real-world usage scenarios.

## Testing Pyramid

### 1. Unit Tests (Existing)
- **Scope**: Individual functions and methods
- **Coverage**: Each crate independently
- **Tooling**: `cargo test`

### 2. Integration Tests (New)
- **Scope**: Interaction between crates
- **Focus**: Data flow, API contracts, type compatibility
- **Coverage**:
  - hodei-dsl ↔ hodei-ir
  - hodei-extractors ↔ hodei-ir
  - hodei-engine ↔ hodei-ir
  - hodei-cli ↔ hodei-engine

### 3. End-to-End Tests (New)
- **Scope**: Complete workflows
- **Focus**: Real-world scenarios
- **Coverage**:
  - Full scan workflow
  - Rule parsing → Extraction → Evaluation → Reporting
  - Quality gates validation
  - CLI usage

## Test Suites

### Suite 1: DSL Integration Tests
**File**: `tests/integration/dsl_integration.rs`

#### Test Cases
1. **Rule Parsing Flow**
   - Parse valid rule files
   - Convert AST to IR
   - Type check rules
   - Generate compiled rules

2. **Rule Validation**
   - Invalid syntax handling
   - Type checking errors
   - Semantic validation
   - Error reporting

3. **Rule Compilation**
   - AST → IR conversion
   - Pattern matching compilation
   - Expression evaluation setup

### Suite 2: Extractor Integration Tests
**File**: `tests/integration/extractor_integration.rs`

#### Test Cases
1. **Java Code Extraction**
   - Parse Java files
   - Extract facts (methods, classes, annotations)
   - Generate IR facts
   - Validate fact structure

2. **Multi-File Analysis**
   - Process multiple files
   - Cross-reference facts
   - Build project graph
   - Handle dependencies

3. **Fact Validation**
   - Verify fact structure
   - Check field completeness
   - Validate references
   - Test fact uniqueness

### Suite 3: Engine Integration Tests
**File**: `tests/integration/engine_integration.rs`

#### Test Cases
1. **Rule Evaluation Flow**
   - Load compiled rules
   - Query facts from stores
   - Evaluate patterns
   - Generate findings

2. **Store Integration**
   - Type index queries
   - Spatial index queries
   - Flow index tracking
   - Cross-store joins

3. **Finding Generation**
   - Pattern matching
   - Confidence calculation
   - Severity assignment
   - Message formatting

### Suite 4: CLI Integration Tests
**File**: `tests/integration/cli_integration.rs`

#### Test Cases
1. **Scan Command**
   - Parse CLI arguments
   - Configure extractors
   - Run analysis
   - Generate output

2. **Output Formats**
   - JSON output
   - HTML reports
   - Markdown summaries
   - Exit codes

3. **Error Handling**
   - Invalid paths
   - Malformed rules
   - Missing files
   - Permission errors

### Suite 5: End-to-End Tests
**File**: `tests/e2e/full_workflow.rs`

#### Test Cases

##### Test 1: Complete Java Project Scan
```rust
#[test]
fn test_full_java_project_scan() {
    // Setup: Create test Java project
    // Execute: Run full scan
    // Validate: All stages work together
    // Cleanup: Remove test artifacts
}
```

##### Test 2: PetClinic Analysis
```rust
#[test]
fn test_petclinic_analysis() {
    // Setup: Clone PetClinic
    // Execute: Run hodei-scan
    // Validate: Reports generated
    // Verify: Quality gates pass
}
```

##### Test 3: Custom Rules Flow
```rust
#[test]
fn test_custom_rules_flow() {
    // Create custom rules
    // Parse and validate
    // Execute scan
    // Verify findings
}
```

##### Test 4: Quality Gates Validation
```rust
#[test]
fn test_quality_gates() {
    // Define quality gates
    // Run scan
    // Validate gates
    // Check pass/fail logic
}
```

### Suite 6: Real-World Scenarios
**File**: `tests/e2e/real_world_scenarios.rs`

#### Test Cases
1. **Spring Boot Application**
   - Scan real Spring Boot project
   - Detect annotations, dependencies
   - Validate security rules
   - Check test coverage

2. **Microservices Architecture**
   - Multiple modules
   - Service boundaries
   - API contracts
   - Cross-module issues

3. **Legacy Codebase**
   - Old Java versions
   - Deprecated APIs
   - Code smells
   - Technical debt

### Suite 7: Performance Tests
**File**: `tests/perf/performance.rs`

#### Test Cases
1. **Large Project Handling**
   - 1000+ files
   - Memory usage
   - Processing time
   - Concurrent scanning

2. **Rule Evaluation Performance**
   - Complex patterns
   - Large fact sets
   - Query optimization
   - Caching effectiveness

### Suite 8: Error Handling Tests
**File**: `tests/integration/error_handling.rs`

#### Test Cases
1. **Malformed Input**
   - Invalid Java code
   - Corrupted files
   - Encoding issues
   - Large files

2. **System Errors**
   - Disk space
   - Memory limits
   - Network issues
   - Permission denied

## Test Data Management

### Fixtures
- `tests/fixtures/java/`: Sample Java code
- `tests/fixtures/rules/`: Rule files
- `tests/fixtures/projects/`: Test projects
- `tests/fixtures/expected/`: Expected outputs

### Test Projects
1. **Simple Java Project** (10 files)
   - Basic classes
   - Simple methods
   - No frameworks

2. **Spring PetClinic** (47 files)
   - Real production code
   - Spring annotations
   - JPA entities
   - Multiple layers

3. **Complex Enterprise** (100+ files)
   - Multiple modules
   - Various patterns
   - Legacy code
   - Edge cases

## Continuous Integration

### GitHub Actions
```yaml
# .github/workflows/integration-tests.yml
name: Integration Tests
on: [push, pull_request]
jobs:
  integration:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Integration Tests
        run: cargo test --test integration
      - name: Run E2E Tests
        run: cargo test --test e2e
      - name: Performance Tests
        run: cargo bench
```

## Success Criteria

### Coverage Targets
- **Unit Tests**: 80% code coverage
- **Integration Tests**: 90% of public APIs
- **E2E Tests**: 100% of critical workflows

### Performance Targets
- **PetClinic scan**: < 10 seconds
- **Memory usage**: < 500MB for 1000 files
- **Quality gates**: < 1 second evaluation

### Quality Gates
- All integration tests pass
- All E2E tests pass
- No memory leaks
- No deadlocks
- Proper error messages

## Implementation Plan

### Phase 1: Core Integration (Week 1)
- [ ] DSL → IR integration
- [ ] Extractor → IR integration
- [ ] Engine → IR integration
- [ ] CLI → Engine integration

### Phase 2: E2E Workflows (Week 2)
- [ ] Full scan workflow
- [ ] PetClinic analysis
- [ ] Custom rules flow
- [ ] Quality gates

### Phase 3: Real-World (Week 3)
- [ ] Real project scans
- [ ] Performance testing
- [ ] Error handling
- [ ] Documentation

## Tooling

### Test Frameworks
- **Rust**: `cargo test`, `proptest`
- **Fixtures**: `tempfile`, `fs_extra`
- **Assertions**: `pretty_assertions`
- **Performance**: `criterion`, `tokio-test`

### Test Utilities
- `TestProjectBuilder`: Create test projects
- `RuleTest Harness`: Test rule parsing
- `FactValidator`: Validate extracted facts
- `ReportComparator`: Compare outputs
