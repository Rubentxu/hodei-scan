# ÉPICA-05: TECHNICAL DEBT CALCULATION

**Versión:** 2.0
**Fecha:** 10 de noviembre de 2025
**Story Points:** 52 SP
**Sprint Estimado:** 4 sprints
**Dependencias:** EPIC-01-CORE_STATIC_ANALYSIS_ENGINE
**Estado:** 🚀 Ready for Development

---

## 📋 Descripción de la Épica

Esta épica implementa el **motor de cálculo de technical debt basado en IR** usando el framework NIST para estimación automática de costos de remediación. Proporciona tracking histórico, ROI analysis, y executive reporting.

**Objetivo Principal:** Calcular technical debt hours y costos usando NIST framework, proporcionar remediation roadmap, y trackear evolución del debt over time.

---

## 🎯 Objetivos y Alcance

### Objetivos Estratégicos
1. **NIST Framework**: Automated remediation cost estimation
2. **Language Rates**: Rust $150/hr, Go $130/hr, TS $125/hr, Python $120/hr, Java $120/hr
3. **Historical Tracking**: Debt evolution over time
4. **Priority Scheduling**: Remediation roadmap
5. **ROI Analysis**: Cost vs benefit por issue
6. **Executive Reporting**: Business-friendly metrics

### Alcance Funcional
- ✅ **Issue-based Debt**: Security, Quality, Complexity issues
- ✅ **Estimation Engine**: NIST-based calculation
- ✅ **Historical Tracking**: Debt trends
- ✅ **Remediation Planning**: Prioritization algorithm
- ✅ **ROI Calculation**: Cost-benefit analysis
- ✅ **Executive Dashboard**: High-level metrics
- ✅ **Integration**: IR facts correlation

---

## 👥 Historias de Usuario

### US-01: Debt Estimation Engine
**Como** technical lead
**Quiero** que el sistema calcule debt hours automáticamente
**Para** entender costo de remediación

### US-02: Historical Tracking
**Como** engineering manager
**Quiero** trackear debt evolution over time
**Para** medir progreso de refactoring

### US-03: Remediation Roadmap
**Como** tech lead
**Quiero** un roadmap de remediation priorizado
**Para** planificar refactoring sprints

### US-04: ROI Analysis
**Como** engineering manager
**Quiero** ver cost vs benefit de debt reduction
**Para** justificar investment en quality

---

## ✅ Criterios de Validación

### Funcionales
- [ ] NIST framework implementado
- [ ] Language rates configurados
- [ ] Historical tracking
- [ ] Remediation planning
- [ ] ROI calculation

### Performance
- [ ] Calculation: <3s para 10K issues
- [ ] Trend analysis: <5s
- [ ] Report generation: <10s

---

## 📊 Métricas de Éxito

| Métrica | Target | Status |
|---------|--------|--------|
| **Estimation Accuracy** | >85% | ⏳ |
| **Calculation Speed** | <3s | ⏳ |
| **Debt Tracking** | 100% issues | ⏳ |

---

## 🚀 Plan de Implementación

### Sprint 1: NIST Framework + Estimation
### Sprint 2: Historical Tracking + Trends
### Sprint 3: Remediation Planning + ROI
### Sprint 4: Executive Reporting + Dashboards
