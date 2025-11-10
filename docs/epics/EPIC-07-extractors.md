# EPIC-07: Extractors Framework & Core Extractors

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: Hodei Scan v3.2  
**Dependencias**: EPIC-02 (IR Core)  
**Owner**: Extractors Team  
**Prioridad**: Critical Path

---

## 1. Resumen Ejecutivo

Implementar el **framework de extractores** y los **extractores core** que emiten hechos atómicos al IR. Los extractores son plugins independientes que analizan código fuente, dependencias, coverage reports, etc.

### Objetivo de Negocio
Permitir análisis extensible de múltiples fuentes (SAST, coverage, dependencies) con un modelo de plugin consistente y type-safe.

### Métricas de Éxito
- **Extensibilidad**: API clara para custom extractors (trait-based).
- **Rendimiento**: Procesamiento paralelo de archivos; >10k LOC/s.
- **Cobertura**: 5+ extractores core implementados (taint, coverage, deps, semgrep, gitleaks).

---

## 2. Arquitectura

### 2.1. Extractor Trait

```rust
// hodei-extractors/src/lib.rs
use hodei_ir::{Fact, IntermediateRepresentation};
use std::path::Path;
use async_trait::async_trait;

#[async_trait]
pub trait Extractor: Send + Sync {
    /// Nombre único del extractor
    fn name(&self) -> &str;
    
    /// Versión del extractor
    fn version(&self) -> &str;
    
    /// Ejecuta extracción sobre un proyecto
    async fn extract(&self, ctx: &ExtractionContext) -> Result<Vec<Fact>, ExtractorError>;
    
    /// Configuración del extractor (opcional)
    fn configure(&mut self, config: ExtractorConfig) -> Result<(), ExtractorError> {
        Ok(())
    }
}

pub struct ExtractionContext {
    pub project_root: PathBuf,
    pub config: ExtractorConfig,
    pub ir_builder: IRBuilder,
}

#[derive(Debug, Clone)]
pub struct ExtractorConfig {
    pub include_patterns: Vec<String>,
    pub exclude_patterns: Vec<String>,
    pub max_file_size: usize,
    pub timeout: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum ExtractorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Timeout")]
    Timeout,
}
```

### 2.2. Core Extractors

#### TaintAnalysisExtractor
```rust
// hodei-extractors/src/taint.rs
pub struct TaintAnalysisExtractor {
    config: TaintConfig,
}

#[derive(Debug, Clone)]
pub struct TaintConfig {
    pub sources: Vec<String>,  // ["user_input", "request.body"]
    pub sinks: Vec<String>,    // ["exec", "sql.query"]
}

#[async_trait]
impl Extractor for TaintAnalysisExtractor {
    fn name(&self) -> &str { "taint-analysis" }
    fn version(&self) -> &str { "1.0.0" }
    
    async fn extract(&self, ctx: &ExtractionContext) -> Result<Vec<Fact>, ExtractorError> {
        let mut facts = Vec::new();
        
        // Parsear archivos fuente con tree-sitter
        for file in walk_source_files(&ctx.project_root, &ctx.config)? {
            let ast = parse_file_with_tree_sitter(&file)?;
            
            // Detectar sources
            for source in find_taint_sources(&ast, &self.config.sources) {
                facts.push(Fact {
                    fact_type: FactType::TaintSource {
                        source_type: source.source_type.clone(),
                    },
                    source_location: Some(source.location),
                    confidence: Confidence::new(0.8)?,
                    provenance: Provenance {
                        extractor: self.name().to_string(),
                        version: self.version().to_string(),
                        timestamp: Utc::now(),
                    },
                    ..Default::default()
                });
            }
            
            // Detectar sinks
            for sink in find_taint_sinks(&ast, &self.config.sinks) {
                facts.push(Fact {
                    fact_type: FactType::TaintSink {
                        sink_type: sink.sink_type.clone(),
                    },
                    source_location: Some(sink.location),
                    confidence: Confidence::new(0.9)?,
                    provenance: Provenance {
                        extractor: self.name().to_string(),
                        version: self.version().to_string(),
                        timestamp: Utc::now(),
                    },
                    ..Default::default()
                });
            }
            
            // Detectar flows (DataFlow facts)
            let flows = analyze_data_flow(&ast);
            for flow in flows {
                facts.push(Fact {
                    fact_type: FactType::DataFlow {
                        from: flow.from_id,
                        to: flow.to_id,
                    },
                    flow_id: Some(flow.flow_id),
                    confidence: Confidence::new(0.7)?,
                    provenance: Provenance {
                        extractor: self.name().to_string(),
                        version: self.version().to_string(),
                        timestamp: Utc::now(),
                    },
                    ..Default::default()
                });
            }
        }
        
        Ok(facts)
    }
}
```

#### CoverageExtractor
```rust
// hodei-extractors/src/coverage.rs
pub struct CoverageExtractor;

#[async_trait]
impl Extractor for CoverageExtractor {
    fn name(&self) -> &str { "coverage" }
    fn version(&self) -> &str { "1.0.0" }
    
    async fn extract(&self, ctx: &ExtractionContext) -> Result<Vec<Fact>, ExtractorError> {
        let mut facts = Vec::new();
        
        // Leer coverage report (lcov, cobertura, etc.)
        let coverage_file = ctx.project_root.join("coverage/lcov.info");
        let coverage_data = parse_lcov_file(&coverage_file)?;
        
        for file_cov in coverage_data.files {
            for line_num in file_cov.uncovered_lines {
                facts.push(Fact {
                    fact_type: FactType::UncoveredLine,
                    source_location: Some(SourceLocation {
                        file: ProjectPath::new(&file_cov.path)?,
                        start: Position {
                            line: LineNumber(line_num),
                            column: ColumnNumber(0),
                        },
                        end: Position {
                            line: LineNumber(line_num),
                            column: ColumnNumber(u32::MAX),
                        },
                    }),
                    confidence: Confidence::new(1.0)?,  // Coverage data es exacto
                    provenance: Provenance {
                        extractor: self.name().to_string(),
                        version: self.version().to_string(),
                        timestamp: Utc::now(),
                    },
                    ..Default::default()
                });
            }
            
            for line_num in file_cov.covered_lines {
                facts.push(Fact {
                    fact_type: FactType::CoveredLine,
                    source_location: Some(SourceLocation {
                        file: ProjectPath::new(&file_cov.path)?,
                        start: Position {
                            line: LineNumber(line_num),
                            column: ColumnNumber(0),
                        },
                        end: Position {
                            line: LineNumber(line_num),
                            column: ColumnNumber(u32::MAX),
                        },
                    }),
                    confidence: Confidence::new(1.0)?,
                    provenance: Provenance {
                        extractor: self.name().to_string(),
                        version: self.version().to_string(),
                        timestamp: Utc::now(),
                    },
                    ..Default::default()
                });
            }
        }
        
        Ok(facts)
    }
}
```

#### DependencyExtractor
```rust
// hodei-extractors/src/dependency.rs
pub struct DependencyExtractor;

#[async_trait]
impl Extractor for DependencyExtractor {
    fn name(&self) -> &str { "dependency" }
    fn version(&self) -> &str { "1.0.0" }
    
    async fn extract(&self, ctx: &ExtractionContext) -> Result<Vec<Fact>, ExtractorError> {
        let mut facts = Vec::new();
        
        // Detectar tipo de proyecto (Cargo.toml, package.json, etc.)
        if ctx.project_root.join("Cargo.toml").exists() {
            facts.extend(self.extract_rust_deps(ctx)?);
        }
        
        if ctx.project_root.join("package.json").exists() {
            facts.extend(self.extract_npm_deps(ctx)?);
        }
        
        Ok(facts)
    }
}

impl DependencyExtractor {
    fn extract_rust_deps(&self, ctx: &ExtractionContext) -> Result<Vec<Fact>, ExtractorError> {
        let cargo_toml = ctx.project_root.join("Cargo.toml");
        let manifest = cargo_toml::Manifest::from_path(&cargo_toml)?;
        
        let mut facts = Vec::new();
        
        for (name, dep) in manifest.dependencies {
            facts.push(Fact {
                fact_type: FactType::Dependency {
                    name: name.clone(),
                    version: dep.version().to_string(),
                    ecosystem: "cargo".to_string(),
                },
                confidence: Confidence::new(1.0)?,
                provenance: Provenance {
                    extractor: self.name().to_string(),
                    version: self.version().to_string(),
                    timestamp: Utc::now(),
                },
                ..Default::default()
            });
        }
        
        Ok(facts)
    }
    
    fn extract_npm_deps(&self, ctx: &ExtractionContext) -> Result<Vec<Fact>, ExtractorError> {
        let package_json = ctx.project_root.join("package.json");
        let pkg: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&package_json)?)?;
        
        let mut facts = Vec::new();
        
        if let Some(deps) = pkg["dependencies"].as_object() {
            for (name, version) in deps {
                facts.push(Fact {
                    fact_type: FactType::Dependency {
                        name: name.clone(),
                        version: version.as_str().unwrap_or("*").to_string(),
                        ecosystem: "npm".to_string(),
                    },
                    confidence: Confidence::new(1.0)?,
                    provenance: Provenance {
                        extractor: self.name().to_string(),
                        version: self.version().to_string(),
                        timestamp: Utc::now(),
                    },
                    ..Default::default()
                });
            }
        }
        
        Ok(facts)
    }
}
```

#### SemgrepExtractor (wrapper)
```rust
// hodei-extractors/src/semgrep.rs
pub struct SemgrepExtractor {
    semgrep_path: PathBuf,
}

#[async_trait]
impl Extractor for SemgrepExtractor {
    fn name(&self) -> &str { "semgrep" }
    fn version(&self) -> &str { "1.0.0" }
    
    async fn extract(&self, ctx: &ExtractionContext) -> Result<Vec<Fact>, ExtractorError> {
        // Ejecutar semgrep y parsear JSON output
        let output = tokio::process::Command::new(&self.semgrep_path)
            .arg("--config=auto")
            .arg("--json")
            .arg(&ctx.project_root)
            .output()
            .await?;
        
        let results: SemgrepOutput = serde_json::from_slice(&output.stdout)?;
        
        let facts: Vec<Fact> = results.results.into_iter()
            .map(|result| Fact {
                fact_type: FactType::Vulnerability {
                    vuln_type: result.check_id,
                    severity: map_semgrep_severity(result.severity),
                    description: result.extra.message,
                },
                source_location: Some(SourceLocation {
                    file: ProjectPath::new(&result.path)?,
                    start: Position {
                        line: LineNumber(result.start.line),
                        column: ColumnNumber(result.start.col),
                    },
                    end: Position {
                        line: LineNumber(result.end.line),
                        column: ColumnNumber(result.end.col),
                    },
                }),
                confidence: Confidence::new(0.8)?,
                provenance: Provenance {
                    extractor: self.name().to_string(),
                    version: self.version().to_string(),
                    timestamp: Utc::now(),
                },
                ..Default::default()
            })
            .collect();
        
        Ok(facts)
    }
}
```

### 2.3. ExtractorRunner (Orchestrator)

```rust
// hodei-extractors/src/runner.rs
use rayon::prelude::*;

pub struct ExtractorRunner {
    extractors: Vec<Box<dyn Extractor>>,
}

impl ExtractorRunner {
    pub fn new() -> Self {
        Self { extractors: Vec::new() }
    }
    
    pub fn register(&mut self, extractor: Box<dyn Extractor>) {
        self.extractors.push(extractor);
    }
    
    pub async fn run_all(&self, ctx: &ExtractionContext) -> Result<IntermediateRepresentation, RunnerError> {
        let facts_results: Vec<_> = self.extractors.par_iter()
            .map(|extractor| {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                runtime.block_on(extractor.extract(ctx))
            })
            .collect();
        
        let mut all_facts = Vec::new();
        for result in facts_results {
            match result {
                Ok(facts) => all_facts.extend(facts),
                Err(e) => eprintln!("Extractor failed: {}", e),
            }
        }
        
        let ir = IntermediateRepresentation {
            version: "1.0".to_string(),
            project: ctx.project_root.to_string_lossy().to_string(),
            timestamp: Utc::now(),
            facts: all_facts,
        };
        
        Ok(ir)
    }
}
```

---

## 3. Plan de Implementación

**Fase 1: Framework** (Semana 1)
- [ ] Definir `Extractor` trait.
- [ ] Implementar `ExtractorRunner`.
- [ ] Tests: registrar y ejecutar extractors.

**Fase 2: Core Extractors** (Semana 2-4)
- [ ] Implementar TaintAnalysisExtractor (tree-sitter).
- [ ] Implementar CoverageExtractor (lcov parser).
- [ ] Implementar DependencyExtractor (Cargo.toml, package.json).
- [ ] Implementar SemgrepExtractor (wrapper).
- [ ] Implementar GitleaksExtractor (wrapper).

**Fase 3: Paralelización** (Semana 4)
- [ ] Ejecutar extractors en paralelo con rayon.
- [ ] Benchmarks: tiempo total de extracción.

---

## 4. Tests & Validación

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn taint_extractor_finds_sources_and_sinks() {
        let extractor = TaintAnalysisExtractor {
            config: TaintConfig {
                sources: vec!["user_input".to_string()],
                sinks: vec!["exec".to_string()],
            },
        };
        
        let ctx = create_test_context("fixtures/vulnerable_app");
        let facts = extractor.extract(&ctx).await.unwrap();
        
        assert!(facts.iter().any(|f| matches!(f.fact_type, FactType::TaintSource { .. })));
        assert!(facts.iter().any(|f| matches!(f.fact_type, FactType::TaintSink { .. })));
    }
    
    #[tokio::test]
    async fn coverage_extractor_parses_lcov() {
        let extractor = CoverageExtractor;
        
        let ctx = create_test_context("fixtures/project_with_coverage");
        let facts = extractor.extract(&ctx).await.unwrap();
        
        assert!(facts.iter().any(|f| matches!(f.fact_type, FactType::UncoveredLine)));
        assert!(facts.iter().any(|f| matches!(f.fact_type, FactType::CoveredLine)));
    }
}
```

---

## 5. Criterios de Aceptación

- [ ] 5+ extractores core implementados.
- [ ] Ejecución paralela funcional.
- [ ] Tests con fixtures realistas.
- [ ] Documentación de cómo crear custom extractors.

---

**Última Actualización**: 2025-01-XX
