# EPIC-08: Quality Gates Plugin

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: Hodei Scan v3.2  
**Dependencias**: EPIC-06 (Rule Engine)  
**Owner**: Quality Team  
**Prioridad**: High

---

## 1. Resumen Ejecutivo

Implementar **Quality Gates**: evaluación de umbrales sobre findings para aprobar/rechazar builds en CI/CD. Permite definir políticas como "bloquear si hay >5 vulnerabilidades Critical" o "warn si cobertura <80%".

### Objetivo de Negocio
Automatizar decisiones de calidad/seguridad en CI/CD, reduciendo el "security debt" y previniendo regresiones.

### Métricas de Éxito
- **Flexibilidad**: Soporte para umbrales sobre severidad, tipo, cobertura, deuda técnica.
- **CI/CD Integration**: Exit codes apropiados (0=pass, 1=fail).
- **Usabilidad**: Configuración declarativa (YAML/TOML).

---

## 2. Arquitectura

### 2.1. Gate Definition

```rust
// hodei-gates/src/lib.rs
use hodei_engine::finding::Finding;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    pub name: String,
    pub conditions: Vec<GateCondition>,
    pub action: GateAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCondition {
    pub metric: Metric,
    pub operator: Operator,
    pub threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Metric {
    /// Número de findings por severidad
    FindingsBySeverity(Severity),
    
    /// Número de findings por tipo
    FindingsByType(String),
    
    /// Cobertura de líneas (%)
    LineCoverage,
    
    /// Deuda técnica (horas estimadas)
    TechnicalDebt,
    
    /// Ratio de vulnerabilidades en código sin tests
    VulnerableUncoveredRatio,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Operator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Equal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GateAction {
    Fail,   // Exit code 1
    Warn,   // Print warning, exit 0
    Block,  // Exit code 2 (para distinguir de otros errores)
}

pub struct QualityGateEvaluator;

impl QualityGateEvaluator {
    pub fn evaluate(
        gate: &QualityGate,
        findings: &[Finding],
        ir: &IntermediateRepresentation,
    ) -> GateResult {
        let mut passed = true;
        let mut violations = Vec::new();
        
        for condition in &gate.conditions {
            let actual_value = Self::compute_metric(&condition.metric, findings, ir);
            let satisfied = Self::check_condition(actual_value, condition.operator, condition.threshold);
            
            if !satisfied {
                passed = false;
                violations.push(GateViolation {
                    condition: condition.clone(),
                    actual_value,
                    expected: condition.threshold,
                });
            }
        }
        
        GateResult {
            gate_name: gate.name.clone(),
            passed,
            action: gate.action,
            violations,
        }
    }
    
    fn compute_metric(metric: &Metric, findings: &[Finding], ir: &IntermediateRepresentation) -> f64 {
        match metric {
            Metric::FindingsBySeverity(sev) => {
                findings.iter()
                    .filter(|f| f.severity == *sev)
                    .count() as f64
            }
            
            Metric::LineCoverage => {
                let total_lines = ir.facts.iter()
                    .filter(|f| matches!(f.fact_type, FactType::CoveredLine | FactType::UncoveredLine))
                    .count();
                
                let covered_lines = ir.facts.iter()
                    .filter(|f| matches!(f.fact_type, FactType::CoveredLine))
                    .count();
                
                if total_lines == 0 {
                    0.0
                } else {
                    (covered_lines as f64 / total_lines as f64) * 100.0
                }
            }
            
            Metric::TechnicalDebt => {
                findings.iter()
                    .filter_map(|f| f.metadata.get("debt_hours"))
                    .filter_map(|v| v.parse::<f64>().ok())
                    .sum()
            }
            
            _ => 0.0,
        }
    }
    
    fn check_condition(actual: f64, op: Operator, threshold: f64) -> bool {
        match op {
            Operator::LessThan => actual < threshold,
            Operator::LessThanOrEqual => actual <= threshold,
            Operator::GreaterThan => actual > threshold,
            Operator::GreaterThanOrEqual => actual >= threshold,
            Operator::Equal => (actual - threshold).abs() < f64::EPSILON,
        }
    }
}

#[derive(Debug)]
pub struct GateResult {
    pub gate_name: String,
    pub passed: bool,
    pub action: GateAction,
    pub violations: Vec<GateViolation>,
}

#[derive(Debug)]
pub struct GateViolation {
    pub condition: GateCondition,
    pub actual_value: f64,
    pub expected: f64,
}
```

### 2.2. Configuración (YAML)

```yaml
# .hodei/quality-gates.yaml
gates:
  - name: "Security Critical"
    conditions:
      - metric: FindingsBySeverity(Critical)
        operator: Equal
        threshold: 0
    action: Fail
  
  - name: "Code Coverage"
    conditions:
      - metric: LineCoverage
        operator: GreaterThanOrEqual
        threshold: 80.0
    action: Warn
  
  - name: "Technical Debt"
    conditions:
      - metric: TechnicalDebt
        operator: LessThan
        threshold: 40.0  # horas
    action: Warn
  
  - name: "Vulnerable Uncovered Code"
    conditions:
      - metric: VulnerableUncoveredRatio
        operator: LessThan
        threshold: 0.1  # <10% de vulnerabilidades sin tests
    action: Fail
```

### 2.3. CLI Integration

```rust
// hodei-cli/src/commands/check.rs
pub async fn cmd_check(args: CheckArgs) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Ejecutar extractors
    let ir = run_extractors(&args.project_path).await?;
    
    // 2. Ejecutar rule engine
    let rules = load_rules(&args.rules_path)?;
    let eval_result = RuleEngine::default().evaluate(&rules, &ir)?;
    
    // 3. Evaluar quality gates
    let gates = load_quality_gates(&args.gates_config)?;
    let mut any_failed = false;
    
    for gate in gates {
        let result = QualityGateEvaluator::evaluate(&gate, &eval_result.findings, &ir);
        
        print_gate_result(&result);
        
        if !result.passed {
            match result.action {
                GateAction::Fail | GateAction::Block => {
                    any_failed = true;
                }
                GateAction::Warn => {
                    eprintln!("⚠️  Warning: Gate '{}' failed", gate.name);
                }
            }
        }
    }
    
    // 4. Exit con código apropiado
    if any_failed {
        std::process::exit(1);
    } else {
        println!("✅ All quality gates passed");
        Ok(())
    }
}

fn print_gate_result(result: &GateResult) {
    if result.passed {
        println!("✅ Gate '{}' passed", result.gate_name);
    } else {
        eprintln!("❌ Gate '{}' failed:", result.gate_name);
        for violation in &result.violations {
            eprintln!(
                "   - Expected {:?} {:?} {}, got {}",
                violation.condition.metric,
                violation.condition.operator,
                violation.expected,
                violation.actual_value
            );
        }
    }
}
```

---

## 3. Plan de Implementación

**Fase 1: Core** (Semana 1)
- [ ] Definir `QualityGate`, `GateCondition`, `Metric`.
- [ ] Implementar `QualityGateEvaluator`.
- [ ] Tests: evaluar umbrales simples.

**Fase 2: Métricas Avanzadas** (Semana 2)
- [ ] Implementar métricas complejas (VulnerableUncoveredRatio, TechnicalDebt).
- [ ] Tests con fixtures realistas.

**Fase 3: CLI Integration** (Semana 2)
- [ ] Comando `hodei check` con gates.
- [ ] Exit codes correctos.
- [ ] Formateo de output.

**Fase 4: CI/CD Examples** (Semana 3)
- [ ] Ejemplos para GitHub Actions, GitLab CI.
- [ ] Documentación de integración.

---

## 4. Tests & Validación

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn gate_fails_when_critical_findings_exceed_threshold() {
        let gate = QualityGate {
            name: "Security".to_string(),
            conditions: vec![
                GateCondition {
                    metric: Metric::FindingsBySeverity(Severity::Critical),
                    operator: Operator::LessThan,
                    threshold: 5.0,
                }
            ],
            action: GateAction::Fail,
        };
        
        let findings = create_findings_with_severity(Severity::Critical, 10);
        let ir = IntermediateRepresentation::default();
        
        let result = QualityGateEvaluator::evaluate(&gate, &findings, &ir);
        
        assert!(!result.passed);
        assert_eq!(result.violations.len(), 1);
    }
    
    #[test]
    fn gate_passes_when_coverage_above_threshold() {
        let gate = QualityGate {
            name: "Coverage".to_string(),
            conditions: vec![
                GateCondition {
                    metric: Metric::LineCoverage,
                    operator: Operator::GreaterThanOrEqual,
                    threshold: 80.0,
                }
            ],
            action: GateAction::Warn,
        };
        
        let ir = create_ir_with_coverage(85.0);  // 85% coverage
        
        let result = QualityGateEvaluator::evaluate(&gate, &[], &ir);
        
        assert!(result.passed);
    }
}
```

---

## 5. Criterios de Aceptación

- [ ] Soporte para 5+ métricas.
- [ ] Configuración YAML funcional.
- [ ] CLI con exit codes correctos.
- [ ] Documentación para CI/CD.
- [ ] Tests con cobertura >90%.

---

**Última Actualización**: 2025-01-XX
