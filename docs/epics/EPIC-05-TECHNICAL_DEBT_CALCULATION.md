# Épica 5: Technical Debt Calculation
## Cálculo Automatizado de Technical Debt con NIST Framework

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 🚧 Planning
**Época:** Fase 2 (Meses 7-12)
**Prioridad:** 🟡 High

---

## 📋 Resumen Ejecutivo

Implementar el motor de cálculo de technical debt para hodei-scan usando NIST framework. Este motor calculará automáticamente el costo de remediación, proporcionará prioritization y tracking histórico del debt evolution.

**Objetivos:**
- ✅ Automated remediation cost estimation (NIST framework)
- ✅ Language-specific rates (Rust: $150/hr, Java: $120/hr)
- ✅ Issue-type weighting (Critical: 8x, Major: 4x, Minor: 2x)
- ✅ Historical tracking
- ✅ Priority-based remediation scheduling

**Métricas:** <5s calculation para proyecto 100K LOC, 95% accuracy en cost estimation

---

## 👥 Historias de Usuario

### US-17: Como CTO, quiero saber el costo total de technical debt

**Prioridad:** 🔴 Critical
**Story Points:** 8

```gherkin
Feature: Technical Debt Cost Calculation
  Como CTO planificando budget
  Quiero conocer costo total de technical debt
  Para justiciar investment en refactoring

  Scenario: Calculate debt for entire project
    Given proyecto con 150 issues de different severities
    When ejecuto hodei-scan debt calculate
    Then debería mostrar total cost en dollars
    And debería break down por severity
    And debería show remediation timeline

  Scenario: Debt por language
    Given proyecto multi-language con technical debt
    When ejecuto debt report
    Then debería mostrar debt breakdown por language
    And debería usar rates específicos por language
```

**Tareas:**

1. **TASK-05-01: Implementar NIST Debt Calculator** (5 días)
2. **TASK-05-02: Implementar Language Rates Matrix** (2 días)
3. **TASK-05-03: Implementar Historical Tracking** (3 días)
4. **TASK-05-04: Implementar Remediation Scheduler** (3 días)

**Tests:**

```rust
#[test]
fn test_debt_calculation_nist_framework() {
    let calculator = DebtCalculator::new();
    let issues = vec![
        Issue { severity: Severity::Critical, issue_type: IssueType::CodeSmell, language: "rust" },
        Issue { severity: Severity::Major, issue_type: IssueType::Bug, language: "go" },
        Issue { severity: Severity::Minor, issue_type: IssueType::CodeSmell, language: "typescript" },
    ];

    let report = calculator.calculate(&issues).unwrap();
    assert!(report.total_cost > 0);
    assert_eq!(report.by_severity.get(&Severity::Critical).unwrap(), &(8.0, "$1200/hr"));
}
```

---

### US-18: Como team lead, quiero prioritization de debt remediation

**Prioridad:** 🟡 Medium
**Story Points:** 5

```gherkin
Feature: Debt Prioritization
  Como team lead planning sprint
  Quiero prioritization automática de technical debt
  Para optimizar impact vs effort

  Scenario: Prioritize by impact/effort
    Given issues con different impact scores
    When ejecuto hodei-scan debt prioritize
    Then debería order issues por ROI
    And debería suggest quick wins primero
    And debería group related issues
```

**Tareas:**

1. **TASK-05-05: Implementar Impact/Effort Matrix** (3 días)
2. **TASK-05-06: Implementar Quick Wins Detector** (2 días)

---

## 🏗️ Arquitectura

**NIST Framework Implementation:**

```rust
pub struct TechnicalDebtCalculator {
    pub language_rates: HashMap<String, DollarPerHour>,
    pub issue_weights: HashMap<Severity, WeightMultiplier>,
    pub remediation_speeds: HashMap<IssueType, HoursPerIssue>,
}

pub struct TechnicalDebtReport {
    pub total_debt: DollarAmount,
    pub by_severity: HashMap<Severity, DollarAmount>,
    pub by_language: HashMap<String, DollarAmount>,
    pub remediation_timeline: RemediationSchedule,
    pub cost_benefit_analysis: CostBenefitAnalysis,
}
```

**Language Rates (NIST Based):**
- Rust: $150/hr
- Go: $130/hr
- TypeScript/JavaScript: $125/hr
- Python: $120/hr
- Java: $120/hr
- C++: $140/hr

---

## 🔄 Criterios de Done

- [ ] ✅ NIST framework implementado
- [ ] ✅ <5s calculation time
- [ ] ✅ 6 languages support
- [ ] ✅ Historical tracking
- [ ] ✅ Prioritization engine
- [ ] ✅ Cost-benefit analysis
- [ ] ✅ 100% tests en verde

**Total Story Points:** 26 | **Duración:** 6 semanas
