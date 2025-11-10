# Épica 2: Security Analysis (SAST)
## Motor de Análisis de Seguridad con Motor de Reglas Determinista

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 🚧 Planning
**Época:** Fase 1 (Meses 1-6)
**Prioridad:** 🔴 Crítica

---

## 📋 Resumen Ejecutivo

Desarrollar el motor de análisis de seguridad estático (SAST) para hodei-scan, basado en el motor de reglas determinista inspirado en Cedar. Este motor proporcionará detección avanzada de vulnerabilidades con 67% menos falsos positivos que SonarQube y tiempo de ejecución O(n) garantizado.

### Objetivos Principales
- ✅ Motor de reglas Cedar-inspired determinista
- ✅ Detección OWASP Top 10 con análisis semántico profundo
- ✅ Reglas CWE/SANS Top 25 framework-específicas
- ✅ Taint analysis real con dataflow tracking
- ✅ WASM sandbox para reglas enterprise complejas
- ✅ <2ms evaluación de reglas por archivo

### Métricas de Éxito
- **Performance**: <2ms evaluación de reglas (vs 10-20ms SonarQube)
- **Accuracy**: >90% accuracy en vulnerability detection
- **False Positives**: <10% (vs 30-40% SonarQube)
- **Coverage**: OWASP Top 10 + CWE Top 25 completo
- **Languages**: 3 lenguajes (Rust, Go, TypeScript)
- **Determinism**: Tiempo acotado O(n) garantizado

---

## 👥 Historias de Usuario

### US-05: Como security engineer, quiero detectar vulnerabilidades SQL Injection con precisión

**Prioridad:** 🔴 Critical
**Story Points:** 8
**Criterios de Aceptación:**

```gherkin
Feature: Detección de SQL Injection
  Como security engineer
  Quiero que hodei-scan detecte vulnerabilidades SQL Injection
  Para prevenir ataques de base de datos

  Scenario: SQL Injection con user input directo
    Given código Go con query construido con input de usuario
    When hodei-scan analiza el código
    Then debería detectar vulnerabilidad SQL Injection
    And debería reportar línea exacta
    And debería sugerir fix con prepared statements

  Scenario: SQL Injection con sanitización
    Given código que sanitiza input antes de query
    When hodei-scan analiza el código
    Then NO debería reportar falso positivo
    And debería verificar que sanitización es completa

  Scenario: SQL Injection con taint analysis
    Given código con data flow desde user input hasta SQL query
    When hodei-scan hace taint tracking
    Then debería seguir el data flow completo
    And debería detectar si hay sanitización en el path

  Scenario: False positive: query con constantes
    Given código con query construido solo con constantes
    When hodei-scan analiza el código
    Then NO debería reportar vulnerabilidad
    And debería reconocer que no hay user input
```

**Tareas de Desarrollo:**

1. **TASK-02-01: Implementar RuleEngine trait**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: Épica 1 - TASK-01-02
   - Deliverable: Trait RuleEngine con evaluación determinista

   ```rust
   // Implementación mínima requerida:
   #[test]
   fn test_rule_engine_evaluation() {
       let engine = RuleEngine::new();
       let sql_rule = SQLInjectionRule::new();
       engine.register_rule(sql_rule);

       let code = r#"
           func queryUser(id string) {
               db.Query("SELECT * FROM users WHERE id = " + id)
           }
       "#;

       let findings = engine.evaluate(code, "go");
       assert_eq!(findings.len(), 1);
       assert_eq!(findings[0].rule_id, "GO_SQL_INJECTION");
       assert_eq!(findings[0].severity, Severity::Critical);
   }
   ```

2. **TASK-02-02: Implementar SQL Injection Rule**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-02-01
   - Deliverable: SQLInjectionRule con taint analysis

3. **TASK-02-03: Implementar Taint Analysis Engine**
   - Criterio: Tests en verde
   - Estimación: 4 días
   - Dependencias: TASK-02-02
   - Deliverable: TaintTracker con dataflow

4. **TASK-02-04: Agregar sanitización patterns**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-02-03
   - Deliverable: SanitizationPatternMatcher

**Tests de Validación:**

```rust
// TEST-02-01: SQL Injection detection
#[test]
fn test_sql_injection_go() {
    let code = r#"
        func getUser(w http.ResponseWriter, r *http.Request) {
            id := r.URL.Query().Get("id")
            query := "SELECT * FROM users WHERE id = " + id
            db.Query(query)
        }
    "#;

    let findings = analyze_security(code, "go");
    assert!(findings.iter().any(|f| f.rule_id == "GO_SQL_INJECTION"));
}

// TEST-02-02: False positive prevention
#[test]
fn test_no_false_positive_sql_constants() {
    let code = r#"
        func getUser() {
            query := "SELECT * FROM users WHERE id = '123'"
            db.Query(query)
        }
    "#;

    let findings = analyze_security(code, "go");
    assert!(!findings.iter().any(|f| f.rule_id == "GO_SQL_INJECTION"));
}

// TEST-02-03: Taint analysis
#[test]
fn test_taint_tracking() {
    let code = r#"
        func process(input string) string {
            temp := input
            temp = strings.TrimSpace(temp)
            return temp
        }
    "#;

    let taint = TaintTracker::trace(code, "go");
    assert!(taint.has_taint("input"));
    assert!(taint.is_sanitized("temp")); // TrimSpace es sanitización
}

// TEST-02-04: Sanitization detection
#[test]
fn test_sanitization_patterns() {
    let sanitizer = SanitizationPatternMatcher::new();
    assert!(sanitizer.is_sanitization("strings.TrimSpace"));
    assert!(sanitizer.is_sanitization("regexp.QuoteMeta"));
    assert!(!sanitizer.is_sanitization("strings.ToUpper"));
}
```

---

### US-06: Como developer, quiero que hodei-scan detecte usos inseguros de unsafe en Rust

**Prioridad:** 🔴 Critical
**Story Points:** 5
**Criterios de Aceptación:**

```gherkin
Feature: Detección de unsafe en Rust
  Como developer escribiendo código Rust
  Quiero que hodei-scan identifique usos inseguros de unsafe
  Para prevenir undefined behavior

  Scenario: Unsafe sin comentarios de seguridad
    Given función unsafe sin documentación de safety
    When hodei-scan analiza el código
    Then debería reportar warning
    And debería sugerir agregar safety comments

  Scenario: Unsafe sin проверка de invariantes
    Given bloque unsafe sin checks previos
    When hodei-scan analiza el código
    Then debería reportar error crítico
    And debería sugerir validar invariantes

  Scenario: Safe wrapper around unsafe
    Given función safe que envuelve unsafe
    When hodei-scan analiza el código
    Then debería validar que unsafe está contenido
    And NO debería reportar como inseguro
```

**Tareas de Desarrollo:**

1. **TASK-02-05: Implementar Rust Unsafe Rule**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-02-01
   - Deliverable: RustUnsafeRule con safety checking

**Tests de Validación:**

```rust
// TEST-02-05: Unsafe without safety comment
#[test]
fn test_unsafe_no_safety_comment() {
    let code = r#"
        unsafe fn deref_ptr(ptr: *const i32) -> i32 {
            *ptr
        }
    "#;

    let findings = analyze_security(code, "rust");
    assert!(findings.iter().any(|f| {
        f.rule_id == "RUST_UNSAFE_NO_COMMENT" && f.severity == Severity::Warning
    }));
}

// TEST-02-06: Unsafe with safety comment
#[test]
fn test_unsafe_with_safety_comment() {
    let code = r#"
        /// Safety: ptr must be non-null and aligned
        unsafe fn deref_ptr(ptr: *const i32) -> i32 {
            *ptr
        }
    "#;

    let findings = analyze_security(code, "rust");
    assert!(!findings.iter().any(|f| f.rule_id == "RUST_UNSAFE_NO_COMMENT"));
}
```

---

### US-07: Como security team, queremos cobertura completa de OWASP Top 10

**Prioridad:** 🔴 Critical
**Story Points:** 13
**Criterios de Aceptación:**

```gherkin
Feature: OWASP Top 10 Coverage
  Como security team
  Quiero detección completa de OWASP Top 10
  Para cumplir estándares de seguridad industry

  Scenario: A01 - Broken Access Control
    Given endpoints sin autorización
    When hodei-scan analiza el código
    Then debería detectar missing authorization
    And debería verificar decorators/annotations de seguridad

  Scenario: A02 - Cryptographic Failures
    Given uso de algoritmos criptográficos débiles
    When hodei-scan analiza el código
    Then debería detectar uso de MD5/SHA1
    And debería sugerir algoritmos fuertes (SHA-256, Argon2)

  Scenario: A03 - Injection
    Given user input concatenado en comandos/queries
    When hodei-scan analiza el código
    Then debería detectar SQL, NoSQL, OS command injection
    And debería verificar parameterized queries

  Scenario: A04 - Insecure Design
    Given falta de validación de input
    When hodei-scan analiza el código
    Then debería detectar missing input validation
    And debería sugerir validación robusta

  Scenario: A05 - Security Misconfiguration
    Given configuración de seguridad débil
    When hodei-scan analiza el código
    Then debería detectar headers inseguros
    And debería sugerir configuración segura
```

**Tareas de Desarrollo:**

1. **TASK-02-06: Implementar OWASP Top 10 Rules**
   - Criterio: Tests en verde (todas las reglas)
   - Estimación: 10 días
   - Dependencias: TASK-02-04
   - Deliverable: 10 reglas OWASP Top 10

2. **TASK-02-07: Implementar CWE Top 25 Rules**
   - Criterio: Tests en verde
   - Estimación: 8 días
   - Dependencias: TASK-02-06
   - Deliverable: 25 reglas CWE

**Tests de Validación:**

```rust
// TEST-02-07: OWASP Top 10 complete coverage
#[test]
fn test_owasp_top10_coverage() {
    let rules = OWASPTop10Rules::all();
    assert_eq!(rules.len(), 10);

    // A01 - Broken Access Control
    assert!(rules.iter().any(|r| r.id == "OWASP_A01_BROKEN_ACCESS_CONTROL"));

    // A02 - Cryptographic Failures
    assert!(rules.iter().any(|r| r.id == "OWASP_A02_CRYPTO_FAILURES"));

    // A03 - Injection
    assert!(rules.iter().any(|r| r.id == "OWASP_A03_INJECTION"));

    // ... etc para todas las 10
}

// TEST-02-08: Cryptographic algorithm detection
#[test]
fn test_weak_crypto_detection() {
    let code = r#"
        import hashlib
        def hash_password(password):
            return hashlib.md5(password.encode()).hexdigest()
    "#;

    let findings = analyze_security(code, "python");
    assert!(findings.iter().any(|f| {
        f.rule_id == "PY_WEAK_HASH_MD5" && f.severity == Severity::Critical
    }));
}
```

---

### US-08: Como developer, quiero reglas específicas para mi framework (React, Spring, Django)

**Prioridad:** 🟡 Medium
**Story Points:** 8
**Criterios de Aceptación:**

```gherkin
Feature: Framework-Specific Rules
  Como developer usando framework específico
  Quiero reglas adaptadas a ese framework
  Para detectar vulnerabilidades framework-specific

  Scenario: React XSS detection
    Given uso de dangerouslySetInnerHTML
    When hodei-scan analiza código React
    Then debería detectar potential XSS
    And debería verificar sanitización

  Scenario: Django SQL injection
    Given uso de raw() o extra() en Django ORM
    When hodei-scan analiza código Django
    Then debería verificar que input está sanitizado
    And debería sugerir usar ORM safe methods

  Scenario: Spring Security misconfiguration
    Given aplicación Spring sin CSRF protection
    When hodei-scan analiza código Spring
    Then debería detectar missing CSRF
    And debería sugerir configuración segura
```

**Tareas de Desarrollo:**

1. **TASK-02-08: Implementar framework detection**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-02-01
   - Deliverable: FrameworkDetector

2. **TASK-02-09: Implementar React rules**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-02-08
   - Deliverable: ReactSecurityRules

3. **TASK-02-10: Implementar Django rules**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-02-08
   - Deliverable: DjangoSecurityRules

4. **TASK-02-11: Implementar Spring rules**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-02-08
   - Deliverable: SpringSecurityRules

**Tests de Validación:**

```rust
// TEST-02-09: React XSS detection
#[test]
fn test_react_xss_dangerously_set_inner_html() {
    let code = r#"
        function UserInput({ content }) {
            return <div dangerouslySetInnerHTML={{__html: content}} />;
        }
    "#;

    let findings = analyze_security(code, "typescript");
    assert!(findings.iter().any(|f| {
        f.rule_id == "REACT_XSS_DANGEROUSLY_SET_INNER_HTML"
    }));
}

// TEST-02-10: Django safe ORM usage
#[test]
fn test_django_safe_orm() {
    let code = r#"
        # Safe: Using ORM
        users = User.objects.filter(id=user_id)

        # Unsafe: Using raw()
        users = User.objects.raw('SELECT * FROM users WHERE id = %s', [user_id])
    "#;

    let findings = analyze_security(code, "python");
    assert!(findings.iter().any(|f| f.rule_id == "DJANGO_UNSAFE_RAW_QUERY"));
}
```

---

### US-09: Como enterprise user, quiero reglas custom en WASM sandbox

**Prioridad:** 🟡 Medium
**Story Points:** 13
**Criterios de Aceptación:**

```gherkin
Feature: WASM Custom Rules
  Como enterprise user con reglas específicas
  Quiero implementar reglas custom en WASM
  Para adaptar hodei-scan a mis necesidades específicas

  Scenario: Cargar regla custom desde WASM module
    Given archivo .wasm con regla personalizada
    When hodei-scan ejecuta análisis
    Then debería cargar y ejecutar regla WASM
    And debería retornar findings correctos
    And debería mantener sandbox isolation

  Scenario: Regla WASM con acceso limitado
    Given regla WASM que intenta acceso a file system
    When hodei-scan ejecuta regla
    Then debería ser bloqueado por sandbox
    And debería log de security violation
    And NO debería crash hodei-scan

  Scenario: Performance de reglas WASM
    Given 100 reglas WASM ejecutándose
    When hodei-scan analiza proyecto
    Then debería completar en <30 segundos
    And debería usar <500MB memoria extra
```

**Tareas de Desarrollo:**

1. **TASK-02-12: Implementar WASM sandbox**
   - Criterio: Tests en verde
   - Estimación: 5 días
   - Dependencias: TASK-02-01
   - Deliverable: WASMRuntime con sandbox

2. **TASK-02-13: Implementar WASM rule interface**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-02-12
   - Deliverable: WASMRule trait

3. **TASK-02-14: Implementar performance monitoring**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-02-13
   - Deliverable: PerformanceTracker

**Tests de Validación:**

```rust
// TEST-02-11: WASM rule loading
#[test]
fn test_wasm_rule_loading() {
    let wasm_bytes = load_wasm_rule("custom_rule.wasm");
    let runtime = WASMRuntime::new();

    let rule = runtime.load_rule(wasm_bytes).unwrap();
    assert!(rule.execute().is_ok());
}

// TEST-02-12: WASM sandbox isolation
#[test]
#[should_panic]
fn test_wasm_sandbox_isolation() {
    let wasm_bytes = malicious_wasm_bytes();
    let runtime = WASMRuntime::new();

    // Should panic when trying to access restricted resources
    runtime.load_rule(wasm_bytes).unwrap();
}
```

---

## 🏗️ Arquitectura Técnica

### Motor de Reglas Cedar-Inspired

```
┌─────────────────────────────────────────┐
│         Cedar-Inspired Rule Engine      │
├─────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────────┐  │
│  │  Rule Index │  │  Rule Evaluator │  │
│  │  (Fast Slicing)│  │  (Parallel)   │  │
│  └─────────────┘  └─────────────────┘  │
│         │                 │             │
│  ┌──────▼──────┐  ┌──────▼──────┐     │
│  │  By Lang    │  │  Rayon      │     │
│  │  By Severity│  │  Pool       │     │
│  │  By Category│  │  (<2ms)     │     │
│  └─────────────┘  └─────────────┘     │
│         │                 │             │
│  ┌──────▼─────────────────▼──────┐      │
│  │  Rule Verifier                │      │
│  │  • Type checking              │      │
│  │  • Cyclomatic complexity      │      │
│  │  • Dead code detection        │      │
│  └───────────────────────────────┘      │
└─────────────────────────────────────────┘
                        │
┌─────────────────────────────────────────┐
│         Security Analysis Pipeline      │
├─────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────────┐  │
│  │  Taint      │  │  Framework      │  │
│  │  Analysis   │  │  Detection      │  │
│  └─────────────┘  └─────────────────┘  │
│         │                 │             │
│  ┌──────▼───────────────▼──────┐      │
│  │  OWASP Top 10 Rules         │      │
│  │  CWE Top 25 Rules           │      │
│  │  Framework-Specific Rules   │      │
│  │  Custom WASM Rules          │      │
│  └─────────────────────────────┘      │
└─────────────────────────────────────────┘
```

### Dependencias Security

```toml
[dependencies]
# Rule Engine
serde = "1.0"
nom = "7.1"
regex = "1.0"

# WASM Runtime
wasmtime = "20.0"
wasmer = "4.0"

# Taint Analysis
petgraph = "0.6"
```

---

## 📊 Estimación y Plan de Entrega

### Cronograma Épica 2 (5 meses, paralelo con Épica 1)

| Mes | Tareas | Story Points | Entregables |
|-----|--------|--------------|-------------|
| 1-2 | TASK-02-01 a 02-04: Core Engine | 34 | RuleEngine + Taint |
| 2-3 | TASK-02-05: Rust Unsafe | 5 | Rust security rules |
| 3-4 | TASK-02-06: OWASP Top 10 | 34 | OWASP completo |
| 4-5 | TASK-02-07: CWE Top 25 | 21 | CWE completo |
| 5-6 | TASK-02-08 a 02-11: Framework Rules | 34 | React/Django/Spring |
| 6 | TASK-02-12 a 02-14: WASM | 21 | WASM sandbox |

**Total Story Points:** 149
**Parallelization:** 40% paralelo con Épica 1
**Duración Real:** 6 meses

---

## 🧪 Estrategia de Testing

### Security Testing Pyramid

1. **Unit Tests (60%)**
   - Rule evaluation tests
   - Taint analysis tests
   - Framework detection tests
   - WASM sandbox tests

2. **Integration Tests (30%)**
   - OWASP Top 10 validation
   - False positive testing
   - Performance benchmarks
   - End-to-end security scan

3. **Red Team Tests (10%)**
   - Known vulnerability detection
   - Bypass techniques
   - Evasion attempts

### Herramientas de Testing

```toml
[dev-dependencies]
# Security testing
synth = "0.3"  # Synthetic data generation
proptest = "1.4"  # Property-based testing
```

---

## 📚 Reglas de Seguridad Implementadas

### OWASP Top 10 (2021)

1. **A01 - Broken Access Control**
   - Missing authorization checks
   - Privilege escalation
   - Direct object references

2. **A02 - Cryptographic Failures**
   - Weak algorithms (MD5, SHA1)
   - Insecure key generation
   - Missing encryption

3. **A03 - Injection**
   - SQL Injection
   - NoSQL Injection
   - OS Command Injection
   - LDAP Injection

4. **A04 - Insecure Design**
   - Missing input validation
   - Insecure defaults
   - Missing security controls

5. **A05 - Security Misconfiguration**
   - Insecure headers
   - Default credentials
   - Missing security updates

6. **A06 - Vulnerable Components**
   - Outdated dependencies
   - Known CVEs
   - Unpatched libraries

7. **A07 - ID and Auth Failures**
   - Weak password policies
   - Session management issues
   - Missing MFA

8. **A08 - Software Integrity Failures**
   - Unsigned updates
   - Insecure CI/CD
   - Missing integrity checks

9. **A09 - Logging Failures**
   - Missing audit logs
   - Sensitive data in logs
   - Inadequate log retention

10. **A10 - SSRF**
    - Missing SSRF protection
    - Unrestricted URL protocols
    - Missing input validation

### CWE Top 25 (2024)

[Lista completa de 25 vulnerabilidades más peligrosas]

### Framework-Specific

- **React**: XSS, CSRF, insecure state management
- **Django**: SQL injection, XSS, CSRF, clickjacking
- **Spring**: Security misconfig, XXE, deserialization
- **Express.js**: XSS, CSRF, header injection
- **Flask**: SQL injection, XSS, session security

---

## 🔄 Criterios de Done

Para que esta épica se considere **COMPLETADA**:

- [ ] ✅ Motor de reglas determinista funcionando
- [ ] ✅ <2ms evaluación de reglas por archivo
- [ ] ✅ 100% OWASP Top 10 coverage
- [ ] ✅ 100% CWE Top 25 coverage
- [ ] ✅ <10% false positive rate
- [ ] ✅ >90% accuracy en vulnerability detection
- [ ] ✅ Taint analysis completo
- [ ] ✅ Framework-specific rules (React, Django, Spring)
- [ ] ✅ WASM sandbox para reglas custom
- [ ] ✅ 100% tests en verde
- [ ] ✅ Performance benchmarks validados
- [ ] ✅ Security audit passed

---

## 🚀 Siguiente Épica

Una vez completada esta épica, proceder con:
**[Épica 3: Software Composition Analysis (SCA)](./EPIC-03-SOFTWARE_COMPOSITION_ANALYSIS.md)**

---

## 📞 Contacto y Soporte

**Security Lead:** [A definir]
**Epic Owner:** [A definir]
**Slack Channel:** #hodei-scan-security
**Security Audit:** security@hodei-scan.dev

---

*Última actualización: 10 de noviembre de 2025*
