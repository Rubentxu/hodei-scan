# Épica 8: Pull Request Analysis & Decoration
## Análisis Automático y Decoración de PRs

**Versión:** 1.0
**Fecha:** 10 de noviembre de 2025
**Estado:** 🚧 Planning
**Época:** Fase 3 (Meses 13-24)
**Prioridad:** 🔴 High

---

## 📋 Resumen Ejecutivo

Implementar análisis automático de Pull Requests y decoración en GitHub/GitLab/Bitbucket, proporcionando inline comments, coverage deltas y quality gate status directamente en PRs.

**Objetivos:**
- ✅ PR decoration en GitHub/GitLab/Bitbucket
- ✅ Inline comments con issues encontrados
- ✅ Branch-specific analysis results
- ✅ Code coverage deltas en PRs
- ✅ Quality gate status per PR
- ✅ Security findings highlighting

**Métricas:** <60s analysis por PR, 100% VCS integrations, real-time updates

---

## 👥 Historias de Usuario

### US-23: Como developer, quiero ver analysis results directly en PR

**Prioridad:** 🔴 Critical
**Story Points:** 8

```gherkin
Feature: PR Decoration
  Como developer revisando PR
  Quiero ver analysis results en PR interface
  Para get immediate feedback

  Scenario: New issues in PR
    Given PR con nuevos code issues
    When hodei-scan analiza PR
    Then debería add inline comments
    And debería mark problematic lines
    And debería suggest fixes

  Scenario: Coverage delta
    Given PR que changes coverage
    When hodei-scan analiza PR
    Then debería show coverage change
    And debería comment coverage impact
```

**Tareas:**

1. **TASK-08-01: Implementar VCS Integrations (GitHub, GitLab, Bitbucket)** (5 días)
2. **TASK-08-02: Implementar Inline Comment Generator** (3 días)
3. **TASK-08-03: Implementar PR Analysis Engine** (4 días)
4. **TASK-08-04: Implementar Quality Gate Status** (2 días)

**Tests:**

```rust
#[test]
fn test_github_pr_decoration() {
    let client = GitHubClient::new("token");
    let result = PRAnalysisResult {
        new_issues: vec![Issue { file: "src/main.rs", line: 10, severity: Severity::Critical }],
        coverage_change: Some(CoverageDelta { delta: -5 }),
        quality_gate_status: QualityGateResult::Failed,
    };

    client.decorate_pr("owner", "repo", 123, &result).unwrap();
    assert!(client.has_comment(123));
}
```

---

### US-24: Como reviewer, quiero understand PR impact

**Prioridad:** 🟡 Medium
**Story Points:** 5

```gherkin
Feature: PR Impact Analysis
  Como reviewer evaluando PR
  Quiero understand PR's impact en quality
  Para make informed review decisions

  Scenario: Security impact
    Given PR con security-related changes
    When hodei-scan analiza PR
    Then debería highlight security findings
    And debería show security score change
    And debería suggest security review
```

**Tareas:**

1. **TASK-08-05: Implementar Impact Analyzer** (3 días)
2. **TASK-08-06: Implementar Security Highlighting** (2 días)

---

## 🏗️ Arquitectura

```rust
pub struct PRDecorationEngine {
    pub vcs_integrations: HashMap<String, VCSIntegration>,
    pub comment_generator: IssueCommentGenerator,
    pub coverage_reporter: CoverageReporter,
    pub quality_gate_status: QualityGateChecker,
}

pub struct PRAnalysisResult {
    pub new_issues: Vec<Issue>,
    pub fixed_issues: Vec<Issue>,
    pub coverage_change: Option<CoverageDelta>,
    pub quality_gate_status: QualityGateResult,
    pub security_findings: Vec<SecurityIssue>,
}
```

---

## 🔄 Criterios de Done

- [ ] ✅ GitHub integration
- [ ] ✅ GitLab integration
- [ ] ✅ Bitbucket integration
- [ ] ✅ <60s analysis
- [ ] ✅ Inline comments
- [ ] ✅ Quality gate status
- [ ] ✅ 100% tests

**Total Story Points:** 26 | **Duración:** 6 semanas
