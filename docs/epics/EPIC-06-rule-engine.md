# EPIC-06: Rule Engine (Evaluation & Finding Generation)

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: Hodei Scan v3.2  
**Dependencias**: EPIC-02 (IR Core), EPIC-04 (IndexedFactStore), EPIC-05 (DSL Parser)  
**Owner**: Engine Team  
**Prioridad**: Critical Path

---

## 1. Resumen Ejecutivo

Implementar el **Rule Engine**, el núcleo del sistema que evalúa reglas sobre el IR y genera **Findings** (vulnerabilidades, deuda técnica, correlaciones). El engine es **stateless**, **paralelo** (rayon) y tiene **resource limits** para prevenir DoS.

### Objetivo de Negocio
Ejecutar miles de reglas sobre IRs grandes (100MB+) en <5 segundos, generando findings accionables con alta precisión (baja tasa de falsos positivos).

### Métricas de Éxito
- **Rendimiento**: Evaluar 1000 reglas sobre 100MB IR en <5s.
- **Precisión**: Confidence score por finding; <10% falsos positivos en reglas core.
- **Seguridad**: Timeouts por regla (1s default); límites de memoria.
- **Observabilidad**: Telemetría detallada (tiempo por regla, findings por severidad).

---

## 2. Contexto Técnico

### 2.1. Problema
El engine debe:
1. **Cargar reglas** desde archivos `.hodei`.
2. **Indexar el IR** (usando IndexedFactStore).
3. **Evaluar cada regla** paralelamente:
   - Ejecutar patterns (buscar hechos que matchean).
   - Evaluar where clause (filtrar por condiciones).
   - Generar Finding si match exitoso.
4. **Agregar findings** y reportar resultados.

Desafíos:
- **Rendimiento**: Reglas complejas pueden ser O(N²) (spatial joins).
- **Seguridad**: Reglas maliciosas (bucles infinitos, uso excesivo de memoria).
- **Usabilidad**: Errores en reglas deben ser claros.

### 2.2. Solución: Stateless Parallel Evaluator
```
┌────────────────────────────────────────────────────┐
│              RuleEngine                            │
│                                                    │
│  1. Load Rules (parse + type-check)               │
│  2. Build IndexedFactStore                        │
│  3. Evaluate Rules (parallel with rayon)          │
│     ├─ Pattern Matching (use indexes)             │
│     ├─ Where Clause Evaluation (short-circuit)    │
│     └─ Finding Generation (with provenance)       │
│  4. Aggregate & Report                            │
└────────────────────────────────────────────────────┘
```

---

## 3. Alcance

### 3.1. En Alcance (MUST)
1. **RuleEngine**: Orquestador principal.
2. **PatternMatcher**: Ejecuta patterns usando IndexedFactStore.
3. **ExprEvaluator**: Evalúa where clauses (con short-circuit).
4. **FindingBuilder**: Construye findings con metadata.
5. **Resource Limits**: Timeouts, límites de memoria.
6. **Telemetry**: Métricas por regla (tiempo, findings count).

### 3.2. En Alcance (SHOULD)
7. **Incremental Evaluation**: Cache de resultados parciales (v3.3).
8. **Rule Prioritization**: Evaluar reglas críticas primero.
9. **Finding Deduplication**: Eliminar findings duplicados.

### 3.3. Fuera de Alcance
- Persistencia de findings (JSON/SQLite) → EPIC-10.
- Integration con CI/CD → EPIC-12.
- UI para explorar findings → v4.0.

---

## 4. Arquitectura Detallada

### 4.1. RuleEngine (Orchestrator)

```rust
// hodei-engine/src/engine.rs
use hodei_ir::IntermediateRepresentation;
use hodei_dsl::ast::RuleDef;
use crate::store::IndexedFactStore;
use crate::evaluator::{PatternMatcher, ExprEvaluator};
use crate::finding::{Finding, FindingBuilder};
use rayon::prelude::*;
use std::time::Duration;

pub struct RuleEngine {
    config: EngineConfig,
}

#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Timeout por regla (default: 1s)
    pub per_rule_timeout: Duration,
    
    /// Máximo de findings por regla (previene memory exhaustion)
    pub max_findings_per_rule: usize,
    
    /// Paralelismo (default: num_cpus)
    pub parallelism: usize,
    
    /// Habilitar telemetría
    pub enable_telemetry: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            per_rule_timeout: Duration::from_secs(1),
            max_findings_per_rule: 10_000,
            parallelism: rayon::current_num_threads(),
            enable_telemetry: true,
        }
    }
}

pub struct EvaluationResult {
    pub findings: Vec<Finding>,
    pub stats: EvaluationStats,
}

#[derive(Debug, Default)]
pub struct EvaluationStats {
    pub total_rules: usize,
    pub successful_rules: usize,
    pub failed_rules: usize,
    pub total_findings: usize,
    pub total_duration: Duration,
    pub per_rule_stats: Vec<RuleStats>,
}

#[derive(Debug)]
pub struct RuleStats {
    pub rule_name: String,
    pub duration: Duration,
    pub findings_count: usize,
    pub error: Option<String>,
}

impl RuleEngine {
    pub fn new(config: EngineConfig) -> Self {
        Self { config }
    }
    
    /// Evalúa todas las reglas sobre el IR
    pub fn evaluate(
        &self,
        rules: &[RuleDef],
        ir: &IntermediateRepresentation,
    ) -> Result<EvaluationResult, EngineError> {
        let start = std::time::Instant::now();
        
        // 1. Construir índices
        let store = IndexedFactStore::new(ir.facts.clone());
        
        // 2. Evaluar reglas en paralelo
        let rule_results: Vec<_> = rules.par_iter()
            .map(|rule| self.evaluate_rule(rule, &store))
            .collect();
        
        // 3. Agregar resultados
        let mut findings = Vec::new();
        let mut stats = EvaluationStats {
            total_rules: rules.len(),
            ..Default::default()
        };
        
        for result in rule_results {
            match result {
                Ok((rule_findings, rule_stats)) => {
                    findings.extend(rule_findings);
                    stats.successful_rules += 1;
                    stats.per_rule_stats.push(rule_stats);
                }
                Err(err) => {
                    stats.failed_rules += 1;
                    stats.per_rule_stats.push(RuleStats {
                        rule_name: err.rule_name.clone(),
                        duration: Duration::ZERO,
                        findings_count: 0,
                        error: Some(err.to_string()),
                    });
                }
            }
        }
        
        stats.total_findings = findings.len();
        stats.total_duration = start.elapsed();
        
        Ok(EvaluationResult { findings, stats })
    }
    
    /// Evalúa una regla individual (con timeout)
    fn evaluate_rule(
        &self,
        rule: &RuleDef,
        store: &IndexedFactStore,
    ) -> Result<(Vec<Finding>, RuleStats), RuleEvaluationError> {
        let start = std::time::Instant::now();
        
        // Setup timeout (usando crossbeam-channel con timeout)
        let (tx, rx) = crossbeam::channel::bounded(1);
        
        std::thread::scope(|s| {
            s.spawn(|| {
                let result = self.evaluate_rule_impl(rule, store);
                let _ = tx.send(result);
            });
            
            // Esperar resultado con timeout
            match rx.recv_timeout(self.config.per_rule_timeout) {
                Ok(result) => {
                    let duration = start.elapsed();
                    
                    match result {
                        Ok(findings) => {
                            let stats = RuleStats {
                                rule_name: rule.name.clone(),
                                duration,
                                findings_count: findings.len(),
                                error: None,
                            };
                            Ok((findings, stats))
                        }
                        Err(err) => Err(RuleEvaluationError {
                            rule_name: rule.name.clone(),
                            kind: err,
                        }),
                    }
                }
                Err(_timeout) => {
                    Err(RuleEvaluationError {
                        rule_name: rule.name.clone(),
                        kind: EvaluationErrorKind::Timeout,
                    })
                }
            }
        })
    }
    
    /// Implementación de evaluación (sin timeout handling)
    fn evaluate_rule_impl(
        &self,
        rule: &RuleDef,
        store: &IndexedFactStore,
    ) -> Result<Vec<Finding>, EvaluationErrorKind> {
        // 1. Pattern matching
        let matcher = PatternMatcher::new(store);
        let bindings_set = matcher.match_patterns(&rule.match_block.patterns)?;
        
        // 2. Filter por where clause
        let evaluator = ExprEvaluator::new(store);
        let filtered_bindings = if let Some(where_clause) = &rule.match_block.where_clause {
            bindings_set.into_iter()
                .filter(|bindings| {
                    evaluator.eval_expr(where_clause, bindings)
                        .unwrap_or(false)
                })
                .collect()
        } else {
            bindings_set
        };
        
        // 3. Generar findings
        let mut findings = Vec::new();
        let builder = FindingBuilder::new(rule);
        
        for bindings in filtered_bindings {
            if findings.len() >= self.config.max_findings_per_rule {
                return Err(EvaluationErrorKind::TooManyFindings);
            }
            
            let finding = builder.build(&bindings)?;
            findings.push(finding);
        }
        
        Ok(findings)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Failed to build index: {0}")]
    IndexBuildError(String),
}

#[derive(Debug)]
pub struct RuleEvaluationError {
    pub rule_name: String,
    pub kind: EvaluationErrorKind,
}

#[derive(Debug, thiserror::Error)]
pub enum EvaluationErrorKind {
    #[error("Rule evaluation timed out")]
    Timeout,
    
    #[error("Too many findings (>{0})")]
    TooManyFindings,
    
    #[error("Pattern matching failed: {0}")]
    PatternMatchError(String),
    
    #[error("Expression evaluation failed: {0}")]
    ExprEvalError(String),
}
```

### 4.2. PatternMatcher

```rust
// hodei-engine/src/evaluator/pattern_matcher.rs
use hodei_dsl::ast::Pattern;
use hodei_ir::{Fact, FactId};
use crate::store::IndexedFactStore;
use std::collections::HashMap;

/// Binding de variable a FactId
pub type Bindings = HashMap<String, FactId>;

pub struct PatternMatcher<'a> {
    store: &'a IndexedFactStore,
}

impl<'a> PatternMatcher<'a> {
    pub fn new(store: &'a IndexedFactStore) -> Self {
        Self { store }
    }
    
    /// Ejecuta patterns y retorna todas las combinaciones de bindings
    pub fn match_patterns(&self, patterns: &[Pattern]) -> Result<Vec<Bindings>, String> {
        if patterns.is_empty() {
            return Ok(vec![HashMap::new()]);
        }
        
        // Estrategia: empezar por el pattern más selectivo
        let mut patterns_sorted = patterns.to_vec();
        patterns_sorted.sort_by_key(|p| {
            self.store.type_index().cardinality(p.fact_type)
        });
        
        // Ejecutar primer pattern
        let first = &patterns_sorted[0];
        let first_facts: Vec<_> = self.store.by_type(first.fact_type)
            .filter(|fact| self.matches_conditions(fact, &first.conditions))
            .collect();
        
        let mut bindings_set: Vec<Bindings> = first_facts.into_iter()
            .map(|fact| {
                let mut bindings = HashMap::new();
                bindings.insert(first.binding.clone(), fact.id);
                bindings
            })
            .collect();
        
        // Join con resto de patterns
        for pattern in &patterns_sorted[1..] {
            bindings_set = self.join_pattern(bindings_set, pattern)?;
        }
        
        Ok(bindings_set)
    }
    
    /// Join de bindings existentes con un nuevo pattern
    fn join_pattern(
        &self,
        existing_bindings: Vec<Bindings>,
        pattern: &Pattern,
    ) -> Result<Vec<Bindings>, String> {
        let mut new_bindings_set = Vec::new();
        
        for bindings in existing_bindings {
            // Por cada binding existente, encontrar hechos que satisfacen el nuevo pattern
            let candidate_facts: Vec<_> = self.store.by_type(pattern.fact_type)
                .filter(|fact| self.matches_conditions(fact, &pattern.conditions))
                .collect();
            
            for fact in candidate_facts {
                let mut new_bindings = bindings.clone();
                new_bindings.insert(pattern.binding.clone(), fact.id);
                new_bindings_set.push(new_bindings);
            }
        }
        
        Ok(new_bindings_set)
    }
    
    /// Verifica si un hecho satisface todas las condiciones
    fn matches_conditions(&self, fact: &Fact, conditions: &[Condition]) -> bool {
        conditions.iter().all(|cond| self.matches_condition(fact, cond))
    }
    
    fn matches_condition(&self, fact: &Fact, condition: &Condition) -> bool {
        // Resolver path sobre el fact
        let fact_value = self.resolve_path_on_fact(fact, &condition.path);
        
        match condition.op {
            ComparisonOp::Eq => fact_value == Some(&condition.value),
            ComparisonOp::Ne => fact_value != Some(&condition.value),
            ComparisonOp::Contains => {
                // Implementar string contains
                todo!("Implement Contains operator")
            }
            ComparisonOp::Matches => {
                // Implementar regex match
                todo!("Implement Matches operator")
            }
        }
    }
    
    fn resolve_path_on_fact(&self, fact: &Fact, path: &Path) -> Option<&Literal> {
        // Simplificado: asumir path es un campo directo
        // En producción, esto debe caminar el path completo
        match path.segments[0].as_str() {
            "confidence" => {
                Some(&Literal::String(format!("{:?}", fact.confidence)))
            }
            // ... otros campos
            _ => None,
        }
    }
}
```

### 4.3. ExprEvaluator

```rust
// hodei-engine/src/evaluator/expr_evaluator.rs
use hodei_dsl::ast::{Expr, BinaryOp, Literal, Path};
use crate::evaluator::pattern_matcher::Bindings;
use crate::store::IndexedFactStore;

pub struct ExprEvaluator<'a> {
    store: &'a IndexedFactStore,
}

impl<'a> ExprEvaluator<'a> {
    pub fn new(store: &'a IndexedFactStore) -> Self {
        Self { store }
    }
    
    /// Evalúa una expresión contra bindings actuales
    pub fn eval_expr(&self, expr: &Expr, bindings: &Bindings) -> Result<bool, String> {
        match expr {
            Expr::Literal(lit) => {
                match lit {
                    Literal::Boolean(b) => Ok(*b),
                    _ => Err("Expected boolean literal".to_string()),
                }
            }
            
            Expr::Path(path) => {
                let value = self.resolve_path(path, bindings)?;
                match value {
                    Literal::Boolean(b) => Ok(b),
                    _ => Err("Path did not resolve to boolean".to_string()),
                }
            }
            
            Expr::Binary { left, op, right } => {
                match op {
                    BinaryOp::And => {
                        // Short-circuit: si left es false, no evaluar right
                        let left_val = self.eval_expr(left, bindings)?;
                        if !left_val {
                            return Ok(false);
                        }
                        self.eval_expr(right, bindings)
                    }
                    
                    BinaryOp::Or => {
                        let left_val = self.eval_expr(left, bindings)?;
                        if left_val {
                            return Ok(true);
                        }
                        self.eval_expr(right, bindings)
                    }
                    
                    BinaryOp::Eq => {
                        let left_val = self.eval_value_expr(left, bindings)?;
                        let right_val = self.eval_value_expr(right, bindings)?;
                        Ok(left_val == right_val)
                    }
                    
                    // ... otros operadores
                    _ => todo!("Implement other binary operators"),
                }
            }
            
            Expr::FunctionCall { name, args } => {
                self.eval_function_call(name, args, bindings)
            }
        }
    }
    
    /// Evalúa expresión que retorna un valor (no boolean)
    fn eval_value_expr(&self, expr: &Expr, bindings: &Bindings) -> Result<Literal, String> {
        match expr {
            Expr::Literal(lit) => Ok(lit.clone()),
            Expr::Path(path) => self.resolve_path(path, bindings),
            _ => todo!("Implement value expressions"),
        }
    }
    
    fn resolve_path(&self, path: &Path, bindings: &Bindings) -> Result<Literal, String> {
        let first = &path.segments[0];
        let fact_id = bindings.get(first)
            .ok_or_else(|| format!("Undefined variable: {}", first))?;
        
        let fact = self.store.get_fact(*fact_id)
            .ok_or_else(|| format!("Fact not found: {:?}", fact_id))?;
        
        // Resolver resto del path sobre el fact
        self.resolve_path_on_fact(fact, &path.segments[1..])
    }
    
    fn resolve_path_on_fact(&self, fact: &Fact, segments: &[String]) -> Result<Literal, String> {
        if segments.is_empty() {
            return Err("Cannot resolve empty path".to_string());
        }
        
        match segments[0].as_str() {
            "location" => {
                if segments.len() == 1 {
                    // Retornar SourceLocation como string serializado (temporal)
                    Ok(Literal::String(format!("{:?}", fact.source_location)))
                } else {
                    // Resolver sub-campo (e.g., location.file)
                    match segments[1].as_str() {
                        "file" => {
                            let loc = fact.source_location.as_ref()
                                .ok_or("Fact has no location")?;
                            Ok(Literal::String(loc.file.to_string()))
                        }
                        _ => Err(format!("Unknown location field: {}", segments[1])),
                    }
                }
            }
            "confidence" => {
                Ok(Literal::String(format!("{:?}", fact.confidence)))
            }
            _ => Err(format!("Unknown field: {}", segments[0])),
        }
    }
    
    fn eval_function_call(
        &self,
        name: &str,
        args: &[Expr],
        bindings: &Bindings,
    ) -> Result<bool, String> {
        match name {
            "reachable" => {
                // reachable(source_fact, sink_fact) → bool
                if args.len() != 2 {
                    return Err("reachable() requires 2 arguments".to_string());
                }
                
                let source_id = self.eval_fact_id(&args[0], bindings)?;
                let sink_id = self.eval_fact_id(&args[1], bindings)?;
                
                let reachable = self.store.flow_index().shortest_path(source_id, sink_id);
                Ok(reachable.is_some())
            }
            _ => Err(format!("Unknown function: {}", name)),
        }
    }
    
    fn eval_fact_id(&self, expr: &Expr, bindings: &Bindings) -> Result<FactId, String> {
        match expr {
            Expr::Path(path) => {
                let first = &path.segments[0];
                bindings.get(first)
                    .copied()
                    .ok_or_else(|| format!("Undefined variable: {}", first))
            }
            _ => Err("Expected path to fact".to_string()),
        }
    }
}
```

### 4.4. FindingBuilder

```rust
// hodei-engine/src/finding.rs
use hodei_dsl::ast::{RuleDef, Confidence as DslConfidence};
use hodei_ir::{FactId, SourceLocation, Confidence};
use crate::evaluator::pattern_matcher::Bindings;
use crate::store::IndexedFactStore;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub rule_name: String,
    pub severity: Severity,
    pub message: String,
    pub confidence: Confidence,
    pub location: Option<SourceLocation>,
    pub related_facts: Vec<FactId>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

pub struct FindingBuilder<'a> {
    rule: &'a RuleDef,
}

impl<'a> FindingBuilder<'a> {
    pub fn new(rule: &'a RuleDef) -> Self {
        Self { rule }
    }
    
    pub fn build(&self, bindings: &Bindings, store: &IndexedFactStore) -> Result<Finding, String> {
        // 1. Interpolar message template
        let message = self.interpolate_template(&self.rule.emit_block.message_template, bindings, store)?;
        
        // 2. Determinar location (usar primer fact con location)
        let location = bindings.values()
            .filter_map(|fact_id| store.get_fact(*fact_id))
            .find_map(|fact| fact.source_location.clone());
        
        // 3. Colectar related_facts
        let related_facts: Vec<_> = bindings.values().copied().collect();
        
        // 4. Convertir metadata
        let metadata: HashMap<String, String> = self.rule.emit_block.metadata.iter()
            .map(|(k, v)| (k.clone(), format!("{:?}", v)))
            .collect();
        
        Ok(Finding {
            rule_name: self.rule.name.clone(),
            severity: self.convert_severity(self.rule.metadata.severity),
            message,
            confidence: self.convert_confidence(self.rule.emit_block.confidence),
            location,
            related_facts,
            metadata,
        })
    }
    
    fn interpolate_template(
        &self,
        template: &str,
        bindings: &Bindings,
        store: &IndexedFactStore,
    ) -> Result<String, String> {
        let mut result = template.to_string();
        
        // Buscar placeholders: {variable.path}
        let re = regex::Regex::new(r"\{([^}]+)\}").unwrap();
        
        for cap in re.captures_iter(template) {
            let placeholder = &cap[0];
            let path_str = &cap[1];
            
            // Parse path
            let segments: Vec<String> = path_str.split('.').map(String::from).collect();
            let path = Path { segments, span: Span { start: 0, end: 0 } };
            
            // Resolver valor
            let evaluator = ExprEvaluator::new(store);
            let value = evaluator.resolve_path(&path, bindings)?;
            
            result = result.replace(placeholder, &format!("{:?}", value));
        }
        
        Ok(result)
    }
    
    fn convert_severity(&self, sev: hodei_dsl::ast::Severity) -> Severity {
        match sev {
            hodei_dsl::ast::Severity::Critical => Severity::Critical,
            hodei_dsl::ast::Severity::High => Severity::High,
            hodei_dsl::ast::Severity::Medium => Severity::Medium,
            hodei_dsl::ast::Severity::Low => Severity::Low,
            hodei_dsl::ast::Severity::Info => Severity::Info,
        }
    }
    
    fn convert_confidence(&self, conf: DslConfidence) -> Confidence {
        match conf {
            DslConfidence::High => Confidence::new(0.9).unwrap(),
            DslConfidence::Medium => Confidence::new(0.6).unwrap(),
            DslConfidence::Low => Confidence::new(0.3).unwrap(),
        }
    }
}
```

---

## 5. Plan de Implementación

### 5.1. Fases

**Fase 1: RuleEngine Scaffold** (Semana 1)
- [ ] Implementar `RuleEngine` con evaluación secuencial.
- [ ] Tests: evaluar regla simple (1 pattern, no where clause).
- [ ] Telemetry básica (duración por regla).

**Fase 2: PatternMatcher** (Semana 1-2)
- [ ] Implementar `PatternMatcher` (join de patterns).
- [ ] Tests: multi-pattern rules (spatial join).
- [ ] Benchmark: evaluar regla sobre 100k hechos.

**Fase 3: ExprEvaluator** (Semana 2)
- [ ] Implementar `ExprEvaluator` con short-circuit.
- [ ] Implementar built-in functions (reachable, distance).
- [ ] Tests: where clauses complejas.

**Fase 4: FindingBuilder** (Semana 2-3)
- [ ] Implementar `FindingBuilder` con template interpolation.
- [ ] Tests: verificar format de findings.

**Fase 5: Paralelización & Resource Limits** (Semana 3)
- [ ] Paralelizar evaluación con rayon.
- [ ] Implementar timeouts por regla.
- [ ] Tests: verificar timeout mata threads.
- [ ] Benchmark: evaluar 1000 reglas en paralelo.

**Fase 6: Telemetry & Observability** (Semana 3-4)
- [ ] Métricas detalladas (tiempo por fase, memoria peak).
- [ ] Export a JSON/Prometheus.
- [ ] Tests: verificar métricas se colectan correctamente.

### 5.2. Dependencias de Crates
```toml
[dependencies]
rayon = "1.10"
crossbeam = "0.8"
regex = "1.10"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"

[dev-dependencies]
criterion = "0.5"
```

---

## 6. Tests & Validación

### 6.1. Tests Unitarios

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn evaluate_simple_rule() {
        let rule = parse_rule(r#"
            rule FindSinks {
                match {
                    sink: TaintSink
                }
                emit Finding {
                    message: "Found sink"
                    confidence: High
                }
            }
        "#);
        
        let ir = create_test_ir_with_sinks(10);
        let engine = RuleEngine::default();
        
        let result = engine.evaluate(&[rule], &ir).unwrap();
        
        assert_eq!(result.findings.len(), 10);
        assert_eq!(result.stats.successful_rules, 1);
    }
    
    #[test]
    fn evaluate_spatial_join_rule() {
        let rule = parse_rule(r#"
            rule VulnerableUncovered {
                match {
                    sink: TaintSink and
                    uncovered: UncoveredLine
                    where sink.location == uncovered.location
                }
                emit Finding {
                    message: "Vulnerable code uncovered"
                    confidence: High
                }
            }
        "#);
        
        let ir = create_ir_with_colocated_facts();
        let engine = RuleEngine::default();
        
        let result = engine.evaluate(&[rule], &ir).unwrap();
        
        assert!(result.findings.len() > 0);
    }
    
    #[test]
    fn timeout_on_slow_rule() {
        let rule = parse_rule(r#"
            rule SlowRule {
                match {
                    sink: TaintSink
                    where infinite_loop()  // Función que nunca retorna
                }
                emit Finding {
                    message: "Never"
                    confidence: High
                }
            }
        "#);
        
        let ir = create_test_ir();
        let engine = RuleEngine::new(EngineConfig {
            per_rule_timeout: Duration::from_millis(100),
            ..Default::default()
        });
        
        let result = engine.evaluate(&[rule], &ir).unwrap();
        
        assert_eq!(result.stats.failed_rules, 1);
        assert!(result.stats.per_rule_stats[0].error.is_some());
    }
}
```

### 6.2. Benchmarks

```rust
// benches/rule_engine.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_rule_engine(c: &mut Criterion) {
    let mut group = c.benchmark_group("RuleEngine");
    
    let rules = load_test_rules(1000);  // 1000 reglas
    let ir = create_large_ir(100_000);   // 100k hechos
    
    group.bench_function("evaluate_1000_rules", |b| {
        let engine = RuleEngine::default();
        b.iter(|| {
            let result = engine.evaluate(&rules, &ir).unwrap();
            black_box(result)
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_rule_engine);
criterion_main!(benches);
```

---

## 7. Seguridad & Mitigaciones

### 7.1. Resource Limits

```rust
// Configuración de límites
let config = EngineConfig {
    per_rule_timeout: Duration::from_secs(1),
    max_findings_per_rule: 10_000,
    max_memory_per_rule: 100 * 1024 * 1024,  // 100 MB
    ..Default::default()
};
```

### 7.2. Threat Model

| Amenaza | Mitigación |
|---------|------------|
| DoS via reglas lentas | Timeout de 1s por regla |
| Memory exhaustion | Límite de 10k findings por regla |
| Reglas mal formadas | Type checking antes de evaluación |
| Path traversal en findings | ProjectPath sanitization |

---

## 8. Criterios de Aceptación

- [ ] **Funcional**: Evalúa reglas con patterns, where clauses, findings.
- [ ] **Rendimiento**: 1000 reglas sobre 100MB IR en <5s.
- [ ] **Seguridad**: Timeouts implementados; límites de memoria.
- [ ] **Telemetría**: Métricas detalladas por regla.
- [ ] **Tests**: Cobertura >90%; benchmarks en CI.
- [ ] **Docs**: Guía de uso del engine; arquitectura documentada.

---

**Última Actualización**: 2025-01-XX  
**Próxima Revisión**: Después de Fase 3 (ExprEvaluator implementado)
