# EPIC-10: Extractor Ecosystem - Multi-Process Architecture

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: hodei-scan v3.3  
**Dependencias**: EPIC-06 (RuleEngine)  
**Owner**: Core Platform Team  
**Prioridad**: Critical Path

---

## 1. Resumen Ejecutivo

Implementar un **ecosistema de extractores multi-proceso** que permita integración con herramientas externas y análisis multi-lenguaje sin recompilación del core. Esta épica transforma hodei-scan de un sistema monolítico a una **plataforma extensible**.

### Objetivo de Negocio
Permitir que cualquier herramienta de análisis (Ruff, ESLint, SARIF-based tools) se integre con hodei-scan en **<5 minutos**, ampliando la cobertura de análisis drásticamente sin desarrollo nativo.

### Métricas de Éxito
- **Integración**: 10+ herramientas externas integradas en 3 meses
- **Performance**: <20% overhead vs análisis nativo para herramientas adaptadas
- **Extensibilidad**: Nuevo extractor sin recompilación del core
- **Robustez**: 99.9% uptime con manejo de fallos de extractores

---

## 2. Contexto Técnico

### 2.1. Problema Actual
Los extractores están acoplados al core de hodei-scan:
- Nuevo extractor = modificar core + recompilar
- Solo lenguaje Rust soportado
- No hay integración con herramientas existentes (Ruff, ESLint, etc.)

### 2.2. Solución: Contrato de Proceso

```
hodei-scan (orchestrator)
    ├── extractor-1 (process) ← stdin: config, stdout: IR (Cap'n Proto)
    ├── extractor-2 (process)
    └── extractor-N (process)

[hodei.toml]
[[extractors]]
name = "ruff"
command = "ruff-to-hodei"
args = ["--format", "json"]
```

**Contrato del Extractor:**
- **Input**: Config JSON por stdin (project_path, language, rules)
- **Output**: IR Cap'n Proto por stdout (facts, metadata)
- **Error**: stderr logs + exit code
- **Timeout**: Límite configurable (default: 30s)

---

## 3. Arquitectura Detallada

### 3.1. Componentes Core

#### ExtractorOrchestrator
```rust
pub struct ExtractorOrchestrator {
    config: ExtractorConfig,
    running_extractors: HashMap<String, ChildProcess>,
    result_aggregator: ResultAggregator,
}

impl ExtractorOrchestrator {
    /// Execute all extractors defined in hodei.toml
    pub async fn execute_all(&self) -> Result<AggregatedIR, OrchestratorError> {
        // Spawn all extractors concurrently
        // Aggregate results from stdout streams
        // Handle timeouts and failures
        // Return merged IR
    }
}
```

#### Cap'n Proto Protocol
```rust
// Schema: extractor_protocol.capnp
struct ExtractorRequest {
    projectPath @0 :Text;
    language @1 :Text;
    config @2 :Text;  // JSON config
    timeoutMs @3 :UInt32;
}

struct ExtractorResponse {
    success @0 :Bool;
    ir @1 :Data;  // Cap'n Proto serialized IR
    errorMessage @2 :Text;
    metadata @3 :Text;  // JSON (extractor version, stats)
}
```

### 3.2. Proceso de Ejecución

```
1. Parse hodei.toml
   ↓
2. Validate extractor definitions
   ↓
3. Spawn extractor processes (async)
   ↓
4. Send request via stdin
   ↓
5. Read IR from stdout
   ↓
6. Validate IR against schema
   ↓
7. Aggregate all IRs
   ↓
8. Return unified IR
```

---

## 4. Plan de Implementación

### 4.1. Fases

**Fase 1: Core Protocol (Semana 1)**
- [ ] Implementar ExtractorOrchestrator básico
- [ ] Definir Cap'n Proto schema
- [ ] Test con extractor "echo" (mock)

**Fase 2: SARIF Adapter (Semana 2)**
- [ ] Crear `sarif-to-hodei` extractor
- [ ] Parse SARIF → IR transformation
- [ ] Test con herramientas reales (Semgrep SARIF)

**Fase 3: Ruff Adapter (Semana 3)**
- [ ] Crear `ruff-to-hodei` adapter
- [ ] Ruff JSON output → IR mapping
- [ ] Performance optimization

**Fase 4: Advanced Orchestration (Semana 4)**
- [ ] Concurrency limits (max extractors in parallel)
- [ ] Resource monitoring (CPU, memory per extractor)
- [ ] Graceful shutdown on timeout

---

## 5. User Stories

### US-10.01: Implementar ExtractorOrchestrator con Cap'n Proto

**Como:** Desarrollador del Core  
**Quiero:** Un orquestador que ejecute extractores como procesos externos  
**Para:** Permitir extensibilidad sin recompilación

**Criterios de Aceptación:**
- [ ] Lee configuración desde `hodei.toml`
- [ ] Spawn extractor como proceso hijo
- [ ] Envía request JSON por stdin
- [ ] Lee IR Cap'n Proto desde stdout
- [ ] Maneja timeouts y errores
- [ ] Agrega múltiples IRs en uno solo

**TDD - Red (Test que falla):**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_execute_simple_extractor() {
        let orchestrator = ExtractorOrchestrator::new();
        let config = ExtractorConfig {
            extractors: vec![ExtractorDef {
                name: "mock".to_string(),
                command: "mock-extractor".to_string(),
                args: vec![],
            }],
        };
        
        let result = orchestrator.execute_all().await;
        assert!(result.is_ok());
    }
}
```

**TDD - Green (Implementación mínima):**
```rust
pub async fn execute_all(&self) -> Result<AggregatedIR, OrchestratorError> {
    let mut tasks = Vec::new();
    
    for extractor in &self.config.extractors {
        let task = self.execute_extractor(extractor);
        tasks.push(task);
    }
    
    let results = futures::future::join_all(tasks).await;
    // Aggregate results...
    Ok(AggregatedIR::new())
}
```

**TDD - Refactor (Optimización):**
- Implementar concurrency limits
- Añadir retry logic para extractores fallidos
- Optimizar buffer sizes para IR grandes

**Conventional Commit:**
`feat(extractor): implement ExtractorOrchestrator with Cap'n Proto protocol`

---

### US-10.02: Crear SARIF Adapter Extractor

**Como:** Usuario de Herramientas  
**Quiero:** Ver resultados de herramientas SARIF en hodei-scan  
**Para:** Aprovechar el ecosistema SARIF existente

**Criterios de Aceptación:**
- [ ] Lee SARIF JSON desde tool de terceros
- [ ] Parsea runs → facts (Vulnerability, CodeSmell, etc.)
- [ ] Mantiene metadata (ruleId, severity, locations)
- [ ] Genera IR válido Cap'n Proto
- [ ] Test con SARIF de Semgrep, CodeQL

**TDD - Red:**
```rust
#[test]
fn parse_sarif_to_ir() {
    let sarif_json = r#"{
        "runs": [{
            "results": [{
                "ruleId": "RUST/SEC001",
                "severity": "error",
                "message": {"text": "XSS vulnerability"}
            }]
        }]
    }"#;
    
    let ir = parse_sarif(sarif_json).unwrap();
    assert!(ir.facts.len() == 1);
    assert_eq!(ir.facts[0].fact_type, FactType::Vulnerability);
}
```

**TDD - Green:**
```rust
pub fn parse_sarif(sarif_json: &str) -> Result<IR, SarifError> {
    let sarif: SarifFile = serde_json::from_str(sarif_json)?;
    
    let mut facts = Vec::new();
    for run in &sarif.runs {
        for result in &run.results {
            facts.push(Fact {
                fact_type: FactType::Vulnerability { /* map fields */ },
                location: parse_location(&result.locations[0]),
                // ...
            });
        }
    }
    Ok(IR { facts })
}
```

**TDD - Refactor:**
- Batch processing para SARIFs grandes
- Streaming parse para evitar memory spikes
- Parallel fact processing

**Conventional Commit:**
`feat(extractor): add SARIF adapter for third-party tool integration`

---

### US-10.03: Implementar Ruff Adapter

**Como:** Desarrollador Python  
**Quiero:** Usar Ruff (super-fast linter) con hodei-scan  
**Para:** Aprovechar velocidad y reglas de Ruff

**Criterios de Aceptación:**
- [ ] Ejecuta `ruff check --format=json`
- [ ] Parsea output JSON a IR
- [ ] Mapea Ruff diagnostics → FactType::CodeSmell
- [ ] Soporte multi-file en paralelo
- [ ] Performance < 2x Ruff time

**TDD - Red:**
```rust
#[tokio::test]
async fn ruff_check_parses_correctly() {
    let ruff_output = r#"[{
        "filename": "test.py",
        "rule": "E501",
        "message": "Line too long",
        "line": 10
    }]"#;
    
    let ir = parse_ruff_json(ruff_output).unwrap();
    assert!(ir.facts[0].fact_type.is_code_smell());
}
```

**TDD - Green:**
```rust
pub async fn run_ruff_adapter(project_path: &Path) -> Result<IR, RuffError> {
    let output = Command::new("ruff")
        .arg("check")
        .arg("--format=json")
        .arg(project_path)
        .output()
        .await?;
    
    parse_ruff_json(&String::from_utf8_lossy(&output.stdout))
}
```

**TDD - Refactor:**
- Parallel file processing
- Incremental Ruff (only changed files)
- Cache para Ruff AST

**Conventional Commit:**
`feat(extractor): implement Ruff adapter for Python linting`

---

### US-10.04: Sistema de Timeouts y Resource Limits

**Como:** Plataforma  
**Quiero:** Controlar recursos de extractores externos  
**Para:** Prevenir ataques DoS y runaway processes

**Criterios de Aceptación:**
- [ ] Timeout por extractor (configurable)
- [ ] Kill process en timeout
- [ ] Memory limit por extractor
- [ ] CPU limit (nice/ionice)
- [ ] Concurrent extractor limit
- [ ] Graceful shutdown (SIGTERM → SIGKILL)

**TDD - Red:**
```rust
#[tokio::test]
async fn timeout_kills_extractor() {
    let extractor = ExtractorDef {
        name: "slow".to_string(),
        command: "sleep".to_string(),
        args: vec!["100".to_string()],
    };
    
    let orchestrator = ExtractorOrchestrator::new();
    let result = orchestrator.execute_with_timeout(&extractor, Duration::from_millis(100)).await;
    
    assert!(matches!(result, Err(OrchestratorError::Timeout)));
}
```

**TDD - Green:**
```rust
pub async fn execute_with_timeout(
    &self,
    extractor: &ExtractorDef,
    timeout: Duration,
) -> Result<IR, OrchestratorError> {
    let mut child = Command::new(&extractor.command)
        .spawn()
        .map_err(OrchestratorError::SpawnFailed)?;
    
    match tokio::time::timeout(timeout, child.wait()).await {
        Ok(Ok(_)) => self.read_ir_from_stdout(&mut child),
        Ok(Err(e)) => Err(OrchestratorError::ProcessFailed(e)),
        Err(_) => {
            child.kill().await.map_err(OrchestratorError::KillFailed)?;
            Err(OrchestratorError::Timeout)
        }
    }
}
```

**TDD - Refactor:**
- Linux cgroups integration para hard limits
- Process priority management
- Resource usage monitoring

**Conventional Commit:**
`feat(extractor): add timeout and resource limit controls`

---

### US-10.05: Configuración hodei.toml

**Como:** Usuario  
**Quiero:** Definir extractores en hodei.toml  
**Para:** Configurar análisis personalizado

**Criterios de Aceptación:**
- [ ] Toml format documentado
- [ ] Validación de configuración
- [ ] Soporte para múltiples extractores
- [ ] Override per-project settings
- [ ] Config inheritance (global → project)

**Formato hodei.toml:**
```toml
[extractors]
enabled = true
max_concurrent = 4

[[extractors.def]]
name = "ruff"
command = "ruff-to-hodei"
args = ["--config", "ruff.toml"]
timeout = "30s"

[[extractors.def]]
name = "sarif"
command = "sarif-to-hodei"
source = "semgrep"
timeout = "60s"
```

**TDD - Red:**
```rust
#[test]
fn parse_hodei_toml() {
    let toml = r#"
[extractors]
max_concurrent = 4

[[extractors.def]]
name = "ruff"
command = "ruff-to-hodei"
"#;
    
    let config = parse_hodei_toml(toml).unwrap();
    assert_eq!(config.extractors.max_concurrent, 4);
}
```

**TDD - Green:**
```rust
#[derive(Deserialize)]
struct HodeiConfig {
    extractors: ExtractorConfig,
}

pub fn parse_hodei_toml(toml: &str) -> Result<HodeiConfig, ConfigError> {
    let config: HodeiConfig = toml::from_str(toml)?;
    validate_config(&config)?;
    Ok(config)
}
```

**Conventional Commit:**
`feat(config): add hodei.toml configuration for extractors`

---

## 6. Testing Strategy

### 6.1. Unit Tests
- Mock extractor processes
- Protocol parsing
- IR aggregation logic
- Timeout handling

### 6.2. Integration Tests
- Test with real tools (Ruff, ESLint)
- SARIF round-trip (tool → hodei → JSON)
- Performance benchmarks

### 6.3. E2E Tests
- Full pipeline: hodei.toml → extractors → IR → RuleEngine → Findings
- Multi-tool scenarios (Ruff + SARIF)
- Failure scenarios (timeout, crash, invalid IR)

---

## 7. Riesgos y Mitigaciones

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| Malicious extractor | Alto | Bajo | Sandbox + resource limits |
| Protocol version mismatch | Medio | Alto | Version field in messages |
| Extractor stdout blocking | Alto | Medio | Async IO + buffer tuning |
| IR schema evolution | Medio | Medio | Backward compatible Cap'n Proto |

---

## 8. Definition of Done

- [ ] ExtractorOrchestrator implementado y testeado
- [ ] SARIF adapter funcional con ≥3 herramientas
- [ ] Ruff adapter con performance <2x Ruff
- [ ] hodei.toml configuration completa
- [ ] Timeouts y resource limits funcionando
- [ ] Documentación de integración
- [ ] Benchmarks: 100 files <5s with Ruff
- [ ] Tests: >90% coverage
- [ ] Security review passed

---

**Estimación Total**: 4 Sprints (8 semanas)  
**Commit Messages**:  
- `feat(extractor): implement core orchestrator with Cap'n Proto`  
- `feat(extractor): add SARIF adapter`  
- `feat(extractor): add Ruff adapter`  
- `feat(extractor): implement timeout and resource limits`  
- `feat(config): add hodei.toml configuration`

---

**Referencias Técnicas:**
- Cap'n Proto: https://capnproto.org/
- Tokio async: https://tokio.rs/
- Tree-sitter: https://tree-sitter.github.io/
- Ruff: https://github.com/astral-sh/ruff
