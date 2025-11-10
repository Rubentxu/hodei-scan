# Metodología TDD/BDD para hodei-scan
## Desarrollo Guiado por Tests con Kotest

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 📋 Active
**Alcance:** Todo el proyecto hodei-scan

---

## 📋 Resumen Ejecutivo

Este documento establece la metodología de **Test-Driven Development (TDD)** y **Behavior-Driven Development (BDD)** para el proyecto hodei-scan. Todos los desarrollos deben seguir el ciclo Red-Green-Refactor con tests escritos **antes** de la implementación.

**Principio Fundamental:**
> **NINGUNA IMPLEMENTACIÓN SIN TEST PRIMERO** - Cada línea de código productivo debe estar precedida por un test fallido.

---

## 🔄 Ciclo TDD Estándar

### Los 3 Pasos (Red-Green-Refactor)

```
1. 🔴 RED    → Escribir test que falla
2. 🟢 GREEN  → Escribir código mínimo para que pase
3. 🔵 REFACTOR → Mejorar código manteniendo tests verdes
```

### Reglas de Oro

1. **NUNCA escribir código productivo sin test que falle primero**
2. **NUNCA escribir más de un test a la vez**
3. **NUNCA escribir más código del necesario para pasar el test**
4. **REFACTORIZAR solo cuando TODOS los tests estén verdes**

---

## 🧪 Framework de Testing: Kotest

### Configuración en Cargo.toml

```toml
[dev-dependencies]
# TDD/BDD Testing Framework
kotest = "0.10"
kotest-async = "0.10"

# Test utilities
mockall = "0.12"
tempfile = "3.0"
proptest = "1.4"  # Property-based testing

# Test organization
criterion = { version = "0.5", features = ["html_reports"] }
```

### Estructura Base de Tests

```rust
// tests/analyzer_rust_tests.rs
use kotei_scan::analyzers::rust::RustAnalyzer;
use kotei_scan::models::{AnalysisContext, Issue, Severity};
use tempfile::NamedTempFile;
use std::io::Write;

#[cfg(test)]
mod rust_analyzer_tests {
    use super::*;

    // Test Module por Feature
    mod parsing {
        use super::*;

        // Test con Kotest
        #[kotest::test]
        fn should_parse_simple_function() {
            //Arrange
            let code = r#"
                fn main() {
                    println!("Hello, World!");
                }
            "#;
            let analyzer = RustAnalyzer::new();

            //Act
            let result = analyzer.parse(code);

            //Assert
            result.should_be_ok();
            let ast = result.unwrap();
            ast.functions.should_have_length(1);
            ast.functions[0].name.should_equal("main");
        }
    }

    // Test Module para cada feature
    mod semantic_analysis {
        // Tests aquí
    }

    mod cfg_builder {
        // Tests aquí
    }

    mod data_flow_analysis {
        // Tests aquí
    }
}
```

---

## 📝 Convenciones de Naming

### Test Naming Pattern

**Formato:** `should_expected_behavior_when_condition`

```rust
// ✅ BUENOS nombres de test
#[kotest::test]
fn should_parse_rust_function_with_parameters() { }

#[kotest::test]
fn should_detect_sql_injection_when_user_input_concatenated() { }

#[kotest::test]
fn should_calculate_technical_debt_with_nist_rates() { }

// ❌ MALOS nombres de test
#[kotest::test]
fn test_parse() { }  // Too vague

#[kotest::test]
fn test_function() { }  // No behavior described

#[kotest::test]
fn check_sql() { }  // No scenario
```

### Test Organization

```
tests/
├── unit/
│   ├── analyzers/
│   │   ├── rust/
│   │   │   ├── parsing_tests.rs
│   │   │   ├── semantic_analysis_tests.rs
│   │   │   └── cfg_builder_tests.rs
│   │   ├── go/
│   │   └── typescript/
│   ├── security/
│   │   ├── rule_engine_tests.rs
│   │   ├── taint_analysis_tests.rs
│   │   └── owasp_top10_tests.rs
│   └── sca/
│       ├── dependency_resolver_tests.rs
│       ├── cve_scanner_tests.rs
│       └── sbom_generator_tests.rs
├── integration/
│   ├── full_analysis_tests.rs
│   ├── ci_cd_integration_tests.rs
│   └── end_to_end_tests.rs
├── performance/
│   ├── benchmark_analysis_tests.rs
│   ├── memory_usage_tests.rs
│   └── scalability_tests.rs
└── fixtures/
    ├── rust_projects/
    ├── go_projects/
    ├── vulnerable_dependencies/
    └── security_test_cases/
```

---

## 🎯 Tipos de Tests

### 1. Unit Tests (70% del total)

**Objetivo:** Probar componentes individuales en aislamiento

```rust
#[kotest::test]
fn should_identify_unsafe_block_without_safety_comment() {
    // Arrange
    let code = r#"
        unsafe fn dangerous_operation() -> i32 {
            *ptr
        }
    "#;

    // Act
    let findings = analyze_security(code, "rust");

    // Assert
    findings.should_have_length(1);
    findings[0].rule_id.should_equal("RUST_UNSAFE_NO_COMMENT");
    findings[0].severity.should_equal(Severity::Warning);
}
```

### 2. Integration Tests (20% del total)

**Objetivo:** Probar interacción entre componentes

```rust
#[tokio::test]
async fn should_analyze_project_end_to_end() {
    // Arrange
    let project_path = PathBuf::from("tests/fixtures/rust/project1");
    let analyzer = ProjectAnalyzer::new();

    // Act
    let report = analyzer.analyze_project(&project_path).await.unwrap();

    // Assert
    report.summary.total_issues.should_be_greater_than(0);
    report.coverage.line_coverage.should_be_some();
    report.security.vulnerabilities.should_not_be_empty();
}
```

### 3. Property-Based Tests (5% del total)

**Objetivo:** Probar invariantes con datos generados automáticamente

```rust
#[proptest]
fn parse_rust_function_properties(idents: Vec<Identifier>) {
    let code = format!("fn {}({{}}) {{}}", idents.join("_"));
    let result = parse_rust(&code);

    result.should_be_ok();
    let ast = result.unwrap();
    ast.functions.should_have_length(1);
    ast.functions[0].name.should_equal(idents[0]);
}
```

### 4. Performance Tests (5% del total)

**Objetivo:** Validar performance requirements

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn should_analyze_100k_loc_in_under_2_minutes() {
        let start = Instant::now();
        let result = analyze_large_project("tests/fixtures/100k-loc");

        let duration = start.elapsed();
        duration.should_be_less_than(Duration::from_secs(120));
        result.should_be_ok();
    }

    #[test]
    fn should_use_less_than_800mb_for_1m_loc() {
        let before = get_memory_usage_bytes();
        let _result = analyze_large_project("tests/fixtures/1m-loc");
        let after = get_memory_usage_bytes();

        let used = after - before;
        used.should_be_less_than(800 * 1024 * 1024);
    }
}
```

---

## 📋 BDD con Gherkin

### Usando Gherkin para Definir Comportamiento

**Archivos .feature en `tests/bdd/`**

```gherkin
# tests/bdd/sql_injection_detection.feature
Feature: SQL Injection Detection

  Scenario: User input concatenated directly into SQL query
    Given code with user input variable "user_id"
    And a SQL query concatenating the variable
    When the security analyzer runs
    Then it should detect SQL Injection vulnerability
    And it should suggest using prepared statements

  Scenario: User input sanitized before SQL query
    Given code with user input variable
    And input is sanitized with a known safe function
    And a SQL query using the sanitized input
    When the security analyzer runs
    Then it should NOT detect SQL Injection
    And it should log the sanitization path

  Scenario: Query using only constant values
    Given code with SQL query using only constants
    When the security analyzer runs
    Then it should NOT detect SQL Injection
    And it should recognize no user input is used
```

**Parser de Gherkin en Tests**

```rust
// tests/bdd/gherkin_runner.rs
use gherkin::{Parser, Feature};

#[kotest::test]
fn test_sql_injection_scenarios() {
    let feature = Parser::parse_feature(
        "tests/bdd/sql_injection_detection.feature"
    ).unwrap();

    // Scenario 1
    run_scenario(&feature, "User input concatenated directly", |ctx| {
        let code = generate_code_with_user_input();
        let findings = analyze_security(code, "go");

        findings.should_contain_an_issue()
            .with_rule_id("GO_SQL_INJECTION")
            .with_severity(Severity::Critical);
    });

    // Scenario 2
    run_scenario(&feature, "User input sanitized", |ctx| {
        let code = generate_code_with_sanitization();
        let findings = analyze_security(code, "go");

        findings.should_be_empty();
    });
}
```

---

## 🚦 Criterios de Aceptación

### Template para Criterios de Aceptación

```gherkin
Feature: [Feature Name]

  Scenario: [Scenario Description]
    Given [Initial State]
    When [Action Performed]
    Then [Expected Outcome]
    And [Additional Outcome]
    But [Unexpected Outcome]
```

### Ejemplo Completo: SQL Injection Rule

```gherkin
Feature: SQL Injection Detection in Go Code

  Scenario: Vulnerable code with string concatenation
    Given Go code with user input from HTTP request
    And SQL query concatenating user input with "+"
    When security analyzer runs
    Then it should detect SQL Injection vulnerability
    And it should report line number of concatenation
    And it should suggest using database/sql Prepare method
    And it should set severity to Critical

  Scenario: Safe code with prepared statements
    Given Go code with user input
    And SQL query using prepared statement with "?" placeholders
    When security analyzer runs
    Then it should NOT detect SQL Injection
    And it should log that prepared statement is used

  Scenario: Vulnerable code with fmt.Sprintf
    Given Go code with user input
    And SQL query using fmt.Sprintf to build query
    When security analyzer runs
    Then it should detect SQL Injection vulnerability
    And it should report usage of fmt.Sprintf as unsafe
    And it should suggest using prepared statements

  Scenario: False positive with constant query
    Given Go code with SQL query containing only constants
    And no user input involved
    When security analyzer runs
    Then it should NOT detect SQL Injection
    And it should recognize constant-only query
```

---

## 🛠️ Herramientas y Utilidades

### Test Fixtures

```rust
// tests/fixtures.rs
pub struct TestFixture {
    pub name: String,
    pub language: String,
    pub source_code: String,
    pub expected_issues: Vec<ExpectedIssue>,
}

impl TestFixture {
    pub fn load(name: &str) -> Self {
        let path = format!("tests/fixtures/{}", name);
        let content = std::fs::read_to_string(&path).unwrap();
        serde_json::from_str(&content).unwrap()
    }
}
```

### Mocking External Dependencies

```rust
// tests/mocks/mock_cve_database.rs
#[mockall::automock]
pub trait CVEDatabase {
    fn lookup_cve(&self, package: &str, version: &str) -> Option<CVE>;
    fn update_database(&self) -> Result<(), UpdateError>;
}

#[kotest::test]
fn should_report_vulnerability_from_database() {
    // Arrange
    let mut mock_db = MockCVEDatabase::new();
    mock_db
        .expect_lookup_cve()
        .with(predicate::eq("lodash"), predicate::eq("4.17.15"))
        .returning(|_, _| Some(CVE {
            id: "CVE-2019-10744".to_string(),
            severity: Severity::Critical,
            // ...
        }));

    let scanner = CVEScanner::new(Box::new(mock_db));

    // Act
    let findings = scanner.scan_package("lodash", "4.17.15");

    // Assert
    findings.should_have_length(1);
    findings[0].cve_id.should_equal("CVE-2019-10744");
}
```

### Database Testing

```rust
// tests/integration/db_tests.rs
#[tokio::test]
async fn should_persist_analysis_results() {
    // Use temporary database
    let temp_db = NamedTempFile::new().unwrap();
    let db_path = temp_db.path().to_str().unwrap();

    let analyzer = ProjectAnalyzer::new_with_db(db_path);
    let project = Project::load("tests/fixtures/sample_project");

    // Act
    let result = analyzer.analyze(&project).await.unwrap();

    // Verify in database
    let stored = analyzer.get_analysis(result.analysis_id).await.unwrap();
    stored.should_be_some();
}
```

---

## 📊 Métricas de Quality

### Coverage Requirements

| Tipo de Test | Coverage Mínimo | Target |
|--------------|-----------------|--------|
| **Unit Tests** | 80% | 90% |
| **Integration Tests** | 70% | 80% |
| **Performance Tests** | 60% | 70% |
| **Security Tests** | 90% | 95% |
| **Overall** | 80% | 85% |

### Test Performance SLAs

| Test Type | Max Duration |
|-----------|-------------|
| **Unit Test** | 100ms |
| **Integration Test** | 5s |
| **Full E2E Test** | 30s |
| **Performance Test** | 120s |

---

## 🔄 Flujo de Trabajo TDD

### Para Cada Story/Task

```
1. 📋 Definir criterios de aceptación en Gherkin
2. 🔴 Escribir test que falla
3. 🟢 Implementar código mínimo
4. 🟢 Verificar que test pasa
5. 🔵 Refactorizar si necesario
6. 🔄 Repetir para próximo test
7. ✅ Verificar todos los tests pasan
8. 📝 Documentar decision (ADR si necesario)
```

### Ejemplo Paso a Paso: SQL Injection Rule

**Paso 1: Escribir Test (RED)**

```rust
#[kotest::test]
fn should_detect_sql_injection_go() {
    let code = r#"
        func getUser(w http.ResponseWriter, r *http.Request) {
            id := r.URL.Query().Get("id")
            query := "SELECT * FROM users WHERE id = " + id
            db.Query(query)
        }
    "#;

    let findings = analyze_security(code, "go");

    findings.should_have_length(1);
    findings[0].rule_id.should_equal("GO_SQL_INJECTION");
}
```

**Paso 2: Ejecutar Test (Falla)**

```bash
$ cargo test should_detect_sql_injection_go
test result: FAILED - test no passes yet
```

**Paso 3: Implementar Código Mínimo (GREEN)**

```rust
// En rule_engine.rs
pub struct SQLInjectionRule;

impl StaticAnalysisRule for SQLInjectionRule {
    fn check(&self, context: &AnalysisContext) -> Vec<Finding> {
        // Implementación mínima para hacer pasar el test
        if context.language == "go" {
            vec![Finding {
                rule_id: "GO_SQL_INJECTION".to_string(),
                severity: Severity::Critical,
                // ...
            }]
        } else {
            vec![]
        }
    }
}
```

**Paso 4: Verificar Test Pasa**

```bash
$ cargo test should_detect_sql_injection_go
test result: OK - 1 test passed
```

**Paso 5: Agregar Más Tests (RED)**

```rust
#[kotest::test]
fn should_not_detect_sql_injection_with_constants() {
    let code = r#"
        func getUser() {
            query := "SELECT * FROM users WHERE id = '123'"
            db.Query(query)
        }
    "#;

    let findings = analyze_security(code, "go");

    findings.should_be_empty();
}
```

**Paso 6: Mejorar Implementación (GREEN)**

```rust
// Implementación más completa con taint analysis
impl SQLInjectionRule {
    fn check_impl(&self, context: &AnalysisContext) -> Vec<Finding> {
        // Implementación real con detection logic
        let sources = context.dfg.find_taint_sources();
        let sinks = context.dfg.find_sql_sinks();

        // Taint analysis logic here
        vec![]
    }
}
```

**Paso 7: Refactor (BLUE)**

```rust
// Extraer helper functions
impl SQLInjectionRule {
    fn has_user_input(&self, code: &str) -> bool {
        // Check for HTTP request parameters
    }

    fn has_string_concatenation(&self, code: &str) -> bool {
        // Check for + operator in SQL context
    }
}
```

---

## ✅ Checklist Pre-Commit

Antes de hacer commit, asegurar:

- [ ] ✅ Todos los tests en verde (`cargo test`)
- [ ] ✅ Coverage meets minimums (`cargo tarpaulin`)
- [ ] ✅ Performance tests passing
- [ ] ✅ No failing CI pipeline
- [ ] ✅ Documentación actualizada
- [ ] ✅ ADRs creados para architectural decisions
- [ ] ✅ Tests para edge cases
- [ ] ✅ Tests para error handling

---

## 📚 Recursos Adicionales

### Documentos Relacionados

- [Épica 1: Core Engine](./epics/EPIC-01-CORE_STATIC_ANALYSIS_ENGINE.md)
- [Épica 2: Security Analysis](./epics/EPIC-02-SECURITY_ANALYSIS_SAST.md)
- [Arquitectura Técnica](./ARCHITECTURE.md)

### Enlaces Útiles

- [Kotest Documentation](https://kotest.io/)
- [Rust Testing Best Practices](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [BDD in Rust](https://cucumber.io/docs/bdd/)
- [Property-Based Testing](https://altsysrq.github.io/proptest-book/)

---

## 🎯 Criterios de Done (TDD)

Para considerar completada una tarea con TDD:

- [ ] ✅ Test escrito ANTES de implementación
- [ ] ✅ Test falla inicialmente (RED)
- [ ] ✅ Implementación mínima escrita
- [ ] ✅ Test pasa (GREEN)
- [ ] ✅ Código refactorizado manteniendo tests verdes
- [ ] ✅ Coverage requirements met
- [ ] ✅ Performance tests passing
- [ ] ✅ Edge cases covered
- [ ] ✅ Documentation updated

---

**Remember: TDD is not about writing tests, it's about designing through tests!**

---

*Última actualización: 10 de noviembre de 2025*
