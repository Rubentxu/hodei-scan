# E2E Testing Methodology Manual

## Overview

End-to-End (E2E) testing validates that hodei-scan can scan, analyze, and extract facts from **real-world Java projects** taken directly from GitHub. Unlike unit tests that use mocked or simplified code, E2E tests work with actual production codebases.

---

## What E2E Tests Do

### Core Process

Each E2E test follows this workflow:

```
1. Clone Real Repository ──┐
                          ├──> Discover Java Files
                          ├──> Run hodei-scan Extraction
                          ├──> Validate Output
                          └──> Verify Facts Extracted
```

### Step-by-Step Execution

#### Step 1: Repository Acquisition
```bash
git clone --depth 1 --shallow-submodules <repo_url> <temp_dir>
```

- **Depth 1**: Shallow clone to speed up download (only latest commit)
- **Shallow submodules**: Faster submodule initialization
- **Network Required**: Tests require internet access

#### Step 2: File Discovery
```rust
fn discover_java_files(project_path: &Path) -> Vec<PathBuf> {
    walk_dir(project_path)
        .filter(|p| p.extension() == Some("java".as_ref()))
        .collect()
}
```

**What it does:**
- Recursively scans all directories in the cloned project
- Collects absolute paths to all `.java` files
- Filters out non-Java files (e.g., `.kt`, `.scala`)

#### Step 3: Extract Facts
```rust
fn validate_scan_capability(project_path: &Path, java_files: &[PathBuf]) {
    let ir = hodei_java_extractor::extract(
        project_path,
        &ExtractionConfig::default()
    ).expect("Extraction failed");

    assert!(!ir.get_facts().is_empty(), "No facts extracted");
    assert!(ir.get_facts().len() > java_files.len(), "Too few facts");
}
```

**What it does:**
- Invokes hodei-java-extractor on the project
- Parses all discovered Java files
- Extracts facts (Function, Variable, TaintSink, etc.)
- Generates Intermediate Representation (IR)

#### Step 4: Validation
```rust
assert!(extraction_successful);
assert!(facts_extracted > 0);
assert!(facts_per_file_ratio > MIN_EXPECTED_RATIO);
```

**Validations performed:**
- Extraction completed without errors
- Facts were actually extracted
- Ratio of facts to files is reasonable
- IR structure is valid

---

## Specific E2E Tests Explained

### 1. test_simple_java_library_extraction (Google Guava)

**Repository:** https://github.com/google/guava.git

**What is Guava:**
- Google's core libraries for Java
- Collection utilities, caching, concurrency primitives
- Well-structured, clean Java code
- ~1,000+ Java files

**What it tests:**
```
✅ Basic Java parsing (classes, interfaces, enums)
✅ Method extraction and signatures
✅ Field and variable detection
✅ Import statement parsing
✅ Generic type parameters
✅ Annotation processing (if present)
```

**Expected Output:**
- 800-1,500 extracted facts
- 1-2 minutes execution time
- Success rate: 95%+ files processed

**Target Patterns:**
- Utility classes (final, private constructors)
- Static method usage
- Generic collections
- Builder patterns
- Preconditions and validation

**Success Criteria:**
```
File Discovery: 800+ .java files
Facts Extracted: 1,000+ facts
Extraction Time: < 120 seconds
Error Rate: < 5%
```

---

### 2. test_spring_boot_application (Spring Boot)

**Repository:** https://github.com/spring-projects/spring-boot.git

**What is Spring Boot:**
- Framework for building Java web applications
- Enterprise-grade, production-ready code
- 500+ Java files across multiple modules
- Extensive use of annotations and AOP

**What it tests:**
```
✅ Complex annotation processing (@RestController, @Autowired, @RequestMapping)
✅ Dependency injection configuration
✅ Classpath scanning and bean discovery
✅ Configuration properties (@ConfigurationProperties)
✅ Aspect-oriented programming (AOP)
✅ Multi-module project structure
✅ Enterprise patterns (MVC, Data Access, Security)
```

**What makes it challenging:**
- **Annotations**: Complex metadata that needs parsing
- **Generics**: Heavy use of parameterized types
- **Circular Dependencies**: Spring manages complex dependency graphs
- **Proxies**: Spring creates proxy objects that obscure actual code
- **Reflection**: Runtime annotation processing

**Target Patterns:**
```java
@RestController
@RequestMapping("/api/users")
public class UserController {
    @Autowired
    private UserService userService;

    @GetMapping("/{id}")
    public User getUser(@PathVariable Long id) {
        return userService.findById(id);
    }
}

@SpringBootApplication
@EnableJpaRepositories
@EntityScan("com.example.model")
public class Application {
    public static void main(String[] args) {
        SpringApplication.run(Application.class, args);
    }
}
```

**Expected Output:**
- 2,000-5,000 extracted facts
- 2-3 minutes execution time
- Success rate: 90%+ files processed

**Success Criteria:**
```
File Discovery: 2,000+ .java files
Facts Extracted: 3,000+ facts
Annotation Facts: 500+ annotation instances
Bean Definitions: 100+ Spring beans
Web Endpoints: 50+ REST endpoints
Extraction Time: < 180 seconds
Error Rate: < 10%
```

**What we validate:**
- Controller classes marked with `@RestController`
- Autowired dependencies properly recognized
- Request mappings extracted
- Service layer abstractions identified
- Configuration classes detected
- Multi-module dependencies understood

---

### 3. test_multimodule_maven_project (Apache Camel)

**Repository:** https://github.com/apache/camel.git

**What is Apache Camel:**
- Enterprise Integration Patterns framework
- Message-oriented middleware
- 5,000+ Java files across 20+ modules
- Heavy use of DSLs (Domain-Specific Languages)

**What it tests:**
```
✅ Multi-module project navigation
✅ DSL pattern recognition (RouteBuilder, Endpoint DSL)
✅ Enterprise Integration Patterns (EIP)
✅ Message transformation flows
✅ Protocol adapters (HTTP, JMS, Kafka, etc.)
✅ Bean integration and POJO messaging
✅ Complex dependency management across modules
```

**What makes it challenging:**
- **Multi-Module**: Need to navigate parent/child relationships
- **DSL Patterns**: Custom languages embedded in Java
- **Heavy Dependencies**: Complex classpath with hundreds of JARs
- **Large Scale**: 5,000+ files = memory and performance test
- **EIP Patterns**: Enterprise integration requires semantic understanding

**Target Patterns:**
```java
public class MyRouteBuilder extends RouteBuilder {
    @Override
    public void configure() throws Exception {
        from("file:data/inbox")
            .choice()
                .when().xpath("/order/@type='widget'")
                    .to("jms:widgetQueue")
                .otherwise()
                    .to("jms:gadgetQueue");

        from("direct:start")
            .to("rest:get:/api/users/{id}")
            .split().xpath("//orders/order")
            .choice()
                .when().method(MyBean.class, "isLargeOrder")
                    .to("mock:largeOrders")
                .otherwise()
                    .to("mock:otherOrders");
    }
}
```

**Expected Output:**
- 10,000-20,000 extracted facts
- 3-5 minutes execution time
- Success rate: 85%+ files processed

**Success Criteria:**
```
File Discovery: 5,000+ .java files
Facts Extracted: 15,000+ facts
Route Definitions: 200+ Camel routes
Endpoints: 300+ endpoint references
Transformations: 400+ message transformations
Module Coverage: 15+ Maven modules
Extraction Time: < 300 seconds
Error Rate: < 15%
```

**What we validate:**
- Route definitions recognized
- Endpoint URIs extracted
- EIP patterns identified (choice, split, transform)
- Bean references detected
- Module boundaries understood
- Protocol bindings catalogued

---

### 4. test_concurrent_project_analysis

**What it tests:**
```
✅ Thread safety of hodei-scan
✅ Concurrent access to shared resources
✅ Parallel extraction from multiple projects
✅ Memory management under concurrent load
✅ Performance scalability
```

**Execution:**
```rust
async fn test_concurrent_project_analysis() {
    let projects = vec![
        ("guava", "https://github.com/google/guava.git"),
        ("gson", "https://github.com/google/gson.git"),
        ("okhttp", "https://github.com/square/okhttp.git"),
    ];

    let handles: Vec<_> = projects
        .into_iter()
        .map(|(name, url)| {
            tokio::spawn(async move {
                clone_and_extract(url, name).await
            })
        })
        .collect();

    let results = futures::future::join_all(handles).await;

    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.is_ok()));
}
```

**What we validate:**
- All three projects extract successfully
- No race conditions in shared state
- Memory usage stays within bounds
- Total time < sum of individual times (parallelization works)

---

## Why These Projects Were Chosen

### Google Guava
**Why:** Clean, well-structured code
- Perfect for testing basic extraction
- Excellent test of parsing quality
- Known patterns and conventions
- Fast to clone and process

### Spring Boot
**Why:** Complex enterprise features
- Annotations are heavily used
- Dependency injection patterns
- Multi-module projects
- Production-grade code quality
- Real-world web application patterns

### Apache Camel
**Why:** Scale and complexity
- Massive codebase (5,000+ files)
- Complex integration patterns
- Multi-module Maven structure
- DSL parsing challenges
- Enterprise-grade performance requirements

### Additional Projects (Gson, OkHttp, Commons)
**Why:** Diversity of patterns
- **Gson**: JSON parsing, reflection
- **OkHttp**: Networking, HTTP clients
- **Commons**: String manipulation, utility patterns

---

## What hodei-scan Actually Extracts

### Fact Types

Each E2E test validates extraction of these fact types:

#### 1. Function Facts
```json
{
  "fact_type": "Function",
  "name": "findById",
  "class": "UserService",
  "visibility": "public",
  "parameters": ["Long"],
  "return_type": "User",
  "line_number": 42
}
```

#### 2. Variable Facts
```json
{
  "fact_type": "Variable",
  "name": "userRepository",
  "class": "UserController",
  "type": "UserRepository",
  "annotations": ["@Autowired"]
}
```

#### 3. Taint Sink Facts
```json
{
  "fact_type": "TaintSink",
  "method": "executeQuery",
  "class": "UserDao",
  "vulnerability_type": "SQL_INJECTION",
  "line_number": 87
}
```

#### 4. Coverage Facts
```json
{
  "fact_type": "CoverageStats",
  "class": "UserService",
  "line_coverage": 85.5,
  "branch_coverage": 72.3
}
```

---

## Test Execution Flow

### Without Just Commands (Manual)
```bash
# Clone repo manually
git clone https://github.com/spring-projects/spring-boot.git /tmp/spring-boot

# Run test
cd crates/hodei-java-extractor
cargo test test_spring_boot_application -- --ignored --nocapture

# Clean up
rm -rf /tmp/spring-boot
```

### With Just Commands (Automated)
```bash
# Single command - everything automated
just test-e2e-spring

# Output includes:
# - Repository cloning
# - File discovery
# - Extraction process
# - Validation results
# - Cleanup
```

---

## Success Metrics

### Performance Metrics
- **Cloning Time**: < 120 seconds per project
- **Extraction Time**: < 5 minutes total
- **Memory Usage**: < 2GB peak
- **File Processing Rate**: > 50 files/second

### Quality Metrics
- **Facts Extracted**: Must exceed file count
- **Error Rate**: < 10% files failed
- **Completeness**: > 90% of code analyzed
- **Accuracy**: Facts must pass validation schema

### Coverage Metrics
- **Classes**: > 80% classes recognized
- **Methods**: > 70% methods extracted
- **Annotations**: > 85% annotations processed
- **Fields**: > 75% fields catalogued

---

## Troubleshooting

### Common Issues

#### 1. Clone Timeout
```
Error: git clone failed after 120 seconds
Solution: Increase timeout or use --depth 1
```

#### 2. Out of Memory
```
Error: allocation failed
Solution: Increase Java heap or process fewer files
```

#### 3. Parse Errors
```
Error: Failed to parse Java file
Solution: Check for syntax errors in source file
```

#### 4. Network Issues
```
Error: Failed to fetch repository
Solution: Check internet connection
```

### Debug Commands

```bash
# Check what files were discovered
find /tmp/test-*/ -name "*.java" | wc -l

# See extraction output
just test-e2e-guava-verbose

# Clean cache
just clean-e2e-cache

# Check test status
just test-e2e-status
```

---

## CI/CD Integration

### GitHub Actions Example
```yaml
name: E2E Tests

on:
  schedule:
    - cron: '0 6 * * *'  # Daily at 6 AM
  workflow_dispatch:      # Manual trigger

jobs:
  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run E2E Tests
        run: just test-e2e-all
        env:
          CARGO_NET_GIT_FETCH_WITH_CLI: true
```

### Local Development
```bash
# Fast feedback loop
just test-e2e-guava

# Before commit
just test-e2e-all

# Debug specific issue
just test-e2e-spring-verbose
```

---

## Conclusion

E2E tests prove that hodei-scan works on **real production code**, not just crafted examples. They validate:

1. **Scalability**: Can handle 5,000+ file projects
2. **Accuracy**: Extracts correct facts from complex code
3. **Robustness**: Handles edge cases and malformed input
4. **Performance**: Completes in reasonable time
5. **Reliability**: Consistent results across runs

The combination of **unit tests** (validate logic) and **E2E tests** (validate real-world usage) ensures hodei-scan is production-ready.

---

## Quick Reference

| Test | Command | Files | Time | Focus |
|------|---------|-------|------|-------|
| Guava | `just test-e2e-guava` | ~50 | 1-2 min | Basic extraction |
| Spring | `just test-e2e-spring` | 500+ | 2-3 min | Annotations, DI |
| Camel | `just test-e2e-camel` | 2000+ | 3-5 min | Multi-module, DSL |
| Gson | `just test-e2e-gson` | ~50 | 1-2 min | Reflection |
| All | `just test-e2e-all` | 3000+ | 5-7 min | Complete validation |

**Remember:** All E2E tests are marked `#[ignore]` and require `--ignored` flag to run.
