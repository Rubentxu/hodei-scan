# hodei-scan v3.0: Arquitectura Definitiva del Motor de Correlación de Hechos

**Versión:** 3.0  
**Fecha:** 10 de noviembre de 2025  
**Estado:** 🚀 Arquitectura Final - Ready for Implementation  
**Base:** Principios de Cedar + Hechos Atómicos + Correlación Multi-Dominio

---

## 📋 Resumen Ejecutivo

hodei-scan v3.0 no es un "clon más rápido" de SonarQube. Es un **Motor de Gobernanza de Calidad** de nueva generación que adopta los principios de **Cedar** (motor de autorización de AWS) para lograr:

- ✅ **Idempotencia Absoluta** - Mismos Hechos + Reglas = Mismos Hallazgos (siempre)
- ✅ **Evaluación Stateless** - Motor de correlación en memoria (<2ms)
- ✅ **Esquema Formal** - IR Schema como contrato entre Extractores y Motor
- ✅ **DSL Declarativo** - Reglas `permit`/`forbid` legibles por humanos
- ✅ **Correlación Multi-Dominio** - SAST + SCA + Coverage en una sola consulta

---

## 🎯 El Paradigma: De SAST a "Governance Engine"

| Tradicional SAST (SonarQube) | hodei-scan v3.0 (Governance Engine) |
|------------------------------|-------------------------------------|
| "¿Tiene este código bugs?" | "¿Cumple este proyecto nuestra política de gobernanza?" |
| Reglas imperativas (Java) | Reglas declarativas (Cedar-like DSL) |
| Análisis con estado (JVM) | Motor stateless (<2ms) |
| Resultados instables | Idempotencia absoluta |
| Un dominio (SAST) | Correlación multi-dominio |

---

## 🏗️ Arquitectura v3.0: Flujo de Hechos Atómicos

```
┌──────────────────────────────────────────────────────────────────────────┐
│                        hodei-scan v3.0                                  │
│              Motor de Gobernanza de Calidad                              │
└──────────────────────────────────────────────────────────────────────────┘

ETAPA 1: EXTRACCIÓN (Extractores por Niveles, "Tontos")
┌─────────────────────────────────────────────────────────────────────────┐
│                                                                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │ Extractor    │  │ Extractor    │  │ Extractor    │  │ Extractor    │ │
│  │ Nivel 1      │  │ Nivel 2      │  │ SCA          │  │ Coverage     │ │
│  │ (AST Rápido) │  │ (SAST Deep)  │  │ (Deps)       │  │ (Tests)      │ │
│  │              │  │              │  │              │  │              │ │
│  │ • tree-sitter│  │ • DFA/CFG    │  │ • Cargo/NPM  │  │ • Coverage   │ │
│  │ • Oxc        │  │ • Taint      │  │ • CVE DB     │  │ • LCOV       │ │
│  │ • Regex      │  │ • OWASP      │  │ • Snyk API   │  │ • JaCoCo     │ │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘ │
│         │                 │                 │                 │         │
│         └─────────────────┼─────────────────┼─────────────────┘         │
│                           │                 │                             │
│                           ▼                 ▼                             │
│ ┌───────────────────────────────────────────────────────────────────────┐ │
│ │         IR (Intermediate Representation)                               │ │
│ │         "La Base de Datos de Hechos Atómicos"                          │ │
│ │         (Validado por 'IR Schema')                                     │ │
│ └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ ETAPA 2: EVALUACIÓN (Motor DSL Cedar-like, "Inteligente")                │
│          (Stateless, Idempotente, <2ms)                                    │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  Reglas DSL:                                                            │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ forbid on { TaintSource && TaintSink }                             │ │
│  │   when { severity == "critical" }                                  │ │
│  │                                                                        │ │
│  │ forbid on { Complexity > 20 }                                      │ │
│  │   when { user.role == "junior" }                                   │ │
│  │                                                                        │ │
│  │ PERLA: forbid on { TaintSink && UncoveredLine }                    │ │
│  │   "Vulnerabilidades sin tests = Bloqueo de merge"                  │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                    HALLAZGOS (Findings)                                  │
│ • Finding(rule="CRITICAL_RISK", file="a.js", line=42,                    │
│     correlation=["SAST", "COVERAGE"])                                   │
│ • Finding(rule="DEPENDENCY_VULNERABILITY", cve="CVE-2024-1234",          │
│     correlation=["SCA", "SAST"])                                        │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 💎 Principio 1: Hechos Atómicos (Atomic Facts)

### ¿Qué es un "Hecho Atómico"?

Un **Hecho Atómico** es la unidad mínima e indivisible de información sobre un proyecto de software. Es **autónomo** y **verificable** por sí mismo.

```rust
// Ejemplo de Hechos Atómicos
Fact {
    type: "Function",
    attributes: {
        name: "authenticate",
        file: "auth.js",
        line: 42,
        complexity: 15
    }
}

Fact {
    type: "TaintSource",
    attributes: {
        var: "$_GET['id']",
        file: "handler.php",
        line: 10
    }
}

Fact {
    type: "TaintSink",
    attributes: {
        func: "db.query",
        file: "auth.js",
        line: 45
    }
}

Fact {
    type: "UncoveredLine",
    attributes: {
        file: "auth.js",
        line: 45,
        coverage: 0
    }
}

Fact {
    type: "Dependency",
    attributes: {
        name: "lodash",
        version: "4.17.20",
        cve: "CVE-2024-5678",
        severity: "critical"
    }
}
```

### Ventajas de los Hechos Atómicos

1. **Trazabilidad**: Cada hecho puede ser verificado independientemente
2. **Escalabilidad**: Procesamiento paralelo de hechos independientes
3. **Flexibilidad**: Nuevos tipos de hechos sin cambiar la lógica de evaluación
4. **Correlación**: Hechos de diferentes dominios pueden combinarse

---

## ⚖️ Principio 2: IR Schema (El Contrato)

El **IR Schema** es un contrato formal que define qué tipos de hechos son válidos. Es la "gramática" que tanto extractores como el motor de evaluación entienden.

```rust
// Esquema de Tipos de Hechos (IR Schema v3.0)
pub enum FactType {
    // === DOMINIO SAST ===
    Function {
        name: String,
        visibility: String,
        complexity: u32,
    },
    TaintSource {
        var: String,
        confidence: f64,
    },
    TaintSink {
        func: String,
        category: String,
    },
    UnsafeCall {
        function_name: String,
        severity: String,
    },
    Vulnerability {
        cwe_id: String,
        owasp_category: String,
        confidence: f64,
    },
    
    // === DOMINIO SCA (Dependencias) ===
    Dependency {
        name: String,
        version: String,
        cve: Option<String>,
        cvss_score: Option<f64>,
    },
    
    // === DOMINIO COVERAGE (Calidad) ===
    UncoveredLine {
        file: String,
        line: u32,
        coverage_percentage: f64,
    },
    LowTestCoverage {
        file: String,
        percentage: f64,
    },
    
    // === DOMINIO CORRELATION ===
    Correlation {
        domains: Vec<String>, // ["SAST", "SCA", "COVERAGE"]
        fact_ids: Vec<String>,
    },
}
```

**Validación por Schema**: Si un extractor intenta crear un hecho malformado, el IR lo **rechaza** en tiempo de compilación.

---

## 🔌 Principio 3: Motor Stateless (Evaluación en Memoria)

### ¿Por qué Stateless?

Un motor **stateless** (sin estado) no guarda información entre ejecuciones. Cada evaluación es independiente.

```rust
// La función de evaluación es una FUNCIÓN PURA
pub fn evaluate(
    facts: &[Fact],        // Solo entrada: Hechos
    rules: &[Rule],        // Solo entrada: Reglas
) -> Vec<Finding> {       // Solo salida: Hallazgos
    // Evaluación determinística
    // Mismo input = Mismo output (siempre)
}
```

### Ventajas del Patrón Stateless

1. **Idempotencia Absoluta**:
   - Commit A falla → Siempre falla hasta que cambien Hechos o Reglas
   - No hay "carreras de estado" en CI/CD

2. **Escalabilidad Masiva**:
   - 10,000 evaluaciones en paralelo (Kubernetes, AWS Lambda)
   - Cada evaluación es independiente

3. **Caching Perfecto**:
   - Hash(Hechos) + Hash(Reglas) = Resultado cacheable
   - Si hash no cambió → Resultado es idéntico

4. **Rendimiento**:
   - Evaluación en memoria (<2ms)
   - No overhead de state management

---

## 📜 Principio 4: DSL Declarativo (Cedar-like)

Las reglas no son código imperativo, sino **especificaciones declarativas** de qué queremos permitir o prohibir.

### Sintaxis DSL v3.0

```cedar
// === REGLAS DE SEGURIDAD (OWASP) ===

// SQL Injection: Prohibir fuentes de datos no confiables que lleguen a sinks SQL
forbid on { TaintSource && TaintSink }
  when { sink.category == "SQL" }
  severity: "critical"
  message: "SQL Injection vulnerability detected"

// Command Injection: Prohibir system calls con datos no confiables
forbid on { TaintSource && UnsafeCall }
  when { unsafe_call.function_name in ["system", "exec", "shell"] }
  severity: "critical"
  message: "Command injection vulnerability"

// === REGLAS DE CALIDAD ===

// Complejidad: Prohibir funciones muy complejas para junior developers
forbid on { Function }
  when { 
    complexity > 15 && 
    user.role == "junior"
  }
  severity: "medium"
  message: "Function complexity too high for junior developer"

// === REGLA KILLER: CORRELACIÓN SAST + COVERAGE ===

// Prohibir vulnerabilidades en líneas sin tests
forbid on { TaintSink && UncoveredLine }
  when { vulnerability.confidence > 0.9 }
  severity: "blocker"
  message: "Critical vulnerability in uncovered line - BLOCK MERGE"
  correlation: ["SAST", "COVERAGE"]

// === CORRELACIÓN SCA + SAST ===

// Prohibir uso de dependencias vulnerables en funciones críticas
forbid on { Dependency && TaintSink }
  when { 
    dependency.cvss_score > 9.0 &&
    sink.func in ["authenticate", "authorize", "process_payment"]
  }
  severity: "blocker"
  message: "Critical dependency vulnerability in security-sensitive function"
  correlation: ["SCA", "SAST"]
```

### Ventajas del DSL Declarativo

1. **Legibilidad**: Un CISO puede leer y entender las reglas
2. **Auditoría**: Cambios de reglas = Git diff claro
3. **Agilidad**: Modificar reglas sin recompilar
4. **Validación**: Errores de sintaxis en tiempo de parse

---

## 🔗 Principio 5: Correlación Multi-Dominio (El "Moat" Competitivo)

Esta es la capacidad única de hodei-scan: **combinar hechos de diferentes dominios** en una sola consulta.

### Ejemplos de Correlación

#### Ejemplo 1: SAST + Coverage = "Vulnerabilidades sin Tests"
```cedar
forbid on { TaintSink && UncoveredLine }
```
**Traducción**: "Bloquear merge si hay vulnerabilidades en líneas sin tests"

**Valor**: Las herramientas tradicionales no pueden hacer esta correlación. Un vulnerability scanner dice "tienes XSS". Un coverage tool dice "línea 45 no está cubierta". **hodei-scan dice**: "Tienes XSS en línea 45, que además no está cubierta por tests → RIESGO CRÍTICO"

#### Ejemplo 2: SCA + SAST = "Dependencias Vulnerables en Funciones Críticas"
```cedar
forbid on { Dependency && TaintSink }
  when { dependency.cvss_score > 9.0 && sink.func in security_functions }
```
**Traducción**: "Prohibir dependencias con CVSS >9.0 en funciones de seguridad"

**Valor**: Un dependency scanner dice "lodash tiene CVE-2024-5678". Un SAST tool dice "función processPayment es compleja". **hodei-scan dice**: "lodash (CVE-2024-5678, CVSS 9.8) se usa en processPayment → BLOQUEAR"

#### Ejemplo 3: SAST + SCA + Coverage = "Tormenta Perfecta"
```cedar
forbid on { TaintSource && TaintSink && Dependency && UncoveredLine }
  when { dependency.cvss_score > 8.0 }
  severity: "blocker"
  message: "Perfect storm: vulnerable dependency + uncovered taint flow"
  correlation: ["SAST", "SCA", "COVERAGE"]
```

### Matriz de Correlación

| Dominio 1 | Dominio 2 | Correlación | Ejemplo de Regla |
|-----------|-----------|-------------|------------------|
| SAST | Coverage | Vulnerabilidades sin tests | `TaintSink && UncoveredLine` |
| SCA | SAST | Dependencias vulnerables en funciones críticas | `Dependency && TaintSink` |
| SAST | SCA | Vulnerabilidades de código + dependencias | `Vulnerability && Dependency` |
| Coverage | SAST | Código complejo sin tests | `Function.complexity > 20 && UncoveredLine` |
| SCA | Coverage | Dependencias vulnerables no testadas | `Dependency && UncoveredLine` |
| SAST | SCA | Coverage | Los 3 juntos = "Tormenta perfecta" |

---

## 🚀 Flujo de Implementación

### Fase 1: Refactorización del IR Schema
- [ ] Actualizar `FactType` enum para incluir todos los dominios
- [ ] Añadir tipos de correlación
- [ ] Crear validador de schema

### Fase 2: Extractores "Tontos"
- [ ] Refactor extractors para solo generar hechos
- [ ] Separar extracción (nivel 1: rápido) de análisis profundo (nivel 2)
- [ ] Añadir extractores SCA y Coverage

### Fase 3: Motor DSL v3.0
- [ ] Implementar parser Cedar-like
- [ ] Crear evaluador stateless
- [ ] Añadir correlación multi-dominio

### Fase 4: Reglas de Correlación
- [ ] Escribir reglas SAST + Coverage
- [ ] Escribir reglas SCA + SAST
- [ ] Crear reglas de "tormenta perfecta"

### Fase 5: Optimización
- [ ] Caching basado en hashes
- [ ] Paralelización con Rayon
- [ ] Benchmarking <2ms

---

## 📊 Métricas de Éxito

| Métrica | Target | SonarQube | hodei-scan v3.0 |
|---------|--------|-----------|-----------------|
| **Tiempo de evaluación** | <2ms | ~500ms | ✅ <2ms |
| **Idempotencia** | 100% | No | ✅ Sí |
| **Correlación multi-dominio** | Sí | No | ✅ Sí |
| **Legibilidad de reglas** | 100% | Java code | ✅ DSL declarativo |
| **Paralelización** | 10K evals | Limitado | ✅ Stateless = ilimitado |

---

## 🎓 Conclusiones

hodei-scan v3.0 no compite en la misma categoría que SonarQube. **Crea una nueva categoría**:

- **SonarQube**: Static Analysis Tool
- **hodei-scan**: **Quality Governance Engine**

Al adoptar principios de Cedar + Hechos Atómicos + Correlación Multi-Dominio, obtenemos:

1. ✅ **Fiabilidad** (Idempotencia)
2. ✅ **Velocidad** (Motor Stateless)
3. ✅ **Robustez** (IR Schema)
4. ✅ **Mantenibilidad** (DSL Declarativo)
5. ✅ **Ventaja Competitiva** (Correlación Multi-Dominio)

Esta arquitectura no es solo viable; es **superior** y aborda las necesidades del análisis de software moderno.

---

## 📚 Referencias

- [Cedar Policy Language (AWS)](https://www.cedarpolicy.com/)
- [Intermediate Representation Best Practices](https://www.microsoft.com/en-us/research/project/static-single-assignment/)
- [Stateless Architecture Patterns](https://www.martinfowler.com/articles/201403-course.html)
- [Atomic Facts in Correlation Engines](https://www.sciencedirect.com/topics/computer-science/data-correlation)
- [Declarative DSL Design](https://martinfowler.com/articles/declarative-dsl.html)

---

**Estado Final**: ✅ Arquitectura v3.0 Definida  
**Siguiente**: Implementación siguiendo esta arquitectura
