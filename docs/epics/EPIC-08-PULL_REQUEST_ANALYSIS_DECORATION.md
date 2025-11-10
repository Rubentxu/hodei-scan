# ÉPICA-08: PULL REQUEST ANALYSIS & DECORATION

**Versión:** 2.0
**Fecha:** 10 de noviembre de 2025
**Story Points:** 47 SP
**Sprint Estimado:** 3 sprints
**Dependencias:** EPIC-01, EPIC-02, EPIC-04
**Estado:** 🚀 Ready for Development

---

## 📋 Descripción de la Épica

Esta épica implementa **incremental analysis via IR cache con PR decoration** que proporciona feedback en tiempo real en Pull Requests, coverage deltas, y merge protection basado en quality gates.

**Objetivo Principal:** Proporcionar analysis incremental ultra-rápido (<1s) usando IR cache, con PR decoration en GitHub/GitLab/Bitbucket, coverage deltas, y merge protection basado en quality gates.

---

## 🎯 Objetivos y Alcance

### Objetivos Estratégicos
1. **IR Caching**: Fast incremental analysis
2. **PR Decoration**: GitHub/GitLab/Bitbucket inline comments
3. **Change Impact**: IR diff analysis
4. **Merge Protection**: IR-based quality gates
5. **Coverage Deltas**: Coverage changes en PRs
6. **Security Findings**: New vulnerabilities highlighting

### Alcance Funcional
- ✅ **Incremental Analysis**: <1s usando IR cache
- ✅ **GitHub Integration**: PR comments + check runs
- ✅ **GitLab Integration**: MR comments + status checks
- ✅ **Bitbucket Integration**: PR comments + status
- ✅ **Coverage Deltas**: +X lines covered, -Y uncovered
- ✅ **Security Highlighting**: New vulnerabilities in diff
- ✅ **Quality Gates**: Prevent merge si thresholds no cumplidos

---

## 👥 Historias de Usuario

### US-01: Incremental Analysis
**Como** developer
**Quiero** que analysis sea <1s en PR changes
**Para** get fast feedback

### US-02: PR Decoration
**Como** developer
**Quiero** ver findings inline en PR
**Para** understand impact de changes

### US-03: Coverage Deltas
**Como** tech lead
**Quiero** ver coverage delta en PR
**Para** ensure tests coverage se mantiene

### US-04: Merge Protection
**Como** engineering manager
**Quiero** prevent merge si quality gates fail
**Para** maintain quality standards

### US-05: Security Highlighting
**Como** security engineer
**Quiero** ver new vulnerabilities en PR
**Para** prevent security regressions

---

## ✅ Criterios de Validación

### Funcionales
- [ ] Incremental analysis <1s
- [ ] PR decoration (GitHub, GitLab, Bitbucket)
- [ ] Coverage deltas
- [ ] Security highlighting
- [ ] Merge protection

### Performance
- [ ] Incremental analysis: <1s
- [ ] Diff calculation: <500ms
- [ ] Comment posting: <2s

---

## 📊 Métricas de Éxito

| Métrica | Target | Status |
|---------|--------|--------|
| **Incremental Speed** | <1s | ⏳ |
| **Platforms** | 3/3 | ⏳ |
| **Accuracy** | >95% | ⏳ |

---

## 🚀 Plan de Implementación

### Sprint 1: IR Caching + Incremental Analysis
### Sprint 2: GitHub Integration + Decoration
### Sprint 3: GitLab + Bitbucket + Merge Protection
