# Inventario de Implementaciones para Extractores de Nivel 3

**Fecha:** 2025-11-13  
**Proyecto:** hodei-scan v3.2  
**Análisis:** Estado actual vs. Requerimientos EPIC-20

---

## 📊 Resumen Ejecutivo

**RESULTADO:** El proyecto ya tiene **fundamentos sólidos** para extractores de Nivel 3, pero **falta la integración** y algunas piezas clave. Se estima que hay un **40-50% de implementación** ya disponible.

---

## ✅ **LO QUE YA ESTÁ IMPLEMENTADO**

### 1. **hodei-pattern-engine** (Completamente implementado)

**Ubicación:** `crates/hodei-pattern-engine/`

**Funcionalidades:**
- ✅ **Tree-sitter integration** con cache LRU
- ✅ **YAML Rule Engine** para reglas declarativas
- ✅ **Query compilation** y ejecución
- ✅ **Match transformation** a Facts
- ✅ **Batch processing** paralelo
- ✅ **Testing framework** para reglas
- ✅ **Multi-lenguaje** (Java, Python, Rust con features)

**Código clave:**
```rust
// tree_sitter.rs - QueryCache implementado
pub struct QueryCache {
    cache: HashMap<String, CacheEntry>,
    max_size: usize,
    access_counter: u64,
}

// yaml_rule.rs - Reglas declarativas
pub struct YamlRule {
    pub pattern: String,
    pub language: Language,
    pub fact_type: FactType,
}
```

**Valor:** Este es el **Tier 2** ya completo y funcional.

---

### 2. **hodei-declarative-extractors** (Parcialmente implementado)

**Ubicación:** `crates/hodei-declarative-extractors/`

**Funcionalidades:**
- ✅ **MultiLanguageParser** con ASTNode
- ✅ **Language enum** (Python, JavaScript, TypeScript, Rust, Go, Java, C, C++)
- ✅ **ParseResult** con metadata
- ✅ **ParseMetrics** para performance
- ✅ **Language warming** (parsers pre-inicializados)

**Código clave:**
```rust
// tree_sitter.rs
pub enum Language {
    Python,
    JavaScript,
    TypeScript,
    Rust,
    Go,
    Java,
    C,
    Cpp,
}

pub struct ASTNode {
    pub node_type: String,
    pub text: String,
    pub start_position: usize,
    pub end_position: usize,
    pub children: Vec<ASTNode>,
}
```

**Valor:** **Infraestructura de parsing** lista, pero **falta integración real con tree-sitter** (actualmente usa AST simple).

---

### 3. **hodei-engine - FlowIndex** (Implementación avanzada)

**Ubicación:** `crates/hodei-engine/src/store/flow_index.rs`

**Funcionalidades:**
- ✅ **petgraph::DiGraph** para tracking de flujo
- ✅ **Algoritmos implementados:** astar, dijkstra
- ✅ **Reachable queries** - "desde este fact, ¿cuáles son alcanzables?"
- ✅ **Shortest path** - "ruta más corta entre dos facts"
- ✅ **Flow ID grouping** - Agrupa facts por FlowId
- ✅ **Tests unitarios** implementados

**Código clave:**
```rust
pub struct FlowIndex {
    graph: DiGraph<FactId, ()>,
    fact_to_node: HashMap<FactId, NodeIndex>,
    flow_to_facts: HashMap<FlowId, Vec<FactId>>,
}

impl FlowIndex {
    pub fn reachable_from(&self, fact_id: FactId) -> Vec<FactId> {
        let distances = dijkstra(&self.graph, start_node, None, |_| 1);
        distances.keys().map(|&node| self.graph[node]).collect()
    }

    pub fn shortest_path(&self, from: FactId, to: FactId) -> Option<Vec<FactId>> {
        astar(&self.graph, *start_node, |n| n == *end_node, |_| 1, |_| 0)
    }
}
```

**Valor:** **¡EXCELENTE!** Esta es la **base perfecta** para Taint Analysis. Solo falta integrar con `datafrog`.

**Pendiente:** No usa `datafrog` aún, implementa su propio grafo.

---

### 4. **hodei-ir - Schema** (Completamente implementado)

**Ubicación:** `crates/hodei-ir/schema/facts.capnp`

**Tipos ya definidos:**
- ✅ **TaintSource** - `var`, `flowId`, `sourceType`, `confidence`
- ✅ **TaintSink** - `func`, `consumesFlow`, `category`, `severity`
- ✅ **Sanitization** - `method`, `sanitizesFlow`, `effective`, `confidence`
- ✅ **FlowId** - Type-safe UUID wrapper
- ✅ **Confidence** - Type-safe con validación
- ✅ **SourceLocation** - Ubicación precisa en código

**Código clave:**
```capnp
struct TaintSource {
  var @0 :VariableName;
  flowId @1 :FlowId;
  sourceType @2 :Text;
  confidence @3 :Confidence;
}

struct TaintSink {
  func @4 :FunctionName;
  consumesFlow @5 :FlowId;
  category @6 :Text;
  severity @7 :Severity;
}
```

**Valor:** **Schema perfecto** para Taint Analysis. Listo para usar.

---

### 5. **Documentación EPIC-15** (Draft disponible)

**Ubicación:** `docs/epics/EPIC-15-taint-analysis-engine.md`

**Contenido:**
- ✅ **Arquitectura propuesta** del motor de Taint
- ✅ **Diagrama conceptual** (fuentes → propagación → sinks)
- ✅ **Pseudocódigo** de ControlFlowGraphBuilder
- ✅ **Estructura TaintConfig** con sources, sinks, sanitizers

**Estado:** **Documentación como referencia**, no código implementado.

---

## ❌ **LO QUE FALTA POR IMPLEMENTAR**

### 1. **hodei-deep-analysis-engine** (Librería principal - NO EXISTE)

**Estado:** **No existe el crate**

**Requerido:**
```rust
// Crate nuevo: crates/hodei-deep-analysis-engine/Cargo.toml
[dependencies]
datafrog = "2.0.1"          # ⚠️ NO INCLUIDO AÚN
petgraph = { workspace = true }  # ✅ YA DISPONIBLE
tree-sitter = "0.25"        # ⚠️ NO INCLUIDO AÚN
hodei-ir = { path = "../hodei-ir" }
```

**A implementar:**
- ❌ **SemanticModel** struct
- ❌ **TaintPropagator** con datafrog
- ❌ **ConnascenceAnalyzer**
- ❌ **Policy manager** (TOML config)

---

### 2. **TaintPropagator con datafrog** (NO IMPLEMENTADO)

**Estado actual:** FlowIndex usa petgraph directamente

**Falta:**
```rust
// NUEVO: Implementación con datafrog
pub struct TaintPropagator {
    iteration: Iteration<'static>,
    sources: Variable<(FlowId, VariableName)>,
    sinks: Variable<(FlowId, SinkCategory)>,
    sanitizers: Variable<(FlowId, SanitizationMethod)>,
    tainted: Variable<FlowId>,
}

impl TaintPropagator {
    pub fn run_analysis(
        &mut self,
        dfg: &DataFlowGraph,
        policy: &TaintPolicy,
    ) -> Result<Vec<TaintFlow>, TaintAnalysisError> {
        // Reglas Datalog usando datafrog
        while self.iteration.changed() {
            // Regla: tainted(Flow) :- source(Flow, Var), flows_to(Var, Sink)
            self.tainted.from_join(&self.sources, &self.sinks, |...| ...);
        }
        Ok(self.extract_results()?)
    }
}
```

---

### 3. **ConnascenceAnalyzer** (NO IMPLEMENTADO)

**Estado:** **No existe**

**Falta implementar:**
```rust
pub struct ConnascenceAnalyzer {
    semantic_model: Arc<SemanticModel>,
}

impl ConnascenceAnalyzer {
    pub fn detect_positional_connascence(&self) -> Vec<CouplingFinding> {
        // Detectar funciones con 3+ parámetros del mismo tipo
    }

    pub fn detect_meaning_connascence(&self) -> Vec<CouplingFinding> {
        // Detectar valores mágicos repetidos
    }

    pub fn detect_type_connascence(&self) -> Vec<CouplingFinding> {
        // Detectar dependencias de tipo fuerte
    }
}
```

---

### 4. **SemanticModel Builder** (Parcial)

**Estado actual:** `hodei-declarative-extractors` tiene AST básico

**Falta:**
```rust
pub struct SemanticModel {
    cfg: Graph<BasicBlock, ControlEdge, petgraph::Directed, u32>,
    dfg: CsrGraph<DataNode, DataEdge>,
    scope_tree: ScopeTree,
    coupling_graph: Graph<CodeEntity, ConnascenceEdge>,
    symbol_table: SymbolTable,
}

impl SemanticModel {
    pub fn from_tree_sitter_ast(&self, ast: &ASTNode) -> Result<Self> {
        // 1. Build CFG desde AST
        let cfg = self.build_cfg(ast)?;

        // 2. Build DFG (data flow)
        let dfg = self.build_dfg(&cfg)?;

        // 3. Build scope tree
        let scope_tree = self.build_scope_tree(ast)?;

        Ok(SemanticModel { cfg, dfg, scope_tree, ... })
    }
}
```

---

### 5. **CFG/DFG Construction desde AST** (NO IMPLEMENTADO)

**Estado actual:** AST simple sin estructura de control

**Falta:**
```rust
pub struct ControlFlowGraphBuilder {
    language: Language,
}

impl ControlFlowGraphBuilder {
    pub fn build_cfg(&self, ast: &ASTNode) -> Result<CFG> {
        // 1. Identificar basic blocks
        let blocks = self.identify_basic_blocks(ast)?;

        // 2. Crear nodos en el grafo
        for block in blocks {
            cfg.add_node(block);
        }

        // 3. Crear edges (control flow)
        for block in &blocks {
            let successors = self.get_successors(block)?;
            for successor in successors {
                cfg.add_edge(block.id, successor.id, ());
            }
        }

        Ok(cfg)
    }
}
```

---

### 6. **Policy TOML para Taint** (NO IMPLEMENTADO)

**Estado:** Schema Cap'n Proto define tipos, pero no hay parser TOML

**Falta:**
```rust
// Policy configurable desde archivo TOML
#[derive(Deserialize)]
pub struct TaintPolicy {
    pub sources: Vec<SourceDefinition>,
    pub sinks: Vec<SinkDefinition>,
    pub sanitizers: Vec<SanitizerDefinition>,
}

#[derive(Deserialize)]
pub struct SourceDefinition {
    pub pattern: String,        // Regex o tree-sitter pattern
    pub source_type: String,    // "HttpRequest", "UserInput", etc.
    pub tags: Vec<String>,      // ["PII", "Finance", "Credentials"]
}

#[derive(Deserialize)]
pub struct SinkDefinition {
    pub pattern: String,        // Regex o tree-sitter pattern
    pub category: String,       // "SqlQuery", "CommandExecution", etc.
    pub severity: Severity,     // "critical", "major", etc.
}
```

---

### 7. **Integración tree-sitter real** (Implementación stub)

**Estado actual:** `hodei-declarative-extractors` usa AST simple

**Falta:**
```rust
// tree-sitter real integration
impl MultiLanguageParser {
    pub async fn parse_real(&self, language: Language, code: &str) -> Result<ParseResult> {
        let parser = Parser::new();

        // Configurar lenguaje específico
        #[cfg(feature = "java")]
        let language = tree_sitter_java::language();
        #[cfg(feature = "rust")]
        let language = tree_sitter_rust::language();

        parser.set_language(language)?;

        // Parsear código
        let tree = parser.parse(code, None).ok_or(ParseError::ParseFailed)?;

        // Convertir a ASTNode con tree-sitter
        let ast = self.tree_to_ast(tree.root_node());

        Ok(ParseResult::new(ast, language, ...))
    }
}
```

---

## 📈 **ANÁLISIS DE PROGRESO**

### Estado por Componente

| Componente | Estado | Progreso | Observaciones |
|------------|--------|----------|---------------|
| **IR Schema (Cap'n Proto)** | ✅ Completo | **100%** | Perfecto, listo para usar |
| **FlowIndex (petgraph)** | ✅ Implementado | **90%** | Solo falta integración datafrog |
| **Pattern Engine** | ✅ Completo | **100%** | Tier 2 totalmente funcional |
| **MultiLanguageParser** | ⚠️ Stub | **30%** | Estructura OK, falta tree-sitter real |
| **TaintPropagator** | ❌ No existe | **0%** | Requiere implementación completa |
| **ConnascenceAnalyzer** | ❌ No existe | **0%** | Requiere implementación completa |
| **SemanticModel Builder** | ❌ No existe | **0%** | Requiere implementación completa |
| **Policy TOML** | ❌ No existe | **0%** | Parser simple necesario |
| **CFG/DFG Builder** | ❌ No existe | **0%** | Integración petgraph necesaria |
| **hodei-deep-analysis-engine** | ❌ No existe | **0%** | Crate principal por crear |

### Estimación Total

**Progreso actual:** ~**40-50%** del total necesario para Nivel 3

**Componentes reutilizables:** 60-70%
**Componentes por implementar:** 30-40%

---

## 🎯 **PLAN DE IMPLEMENTACIÓN OPTIMIZADO**

### Estrategia: **Aprovechar lo existente**

#### Paso 1: **Crear hodei-deep-analysis-engine** (Semana 1)
```bash
# Crear crate
cargo new --lib crates/hodei-deep-analysis-engine

# Dependencies
[dependencies]
datafrog = "2.0.1"                    # NUEVO
petgraph = { workspace = true }        # REUTILIZAR
hodei-ir = { path = "../hodei-ir" }    # REUTILIZAR
```

#### Paso 2: **Integrar datafrog con FlowIndex existente** (Semana 1-2)
```rust
// ENHANCE: flow_index.rs con datafrog
pub struct FlowIndex {
    // petgraph para queries de conectividad
    graph: DiGraph<FactId, ()>,

    // datafrog para reglas Datalog de taint
    iteration: Iteration<'static>,
    taint_sources: Variable<(FlowId, VariableName)>,
    taint_sinks: Variable<(FlowId, SinkCategory)>,
    // ...
}
```

#### Paso 3: **Conectar tree-sitter real** (Semana 2-3)
```rust
// ENHANCE: tree_sitter.rs de hodei-declarative-extractors
impl MultiLanguageParser {
    pub fn parse_with_tree_sitter(&self, lang: Language, code: &str) -> Result<ParseResult> {
        // Usar tree-sitter real en lugar de AST stub
        let tree = self.parser.parse(code, None)?;
        Ok(self.tree_to_ast(tree.root_node()))
    }
}
```

#### Paso 4: **Implementar ConnascenceAnalyzer** (Semana 3-4)
```rust
// NUEVO: connascence_analyzer.rs
impl ConnascenceAnalyzer {
    pub fn detect_positional(&self) -> Vec<CouplingFinding> {
        // Heurística: 3+ parámetros mismo tipo = CoP
    }
}
```

#### Paso 5: **SemanticModel desde AST** (Semana 4-5)
```rust
// NUEVO: semantic_model.rs
impl SemanticModel {
    pub fn from_ast(ast: &ASTNode) -> Result<Self> {
        let cfg = self.build_cfg(ast)?;
        let dfg = self.build_dfg(&cfg)?;
        Ok(SemanticModel { cfg, dfg, ... })
    }
}
```

---

## 💡 **RECOMENDACIONES**

### 1. **Priorizar Datafrog Integration**
- El **FlowIndex ya existe** con petgraph
- Solo necesita **overlay de datafrog** para reglas Datalog
- **ROI más alto** (máximo reuso, mínimo esfuerzo)

### 2. **Conectar tree-sitter real**
- La **infraestructura ya está** en `hodei-declarative-extractors`
- Solo cambiar de AST stub a **tree-sitter real**
- **Impacto inmediato** en parsing de AST

### 3. **Crear ConnascenceAnalyzer incrementally**
- Empezar con **CoP y CoM** (más fáciles)
- Usar **SemanticModel** parcialmente construido
- Tests primero (TDD)

### 4. **No duplicar esfuerzos**
- **FlowIndex** → Reutilizar, no reescribir
- **Pattern Engine** → Ya funciona, usar tal cual
- **IR Schema** → Perfecto, usar tal cual

---

## 🔍 **CONCLUSIÓN**

**El proyecto tiene una base sólida.** No estamos empezando de cero:

✅ **Tier 2** (Declarativo) ya está completo  
✅ **IR Schema** es robusto  
✅ **petgraph** ya integrado  
✅ **FlowIndex** implementado  

**El camino a Nivel 3 es mucho más corto de lo que parecía.** La inversión principal es:
1. Integrar `datafrog` (1-2 semanas)
2. Conectar tree-sitter real (1 semana)
3. Implementar ConnascenceAnalyzer (2 semanas)

**Estimación realista:** **5-6 semanas** en lugar de 12-16 semanas.

La **arquitectura modular** del proyecto permite **reutilización máxima** y **desarrollo incremental**. 🎯
