# EPIC-12: Declarative Pattern Engine - Tree-sitter + YAML Rules

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: hodei-scan v3.3  
**Dependencias**: EPIC-10 (Extractor Ecosystem), EPIC-11 (IR Schema)  
**Owner**: Developer Experience Team  
**Prioridad**: High

---

## 1. Resumen Ejecutivo

Implementar un **motor de patrones declarativo** basado en **tree-sitter** que permita a usuarios escribir reglas de análisis en **YAML** sin programar. Democratiza la creación de reglas, permitiendo que analistas y desarrolladores definan patrones de código en minutos.

### Objetivo de Negocio
Reducir el tiempo de desarrollo de reglas de **días (Rust)** a **minutos (YAML)**, facilitando la reacción rápida a nuevas vulnerabilidades y patrones de riesgo. Meta: 1000+ reglas en formato YAML en 6 meses.

### Métricas de Éxito
- **Productividad**: Crear regla básica en <5 minutos
- **Performance**: 1000 reglas YAML vs 10K files <30s
- **Adopción**: 500+ reglas YAML en ecosystem
- **Multi-lenguaje**: Soporte para 10+ lenguajes

---

## 2. Contexto Técnico

### 2.1. Problema Actual
Crear reglas requiere:
- Programar en Rust
- Compilar y testear
- Desplegar nueva versión
- **Tiempo**: 1-2 días por regla

### 2.2. Solución: Reglas YAML Declarativas

```yaml
# rules/java/empty-catch-block.hodei.yml
id: JAVA-EMPTY-CATCH-BLOCK
language: java
message: "Empty catch block detected. Errors are being silently ignored."
severity: warning
category: error-handling
pattern: |
  try {
    $STMT
  } catch ($EXCEPTION $VAR) {
    // $COMMENT
  }

# Generated FactType::CodeSmell
```

**Flujo de Ejecución:**
```
YAML Rule
    ↓ (parse)
Tree-sitter Query
    ↓ (execute)
AST Matches
    ↓ (transform)
Facts (IR)
    ↓ (RuleEngine)
Findings
```

---

## 3. Arquitectura Detallada

### 3.1. Componentes Core

#### YamlRuleLoader
```rust
pub struct YamlRuleLoader {
    rules: HashMap<String, YamlRule>,
}

pub struct YamlRule {
    pub id: String,
    pub language: String,
    pub message: String,
    pub severity: String,
    pub category: String,
    pub pattern: String,  // Tree-sitter query
    pub metadata: HashMap<String, String>,
}
```

#### TreeSitterMatcher
```rust
pub struct TreeSitterMatcher {
    parser: Parser,
    query_cache: HashMap<String, CompiledQuery>,
}

impl TreeSitterMatcher {
    pub fn execute_rule(
        &mut self,
        rule: &YamlRule,
        source_code: &str,
    ) -> Result<Vec<Fact>, MatcherError> {
        // Parse source code
        let tree = self.parser.parse(source_code, None)?;
        
        // Get or compile query
        let query = self.get_or_compile_query(&rule.pattern, &rule.language)?;
        
        // Execute query
        let matches = self.execute_query(&query, &tree, source_code)?;
        
        // Transform matches to facts
        self.matches_to_facts(matches, source_code)
    }
}
```

### 3.2. Query Compilation

```rust
pub struct CompiledQuery {
    language: Language,
    query: Query,
    capture_names: Vec<String>,
}

impl CompiledQuery {
    pub fn compile(pattern: &str, language: &str) -> Result<Self, CompileError> {
        let tree_sitter_lang = get_language(language)?;
        let query = Query::new(tree_sitter_lang, pattern)
            .map_err(CompileError::InvalidPattern)?;
        
        let capture_names = query.capture_names().to_vec();
        
        Ok(CompiledQuery {
            language: tree_sitter_lang,
            query,
            capture_names,
        })
    }
}
```

---

## 4. User Stories

### US-12.01: Implementar YAML Rule Loader

**Como:** Usuario  
**Quiero:** Cargar reglas desde archivos YAML  
**Para:** Gestionar mis reglas personalizadas

**Criterios de Aceptación:**
- [ ] Lee archivos `.hodei.yml` desde directorio
- [ ] Parsea YAML a estructura Rule
- [ ] Validación de campos requeridos
- [ ] Soporte para múltiples reglas por archivo
- [ ] Configuración de paths de reglas

**TDD - Red:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn load_yaml_rule() {
        let yaml = r#"
id: JAVA-EMPTY-CATCH
language: java
message: "Empty catch block"
severity: warning
pattern: |
  try { $STMT } catch ($E $V) { }
"#;
        
        let rule = parse_yaml_rule(yaml).unwrap();
        assert_eq!(rule.id, "JAVA-EMPTY-CATCH");
        assert_eq!(rule.language, "java");
        assert!(rule.pattern.contains("try"));
    }
}
```

**TDD - Green:**
```rust
pub fn parse_yaml_rule(yaml: &str) -> Result<YamlRule, YamlError> {
    let rule: YamlRule = serde_yaml::from_str(yaml)?;
    
    // Validate required fields
    if rule.id.is_empty() {
        return Err(YamlError::MissingField("id".to_string()));
    }
    if rule.pattern.is_empty() {
        return Err(YamlError::MissingField("pattern".to_string()));
    }
    
    Ok(rule)
}
```

**Conventional Commit:**
`feat(yaml-rule): implement YAML rule loader with validation`

---

### US-12.02: Tree-sitter Integration para Multi-lenguaje

**Como:** Desarrollador Core  
**Quiero:** Ejecutar queries tree-sitter sobre código fuente  
**Para:** Pattern matching eficiente y preciso

**Criterios de Aceptación:**
- [ ] Soporte para 10+ lenguajes (Python, Java, Rust, Go, etc.)
- [ ] Compilación de queries lazy y caching
- [ ] Parallel file processing
- [ ] Memory-efficient tree handling
- [ ] Error handling con context

**TDD - Red:**
```rust
#[test]
fn execute_python_pattern() {
    let source = r#"
def login(user, password):
    query = "SELECT * FROM users"
    cursor.execute(query)
"#;
    
    let pattern = r#"
(call_expression
  function: (attribute
    object: (identifier) @obj
    attribute: (identifier) @method)
  arguments: (arguments (string) @sql))
"#;
    
    let matcher = TreeSitterMatcher::new();
    let matches = matcher.execute_pattern("python", pattern, source).unwrap();
    
    assert!(matches.len() > 0);
    assert!(matches[0].captures.contains_key("sql"));
}
```

**TDD - Green:**
```rust
impl TreeSitterMatcher {
    pub fn execute_pattern(
        &mut self,
        language: &str,
        pattern: &str,
        source_code: &str,
    ) -> Result<Vec<QueryMatch>, MatcherError> {
        // Get or create parser for language
        let parser = self.get_or_create_parser(language)?;
        
        // Parse source code
        let tree = parser.parse(source_code, None)
            .ok_or(MatcherError::ParseFailed)?;
        
        // Compile query
        let compiled = self.compile_query(pattern, language)?;
        
        // Execute query
        let mut cursor = QueryCursor::new();
        let query_matches = cursor.matches(
            &compiled.query,
            tree.root_node(),
            source_code.as_bytes(),
        );
        
        // Convert to QueryMatch
        let matches: Vec<_> = query_matches.map(|m| QueryMatch::from_tree_sitter_match(m, source_code)).collect();
        
        Ok(matches)
    }
}
```

**Conventional Commit:**
`feat(tree-sitter): add multi-language pattern matching engine`

---

### US-12.03: Query Compilation y Caching

**Como:** Desarrollador  
**Quiero:** Queries se compilen una sola vez y se re-utilicen  
**Para:** Performance óptimo

**Criterios de Aceptación:**
- [ ] Cache de queries compiladas por pattern
- [ ] LRU eviction policy para memory control
- [ ] Thread-safe cache operations
- [ ] Invalidation on rule changes
- [ ] Hit ratio >90% en uso normal

**TDD - Red:**
```rust
#[test]
fn query_caching() {
    let mut matcher = TreeSitterMatcher::new();
    let pattern = "(identifier) @id";
    
    // First execution - compile
    let result1 = matcher.execute_pattern("python", pattern, "x = 1").unwrap();
    
    // Second execution - should use cache
    let result2 = matcher.execute_pattern("python", pattern, "y = 2").unwrap();
    
    // Verify cache was used (same compiled query)
    assert_eq!(matcher.query_cache.len(), 1);
}
```

**TDD - Green:**
```rust
pub struct TreeSitterMatcher {
    parsers: HashMap<String, Parser>,
    query_cache: Arc<RwLock<LruCache<String, CompiledQuery>>>,
    max_cache_size: usize,
}

impl TreeSitterMatcher {
    fn compile_query(&self, pattern: &str, language: &str) -> Result<CompiledQuery, CompileError> {
        let cache_key = format!("{}:{}", language, hash_pattern(pattern));
        
        {
            let cache = self.query_cache.read().unwrap();
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }
        
        // Compile new query
        let compiled = CompiledQuery::compile(pattern, language)?;
        
        {
            let mut cache = self.query_cache.write().unwrap();
            cache.put(cache_key, compiled.clone());
        }
        
        Ok(compiled)
    }
}
```

**Conventional Commit:**
`feat(query): add LRU caching for compiled tree-sitter queries`

---

### US-12.04: Transformations: Matches → Facts

**Como:** Desarrollador  
**Quiero:** Convertir tree-sitter matches a Facts del IR  
**Para:** Integración con RuleEngine

**Criterios de Aceptación:**
- [ ] Extract matched nodes como strings
- [ ] Crear Location desde match range
- [ ] Map a FactType apropiado
- [ ] Soporte para multiple captures
- [ ] Context preservation (parent nodes)

**TDD - Red:**
```rust
#[test]
fn match_to_fact() {
    let source = "def login(user): pass";
    let match_node = create_mock_match(source, 0..19);  // "def login(user):"
    
    let fact = match_to_fact(&match_node, "function-definition", source).unwrap();
    
    assert!(matches!(fact.fact_type, FactType::Function { .. }));
    assert_eq!(fact.location.start_line, 1);
}
```

**TDD - Green:**
```rust
pub fn match_to_fact(
    query_match: &QueryMatch,
    expected_type: &str,
    source_code: &str,
) -> Result<Fact, TransformError> {
    // Extract text from captures
    let captures = extract_captures(query_match, source_code);
    
    // Create location from match range
    let range = query_match.range();
    let location = create_source_location(range, source_code);
    
    // Map to appropriate FactType
    let fact_type = match expected_type {
        "function-definition" => FactType::Function {
            name: extract_function_name(&captures),
            complexity: 1,  // Default, can be enhanced
            lines_of_code: calculate_loc(range),
        },
        "sql-injection" => FactType::Vulnerability {
            cwe_id: Some("CWE-89".to_string()),
            description: "SQL injection vulnerability".to_string(),
            severity: Severity::High,
            cvss_score: Some(8.6),
            confidence: Confidence::High,
        },
        _ => FactType::CodeSmell {
            smell_type: expected_type.to_string(),
            severity: Severity::Medium,
            message: format!("Detected pattern: {}", expected_type),
        },
    };
    
    Ok(Fact {
        id: FactId::new(),
        fact_type,
        location,
        provenance: Provenance::default(),  // From YAML rule
    })
}
```

**Conventional Commit:**
`feat(transform): add tree-sitter match to Fact transformation`

---

### US-12.05: Batch Processing con Paralelismo

**Como:** Usuario  
**Quiero:** Ejecutar 1000 reglas sobre 10K archivos rápidamente  
**Para:** CI/CD performance

**Criterios de Aceptación:**
- [ ] Parallel file processing con rayon
- [ ] Parallel rule execution por file
- [ ] Memory pooling para trees
- [ ] Resource limits configurables
- [ ] Progress reporting

**TDD - Red:**
```rust
#[tokio::test]
async fn batch_process_files() {
    let files = generate_test_files(1000);
    let rules = load_test_rules(100);
    
    let processor = YamlRuleProcessor::new(rules);
    let start = Instant::now();
    let results = processor.process_files(files).await.unwrap();
    let duration = start.elapsed();
    
    assert!(duration < Duration::from_secs(30));
    assert_eq!(results.processed_files, 1000);
}
```

**TDD - Green:**
```rust
pub struct YamlRuleProcessor {
    rules: Arc<Vec<YamlRule>>,
    matcher: Arc<RwLock<TreeSitterMatcher>>,
    concurrency: usize,
}

impl YamlRuleProcessor {
    pub async fn process_files(&self, files: Vec<FilePath>) -> Result<ProcessingResults, ProcessorError> {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(self.concurrency)
            .build()
            .map_err(ProcessorError::PoolCreationFailed)?;
        
        let (tx, rx) = mpsc::channel(1000);
        
        // Process files in parallel
        pool.install(|| {
            files.into_par_iter().for_each_with(tx, |tx, file| {
                let result = self.process_single_file(&file);
                tx.send(result).unwrap();
            });
        });
        
        // Aggregate results
        let mut results = ProcessingResults::default();
        for result in rx {
            results.aggregate(result);
        }
        
        Ok(results)
    }
}
```

**Conventional Commit:**
`feat(processor): add parallel batch processing for rules and files`

---

### US-12.06: Validation y Testing Framework

**Como:** Desarrollador Reglas  
**Quiero:** Testear reglas YAML antes de usarlas  
**Para:** Evitar falsos positivos/negativos

**Criterios de Aceptación:**
- [ ] `hodei-scan test-rule rule.hodei.yml`
- [ ] Test cases en YAML (positive/negative)
- [ ] Auto-generated test scaffolds
- [ ] Diff reporting (expected vs actual)
- [ ] CI integration

**Formato Test Case:**
```yaml
# test/java/empty-catch.hodei.test.yml
rule: JAVA-EMPTY-CATCH

tests:
  - name: "Empty catch should trigger"
    code: |
      try { file.read(); } catch (Exception e) { }
    expected:
      findings: 1
      
  - name: "Non-empty catch should not trigger"
    code: |
      try { file.read(); } catch (Exception e) { log.error(e); }
    expected:
      findings: 0
```

**TDD - Red:**
```rust
#[test]
fn run_rule_test_case() {
    let test_case = r#"
rule: JAVA-EMPTY-CATCH
tests:
  - name: "Empty catch"
    code: "try {} catch (Exception e) {}"
    expected:
      findings: 1
"#;
    
    let results = run_test_case(test_case).unwrap();
    assert!(results.passed);
}
```

**Conventional Commit:**
`feat(testing): add YAML rule testing framework`

---

## 5. Performance Benchmarks

```rust
// benches/yaml_rule_engine.rs
fn bench_1000_rules_10k_files(c: &mut Criterion) {
    let files = generate_test_files(10_000);
    let rules = load_yaml_rules(1000);
    
    c.bench_function("yaml_rule_engine", |b| {
        b.iter(|| {
            let processor = YamlRuleProcessor::new(rules.clone());
            let results = processor.process_files(files.clone()).unwrap();
            black_box(results);
        });
    });
}

fn bench_query_compilation(c: &mut Criterion) {
    let patterns = generate_test_patterns(100);
    
    c.bench_function("query_compilation", |b| {
        b.iter(|| {
            let mut matcher = TreeSitterMatcher::new();
            for pattern in &patterns {
                matcher.compile_query(pattern, "python").unwrap();
            }
        });
    });
}
```

**Targets:**
- 1000 rules × 10K files <30s
- Query compilation <1ms per pattern
- Memory usage <2GB for full run

---

## 6. Testing Strategy

### 6.1. Unit Tests
- YAML parsing y validation
- Query compilation
- Match transformation logic
- Caching behavior

### 6.2. Integration Tests
- End-to-end: YAML → Facts → Findings
- Multi-language scenarios
- Large-scale batch processing

### 6.3. Property-Based Tests
- Arbitrary tree-sitter patterns
- Round-trip: Rule → Compiled → Executed → Matched
- AST invariant preservation

---

## 7. Riesgos y Mitigaciones

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| Tree-sitter grammar bugs | Alto | Medio | Test against real projects |
| Performance degradation | Alto | Alto | Continuous benchmarking + profiling |
| YAML injection attacks | Alto | Bajo | Strict YAML parsing + sanitization |
| Memory leaks in tree caching | Medio | Medium | LRU cache limits + monitoring |

---

## 8. Definition of Done

- [ ] YAML rule loader con validation
- [ ] Tree-sitter engine para 10+ lenguajes
- [ ] Query compilation y LRU caching
- [ ] Match → Fact transformation
- [ ] Parallel batch processing
- [ ] Rule testing framework
- [ ] Benchmarks: 1000 rules <30s
- [ ] Tests: >90% coverage
- [ ] Documentation: Rule authoring guide
- [ ] 100+ example rules included

---

**Estimación Total**: 4 Sprints (8 semanas)  
**Commit Messages**:  
- `feat(yaml-rule): implement YAML rule loader with validation`  
- `feat(tree-sitter): add multi-language pattern matching engine`  
- `feat(query): add LRU caching for compiled tree-sitter queries`  
- `feat(transform): add tree-sitter match to Fact transformation`  
- `feat(processor): add parallel batch processing for rules and files`  
- `feat(testing): add YAML rule testing framework`

---

**Referencias Técnicas:**
- Tree-sitter: https://tree-sitter.github.io/
- Serde YAML: https://serde.rs/
- Rayon: https://github.com/rayon-rs/rayon
- Semgrep patterns: https://semgrep.dev/docs/writing-rules/pattern-syntax/
