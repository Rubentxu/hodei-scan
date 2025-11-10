# hodei-scan Justfile
# Common commands for development, testing, and CI/CD

# Default target
_default:
    @just --list

# ==========================================
# TESTING COMMANDS
# ==========================================

# Run all tests
test-all:
    cargo test

# Run all tests with verbose output
test-verbose:
    cargo test --verbose

# Run all tests and show output
test-quiet:
    cargo test --quiet

# Run tests in parallel
test-parallel:
    cargo test -- --test-threads=auto

# Run tests with coverage
test-coverage:
    cargo install cargo-tarpaulin
    cargo tarpaulin --out html --output-dir coverage
    echo "Coverage report generated in coverage/tarpaulin-report.html"

# Run all workspace tests
test-workspace:
    cargo test --workspace

# Run specific crate tests
test-engine:
    cargo test -p hodei-engine

test-extractors:
    cargo test -p hodei-extractors

test-metrics:
    cargo test -p hodei-metrics

test-persistence:
    cargo test -p hodei-persistence

test-dsl:
    cargo test -p hodei-dsl

# Run unit tests only
test-unit:
    cargo test --lib

# Run integration tests only
test-integration:
    cargo test --test '*'

# Run E2E tests (real-world testing with actual Java projects)
test-e2e:
    cargo test --test e2e

# Run E2E tests with verbose output
test-e2e-verbose:
    cargo test --test e2e -- --nocapture

# Run specific E2E test
test-e2e-name name:
    cargo test --test e2e {{name}} -- --nocapture

# Run performance tests
test-perf:
    cargo test --test performance

# Run all tests including E2E
test-all-types: test-integration test-e2e test-perf
    echo "✅ All test types passed!"

# Run doc tests
test-docs:
    cargo test --doc

# Run tests for a specific module
test-module module:
    cargo test -p hodei-engine {{module}}

# Run tests and generate junit output (for CI)
test-junit:
    cargo install cargo-junit
    cargo test --junit junit.xml

# Run tests with custom thread count
test-threads n:
    cargo test -- --test-threads={{n}}

# ==========================================
# CODE QUALITY COMMANDS
# ==========================================

# Run clippy linting
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run clippy with fixes
lint-fix:
    cargo clippy --all-targets --all-features -- -D warnings --fix

# Run clippy for specific crate
lint-engine:
    cargo clippy -p hodei-engine

# Run rustfmt check
fmt-check:
    cargo fmt --all -- --check

# Fix formatting
fmt:
    cargo fmt --all

# Run all quality checks
quality: fmt-check lint
    echo "✅ All quality checks passed!"

# ==========================================
# BUILD COMMANDS
# ==========================================

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Build for specific crate
build-engine:
    cargo build -p hodei-engine

# Build all workspace crates
build-workspace:
    cargo build --workspace

# Clean build artifacts
clean:
    cargo clean
    cargo clean -p hodei-engine
    cargo clean -p hodei-extractors
    cargo clean -p hodei-metrics
    cargo clean -p hodei-persistence
    cargo clean -p hodei-dsl
    cargo clean -p hodei-cli

# ==========================================
# DOCUMENTATION COMMANDS
# ==========================================

# Generate documentation
docs:
    cargo doc --no-deps --open

# Generate documentation without opening
docs-no-open:
    cargo doc --no-deps

# Generate all workspace documentation
docs-workspace:
    cargo doc --workspace --no-deps

# Watch documentation for changes
docs-watch:
    cargo doc --no-deps --open --watch

# ==========================================
# BENCHMARKING COMMANDS
# ==========================================

# Run benchmarks
bench:
    cargo bench

# Run specific benchmark
bench-name name:
    cargo bench -- {{name}}

# Run benchmarks and compare
bench-compare:
    cargo bench -- --baseline master

# ==========================================
# SECURITY COMMANDS
# ==========================================

# Run security audit
audit:
    cargo install cargo-audit
    cargo audit

# Check for deny warnings
deny:
    cargo install cargo-deny
    cargo deny check

# Scan for secrets
secrets-scan:
    cargo install secrets-scan
    secrets-scan

# ==========================================
# DEPENDENCY COMMANDS
# ==========================================

# Update dependencies
update:
    cargo update

# Show dependency tree
deps-tree:
    cargo tree

# Show outdated dependencies
deps-outdated:
    cargo install cargo-outdated
    cargo outdated

# Check for unused dependencies
deps-unused:
    cargo install cargo-udeps
    cargo udeps

# ==========================================
# CLEANUP COMMANDS
# ==========================================

# Remove all build artifacts and dependencies
distclean: clean
    rm -rf target/
    rm -rf Cargo.lock

# Prune cargo registry
prune:
    cargo prune

# Remove temporary files
clean-temp:
    find . -name "*.tmp" -delete
    find . -name ".DS_Store" -delete
    find . -name "Thumbs.db" -delete

# ==========================================
# CI/CD COMMANDS
# ==========================================

# Full CI pipeline (includes all test types)
ci:
    just test-all-types
    just quality
    just audit
    just docs
    echo "✅ All CI checks passed!"

# Quick CI (faster, less thorough)
ci-quick:
    just test-workspace
    just lint
    echo "✅ Quick CI passed!"

# CI for pull requests (includes E2E tests)
ci-pr:
    just test-all-types
    just quality
    echo "✅ PR CI passed!"

# Full CI with detailed E2E test output
ci-e2e:
    just test-integration
    just test-e2e-verbose
    just test-perf
    just quality
    just audit
    echo "✅ Full CI with E2E tests passed!"

# Pre-commit checks
pre-commit:
    just test-workspace
    just quality
    just fmt
    echo "✅ Pre-commit checks passed!"

# Release preparation
release-prep:
    just ci
    just build-release
    just changelog
    echo "✅ Release preparation complete!"

# ==========================================
# DEVELOPMENT WORKFLOW
# ==========================================

# Watch mode for development
watch:
    cargo watch -x "test --lib" -x "clippy --all-targets --all-features -- -D warnings"

# Watch tests only
watch-test:
    cargo watch -x "test --lib"

# Watch build only
watch-build:
    cargo watch -x build

# Development mode (build + test on change)
dev:
    cargo watch -x build -x test --lib

# ==========================================
# UTILITY COMMANDS
# ==========================================

# Check Rust version
check-rust:
    rustc --version
    cargo --version

# Show workspace info
info:
    cargo info

# Check formatting on changed files only
fmt-changed:
    cargo fmt --all

# Run a specific test by name
test-name name:
    cargo test {{name}}

# Install development tools
install-tools:
    cargo install cargo-tarpaulin cargo-audit cargo-deny cargo-outdated cargo-udeps cargo-watch cargo-junit
    echo "✅ All development tools installed!"

# ==========================================
# EPIC-SPECIFIC COMMANDS
# ==========================================

# Test EPIC-06: Rule Engine
test-epic-06:
    cargo test -p hodei-engine --lib -- engine::evaluator
    echo "✅ EPIC-06 tests passed!"

# Test EPIC-07: Extractors
test-epic-07:
    cargo test -p hodei-extractors
    echo "✅ EPIC-07 tests passed!"

# Test EPIC-08: Quality Gates
test-epic-08:
    cargo test -p hodei-engine --lib -- gates
    echo "✅ EPIC-08 tests passed!"

# Test all core epics
test-core:
    just test-engine
    just test-extractors
    just test-metrics
    just test-persistence
    echo "✅ All core epics tested!"

# ==========================================
# REPORTING COMMANDS
# ==========================================

# Generate test report
report-tests:
    cargo test -- --report=json test-results.json
    echo "Test report saved to test-results.json"

# Generate coverage badge
coverage-badge:
    cargo install cargo-badge
    cargo badge coverage --format markdown --output README-COVERAGE.md

# Show test summary
summary:
    cargo test -- --report summary
    @echo ""
    @echo "Test Summary:"
    @cargo test -- --list | grep "test result:" | tail -1
