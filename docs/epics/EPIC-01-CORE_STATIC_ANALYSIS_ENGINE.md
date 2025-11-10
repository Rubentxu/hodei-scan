# ÉPICA-01: CORE STATIC ANALYSIS ENGINE

**Versión:** 2.0
**Fecha:** 10 de noviembre de 2025
**Story Points:** 89 SP
**Sprint Estimado:** 6 sprints
**Dependencias:** Ninguna (ÉPICA BASE)
**Estado:** 🚀 Ready for Implementation

---

## 📋 Descripción de la Épica

Esta épica implementa el **core del motor de análisis estático basado en Arquitectura IR (Intermediate Representation)** que separa la extracción de datos de la evaluación de reglas. Es la foundation sobre la cual se construyen todas las demás funcionalidades de hodei-scan v2.0.

**Cambio de Paradigma:**
```
v1.0 (Obsoleto):  Parsing → Rules → Findings (Acoplado a lenguajes)
v2.0 (IR):        Parsers → IR → DSL → Findings (Universal y escalable)
```

**Objetivo Principal:** Establecer una arquitectura IR que resuelva los problemas de escalabilidad y permita análisis universales cross-language con correlación multi-dominio.

---

## 🎯 Objetivos y Alcance

### Objetivos Estratégicos
1. **Implementar IR Schema v2.0** - Facts universales + correlaciones cross-domain
2. **Desarrollar Rule Engine DSL** - Motor que consulta IR (no ASTs)
3. **Crear extractores multi-lenguaje** - JS/Python/Go → IR
4. **Establecer caching layer** - IR storage y retrieval optimizado
5. **Implementar WASM runtime** - Sandbox para reglas custom

### Alcance Funcional
- ✅ **IR Core Engine**: Schema, facts, correlaciones
- ✅ **Rule Engine DSL**: Parser + evaluator
- ✅ **JavaScript Extractor**: Oxc → IR
- ✅ **Python Extractor**: tree-sitter + ruff → IR
- ✅ **Go Extractor**: tree-sitter → IR
- ✅ **TypeScript Extractor**: Oxc → IR
- ✅ **Caching System**: Redis + PostgreSQL
- ✅ **WASM Sandbox**: Runtime para reglas custom
- ✅ **Performance Benchmarks**: vs SonarQube/CodeQL

### Fuera de Alcance
- ❌ Frontend UI (ÉPICA-WEB-01+)
- ❌ Security rules específicas (ÉPICA-02)
- ❌ SCA analysis (ÉPICA-03)
- ❌ Code coverage (ÉPICA-04)

---

## 👥 Historias de Usuario

### US-01: IR Schema Definition
**Como** arquitecto de software
**Quiero** definir un IR Schema v2.0 que represente facts universales
**Para** desacoplar la extracción de la evaluación de reglas

**Criterios de Aceptación:**
```
GIVEN un código fuente en cualquier lenguaje
WHEN el sistema analiza el código
THEN se genera un IR con facts universales independientes del lenguaje

GIVEN facts de seguridad, calidad, coverage y dependencias
WHEN se almacenan en IR
THEN se pueden correlacionar cross-domain

GIVEN un IR schema v2.0
WHEN se serializa con Cap'n Proto
THEN el tamaño es 10x menor que JSON

GIVEN un analysis_id único
WHEN se genera IR
THEN incluye timestamp, metadata, facts, dependencies y correlations
```

**Tareas Técnicas:**
- [ ] Diseñar IR Schema v2.0 con Fact, FactType, CodeLocation
- [ ] Implementar FactType enum con Security, Quality, Coverage, SCA
- [ ] Crear FactProvenance para tracking de source
- [ ] Implementar IR serialization con Cap'n Proto
- [ ] Crear IR deserialization
- [ ] Implementar Fact correlation engine
- [ ] Escribir tests unitarios para IR Schema

**TDD Tests:**
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn should_create_ir_with_facts() {
        // Given: Código fuente
        // When: Se genera IR
        // Then: Contiene facts universales
    }

    #[test]
    fn should_serialize_ir_with_capnp() {
        // Given: IR con facts
        // When: Se serializa
        // Then: Tamaño es 10x menor que JSON
    }

    #[test]
    fn should_correlate_facts_cross_domain() {
        // Given: Facts de security + coverage
        // When: Se correlacionan
        // Then: Se genera correlated fact
    }

    #[test]
    fn should_handle_multiple_fact_types() {
        // Given: Facts de security, quality, coverage
        // When: Se procesan
        // Then: Todos se almacenan correctamente
    }
}
```

---

### US-02: Cedar-Inspired DSL Rule Engine
**Como** security engineer
**Quiero** definir reglas usando una DSL Cedar-inspired
**Para** escribir reglas universales que funcionen en todos los lenguajes

**Criterios de Aceptación:**
```
GIVEN una regla en DSL: permit(rule: "SEC-001", severity: "critical") on { unsafe_call + sql_sink }
WHEN se evalúa contra IR facts
THEN se genera un finding con severidad critical si la condición se cumple

GIVEN una regla DSL con correlaciones
WHEN se evalúa
THEN funciona igual en JS, Python, Go

GIVEN un parser de DSL
WHEN recibe una regla malformada
THEN retorna error detallado con línea y columna

GIVEN múltiples reglas
WHEN se evalúan en paralelo
THEN el tiempo de evaluación es <100ms para 1000 facts
```

**Tareas Técnicas:**
- [ ] Diseñar DSL grammar basada en Cedar
- [ ] Implementar parser combinator con Nom
- [ ] Crear rule evaluator engine
- [ ] Implementar parallel evaluation con Rayon
- [ ] Crear error handling con context
- [ ] Implementar rule caching
- [ ] Escribir tests de parser y evaluator

**TDD Tests:**
```rust
#[cfg(test)]
mod dsl_tests {
    #[test]
    fn should_parse_simple_rule() {
        // Given: DSL string
        // When: Se parsea
        // Then: Se genera AST correcto
    }

    #[test]
    fn should_evaluate_rule_against_ir() {
        // Given: Regla DSL + IR facts
        // When: Se evalúa
        // Then: Se generan findings correctos
    }

    #[test]
    fn should_handle_rule_syntax_error() {
        // Given: DSL malformado
        // When: Se parsea
        // Then: Error con línea y columna
    }

    #[test]
    fn should_evaluate_rules_in_parallel() {
        // Given: 1000 reglas + facts
        // When: Se evalúan en paralelo
        // Then: Tiempo <100ms
    }

    #[test]
    fn should_cache_evaluated_rules() {
        // Given: Regla evaluada
        // When: Se vuelve a evaluar con mismos facts
        // Then: Se usa cache (10x más rápido)
    }
}
```

---

### US-03: JavaScript Extractor (Oxc)
**Como** developer
**Quiero** que el sistema extraiga facts de JavaScript usando Oxc
**Para** tener análisis universal de código JS

**Criterios de Aceptación:**
```
GIVEN un archivo JavaScript (.js, .mjs, .cjs)
WHEN se analiza con Oxc
THEN se extraen facts: functions, variables, unsafe_calls, dependencies

GIVEN código TypeScript
WHEN se analiza
THEN se extraen facts de tipos y semántica

GIVEN un proyecto de 100K LOC JS
WHEN se extrae IR
THEN toma <5 segundos

GIVEN Oxc semantic model
WHEN se extraen facts
THEN se convierten a IR facts universales
```

**Tareas Técnicas:**
- [ ] Integrar Oxc parser library
- [ ] Implementar semantic analyzer adapter
- [ ] Crear mapper Oxc → IR facts
- [ ] Implementar AST traversal
- [ ] Crear error handling para parsing errors
- [ ] Optimizar performance para 100K LOC
- [ ] Escribir tests con código JS real

**TDD Tests:**
```rust
#[cfg(test)]
mod js_extractor_tests {
    #[test]
    fn should_extract_facts_from_js_file() {
        // Given: Archivo JS con function y variable
        // When: Se extrae IR
        // Then: Contiene Function y Variable facts
    }

    #[test]
    fn should_extract_unsafe_calls() {
        // Given: Código con eval(), innerHTML
        // When: Se analiza
        // Then: Se extraen UnsafeCall facts
    }

    #[test]
    fn should_handle_typescript() {
        // Given: Archivo TS con tipos
        // When: Se analiza
        // Then: Se extraen facts de tipos
    }

    #[test]
    fn should_process_100k_loc_in_5s() {
        // Given: Proyecto JS 100K LOC
        // When: Se extrae IR
        // Then: Tiempo <5s
    }

    #[test]
    fn should_handle_parse_errors_gracefully() {
        // Given: JS con syntax error
        // When: Se parsea
        // Then: Error con location y contexto
    }
}
```

---

### US-04: Python Extractor (tree-sitter + ruff)
**Como** developer
**Quiero** que el sistema extraiga facts de Python usando tree-sitter y ruff
**Para** tener análisis universal de código Python

**Criterios de Aceptación:**
```
GIVEN un archivo Python (.py)
WHEN se analiza con tree-sitter + ruff
THEN se extraen facts: functions, classes, variables, imports, unsafe_calls

GIVEN código Python con type hints
WHEN se analiza
THEN se extraen facts de tipos

GIVEN un proyecto Python con 50K LOC
WHEN se extrae IR
THEN toma <3 segundos

GIVEN tree-sitter AST + ruff diagnostics
WHEN se combinan
THEN se generan IR facts completos
```

**Tareas Técnicas:**
- [ ] Integrar tree-sitter-python
- [ ] Integrar ruff library
- [ ] Crear mapper AST + diagnostics → IR facts
- [ ] Implementar symbol table extraction
- [ ] Crear import resolution
- [ ] Optimizar para 50K LOC
- [ ] Escribir tests con código Python real

**TDD Tests:**
```rust
#[cfg(test)]
mod python_extractor_tests {
    #[test]
    fn should_extract_facts_from_py_file() {
        // Given: Archivo Python con class y function
        // When: Se extrae IR
        // Then: Contiene Class y Function facts
    }

    #[test]
    fn should_extract_ruff_diagnostics() {
        // Given: Código con ruff violations
        // When: Se analiza
        // Then: Se extraen facts de calidad
    }

    #[test]
    fn should_extract_imports() {
        // Given: Código con imports
        // When: Se extrae IR
        // Then: Contiene Dependency facts
    }

    #[test]
    fn should_handle_type_hints() {
        // Given: Código con type hints
        // When: Se analiza
        // Then: Se extraen Type facts
    }
}
```

---

### US-05: Go Extractor (tree-sitter)
**Como** developer
**Quiero** que el sistema extraiga facts de Go usando tree-sitter
**Para** tener análisis universal de código Go

**Criterios de Aceptación:**
```
GIVEN un archivo Go (.go)
WHEN se analiza con tree-sitter-go
THEN se extraen facts: functions, structs, variables, imports, interfaces

GIVEN código Go con generics
WHEN se analiza
THEN se extraen facts de tipos

GIVEN un proyecto Go con 100K LOC
WHEN se extrae IR
THEN toma <4 segundos

GIVEN Go modules
WHEN se analizan
THEN se extraen dependency facts
```

**Tareas Técnicas:**
- [ ] Integrar tree-sitter-go
- [ ] Implementar Go semantic analyzer
- [ ] Crear mapper Go AST → IR facts
- [ ] Implementar Go modules resolution
- [ ] Crear interface extraction
- [ ] Optimizar para 100K LOC
- [ ] Escribir tests con código Go real

**TDD Tests:**
```rust
#[cfg(test)]
mod go_extractor_tests {
    #[test]
    fn should_extract_facts_from_go_file() {
        // Given: Archivo Go con struct y function
        // When: Se extrae IR
        // Then: Contiene Struct y Function facts
    }

    #[test]
    fn should_extract_interfaces() {
        // Given: Código con interfaces
        // When: Se extrae IR
        // Then: Contiene Interface facts
    }

    #[test]
    fn should_handle_generics() {
        // Given: Código con generics
        // When: Se analiza
        // Then: Se extraen Type facts
    }

    #[test]
    fn should_extract_go_modules() {
        // Given: go.mod file
        // When: Se analiza
        // Then: Se extraen Dependency facts
    }
}
```

---

### US-06: TypeScript Extractor
**Como** developer
**Quiero** que el sistema extraiga facts de TypeScript usando Oxc
**Para** tener análisis universal de código TypeScript

**Criterios de Aceptación:**
```
GIVEN un archivo TypeScript (.ts, .tsx)
WHEN se analiza con Oxc
THEN se extraen facts: functions, interfaces, types, unsafe_calls

GIVEN código TypeScript con generics y conditional types
WHEN se analiza
THEN se extraen facts de tipos complejos

GIVEN un proyecto TS de 150K LOC
WHEN se extrae IR
THEN toma <6 segundos

GIVEN Oxc semantic model
WHEN se extraen types
THEN se preserva información semántica
```

**Tareas Técnicas:**
- [ ] Integrar Oxc para TypeScript
- [ ] Implementar type resolver
- [ ] Crear mapper TS → IR facts
- [ ] Implementar generics handling
- [ ] Crear interface merging
- [ ] Optimizar para 150K LOC
- [ ] Escribir tests con código TS real

**TDD Tests:**
```rust
#[cfg(test)]
mod ts_extractor_tests {
    #[test]
    fn should_extract_facts_from_ts_file() {
        // Given: Archivo TypeScript con interface
        // When: Se extrae IR
        // Then: Contiene Interface y Type facts
    }

    #[test]
    fn should_extract_generics() {
        // Given: Código con generics
        // When: Se analiza
        // Then: Se extraen GenericType facts
    }

    #[test]
    fn should_handle_jsx() {
        // Given: Archivo TSX
        // When: Se extrae IR
        // Then: Contiene JSXElement facts
    }

    #[test]
    fn should_extract_conditional_types() {
        // Given: Código con conditional types
        // When: Se analiza
        // Then: Se extraen Type facts
    }
}
```

---

### US-07: Caching System (Redis + PostgreSQL)
**Como** developer
**Quiero** un sistema de caching inteligente para IR
**Para** acelerar análisis incrementales 30-120x

**Criterios de Aceptación:**
```
GIVEN un archivo modificado
WHEN se re-analiza
THEN se usa IR cache y toma <1s (vs 30-120s sin cache)

GIVEN un analysis_id
WHEN se guarda IR en cache
THEN se almacena en Redis para acceso rápido

GIVEN datos históricos
WHEN se necesitan persistencia
THEN se almacenan en PostgreSQL

GIVEN cache miss
WHEN se necesita IR
THEN se recalcula y se cachea automáticamente
```

**Tareas Técnicas:**
- [ ] Diseñar cache key strategy
- [ ] Implementar Redis cache layer
- [ ] Implementar PostgreSQL persistence
- [ ] Crear cache invalidation strategy
- [ ] Implementar cache warming
- [ ] Crear cache hit/miss metrics
- [ ] Escribir tests de cache

**TDD Tests:**
```rust
#[cfg(test)]
mod cache_tests {
    #[test]
    fn should_cache_ir_with_key() {
        // Given: IR y analysis_id
        // When: Se cachea
        // Then: Se puede recuperar con key
    }

    #[test]
    fn should_retrieve_cached_ir() {
        // Given: IR cacheado
        // When: Se recupera
        // Then: Datos idénticos al original
    }

    #[test]
    fn should_invalidate_cache_on_change() {
        // Given: IR cacheado
        // When: Archivo cambia
        // Then: Cache se invalida
    }

    #[test]
    fn should_warm_cache_preemptively() {
        // Given: Archivos frecuentemente accedidos
        // When: Sistema idle
        // Then: Se pre-cargan en cache
    }

    #[test]
    fn should_persist_to_postgresql() {
        // Given: Datos históricos
        // When: Se persisten
        // Then: Disponibles después de restart
    }
}
```

---

### US-08: WASM Runtime for Custom Rules
**Como** security engineer
**Quiero** ejecutar reglas custom en sandbox WASM
**Para** tener reglas enterprise seguras sin comprometer el core

**Criterios de Aceptación:**
```
GIVEN una regla custom en WASM
WHEN se ejecuta en sandbox
THEN no puede acceder al filesystem ni red

GIVEN múltiples reglas WASM
WHEN se ejecutan
THEN están aisladas entre sí

GIVEN una regla con infinite loop
WHEN se ejecuta
THEN se termina después de timeout configurable

GIVEN reglas WASM compiled
WHEN se cargan
THEN tiempo de carga <100ms
```

**Tareas Técnicas:**
- [ ] Integrar WASM runtime (Wasmtime)
- [ ] Implementar sandbox isolation
- [ ] Crear rule loader
- [ ] Implementar timeout mechanism
- [ ] Crear memory limits
- [ ] Implementar error handling
- [ ] Escribir tests de sandbox

**TDD Tests:**
```rust
#[cfg(test)]
mod wasm_tests {
    #[test]
    fn should_load_wasm_rule() {
        // Given: WASM rule binary
        // When: Se carga
        // Then: Se puede ejecutar
    }

    #[test]
    fn should_isolate_wasm_execution() {
        // Given: WASM rule ejecutándose
        // When: Intenta acceso a filesystem
        // Then: Error de sandbox violation
    }

    #[test]
    fn should_timeout_infinite_loop() {
        // Given: WASM rule con infinite loop
        // When: Se ejecuta
        // Then: Termina por timeout
    }

    #[test]
    fn should_enforce_memory_limit() {
        // Given: WASM rule que usa mucha memoria
        // When: Se ejecuta
        // Then: Termina por memory limit
    }
}
```

---

### US-09: Performance Benchmarking
**Como** technical lead
**Quiero** comparar performance vs SonarQube y CodeQL
**Para** validar ventajas del paradigma IR

**Criterios de Aceptación:**
```
GIVEN proyecto 100K LOC
WHEN se analiza con hodei-scan
THEN toma <5s (vs 5min SonarQube = 60x más rápido)

GIVEN análisis incremental
WHEN se re-analiza archivo modificado
THEN toma <1s (vs 30-120s SonarQube = 30-120x más rápido)

GIVEN rule evaluation
WHEN se evalúan 1000 reglas sobre IR cacheado
THEN toma <100ms (vs 10-20ms SonarQube)

GIVEN 6 lenguajes soportados
WHEN se añade lenguaje 7
THEN toma 2-3 semanas (vs 3-6 meses SonarQube)
```

**Tareas Técnicas:**
- [ ] Crear benchmarking suite
- [ ] Implementar comparison metrics
- [ ] Crear test projects (JS, Python, Go, TS)
- [ ] Implementar performance profiling
- [ ] Crear automated reports
- [ ] Integrar CI benchmarking
- [ ] Escribir benchmarks tests

**TDD Tests:**
```rust
#[cfg(test)]
mod benchmark_tests {
    #[test]
    fn should_analyze_100k_loc_in_5s() {
        // Given: Proyecto 100K LOC
        // When: Se analiza
        // Then: Tiempo <5s
    }

    #[test]
    fn should_do_incremental_in_1s() {
        // Given: IR cacheado
        // When: Se re-analiza archivo modificado
        // Then: Tiempo <1s
    }

    #[test]
    fn should_evaluate_1000_rules_in_100ms() {
        // Given: 1000 reglas + IR
        // When: Se evalúan
        // Then: Tiempo <100ms
    }

    #[test]
    fn should_measure_cache_hit_ratio() {
        // Given: Sistema con cache
        // When: Se hacen múltiples análisis
        // Then: Cache hit ratio >90%
    }
}
```

---

## ✅ Criterios de Validación

### Funcionales
- [ ] IR Schema v2.0 implementado y documentado
- [ ] DSL Parser/Evaluator funcional para reglas universales
- [ ] Extractores para JS, Python, Go, TypeScript operativos
- [ ] Caching layer con >90% hit ratio
- [ ] WASM sandbox seguro y funcional
- [ ] Benchmarks superan targets establecidos

### No Funcionales
- [ ] **Performance**: <5s para 100K LOC
- [ ] **Escalabilidad**: O(N+M) vs O(N×M) tradicional
- [ ] **Accuracy**: >95% consistency cross-language
- [ ] **False Positives**: <10%
- [ ] **Cache Hit Ratio**: >90%
- [ ] **Memory Usage**: <800MB pico
- [ ] **Startup Time**: <2s cold start

### Calidad de Código
- [ ] Cobertura de tests: >90%
- [ ] Linting: 100% pass
- [ ] Documentación KDoc: 100% públicas
- [ ] Review de código: 2 approvals
- [ ] CI pipeline: 100% green

---

## 📊 Métricas de Éxito

| Métrica | Target | Actual | Status |
|---------|--------|--------|--------|
| **IR Generation Speed** | <5s / 100K LOC | - | ⏳ |
| **Rule Evaluation** | <100ms cached | - | ⏳ |
| **Incremental Analysis** | <1s changes | - | ⏳ |
| **Cache Hit Ratio** | >90% | - | ⏳ |
| **Multi-language Accuracy** | >95% consistency | - | ⏳ |
| **False Positive Rate** | <10% | - | ⏳ |
| **System Uptime** | 99.9% | - | ⏳ |
| **Add New Language** | 2-3 semanas | - | ⏳ |

---

## 🔗 Dependencias

### Internas
- Ninguna (ÉPICA BASE)

### Externas
- **Oxc Parser** (JavaScript/TypeScript)
- **tree-sitter** (Python, Go)
- **ruff** (Python linting)
- **Redis** (caching)
- **PostgreSQL** (persistence)
- **Wasmtime** (WASM runtime)
- **Cap'n Proto** (serialization)
- **Tokio** (async runtime)
- **Axum** (HTTP framework)
- **Rayon** (parallelism)

---

## ⚠️ Riesgos y Mitigación

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|-------------|---------|------------|
| **Oxc API instability** | Media | Alto | Usar version pinning + wrapper |
| **Performance targets no alcanzados** | Media | Alto | Early benchmarking + optimization |
| **IR Schema changes** | Alta | Medio | Versioning strategy + migration |
| **WASM complexity** | Media | Medio | Start simple + iterate |
| **Cross-language consistency** | Alta | Alto | Comprehensive test suite |

---

## 🚀 Plan de Implementación

### Sprint 1 (2 semanas): IR Foundation
- Implementar IR Schema v2.0
- Crear Cap'n Proto serialization
- Implementar base Fact structures

### Sprint 2 (2 semanas): DSL Engine
- Implementar DSL parser
- Crear rule evaluator
- Implementar parallel evaluation

### Sprint 3 (2 semanas): JavaScript Extractor
- Integrar Oxc
- Implementar mapper JS → IR
- Optimizar performance

### Sprint 4 (2 semanas): Python + Go Extractors
- Implementar Python extractor (tree-sitter + ruff)
- Implementar Go extractor (tree-sitter)
- Cross-language validation

### Sprint 5 (2 semanas): TypeScript Extractor
- Implementar TypeScript extractor
- Generics + conditional types support
- Integration testing

### Sprint 6 (2 semanas): Caching + WASM + Benchmarks
- Implementar caching system
- Implementar WASM sandbox
- Create benchmarking suite
- Performance validation

---

## 📚 Referencias Técnicas

- [IR Architecture Specification](./ARCHITECTURE.md#ir-architecture)
- [TDD Methodology](../TDD_METHODOLOGY.md)
- [Oxc Documentation](https://oxc-project.github.io/)
- [tree-sitter](https://tree-sitter.github.io/)
- [Cedar Policy Language](https://cedarpolicy.github.io/)
- [Cap'n Proto](https://capnproto.org/)
- [Wasmtime](https://wasmtime.dev/)

---

**Estado:** ✅ Documentación Completa - Ready for Development
**Próximos Pasos:** Crear EPIC-02-SECURITY_ANALYSIS_SAST.md
