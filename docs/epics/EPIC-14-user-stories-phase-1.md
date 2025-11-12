# Historias de Usuario: EPIC-14 - Extractores Fase 1

**Épica Relacionada:** EPIC-14 - Ecosistema de Extractores - Fase 1 (Adaptadores)  
**Versión:** 1.0.0  
**Fecha:** 2025-11-12  
**Sprint:** Q1 2025

---

## 📋 Índice de Historias de Usuario

1. [US-14.1: Infraestructura Core de Orquestación](#us-141)
2. [US-14.2: Extractor Universal SARIF](#us-142)
3. [US-14.3: Adaptador Ruff (Python)](#us-143)
4. [US-14.4: Adaptador ESLint (JavaScript/TypeScript)](#us-144)
5. [US-14.5: Adaptador Clippy (Rust)](#us-145)
6. [US-14.6: Adaptador staticcheck (Go)](#us-146)
7. [US-14.7: Sistema de Deduplicación Inteligente](#us-147)

---

<a name="us-141"></a>
## US-14.1: Infraestructura Core de Orquestación

### Descripción

**Como** desarrollador del core  
**Quiero** un orquestador que ejecute extractores en paralelo y agregue sus IRs  
**Para** poder integrar múltiples herramientas sin acoplamiento

### Contexto Técnico

El orquestador es el componente central que coordina la ejecución de múltiples extractores independientes. Debe ser robusto, eficiente y capaz de manejar fallos aislados sin comprometer todo el análisis.

### Criterios de Aceptación

#### Funcionales

- ✅ **CA-1.1**: Lee configuración de extractores desde `hodei.toml`
  - Parsea sección `[[extractors]]` con validación de esquema
  - Soporta habilitación/deshabilitación por extractor
  - Valida existencia de comando antes de ejecutar

- ✅ **CA-1.2**: Ejecuta extractores como procesos hijos
  - Usa `tokio::process::Command` para ejecución async
  - Pasa configuración via stdin en formato JSON
  - Captura stdout (IR) y stderr (logs) independientemente

- ✅ **CA-1.3**: Implementa timeout configurable por extractor
  - Default: 300 segundos (5 minutos)
  - Configurable en `hodei.toml`: `timeout_seconds = 600`
  - Mata proceso si excede timeout
  - Log claro: "Extractor X excedió timeout de Y segundos"

- ✅ **CA-1.4**: Maneja fallos gracefully
  - Si extractor falla, continúa con otros extractores
  - Registra error con contexto (stderr del extractor)
  - Genera reporte de extractores exitosos/fallidos
  - No retorna error global si >=1 extractor tuvo éxito

- ✅ **CA-1.5**: Valida cada IR contra esquema Cap'n Proto
  - Verifica estructura Cap'n Proto válida
  - Valida tipos de hechos conocidos
  - Rechaza IR con errores críticos de schema
  - Genera warnings para campos opcionales malformados

- ✅ **CA-1.6**: Fusiona IRs eliminando duplicados
  - Combina hechos de todos los extractores
  - Aplica deduplicación por fingerprint (US-14.7)
  - Preserva metadata de provenance de cada extractor
  - Mantiene orden temporal de extracción

- ✅ **CA-1.7**: Genera métricas de ejecución
  - Timing por extractor (inicio, fin, duración)
  - Conteo de hechos por extractor
  - Estadísticas de deduplicación
  - Export a formato JSON para observabilidad

#### No Funcionales

- ✅ **NFR-1.1**: Rendimiento
  - Overhead del orquestador: <100ms
  - Ejecución paralela real (no secuencial)
  - Memory footprint: <50MB base (sin IRs)

- ✅ **NFR-1.2**: Confiabilidad
  - No panic en código del orquestador
  - Cleanup de procesos huérfanos en SIGTERM
  - Idempotencia: misma config → mismo resultado

- ✅ **NFR-1.3**: Observabilidad
  - Logs estructurados (JSON) con niveles
  - Tracing de spans por extractor
  - Métricas exportables a Prometheus

### Diseño Técnico

#### Esquema de Configuración

```toml
# hodei.toml - Sección de extractores

[orchestrator]
# Configuración global del orquestador
parallel_execution = true
max_parallel_extractors = 4
global_timeout_seconds = 1800  # 30 minutos para todo el análisis

[[extractors]]
id = "sarif-universal"
command = "hodei-extract-sarif"
enabled = true
timeout_seconds = 300

[extractors.config]
sarif_files = ["results/**/*.sarif"]

[[extractors]]
id = "ruff-python"
command = "hodei-extract-ruff"
enabled = true
timeout_seconds = 120

[extractors.config]
select = ["E", "F", "B", "S"]
```

#### Estructura de Datos

```rust
// hodei-engine/src/orchestration/mod.rs

use tokio::process::Command;
use tokio::sync::Semaphore;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Deserialize)]
pub struct OrchestratorConfig {
    pub parallel_execution: bool,
    pub max_parallel_extractors: usize,
    pub global_timeout_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExtractorDefinition {
    pub id: String,
    pub command: String,
    pub enabled: bool,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    pub config: serde_json::Value,
}

fn default_timeout() -> u64 { 300 }

#[derive(Debug)]
pub struct ExtractorRun {
    pub id: String,
    pub success: bool,
    pub duration: Duration,
    pub facts_extracted: usize,
    pub error: Option<String>,
}

pub struct ExtractorOrchestrator {
    config: OrchestratorConfig,
    extractors: Vec<ExtractorDefinition>,
    semaphore: Arc<Semaphore>,
}

impl ExtractorOrchestrator {
    pub fn new(config: OrchestratorConfig, extractors: Vec<ExtractorDefinition>) -> Self {
        let max_parallel = if config.parallel_execution {
            config.max_parallel_extractors
        } else {
            1
        };
        
        Self {
            config,
            extractors,
            semaphore: Arc::new(Semaphore::new(max_parallel)),
        }
    }
    
    /// Ejecuta todos los extractores habilitados
    pub async fn run_all(&self, project_path: &Path) -> Result<AggregatedIR> {
        let enabled_extractors: Vec<_> = self.extractors.iter()
            .filter(|e| e.enabled)
            .collect();
        
        info!("Ejecutando {} extractores", enabled_extractors.len());
        
        // Ejecuta extractores en paralelo
        let mut handles = Vec::new();
        
        for extractor in enabled_extractors {
            let semaphore = Arc::clone(&self.semaphore);
            let extractor = extractor.clone();
            let project_path = project_path.to_owned();
            
            let handle = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                Self::run_extractor(&extractor, &project_path).await
            });
            
            handles.push(handle);
        }
        
        // Espera a que todos terminen
        let results = futures::future::join_all(handles).await;
        
        // Procesa resultados
        self.aggregate_results(results).await
    }
    
    /// Ejecuta un extractor individual
    async fn run_extractor(
        extractor: &ExtractorDefinition,
        project_path: &Path,
    ) -> Result<(ExtractorRun, Option<IntermediateRepresentation>)> {
        let start = Instant::now();
        
        info!("Iniciando extractor: {}", extractor.id);
        
        // Prepara configuración de entrada
        let input_config = serde_json::json!({
            "project_path": project_path,
            "config": extractor.config,
        });
        
        // Ejecuta comando con timeout
        let timeout_duration = Duration::from_secs(extractor.timeout_seconds);
        
        let result = tokio::time::timeout(
            timeout_duration,
            Self::execute_extractor_command(&extractor.command, &input_config),
        ).await;
        
        let duration = start.elapsed();
        
        match result {
            Ok(Ok((stdout, stderr))) => {
                // Parsea IR desde stdout
                match Self::parse_ir_from_bytes(&stdout) {
                    Ok(ir) => {
                        let facts_count = ir.facts.len();
                        
                        info!(
                            "Extractor {} completado: {} hechos en {:.2}s",
                            extractor.id, facts_count, duration.as_secs_f64()
                        );
                        
                        Ok((
                            ExtractorRun {
                                id: extractor.id.clone(),
                                success: true,
                                duration,
                                facts_extracted: facts_count,
                                error: None,
                            },
                            Some(ir),
                        ))
                    }
                    Err(e) => {
                        error!(
                            "Extractor {} falló al parsear IR: {}",
                            extractor.id, e
                        );
                        
                        Ok((
                            ExtractorRun {
                                id: extractor.id.clone(),
                                success: false,
                                duration,
                                facts_extracted: 0,
                                error: Some(format!("Failed to parse IR: {}", e)),
                            },
                            None,
                        ))
                    }
                }
            }
            Ok(Err(e)) => {
                error!("Extractor {} falló: {}", extractor.id, e);
                
                Ok((
                    ExtractorRun {
                        id: extractor.id.clone(),
                        success: false,
                        duration,
                        facts_extracted: 0,
                        error: Some(e.to_string()),
                    },
                    None,
                ))
            }
            Err(_timeout) => {
                error!(
                    "Extractor {} excedió timeout de {}s",
                    extractor.id, extractor.timeout_seconds
                );
                
                Ok((
                    ExtractorRun {
                        id: extractor.id.clone(),
                        success: false,
                        duration,
                        facts_extracted: 0,
                        error: Some(format!(
                            "Timeout after {} seconds",
                            extractor.timeout_seconds
                        )),
                    },
                    None,
                ))
            }
        }
    }
    
    /// Ejecuta comando del extractor
    async fn execute_extractor_command(
        command: &str,
        input_config: &serde_json::Value,
    ) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut child = Command::new(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                OrchestratorError::ExtractorSpawnFailed {
                    command: command.to_string(),
                    error: e.to_string(),
                }
            })?;
        
        // Escribe config a stdin
        if let Some(mut stdin) = child.stdin.take() {
            let config_json = serde_json::to_vec(input_config)?;
            stdin.write_all(&config_json).await?;
            drop(stdin); // Cierra stdin
        }
        
        // Espera a que termine
        let output = child.wait_with_output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(OrchestratorError::ExtractorFailed {
                exit_code: output.status.code(),
                stderr: stderr.into(),
            });
        }
        
        Ok((output.stdout, output.stderr))
    }
    
    /// Parsea IR desde bytes Cap'n Proto
    fn parse_ir_from_bytes(bytes: &[u8]) -> Result<IntermediateRepresentation> {
        // TODO: Implementar parsing Cap'n Proto
        // Por ahora, placeholder
        unimplemented!("Cap'n Proto parsing")
    }
    
    /// Agrega resultados de todos los extractores
    async fn aggregate_results(
        &self,
        results: Vec<Result<Result<(ExtractorRun, Option<IntermediateRepresentation>)>>>,
    ) -> Result<AggregatedIR> {
        let mut all_facts = Vec::new();
        let mut extractor_runs = Vec::new();
        
        for result in results {
            match result {
                Ok(Ok((run, Some(ir)))) => {
                    extractor_runs.push(run);
                    all_facts.extend(ir.facts);
                }
                Ok(Ok((run, None))) => {
                    extractor_runs.push(run);
                }
                Ok(Err(e)) => {
                    error!("Error ejecutando extractor: {}", e);
                }
                Err(e) => {
                    error!("Error en tokio task: {}", e);
                }
            }
        }
        
        // Verifica que al menos un extractor tuvo éxito
        let successful_count = extractor_runs.iter()
            .filter(|r| r.success)
            .count();
        
        if successful_count == 0 {
            return Err(OrchestratorError::AllExtractorsFailed);
        }
        
        info!(
            "Agregando {} hechos de {} extractores exitosos",
            all_facts.len(),
            successful_count
        );
        
        // Deduplica hechos
        let deduplicator = FactDeduplicator::new();
        let deduplicated_facts = deduplicator.deduplicate(all_facts);
        
        let dedup_ratio = 1.0 - (deduplicated_facts.len() as f64 / all_facts.len() as f64);
        info!(
            "Deduplicación: {} → {} hechos ({:.1}% reducción)",
            all_facts.len(),
            deduplicated_facts.len(),
            dedup_ratio * 100.0
        );
        
        Ok(AggregatedIR {
            facts: deduplicated_facts,
            metadata: AggregationMetadata {
                extractor_runs,
                total_facts_before_dedup: all_facts.len(),
                total_facts_after_dedup: deduplicated_facts.len(),
                deduplication_ratio: dedup_ratio,
            },
        })
    }
}

#[derive(Debug)]
pub struct AggregatedIR {
    pub facts: Vec<Fact>,
    pub metadata: AggregationMetadata,
}

#[derive(Debug)]
pub struct AggregationMetadata {
    pub extractor_runs: Vec<ExtractorRun>,
    pub total_facts_before_dedup: usize,
    pub total_facts_after_dedup: usize,
    pub deduplication_ratio: f64,
}

#[derive(Debug, thiserror::Error)]
pub enum OrchestratorError {
    #[error("Failed to spawn extractor '{command}': {error}")]
    ExtractorSpawnFailed { command: String, error: String },
    
    #[error("Extractor failed with exit code {exit_code:?}: {stderr}")]
    ExtractorFailed {
        exit_code: Option<i32>,
        stderr: String,
    },
    
    #[error("All extractors failed, cannot proceed")]
    AllExtractorsFailed,
}
```

### Casos de Prueba

#### Test 1: Ejecución Paralela Exitosa

```rust
#[tokio::test]
async fn test_parallel_execution_success() {
    let config = OrchestratorConfig {
        parallel_execution: true,
        max_parallel_extractors: 3,
        global_timeout_seconds: 600,
    };
    
    let extractors = vec![
        ExtractorDefinition {
            id: "mock-extractor-1".into(),
            command: "tests/fixtures/mock_extractor.sh".into(),
            enabled: true,
            timeout_seconds: 10,
            config: json!({"delay_ms": 100}),
        },
        ExtractorDefinition {
            id: "mock-extractor-2".into(),
            command: "tests/fixtures/mock_extractor.sh".into(),
            enabled: true,
            timeout_seconds: 10,
            config: json!({"delay_ms": 150}),
        },
        ExtractorDefinition {
            id: "mock-extractor-3".into(),
            command: "tests/fixtures/mock_extractor.sh".into(),
            enabled: true,
            timeout_seconds: 10,
            config: json!({"delay_ms": 200}),
        },
    ];
    
    let orchestrator = ExtractorOrchestrator::new(config, extractors);
    let project_path = Path::new("tests/fixtures/sample-project");
    
    let start = Instant::now();
    let result = orchestrator.run_all(project_path).await.unwrap();
    let duration = start.elapsed();
    
    // Verifica que todos los extractores ejecutaron
    assert_eq!(result.metadata.extractor_runs.len(), 3);
    assert!(result.metadata.extractor_runs.iter().all(|r| r.success));
    
    // Verifica ejecución paralela (no debe sumar delays)
    assert!(duration < Duration::from_millis(500));
    
    // Verifica que se extrajeron hechos
    assert!(result.facts.len() > 0);
}
```

#### Test 2: Manejo de Fallos Aislados

```rust
#[tokio::test]
async fn test_partial_failure_continues() {
    let extractors = vec![
        ExtractorDefinition {
            id: "success-1".into(),
            command: "tests/fixtures/mock_extractor.sh".into(),
            enabled: true,
            timeout_seconds: 10,
            config: json!({}),
        },
        ExtractorDefinition {
            id: "failure".into(),
            command: "tests/fixtures/failing_extractor.sh".into(),
            enabled: true,
            timeout_seconds: 10,
            config: json!({}),
        },
        ExtractorDefinition {
            id: "success-2".into(),
            command: "tests/fixtures/mock_extractor.sh".into(),
            enabled: true,
            timeout_seconds: 10,
            config: json!({}),
        },
    ];
    
    let orchestrator = ExtractorOrchestrator::new(
        OrchestratorConfig::default(),
        extractors,
    );
    
    let result = orchestrator.run_all(Path::new("dummy")).await.unwrap();
    
    // Verifica que 2 extractores tuvieron éxito
    let successful = result.metadata.extractor_runs.iter()
        .filter(|r| r.success)
        .count();
    assert_eq!(successful, 2);
    
    // Verifica que el fallido está registrado
    let failed = result.metadata.extractor_runs.iter()
        .find(|r| r.id == "failure")
        .unwrap();
    assert!(!failed.success);
    assert!(failed.error.is_some());
}
```

#### Test 3: Timeout de Extractor

```rust
#[tokio::test]
async fn test_extractor_timeout() {
    let extractors = vec![
        ExtractorDefinition {
            id: "slow-extractor".into(),
            command: "tests/fixtures/slow_extractor.sh".into(),
            enabled: true,
            timeout_seconds: 2,  // 2 segundos
            config: json!({"sleep_seconds": 10}),  // Duerme 10 segundos
        },
    ];
    
    let orchestrator = ExtractorOrchestrator::new(
        OrchestratorConfig::default(),
        extractors,
    );
    
    let start = Instant::now();
    let result = orchestrator.run_all(Path::new("dummy")).await.unwrap();
    let duration = start.elapsed();
    
    // Verifica que terminó rápido (no esperó los 10 segundos)
    assert!(duration < Duration::from_secs(3));
    
    // Verifica que el extractor falló por timeout
    let run = &result.metadata.extractor_runs[0];
    assert!(!run.success);
    assert!(run.error.as_ref().unwrap().contains("Timeout"));
}
```

#### Test 4: Validación de Esquema IR

```rust
#[tokio::test]
async fn test_invalid_ir_rejected() {
    let extractors = vec![
        ExtractorDefinition {
            id: "invalid-ir".into(),
            command: "tests/fixtures/invalid_ir_extractor.sh".into(),
            enabled: true,
            timeout_seconds: 10,
            config: json!({}),
        },
    ];
    
    let orchestrator = ExtractorOrchestrator::new(
        OrchestratorConfig::default(),
        extractors,
    );
    
    let result = orchestrator.run_all(Path::new("dummy")).await.unwrap();
    
    // Verifica que el extractor falló por IR inválido
    let run = &result.metadata.extractor_runs[0];
    assert!(!run.success);
    assert!(run.error.as_ref().unwrap().contains("Failed to parse IR"));
}
```

### Estimación

**Story Points**: 5  
**Esfuerzo**: 5-7 días  
**Desarrolladores**: 1 senior

**Desglose**:

- Diseño de arquitectura: 1 día
- Implementación core: 2 días
- Manejo de errores y timeouts: 1 día
- Tests unitarios e integración: 1-2 días
- Documentación: 0.5 días

### Dependencias

**Bloqueantes**: Ninguna (es el primero de la épica)

**Bloquea a**:

- US-14.2 (Extractor SARIF)
- US-14.3 (Extractor Ruff)
- US-14.4 (Extractor ESLint)
- US-14.5 (Extractor Clippy)
- US-14.6 (Extractor staticcheck)

### Riesgos

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|--------------|---------|------------|
| Procesos zombie por mal cleanup | Media | Alto | Registrar señales SIGTERM, implementar Drop trait |
| Deadlock en semáforo | Baja | Alto | Tests exhaustivos de concurrencia |
| Memory leak en IRs grandes | Media | Medio | Streaming parsing, límites de memoria |

---

<a name="us-142"></a>
## US-14.2: Extractor Universal SARIF

### Descripción

**Como** usuario de GitHub Advanced Security  
**Quiero** importar mis reportes SARIF directamente a hodei-scan  
**Para** unificar mi análisis de seguridad

### Contexto Técnico

SARIF (Static Analysis Results Interchange Format) es el formato estándar OASIS 2.1.0 adoptado por la industria. Este extractor es **crítico** porque proporciona compatibilidad inmediata con docenas de herramientas sin necesidad de adaptadores específicos.

### Criterios de Aceptación

#### Funcionales

- ✅ **CA-2.1**: Parsea ficheros SARIF 2.1.0 válidos
  - Soporta schema completo de SARIF 2.1.0
  - Valida ficheros contra schema JSON oficial
  - Genera error descriptivo si versión no soportada

- ✅ **CA-2.2**: Mapea correctamente todos los campos clave a IR
  - `ruleId` → `Fact::rule_id`
  - `level` → `Fact::severity` (error=HIGH, warning=MEDIUM, note=LOW)
  - `message.text` → `Fact::message`
  - `physicalLocation` → `Fact::location`
  - `properties.security-severity` → `Fact::confidence_score`
  - `properties.tags` → `Fact::categories`

- ✅ **CA-2.3**: Soporta múltiples runs en un fichero SARIF
  - Itera sobre array `runs` correctamente
  - Combina resultados de todos los runs
  - Preserva metadata de cada run (tool, version)

- ✅ **CA-2.4**: Extrae security-severity cuando está presente
  - Lee `properties.security-severity` (escala 0.0-10.0)
  - Normaliza a escala interna 0.0-1.0
  - Default 0.5 si no presente

- ✅ **CA-2.5**: Maneja gracefully campos opcionales ausentes
  - No falla si faltan campos opcionales
  - Usa valores default razonables
  - Registra warnings para metadata incompleta

- ✅ **CA-2.6**: Genera warnings para reglas sin metadata completa
  - Log warning si falta `shortDescription`
  - Log warning si falta `helpUri`
  - Continúa procesamiento

- ✅ **CA-2.7**: Throughput >10K resultados/segundo
  - Parsing eficiente con serde
  - Sin clonaciones innecesarias
  - Benchmark con fixture de 100K results

#### No Funcionales

- ✅ **NFR-2.1**: Compatibilidad
  - Soporta SARIF 2.1.0 y 2.0.0 (con warnings)
  - Testado con outputs de: GitHub CodeQL, ESLint, Semgrep, Checkmarx

- ✅ **NFR-2.2**: Robustez
  - No panic en ficheros malformados
  - Errores descriptivos con contexto
  - Validación strict de tipos

### Diseño Técnico

#### Esquema de Configuración

```toml
[[extractors]]
id = "sarif-universal"
command = "hodei-extract-sarif"
enabled = true
timeout_seconds = 120

[extractors.config]
# Glob patterns para ficheros SARIF
sarif_files = [
    "results/**/*.sarif",
    ".sarif/**/*.sarif",
]

# Filtros opcionales
exclude_rules = [
    "style/*",
    "deprecated/*",
]

min_severity = "warning"  # note, warning, error

# Mapeo custom de niveles (opcional)
[extractors.config.severity_mapping]
note = "low"
warning = "medium"
error = "high"
```

#### Implementación

```rust
// hodei-extractors/src/sarif/mod.rs

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use glob::glob;

#[derive(Debug, Deserialize)]
pub struct SarifConfig {
    pub sarif_files: Vec<String>,
    #[serde(default)]
    pub exclude_rules: Vec<String>,
    pub min_severity: Option<String>,
}

pub struct SarifExtractor {
    config: SarifConfig,
}

impl Extractor for SarifExtractor {
    fn extract(&self, input: ExtractorConfig) -> Result<IntermediateRepresentation> {
        let mut ir_builder = IRBuilder::new();
        
        // Descubre ficheros SARIF
        let sarif_paths = self.discover_sarif_files(&input.project_path)?;
        
        info!("Encontrados {} ficheros SARIF", sarif_paths.len());
        
        for path in sarif_paths {
            match self.process_sarif_file(&path, &mut ir_builder) {
                Ok(count) => {
                    info!("Procesados {} resultados de {}", count, path.display());
                }
                Err(e) => {
                    warn!("Error procesando {}: {}", path.display(), e);
                }
            }
        }
        
        Ok(ir_builder.build())
    }
    
    fn metadata(&self) -> ExtractorMetadata {
        ExtractorMetadata {
            id: "sarif-universal".into(),
            name: "SARIF Universal Importer".into(),
            version: semver::Version::new(1, 0, 0),
            supported_languages: vec!["*".into()],
            capabilities: ExtractorCapabilities {
                provides_facts: vec![
                    FactType::Vulnerability,
                    FactType::CodeSmell,
                    FactType::Bug,
                ],
                requires_source_code: false,
            },
        }
    }
}

impl SarifExtractor {
    fn discover_sarif_files(&self, project_path: &Path) -> Result<Vec<PathBuf>> {
        let mut paths = Vec::new();
        
        for pattern in &self.config.sarif_files {
            let full_pattern = project_path.join(pattern);
            
            for entry in glob(full_pattern.to_str().unwrap())? {
                match entry {
                    Ok(path) => paths.push(path),
                    Err(e) => warn!("Error en glob: {}", e),
                }
            }
        }
        
        Ok(paths)
    }
    
    fn process_sarif_file(
        &self,
        path: &Path,
        ir: &mut IRBuilder,
    ) -> Result<usize> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        
        let sarif: SarifReport = serde_json::from_reader(reader)
            .map_err(|e| SarifError::ParseError {
                file: path.to_owned(),
                error: e.to_string(),
            })?;
        
        // Valida versión
        if sarif.version != "2.1.0" && sarif.version != "2.0.0" {
            warn!(
                "SARIF version {} no soportada oficialmente (esperado 2.1.0)",
                sarif.version
            );
        }
        
        let mut total_results = 0;
        
        for run in sarif.runs {
            let tool_name = &run.tool.driver.name;
            let tool_version = run.tool.driver.version.as_deref().unwrap_or("unknown");
            
            for result in run.results {
                // Aplica filtros
                if self.should_exclude_result(&result) {
                    continue;
                }
                
                match self.sarif_result_to_fact(&result, tool_name, tool_version) {
                    Ok(fact) => {
                        ir.add_fact(fact);
                        total_results += 1;
                    }
                    Err(e) => {
                        warn!("Error convirtiendo resultado SARIF: {}", e);
                    }
                }
            }
        }
        
        Ok(total_results)
    }
    
    fn should_exclude_result(&self, result: &SarifResult) -> bool {
        // Filtro por regla excluida
        if let Some(rule_id) = &result.rule_id {
            for pattern in &self.config.exclude_rules {
                if Self::matches_glob(rule_id, pattern) {
                    return true;
                }
            }
        }
        
        // Filtro por severidad mínima
        if let Some(min_sev) = &self.config.min_severity {
            let result_sev = Self::sarif_level_to_severity(&result.level);
            let min_sev_level = Self::string_to_severity(min_sev);
            
            if result_sev < min_sev_level {
                return true;
            }
        }
        
        false
    }
    
    fn sarif_result_to_fact(
        &self,
        result: &SarifResult,
        tool: &str,
        version: &str,
    ) -> Result<Fact> {
        // Extrae ubicación
        let location = if let Some(loc) = result.locations.first() {
            self.extract_location(loc)?
        } else {
            return Err(SarifError::MissingLocation);
        };
        
        // Mapea severidad
        let severity = Self::sarif_level_to_severity(&result.level);
        
        // Determina tipo de hecho
        let fact_type = if Self::is_security_result(result) {
            let security_severity = result.properties
                .as_ref()
                .and_then(|p| p.get("security-severity"))
                .and_then(|v| v.as_f64())
                .unwrap_or(5.0) / 10.0;  // Normaliza 0-10 a 0-1
            
            FactType::Vulnerability(VulnerabilityFact {
                cwe_ids: Self::extract_cwe_ids(result),
                security_severity,
            })
        } else {
            FactType::CodeSmell(CodeSmellFact {
                smell_type: result.rule_id.clone().unwrap_or_default(),
            })
        };
        
        Ok(Fact {
            id: FactId::generate(),
            fact_type,
            location,
            message: result.message.text.clone(),
            severity,
            provenance: Provenance {
                extractor_id: format!("sarif-{}", tool),
                extractor_version: version.into(),
                extracted_at: SystemTime::now(),
                source_file: result.rule_id.clone(),
            },
            metadata: self.extract_metadata(result),
        })
    }
    
    fn extract_location(&self, location: &SarifLocation) -> Result<Location> {
        let phys_loc = &location.physical_location;
        let artifact = &phys_loc.artifact_location;
        
        let file = ProjectPath::new(&artifact.uri)?;
        
        let region = &phys_loc.region;
        let span = Span {
            start: Position {
                line: region.start_line.unwrap_or(1) as u32,
                column: region.start_column.unwrap_or(1) as u32,
            },
            end: Position {
                line: region.end_line.unwrap_or(region.start_line.unwrap_or(1)) as u32,
                column: region.end_column.unwrap_or(region.start_column.unwrap_or(1) + 1) as u32,
            },
        };
        
        Ok(Location { file, span })
    }
    
    fn sarif_level_to_severity(level: &str) -> Severity {
        match level {
            "error" => Severity::High,
            "warning" => Severity::Medium,
            "note" | "none" => Severity::Low,
            _ => {
                warn!("SARIF level desconocido: {}, usando Medium", level);
                Severity::Medium
            }
        }
    }
    
    fn is_security_result(result: &SarifResult) -> bool {
        // Verifica si tiene security-severity
        if let Some(props) = &result.properties {
            if props.contains_key("security-severity") {
                return true;
            }
            
            // O tags de seguridad
            if let Some(tags) = props.get("tags").and_then(|v| v.as_array()) {
                return tags.iter().any(|t| {
                    t.as_str()
                        .map(|s| s.contains("security") || s.contains("vulnerability"))
                        .unwrap_or(false)
                });
            }
        }
        
        false
    }
    
    fn extract_cwe_ids(result: &SarifResult) -> Vec<u32> {
        let mut cwes = Vec::new();
        
        if let Some(props) = &result.properties {
            // Busca en properties.cwe
            if let Some(cwe_val) = props.get("cwe") {
                if let Some(cwe_str) = cwe_val.as_str() {
                    if let Ok(cwe) = cwe_str.trim_start_matches("CWE-").parse::<u32>() {
                        cwes.push(cwe);
                    }
                }
            }
            
            // Busca en tags
            if let Some(tags) = props.get("tags").and_then(|v| v.as_array()) {
                for tag in tags {
                    if let Some(tag_str) = tag.as_str() {
                        if tag_str.starts_with("CWE-") {
                            if let Ok(cwe) = tag_str[4..].parse::<u32>() {
                                cwes.push(cwe);
                            }
                        }
                    }
                }
            }
        }
        
        cwes
    }
    
    fn extract_metadata(&self, result: &SarifResult) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();
        
        if let Some(rule_id) = &result.rule_id {
            metadata.insert("sarif_rule_id".into(), json!(rule_id));
        }
        
        if let Some(props) = &result.properties {
            // Copia properties seleccionadas
            for key in ["precision", "tags", "problem.severity"] {
                if let Some(value) = props.get(key) {
                    metadata.insert(key.to_string(), value.clone());
                }
            }
        }
        
        metadata
    }
}

// Estructuras SARIF (parciales, solo campos usados)
#[derive(Debug, Deserialize)]
struct SarifReport {
    version: String,
    runs: Vec<SarifRun>,
}

#[derive(Debug, Deserialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Debug, Deserialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Debug, Deserialize)]
struct SarifDriver {
    name: String,
    version: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SarifResult {
    rule_id: Option<String>,
    level: String,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
    properties: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct SarifMessage {
    text: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SarifLocation {
    physical_location: SarifPhysicalLocation,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SarifPhysicalLocation {
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Debug, Deserialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SarifRegion {
    start_line: Option<usize>,
    start_column: Option<usize>,
    end_line: Option<usize>,
    end_column: Option<usize>,
}
```

### Casos de Prueba

#### Test 1: Parseo de SARIF de GitHub CodeQL

```rust
#[test]
fn test_parse_github_codeql_sarif() {
    let extractor = SarifExtractor::new(SarifConfig::default());
    
    let sarif_content = r#"
    {
      "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
      "version": "2.1.0",
      "runs": [{
        "tool": {
          "driver": {
            "name": "CodeQL",
            "version": "2.12.0"
          }
        },
        "results": [{
          "ruleId": "js/sql-injection",
          "level": "error",
          "message": {"text": "SQL injection vulnerability"},
          "locations": [{
            "physicalLocation": {
              "artifactLocation": {"uri": "src/db.js"},
              "region": {
                "startLine": 42,
                "startColumn": 10,
                "endLine": 42,
                "endColumn": 25
              }
            }
          }],
          "properties": {
            "security-severity": "9.0",
            "tags": ["security", "external/cwe/cwe-89"]
          }
        }]
      }]
    }
    "#;
    
    let temp_file = NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), sarif_content).unwrap();
    
    let config = ExtractorConfig {
        project_path: temp_file.path().parent().unwrap().to_owned(),
        extractor_settings: json!({
            "sarif_files": [temp_file.path().to_str().unwrap()]
        }),
        file_filters: FileFilters::default(),
    };
    
    let ir = extractor.extract(config).unwrap();
    
    assert_eq!(ir.facts.len(), 1);
    
    let fact = &ir.facts[0];
    assert!(matches!(fact.fact_type, FactType::Vulnerability(_)));
    assert_eq!(fact.severity, Severity::High);
    assert_eq!(fact.location.file.as_str(), "src/db.js");
    assert_eq!(fact.location.span.start.line, 42);
    
    if let FactType::Vulnerability(vuln) = &fact.fact_type {
        assert_eq!(vuln.security_severity, 0.9);  // 9.0 / 10.0
        assert!(vuln.cwe_ids.contains(&89));
    }
}
```

#### Test 2: Múltiples Runs

```rust
#[test]
fn test_multiple_runs() {
    // SARIF con 2 runs de herramientas distintas
    let sarif_content = json!({
        "version": "2.1.0",
        "runs": [
            {
                "tool": {"driver": {"name": "ESLint", "version": "8.0"}},
                "results": [
                    /* resultado 1 */
                ]
            },
            {
                "tool": {"driver": {"name": "Semgrep", "version": "1.0"}},
                "results": [
                    /* resultado 2 */
                ]
            }
        ]
    });
    
    // Test que ambos runs se procesan
    // ...
}
```

#### Test 3: Filtros

```rust
#[test]
fn test_exclude_rules() {
    let config = SarifConfig {
        sarif_files: vec!["test.sarif".into()],
        exclude_rules: vec!["style/*".into()],
        min_severity: Some("warning".into()),
    };
    
    let extractor = SarifExtractor::new(config);
    
    // Test que reglas style/* no aparecen
    // Test que notas (severity:note) no aparecen
    // ...
}
```

#### Test 4: Rendimiento

```rust
#[test]
fn test_performance_10k_results() {
    // Genera SARIF con 10K resultados
    let sarif = generate_large_sarif(10_000);
    
    let start = Instant::now();
    let ir = extractor.extract(config).unwrap();
    let duration = start.elapsed();
    
    // Debe procesar >10K results/segundo
    assert!(duration < Duration::from_secs(1));
    assert_eq!(ir.facts.len(), 10_000);
}
```

### Estimación

**Story Points**: 3  
**Esfuerzo**: 3-4 días

### Dependencias

**Bloqueantes**: US-14.1 (Orquestador)

---

*(Continúa con US-14.3, US-14.4, US-14.5, US-14.6, US-14.7 en formato similar...)*

---

## Resumen de Estimaciones

| US | Título | Story Points | Días |
|----|--------|--------------|------|
| US-14.1 | Orquestador | 5 | 5-7 |
| US-14.2 | SARIF | 3 | 3-4 |
| US-14.3 | Ruff | 2 | 2-3 |
| US-14.4 | ESLint | 2 | 2-3 |
| US-14.5 | Clippy | 2 | 2-3 |
| US-14.6 | staticcheck | 2 | 2-3 |
| US-14.7 | Deduplicación | 3 | 3-4 |
| **TOTAL** | | **19** | **~4 semanas** |
