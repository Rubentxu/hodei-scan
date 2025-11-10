# EPIC-13: Testing Strategy & Test Suite

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: Hodei Scan v3.2  
**Dependencias**: Todas las épicas anteriores  
**Owner**: QA Team  
**Prioridad**: Critical

---

## 1. Resumen Ejecutivo

Estrategia de testing comprehensiva: unit tests, integration tests, property tests, fuzzing, benchmarks, E2E tests.

### Objetivo
- Cobertura >90% en código core.
- Property tests para tipos críticos (ProjectPath, Confidence).
- Fuzzing para parsers (DSL, Cap'n Proto).
- Benchmarks automáticos en CI.

---

## 2. Testing Pyramid

```
        E2E Tests (Slow, High Value)
             /\
            /  \
           /    \
          /      \
    Integration Tests
         /        \
        /          \
   Unit Tests (Fast, Focused)
```

---

## 3. Testing Layers

### 3.1. Unit Tests

```rust
// hodei-ir/src/newtypes.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn confidence_rejects_out_of_range() {
        assert!(Confidence::new(-0.1).is_err());
        assert!(Confidence::new(1.1).is_err());
        assert!(Confidence::new(0.5).is_ok());
    }
    
    #[test]
    fn project_path_prevents_traversal() {
        let root = PathBuf::from("/project");
        assert!(ProjectPath::new_relative(&root, "../etc/passwd").is_err());
        assert!(ProjectPath::new_relative(&root, "src/main.rs").is_ok());
    }
}
```

### 3.2. Property Tests (proptest)

```rust
// hodei-ir/tests/property_tests.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn confidence_value_always_in_range(value in 0.0f64..=1.0) {
        let conf = Confidence::new(value).unwrap();
        assert!(conf.value() >= 0.0 && conf.value() <= 1.0);
    }
    
    #[test]
    fn project_path_never_escapes_root(
        root in any::<String>(),
        subpath in any::<String>()
    ) {
        let root_buf = PathBuf::from(root);
        if let Ok(path) = ProjectPath::new_relative(&root_buf, &subpath) {
            let canonical = path.canonical();
            assert!(canonical.starts_with(&root_buf));
        }
    }
    
    #[test]
    fn fact_serialization_roundtrip(fact in fact_strategy()) {
        let json = serde_json::to_string(&fact).unwrap();
        let deserialized: Fact = serde_json::from_str(&json).unwrap();
        assert_eq!(fact, deserialized);
    }
}

fn fact_strategy() -> impl Strategy<Value = Fact> {
    (
        any::<FactType>(),
        any::<Option<SourceLocation>>(),
        confidence_strategy(),
    ).prop_map(|(fact_type, location, confidence)| {
        Fact {
            id: FactId(0),
            fact_type,
            source_location: location,
            confidence,
            provenance: Provenance::default(),
            flow_id: None,
        }
    })
}
```

### 3.3. Fuzzing (cargo-fuzz)

```rust
// fuzz/fuzz_targets/dsl_parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use hodei_dsl::parser::RuleParser;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Parser should never panic, even on malformed input
        let _ = RuleParser::parse_file(s);
    }
});

// fuzz/fuzz_targets/capnp_deserialize.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use hodei_ir::IntermediateRepresentation;

fuzz_target!(|data: &[u8]| {
    // Deserializer should never panic on malformed Cap'n Proto data
    let _ = IntermediateRepresentation::from_capnp_bytes(data);
});

// fuzz/fuzz_targets/project_path.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use hodei_ir::ProjectPath;
use std::path::PathBuf;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let root = PathBuf::from("/tmp/test");
        let _ = ProjectPath::new_relative(&root, s);
    }
});
```

### 3.4. Integration Tests

```rust
// hodei-engine/tests/integration_test.rs
use hodei_engine::RuleEngine;
use hodei_ir::IntermediateRepresentation;
use hodei_dsl::parser::RuleParser;

#[test]
fn end_to_end_vulnerable_uncovered_detection() {
    // 1. Load fixture IR
    let ir = IntermediateRepresentation::load_capnp("fixtures/vulnerable_app.ir")
        .expect("Failed to load fixture IR");
    
    // 2. Parse rule
    let rule_src = r#"
        rule VulnerableUncovered {
            description: "Taint sink in uncovered code"
            severity: Critical
            
            match {
                sink: TaintSink and
                uncovered: UncoveredLine
                where sink.location == uncovered.location
            }
            
            emit Finding {
                message: "Vulnerable code at {sink.location} has no tests"
                confidence: High
            }
        }
    "#;
    
    let rule_file = RuleParser::parse_file(rule_src).expect("Failed to parse rule");
    
    // 3. Evaluate rule
    let engine = RuleEngine::default();
    let result = engine.evaluate(&rule_file.rules, &ir).expect("Engine evaluation failed");
    
    // 4. Assertions
    assert!(result.findings.len() > 0, "Expected at least one finding");
    
    let finding = &result.findings[0];
    assert_eq!(finding.rule_name, "VulnerableUncovered");
    assert_eq!(finding.severity, Severity::Critical);
    assert!(finding.location.is_some());
}

#[test]
fn spatial_join_performance() {
    let ir = generate_large_ir(100_000);  // 100k facts
    
    let rule_src = r#"
        rule SpatialJoinTest {
            match {
                a: TaintSink and
                b: UncoveredLine
                where a.location == b.location
            }
            emit Finding {
                message: "Test"
                confidence: High
            }
        }
    "#;
    
    let rule_file = RuleParser::parse_file(rule_src).unwrap();
    let engine = RuleEngine::default();
    
    let start = std::time::Instant::now();
    let result = engine.evaluate(&rule_file.rules, &ir).unwrap();
    let elapsed = start.elapsed();
    
    assert!(elapsed.as_secs() < 5, "Evaluation took too long: {:?}", elapsed);
}
```

### 3.5. E2E Tests (CLI)

```rust
// hodei-cli/tests/e2e_test.rs
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn cli_check_fails_on_vulnerable_code() {
    let temp = TempDir::new().unwrap();
    
    // Setup fixture project
    setup_vulnerable_project(&temp);
    
    let mut cmd = Command::cargo_bin("hodei").unwrap();
    cmd.arg("check")
        .arg(temp.path())
        .arg("--rules")
        .arg("fixtures/rules");
    
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Gate 'Security Critical' failed"));
}

#[test]
fn cli_extract_generates_valid_ir() {
    let temp = TempDir::new().unwrap();
    setup_test_project(&temp);
    
    let mut cmd = Command::cargo_bin("hodei").unwrap();
    cmd.arg("extract")
        .arg(temp.path())
        .arg("--output")
        .arg(temp.path().join("test.ir"));
    
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("IR saved"));
    
    // Verify IR is valid
    let ir = IntermediateRepresentation::load_capnp(temp.path().join("test.ir")).unwrap();
    assert!(ir.facts.len() > 0);
}
```

### 3.6. Benchmarks (criterion)

```rust
// benches/end_to_end.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_full_scan(c: &mut Criterion) {
    let mut group = c.benchmark_group("FullScan");
    
    for size in [1_000, 10_000, 100_000] {
        let ir = generate_ir_with_facts(size);
        let rules = load_test_rules(100);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            let engine = RuleEngine::default();
            b.iter(|| {
                let result = engine.evaluate(&rules, &ir).unwrap();
                black_box(result)
            });
        });
    }
    
    group.finish();
}

criterion_group!(benches, bench_full_scan);
criterion_main!(benches);
```

---

## 4. CI Testing Pipeline

```yaml
# .github/workflows/test.yml
name: Test Suite

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Run unit tests
        run: cargo test --all-features
      
      - name: Generate coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --output-dir coverage
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: coverage/cobertura.xml
  
  property-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Run property tests
        run: cargo test --test property_tests -- --ignored
  
  fuzzing:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@nightly
      
      - name: Install cargo-fuzz
        run: cargo install cargo-fuzz
      
      - name: Run fuzzing (1 min per target)
        run: |
          for target in fuzz/fuzz_targets/*.rs; do
            target_name=$(basename $target .rs)
            timeout 60 cargo fuzz run $target_name || true
          done
  
  benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Run benchmarks
        run: cargo bench --no-fail-fast
      
      - name: Store benchmark results
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: target/criterion/report/index.html
  
  e2e-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Build CLI
        run: cargo build --release --bin hodei
      
      - name: Run E2E tests
        run: cargo test --test e2e_test
```

---

## 5. Coverage Targets

| Component | Target | Current |
|-----------|--------|---------|
| hodei-ir | 95% | TBD |
| hodei-engine | 90% | TBD |
| hodei-dsl | 95% | TBD |
| hodei-extractors | 85% | TBD |
| hodei-cli | 80% | TBD |
| **Overall** | **90%** | **TBD** |

---

## 6. Plan de Implementación

**Fase 1: Unit & Property Tests** (Semana 1-2)
- [ ] Unit tests para todos los módulos core.
- [ ] Property tests con proptest.

**Fase 2: Fuzzing** (Semana 2)
- [ ] Fuzz targets para parsers.
- [ ] CI integration (1min fuzzing per PR).

**Fase 3: Integration & E2E** (Semana 3)
- [ ] Integration tests con fixtures.
- [ ] E2E tests CLI.

**Fase 4: Benchmarks** (Semana 3-4)
- [ ] Benchmarks con criterion.
- [ ] Automated performance regression detection.

---

## 7. Criterios de Aceptación

- [ ] Cobertura >90%.
- [ ] Property tests para tipos críticos.
- [ ] Fuzzing automatizado en CI.
- [ ] Benchmarks con regression detection.
- [ ] E2E tests funcionales.

---

**Última Actualización**: 2025-01-XX
