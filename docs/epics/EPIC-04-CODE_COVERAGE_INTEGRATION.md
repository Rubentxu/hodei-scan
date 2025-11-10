# Épica 4: Code Coverage Integration
## Integración Multi-Formato de Métricas de Cobertura

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 🚧 Planning
**Época:** Fase 2 (Meses 7-12)
**Prioridad:** 🟡 High

---

## 📋 Resumen Ejecutivo

Implementar el motor de integración de code coverage para hodei-scan, soporte para 5+ herramientas de coverage, tracking histórico y enforcement de thresholds. Esta funcionalidad es clave para medir quality gate de test coverage.

**Objetivos:**
- ✅ Soporte multi-formato: JaCoCo, Istanbul, Coverage.py, LLVM, gcov
- ✅ Branch coverage analysis
- ✅ Line coverage metrics
- ✅ Coverage threshold enforcement
- ✅ Historical tracking
- ✅ PR decoration con coverage deltas

**Métricas:** <15s parse de coverage reports, 100% format support, >95% accuracy

---

## 👥 Historias de Usuario

### US-15: Como developer, quiero ver coverage changes en Pull Requests

**Prioridad:** 🔴 Critical
**Story Points:** 8

```gherkin
Feature: PR Coverage Decoration
  Como developer revisando PR
  Quiero ver coverage changes directamente en PR
  Para evaluar impacto de cambios en test coverage

  Scenario: Coverage decrease en PR
    Given proyecto con 80% overall coverage
    When se abre PR que reduce coverage a 75%
    Then hodei-scan debería reportar coverage drop
    And debería marcar PR como "Coverage gate failed"
    And debería sugerir agregar tests

  Scenario: Coverage increase en PR
    Given proyecto con 80% overall coverage
    When se abre PR que aumenta coverage a 85%
    Then hodei-scan debería reportar improvement
    And debería mostrar celebration message
    And debería mark PR como passing quality gate
```

**Tareas:**

1. **TASK-04-01: Implementar Coverage Parser multi-formato** (3 días)
2. **TASK-04-02: Implementar Coverage Delta Calculator** (3 días)
3. **TASK-04-03: Implementar Threshold Enforcer** (2 días)
4. **TASK-04-04: Implementar Historical Tracker** (3 días)

**Tests:**

```rust
#[test]
fn test_coverage_delta_calculation() {
    let before = CoverageSummary { line_coverage: 80, branch_coverage: 75 };
    let after = CoverageSummary { line_coverage: 75, branch_coverage: 70 };
    let delta = CoverageDelta::calculate(&before, &after);

    assert_eq!(delta.line_coverage_delta, -5);
    assert!(delta.is_decrease);
}
```

### US-16: Como QA, quiero enforcement de coverage minimums

**Prioridad:** 🔴 Critical
**Story Points:** 5

```gherkin
Feature: Coverage Threshold Enforcement
  Como QA configurando quality standards
  Quiero enforce minimum coverage thresholds
  Para asegurar quality mínimo en code

  Scenario: Configurar threshold
    Given proyecto con quality gate de 80% coverage
    When CI run con coverage < 80%
    Then hodei-scan debería fallar el build
    And debería reportar threshold violation

  Scenario: Coverage por directorio
    Given threshold diferentes por directorio
    When se ejecuta coverage
    Then debería aplicar threshold específico por path
```

**Tareas:**

1. **TASK-04-05: Implementar Coverage Thresholds Configuration** (2 días)
2. **TASK-04-06: Implementar Gate Failure Handler** (2 días)

---

## 🏗️ Arquitectura

```
┌─────────────────────────────────────┐
│     Code Coverage Engine            │
├─────────────────────────────────────┤
│  ┌──────────┐ ┌──────────────────┐ │
│  │ Parser   │ │ Threshold        │ │
│  │ Registry │ │ Enforcer         │ │
│  └──────────┘ └──────────────────┘ │
│        │              │            │
│  ┌─────▼──────┬───────▼──────┐    │
│  │ Format    │ Historical   │    │
│  │ Detectors │ Tracker      │    │
│  └───────────┴──────────────┘    │
│              │                   │
│  ┌───────────▼───────────────┐   │
│  │ Integration (CI/CD, PR)   │   │
│  └───────────────────────────┘   │
└─────────────────────────────────────┘
```

**Dependencias:**
```toml
[dependencies]
quick-xml = "0.31"  # JaCoCo XML
regex = "1.0"       # Pattern matching
```

---

## 🔄 Criterios de Done

- [ ] ✅ 5+ coverage formats soportados
- [ ] ✅ <15s parse time
- [ ] ✅ Historical tracking funcional
- [ ] ✅ Threshold enforcement working
- [ ] ✅ PR decoration completo
- [ ] ✅ 100% tests en verde

**Total Story Points:** 26 | **Duración:** 6 semanas
