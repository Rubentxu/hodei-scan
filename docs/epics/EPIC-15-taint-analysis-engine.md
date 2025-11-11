# EPIC-15: Taint Analysis Engine - Deep Extractors

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: hodei-scan v3.3  
**Dependencias**: EPIC-10 (ExtractorOrchestrator), EPIC-11 (IR Schema Evolution)  
**Owner**: Security Analysis Team  
**Prioridad**: Medium Path (Fase 3)

---

## 1. Resumen Ejecutivo

Implementar **`hodei-taint-engine`**, un motor de análisis de flujo de datos que permite construir **extractores profundos** para detectar vulnerabilidades complejas como SQL Injection, XSS, y Path Traversal. Esta épica habilita el **Nivel 3** de la estrategia de extractores.

### Objetivo de Negocio
Proporcionar detección de vulnerabilidades **de nivel empresarial** sin necesidad de que cada extractor reimplemente algoritmos complejos de análisis de flujo.

### Métricas de Éxito
- **Precisión**: <5% false positive rate para vulnerabilidades críticas
- **Performance**: Análisis de proyectos grandes (100K+ LOC) en <5 minutos
- **Cobertura**: 10+ vulnerabilidades críticas detectables
- **Extensibilidad**: Nuevo extractor de taint en <1 semana

---

## 2. Contexto Técnico

### 2.1. Problema Actual
Detectar vulnerabilidades complejas requiere:
- Análisis de flujo de datos (data flow analysis)
- Construcción de Control Flow Graphs (CFG)
- Propagación de "taint" (datos no confiables)
- Identificación de sources, sinks, y sanitizers
- **Muy complejo** de implementar por extractor

### 2.2. Solución: Motor de Taint Genérico

```
┌─────────────────────────────────────────────────────────────────┐
│              hodei-taint-engine Architecture                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────┐    ┌──────────────────┐                  │
│  │  hodei-taint-    │    │   Language       │                  │
│  │  engine Core     │    │   Extractors     │                  │
│  │                  │    │                  │                  │
│  │ • CFG Builder    │◄───│ • Java Taint     │                  │
│  │ • Data Flow      │    │ • Python Taint   │                  │
│  │ • Taint Prop.    │    │ • JavaScript     │                  │
│  │ • Rules Engine   │    │ • ...            │                  │
│  └────────┬─────────┘    └──────────────────┘                  │
│           │                                                     │
│           ▼                                                     │
│  ┌────────────────────────────────────────────────────────┐   │
│  │              Vulnerability Detection                   │   │
│  │                                                        │   │
│  │  Sources → [User Input]                               │   │
│  │    ↓                                                  │   │
│  │  Propagation → [Through functions, variables]         │   │
│  │    ↓                                                  │   │
│  │  Sinks → [SQL Query, eval(), file_write()]           │   │
│  │    ↓                                                  │   │
│  │  ALERT: Tainted data reached sensitive sink!         │   │
│  └────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Arquitectura Detallada

### 3.1. hodei-taint-engine Core

#### Componentes Principales
```rust
pub struct TaintEngine {
    cfg_builder: ControlFlowGraphBuilder,
    dataflow_analyzer: DataFlowAnalyzer,
    taint_propagator: TaintPropagator,
    sanitizer_detector: SanitizerDetector,
}

pub struct TaintConfig {
    pub language: Language,
    pub sources: Vec<Source>,
    pub sinks: Vec<Sink>,
    pub sanitizers: Vec<Sanitizer>,
    pub rules: Vec<TaintRule>,
}

impl TaintEngine {
    pub fn analyze(&self, ast: &AST, config: &TaintConfig) -> Result<Vec<TaintFinding>> {
        // 1. Build Control Flow Graph
        let cfg = self.cfg_builder.build(ast)?;
        
        // 2. Identify sources (user input)
        let sources = self.identify_sources(&cfg, &config.sources)?;
        
        // 3. Propagate taint through data flow
        let taint_flows = self.taint_propagator.propagate(&cfg, &sources)?;
        
        // 4. Check if taint reaches sinks (vulnerable points)
        let findings = self.detect_vulnerabilities(&taint_flows, &config.sinks)?;
        
        // 5. Check for sanitizers that neutralize taint
        let findings = self.apply_sanitizers(&findings, &config.sanitizers)?;
        
        Ok(findings)
    }
}
```

#### Control Flow Graph Builder
```rust
pub struct ControlFlowGraphBuilder {
    language: Language,
}

impl ControlFlowGraphBuilder {
    pub fn build(&self, ast: &AST) -> Result<ControlFlowGraph> {
        let mut cfg = ControlFlowGraph::new();
        
        // 1. Create nodes for each statement
        for (index, statement) in ast.statements.iter().enumerate() {
            let node = CFGNode::new(index, statement.clone());
            cfg.add_node(node);
        }
        
        // 2. Create edges for control flow
        for (index, statement) in ast.statements.iter().enumerate() {
            let successors = self.get_successors(statement);
            for successor in successors {
                cfg.add_edge(index, successor)?;
            }
        }
        
        Ok(cfg)
    }
    
    fn get_successors(&self, stmt: &Stmt) -> Vec<usize> {
        match stmt {
            Stmt::If { then_branch, else_branch, .. } => {
                vec![then_branch.end, else_branch.end]
            }
            Stmt::While { body, .. } => {
                vec![body.end, body.following]
            }
            Stmt::Call { target, .. } => {
                vec![target + 1]
            }
            _ => vec![],
        }
    }
}
```

#### Taint Propagation Engine
```rust
pub struct TaintPropagator {
    taint_tracker: TaintTracker,
}

impl TaintPropagator {
    pub fn propagate(
        &self,
        cfg: &ControlFlowGraph,
        sources: &[SourceLocation],
    ) -> Result<Vec<TaintFlow>> {
        let mut flows = Vec::new();
        let mut visited_nodes = HashSet::new();
        
        // Start from each source
        for source in sources {
            let mut worklist = vec![source.node_id];
            
            while let Some(node_id) = worklist.pop() {
                if visited_nodes.contains(&node_id) {
                    continue;
                }
                visited_nodes.insert(node_id);
                
                let node = cfg.get_node(node_id);
                
                // Propagate taint through this node
                let taint_out = self.propagate_through_node(node, &source.taint)?;
                
                // Add to flows
                for taint in &taint_out {
                    flows.push(TaintFlow {
                        source: source.clone(),
                        path: self.reconstruct_path(cfg, source.node_id, node_id)?,
                        sink: taint.location.clone(),
                        taint,
                    });
                }
                
                // Continue propagation to successors
                for successor in cfg.get_successors(node_id) {
                    worklist.push(successor);
                }
            }
        }
        
        Ok(flows)
    }
    
    fn propagate_through_node(
        &self,
        node: &CFGNode,
        taint_in: &[TaintedValue],
    ) -> Result<Vec<TaintedValue>> {
        match &node.statement {
            Stmt::Assignment { target, source, .. } => {
                // If source is tainted, target becomes tainted
                let mut taint_out = taint_in.to_vec();
                if self.is_tainted(source, taint_in) {
                    taint_out.push(TaintedValue {
                        variable: target.clone(),
                        location: node.location.clone(),
                        source: taint_in[0].source.clone(),
                    });
                }
                Ok(taint_out)
            }
            Stmt::Call { function, args, .. } => {
                // Check if function is a sanitizer
                if self.is_sanitizer(function) {
                    // Taint is neutralized
                    Ok(vec![])
                } else {
                    // Propagate taint through function call
                    self.propagate_through_function_call(node, taint_in)
                }
            }
            _ => Ok(taint_in.to_vec()),
        }
    }
}
```

### 3.2. Language-Specific Extractors

#### Java Taint Extractor
```rust
pub struct JavaTaintExtractor {
    engine: TaintEngine,
}

impl JavaTaintExtractor {
    pub fn new() -> Self {
        let config = TaintConfig {
            language: Language::Java,
            sources: vec![
                Source {
                    pattern: "request.getParameter".to_string(),
                    kind: SourceKind::HttpParameter,
                },
                Source {
                    pattern: "System.getenv".to_string(),
                    kind: SourceKind::EnvironmentVariable,
                },
            ],
            sinks: vec![
                Sink {
                    pattern: "executeQuery".to_string(),
                    kind: SinkKind::SqlQuery,
                    severity: Severity::Critical,
                },
                Sink {
                    pattern: "out.println".to_string(),
                    kind: SinkKind::XssSink,
                    severity: Severity::High,
                },
            ],
            sanitizers: vec![
                Sanitizer {
                    pattern: "escapeHtml".to_string(),
                    kind: SanitizerKind::OutputEncoding,
                },
                Sanitizer {
                    pattern: "PreparedStatement".to_string(),
                    kind: SanitizerKind::PreparedStatement,
                },
            ],
            rules: vec![
                TaintRule {
                    id: "SQL_INJECTION".to_string(),
                    description: "Potential SQL Injection".to_string(),
                    severity: Severity::Critical,
                },
            ],
        };
        
        Self {
            engine: TaintEngine::new(config),
        }
    }
    
    pub async fn extract(&self, java_files: &[Path]) -> Result<IR> {
        let mut findings = Vec::new();
        
        for file in java_files {
            let ast = self.parse_java_file(file)?;
            let file_findings = self.engine.analyze(&ast, &self.engine.config)?;
            
            for finding in file_findings {
                let fact = self.finding_to_fact(finding, file)?;
                findings.push(fact);
            }
        }
        
        Ok(IR { facts: findings })
    }
}
```

#### Python Taint Extractor
```rust
pub struct PythonTaintExtractor {
    engine: TaintEngine,
}

impl PythonTaintExtractor {
    pub fn new() -> Self {
        let config = TaintConfig {
            language: Language::Python,
            sources: vec![
                Source {
                    pattern: "request.args.get".to_string(),
                    kind: SourceKind::HttpParameter,
                },
                Source {
                    pattern: "os.environ".to_string(),
                    kind: SourceKind::EnvironmentVariable,
                },
            ],
            sinks: vec![
                Sink {
                    pattern: "cursor.execute".to_string(),
                    kind: SinkKind::SqlQuery,
                    severity: Severity::Critical,
                },
                Sink {
                    pattern: "eval".to_string(),
                    kind: SinkKind::CodeInjection,
                    severity: Severity::Critical,
                },
            ],
            sanitizers: vec![
                Sanitizer {
                    pattern: "escape".to_string(),
                    kind: SanitizerKind::OutputEncoding,
                },
            ],
            rules: vec![],
        };
        
        Self {
            engine: TaintEngine::new(config),
        }
    }
}
```

### 3.3. Vulnerability Detection Rules

#### SQL Injection Detection
```rust
pub struct SqlInjectionDetector {
    sql_keywords: HashSet<String>,
    dangerous_functions: HashSet<String>,
}

impl SqlInjectionDetector {
    pub fn detect(&self, taint_flow: &TaintFlow) -> Option<Vulnerability> {
        // Check if taint reaches SQL sink
        if taint_flow.sink.kind == SinkKind::SqlQuery {
            // Check if there's a sanitizer (PreparedStatement)
            let has_sanitizer = taint_flow.path.iter().any(|node| {
                self.is_sanitizer(node)
            });
            
            if !has_sanitizer {
                return Some(Vulnerability {
                    id: "SQL_INJECTION".to_string(),
                    severity: Severity::Critical,
                    confidence: Confidence::High,
                    message: format!(
                        "Tainted data from {} reaches SQL sink without sanitization",
                        taint_flow.source.kind
                    ),
                    location: taint_flow.sink.location.clone(),
                    cwe: Some("CWE-89".to_string()),
                });
            }
        }
        
        None
    }
}
```

#### XSS Detection
```rust
pub struct XssDetector {
    html_output_functions: HashSet<String>,
    json_output_functions: HashSet<String>,
}

impl XssDetector {
    pub fn detect(&self, taint_flow: &TaintFlow) -> Option<Vulnerability> {
        // Check if taint reaches HTML output
        if self.is_html_output(&taint_flow.sink) {
            let has_encoding = taint_flow.path.iter().any(|node| {
                self.is_encoding_sanitizer(node)
            });
            
            if !has_encoding {
                return Some(Vulnerability {
                    id: "XSS".to_string(),
                    severity: Severity::High,
                    confidence: Confidence::Medium,
                    message: "Unescaped user input in HTML output".to_string(),
                    location: taint_flow.sink.location.clone(),
                    cwe: Some("CWE-79".to_string()),
                });
            }
        }
        
        None
    }
}
```

---

## 4. Plan de Implementación

### 4.1. Fases

**Fase 1: Core Engine (Semana 1-3)**
- [ ] CFG Builder
- [ ] Data Flow Analyzer
- [ ] Basic Taint Propagation
- [ ] Source/Sink detection

**Fase 2: Java Extractor (Semana 4-5)**
- [ ] Java AST parser integration
- [ ] Java-specific sources/sinks/sanitizers
- [ ] SQL Injection detection
- [ ] XSS detection

**Fase 3: Python Extractor (Semana 6)**
- [ ] Python AST parser integration
- [ ] Python-specific taint rules
- [ ] Command injection detection

**Fase 4: Advanced Features (Semana 7-8)**
- [ ] Inter-procedural analysis
- [ ] Taint summary generation
- [ ] Performance optimization

---

## 5. User Stories

### US-15.01: Taint Engine Core Implementation

**Como:** Developer de Extractores  
**Quiero:** Un motor genérico de análisis de taint  
**Para:** No reimplementar algoritmos complejos en cada extractor

**Criterios de Aceptación:**
- [ ] Control Flow Graph builder funcional
- [ ] Data flow analysis implementado
- [ ] Taint propagation through functions
- [ ] Source/Sink detection
- [ ] Performance <5 min for 100K LOC

**TDD - Red:**
```rust
#[test]
fn test_taint_propagation() {
    let cfg = build_simple_cfg();
    let source = SourceLocation {
        node_id: 0,
        variable: "user_input".to_string(),
        location: Location::new("file.rs", 10, 5),
    };
    
    let propagator = TaintPropagator::new();
    let flows = propagator.propagate(&cfg, &[source]).unwrap();
    
    assert_eq!(flows.len(), 1);
    assert_eq!(flows[0].sink.node_id, 3);
}
```

**TDD - Green:**
```rust
impl TaintPropagator {
    pub fn propagate(
        &self,
        cfg: &ControlFlowGraph,
        sources: &[SourceLocation],
    ) -> Result<Vec<TaintFlow>> {
        let mut flows = Vec::new();
        
        for source in sources {
            let taint = TaintedValue {
                variable: source.variable.clone(),
                source: source.clone(),
            };
            
            // Traverse CFG
            let mut current_nodes = vec![source.node_id];
            let mut visited = HashSet::new();
            
            while let Some(node_id) = current_nodes.pop() {
                if visited.contains(&node_id) {
                    continue;
                }
                visited.insert(node_id);
                
                // Propagate to successors
                for successor in cfg.get_successors(node_id) {
                    current_nodes.push(successor);
                    
                    flows.push(TaintFlow {
                        source: source.clone(),
                        path: vec![node_id, successor],
                        sink: CFGNodeRef {
                            node_id: successor,
                            taint: taint.clone(),
                        },
                    });
                }
            }
        }
        
        Ok(flows)
    }
}
```

**Conventional Commit:**
`feat(taint): implement core taint analysis engine`

---

### US-15.02: Java Taint Extractor

**Como:** Security Analyst  
**Quiero:** Detectar SQL Injection en código Java  
**Para:** Identificar vulnerabilidades críticas en aplicaciones Java

**Criterios de Aceptación:**
- [ ] Parse Java source files
- [ ] Detect user input sources (request.getParameter, etc.)
- [ ] Track data flow to SQL queries
- [ ] Alert on unsafe SQL usage
- [ ] Detect PreparedStatement sanitization

**TDD - Red:**
```rust
#[test]
fn test_java_sql_injection() {
    let java_code = r#"
        String userId = request.getParameter("id");
        String query = "SELECT * FROM users WHERE id = " + userId;
        Statement stmt = connection.createStatement();
        ResultSet rs = stmt.executeQuery(query);
    "#;
    
    let extractor = JavaTaintExtractor::new();
    let ir = extractor.extract_from_code(java_code).unwrap();
    
    assert!(ir.facts.iter().any(|f| matches!(f.fact_type, FactType::Vulnerability(_))));
}
```

**TDD - Green:**
```rust
impl JavaTaintExtractor {
    pub fn extract_from_code(&self, code: &str) -> Result<IR> {
        let ast = self.parse_java(code)?;
        let findings = self.engine.analyze(&ast, &self.engine.config)?;
        
        let facts = findings
            .into_iter()
            .map(|f| self.finding_to_fact(f))
            .collect();
        
        Ok(IR { facts })
    }
    
    fn finding_to_fact(&self, finding: TaintFinding) -> Fact {
        Fact {
            fact_type: FactType::Vulnerability {
                vuln_type: finding.id,
                severity: finding.severity,
                cwe: finding.cwe,
            },
            location: finding.location,
            message: finding.message,
            confidence: finding.confidence,
            metadata: HashMap::new(),
        }
    }
}
```

**Conventional Commit:**
`feat(taint): implement Java taint extractor for SQL injection`

---

### US-15.03: Python Taint Extractor

**Como:** Security Analyst  
**Quiero:** Detectar vulnerabilidades en aplicaciones Python  
**Para:** Cobertura completa de vulnerabilidades Python

**Criterios de Aceptación:**
- [ ] Parse Python source files
- [ ] Detect Flask/Django inputs
- [ ] Track to SQLAlchemy, Psycopg2
- [ ] Command injection detection
- [ ] Code injection (eval, exec) detection

**TDD - Red:**
```rust
#[test]
fn test_python_code_injection() {
    let python_code = r#"
        user_input = request.args.get('code')
        result = eval(user_input)
    "#;
    
    let extractor = PythonTaintExtractor::new();
    let ir = extractor.extract_from_code(python_code).unwrap();
    
    assert!(ir.facts.iter().any(|f| {
        matches!(f.fact_type, FactType::Vulnerability(_))
    }));
}
```

**Conventional Commit:**
`feat(taint): implement Python taint extractor for code injection`

---

### US-15.04: Advanced Taint Rules

**Como:** Security Researcher  
**Quiero:** Extender detección a nuevas vulnerabilidades  
**Para:** Mantener cobertura de amenazas actual

**Criterios de Aceptación:**
- [ ] Path Traversal detection
- [ ] SSRF (Server-Side Request Forgery)
- [ ] Deserialization vulnerabilities
- [ ] XXE (XML External Entity)
- [ ] Command injection

**Custom Rule Definition:**
```yaml
# custom_taint_rules.yaml
rules:
  - id: "PATH_TRAVERSAL"
    description: "Potential path traversal vulnerability"
    severity: "High"
    pattern:
      source: "request.args.get('path')"
      sink: "open"
      sanitizer: "os.path.basename"
    message: "User-controlled path reaches file operation"
```

**Conventional Commit:**
`feat(taint): add advanced vulnerability detection rules`

---

### US-15.05: Inter-Procedural Analysis

**Como:** Senior Security Engineer  
**Quiero:** Análisis a través de funciones y clases  
**Para:** Detectar vulnerabilidades en código modular

**Criterios de Aceptación:**
- [ ] Function call tracking
- [ ] Parameter taint propagation
- [ ] Return value tracking
- [ ] Class method analysis
- [ ] Inheritance handling

**Implementation:**
```rust
pub struct InterProceduralAnalyzer {
    call_graph: CallGraph,
    summary_cache: HashMap<FunctionId, TaintSummary>,
}

impl InterProceduralAnalyzer {
    pub fn analyze_function(&mut self, function: &Function) -> Result<TaintSummary> {
        // Check if we have a cached summary
        if let Some(summary) = self.summary_cache.get(&function.id) {
            return Ok(summary.clone());
        }
        
        // Analyze function body
        let mut summary = TaintSummary::new();
        
        for param in &function.parameters {
            // Parameters from external calls are tainted
            if self.is_external_call(param.source) {
                summary.add_tainted_param(param.name.clone());
            }
        }
        
        // Analyze statements
        for stmt in &function.body {
            let stmt_summary = self.analyze_statement(stmt)?;
            summary.merge(stmt_summary);
        }
        
        // Cache the summary
        self.summary_cache.insert(function.id, summary.clone());
        
        Ok(summary)
    }
}
```

**Conventional Commit:**
`feat(taint): implement inter-procedural taint analysis`

---

## 6. Testing Strategy

### 6.1. Unit Tests
- CFG building algorithms
- Taint propagation logic
- Source/sink detection
- Sanitizer identification

### 6.2. Integration Tests
- End-to-end Java SQL injection
- End-to-end Python code injection
- False positive validation

### 6.3. Security Validation Tests
- Test con vulnerable applications (DVWA, WebGoat)
- Benchmark against industry tools (SonarQube, Semgrep)
- Performance testing on large codebases

---

## 7. Riesgos y Mitigaciones

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| Performance overhead | Alto | Medio | Incremental analysis + caching |
| False positives | Alto | Alto | Sanitizer detection + tuning |
| Language coverage | Medio | Alto | Community-driven extractors |
| Maintenance complexity | Alto | Medio | Modular architecture |

---

## 8. Definition of Done

- [ ] hodei-taint-engine core functional
- [ ] Java extractor detecting SQL Injection
- [ ] Python extractor detecting code injection
- [ ] Performance <5 min for 100K LOC
- [ ] False positive rate <5%
- [ ] Documentation and examples
- [ ] Integration with ExtractorOrchestrator

---

**Estimación Total**: 8 Sprints (16 semanas)  
**Commit Messages**:  
- `feat(taint): implement core taint analysis engine`  
- `feat(taint): add Java taint extractor`  
- `feat(taint): add Python taint extractor`  
- `feat(taint): implement inter-procedural analysis`  
- `feat(taint): add advanced vulnerability rules`  

---

**Referencias Técnicas**:
- Data Flow Analysis: https://en.wikipedia.org/wiki/Data-flow_analysis
- Taint Checking: https://en.wikipedia.org/wiki/Taint_checking
- Static Single Assignment: https://en.wikipedia.org/wiki/Static_single_assignment_form
- Control Flow Graph: https://en.wikipedia.org/wiki/Control_flow_graph
