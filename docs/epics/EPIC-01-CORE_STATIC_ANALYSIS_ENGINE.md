# Épica 1: Core Static Analysis Engine
## Motor de Análisis de Código Nativo con Arquitectura Determinista

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 🚧 Planning
**Época:** Fase 1 (Meses 1-6)
**Prioridad:** 🔴 Crítica

---

## 📋 Resumen Ejecutivo

Desarrollar el motor core de análisis estático de código para hodei-scan, un analizador nativo en Rust que proporcione análisis semántico profundo con performance superior (2x-5x más rápido que SonarQube). Esta épica establece la base arquitectónica para todo el sistema.

### Objetivos Principales
- ✅ Motor de análisis semántico determinista O(n)
- ✅ Soporte para 3 lenguajes: Rust, Go, TypeScript/JavaScript (Fase 1)
- ✅ Framework extensible para agregar lenguajes
- ✅ Performance superior: <2min para análisis de 100K LOC
- ✅ Análisis semántico profundo: DFA, CFG, taint tracking
- ✅ Arquitectura sin contradicciones (sin LSPs, batch-optimized)

### Métricas de Éxito
- **Performance**: 2x-5x más rápido que SonarQube en análisis end-to-end
- **Accuracy**: >90% accuracy en detección de issues
- **Coverage**: 3 lenguajes en Fase 1, extensible a 6 en Fase 2
- **Memory**: 5x menos uso de RAM (800MB vs 4GB para 1M LOC)
- **Reliability**: 99.9% success rate en análisis

---

## 👥 Historias de Usuario

### US-01: Como desarrollador, quiero analizar código Rust con hodei-scan para identificar problemas de calidad y seguridad

**Prioridad:** 🔴 Critical
**Story Points:** 8
**Criterios de Aceptación:**

```gherkin
Feature: Análisis de código Rust
  Como desarrollador escribiendo código Rust
  Quiero que hodei-scan analice mi código
  Para identificar problemas de calidad, complejidad y posibles bugs

  Scenario: Análisis exitoso de archivo Rust simple
    Given un archivo "main.rs" con código Rust válido
    When ejecuto hodei-scan analyze --language rust main.rs
    Then debería completarse en <10 segundos
    And debería generar un reporte con issues encontrados
    And debería mostrar métricas de complejidad ciclomática
    And debería identificar code smells

  Scenario: Análisis de proyecto Rust completo
    Given un proyecto Rust con múltiples archivos .rs
    When ejecuto hodei-scan analyze --language rust ./src
    Then debería analizar todos los archivos en el directorio
    And debería construir un CFG (Control Flow Graph) para cada función
    And debería realizar análisis de dataflow
    And debería generar reporte consolidado con issues por archivo

  Scenario: Manejo de errores de parsing
    Given un archivo "broken.rs" con sintaxis inválida
    When ejecuto hodei-scan analyze --language rust broken.rs
    Then debería reportar error de parsing con línea y columna
    And debería continuar analizando otros archivos válidos
    And debería retornar exit code != 0
```

**Tareas de Desarrollo:**

1. **TASK-01-01: Implementar parser base para Rust**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: Ninguna
   - Deliverable: `rust_parser` crate funcionando

   ```rust
   // Implementación mínima requerida:
   #[test]
   fn test_parse_simple_rust_function() {
       let code = r#"
           fn main() {
               println!("Hello");
           }
       "#;
       let result = parse_rust(code);
       assert!(result.is_ok());
       let ast = result.unwrap();
       assert_eq!(ast.functions.len(), 1);
   }
   ```

2. **TASK-01-02: Implementar análisis semántico básico**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-01-01
   - Deliverable: Analyzer trait con implementación Rust

3. **TASK-01-03: Construir Control Flow Graph (CFG)**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-01-02
   - Deliverable: CFG builder para funciones Rust

4. **TASK-01-04: Implementar Data Flow Analysis**
   - Criterio: Tests en verde
   - Estimación: 4 días
   - Dependencias: TASK-01-03
   - Deliverable: DFA engine para tracking de variables

5. **TASK-01-05: Agregar soporte para Go**
   - Criterio: Tests en verde
   - Estimación: 5 días
   - Dependencias: TASK-01-04
   - Deliverable: Go analyzer completo

6. **TASK-01-06: Agregar soporte para TypeScript/JavaScript**
   - Criterio: Tests en verde
   - Estimación: 5 días
   - Dependencias: TASK-01-05
   - Deliverable: TypeScript/JavaScript analyzer

**Tests de Validación:**

```rust
// TEST-01-01: Benchmark de performance vs SonarQube
#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_100k_loc_analysis_time() {
        let start = Instant::now();
        let result = analyze_large_rust_project();
        let duration = start.elapsed();

        // Debe completar en <2 minutos (2x más rápido que SonarQube)
        assert!(duration < Duration::from_secs(120));
        assert!(result.is_ok());
    }

    #[test]
    fn test_memory_efficiency() {
        let before = get_memory_usage();
        let _result = analyze_large_rust_project();
        let after = get_memory_usage();
        let used = after - before;

        // Debe usar <800MB para 1M LOC
        assert!(used < 800 * 1024 * 1024);
    }
}

// TEST-01-02: Análisis semántico completo
#[test]
fn test_semantic_analysis_rust() {
    let code = r#"
        fn calculate(x: i32, y: i32) -> i32 {
            if x > 0 {
                x + y
            } else {
                y - x
            }
        }
    "#;

    let ast = parse_rust(code).unwrap();
    let cfg = build_cfg(&ast).unwrap();
    let dfa = dataflow_analysis(&cfg).unwrap();

    // Verificar que CFG se construyó correctamente
    assert_eq!(cfg.nodes.len(), 4); // entry, if, then, else, merge

    // Verificar que DFA tracking de variables funciona
    assert!(dfa.track_variable("x").is_some());
    assert!(dfa.track_variable("y").is_some());
}

// TEST-01-03: Manejo de errores
#[test]
fn test_invalid_rust_syntax() {
    let invalid_code = r#"
        fn broken( {
            let x = ;
        }
    "#;

    let result = parse_rust(invalid_code);
    assert!(result.is_err());

    if let Err(ParseError { line, column, .. }) = result {
        assert_eq!(line, 2);
        assert!(column > 0);
    }
}
```

---

### US-02: Como DevOps engineer, quiero integración con CI/CD para análisis automático

**Prioridad:** 🔴 Critical
**Story Points:** 5
**Criterios de Aceptación:**

```gherkin
Feature: Integración CI/CD
  Como DevOps engineer configurando pipelines
  Quiero ejecutar hodei-scan en CI/CD
  Para analizar código automáticamente en cada commit/PR

  Scenario: GitHub Actions integration
    Given un repositorio con workflow配置
    When se hace push a la rama main
    Then hodei-scan debería ejecutarse automáticamente
    And debería fallar el pipeline si quality gate no pasa
    And debería generar reporte en PR comments

  Scenario: GitLab CI integration
    Given un proyecto con .gitlab-ci.yml
    When se ejecuta el pipeline
    Then hodei-scan debería analizar el código
    And debería reportar status en merge request
    And debería mostrar coverage changes
```

**Tareas de Desarrollo:**

1. **TASK-01-07: CLI interface para CI/CD**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-01-06
   - Deliverable: CLI commands con output format específico

2. **TASK-01-08: GitHub Actions integration**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-01-07
   - Deliverable: GitHub Action marketplace action

**Tests de Validación:**

```rust
// TEST-01-04: CLI interface
#[test]
fn test_cli_ci_mode() {
    let result = Command::new("hodei-scan")
        .args(&["analyze", "--ci", "--format", "github-checks", "./tests/fixtures/rust"])
        .output()
        .unwrap();

    assert!(result.status.success());
    let output = String::from_utf8(result.stdout).unwrap();
    assert!(output.contains("::notice"));
    assert!(output.contains("::error"));
}

// TEST-01-05: Output format para CI
#[test]
fn test_github_checks_format() {
    let issues = vec![
        Issue {
            file: "src/main.rs",
            line: 10,
            severity: Severity::Critical,
            rule_id: "RUST_UNSAFE_NO_COMMENT",
            message: "Unsafe block without safety comment",
        }
    ];

    let output = format_github_checks(&issues);
    assert!(output.contains("::notice"));
    assert!(output.contains("file=src/main.rs"));
    assert!(output.contains("line=10"));
}
```

---

### US-03: Como arquitecto de software, quiero extender el motor con nuevos lenguajes

**Prioridad:** 🟡 Medium
**Story Points:** 13
**Criterios de Aceptación:**

```gherkin
Feature: Extensibilidad de lenguajes
  Como arquitecto de software
  Quiero agregar soporte para nuevos lenguajes
  Para expandir la cobertura de hodei-scan

  Scenario: Implementar analyzer para nuevo lenguaje
    Given un nuevo lenguaje "MyLang"
    When implemento el trait LanguageAnalyzer para MyLang
    Then debería registrarse automáticamente en el motor
    And debería poder ejecutar analyze --language mylang
    And debería generar resultados consistentes con otros lenguajes

  Scenario: Lazy loading de analyzers
    Given múltiples analyzers registrados
    When ejecuto análisis para lenguaje específico
    Then solo debería cargar el analyzer necesario
    And no debería cargar analyzers no utilizados
```

**Tareas de Desarrollo:**

1. **TASK-01-09: Diseñar trait LanguageAnalyzer**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-01-06
   - Deliverable: Trait público estable con documentación

2. **TASK-01-10: Plugin system para analyzers**
   - Criterio: Tests en verde
   - Estimación: 4 días
   - Dependencias: TASK-01-09
   - Deliverable: Registry de analyzers con lazy loading

3. **TASK-01-11: Documentar API para custom analyzers**
   - Criterio: Tests en verde
   - Estimación: 2 días
   - Dependencias: TASK-01-10
   - Deliverable: Documentación completa + ejemplos

**Tests de Validación:**

```rust
// TEST-01-06: LanguageAnalyzer trait
#[test]
fn test_language_analyzer_trait() {
    trait MyLangAnalyzer: LanguageAnalyzer {
        type AST = MyLangAST;
        type CFG = MyLangCFG;
        type DataFlowGraph = MyLangDFG;

        fn parse(&self, source: &str) -> Result<Self::AST, ParseError> {
            // implementación
        }
        // ... otros métodos
    }

    // Verificar que el trait funciona como contrato
    assert!(std::mem::size_of::<dyn LanguageAnalyzer>() > 0);
}

// TEST-01-07: Registry de analyzers
#[test]
fn test_analyzer_registry() {
    let registry = AnalyzerRegistry::new();
    registry.register(Box::new(RustAnalyzer::new()));
    registry.register(Box::new(GoAnalyzer::new()));

    assert!(registry.has_analyzer("rust"));
    assert!(registry.has_analyzer("go"));
    assert!(!registry.has_analyzer("python")); // no registrado

    let analyzer = registry.get("rust").unwrap();
    assert!(analyzer.language() == "rust");
}
```

---

### US-04: Como usuario, quiero análisis incremental rápido durante desarrollo

**Prioridad:** 🟡 Medium
**Story Points:** 8
**Criterios de Aceptación:**

```gherkin
Feature: Análisis incremental
  Como desarrollador en IDE
  Quiero análisis de código en tiempo real
  Para recibir feedback inmediato sin esperar análisis completo

  Scenario: Análisis de archivo modificado
    Given un proyecto ya analizado previamente
    When modifico un solo archivo
    Then hodei-scan debería re-analizar solo ese archivo
    And debería completarse en <1 segundo
    And debería mantener resultados de otros archivos

  Scenario: Incremental con dependencias
    Given un archivo que importa de otros módulos
    When modifico el archivo base
    Then hodei-scan debería re-analizar archivos dependientes
    And debería propagar cambios correctamente
```

**Tareas de Desarrollo:**

1. **TASK-01-12: Implementar cache de análisis**
   - Criterio: Tests en verde
   - Estimación: 3 días
   - Dependencias: TASK-01-06
   - Deliverable: Cache con invalidación inteligente

2. **TASK-01-13: Análisis incremental con dependency tracking**
   - Criterio: Tests en verde
   - Estimación: 4 días
   - Dependencias: TASK-01-12
   - Deliverable: Sistema de tracking de dependencias

**Tests de Validación:**

```rust
// TEST-01-08: Cache de análisis
#[test]
fn test_analysis_cache() {
    let cache = AnalysisCache::new();
    let code = "fn test() {}";

    // Primer análisis
    let result1 = cache.get_or_compute("file.rs", code, || {
        analyze_file("file.rs", code)
    }).unwrap();

    // Segundo análisis (debería usar cache)
    let result2 = cache.get_or_compute("file.rs", code, || {
        panic!("No debería ejecutarse");
    }).unwrap();

    assert_eq!(result1, result2);
}

// TEST-01-09: Análisis incremental
#[test]
fn test_incremental_analysis() {
    let project = ProjectAnalysis::new("./tests/fixtures/rust");
    project.full_analysis().unwrap();

    // Modificar un archivo
    modify_file("src/main.rs", "new code");

    let start = Instant::now();
    let result = project.incremental_analysis();
    let duration = start.elapsed();

    // Debe completar en <1 segundo
    assert!(duration < Duration::from_secs(1));
    assert!(result.is_ok());
}
```

---

## 🏗️ Arquitectura Técnica

### Componentes Principales

```
┌─────────────────────────────────────────┐
│         Core Analysis Engine            │
├─────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────────┐  │
│  │  Parsing    │  │  Semantic       │  │
│  │  Layer      │  │  Analysis       │  │
│  └─────────────┘  └─────────────────┘  │
│         │                 │             │
│  ┌──────▼──────┐  ┌──────▼──────┐     │
│  │  Tree-      │  │  Language   │     │
│  │  sitter     │  │  Specific   │     │
│  │  (Universal)│  │  Analyzers  │     │
│  └─────────────┘  └─────────────┘     │
│         │                 │             │
│  ┌──────▼───────────────▼──────┐      │
│  │   Analysis Pipeline         │      │
│  │  • CFG Builder              │      │
│  │  • Data Flow Analysis       │      │
│  │  • Taint Tracking           │      │
│  └─────────────────────────────┘      │
│                 │                     │
│  ┌──────────────▼─────────────┐       │
│  │  Rule Engine Interface     │       │
│  │  (Épica 2)                 │       │
│  └────────────────────────────┘       │
└─────────────────────────────────────────┘
```

### Dependencias Core

```toml
[dependencies]
# Parsing
tree-sitter = "0.24"
tree-sitter-rust = "0.23"
tree-sitter-go = "0.23"
oxc_parser = "0.20"  # TypeScript/JavaScript

# Async runtime
tokio = { version = "1", features = ["full"] }
rayon = "1.10"  # Parallel processing

# AST manipulation
syn = "2.0"  # Rust AST
quote = "1.0"

# Data structures
petgraph = "0.6"  # For CFG
```

---

## 📊 Estimación y Plan de Entrega

### Cronograma Épica 1 (6 meses)

| Semana | Tarea | Story Points | Entregable |
|--------|-------|--------------|------------|
| 1-2 | TASK-01-01: Parser base Rust | 5 | Parser funcionando |
| 3-5 | TASK-01-02: Análisis semántico | 8 | Analyzer trait |
| 6-8 | TASK-01-03: CFG Builder | 8 | CFG engine |
| 9-12 | TASK-01-04: DFA | 13 | DFA engine |
| 13-17 | TASK-01-05: Soporte Go | 13 | Go analyzer |
| 18-22 | TASK-01-06: Soporte TypeScript | 13 | TS/JS analyzer |
| 23-24 | TASK-01-07, 01-08: CI/CD | 8 | Integrations |
| 25-28 | TASK-01-09, 01-10: Extensibilidad | 13 | Plugin system |
| 29-30 | TASK-01-11: Documentación | 5 | Docs completos |

**Total Story Points:** 96
**Velocity Estimada:** 16 SP/sprint
**Sprints Necesarios:** 6
**Duración:** 6 meses

---

## 🧪 Estrategia de Testing

### Pirámide de Testing

1. **Unit Tests (70%)**
   - Parser tests por lenguaje
   - CFG builder tests
   - DFA tests
   - Analyzer tests

2. **Integration Tests (20%)**
   - End-to-end analysis tests
   - CI/CD integration tests
   - Multi-language project tests

3. **Performance Tests (10%)**
   - Benchmark vs SonarQube
   - Memory profiling
   - Scalability tests

### Herramientas de Testing

```toml
[dev-dependencies]
tokio-test = "0.4"
proptest = "1.4"  # Property-based testing
criterion = "0.5"  # Benchmarking
```

---

## 📚 Documentación Requerida

### Documentos a Crear

1. **API Documentation**
   - LanguageAnalyzer trait docs
   - Architecture guide
   - Extension API guide

2. **User Guides**
   - Getting started guide
   - Language support matrix
   - Performance tuning

3. **Developer Documentation**
   - Contributing guidelines
   - Adding new analyzers
   - Architecture decisions (ADRs)

---

## 🔄 Criterios de Done

Para que esta épica se considere **COMPLETADA**, todos los siguientes criterios deben cumplirse:

- [ ] ✅ 3 lenguajes soportados (Rust, Go, TypeScript/JavaScript)
- [ ] ✅ Performance: 2x-5x más rápido que SonarQube
- [ ] ✅ Memory: <800MB para análisis de 1M LOC
- [ ] ✅ Accuracy: >90% en detección de issues
- [ ] ✅ 100% tests en verde (unit + integration + performance)
- [ ] ✅ Documentación completa publicada
- [ ] ✅ Integración CI/CD funcionando
- [ ] ✅ Plugin system para extensibilidad
- [ ] ✅ Análisis incremental <1s
- [ ] ✅ Cache system con invalidación inteligente
- [ ] ✅ API estable y documentada

---

## 🚀 Siguiente Épica

Una vez completada esta épica, proceder con:
**[Épica 2: Security Analysis (SAST)](./EPIC-02-SECURITY_ANALYSIS_SAST.md)**

---

## 📞 Contacto y Soporte

**Technical Lead:** [A definir]
**Epic Owner:** [A definir]
**Slack Channel:** #hodei-scan-core
**Documentation:** `/docs/epics/`

---

*Última actualización: 10 de noviembre de 2025*
