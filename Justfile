# =============================================================================
# Justfile - Commands for E2E Testing with Real Java Projects
# =============================================================================
#
# Usage:
#   just test-e2e-guava        # Test with Google Guava
#   just test-e2e-spring       # Test with Spring Boot
#   just test-e2e-camel        # Test with Apache Camel
#   just test-e2e-all          # Test all E2E projects
#
# Requirements:
#   - git installed
#   - network access
#   - Rust toolchain
#
# =============================================================================

# Show help
help:
    @echo "🚀 E2E Testing Commands"
    @echo "========================"
    @echo ""
    @echo "Individual Tests:"
    @echo "  just test-e2e-guava      Test with Google Guava"
    @echo "  just test-e2e-spring     Test with Spring Boot"
    @echo "  just test-e2e-camel      Test with Apache Camel"
    @echo "  just test-e2e-gson       Test with Google Gson"
    @echo "  just test-e2e-commons    Test with Apache Commons Lang"
    @echo "  just test-e2e-okhttp     Test with Square OkHttp"
    @echo ""
    @echo "Batch Tests:"
    @echo "  just test-e2e-all        Test all E2E projects"
    @just --list

# =============================================================================
# E2E TESTS WITH REAL JAVA PROJECTS
# =============================================================================

# Test with Google Guava (Simple Java library)
# ~50 files, basic patterns, fast clone
test-e2e-guava:
    @echo "🚀 Starting E2E test: Google Guava"
    @echo "📦 Repository: https://github.com/google/guava.git"
    @echo "⏱️  Expected time: 10-30 seconds"
    @echo ""
    cd crates/hodei-java-extractor && \
    cargo test test_simple_java_library_extraction -- --ignored --nocapture

# Test with Spring Boot (Web framework)
# 500+ files, annotations, dependency injection
test-e2e-spring:
    @echo "🚀 Starting E2E test: Spring Boot"
    @echo "📦 Repository: https://github.com/spring-projects/spring-boot.git"
    @echo "⏱️  Expected time: 30-60 seconds"
    @echo ""
    cd crates/hodei-java-extractor && \
    cargo test test_spring_boot_application -- --ignored --nocapture

# Test with Apache Camel (Enterprise integration)
# 2000+ files, multi-module, DSL patterns
test-e2e-camel:
    @echo "🚀 Starting E2E test: Apache Camel"
    @echo "📦 Repository: https://github.com/apache/camel.git"
    @echo "⏱️  Expected time: 60-120 seconds"
    @echo ""
    cd crates/hodei-java-extractor && \
    cargo test test_multimodule_maven_project -- --ignored --nocapture

# Test with Google Gson (JSON library)
# ~50 files, reflection patterns
test-e2e-gson:
    @echo "🚀 Starting E2E test: Google Gson"
    @echo "📦 Repository: https://github.com/google/gson.git"
    @echo "⏱️  Expected time: 5-15 seconds"
    @echo ""
    cd crates/hodei-java-extractor && \
    cargo test test_concurrent_project_analysis -- --ignored --nocapture

# Test with Apache Commons Lang (Utility library)
# 500+ files, string manipulation
test-e2e-commons:
    @echo "🚀 Starting E2E test: Apache Commons Lang"
    @echo "📦 Repository: https://github.com/apache/commons-lang.git"
    @echo "⏱️  Expected time: 20-40 seconds"
    @echo ""
    cd crates/hodei-java-extractor && \
    cargo test test_concurrent_project_analysis -- --ignored --nocapture

# Test with Square OkHttp (HTTP client)
# 200+ files, networking patterns
test-e2e-okhttp:
    @echo "🚀 Starting E2E test: Square OkHttp"
    @echo "📦 Repository: https://github.com/square/okhttp.git"
    @echo "⏱️  Expected time: 15-30 seconds"
    @echo ""
    cd crates/hodei-java-extractor && \
    cargo test test_concurrent_project_analysis -- --ignored --nocapture

# =============================================================================
# BATCH TESTING
# =============================================================================

# Run all E2E tests sequentially
# This will take 3-5 minutes total
test-e2e-all:
    @echo "🎯 Running ALL E2E Tests"
    @echo "========================="
    @echo "⏱️  Total expected time: 3-5 minutes"
    @echo ""
    @just test-e2e-guava
    @echo ""
    @just test-e2e-spring
    @echo ""
    @just test-e2e-camel
    @echo ""
    @just test-e2e-gson
    @echo ""
    @echo "✅ All E2E tests completed!"

# Run E2E tests in parallel (faster but less detailed)
test-e2e-parallel:
    @echo "🎯 Running E2E Tests in PARALLEL"
    @echo "================================="
    @echo "⚠️  This will use more network and CPU"
    @echo ""
    cd crates/hodei-java-extractor && \
    cargo test --test e2e_github_tests -- --ignored --nocapture

# =============================================================================
# PERFORMANCE TESTING
# =============================================================================

# Test with a large project (Apache Camel)
# Validates performance with 2000+ files
test-e2e-performance:
    @echo "⚡ E2E Performance Test"
    @echo "========================"
    @echo "Testing with Apache Camel (2000+ files)"
    @just test-e2e-camel

# Test with multiple projects concurrently
test-e2e-concurrent:
    @echo "⚡ E2E Concurrent Test"
    @echo "======================"
    @echo "Testing 3 projects concurrently"
    cd crates/hodei-java-extractor && \
    cargo test test_concurrent_project_analysis -- --ignored --nocapture

# =============================================================================
# DEVELOPMENT HELPERS
# =============================================================================

# Clean all cloned repositories
clean-e2e-cache:
    @echo "🧹 Cleaning E2E cache..."
    @rm -rf /tmp/test-*
    @rm -rf /tmp/e2e-*
    @echo "✅ Cache cleaned"

# Show E2E test status
test-e2e-status:
    @echo "📊 E2E Test Status"
    @echo "=================="
    @echo ""
    @echo "Implemented Tests:"
    @echo "  ✅ test_simple_java_library_extraction (Guava)"
    @echo "  ✅ test_spring_boot_application (Spring Boot)"
    @echo "  ✅ test_multimodule_maven_project (Apache Camel)"
    @echo "  ✅ test_concurrent_project_analysis (3 repos)"
    @echo ""
    @echo "Ignored Tests (require network):"
    @echo "  ⏸️  All E2E tests marked with #[ignore]"
    @echo ""
    @echo "To run: just test-e2e-<project-name>"

# Run with detailed output
test-e2e-guava-verbose:
    @echo "🚀 Starting E2E test: Google Guava (VERBOSE)"
    @RUST_BACKTRACE=1 cd crates/hodei-java-extractor && \
    cargo test test_simple_java_library_extraction -- --ignored --nocapture -- --show-output

# =============================================================================
# VALIDATION COMMANDS
# =============================================================================

# Validate workspace compiles
validate-workspace:
    @echo "🔍 Validating workspace compilation..."
    cargo check --workspace
    @echo "✅ Workspace compiles successfully"

# Run all fast tests (no network required)
test-fast:
    @echo "⚡ Running fast tests (no network)..."
    cargo test -p hodei-java-extractor \
      --lib \
      --test property_tests \
      --test contract_tests \
      --test integration_tests \
      --test mutation_tests
    @echo "✅ All fast tests passed"

# Full test suite
test-all: validate-workspace test-fast
    @echo ""
    @echo "🎯 Running full test suite..."
    @cargo test --workspace
    @echo ""
    @echo "✅ All tests passed!"
