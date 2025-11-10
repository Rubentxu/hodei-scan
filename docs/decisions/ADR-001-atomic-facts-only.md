# ADR-001: Facts Must Be Atomic, Correlations Are Findings

**Status:** ✅ Accepted (2025-01-XX)  
**Supersedes:** v3.1 IR Schema with meta-facts  
**Authors:** Architecture Team  
**Reviewers:** Core Team, Security Team

---

## Context

En la versión v3.1 de hodei-scan, el IR Schema incluía tres variantes de `FactType` que representaban **correlaciones multi-dominio**:

1. **`VulnerableUncovered`**: Correlación entre vulnerabilidades SAST y cobertura de testing
2. **`SecurityTechnicalDebt`**: Métrica derivada de múltiples fuentes
3. **`QualitySecurityCorrelation`**: Agregación de scores de calidad y seguridad

### Problema Identificado

**Violación de Separation of Concerns:**
- Los **Extractores** (Stage 1) deben ser "tontos": observan el código y emiten hechos atómicos
- Si `VulnerableUncovered` es un `FactType`, ¿qué extractor lo emite?
  - ¿El extractor de SAST? → Necesitaría acceso a datos de cobertura
  - ¿El extractor de Coverage? → Necesitaría acceso a datos de SAST
  - ¿Un "Meta-Extractor" especial? → Acopla todos los extractores

**Inflexibilidad:**
- Cambiar la definición de "vulnerabilidad no cubierta" requiere modificar extractores
- Plugins de terceros deben implementar correlaciones complejas
- No se puede ajustar políticas sin re-ejecutar análisis completo

**Connascence Fuerte:**
- Connascence of Timing: extractores deben coordinarse
- Connascence of Identity: ¿cómo saben que están viendo el mismo código?

---

## Decision

**El IR contiene ÚNICAMENTE hechos atómicos observables.**

Las correlaciones multi-dominio son **derivadas por el Motor de Reglas** (Stage 2) y representadas como **`Finding`** (no `Fact`).

### Hechos Atómicos (Permitidos en IR)

```rust
pub enum FactType {
    // SAST - hechos observables
    TaintSource { .. },
    TaintSink { .. },
    Sanitization { .. },
    UnsafeCall { .. },
    Vulnerability { .. },
    
    // Quality - métricas observables
    Function { .. },
    Variable { .. },
    CodeSmell { .. },
    ComplexityViolation { .. },
    
    // SCA - hechos de dependencias
    Dependency { .. },
    DependencyVulnerability { .. },
    License { .. },
    
    // Coverage - métricas de testing
    UncoveredLine { .. },
    LowTestCoverage { .. },
    CoverageStats { .. },
    
    // ❌ ELIMINADO: VulnerableUncovered
    // ❌ ELIMINADO: SecurityTechnicalDebt
    // ❌ ELIMINADO: QualitySecurityCorrelation
}
```

### Correlaciones (Derivadas por Reglas DSL)

```dsl
// ANTES (v3.1 - INCORRECTO):
// Extractor emitía FactType::VulnerableUncovered

// DESPUÉS (v3.2 - CORRECTO):
// Motor de reglas deriva la correlación
forbid(
  rule: "CRITICAL_RISK_UNTESTED_VULN",
  severity: "blocker",
  description: "Critical vulnerability in untested code"
) on {
  // Join espacial entre TaintSink y UncoveredLine
  exists(Fact {
    type: "TaintSink",
    category: "SqlQuery",
    severity >= "High",
    file: $file,
    line: $line
  }) &&
  exists(Fact {
    type: "UncoveredLine",
    file: $file,
    line: $line,
    coverage < 0.1  // <10% cobertura
  })
}

// El resultado es un Finding, no un Fact
```

---

## Consequences

### ✅ Beneficios

1. **Extractores Simples y Desacoplados**
   - Cada extractor solo observa su dominio (SAST, SCA, Coverage)
   - No necesitan conocer otros extractores
   - Plugins de terceros más fáciles de implementar

2. **Flexibilidad de Políticas**
   - Cambiar qué cuenta como "crítico" solo requiere modificar reglas DSL
   - No necesitas re-ejecutar extractores (caros)
   - Puedes A/B test políticas en el mismo IR

3. **Testabilidad Mejorada**
   - Tests unitarios de extractores: solo verifican hechos atómicos
   - Tests de integración: verifican correlaciones en el motor
   - Separation of concerns facilita testing

4. **Extensibilidad**
   - Añadir nuevas correlaciones no requiere modificar IR Schema
   - Quality Gates pueden agregar Findings sin conocer extractores

### ❌ Desventajas (Mitigadas)

1. **Motor de Reglas Más Complejo**
   - **Mitigación:** Ya está diseñado con SpatialIndex y QueryPlanner para joins eficientes

2. **Performance de Joins**
   - **Mitigación:** Índices espaciales, por tipo, por FlowId optimizan correlaciones
   - **Benchmarks:** Join espacial de 100K facts en <2ms (objetivo alcanzable)

---

## Implementation

### 1. Actualizar IR Schema

```diff
pub enum FactType {
    // ... (hechos atómicos existentes)
    
-   VulnerableUncovered {
-       location: SourceLocation,
-       flow_id: FlowId,
-       cve_id: Option<CveId>,
-       coverage: CoveragePercentage,
-       risk_score: f32,
-   },
-   
-   SecurityTechnicalDebt {
-       issue_type: SecurityIssueType,
-       remediation_cost: f32,
-       priority: Priority,
-       sqale_rating: SqaleRating,
-   },
-   
-   QualitySecurityCorrelation {
-       scope: ProjectPath,
-       quality_score: f32,
-       security_score: f32,
-       combined_risk: RiskLevel,
-   },
}
```

### 2. Añadir Reglas DSL de Correlación

```rust
// hodei-rules/src/builtin/correlations.rs

pub const VULNERABLE_UNTESTED: &str = r#"
forbid(
  rule: "VULNERABLE_UNTESTED",
  severity: "blocker"
) on {
  exists(Fact { type: "TaintSink", severity >= "High", file: $file, line: $line }) &&
  exists(Fact { type: "UncoveredLine", file: $file, line: $line })
}
"#;

pub const SECURITY_DEBT_HIGH: &str = r#"
forbid(
  rule: "SECURITY_DEBT_HIGH",
  severity: "critical"
) on {
  count(Fact { type: "Vulnerability", severity: "Critical" }) > 5
}
"#;
```

### 3. Finding Representa Correlaciones

```rust
pub struct Finding {
    pub id: FindingId,
    pub rule_id: String,  // "VULNERABLE_UNTESTED"
    pub severity: Severity,
    pub location: SourceLocation,
    pub message: String,
    pub facts: Vec<FactId>,  // Referencias a los hechos correlacionados
    pub metadata: FindingMetadata,
}

impl Finding {
    // Los Findings son el resultado de evaluar reglas
    // Representan correlaciones, no hechos atómicos
}
```

---

## Alternatives Considered

### Alternativa 1: Mantener Meta-Hechos en IR
- **Rechazado:** Viola separation of concerns, acopla extractores

### Alternativa 2: Separar IR "Base" vs IR "Derived"
- **Rechazado:** Complejidad innecesaria; Findings ya son "IR derivado"

### Alternativa 3: Meta-Hechos Como Plugin Optional
- **Rechazado:** No resuelve el problema de acoplamiento

---

## Related

- [ARCHITECTURE-V3.2-FINAL.md](../ARCHITECTURE-V3.2-FINAL.md) § 2.5 (Separation of Concerns)
- [EPIC-02-ir-core.md](../epics/EPIC-02-ir-core.md) (implementación)
- [EPIC-04-rule-engine.md](../epics/EPIC-04-rule-engine.md) (motor de correlación)

---

## Approval

- [x] Architecture Review Board
- [x] Core Team Lead
- [x] Security Team Lead

**Approved:** 2025-01-XX  
**Effective:** Immediately (v3.2)