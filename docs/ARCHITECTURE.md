# Arquitectura Técnica de hodei-scan
## Sistema de Análisis de Código Nativo con Motor de Reglas Determinista

**Versión:** 2.0
**Fecha:** 10 de noviembre de 2025
**Estado:** ✅ Documentado
**Scope:** Arquitectura completa del sistema

---

## 📋 Resumen Ejecutivo

hodei-scan es un sistema de análisis de código estático nativo en Rust diseñado para competir con SonarQube, ofreciendo 2x-5x mejor performance, 5x menos uso de memoria y determinismo O(n) garantizado. La arquitectura sigue principios de **Hexagonal Architecture** con **Clean Architecture** y **SOLID** principles.

**Diferenciadores Arquitectónicos Clave:**
- ✅ **Cedar-Inspired Rule Engine**: Motor de reglas determinista <2ms
- ✅ **Sin contradicciones**: Eliminated LSP dependency del motor core
- ✅ **Análisis Semántico Profundo**: DFA, CFG, taint tracking por lenguaje
- ✅ **Extensibilidad WASM**: Sandbox para reglas enterprise
- ✅ **Análisis Incremental**: Sub-segundo feedback en tiempo real

---

## 🏗️ Vista General de la Arquitectura

### Capas Arquitectónicas

```
┌──────────────────────────────────────────────────────────────┐
│                    PRESENTATION LAYER                        │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐  │
│  │   CLI        │ │   Web API    │ │   GitHub/GitLab      │  │
│  │   Interface  │ │   (Axum)     │ │   Integrations       │  │
│  └──────────────┘ └──────────────┘ └──────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                              │
┌──────────────────────────────────────────────────────────────┐
│                   APPLICATION LAYER                         │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐  │
│  │  Analysis    │ │  SCA Engine  │ │  Coverage Engine     │  │
│  │  Coordinator │ │              │ │                      │  │
│  └──────────────┘ └──────────────┘ └──────────────────────┘  │
│                              │                                │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐  │
│  │ Quality Gate │ │ PR Decoration│ │  Debt Calculator     │  │
│  │  Evaluator   │ │  Engine      │ │                      │  │
│  └──────────────┘ └──────────────┘ └──────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                              │
┌──────────────────────────────────────────────────────────────┐
│                      DOMAIN LAYER                           │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐  │
│  │ Cedar Rule   │ │  Taint       │ │  Portfolio           │  │
│  │  Engine      │ │  Analysis    │ │  Management          │  │
│  └──────────────┘ └──────────────┘ └──────────────────────┘  │
│         │                    │                               │
│  ┌──────▼──────┐  ┌──────────▼─────────┐                     │
│  │  Security   │  │  Language          │                     │
│  │  Analyzer   │  │  Analyzers         │                     │
│  │  (SAST)     │  │  (Rust, Go, TS)    │                     │
│  └─────────────┘  └────────────────────┘                     │
└──────────────────────────────────────────────────────────────┘
                              │
┌──────────────────────────────────────────────────────────────┐
│                 INFRASTRUCTURE LAYER                         │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐  │
│  │ PostgreSQL   │ │    Redis     │ │   NATS Messaging     │  │
│  │   Database   │ │    Cache     │ │                      │  │
│  └──────────────┘ └──────────────┘ └──────────────────────┘  │
│                              │                                │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────┐  │
│  │  File        │ │  Search      │ │   gRPC Workers       │  │
│  │  System      │ │  (Tantivy)   │ │                      │  │
│  └──────────────┘ └──────────────┘ └──────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

### Diagrama de Componentes

```
                    ┌─────────────────────────────────────┐
                    │        hodei-scan System            │
                    └─────────────────────────────────────┘
                                    │
        ┌───────────────────────────┼───────────────────────────┐
        │                           │                           │
┌───────▼──────────┐      ┌────────▼────────┐      ┌─────────▼─────────┐
│   CLI Client     │      │   Web Service   │      │  CI/CD Integrator │
│                  │      │   (Axum)        │      │                   │
│ • analyze        │      │                 │      │ • GitHub Actions  │
│ • sca            │      │ • REST API      │      │ • GitLab CI       │
│ • report         │      │ • WebSocket     │      │ • Bitbucket       │
└──────────────────┘      │ • Static Files  │      └───────────────────┘
                          └────────┬────────┘
                                   │
                    ┌──────────────▼──────────────┐
                    │     API Gateway Layer        │
                    │  ┌─────────┐ ┌───────────┐  │
                    │  │Auth/JWT │ │Rate Limit │  │
                    │  │Middleware│ │Middleware │  │
                    │  └─────────┘ └───────────┘  │
                    └──────────────┬──────────────┘
                                   │
                    ┌──────────────▼──────────────┐
                    │   Analysis Pipeline Core     │
                    └──────────────┬──────────────┘
                                   │
        ┌─────────────────────────┼─────────────────────────┐
        │                         │                         │
┌───────▼─────────────┐  ┌────────▼────────┐  ┌─────────────▼──────────┐
│   Language Parser   │  │ Cedar Rule      │  │   Security Engine     │
│                     │  │ Engine          │  │                       │
│ ┌─────┐ ┌────────┐ │  │                 │  │ ┌─────┐ ┌───────────┐ │
│ │Tree │ │OxC     │ │  │ • DSL Parser    │  │ │Taint│ │OWASP      │ │
│ │Sitter│ │Parser  │ │  │ • Rule Index   │  │ │Anal.│ │Top 10     │ │
│ │Rust │ │TS/JS  │ │  │ • Evaluator     │  │ │     │ │           │ │
│ │Go   │ └────────┘ │  │ • WASM Runtime  │  │ │     │ │CWE Top 25 │ │
│ └─────┘            │  └─────────────────┘  │ └─────┘ └───────────┘ │
└────────────────────┘                       └────────────────────────┘
        │                         │                         │
        └─────────────────────────┼─────────────────────────┘
                                  │
                    ┌─────────────▼─────────────┐
                    │      Data Layer           │
                    │ ┌─────────┐ ┌──────────┐ │
                    │ │PostgreSQL│ │  Redis   │ │
                    │ │  Main DB │ │  Cache   │ │
                    │ └─────────┘ └──────────┘ │
                    │ ┌─────────┐ ┌──────────┐ │
                    │ │  S3     │ │  NATS    │ │
                    │ │Reports  │ │Message   │ │
                    │ │         │ │Queue     │ │
                    │ └─────────┘ └──────────┘ │
                    └──────────────────────────┘
```

---

## 🔧 Componentes Core

### 1. Cedar-Inspired Rule Engine

**Propósito:** Motor de reglas determinista para static analysis

**Características:**
- Evaluación de reglas < 2ms
- Determinismo O(n) garantizado
- Paralelización con Rayon
- WASM sandbox para reglas custom

**Arquitectura Interna:**

```rust
// Core Rule Engine
pub struct RuleEngine {
    schema: AnalysisSchema,              // Schema-driven types
    rules: Arc<RwLock<HashMap<RuleId, Box<dyn StaticAnalysisRule>>>>,
    index: Arc<RuleIndex>,               // Fast rule slicing
    evaluation_pool: Arc<rayon::ThreadPool>,
    wasm_runtime: Option<WASMRuntime>,
}

// Rule Definition (DSL Cedar-inspired)
pub struct SonarRule {
    pub id: RuleId,
    pub effect: RuleEffect,              // Permit/Forbid
    pub scope: RuleScope,                // Node type + attributes
    pub conditions: Vec<Condition>,      // AST-based conditions
    pub context: ContextExpression,      // Project/file context
    pub metadata: RuleMetadata,          // Severity, message, fix
}

// Evaluation Flow
impl RuleEngine {
    pub async fn evaluate(
        &self,
        node: &ASTNode,
        context: &AnalysisContext
    ) -> Vec<RuleViolation> {
        // 1. Rule slicing: fast filter
        let relevant_rules = self.index.slice(context);

        // 2. Parallel evaluation
        let violations: Vec<RuleViolation> = relevant_rules
            .par_iter()
            .filter_map(|rule_id| {
                let rule = self.get_rule(rule_id);
                rule.evaluate(node, context)
            })
            .collect();

        violations
    }
}
```

**DSL de Reglas:**

```cedar
// Regla ejemplo: SQL Injection Detection
permit(
    rule: "GO_SQL_INJECTION",
    severity: "critical",
    category: "security"
) on {
    node_type: "binary_expr",
    operator: "+",
    context: {
        language == "go" &&
        right_operand: { node_type: "identifier" }
    }
} when {
    context.is_user_input(right_operand) &&
    context.is_sql_sink(parent_node)
}

forbid(
    rule: "RUST_UNSAFE_NO_COMMENT",
    severity: "warning"
) on {
    node_type: "unsafe_block",
    condition: { !has_safety_comment }
} when {
    context.language == "rust" &&
    context.complexity > 5
}
```

### 2. Language Analyzer Layer

**Propósito:** Análisis semántico por lenguaje específico

**Trait Base:**

```rust
pub trait LanguageAnalyzer: Send + Sync {
    type AST;
    type CFG;
    type DataFlowGraph;

    fn parse(&self, source: &str) -> Result<Self::AST, ParseError>;

    fn build_cfg(&self, ast: &Self::AST) -> Result<Self::CFG, BuildError>;

    fn dataflow_analysis(&self, cfg: &Self::CFG) -> Result<Self::DataFlowGraph, DFAError>;

    fn taint_tracking(&self, dfg: &Self::DataFlowGraph) -> Result<TaintResults, TaintError>;
}
```

**Implementación Rust:**

```rust
pub struct RustAnalyzer {
    parser: syn::Parser,
    cfg_builder: RustCFGBuilder,
    dfa: RustDFA,
    taint_tracker: RustTaintTracker,
}

impl LanguageAnalyzer for RustAnalyzer {
    type AST = RustAST;
    type CFG = RustCFG;
    type DataFlowGraph = RustDataFlowGraph;

    fn parse(&self, source: &str) -> Result<Self::AST, ParseError> {
        // 1. Parse with syn
        let syntax_tree = syn::parse_file(source)
            .map_err(|e| ParseError::SynError(e))?;

        // 2. Build AST with additional metadata
        Ok(RustAST {
            functions: syntax_tree.items
                .iter()
                .filter_map(|item| match item {
                    Item::Fn(func) => Some(RustFunction::from_syn(func)),
                    _ => None,
                })
                .collect(),
            structs: /* ... */,
            // ...
        })
    }

    fn build_cfg(&self, ast: &Self::AST) -> Result<Self::CFG, BuildError> {
        self.cfg_builder.build(ast)
    }

    fn dataflow_analysis(&self, cfg: &Self::CFG) -> Result<Self::DataFlowGraph, DFAError> {
        self.dfa.analyze(cfg)
    }

    fn taint_tracking(&self, dfg: &Self::DataFlowGraph) -> Result<TaintResults, TaintError> {
        self.taint_tracker.track_sources_to_sinks(dfg)
    }
}
```

### 3. Security Analysis Engine (SAST)

**Propósito:** Análisis de vulnerabilidades de seguridad

**Arquitectura:**

```rust
pub struct SecurityAnalyzer {
    rule_engine: Arc<RuleEngine>,
    taint_analyzer: TaintAnalyzer,
    framework_detector: FrameworkDetector,
}

impl SecurityAnalyzer {
    pub async fn analyze(&self, project: &Project) -> SecurityReport {
        // 1. Detect frameworks
        let frameworks = self.framework_detector.detect(project);

        // 2. Get relevant rules
        let security_rules = self.rule_engine.get_security_rules(&frameworks);

        // 3. Run taint analysis
        let taint_results = self.taint_analyzer.analyze(project);

        // 4. Evaluate rules
        let findings: Vec<SecurityFinding> = security_rules
            .par_iter()
            .flat_map(|rule| rule.check(project, &taint_results))
            .collect();

        SecurityReport {
            findings,
            owasp_coverage: self.calculate_owasp_coverage(&findings),
            risk_score: self.calculate_risk_score(&findings),
        }
    }
}

// Example: SQL Injection Rule
pub struct SQLInjectionRule {
    sink_patterns: Vec<Regex>,
    source_patterns: Vec<Regex>,
    sanitization_patterns: Vec<Regex>,
}

impl StaticAnalysisRule for SQLInjectionRule {
    fn check(&self, context: &AnalysisContext) -> Vec<Finding> {
        let sources = context.dfg.find_taint_sources()
            .filter(|s| self.is_user_input(s))
            .collect::<Vec<_>>();

        if sources.is_empty() {
            return vec![];
        }

        let taint_flow = context.dfg.taint_analysis(&sources);
        let sinks = context.dfg.find_sinks();

        taint_flow.check_violations(&sinks, &self.sanitization_patterns)
    }
}
```

### 4. Software Composition Analysis (SCA)

**Propósito:** Análisis de dependencias y CVEs

**Arquitectura:**

```rust
pub struct SCAEngine {
    dependency_resolvers: HashMap<Ecosystem, Box<dyn DependencyResolver>>,
    cve_scanner: CVEScanner,
    sbom_generator: SBOMGenerator,
    license_checker: LicenseChecker,
}

pub struct DependencyResolver {
    ecosystem: Ecosystem,        // npm, cargo, go mod, pip
    lockfile_parser: LockfileParser,
    version_resolver: VersionResolver,
}

impl SCAEngine {
    pub async fn analyze(&self, project: &Project) -> SCAResult {
        // 1. Resolve dependencies
        let dependencies = self.resolve_all_dependencies(project).await?;

        // 2. Check CVEs
        let cve_findings = self.cve_scanner.scan(&dependencies).await?;

        // 3. Generate SBOM
        let sbom = self.sbom_generator.generate(&dependencies)?;

        // 4. Check licenses
        let license_info = self.license_checker.check(&dependencies)?;

        SCAResult {
            dependencies,
            cve_findings,
            sbom,
            license_info,
        }
    }
}
```

### 5. Data Layer

**Database Schema:**

```sql
-- Projects
CREATE TABLE projects (
    id UUID PRIMARY KEY,
    name VARCHAR NOT NULL,
    repository_url VARCHAR,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Analysis Results
CREATE TABLE analysis_runs (
    id UUID PRIMARY KEY,
    project_id UUID REFERENCES projects(id),
    commit_hash VARCHAR,
    analysis_date TIMESTAMP DEFAULT NOW(),
    total_issues INTEGER,
    critical_issues INTEGER,
    major_issues INTEGER,
    coverage_percentage DECIMAL(5,2),
    technical_debt_hours INTEGER
);

-- Issues/Findings
CREATE TABLE issues (
    id UUID PRIMARY KEY,
    analysis_id UUID REFERENCES analysis_runs(id),
    file_path VARCHAR NOT NULL,
    line_number INTEGER,
    rule_id VARCHAR NOT NULL,
    severity issue_severity NOT NULL,
    message TEXT,
    debt_hours INTEGER
);

-- Dependencies
CREATE TABLE dependencies (
    id UUID PRIMARY KEY,
    project_id UUID REFERENCES projects(id),
    name VARCHAR NOT NULL,
    version VARCHAR NOT NULL,
    ecosystem VARCHAR NOT NULL,
    is_direct BOOLEAN DEFAULT TRUE
);

-- CVEs
CREATE TABLE cve_findings (
    id UUID PRIMARY KEY,
    dependency_id UUID REFERENCES dependencies(id),
    cve_id VARCHAR NOT NULL,
    severity issue_severity NOT NULL,
    cvss_score DECIMAL(3,1),
    fixed_version VARCHAR
);
```

---

## 🚀 Patrones Arquitectónicos

### 1. Hexagonal Architecture

**Ports (Interfaces):**

```rust
// Port: External API
pub trait AnalysisPort {
    async fn analyze_project(&self, project: &Project) -> Result<AnalysisResult>;
    async fn scan_dependencies(&self, project: &Project) -> Result<SCAResult>;
}

// Port: Storage
pub trait ProjectRepository {
    async fn save(&self, project: &Project) -> Result<()>;
    async fn get(&self, id: ProjectId) -> Result<Project>;
}

// Adapters
pub struct PostgresProjectRepository {
    db: Database,
}

#[async_trait]
impl ProjectRepository for PostgresProjectRepository {
    async fn save(&self, project: &Project) -> Result<()> {
        sqlx::query!("INSERT INTO projects ...")
            .execute(&self.db)
            .await?;
        Ok(())
    }
}
```

### 2. Actor Model para Parallel Processing

```rust
// Worker Actors
pub struct AnalysisWorker {
    rule_engine: Arc<RuleEngine>,
    language_analyzer: Arc<dyn LanguageAnalyzer>,
}

impl Actor for AnalysisWorker {
    type Message = AnalyzeFile;

    async fn handle(&mut self, msg: AnalyzeFile) -> Result<(), MailboxError> {
        let result = self.language_analyzer.parse(&msg.source_code)?;
        let cfg = self.language_analyzer.build_cfg(&result)?;
        let findings = self.rule_engine.evaluate(&result, &msg.context)?;

        msg.sender.send(findings).ok();
        Ok(())
    }
}

// Supervisor
pub struct AnalysisSupervisor {
    workers: Vec<Addr<AnalysisWorker>>,
    task_sender: mpsc::Sender<AnalyzeFile>,
}

impl AnalysisSupervisor {
    pub async fn analyze_project(&self, project: &Project) -> AnalysisResult {
        let files = self.collect_project_files(project);

        let futures = files.into_iter().map(|file| {
            self.task_sender.send(AnalyzeFile { file, sender: /* ... */ })
        });

        // Fan-out to workers
        futures::future::join_all(futures).await;

        // Collect results
        self.collect_results()
    }
}
```

### 3. Event-Driven Architecture

```rust
// Events
#[derive(Debug, Clone)]
pub enum AnalysisEvent {
    AnalysisCompleted(AnalysisId),
    SecurityIssueFound(IssueId, Severity),
    CVEFound(CVEInfo),
    QualityGateFailed(QualityGateId),
    CoverageDropped(CoverageDelta),
}

// Event Bus
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<String, Vec<Box<dyn EventSubscriber>>>>>,
}

impl EventBus {
    pub async fn publish(&self, event: &AnalysisEvent) {
        let topic = event.topic_name();
        let subscribers = self.subscribers.read().unwrap();

        if let Some(subs) = subscribers.get(&topic) {
            for subscriber in subs {
                subscriber.handle(event).await;
            }
        }
    }
}

// Subscribers
pub struct SlackNotifier {
    client: SlackClient,
}

#[async_trait]
impl EventSubscriber for SlackNotifier {
    async fn handle(&self, event: &AnalysisEvent) {
        if let AnalysisEvent::SecurityIssueFound(issue, severity) = event {
            if severity >= Severity::Critical {
                self.client.send_alert(issue).await;
            }
        }
    }
}
```

---

## 📊 Performance y Escalabilidad

### Benchmarks Target vs SonarQube

| Componente | SonarQube | hodei-scan v2.0 | Mejora |
|------------|-----------|-----------------|--------|
| **Parser (100K LOC)** | 120s | 30s | **4x** |
| **CFG Build** | 60s | 15s | **4x** |
| **DFA Analysis** | 90s | 20s | **4.5x** |
| **Rule Evaluation** | 10-20ms | <2ms | **5-10x** |
| **Full Analysis (1M LOC)** | 30 min | 15 min | **2x** |
| **Peak Memory** | 4GB | 800MB | **5x** |
| **Rule Throughput** | 1M/hr | 3M/hr | **3x** |

### Optimizations

**1. Paralelización con Rayon**

```rust
// Parallel file processing
use rayon::prelude::*;

pub async fn analyze_project_parallel(project: &Project) -> AnalysisResult {
    let files = collect_files(project);

    let results: Vec<AnalysisResult> = files
        .par_iter()  // Parallel iterator
        .map(|file| analyze_file(file))
        .collect();

    merge_results(results)
}
```

**2. In-Memory Caching**

```rust
// LRU Cache con Redis backend
pub struct AnalysisCache {
    local_cache: Arc<Mutex<LruCache<FileId, CachedAnalysis>>>,
    redis: Arc<Redis>,
    ttl: Duration,
}

impl AnalysisCache {
    pub fn get(&self, file_id: &FileId) -> Option<CachedAnalysis> {
        // Check local cache first
        if let Some(result) = self.local_cache.lock().unwrap().get(file_id) {
            return Some(result.clone());
        }

        // Check Redis
        let key = format!("analysis:{}", file_id);
        if let Some(cached) = self.redis.get(&key).await? {
            // Populate local cache
            self.local_cache.lock().unwrap().put(file_id.clone(), cached.clone());
            Some(cached)
        } else {
            None
        }
    }
}
```

**3. Streaming Processing**

```rust
// Process large files in chunks
pub async fn analyze_large_file(
    &self,
    file: &LargeFile
) -> Result<AnalysisResult> {
    let mut stream = file.chunks(1000);  // 1000 lines at a time
    let mut partial_results = Vec::new();

    while let Some(chunk) = stream.next().await {
        let result = self.analyze_chunk(&chunk)?;
        partial_results.push(result);
    }

    Ok(merge_partial_results(partial_results))
}
```

---

## 🔒 Seguridad

### WASM Sandbox

```rust
pub struct WASMSandbox {
    engine: wasmtime::Engine,
    linker: wasmtime::Linker,
}

impl WASMSandbox {
    pub fn new() -> Result<Self> {
        let engine = wasmtime::Engine::new(
            wasmtime::Config::new()
                .cranelift_opt_level(wasmtime::OptLevel::Speed)
                .max_wasm_stack(1024 * 1024)  // 1MB stack limit
        )?;

        let mut linker = wasmtime::Linker::new(&engine);

        // Restrict imports
        linker.func_wrap("env", "print", |_: wasmtime::Val| {
            // No-op - no external I/O
        })?;

        Ok(WASMSandbox { engine, linker })
    }

    pub fn execute_rule(
        &self,
        wasm_bytes: &[u8],
        context: &AnalysisContext
    ) -> Result<Vec<Finding>, WASMError> {
        let module = wasmtime::Module::new(&self.engine, wasm_bytes)?;
        let mut store = wasmtime::Store::new(&self.engine, context);

        let instance = self.linker.instantiate(&mut store, &module)?;
        let find_violations = instance.get_typed_func::<(), Vec<wasmtime::Val>>(&mut store, "find_violations")?;

        let results = find_violations.call(&mut store, ())?;

        // Parse results
        Ok(self.parse_results(results))
    }
}
```

### Audit Logging

```rust
pub struct AuditLogger {
    event_bus: EventBus,
    database: Database,
}

impl AuditLogger {
    pub async fn log_analysis(&self, analysis: &Analysis) {
        let event = AuditEvent {
            timestamp: Utc::now(),
            user_id: analysis.user_id,
            action: "ANALYSIS_RUN".to_string(),
            resource: analysis.project_id.to_string(),
            details: json!({
                "files_analyzed": analysis.files.len(),
                "issues_found": analysis.total_issues,
                "duration_ms": analysis.duration.as_millis()
            }),
        };

        // Store in database
        sqlx::query!("INSERT INTO audit_log ...")
            .execute(&self.database)
            .await?;

        // Publish event
        self.event_bus.publish(&event).await;
    }
}
```

---

## 📈 Monitoreo y Observabilidad

### Metrics

```rust
use metrics::{counter, gauge, histogram};

pub struct MetricsCollector {
    analysis_duration: Histogram,
    active_analyses: Gauge,
    issues_found: Counter,
    memory_usage: Gauge,
}

impl MetricsCollector {
    pub fn record_analysis_duration(&self, duration: Duration) {
        self.analysis_duration.record(duration.as_secs_f64());
    }

    pub fn increment_active_analyses(&self) {
        self.active_analyses.increment(1.0);
    }

    pub fn record_issues_found(&self, count: usize, severity: Severity) {
        self.issues_found.with_label_values(&[severity.as_str()]).inc();
    }
}
```

### Health Checks

```rust
pub struct HealthChecker {
    database: Database,
    cve_database: CVEDatabase,
    storage: Storage,
}

impl HealthChecker {
    pub async fn check_all(&self) -> HealthStatus {
        let checks = vec![
            self.check_database().await,
            self.check_cve_database().await,
            self.check_storage().await,
        ];

        let all_healthy = checks.iter().all(|c| c.healthy);

        HealthStatus {
            overall: if all_healthy { "healthy" } else { "unhealthy" },
            checks,
        }
    }
}
```

---

## 🛠️ Deployment Architecture

### Kubernetes Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: hodei-scan
spec:
  replicas: 3
  selector:
    matchLabels:
      app: hodei-scan
  template:
    metadata:
      labels:
        app: hodei-scan
    spec:
      containers:
      - name: api
        image: hodei-scan:latest
        ports:
        - containerPort: 8080
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: hodei-secrets
              key: database-url
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
```

### Docker Configuration

```dockerfile
# Dockerfile
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/hodei-scan /usr/local/bin/
COPY --from=builder /app/config /etc/hodei-scan

EXPOSE 8080
CMD ["hodei-scan", "serve"]
```

---

## 🔄 Continuous Integration

### GitHub Actions Pipeline

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - name: Run Tests
      run: cargo test --all -- --test-threads=1
    - name: Run Integration Tests
      run: cargo test --test integration -- --test-threads=1
    - name: Coverage
      run: |
        cargo install cargo-tarpaulin
        cargo tarpaulin --all --out xml

  security:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - name: Security Audit
      run: |
        cargo install cargo-audit
        cargo audit
    - name: License Check
      run: |
        cargo install cargo-license
        cargo license --json > licenses.json
```

---

## 📚 ADRs (Architecture Decision Records)

### ADR-001: Cedar-Inspired Rule Engine

**Status:** Accepted
**Date:** 2025-11-10

**Context:** Necesitamos un motor de reglas rápido y determinista para static analysis.

**Decision:** Usar Cedar-inspired approach con:
- DSL declarativo para reglas
- Indexación por node type
- Evaluación paralela con Rayon
- WASM sandbox para extensibilidad

**Consequences:**
- ✅ Performance: <2ms rule evaluation
- ✅ Determinism: O(n) time complexity
- ✅ Extensibility: WASM rules
- ❌ Complexity: Higher implementation complexity
- ❌ Learning curve: DSL to learn

### ADR-002: Elimination of LSP Dependency

**Status:** Accepted
**Date:** 2025-11-10

**Context:** Identificamos contradicción arquitectónica entre LSP (IDE integration) y batch analysis.

**Decision:** Eliminar todas las referencias a LSPs del motor core, usar únicamente:
- tree-sitter para parsing
- Motores semánticos específicos por lenguaje
- Análisis batch stateless

**Consequences:**
- ✅ Arquitectura coherente
- ✅ Performance optimizada para batch
- ✅ Simplicidad
- ❌ No IDE real-time integration
- ❌ Análisis más simple que CodeQL

### ADR-003: Hexagonal Architecture

**Status:** Accepted
**Date:** 2025-11-10

**Context:** Sistema complejo con múltiples integraciones externas.

**Decision:** Implementar Hexagonal Architecture con:
- Ports (interfaces) para external dependencies
- Adapters para implementaciones específicas
- Dependency inversion para testability

**Consequences:**
- ✅ Testability alta
- ✅ Modularidad
- ✅ Flexibility para cambiar implementaciones
- ❌ Más boilerplate
- ❌ Complejidad adicional

---

## 🔮 Roadmap Técnico

### Fase 1: MVP (Meses 1-6)
- [ ] 3 lenguajes (Rust, Go, TypeScript)
- [ ] Core engine sin LSP
- [ ] OWASP Top 10
- [ ] Performance 2x vs SonarQube

### Fase 2: Expansion (Meses 7-12)
- [ ] +3 lenguajes (Python, C++, Java)
- [ ] SCA engine completo
- [ ] Code coverage integration
- [ ] Quality gates

### Fase 3: Enterprise (Meses 13-24)
- [ ] Portfolio management
- [ ] PR decoration
- [ ] Enterprise features (RBAC, SSO)
- [ ] Compliance reporting

---

## 📞 Contacto

**Chief Architect:** [A definir]
**Technical Lead:** [A definir]
**Architecture Review Board:** architecture@hodei-scan.dev
**Slack:** #hodei-scan-architecture

---

*Última actualización: 10 de noviembre de 2025*

**Next Document:** [TDD Methodology](./TDD_METHODOLOGY.md)
