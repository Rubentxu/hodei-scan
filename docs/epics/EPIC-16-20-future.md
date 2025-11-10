# Épicas Adicionales (v3.3+)

Este documento contiene las épicas restantes planificadas para versiones futuras de Hodei Scan.

---

## EPIC-16: Incremental Analysis & Caching

**Versión**: v3.3  
**Prioridad**: Medium

### Objetivo
Cachear resultados parciales para análisis incremental (solo re-analizar archivos modificados).

### Features
- Cache de IRs parciales por archivo
- Detección de cambios (git diff)
- Merge de IRs parciales
- Invalidación de cache inteligente

### Métricas
- 10x speedup en análisis incremental
- Cache hit rate >80%

---

## EPIC-17: Interactive Mode & REPL

**Versión**: v3.3  
**Prioridad**: Low

### Objetivo
REPL interactivo para explorar IRs y probar reglas.

### Features
```rust
hodei> load ir app.ir
✅ Loaded 12,543 facts

hodei> facts by_type TaintSink
Found 23 TaintSink facts

hodei> eval rule "..."
✅ Rule matched 5 findings

hodei> show finding 0
Rule: VulnerableUncovered
Location: src/auth.rs:145
Message: Vulnerable code has no tests
```

---

## EPIC-18: Web UI & Dashboard

**Versión**: v3.4  
**Prioridad**: Medium

### Objetivo
Web UI para visualizar findings, métricas, trends.

### Features
- Dashboard con charts (Chart.js)
- Filtrado y búsqueda de findings
- Comparación de scans
- Export reports (PDF, HTML)

### Stack
- Backend: Axum (Rust web framework)
- Frontend: Svelte + TailwindCSS
- Database: SQLite

---

## EPIC-19: Language Server Protocol (LSP)

**Versión**: v3.5  
**Prioridad**: Low

### Objetivo
LSP server para autocompletado y diagnósticos en tiempo real al escribir reglas.

### Features
- Autocompletado de FactTypes, campos
- Inline diagnostics (errores de tipo)
- Go-to-definition
- Hover documentation

### Integration
- VSCode extension
- Zed editor support
- Neovim integration

---

## EPIC-20: Advanced Correlation Rules

**Versión**: v3.5  
**Prioridad**: Medium

### Objetivo
Reglas avanzadas con agregaciones, métricas temporales, y machine learning.

### Features

#### Aggregations
```hodei
rule HighVulnerabilityDensity {
    match {
        vulns: Vulnerability[]
        
        where count(vulns) / total_loc > 0.01  // >1% de líneas vulnerables
    }
    
    emit Finding {
        message: "High vulnerability density: {count(vulns)} vulns in {total_loc} LOC"
        confidence: High
    }
}
```

#### Temporal Queries
```hodei
rule VulnerabilityTrend {
    match {
        current_vulns: Vulnerability[] in current_scan
        past_vulns: Vulnerability[] in baseline_scan
        
        where count(current_vulns) > count(past_vulns) * 1.5
    }
    
    emit Finding {
        message: "Vulnerability count increased by 50%"
        confidence: High
    }
}
```

#### ML-based Patterns
```hodei
rule SuspiciousPattern {
    match {
        code_block: CodeBlock
        
        where ml_classifier(code_block.ast, "xss_vulnerability") > 0.8
    }
    
    emit Finding {
        message: "ML classifier detected potential XSS"
        confidence: Medium
    }
}
```

---

## Roadmap Summary

| Epic | Version | Priority | Status |
|------|---------|----------|--------|
| EPIC-01 | v3.2 | Critical | ✅ Done |
| EPIC-02 | v3.2 | Critical | 📝 Draft |
| EPIC-03 | v3.2 | Critical | 📝 Draft |
| EPIC-04 | v3.2 | Critical | 📝 Draft |
| EPIC-05 | v3.2 | Critical | 📝 Draft |
| EPIC-06 | v3.2 | Critical | 📝 Draft |
| EPIC-07 | v3.2 | Critical | 📝 Draft |
| EPIC-08 | v3.2 | High | 📝 Draft |
| EPIC-09 | v3.2 | Medium | 📝 Draft |
| EPIC-10 | v3.2 | Medium | 📝 Draft |
| EPIC-11 | v3.2 | Critical | 📝 Draft |
| EPIC-12 | v3.2 | High | 📝 Draft |
| EPIC-13 | v3.2 | Critical | 📝 Draft |
| EPIC-14 | v3.2 | High | 📝 Draft |
| EPIC-15 | v3.2 | High | 📝 Draft |
| EPIC-16 | v3.3 | Medium | 🔮 Future |
| EPIC-17 | v3.3 | Low | 🔮 Future |
| EPIC-18 | v3.4 | Medium | 🔮 Future |
| EPIC-19 | v3.5 | Low | 🔮 Future |
| EPIC-20 | v3.5 | Medium | 🔮 Future |

---

## Implementation Phases

### Phase 1: Foundation (v3.2) - Q1 2025
- IR Core, Zero-Copy, IndexedFactStore
- DSL Parser, Rule Engine
- Extractors Framework
- CLI, CI/CD Integration

**Milestone**: Production-ready core system

### Phase 2: Optimization (v3.3) - Q2 2025
- Incremental Analysis
- Performance tuning
- Interactive REPL

**Milestone**: 10x faster for incremental scans

### Phase 3: User Experience (v3.4) - Q3 2025
- Web UI & Dashboard
- Enhanced reporting
- VSCode extension

**Milestone**: Enterprise-ready UX

### Phase 4: Advanced Features (v3.5) - Q4 2025
- LSP server
- Advanced correlation rules
- ML integration

**Milestone**: Research-grade capabilities

---

**Última Actualización**: 2025-01-XX
