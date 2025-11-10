# Épica 7: Portfolio Management
## Organization-Wide Code Health Visibility

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 🚧 Planning
**Época:** Fase 3 (Meses 13-24)
**Prioridad:** 🟡 High

---

## 📋 Resumen Ejecutivo

Implementar portfolio management para hodei-scan, proporcionando organization-wide visibility en code health, executive dashboards y scheduled reporting.

**Objetivos:**
- ✅ Organization-wide project grouping
- ✅ Executive dashboards con high-level metrics
- ✅ Scheduled PDF reports
- ✅ Holistic code health views
- ✅ Cross-project compliance reporting
- ✅ Portfolio-level quality trends

**Métricas:** <30s dashboard load para 100 projects, real-time updates, 95% report accuracy

---

## 👥 Historias de Usuario

### US-21: Como C-level executive, quiero holistic view de code quality

**Prioridad:** 🔴 Critical
**Story Points:** 8

```gherkin
Feature: Executive Dashboard
  Como C-level executive
  Quiero dashboard con code health overview
  Para tomar strategic decisions

  Scenario: Organization health overview
    Given organization con 50+ projects
    When accedo a executive dashboard
    Then debería ver overall health score
    And debería ver top/bottom performers
    And debería see trends over time
    And debería get investment recommendations
```

**Tareas:**

1. **TASK-07-01: Implementar Portfolio Grouping** (3 días)
2. **TASK-07-02: Implementar Executive Dashboard** (4 días)
3. **TASK-07-03: Implementar Report Scheduler** (2 días)

### US-22: Como compliance officer, quiero cross-project compliance reporting

**Prioridad:** 🟡 Medium
**Story Points:** 5

```gherkin
Feature: Cross-Project Compliance
  Como compliance officer auditando
  Quiero compliance report across projects
  Para ensure regulatory compliance

  Scenario: SOC 2 compliance report
    Given organization con multiple projects
    When genero compliance report
    Then debería show compliance por project
    And debería aggregate organization-wide
    And debería identify gaps
```

**Tareas:**

1. **TASK-07-04: Implementar Compliance Aggregator** (3 días)
2. **TASK-07-05: Implementar PDF Report Generator** (2 días)

---

## 🏗️ Arquitectura

```rust
pub struct PortfolioManager {
    pub project_groups: HashMap<String, Portfolio>,
    pub executive_dashboards: ExecutiveDashboardBuilder,
    pub report_scheduler: ScheduledReporter,
    pub compliance_reporter: ComplianceReporter,
}

pub struct ExecutiveDashboard {
    pub organization_health: OrgHealthScore,
    pub quality_trends: TimeSeriesMetrics,
    pub security_overview: SecuritySummary,
    pub compliance_status: ComplianceStatus,
    pub investment_recommendations: InvestmentGuidance,
}
```

---

## 🔄 Criterios de Done

- [ ] ✅ Portfolio grouping
- [ ] ✅ Executive dashboard
- [ ] ✅ Report scheduler
- [ ] ✅ Compliance reporting
- [ ] ✅ <30s dashboard load
- [ ] ✅ 100% tests

**Total Story Points:** 26 | **Duración:** 6 semanas
