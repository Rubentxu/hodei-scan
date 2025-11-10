# hodei-scan v3.1: Índice de Épicas
## Backlog Organizado por Épica con Enfoque TDD, Hexagonal y SOLID

**Versión:** 3.1.0  
**Fecha:** 2025-01-XX  
**Metodología:** TDD + Arquitectura Hexagonal + SOLID + Connascence  
**Convención de Commits:** Conventional Commits  

---

## 📋 Estructura del Backlog

Cada épica está en su propio archivo con todas sus historias de usuario, siguiendo:

- **TDD:** Red → Green → Refactor en cada historia
- **Hexagonal:** Separación por capas (Domain, Application, Infrastructure)
- **SOLID:** Principios aplicados explícitamente
- **Connascence:** Minimización documentada (CoP → CoN, CoM → CoT)
- **Conventional Commits:** Template en cada historia

---

## 🎯 Épicas del Proyecto

### Fase 1: Foundation (Q1 2025 - Meses 1-3)

| # | Épica | Archivo | Story Points | Estado | Sprint |
|---|-------|---------|--------------|--------|--------|
| 01 | Setup y Fundamentos | [`EPIC-01-setup.md`](./EPIC-01-setup.md) | 40 SP | 📝 Ready | Sprint 1 |
| 02 | IR Core - Tipos Type-Safe | [`EPIC-02-ir-core.md`](./EPIC-02-ir-core.md) | 55 SP | 📝 Ready | Sprint 1-2 |
| 03 | IR Schema - FactTypes | [`EPIC-03-ir-schema.md`](./EPIC-03-ir-schema.md) | 89 SP | 📝 Ready | Sprint 2-3 |
| 04 | Serialización Zero-Copy | [`EPIC-04-zero-copy.md`](./EPIC-04-zero-copy.md) | 34 SP | 📝 Ready | Sprint 3-4 |
| 05 | IndexedFactStore | [`EPIC-05-indexed-store.md`](./EPIC-05-indexed-store.md) | 55 SP | 📝 Ready | Sprint 4-5 |
| 06 | Query Planner | [`EPIC-06-query-planner.md`](./EPIC-06-query-planner.md) | 34 SP | 📝 Ready | Sprint 5-6 |

### Fase 2: Core Engine (Q2 2025 - Meses 4-6)

| # | Épica | Archivo | Story Points | Estado | Sprint |
|---|-------|---------|--------------|--------|--------|
| 07 | RuleEngine - Motor Evaluación | [`EPIC-07-rule-engine.md`](./EPIC-07-rule-engine.md) | 55 SP | 📝 Ready | Sprint 6-7 |
| 08 | DSL Parser con PEG | [`EPIC-08-dsl-parser.md`](./EPIC-08-dsl-parser.md) | 55 SP | 📝 Ready | Sprint 7-8 |
| 09 | Quality Gates | [`EPIC-09-quality-gates.md`](./EPIC-09-quality-gates.md) | 34 SP | 📝 Ready | Sprint 8-9 |
| 10 | Sistema de Plugins | [`EPIC-10-plugins.md`](./EPIC-10-plugins.md) | 55 SP | 📝 Ready | Sprint 9-10 |

### Fase 3: Extractors (Q2-Q3 2025 - Meses 5-8)

| # | Épica | Archivo | Story Points | Estado | Sprint |
|---|-------|---------|--------------|--------|--------|
| 11 | Extractores Nivel 1 (AST) | [`EPIC-11-extractors-ast.md`](./EPIC-11-extractors-ast.md) | 89 SP | 📝 Ready | Sprint 10-12 |
| 12 | Extractores Nivel 2 (SAST) | [`EPIC-12-extractors-sast.md`](./EPIC-12-extractors-sast.md) | 144 SP | 📝 Ready | Sprint 13-16 |
| 13 | Extractores SCA | [`EPIC-13-extractors-sca.md`](./EPIC-13-extractors-sca.md) | 89 SP | 📝 Ready | Sprint 17-19 |

### Fase 4: Integration & Security (Q3 2025 - Meses 7-9)

| # | Épica | Archivo | Story Points | Estado | Sprint |
|---|-------|---------|--------------|--------|--------|
| 14 | CLI y Configuración | [`EPIC-14-cli.md`](./EPIC-14-cli.md) | 55 SP | 📝 Ready | Sprint 19-20 |
| 15 | Seguridad y Hardening | [`EPIC-15-security.md`](./EPIC-15-security.md) | 55 SP | 📝 Ready | Sprint 20-21 |
| 16 | Performance Optimization | [`EPIC-16-performance.md`](./EPIC-16-performance.md) | 34 SP | 📝 Ready | Sprint 21-22 |

### Fase 5: Enterprise (Q4 2025 - Meses 10-12)

| # | Épica | Archivo | Story Points | Estado | Sprint |
|---|-------|---------|--------------|--------|--------|
| 17 | CI/CD Integraciones | [`EPIC-17-cicd-integration.md`](./EPIC-17-cicd-integration.md) | 55 SP | 🔜 Planned | Sprint 23-24 |
| 18 | Web Dashboard MVP | [`EPIC-18-web-dashboard.md`](./EPIC-18-web-dashboard.md) | 89 SP | 🔜 Planned | Sprint 25-27 |
| 19 | Multi-Project Support | [`EPIC-19-multi-project.md`](./EPIC-19-multi-project.md) | 55 SP | 🔜 Planned | Sprint 28-29 |
| 20 | Enterprise Features | [`EPIC-20-enterprise.md`](./EPIC-20-enterprise.md) | 89 SP | 🔜 Planned | Sprint 30-32 |

---

## 📊 Resumen Cuantitativo

### Por Fase

| Fase | Épicas | Story Points | Duración | Prioridad |
|------|--------|--------------|----------|-----------|
| **Fase 1: Foundation** | 6 | 307 SP | 3 meses | 🔴 Critical |
| **Fase 2: Core Engine** | 4 | 199 SP | 2 meses | 🔴 Critical |
| **Fase 3: Extractors** | 3 | 322 SP | 3 meses | 🟠 High |
| **Fase 4: Integration** | 3 | 144 SP | 2 meses | 🟠 High |
| **Fase 5: Enterprise** | 4 | 288 SP | 3 meses | 🟡 Medium |
| **TOTAL** | **20** | **1260 SP** | **13 meses** | - |

### Velocity Esperada

- **Sprint Duration:** 2 semanas
- **Team Size:** 3-4 desarrolladores
- **Velocity Target:** 40-50 SP/sprint
- **Total Sprints:** ~32 sprints (~16 meses con buffer)

---

## 🎯 Definición de Historia de Usuario

Cada historia sigue este formato estándar:

```markdown
### US-XX.YY: [Título Descriptivo]

**Como:** [Rol]
**Quiero:** [Funcionalidad]
**Para:** [Beneficio de negocio]

**Prioridad:** [Critical/High/Medium/Low]
**Estimación:** X Story Points
**Sprint:** Sprint Y

**Criterios de Aceptación:**
- [ ] Criterio 1 (medible)
- [ ] Criterio 2 (testeable)
- [ ] Criterio 3 (verificable)

**Principios Aplicados:**
- **SOLID:** [SRP/OCP/LSP/ISP/DIP específicos]
- **Connascence:** [Tipo minimizado: CoP→CoN, CoM→CoT, etc.]
- **Hexagonal:** [Domain/Application/Infrastructure + Puerto/Adaptador]

**Dependencias:**
- Depende de: US-XX.YY
- Bloquea a: US-ZZ.WW

**Tareas Técnicas (TDD):**

#### 1. 🔴 RED: Tests que fallan
```rust
#[test]
fn test_name_describes_behavior() {
    // Arrange
    let sut = SystemUnderTest::new();
    
    // Act
    let result = sut.do_something();
    
    // Assert
    assert!(result.is_ok());
}
```

#### 2. 🟢 GREEN: Implementación mínima
```rust
pub struct SystemUnderTest;

impl SystemUnderTest {
    pub fn new() -> Self {
        Self
    }
    
    pub fn do_something(&self) -> Result<(), Error> {
        Ok(())
    }
}
```

#### 3. 🔵 REFACTOR: Optimización
```rust
// Mejoras de diseño, extracción de métodos, etc.
```

**Tests de Regresión:**
```rust
#[test]
fn regression_test_for_issue_xyz() {
    // Test específico para prevenir regresiones
}
```

**Definición de Done:**
- [ ] Tests unitarios pasan (coverage >80%)
- [ ] Tests de integración pasan (si aplica)
- [ ] Documentación actualizada (rustdoc)
- [ ] Code review aprobado (2+ approvals)
- [ ] CI/CD pipeline verde
- [ ] Conventional commit realizado
- [ ] CHANGELOG.md actualizado

**Commit Message Template:**
```
<type>(scope): <description>

[optional body]

[optional footer]
```

**Tipos:**
- `feat`: Nueva funcionalidad
- `fix`: Corrección de bug
- `refactor`: Refactorización sin cambio de comportamiento
- `test`: Añadir/modificar tests
- `docs`: Cambios en documentación
- `chore`: Tareas de mantenimiento

**Ejemplo:**
```
feat(ir): add Confidence newtype with validation

Implements Confidence struct that enforces [0.0, 1.0] range at compile-time.
Uses thiserror for ergonomic error handling.

Reduces Connascence of Meaning (CoM) to Connascence of Type (CoT).

Closes #123
```
```

---

## 🏗️ Arquitectura Hexagonal por Capa

### Domain (Core)

**Épicas:** 02, 03, 04
**Crates:** `hodei-ir`

Contiene:
- Value Objects (Confidence, ProjectPath, LineNumber, FlowId)
- Entities (Fact, FactType)
- Aggregates (IntermediateRepresentation)
- Domain Services (IRValidator)

**Principios:**
- Sin dependencias externas (solo std, thiserror)
- Lógica de negocio pura
- Tipos inmutables por defecto
- Estados inválidos irrepresentables

### Application (Use Cases)

**Épicas:** 05, 06, 07, 08, 09
**Crates:** `hodei-engine`, `hodei-dsl`

Contiene:
- Puertos (traits): `Extractor`, `FactTypePlugin`, `MetricAggregator`
- Use Cases: `EvaluateRules`, `BuildIndex`, `ParseDSL`
- Servicios de aplicación: `RuleEngine`, `QueryPlanner`

**Principios:**
- Depende solo de Domain
- Define interfaces (ports)
- Sin detalles de infraestructura

### Infrastructure (Adapters)

**Épicas:** 10, 11, 12, 13, 14
**Crates:** `hodei-extractors`, `hodei-cli`

Contiene:
- Adaptadores de entrada: CLI, gRPC (futuro)
- Adaptadores de salida: Tree-sitter, Oxc, Semgrep
- Implementaciones de puertos
- Configuración, logging, serialización

**Principios:**
- Implementa interfaces de Application
- Puede tener dependencias externas
- Fácilmente reemplazable

---

## 📋 Workflow de Desarrollo

### Por Historia de Usuario

1. **Planning:**
   - Refinar criterios de aceptación
   - Estimar con planning poker
   - Identificar dependencias

2. **RED (Test First):**
   - Escribir test unitario que falla
   - Definir interfaz pública
   - Commit: `test(scope): add failing test for feature X`

3. **GREEN (Implementación):**
   - Código mínimo para pasar test
   - Sin optimización prematura
   - Commit: `feat(scope): implement feature X`

4. **REFACTOR (Mejora):**
   - Eliminar duplicación
   - Aplicar SOLID
   - Minimizar connascence
   - Commit: `refactor(scope): improve design of X`

5. **Review:**
   - Code review con checklist
   - Verificar principios aplicados
   - Aprobar y mergear

6. **Done:**
   - Todos los checks de DoD pasan
   - Historia movida a "Done"

---

## 🧪 Testing Strategy

### Tipos de Tests por Épica

| Épica | Unit Tests | Integration Tests | E2E Tests | Benchmarks |
|-------|------------|-------------------|-----------|------------|
| 01-04 | ✅ 100% | - | - | ✅ Performance |
| 05-10 | ✅ 100% | ✅ Cross-crate | - | ✅ Query time |
| 11-13 | ✅ 80%+ | ✅ Real files | ✅ CLI | ✅ Throughput |
| 14-16 | ✅ 80%+ | ✅ Full pipeline | ✅ Full scan | ✅ Latency |

### Coverage Targets

- **Domain (hodei-ir):** 95%+ (crítico)
- **Application (hodei-engine, hodei-dsl):** 85%+
- **Infrastructure (hodei-extractors):** 70%+
- **CLI (hodei-cli):** 60%+

---

## 🚀 Getting Started

### Prioridad de Lectura

1. **Arquitecto/Tech Lead:**
   - Leer todas las épicas en orden
   - Validar estimaciones
   - Ajustar dependencias

2. **Desarrollador Core:**
   - Empezar con EPIC-01, EPIC-02
   - Implementar tipos fundamentales
   - Establecer patrones

3. **Desarrollador Feature:**
   - Leer EPIC-11, EPIC-12 (extractores)
   - Implementar adaptadores
   - Seguir patrones establecidos

4. **QA Engineer:**
   - Revisar criterios de aceptación
   - Diseñar test matrix
   - Automatizar verificación

---

## 📞 Contacto y Mantenimiento

**Mantenido por:** hodei-scan Architecture Team  
**Última actualización:** 2025-01-XX  
**Versión del backlog:** 3.1.0  

**Para cambios en el backlog:**
1. Crear issue con template "Epic Proposal"
2. Discutir en architecture meeting
3. Actualizar este índice + épica correspondiente
4. Commit: `docs(epic): add/update EPIC-XX`

**Convención de nombres de archivos:**
- `EPIC-{NN}-{slug}.md`
- NN: número de dos dígitos (01-20)
- slug: identificador kebab-case

**Versionado:**
- Major: Cambio en estructura de épicas
- Minor: Nueva épica o reorganización
- Patch: Ajustes en historias existentes

---

## 📚 Referencias

- [ARCHITECTURE-V3.1-FINAL.md](../ARCHITECTURE-V3.1-FINAL.md) - Especificación técnica
- [V3.1-EXECUTIVE-SUMMARY.md](../V3.1-EXECUTIVE-SUMMARY.md) - Resumen ejecutivo
- [TDD_METHODOLOGY.md](../TDD_METHODOLOGY.md) - Metodología TDD del proyecto

---

**Versión:** 3.1.0  
**Estado:** 📝 Ready for Sprint Planning  
**Total Story Points:** 1260 SP  
**Estimated Duration:** 13-16 meses  
**Team Velocity:** 40-50 SP/sprint