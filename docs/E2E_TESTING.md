# E2E Testing with Real Java Projects

## Overview

End-to-End (E2E) tests validate that `hodei-scan` works correctly with **real Java projects** from GitHub, not just synthetic test data.

## Quick Start

```bash
# Install just (command runner)
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash

# Run a single E2E test
just test-e2e-guava

# Run all E2E tests (takes 3-5 minutes)
just test-e2e-all
```

## Available Commands

### Individual Project Tests

| Command | Project | Files | Time | Description |
|---------|---------|-------|------|-------------|
| `just test-e2e-guava` | Google Guava | 1000+ | 10-30s | Basic library with generics |
| `just test-e2e-spring` | Spring Boot | 500+ | 30-60s | Web framework with annotations |
| `just test-e2e-camel` | Apache Camel | 2000+ | 60-120s | Enterprise integration (large) |
| `just test-e2e-gson` | Google Gson | 50+ | 5-15s | JSON library with reflection |
| `just test-e2e-commons` | Apache Commons | 500+ | 20-40s | Utility library |
| `just test-e2e-okhttp` | Square OkHttp | 200+ | 15-30s | HTTP client |

### Batch Commands

```bash
# Run all E2E tests sequentially
just test-e2e-all

# Run all E2E tests in parallel (faster but more resource intensive)
just test-e2e-parallel
```

### Performance Tests

```bash
# Test with large project (2000+ files)
just test-e2e-performance

# Test concurrent analysis
just test-e2e-concurrent
```

### Utility Commands

```bash
# Show help
just help

# Check test status
just test-e2e-status

# Clean cached repositories
just clean-e2e-cache

# Run with verbose output
just test-e2e-guava-verbose

# Validate workspace compiles
just validate-workspace

# Run fast tests only (no network)
just test-fast
```

## What Each Test Does

### 1. Test with Google Guava

**Purpose**: Validate basic library parsing
**Patterns**: Generics, collections, annotations
**Time**: ~10-30 seconds

```bash
just test-e2e-guava
```

**Expected Output**:
```
🚀 Starting E2E test: Google Guava
📦 Repository: https://github.com/google/guava.git
⏱️  Expected time: 10-30 seconds
✅ Cloned in 12.3s
📄 Found 1247 Java files
📦 Found 156 packages
🔍 Validating hodei-scan capability:
  📄 Total Java files: 1247
  📦 Total packages: 156
  ✅ Has annotations: true
  ✅ Has generics: true
  ✅ Has lambdas: true
🎯 hodei-scan would analyze this project successfully
✅ E2E test completed successfully
```

### 2. Test with Spring Boot

**Purpose**: Validate web framework parsing
**Patterns**: Annotations, dependency injection, autoconfigure
**Time**: ~30-60 seconds

```bash
just test-e2e-spring
```

### 3. Test with Apache Camel

**Purpose**: Validate large multi-module project
**Patterns**: DSL, enterprise integration, complex architecture
**Time**: ~60-120 seconds

```bash
just test-e2e-camel
```

### 4. Concurrent Test

**Purpose**: Validate multiple projects simultaneously
**Projects**: Gson + Commons Lang + OkHttp
**Time**: ~20-40 seconds

```bash
just test-e2e-concurrent
```

**Expected Output**:
```
🚀 Starting E2E test: Concurrent Project Analysis
📦 Analyzing 3 projects concurrently
  ✅ Project gson: 47 Java files
  ✅ Project commons-lang: 589 Java files
  ✅ Project okhttp: 203 Java files
✅ Concurrent analysis completed: 839 files from 3 projects
```

## Requirements

### System Requirements

- **Git**: Must be installed and available in PATH
- **Network**: Internet access to clone from GitHub
- **Rust**: Latest stable toolchain
- **Disk Space**: ~500MB for cloned repositories

### Installing Dependencies

```bash
# Install just (command runner)
curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash

# Verify installation
just --version
```

## Test Methodology

Each E2E test follows this workflow:

```
1. Configure repository URL and expectations
2. Clone repository with shallow clone (--depth 1)
3. Discover all .java files recursively
4. Parse package declarations
5. Analyze Java patterns (annotations, generics, lambdas)
6. Validate hodei-scan can handle the project
7. Clean up temporary files
```

### Code Example

```rust
#[tokio::test]
#[ignore] // Requires network
async fn test_simple_java_library_extraction() {
    // 1. Configure
    let repo_url = "https://github.com/google/guava.git";
    let expected_min_files = 50;
    
    // 2. Clone
    let clone_output = Command::new("git")
        .args(&["clone", "--depth", "1", repo_url, clone_path])
        .output()
        .expect("Git clone failed");
    
    // 3. Discover files
    let java_files = discover_java_files(&clone_path);
    assert!(java_files.len() >= expected_min_files);
    
    // 4. Analyze packages
    let packages = analyze_java_package_structure(&clone_path);
    
    // 5. Validate
    validate_scan_capability(&clone_path, &java_files);
}
```

## Why Tests are Ignored

All E2E tests are marked with `#[ignore]` because:

1. **Network Dependency**: CI environments may not have internet
2. **Time**: 5-120 seconds per test (too slow for CI)
3. **Resource Usage**: Downloads 50-500MB per test
4. **Non-deterministic**: Depends on GitHub availability
5. **Flakiness**: Network issues can cause false failures

## Running in CI

To run E2E tests in CI, use a separate job:

```yaml
e2e-tests:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Install just
      run: curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash
    
    - name: Run E2E tests
      run: just test-e2e-all
      env:
        CARGO_NET_GIT_FETCH_WITH_CLI: true
```

## Troubleshooting

### Git Clone Fails

```bash
# Check git is installed
git --version

# Check network connectivity
ping github.com

# Try manual clone
git clone --depth 1 https://github.com/google/guava.git
```

### Tests Timeout

```bash
# Increase timeout for specific test
RUST_TEST_TIMEOUT=300 just test-e2e-camel
```

### Permission Denied

```bash
# Clean cache
just clean-e2e-cache

# Or manually
rm -rf /tmp/test-*
```

### Want Verbose Output

```bash
# Run with backtrace
RUST_BACKTRACE=1 just test-e2e-guava-verbose

# Or with test output
just test-e2e-guava -- --show-output
```

## Performance Expectations

| Test | Clone Time | Analysis Time | Total |
|------|------------|---------------|-------|
| Guava | 5-15s | 5-15s | 10-30s |
| Spring Boot | 15-30s | 15-30s | 30-60s |
| Camel | 30-60s | 30-60s | 60-120s |
| Gson | 3-10s | 2-5s | 5-15s |
| Concurrent (3x) | 10-20s | 10-20s | 20-40s |
| **All Combined** | - | - | **3-5 minutes** |

## Success Criteria

An E2E test passes if:

1. ✅ Repository clones successfully
2. ✅ Java files are discovered (> minimum expected)
3. ✅ Package structure is parsed
4. ✅ Java patterns are detected
5. ✅ hodei-scan can handle the codebase

## Advanced Usage

### Custom Repository

```bash
# Edit Justfile to add your own repository
test-e2e-custom:
    @echo "🚀 Testing custom repository"
    cd crates/hodei-java-extractor && \
    cargo test test_custom_project -- --ignored --nocapture
```

### Benchmark Mode

```bash
# Time each test
time just test-e2e-all

# Profile memory usage
/usr/bin/time -v just test-e2e-camel
```

### Parallel Execution

```bash
# Run tests in parallel terminals
just test-e2e-guava &
just test-e2e-spring &
just test-e2e-gson &
wait
```

## Contributing

To add a new E2E test:

1. Add test function in `crates/hodei-java-extractor/tests/e2e_github_tests.rs`
2. Mark with `#[ignore]`
3. Add command to `Justfile`
4. Document in this README
5. Test manually with `just test-e2e-<name>`

## Summary

E2E tests ensure `hodei-scan` works with **real-world Java codebases** from GitHub, validating that the scanner can handle diverse patterns, architectures, and scales.

**Run them when you need confidence that hodei-scan works in the real world!** 🎯
