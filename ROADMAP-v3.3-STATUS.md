# 🚀 hodei-scan v3.3 - Estado Actual y Roadmap Completo

**Fecha:** 2025-11-11
**Versión:** 3.2.1 → 3.3.0
**Estado del Proyecto:** En Desarrollo Activo

---

## 📊 Resumen Ejecutivo

Basándome en el análisis completo de SPEC-3.3.md, el proyecto hodei-scan v3.3 está estructurado en **5 épicas mayores** que transformarán la plataforma de un motor de análisis monolítico a una **plataforma de gobernanza extensible**:

| Épica | Estado Actual | Prioridad | Sprint | Estimación |
|-------|---------------|-----------|---------|------------|
| **EPIC-10: Extractor Ecosystem** | 🟡 En Progreso (60%) | Critical | 1-2 | 3 sprints |
| **EPIC-11: IR Schema Evolution** | 🔴 Pendiente | Critical | 3 | 1 sprint |
| **EPIC-12: Declarative Pattern Engine** | 🔴 Pendiente | High | 4-5 | 2 sprints |
| **EPIC-13: Backend de Gobernanza** | 🔴 Pendiente | High | 6-7 | 3 sprints |
| **EPIC-14: Developer Experience Tools** | 🔴 Pendiente | Medium | 8 | 2 sprints |
| **EPIC-15: Taint Analysis Engine** | 🔴 Pendiente | Medium | 9-10 | 4 sprints |

---

## 📋 Estado Detallado por Épica

### ✅ **EPIC-06: Rule Engine** (COMPLETADO - Sprint Actual)
**Estado:** ✅ 100% Completado
- [x] RuleEngine con timeout protection (crossbeam)
- [x] PatternMatcher con spatial joins
- [x] ExprEvaluator completo (all binary operators + built-ins)
- [x] FindingBuilder con template interpolation
- [x] 8 comprehensive tests
- [x] Performance benchmarks
- [x] Documentación completa

---

### 🟡 **EPIC-10: Extractor Ecosystem - Multi-Process Architecture**
**Estado:** 🟡 En Progreso (40% - Iniciando)
**Objetivo:** Transformar extractores acoplados en proceso → procesos independientes

#### US-10.01: ExtractorOrchestrator con Cap'n Proto 🟡 EN PROCESO
**Progress:** 60%
- [x] Crear estructura ExtractorOrchestrator
- [x] Definir protocol.rs (ExtractorRequest/Response)
- [x] Implementar error handling
- [x] Método execute_all() con concurrency limits
- [x] Timeout handling con crossbeam
- [ ] **EN PROGRESO:** Fix compilation errors
- [ ] Implementar Cap'n Proto schema (reemplazar JSON)
- [ ] Test suite completo con mock extractors
- [ ] Resource limits y graceful shutdown
- [ ] Benchmarks de performance

#### US-10.02: SARIF Adapter Extractor
**Estado:** 🔴 Pendiente
**Estimación:** 1 sprint
- [ ] Crear `sarif-to-hodei` extractor
- [ ] Parse SARIF JSON → IR transformation
- [ ] Soporte para múltiples herramientas SARIF
- [ ] Test con Semgrep SARIF, CodeQL
- [ ] Batch processing para SARIFs grandes

#### US-10.03: Ruff Adapter
**Estado:** 🔴 Pendiente
**Estimación:** 1 sprint
- [ ] Crear `ruff-to-hodei` adapter
- [ ] Ruff JSON output → IR mapping
- [ ] Mapeo Ruff diagnostics → FactType::CodeSmell
- [ ] Soporte multi-file en paralelo
- [ ] Performance < 2x Ruff time

#### US-10.04: Sistema Timeouts y Resource Limits
**Estado:** 🔴 Pendiente
**Estimación:** 0.5 sprints
- [ ] Timeout por extractor (configurable)
- [ ] Kill process en timeout
- [ ] Memory limit por extractor
- [ ] CPU limit (nice/ionice)
- [ ] Concurrent extractor limit

#### US-10.05: Configuración hodei.toml
**Estado:** 🔴 Pendiente
**Estimación:** 0.5 sprints
- [ ] Toml format documentado
- [ ] Validación de configuración
- [ ] Soporte múltiples extractors
- [ ] Override per-project settings
- [ ] Config inheritance

---

### 🔴 **EPIC-11: IR Schema Evolution - Custom FactTypes**
**Estado:** 🔴 Pendiente (Bloqueado por EPIC-10)
**Objetivo:** Hacer IR extensible con tipos Custom sin recompilar

#### US-11.01: Extend FactType enum con variante Custom
**Estimación:** 1 sprint
**TDD Approach:**
- [ ] Red: Test que falla con Custom FactType
- [ ] Green: Implementar variante Custom en hodei_ir
- [ ] Refactor: Optimizar serialización

**Implementation:**
```rust
pub enum FactType {
    // ... variantes core
    Custom {
        discriminant: String, // ej. "terraform::aws::insecure_s3_bucket"
        data: HashMap<String, FactValue>,
    },
}
```

#### US-11.02: Plugin Schema Registry
**Estimación:** 1 sprint
- [ ] Crear PluginRegistry struct
- [ ] Cargar esquemas desde config
- [ ] Validar Custom facts contra schema
- [ ] Versioning backward-compatible

#### US-11.03: IR Serialization con Cap'n Proto Custom Support
**Estimación:** 0.5 sprints
- [ ] Actualizar Cap'n Proto schema para Custom
- [ ] Implementar serialización/deserialización
- [ ] Tests de round-trip

---

### 🔴 **EPIC-12: Declarative Pattern Engine - Tree-sitter + YAML**
**Estado:** 🔴 Pendiente (Bloqueado por EPIC-11)
**Objetivo:** Democratizar creación de reglas sin programar

#### US-12.01: Tree-sitter Pattern Engine Core
**Estimación:** 1 sprint
- [ ] Integrar tree-sitter crate
- [ ] Implementar pattern matcher engine
- [ ] Multi-language support (Java, Python, Rust)
- [ ] Performance optimization

#### US-12.02: YAML Rule Format
**Estimación:** 0.5 sprints
- [ ] Diseñar formato YAML para reglas
- [ ] Parser YAML → AST
- [ ] Validación de reglas
- [ ] Documentación formato

**Example:**
```yaml
id: JAVA-EMPTY-CATCH-BLOCK
language: java
message: "Bloque catch vacío detectado"
severity: Major
pattern: |
  try { ... } catch ($EXCEPTION e) {
    // Comentario opcional
  }
```

#### US-12.03: Rule Execution Engine
**Estimación:** 0.5 sprints
- [ ] Ejecutor de reglas YAML
- [ ] Generador de Facts
- [ ] Aggregation de múltiples patterns
- [ ] Error handling

#### US-12.04: YAML → IR Integration
**Estimación:** 0.5 sprints
- [ ] YAML extractor como proceso
- [ ] Integración con ExtractorOrchestrator
- [ ] hodei.toml support
- [ ] E2E tests

---

### 🔴 **EPIC-13: Backend de Gobernanza - hodei-server** ✅ CREADO
**Estado:** 🔴 Pendiente (Fase 3 según SPEC-3.3)
**Objetivo:** Plataforma stateful para análisis histórico y tendencias
**Documento:** `/home/rubentxu/Proyectos/rust/hodei-scan/docs/epics/EPIC-13-backend-governance.md`

#### US-13.01: hodei-server Architecture
**Estimación:** 1 sprint
- [ ] Diseñar arquitectura hodei-server
- [ ] Choose database (TimescaleDB vs ClickHouse)
- [ ] API REST/gRPC design
- [ ] Docker setup

#### US-13.02: Historical Storage APIs
**Estimación:** 1 sprint
- [ ] POST /api/projects/{id}/publish
- [ ] GET /api/projects/{id}/history
- [ ] Storage optimization
- [ ] Data retention policies

#### US-13.03: Diff Analysis APIs
**Estimación:** 0.5 sprints
- [ ] GET /api/projects/{id}/diff?base=main&head=feature
- [ ] NEW issues detection
- [ ] Baselines support
- [ ] Won't fix acceptance

#### US-13.04: Dashboard Frontend
**Estimación:** 1.5 sprints
- [ ] React/Vue dashboard
- [ ] Trends visualization
- [ ] Security metrics
- [ ] Quality gates

---

### 🔴 **EPIC-14: Developer Experience Tools** ✅ CREADO
**Estado:** 🔴 Pendiente (Fase 3 según SPEC-3.3)
**Objetivo:** Facilitar creación y testing de reglas
**Documento:** `/home/rubentxu/Proyectos/rust/hodei-scan/docs/epics/EPIC-14-developer-experience.md`

#### US-14.01: hodei-dsl Language Server (LSP)
**Estimación:** 1 sprint
- [ ] Implementar LSP server
- [ ] Autocompletado FactTypes
- [ ] Syntax validation
- [ ] Hover documentation
- [ ] VS Code extension

#### US-14.02: Rule Testing Framework
**Estimación:** 0.5 sprints
- [ ] hodei-scan test-rule command
- [ ] YAML test case format
- [ ] Assertion system
- [ ] CI integration

#### US-14.03: IR Debug Tools
**Estimación:** 0.5 sprints
- [ ] hodei-scan ir-dump command
- [ ] Cap'n Proto → JSON conversion
- [ ] Interactive explorer
- [ ] Filtering capabilities

---

### 🔴 **EPIC-15: Taint Analysis Engine** ✅ CREADO
**Estado:** 🔴 Pendiente (Nivel 3 - Extractores Profundos)
**Objetivo:** Análisis de flujo de datos para vulnerabilidades complejas
**Documento:** `/home/rubentxu/Proyectos/rust/hodei-scan/docs/epics/EPIC-15-taint-analysis-engine.md`

#### US-15.01: Taint Engine Core Implementation
**Estimación:** 2 sprints
- [ ] Control Flow Graph builder
- [ ] Data flow analysis
- [ ] Taint propagation engine
- [ ] Source/Sink detection

#### US-15.02: Java Taint Extractor
**Estimación:** 1 sprint
- [ ] Java AST parser integration
- [ ] SQL Injection detection
- [ ] XSS detection
- [ ] PreparedStatement sanitization

#### US-15.03: Python Taint Extractor
**Estimación:** 0.5 sprints
- [ ] Python AST parser
- [ ] Code injection detection
- [ ] Command injection detection

#### US-15.04: Advanced Taint Rules
**Estimación:** 0.5 sprints
- [ ] Path Traversal
- [ ] SSRF detection
- [ ] Deserialization vulnerabilities
- [ ] XXE detection

---

## 🗓️ Roadmap Detallado - Siguientes 12 Meses

### **Sprint 1-2 (Diciembre 2025): EPIC-10 Foundation**
**Prioridad:** Completar ExtractorOrchestrator base
1. **Semana 1:**
   - [ ] Fix ExtractorOrchestrator compilation errors
   - [ ] Implement Cap'n Proto schema
   - [ ] Basic timeout/resource limits

2. **Semana 2:**
   - [ ] Mock extractor tests
   - [ ] Performance benchmarks
   - [ ] Documentación completa

3. **Semana 3-4:**
   - [ ] SARIF Adapter (US-10.02)
   - [ ] Ruff Adapter (US-10.03)
   - [ ] hodei.toml config (US-10.05)

**Deliverables:**
- ✅ ExtractorOrchestrator funcional
- ✅ 2 adaptadores (SARIF, Ruff)
- ✅ Benchmarks y documentación
- ✅ Commit: `feat(extractor): implement core orchestrator with Cap'n Proto`

---

### **Sprint 3 (Enero 2026): EPIC-11 IR Schema Evolution**
**Prioridad:** Hacer IR extensible
1. **Semana 1:**
   - [ ] Custom FactType variant (US-11.01)
   - [ ] Tests TDD: Red → Green → Refactor

2. **Semana 2:**
   - [ ] Plugin Schema Registry (US-11.02)
   - [ ] Custom validation
   - [ ] IR serialization updates

**Deliverables:**
- ✅ Custom FactType support
- ✅ Plugin registration system
- ✅ Backward-compatible schema evolution

---

### **Sprint 4-5 (Febrero-Marzo 2026): EPIC-12 Pattern Engine**
**Prioridad:** Democratizar reglas
1. **Semana 1-2:**
   - [ ] Tree-sitter integration (US-12.01)
   - [ ] YAML rule format (US-12.02)

2. **Semana 3-4:**
   - [ ] Rule execution engine (US-12.03)
   - [ ] IR integration (US-12.04)

**Deliverables:**
- ✅ Declarative pattern engine
- ✅ YAML rule support
- ✅ Tree-sitter multi-language

---

### **Sprint 6-7 (Abril-Mayo 2026): EPIC-13 Backend Governance**
**Prioridad:** Plataforma stateful
1. **Semana 1-2:**
   - [ ] hodei-server architecture (US-13.01)
   - [ ] Database setup y API design

2. **Semana 3-4:**
   - [ ] Historical storage (US-13.02)
   - [ ] Diff analysis APIs (US-13.03)

3. **Semana 5-6:**
   - [ ] Dashboard frontend (US-13.04)
   - [ ] E2E integration tests

**Deliverables:**
- ✅ hodei-server funcional
- ✅ Historical analysis APIs
- ✅ Web dashboard
- ✅ CI/CD integration

---

### **Sprint 8 (Junio 2026): EPIC-14 Developer Experience**
**Prioridad:** Facilitar adopción
1. **Semana 1:**
   - [ ] LSP implementation (US-14.01)
   - [ ] VS Code extension

2. **Semana 2:**
   - [ ] Rule testing framework (US-14.02)
   - [ ] IR debug tools (US-14.03)

**Deliverables:**
- ✅ Language Server Protocol
- ✅ Testing framework
- ✅ Debug tools
- ✅ Community onboarding

---

### **Sprint 9-10 (Julio-Agosto 2026): EPIC-15 Taint Analysis Engine**
**Prioridad:** Vulnerabilidades profundas
1. **Semana 1-2:**
   - [ ] Taint engine core (US-15.01)
   - [ ] CFG builder y data flow analysis

2. **Semana 3-4:**
   - [ ] Java taint extractor (US-15.02)
   - [ ] SQL Injection detection

3. **Semana 5-6:**
   - [ ] Python taint extractor (US-15.03)
   - [ ] Advanced taint rules (US-15.04)

**Deliverables:**
- ✅ hodei-taint-engine funcional
- ✅ Java/Python extractors
- ✅ Inter-procedural analysis
- ✅ Performance benchmarks

---

## 🎯 **Prioridades Inmediatas (Próximas 2 Semanas)**

### **Tarea Crítica 1: Completar ExtractorOrchestrator**
```bash
# TDD - Red (Test que falla)
cargo test -p hodei-engine extractor::tests

# Green (Implementación mínima)
# Refactor (Optimización)
```

### **Tarea Crítica 2: Implementar Cap'n Proto Schema**
- Reemplazar JSON con Cap'n Proto para mejor performance
- Mantener backward compatibility

### **Tarea Crítica 3: SARIF Adapter MVP**
- Crear `sarif-to-hodei` extractor
- Test con herramientas reales

---

## 🔍 **Análisis de Riesgos**

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| Cap'n Proto learning curve | Medio | Alto | Start with JSON, migrate later |
| Tree-sitter performance | Alto | Medio | Benchmark-driven optimization |
| Backend database choice | Alto | Bajo | POC with both TimescaleDB y ClickHouse |
| Community adoption | Alto | Medio | Focus on DX desde el inicio |

---

## 📈 **Métricas de Éxito (v3.3 Final)**

- **Cobertura:** 10+ herramientas integradas (Ruff, ESLint, SARIF)
- **Performance:** <20% overhead vs herramientas nativas
- **Extensibilidad:** Nuevo extractor sin recompilar core
- **Usabilidad:** <5 min para crear nueva regla YAML
- **Adopción:** 100+ reglas YAML en 3 meses

---

## 🚀 **Próximo Paso Inmediato**

**ACCIÓN RECOMENDADA:** Continuar con **EPIC-10: ExtractorOrchestrator**

1. **Ahora:** Fix compilation errors
2. **Esta semana:** Implement Cap'n Proto + tests
3. **Próxima semana:** SARIF Adapter

**¿Continuamos con la implementación del ExtractorOrchestrator?**
