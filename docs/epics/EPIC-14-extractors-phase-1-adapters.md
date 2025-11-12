# EPIC-14: Ecosistema de Extractores - Fase 1 (Adaptadores)
## Cosecha Rápida: Integración de Herramientas Existentes

**Versión:** 1.0.0  
**Fecha Creación:** 2025-11-12  
**Estado:** Propuesta  
**Prioridad:** Crítica  
**Fase:** v3.3 - Q1 2025

---

## 📋 Resumen Ejecutivo

### Objetivo Estratégico

Conseguir una cobertura de análisis masiva (cientos de reglas) en múltiples lenguajes durante el primer mes de desarrollo, integrando herramientas líderes del mercado mediante un sistema de adaptadores estandarizado basado en SARIF.

### Propuesta de Valor

**Para usuarios**: Acceso inmediato a análisis de calidad y seguridad de código mediante herramientas probadas en la industria (Ruff, ESLint, Clippy, etc.) bajo una interfaz unificada de `hodei-scan`.

**Para el proyecto**: Establecer rápidamente presencia en el mercado con cobertura comparable a soluciones enterprise (SonarQube, Snyk) sin escribir lógica de análisis desde cero.

### Métricas de Éxito

- ✅ **Cobertura**: >500 reglas activas en 4+ lenguajes principales
- ✅ **Velocidad**: Análisis completo de proyecto medio (100K LOC) en <30 segundos
- ✅ **Calidad**: <5% tasa de falsos positivos en benchmarks de seguridad
- ✅ **Adopción**: Compatibilidad con 100% de herramientas que exportan SARIF

---

## 🎯 Contexto y Motivación

### Análisis del Problema

El desarrollo de extractores nativos es costoso en tiempo:
- Implementar parser para cada lenguaje: 4-8 semanas por lenguaje
- Definir y probar reglas: 2-3 días por regla de calidad
- Mantener actualizaciones con estándares del lenguaje: esfuerzo continuo

**Alternativa estratégica**: Integrar herramientas existentes que ya resuelven estos problemas.

### Benchmarking de Competidores

| Herramienta | Estrategia | Cobertura | Tiempo al Mercado |
|-------------|-----------|-----------|-------------------|
| **SonarQube** | Motores nativos + integraciones | 25+ lenguajes, 5000+ reglas | 10+ años desarrollo |
| **Semgrep** | Motor propio + reglas community | 30+ lenguajes, 2000+ reglas | 3+ años desarrollo |
| **CodeQL** | Motor propietario GitHub | 12 lenguajes, 1000+ queries | 5+ años (adquirido) |
| **hodei-scan v3.3** | **Adaptadores + Agregación** | **4-6 lenguajes, 500+ reglas** | **4-6 semanas** |

### Estrategia de Tres Niveles

```
┌──────────────────────────────────────────────────────────┐
│              ESTRATEGIA DE EXTRACTORES                   │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  NIVEL 1: ADAPTADORES (Esta Épica)                      │
│  ├─ Objetivo: Cobertura masiva rápida                   │
│  ├─ Esfuerzo: 4-6 semanas                               │
│  ├─ Resultado: 500+ reglas, 4+ lenguajes                │
│  └─ Valor: "Fast follower" del mercado                  │
│                                                          │
│  NIVEL 2: EXTRACTORES DECLARATIVOS                      │
│  ├─ Objetivo: Democratizar creación de reglas           │
│  ├─ Esfuerzo: 6-10 semanas                              │
│  ├─ Resultado: DSL YAML + motor tree-sitter             │
│  └─ Valor: Reglas custom en <5 minutos                  │
│                                                          │
│  NIVEL 3: EXTRACTORES PROFUNDOS                         │
│  ├─ Objetivo: Análisis de vanguardia (taint analysis)   │
│  ├─ Esfuerzo: 12-16 semanas por lenguaje                │
│  ├─ Resultado: Detección de vulnerabilidades complejas  │
│  └─ Valor: Diferenciador competitivo                    │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

---

## 🏗️ Arquitectura del Sistema de Adaptadores

### Contrato del Extractor

Todo extractor debe implementar este contrato:

```rust
/// Contrato estándar para extractores de hodei-scan
pub trait Extractor {
    /// Ejecuta el análisis y retorna IR en formato Cap'n Proto
    fn extract(&self, config: ExtractorConfig) -> Result<IntermediateRepresentation>;
    
    /// Metadatos del extractor
    fn metadata(&self) -> ExtractorMetadata;
}

/// Configuración de entrada para extractores
pub struct ExtractorConfig {
    /// Ruta al proyecto a analizar
    pub project_path: PathBuf,
    /// Configuración específica del extractor en JSON
    pub extractor_settings: serde_json::Value,
    /// Paths de archivos a incluir/excluir
    pub file_filters: FileFilters,
}

/// Metadatos del extractor
pub struct ExtractorMetadata {
    pub id: String,
    pub name: String,
    pub version: semver::Version,
    pub supported_languages: Vec<String>,
    pub capabilities: ExtractorCapabilities,
}
```

### Flujo de Ejecución

```
┌─────────────────────────────────────────────────────────────┐
│  1. hodei-scan CLI lee hodei.toml                           │
│     extractors:                                             │
│       - id: sarif-universal                                 │
│         command: hodei-extract-sarif                        │
│       - id: ruff-python                                     │
│         command: hodei-extract-ruff                         │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  2. Orquestador ejecuta extractores en paralelo             │
│     • Cada extractor recibe config via stdin (JSON)         │
│     • Cada extractor escribe IR a stdout (Cap'n Proto)      │
│     • Logs van a stderr                                     │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  3. Agregador valida y fusiona IRs                          │
│     • Valida cada IR contra esquema                         │
│     • Elimina duplicados por fingerprint                    │
│     • Enriquece con metadatos de correlación                │
└─────────────────────┬───────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────┐
│  4. Motor de Evaluación procesa IR unificado                │
│     • Carga en índices espaciales                           │
│     • Ejecuta reglas de correlación                         │
│     • Genera hallazgos agregados                            │
└─────────────────────────────────────────────────────────────┘
```

### Formato SARIF - El Rosetta Stone

SARIF (Static Analysis Results Interchange Format) es el estándar OASIS 2.1.0 adoptado por:
- GitHub Advanced Security
- Microsoft Security Code Analysis
- Checkmarx
- Veracode
- Fortify
- 50+ herramientas más

**Estructura básica**:

```json
{
  "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
  "version": "2.1.0",
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "ESLint",
          "version": "8.50.0",
          "rules": [
            {
              "id": "no-eval",
              "shortDescription": {"text": "Disallow eval()"},
              "properties": {
                "security-severity": "9.0"
              }
            }
          ]
        }
      },
      "results": [
        {
          "ruleId": "no-eval",
          "level": "error",
          "message": {"text": "eval() can execute arbitrary code"},
          "locations": [{
            "physicalLocation": {
              "artifactLocation": {"uri": "src/unsafe.js"},
              "region": {"startLine": 42, "startColumn": 5}
            }
          }],
          "properties": {
            "security-severity": "9.0"
          }
        }
      ]
    }
  ]
}
```

**Mapeo SARIF → hodei-scan IR**:

| Campo SARIF | Campo IR | Transformación |
|-------------|----------|----------------|
| `ruleId` | `Fact::rule_id` | Directo |
| `level` (error/warning) | `Fact::severity` | Mapeo: error=HIGH, warning=MEDIUM, note=LOW |
| `message.text` | `Fact::message` | Directo |
| `physicalLocation` | `Fact::location` | Conversión a ProjectPath + Span |
| `properties.security-severity` | `Fact::confidence_score` | Normalización a 0.0-1.0 |
| `properties.tags` | `Fact::categories` | Extracción de categorías (security, quality, etc.) |

---

## 📊 Historias de Usuario

### US-14.1: Infraestructura Core de Orquestación

**Como** desarrollador del core  
**Quiero** un orquestador que ejecute extractores en paralelo y agregue sus IRs  
**Para** poder integrar múltiples herramientas sin acoplamiento

**Criterios de Aceptación**:
- ✅ Lee configuración de extractores desde `hodei.toml`
- ✅ Ejecuta extractores como procesos hijos
- ✅ Captura stdout (IR) y stderr (logs) independientemente
- ✅ Implementa timeout configurable por extractor (default: 5 min)
- ✅ Maneja fallos gracefully (continúa con otros extractores)
- ✅ Valida cada IR contra esquema Cap'n Proto
- ✅ Fusiona IRs eliminando duplicados por fingerprint
- ✅ Genera métricas de ejecución (timing, hechos por extractor)

**Estimación**: 5 Story Points (1 semana)

**Tareas Técnicas**:
1. Diseñar esquema de configuración en `hodei.toml`
2. Implementar `ExtractorOrchestrator` con proceso pool
3. Crear sistema de fingerprinting para deduplicación
4. Implementar validación de esquema Cap'n Proto
5. Escribir tests de integración con extractores mock

**Pruebas**:
```rust
#[test]
fn test_orchestrator_parallel_execution() {
    let config = ExtractorConfig::from_toml("fixtures/hodei.toml");
    let orchestrator = ExtractorOrchestrator::new(config);
    
    let ir = orchestrator.run_all_extractors().unwrap();
    
    // Verifica que todos los extractores ejecutaron
    assert_eq!(ir.metadata.extractor_runs.len(), 3);
    
    // Verifica deduplicación
    let unique_facts: HashSet<_> = ir.facts.iter()
        .map(|f| f.fingerprint())
        .collect();
    assert_eq!(unique_facts.len(), ir.facts.len());
}
```

---

### US-14.2: Extractor Universal SARIF

**Como** usuario de GitHub Advanced Security  
**Quiero** importar mis reportes SARIF directamente a hodei-scan  
**Para** unificar mi análisis de seguridad

**Criterios de Aceptación**:
- ✅ Parsea ficheros SARIF 2.1.0 válidos
- ✅ Mapea correctamente todos los campos clave a IR
- ✅ Soporta múltiples runs en un fichero SARIF
- ✅ Extrae security-severity cuando está presente
- ✅ Maneja gracefully campos opcionales ausentes
- ✅ Genera warnings para reglas sin metadata completa
- ✅ Throughput >10K resultados/segundo

**Estimación**: 3 Story Points (3-4 días)

**Esquema de Configuración**:
```toml
[[extractors]]
id = "sarif-universal"
command = "hodei-extract-sarif"
enabled = true

[extractors.config]
# Ruta a fichero SARIF o glob pattern
sarif_files = ["results/**/*.sarif"]
# Filtros opcionales
exclude_rules = ["style/*", "deprecated/*"]
min_severity = "warning"
```

**Implementación**:
```rust
pub struct SarifExtractor {
    config: SarifConfig,
}

impl Extractor for SarifExtractor {
    fn extract(&self, input: ExtractorConfig) -> Result<IntermediateRepresentation> {
        let mut ir_builder = IRBuilder::new();
        
        for sarif_path in self.discover_sarif_files(&input.project_path)? {
            let sarif: SarifReport = serde_json::from_reader(
                BufReader::new(File::open(&sarif_path)?)
            )?;
            
            for run in sarif.runs {
                self.process_run(&run, &mut ir_builder)?;
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
    fn process_run(&self, run: &SarifRun, ir: &mut IRBuilder) -> Result<()> {
        let tool_name = &run.tool.driver.name;
        let tool_version = &run.tool.driver.version;
        
        for result in &run.results {
            let fact = self.sarif_result_to_fact(result, tool_name, tool_version)?;
            ir.add_fact(fact);
        }
        
        Ok(())
    }
    
    fn sarif_result_to_fact(
        &self,
        result: &SarifResult,
        tool: &str,
        version: &str,
    ) -> Result<Fact> {
        let location = self.extract_location(&result.locations[0])?;
        let severity = self.map_severity(&result.level);
        
        let fact_type = if result.properties.get("security-severity").is_some() {
            FactType::Vulnerability(VulnerabilityFact {
                cwe_ids: self.extract_cwe_ids(result),
                security_severity: result.properties["security-severity"]
                    .as_f64()
                    .unwrap_or(5.0) / 10.0, // Normaliza a 0.0-1.0
            })
        } else {
            FactType::CodeSmell(CodeSmellFact {
                smell_type: result.rule_id.clone(),
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
                source_file: Some(result.rule_id.clone()),
            },
            metadata: HashMap::new(),
        })
    }
    
    fn map_severity(&self, level: &str) -> Severity {
        match level {
            "error" => Severity::High,
            "warning" => Severity::Medium,
            "note" | "none" => Severity::Low,
            _ => Severity::Medium,
        }
    }
}
```

**Pruebas**:
```rust
#[test]
fn test_sarif_github_security() {
    let extractor = SarifExtractor::default();
    let config = ExtractorConfig {
        project_path: "fixtures/github-security".into(),
        extractor_settings: json!({
            "sarif_files": ["security-results.sarif"]
        }),
        file_filters: FileFilters::default(),
    };
    
    let ir = extractor.extract(config).unwrap();
    
    // Verifica que se importaron vulnerabilidades
    let vulns: Vec<_> = ir.facts.iter()
        .filter(|f| matches!(f.fact_type, FactType::Vulnerability(_)))
        .collect();
    
    assert!(vulns.len() > 0);
    
    // Verifica mapeo de security-severity
    let high_severity = vulns.iter()
        .filter(|f| f.severity == Severity::High)
        .count();
    assert!(high_severity > 0);
}
```

---

### US-14.3: Adaptador Ruff (Python)

**Como** desarrollador de Python  
**Quiero** que hodei-scan ejecute Ruff automáticamente  
**Para** aprovechar sus 700+ reglas sin instalar herramientas adicionales

**Criterios de Aceptación**:
- ✅ Ejecuta `ruff check` con configuración personalizable
- ✅ Parsea salida JSON de Ruff
- ✅ Mapea códigos de error de Ruff a categorías de hodei-scan
- ✅ Respeta configuración `.ruff.toml` del proyecto si existe
- ✅ Soporta fixing automático opcional
- ✅ Rendimiento: >100K LOC/segundo

**Estimación**: 2 Story Points (2-3 días)

**Configuración**:
```toml
[[extractors]]
id = "ruff-python"
command = "hodei-extract-ruff"
enabled = true

[extractors.config]
# Selectores de reglas (ver https://docs.astral.sh/ruff/rules/)
select = ["E", "F", "B", "S", "I"]  # Errors, pyflakes, bugbear, security, imports
ignore = ["E501"]  # Line too long

# Opciones de fixing
fix = false
fix_only = false

# Paths a incluir
include = ["*.py"]
exclude = ["tests/fixtures/**"]
```

**Implementación**:
```rust
pub struct RuffExtractor {
    config: RuffConfig,
}

impl Extractor for RuffExtractor {
    fn extract(&self, input: ExtractorConfig) -> Result<IntermediateRepresentation> {
        // Ejecuta Ruff como subprocess
        let output = Command::new("ruff")
            .arg("check")
            .arg(&input.project_path)
            .arg("--format")
            .arg("json")
            .args(self.build_ruff_args())
            .output()?;
        
        if !output.status.success() && output.stdout.is_empty() {
            return Err(ExtractorError::ToolFailed {
                tool: "ruff",
                stderr: String::from_utf8_lossy(&output.stderr).into(),
            });
        }
        
        let ruff_results: Vec<RuffViolation> = serde_json::from_slice(&output.stdout)?;
        
        let mut ir_builder = IRBuilder::new();
        for violation in ruff_results {
            let fact = self.ruff_violation_to_fact(violation)?;
            ir_builder.add_fact(fact);
        }
        
        Ok(ir_builder.build())
    }
    
    fn metadata(&self) -> ExtractorMetadata {
        ExtractorMetadata {
            id: "ruff-python".into(),
            name: "Ruff Python Linter".into(),
            version: self.get_ruff_version().unwrap_or_default(),
            supported_languages: vec!["python".into()],
            capabilities: ExtractorCapabilities {
                provides_facts: vec![
                    FactType::CodeSmell,
                    FactType::Bug,
                    FactType::Vulnerability,
                ],
                requires_source_code: true,
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct RuffViolation {
    code: String,
    message: String,
    location: RuffLocation,
    end_location: RuffLocation,
    filename: PathBuf,
    noqa_row: Option<usize>,
}

#[derive(Debug, Deserialize)]
struct RuffLocation {
    row: usize,
    column: usize,
}

impl RuffExtractor {
    fn ruff_violation_to_fact(&self, violation: RuffViolation) -> Result<Fact> {
        let location = Location {
            file: ProjectPath::new(&violation.filename)?,
            span: Span {
                start: Position {
                    line: violation.location.row as u32,
                    column: violation.location.column as u32,
                },
                end: Position {
                    line: violation.end_location.row as u32,
                    column: violation.end_location.column as u32,
                },
            },
        };
        
        let (fact_type, severity) = self.categorize_ruff_code(&violation.code);
        
        Ok(Fact {
            id: FactId::generate(),
            fact_type,
            location,
            message: violation.message,
            severity,
            provenance: Provenance {
                extractor_id: "ruff-python".into(),
                extractor_version: self.get_ruff_version().unwrap_or_default(),
                extracted_at: SystemTime::now(),
                source_file: Some(violation.code.clone()),
            },
            metadata: HashMap::from([
                ("ruff_code".into(), violation.code.into()),
            ]),
        })
    }
    
    fn categorize_ruff_code(&self, code: &str) -> (FactType, Severity) {
        // Mapeo de prefijos de código Ruff a categorías
        match code.chars().next() {
            Some('E') | Some('W') => {
                // Errores de estilo/sintaxis
                (FactType::CodeSmell(CodeSmellFact {
                    smell_type: "style".into(),
                }), Severity::Low)
            }
            Some('F') => {
                // Pyflakes (bugs lógicos)
                (FactType::Bug(BugFact {
                    bug_category: "logic_error".into(),
                }), Severity::Medium)
            }
            Some('B') => {
                // Bugbear (anti-patterns)
                (FactType::CodeSmell(CodeSmellFact {
                    smell_type: "anti_pattern".into(),
                }), Severity::Medium)
            }
            Some('S') => {
                // Bandit (seguridad)
                (FactType::Vulnerability(VulnerabilityFact {
                    cwe_ids: vec![],
                    security_severity: 0.6,
                }), Severity::High)
            }
            _ => {
                (FactType::CodeSmell(CodeSmellFact {
                    smell_type: "other".into(),
                }), Severity::Low)
            }
        }
    }
    
    fn get_ruff_version(&self) -> Result<semver::Version> {
        let output = Command::new("ruff")
            .arg("--version")
            .output()?;
        
        let version_str = String::from_utf8_lossy(&output.stdout);
        let version = version_str
            .split_whitespace()
            .nth(1)
            .ok_or(ExtractorError::VersionParsing)?;
        
        Ok(semver::Version::parse(version)?)
    }
}
```

**Pruebas**:
```rust
#[test]
fn test_ruff_security_detection() {
    let extractor = RuffExtractor::default();
    let config = ExtractorConfig {
        project_path: "fixtures/python-insecure".into(),
        extractor_settings: json!({
            "select": ["S"]  // Solo reglas de seguridad
        }),
        file_filters: FileFilters::default(),
    };
    
    let ir = extractor.extract(config).unwrap();
    
    // Fixture tiene 'eval(user_input)' que Ruff detecta como S307
    let eval_vuln = ir.facts.iter()
        .find(|f| matches!(f.fact_type, FactType::Vulnerability(_)))
        .expect("Should detect eval vulnerability");
    
    assert_eq!(eval_vuln.severity, Severity::High);
    assert!(eval_vuln.message.contains("eval"));
}

#[test]
fn test_ruff_performance() {
    // Proyecto con ~50K LOC Python
    let start = Instant::now();
    
    let extractor = RuffExtractor::default();
    let ir = extractor.extract(ExtractorConfig {
        project_path: "fixtures/large-python-project".into(),
        extractor_settings: json!({}),
        file_filters: FileFilters::default(),
    }).unwrap();
    
    let duration = start.elapsed();
    
    // Ruff debe analizar 50K LOC en <5 segundos
    assert!(duration < Duration::from_secs(5));
    assert!(ir.facts.len() > 100);
}
```

---

### US-14.4: Adaptador ESLint (JavaScript/TypeScript)

**Como** desarrollador de JavaScript/TypeScript  
**Quiero** que hodei-scan ejecute ESLint automáticamente  
**Para** detectar bugs y vulnerabilidades en mi código frontend/backend

**Criterios de Aceptación**:
- ✅ Ejecuta ESLint con configuración del proyecto (.eslintrc)
- ✅ Soporta TypeScript mediante plugin
- ✅ Parsea salida JSON de ESLint
- ✅ Mapea niveles de severidad correctamente
- ✅ Detecta vulnerabilidades de seguridad (ej: no-eval, no-innerHTML)
- ✅ Rendimiento: >50K LOC/segundo

**Estimación**: 2 Story Points (2-3 días)

**Configuración**:
```toml
[[extractors]]
id = "eslint-javascript"
command = "hodei-extract-eslint"
enabled = true

[extractors.config]
# Config override (opcional, respeta .eslintrc por defecto)
extends = ["eslint:recommended", "plugin:security/recommended"]
rules = { "no-eval" = "error", "no-console" = "warn" }

# Paths
include = ["**/*.js", "**/*.jsx", "**/*.ts", "**/*.tsx"]
exclude = ["node_modules/**", "dist/**"]

# Plugins
plugins = ["security", "@typescript-eslint"]
```

**Implementación destacada**:
```rust
impl ESLintExtractor {
    fn eslint_message_to_fact(&self, msg: ESLintMessage, file: &Path) -> Result<Fact> {
        let location = Location {
            file: ProjectPath::new(file)?,
            span: Span {
                start: Position {
                    line: msg.line as u32,
                    column: msg.column as u32,
                },
                end: Position {
                    line: msg.end_line.unwrap_or(msg.line) as u32,
                    column: msg.end_column.unwrap_or(msg.column + 1) as u32,
                },
            },
        };
        
        let severity = match msg.severity {
            2 => Severity::High,  // ESLint "error"
            1 => Severity::Medium, // ESLint "warning"
            _ => Severity::Low,
        };
        
        // Detecta si es vulnerabilidad de seguridad
        let is_security = msg.rule_id.as_ref()
            .map(|id| id.starts_with("security/") || 
                      SECURITY_RULES.contains(id.as_str()))
            .unwrap_or(false);
        
        let fact_type = if is_security {
            FactType::Vulnerability(VulnerabilityFact {
                cwe_ids: self.map_eslint_rule_to_cwe(msg.rule_id.as_ref()),
                security_severity: 0.7,
            })
        } else {
            FactType::CodeSmell(CodeSmellFact {
                smell_type: msg.rule_id.clone().unwrap_or_default(),
            })
        };
        
        Ok(Fact {
            id: FactId::generate(),
            fact_type,
            location,
            message: msg.message,
            severity,
            provenance: Provenance {
                extractor_id: "eslint-javascript".into(),
                extractor_version: self.get_eslint_version()?,
                extracted_at: SystemTime::now(),
                source_file: msg.rule_id,
            },
            metadata: HashMap::new(),
        })
    }
    
    fn map_eslint_rule_to_cwe(&self, rule_id: Option<&String>) -> Vec<u32> {
        // Mapeo manual de reglas ESLint a CWEs
        match rule_id.map(|s| s.as_str()) {
            Some("no-eval") => vec![95, 94], // CWE-95: Eval Injection
            Some("security/detect-non-literal-regexp") => vec![625], // CWE-625: RegEx DoS
            Some("security/detect-sql-injection") => vec![89], // CWE-89: SQL Injection
            _ => vec![],
        }
    }
}

const SECURITY_RULES: &[&str] = &[
    "no-eval",
    "no-implied-eval",
    "no-new-func",
    "no-script-url",
];
```

---

### US-14.5: Adaptador Clippy (Rust)

**Como** desarrollador de Rust  
**Quiero** que hodei-scan ejecute Clippy automáticamente  
**Para** mantener código idiomático y detectar errores sutiles

**Criterios de Aceptación**:
- ✅ Ejecuta `cargo clippy` con lints configurables
- ✅ Parsea salida JSON de Clippy
- ✅ Distingue entre correctness, performance, style
- ✅ Soporta lints pedantic y nursery opcionales
- ✅ Integra con `Cargo.toml` del proyecto

**Estimación**: 2 Story Points (2-3 días)

---

### US-14.6: Adaptador staticcheck (Go)

**Como** desarrollador de Go  
**Quiero** que hodei-scan ejecute staticcheck automáticamente  
**Para** detectar bugs comunes y anti-patterns

**Criterios de Aceptación**:
- ✅ Ejecuta `staticcheck` sobre módulos Go
- ✅ Parsea salida JSON
- ✅ Mapea categorías de checks (SA, ST, QF, etc.)
- ✅ Soporta configuración via `staticcheck.conf`

**Estimación**: 2 Story Points (2-3 días)

---

### US-14.7: Sistema de Deduplicación Inteligente

**Como** usuario que ejecuta múltiples extractores  
**Quiero** que hodei-scan elimine hallazgos duplicados automáticamente  
**Para** ver un reporte limpio sin ruido

**Criterios de Aceptación**:
- ✅ Calcula fingerprint estable por hallazgo
- ✅ Agrupa hallazgos por fingerprint
- ✅ Selecciona "mejor" hallazgo del grupo (criterios: severidad, confianza)
- ✅ Preserva metadatos de origen de todos los extractores
- ✅ <1ms por 1000 hechos procesados

**Estimación**: 3 Story Points (3-4 días)

**Algoritmo de Fingerprinting**:
```rust
impl Fact {
    /// Calcula fingerprint estable para deduplicación
    pub fn fingerprint(&self) -> FactFingerprint {
        let mut hasher = blake3::Hasher::new();
        
        // Componentes del fingerprint
        hasher.update(self.location.file.as_str().as_bytes());
        hasher.update(&self.location.span.start.line.to_le_bytes());
        hasher.update(&self.location.span.start.column.to_le_bytes());
        
        // Tipo de hecho (sin metadatos específicos)
        let type_discriminant = std::mem::discriminant(&self.fact_type);
        hasher.update(&format!("{:?}", type_discriminant).as_bytes());
        
        // Primeras 50 chars del mensaje (normalizado)
        let normalized_msg = self.message
            .chars()
            .filter(|c| c.is_alphanumeric())
            .take(50)
            .collect::<String>()
            .to_lowercase();
        hasher.update(normalized_msg.as_bytes());
        
        FactFingerprint(hasher.finalize().as_bytes()[..16].try_into().unwrap())
    }
}

pub struct FactDeduplicator {
    fingerprints: HashMap<FactFingerprint, Vec<Fact>>,
}

impl FactDeduplicator {
    pub fn deduplicate(&mut self, facts: Vec<Fact>) -> Vec<Fact> {
        // Agrupa por fingerprint
        for fact in facts {
            let fp = fact.fingerprint();
            self.fingerprints.entry(fp)
                .or_insert_with(Vec::new)
                .push(fact);
        }
        
        // Selecciona el "mejor" de cada grupo
        self.fingerprints
            .values()
            .map(|group| self.select_best_fact(group))
            .collect()
    }
    
    fn select_best_fact(&self, group: &[Fact]) -> Fact {
        // Criterios de selección (en orden):
        // 1. Mayor severidad
        // 2. Mayor confianza (si es vulnerabilidad)
        // 3. Extractor más reciente
        
        let mut best = &group[0];
        
        for fact in &group[1..] {
            if fact.severity > best.severity {
                best = fact;
            } else if fact.severity == best.severity {
                // Desempate por confianza
                if let (
                    FactType::Vulnerability(v1),
                    FactType::Vulnerability(v2),
                ) = (&fact.fact_type, &best.fact_type) {
                    if v1.security_severity > v2.security_severity {
                        best = fact;
                    }
                }
            }
        }
        
        // Enriquece con metadatos de todos los extractores
        let mut result = best.clone();
        result.metadata.insert(
            "also_found_by".into(),
            group.iter()
                .map(|f| &f.provenance.extractor_id)
                .collect::<Vec<_>>()
                .join(", ")
                .into(),
        );
        
        result
    }
}
```

---

## 📈 Plan de Implementación

### Timeline Semanal

**Semana 1: Fundamentos**
- Día 1-2: US-14.1 (Orquestador) - Diseño + Implementación inicial
- Día 3-4: US-14.1 (Orquestador) - Tests + Validación
- Día 5: US-14.2 (SARIF) - Inicio

**Semana 2: Adaptador Universal + Python**
- Día 1-2: US-14.2 (SARIF) - Completar + Tests
- Día 3-4: US-14.3 (Ruff) - Implementación
- Día 5: US-14.3 (Ruff) - Tests + Benchmarks

**Semana 3: JavaScript + Deduplicación**
- Día 1-2: US-14.4 (ESLint) - Implementación
- Día 3: US-14.4 (ESLint) - Tests
- Día 4-5: US-14.7 (Deduplicación) - Implementación

**Semana 4: Rust + Go + Integración**
- Día 1-2: US-14.5 (Clippy) + US-14.6 (staticcheck)
- Día 3: US-14.7 (Deduplicación) - Completar tests
- Día 4-5: Tests de integración end-to-end

**Semana 5: Pulido + Documentación**
- Optimización de rendimiento
- Documentación de usuario
- Guías de configuración

### Dependencias

```
US-14.1 (Orquestador)
    ├─> US-14.2 (SARIF)
    ├─> US-14.3 (Ruff)
    ├─> US-14.4 (ESLint)
    ├─> US-14.5 (Clippy)
    └─> US-14.6 (staticcheck)

US-14.2..14.6 (Todos los extractores)
    └─> US-14.7 (Deduplicación)
```

### Riesgos y Mitigaciones

| Riesgo | Probabilidad | Impacto | Mitigación |
|--------|--------------|---------|------------|
| Herramientas externas no instaladas | Alta | Medio | Detectar ausencia + mensaje claro de instalación |
| Formatos de salida cambian entre versiones | Media | Alto | Versionar parsers + tests con múltiples versiones |
| Rendimiento de subprocess overhead | Baja | Medio | Cachear resultados + ejecutar en paralelo |
| Conflictos en configuración de herramientas | Media | Bajo | Priorizar config de hodei.toml > config de proyecto |

---

## 🎯 Criterios de Finalización de Épica

### Funcionales
- ✅ Orquestador ejecuta 4+ extractores en paralelo
- ✅ Extractores para Python, JavaScript/TypeScript, Rust, Go funcionando
- ✅ Adaptador SARIF importa reportes de >=3 herramientas distintas
- ✅ Deduplicación reduce hallazgos en 20-40% en proyectos con múltiples extractores
- ✅ Configuración en `hodei.toml` documentada y validada

### No Funcionales
- ✅ Benchmarks: Análisis de proyecto medio (100K LOC, 4 lenguajes) en <30s
- ✅ Tests: Cobertura >=80% en todo el código de extractores
- ✅ Documentación: README por extractor + ejemplos de configuración
- ✅ CI/CD: Pipeline verde con tests de integración

### Métricas de Éxito
- **Cobertura de reglas**: >=500 reglas activas
- **Rendimiento**: <=30s para proyecto medio
- **Calidad**: <5% tasa de falsos positivos en benchmark OWASP
- **Usabilidad**: Usuario puede configurar extractor en <5 minutos

---

## 📚 Recursos y Referencias

### Especificaciones
- [SARIF 2.1.0 Specification](https://docs.oasis-open.org/sarif/sarif/v2.1.0/sarif-v2.1.0.html)
- [Ruff Rules Documentation](https://docs.astral.sh/ruff/rules/)
- [ESLint Rules Reference](https://eslint.org/docs/rules/)
- [Clippy Lints List](https://rust-lang.github.io/rust-clippy/master/)

### Implementaciones de Referencia
- [GitHub CodeQL Action](https://github.com/github/codeql-action) - Pipeline SARIF
- [Microsoft SARIF SDK](https://github.com/microsoft/sarif-sdk) - Validación y utilidades

### Herramientas
- [sarif-tools](https://github.com/microsoft/sarif-tools) - CLI para manipular SARIF
- [ruff](https://github.com/astral-sh/ruff) - Linter Python ultra-rápido
- [eslint](https://github.com/eslint/eslint) - Linter JavaScript estándar
- [clippy](https://github.com/rust-lang/rust-clippy) - Lints de Rust
- [staticcheck](https://staticcheck.io/) - Análisis estático para Go

---

**Próxima Épica**: EPIC-15 - Extractores Declarativos (Fase 2)
