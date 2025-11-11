# EPIC: Optimizaciones de Rendimiento y Seguridad v3.2
## Plan de Implementación con TDD y Enfoque Iterativo

---

## 📋 Resumen Ejecutivo

Este documento define las **epics y historias de usuario** para implementar las optimizaciones de alto rendimiento y seguridad descritas en `ANALISIS-MEJORAS-FUTURAS-v3.2.md`. El plan sigue un enfoque **TDD (Test-Driven Development)** y prioriza optimizaciones de **alto ROI** y **baja complejidad** primero.

### Metodología
- **TDD**: Red → Green → Refactor
- **Commits atómicos**: Un commit por historia de usuario
- **Investigación**: Perplexity para tecnologías específicas
- **Progressive enhancement**: Implementar incrementally

---

## 🎯 EPIC 01: String Interning y EnumMap (Quick Win)
### **Impacto**: Alto | **Complejidad**: Baja | **ROI**: Muy Alto

#### Contexto
Reducir el consumo de memoria del IR y mejorar la cache performance. El proyecto actual usa `Arc<str>` para strings repetitivos, causando alta memoria y comparaciones O(n).

#### User Stories

---

##### US-01: Implementar String Interner
**Como** desarrollador del motor IR  
**Quiero** internar strings para reducir memoria  
**Para** disminuir el uso de memoria en 60-80% y mejorar comparaciones a O(1)

**Criterios de Aceptación:**
- ✅ Implementar `StringInterner` usando `string-interner` crate
- ✅ Crear wrapper `InternedString` que envuelve symbols
- ✅ Implementar cache thread-safe
- ✅ Benchmark debe mostrar 3-5x speedup en comparaciones
- ✅ Test property-based para invariantes de interning

**TDD Tests (RED):**
```rust
#[test]
fn test_string_interner_basic() {
    let mut interner = StringInterner::new();
    let s1 = interner.intern("src/main.rs");
    let s2 = interner.intern("src/main.rs");
    let s3 = interner.intern("src/lib.rs");

    assert_eq!(s1, s2); // Mismo symbol
    assert_ne!(s1, s3); // Diferentes symbols
}

#[test]
fn test_memory_reduction() {
    let paths = vec!["src/main.rs"; 10000];
    let original_size = calculate_size(&paths);

    let mut interner = StringInterner::new();
    let interned: Vec<_> = paths.iter()
        .map(|p| interner.intern(p))
        .collect();

    let interned_size = calculate_size(&interned);
    let reduction = (original_size - interned_size) as f64 / original_size as f64;

    assert!(reduction > 0.6, "Should reduce memory by 60%");
}

#[test]
fn test_interned_string_lookup() {
    let mut interner = StringInterner::new();
    let symbol = interner.intern("test");
    assert_eq!(interner.lookup(symbol), Some("test"));
}
```

**Implementation Tasks:**
1. Agregar `string-interner = "0.3"` a dependencies
2. Crear `hodei-ir/src/interning/mod.rs`
3. Implementar `StringInterner` struct
4. Implementar `InternedString` wrapper
5. Agregar cache con `HashMap<PathBuf, InternedString>`
6. Crear benchmarks con criterion
7. Documentar en KDoc

**Commit**: `feat(hodei-ir): implement string interning for memory optimization`

---

##### US-02: Implementar ProjectPathInterner
**Como** sistema de IR  
**Quiero** internar ProjectPath de forma eficiente  
**Para** reducir memoria y evitar duplicación de paths

**Criterios de Aceptación:**
- ✅ `ProjectPathInterner` que intern paths automáticamente
- ✅ Cache thread-safe para lookups rápidos
- ✅ Compatibilidad con PathBuf existente
- ✅ Tests de property-based para validaciones

**TDD Tests (RED):**
```rust
#[test]
fn test_project_path_interner() {
    let mut interner = ProjectPathInterner::new();
    let path = PathBuf::from("src/main.rs");
    let sym1 = interner.intern_path(&path);
    let sym2 = interner.intern_path(&path);

    assert_eq!(sym1, sym2);
    assert_eq!(interner.get_path(&sym1), Some("src/main.rs"));
}

#[test]
fn test_path_preservation() {
    let mut interner = ProjectPathInterner::new();
    let test_paths = vec![
        "src/main.rs",
        "src/lib.rs",
        "tests/test.rs"
    ];

    for path_str in test_paths {
        let path = PathBuf::from(path_str);
        let sym = interner.intern_path(&path);
        assert_eq!(interner.get_path(&sym), Some(path_str));
    }
}
```

**Implementation Tasks:**
1. Crear `ProjectPathInterner` struct en `interning/mod.rs`
2. Implementar cache HashMap<PathBuf, InternedString>
3. Crear método `intern_path()` y `get_path()`
4. Integrar con `ProjectPath` type existente
5. Tests de thread-safety con proptest

**Commit**: `feat(hodei-ir): add ProjectPathInterner for efficient path storage`

---

##### US-03: Implementar EnumMap para FactTypeIndex
**Como** motor de indexación  
**Quiero** usar EnumMap en lugar de HashMap para FactType  
**Para** mejorar cache locality y rendimiento

**Criterios de Aceptación:**
- ✅ Implementar `FactTypeDiscriminant` enum con todas las variantes
- ✅ Crear `FactTypeIndex` usando `enum-map` crate
- ✅ Benchmark debe mostrar 2-3x speedup vs HashMap
- ✅ Migration guide para cambiar de HashMap a EnumMap

**TDD Tests (RED):**
```rust
#[test]
fn test_fact_type_index_insert() {
    let mut index = FactTypeIndex::new();
    let fact_id = FactId::new();

    index.insert(FactTypeDiscriminant::TaintSource, fact_id);

    let results = index.get_by_type(FactTypeDiscriminant::TaintSource);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], fact_id);
}

#[test]
fn test_enum_map_benchmark() {
    let start = Instant::now();
    let mut map = EnumMap::default();
    for i in 0..1000 {
        map[FactTypeDiscriminant::TaintSource].push(FactId::from_u64(i));
    }
    let duration = start.elapsed();

    // Should be faster than HashMap
    assert!(duration < Duration::from_millis(10));
}
```

**Implementation Tasks:**
1. Agregar `enum-map = "2.0"` a dependencies
2. Crear `FactTypeDiscriminant` enum exhaustivo
3. Implementar `FactTypeIndex` con EnumMap
4. Migrar `IndexedFactStore` a usar EnumMap
5. Crear benchmarks comparativos

**Commit**: `refactor(engine): replace HashMap with EnumMap for FactTypeIndex`

---

## 🎯 EPIC 02: Cap'n Proto Zero-Copy Serialization
### **Impacto**: Muy Alto | **Complejidad**: Media | **ROI**: Muy Alto

#### Contexto
Reemplazar la serialización actual (rkyv/JSON) con Cap'n Proto para lograr zero-copy deserialization con memory-mapped files. Esto es fundamental para el rendimiento.

#### User Stories

---

##### US-04: Implementar Cap'n Proto Schema
**Como** sistema IR  
**Quiero** definir schema Cap'n Proto para Facts  
**Para** habilitar zero-copy deserialization

**Criterios de Aceptación:**
- ✅ Schema `.capnp` que define Fact, FactType, Location
- ✅ Code generation con `capnp` tool
- ✅ Validación de schema correctness
- ✅ Migration de estructura actual a Cap'n Proto

**TDD Tests (RED):**
```rust
#[test]
fn test_capnp_fact_serialization() {
    // Create test fact
    let fact = create_test_fact();

    // Serialize to bytes
    let mut message = capnp::message::Builder::new_default();
    let mut fact_builder = message.init_root::<facts_capnp::Fact>();
    // ... populate

    // Check serialization works
    assert!(message.getSerializedBytes().len() > 0);
}

#[test]
fn test_capnp_zero_copy_read() {
    let temp_file = NamedTempFile::new().unwrap();
    write_test_ir(&temp_file);

    // Read with zero-copy (mmap)
    let ir = ZeroCopyIR::from_file(temp_file.path()).unwrap();

    // Access facts without deserialization
    let fact = ir.get_fact(0).unwrap();
    assert_eq!(fact.get_id(), expected_id);
}
```

**Implementation Tasks:**
1. Crear `hodei-ir/schema/facts.capnp` con definitions
2. Agregar `capnp = "0.20"` y `memmap2 = "0.9"` a deps
3. Build script para generar Rust code desde .capnp
4. Implementar `ZeroCopyIR` struct
5. Tests de serialization roundtrip

**Commit**: `feat(hodei-ir): add Cap'n Proto schema for zero-copy serialization`

---

##### US-05: Implementar ZeroCopyIR con Memory-Mapped Files
**Como** motor de análisis  
**Quiero** cargar IR usando mmap  
**Para** acceso instantáneo a facts sin parsing

**Criterios de Aceptación:**
- ✅ `ZeroCopyIR::from_file()` usando memmap2
- ✅ `get_fact(index)` que retorna FactReader sin copy
- ✅ Benchmark debe ser 10-100x más rápido que JSON
- ✅ Support para IRs de gigabytes sin problemas de memoria

**TDD Tests (RED):**
```rust
#[test]
fn test_zero_copy_load_1m_facts() {
    let ir_path = create_large_ir_file(1_000_000);

    let start = Instant::now();
    let ir = ZeroCopyIR::from_file(&ir_path).unwrap();
    let load_time = start.elapsed();

    // Should load in < 100ms
    assert!(load_time < Duration::from_millis(100));

    // Random access should be O(1)
    let start = Instant::now();
    for i in 0..10_000 {
        let _ = ir.get_fact(i as u32).unwrap();
    }
    let access_time = start.elapsed();

    assert!(access_time < Duration::from_millis(10));
}

#[test]
fn test_memory_efficiency() {
    let ir_path = create_large_ir_file(10_000_000);
    let file_size = fs::metadata(&ir_path).unwrap().len() as usize;

    let ir = ZeroCopyIR::from_file(&ir_path).unwrap();

    // Memory usage should be minimal (just mmap)
    let mem_usage = get_memory_usage();
    assert!(mem_usage < file_size / 2, "Should use less than half file size");
}
```

**Implementation Tasks:**
1. Implementar `ZeroCopyIR` struct con memmap2
2. Crear `FactReader` que hace borrow del mmap
3. Manejar error cases (corrupted files, out of bounds)
4. Integration tests con large files
5. Benchmark con criterion

**Commit**: `feat(hodei-ir): implement ZeroCopyIR with memory-mapped files`

---

##### US-06: Crear Cap'n Proto Benchmarks
**Como** desarrollador  
**Quiero** medir performance de Cap'n Proto vs JSON  
**Para** cuantificar las mejoras

**Criterios de Aceptación:**
- ✅ Benchmarks para load time (1K, 10K, 1M, 10M facts)
- ✅ Benchmarks para random access patterns
- ✅ Memory usage comparison
- ✅ GitHub Actions que automatiza benchmarks

**TDD Tests (RED):**
```rust
fn bench_capnp_load_1k_facts(c: &mut Criterion) {
    let ir_path = test_data_dir().join("1k_facts.ir");

    c.bench_function("capnp_load_1k", |b| {
        b.iter(|| {
            let ir = ZeroCopyIR::from_file(&ir_path).unwrap();
            black_box(ir);
        })
    });
}

fn bench_capnp_random_access(c: &mut Criterion) {
    let ir_path = test_data_dir().join("1m_facts.ir");
    let ir = ZeroCopyIR::from_file(&ir_path).unwrap();

    c.bench_function("capnp_random_access_1k", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let _ = ir.get_fact(black_box(i as u32)).unwrap();
            }
        })
    });
}
```

**Implementation Tasks:**
1. Crear `benches/capnp_benchmarks.rs`
2. Generate test data files (1K, 1M, 10M facts)
3. Benchmarks para load, random access, sequential access
4. Comparison con JSON/rkyv baselines
5. Automate con GitHub Actions

**Commit**: `perf(benches): add comprehensive Cap'n Proto performance benchmarks`

---

## 🎯 EPIC 03: DSL Security y Sandboxing
### **Impacto**: Crítico | **Complejidad**: Alta | **ROI**: Muy Alto

#### Contexto
El DSL debe ser seguro contra injection attacks, DoS, y path traversal. Esto es fundamental para un tool de seguridad.

#### User Stories

---

##### US-07: Implementar DSL Input Validation
**Como** sistema de seguridad  
**Quiero** validar todo input del DSL  
**Para** prevenir injection attacks

**Criterios de Aceptación:**
- ✅ `DSLValidator` con allow-list de caracteres
- ✅ Validación de longitud máxima de reglas
- ✅ Detección de funciones no permitidas
- ✅ Validación de profundidad de anidamiento
- ✅ Fuzz tests con AFL o cargo-fuzz

**TDD Tests (RED):**
```rust
#[test]
fn test_dsl_validator_rejects_malicious_input() {
    let validator = DSLValidator::new();

    // Should reject SQL injection attempts
    assert!(validator.validate_rule("SELECT * FROM users").is_err());

    // Should reject path traversal
    assert!(validator.validate_rule("exists(file: '../../../etc/passwd')").is_err());

    // Should reject overly long rules
    let long_rule = "exists ".repeat(1000);
    assert!(validator.validate_rule(&long_rule).is_err());
}

#[test]
fn test_dsl_validator_accepts_valid_input() {
    let validator = DSLValidator::new();

    let valid_rule = r#"
        exists(
            TaintSource {
                type: "HTTP_REQUEST",
                confidence: HIGH
            }
        )
    "#;

    assert!(validator.validate_rule(valid_rule).is_ok());
}
```

**Implementation Tasks:**
1. Crear `hodei-dsl/src/validation/validator.rs`
2. Implementar character allow-list validation
3. Implement length and depth limits
4. Function whitelist enforcement
5. Fuzz tests setup

**Commit**: `security(dsl): add comprehensive input validation`

---

##### US-08: Implementar Runtime Sandboxing
**Como** motor de ejecución  
**Quiero** ejecutar reglas en sandbox  
**Para** prevenir DoS y resource exhaustion

**Criterios de Aceptación:**
- ✅ `ExecutionSandbox` con time limits
- ✅ Memory limits usando prlimit (Linux)
- ✅ Thread isolation para reglas
- ✅ Timeout enforcement
- ✅ Tests que verifican límites se respetan

**TDD Tests (RED):**
```rust
#[test]
fn test_sandbox_enforces_time_limit() {
    let sandbox = ExecutionSandbox::new(Duration::from_millis(100));

    let result = sandbox.execute(|| {
        std::thread::sleep(Duration::from_millis(200));
        Ok("done")
    });

    assert!(matches!(result, Err(ExecutionError::Timeout)));
}

#[test]
fn test_sandbox_enforces_memory_limit() {
    let sandbox = ExecutionSandbox::new(Duration::from_secs(1), 1024 * 1024); // 1MB

    let result = sandbox.execute(|| {
        let mut big_vec = vec![0u8; 10 * 1024 * 1024]; // 10MB
        Ok(big_vec.len())
    });

    assert!(matches!(result, Err(ExecutionError::MemoryLimitExceeded)));
}
```

**Implementation Tasks:**
1. Crear `hodei-engine/src/execution/sandbox.rs`
2. Implement thread spawning con isolation
3. Integrar prlimit para memory limits
4. Timeout management con watchdogs
5. Tests para todos los límites

**Commit**: `security(engine): add runtime sandboxing for rule execution`

---

##### US-09: Implementar Rule Complexity Analysis
**Como** motor de optimización  
**Quiero** estimar complejidad de reglas  
**Para** evitar evaluar reglas que serían demasiado costosas

**Criterios de Aceptación:**
- ✅ `RulePruner` que estima complexity
- ✅ Thresholds configurables
- ✅ Warnings para reglas complejas
- ✅ Performance tests

**TDD Tests (RED):**
```rust
#[test]
fn test_rule_pruner_estimates_complexity() {
    let pruner = RulePruner::new();

    // Simple rule should be allowed
    let simple_rule = create_simple_rule();
    assert!(pruner.should_evaluate_rule(&simple_rule, 1000));

    // Nested/complex rule should be rejected
    let complex_rule = create_deeply_nested_rule(50);
    assert!(!pruner.should_evaluate_rule(&complex_rule, 1000));
}

#[test]
fn test_complexity_calculation() {
    let pruner = RulePruner::new();

    let rule_and = RuleCondition::And(vec![...; 100]);
    let complexity = pruner.estimate_complexity(&rule_and);

    assert!(complexity > 100, "And with 100 conditions should be complex");
}
```

**Implementation Tasks:**
1. Crear `hodei-engine/src/rules/pruner.rs`
2. Implement complexity estimation algorithm
3. Thresholds y configuración
4. Integration con `RuleEngine`
5. Performance benchmarks

**Commit**: `perf(engine): add rule complexity analysis and pruning`

---

## 🎯 EPIC 04: Incremental Analysis para CI/CD
### **Impacto**: Muy Alto | **Complejidad**: Media | **ROI**: Muy Alto

#### Contexto
El análisis incremental es crítico para CI/CD. Solo re-analizar archivos changed usando git diff y cache.

#### User Stories

---

##### US-10: Implementar Git Diff Analyzer
**Como** sistema de análisis incremental  
**Quiero** detectar archivos modificados con git diff  
**Para** saber qué re-analizar

**Criterios de Aceptación:**
- ✅ `GitAnalyzer` que ejecuta git diff
- ✅ Support para added, modified, deleted files
- ✅ Hash y timestamp de archivos
- ✅ Integration con git repository

**TDD Tests (RED):**
```rust
#[test]
fn test_git_diff_detects_changes() {
    let temp_repo = TempRepository::new();
    temp_repo.commit_file("src/main.rs", "initial");

    // Modify file
    temp_repo.write_file("src/main.rs", "modified");
    temp_repo.commit_file("src/main.rs", "modified");

    let analyzer = GitAnalyzer::new(&temp_repo.path());
    let changes = analyzer.diff("HEAD~1", "HEAD").unwrap();

    assert!(changes.modified_files().contains(Path::new("src/main.rs")));
}

#[test]
fn test_git_diff_handles_deletions() {
    let temp_repo = TempRepository::new();
    temp_repo.commit_file("src/deleted.rs", "content");
    temp_repo.remove_file("src/deleted.rs");
    temp_repo.commit_all("deleted file");

    let analyzer = GitAnalyzer::new(&temp_repo.path());
    let changes = analyzer.diff("HEAD~1", "HEAD").unwrap();

    assert!(changes.deleted_files().contains(Path::new("src/deleted.rs")));
}
```

**Implementation Tasks:**
1. Crear `hodei-cli/src/analysis/git_analyzer.rs`
2. Usar `git2` crate para git operations
3. Implement diff parsing (added, modified, deleted)
4. File hash calculation
5. Tests con temporary repositories

**Commit**: `feat(cli): implement GitAnalyzer for incremental change detection`

---

##### ⚠️ US-10-FALLBACK: Análisis sin Git (Modo Degradado)
**Como** sistema de análisis incremental  
**Quiero** funcionar sin Git disponible  
**Para** mantener funcionalidad en entornos sin control de versiones

**Criterios de Aceptación:**
- ✅ Detección automática de ausencia de Git
- ✅ Modo "Fuerza Bruta con Caché" como fallback
- ✅ Hash de archivos para comparación con cache
- ✅ Performance degradada pero funcional
- ✅ Warning/logs indicando modo sin Git

**Explicación del Modo Sin Git:**

**Con Git (Modo Óptimo):**
- Git nos dice exactamente qué 5 archivos cambiaron
- Solo analizamos esos 5 archivos
- **Tiempo**: ~2 segundos para un PR típico
- **Eficiencia**: 95-99% de cache hit rate

**Sin Git (Modo Degradado):**
- El sistema debe revisar TODOS los archivos
- Para cada archivo, calcula su hash actual
- Compara con el hash en cache
- **Cache Hit** → Reutiliza resultado (no analiza)
- **Cache Miss** → Analiza el archivo
- **Tiempo**: ~15-30 segundos para un proyecto típico
- **Eficiencia**: 70-90% de cache hit rate (dependiendo de cambios)

**Ejemplo Práctico:**

```
Proyecto: 10,000 archivos
PR típico con Git: Solo 3 archivos modificados → 2 segundos
PR típico sin Git: Revisa 10,000 archivos × hash = ~0.1ms = 1 segundo
                    Analiza 3 archivos modificados = ~29 segundos
                    Total: ~30 segundos
```

**Trade-offs:**
- ✅ **Ventaja**: Funciona sin Git (upload folders, análisis local)
- ✅ **Ventaja**: Sigue siendo 5-10x más rápido que full analysis
- ✅ **Desventaja**: 10-15x más lento que con Git
- ✅ **Desventaja**: Requiere calcular hash de todos los archivos

**Implementation Tasks:**
1. Detectar presencia/ausencia de `.git` directory
2. Implementar `HashBasedAnalyzer` fallback
3. Integrar con `IncrementalAnalyzer` existente
4. Logging y métricas de modo activo
5. Tests para ambos modos (con y sin Git)

**Commit**: `feat(cli): add hash-based fallback for incremental analysis without Git`

---

##### US-11: Implementar Cache Manager
**Como** sistema de cache  
**Quiero** cache de results por file hash  
**Para** evitar re-procesar archivos unchanged

**Criterios de Aceptación:**
- ✅ `CacheManager` con RocksDB
- ✅ File hash + timestamp como cache key
- ✅ TTL y cache invalidation
- ✅ Hit/miss statistics

**TDD Tests (RED):**
```rust
#[test]
fn test_cache_hit_on_unchanged_file() {
    let mut cache = CacheManager::new_temp();

    let file_path = PathBuf::from("src/main.rs");
    let file_hash = calculate_file_hash(&file_path).unwrap();
    let facts = vec![create_test_fact()];

    // Store in cache
    cache.store_facts(&file_path, &file_hash, &facts);

    // Retrieve (should be hit)
    let (cached_facts, is_hit) = cache.get_facts(&file_path, &file_hash);
    assert!(is_hit);
    assert_eq!(cached_facts.len(), 1);
}

#[test]
fn test_cache_miss_on_changed_file() {
    let mut cache = CacheManager::new_temp();
    let file_path = PathBuf::from("src/main.rs");

    // Different hash = miss
    let (facts, is_hit) = cache.get_facts(&file_path, &vec![1, 2, 3]);
    assert!(!is_hit);
    assert!(facts.is_empty());
}
```

**Implementation Tasks:**
1. Agregar `rocksdb = "0.22"` a deps
2. Crear `hodei-cli/src/analysis/cache_manager.rs`
3. Hash + timestamp como key
4. Cache store/retrieve logic
5. Statistics tracking

**Commit**: `feat(cli): implement CacheManager with RocksDB for incremental analysis`

---

##### US-12: Implementar Incremental Analyzer
**Como** CLI  
**Quiero** ejecutar análisis incremental  
**Para** acelerar CI/CD pipelines

**Criterios de Aceptación:**
- ✅ `IncrementalAnalyzer` orchestrates everything
- ✅ 70-90% reducción en tiempo para typical PR
- ✅ Statistics de cache hit rate
- ✅ Fallback a full analysis si necesario

**TDD Tests (RED):**
```rust
#[test]
fn test_incremental_analysis_speedup() {
    let temp_repo = create_test_repo();
    temp_repo.commit_all("initial state");

    // Full analysis first
    let start = Instant::now();
    let full_result = analyze_full(&temp_repo);
    let full_time = start.elapsed();

    // Modify one file
    temp_repo.write_file("src/main.rs", "small change");
    temp_repo.commit_all("small change");

    // Incremental analysis
    let start = Instant::now();
    let incremental_result = analyze_incremental(&temp_repo, "HEAD~1", "HEAD").unwrap();
    let incremental_time = start.elapsed();

    // Should be much faster
    let speedup = full_time.as_millis() / incremental_time.as_millis();
    assert!(speedup > 5, "Should be 5x faster, got {}x", speedup);
}
```

**Implementation Tasks:**
1. Crear `hodei-cli/src/analysis/incremental.rs`
2. Orchestrate GitAnalyzer + CacheManager
3. Combine cached y fresh results
4. Statistics reporting
5. E2E tests con real repositories

**Commit**: `feat(cli): implement IncrementalAnalyzer for CI/CD optimization`

---

## 🎯 EPIC 05: Spatial Indexing con R-Tree
### **Impacto**: Alto | **Complejidad**: Alta | **ROI**: Medio

#### Contexto
Correlación multi-dominio (ej. SAST + Coverage) requiere queries espaciales rápidas. R-Tree es perfecto para esto.

#### User Stories

---

##### US-13: Implementar R-Tree Spatial Index
**Como** motor de correlación  
**Quiero** indexar facts por location  
**Para** queries espaciales rápidas

**Criterios de Aceptación:**
- ✅ `SpatialIndex` usando `rstar` crate
- ✅ Insert/query por file:line
- ✅ O(log N) query time
- ✅ Support para proximity queries

**TDD Tests (RED):**
```rust
#[test]
fn test_spatial_index_insert_and_query() {
    let mut index = SpatialIndex::new();

    let file = ProjectPath::new("src/main.rs").unwrap();
    let loc1 = SourceLocation::new(file.clone(), 42, 1);
    let loc2 = SourceLocation::new(file.clone(), 43, 1);

    index.insert(loc1.clone(), FactId::from_u64(1));
    index.insert(loc2.clone(), FactId::from_u64(2));

    // Query by window
    let results = index.query_window(&file, 42, 43);
    assert_eq!(results.len(), 2);
}

#[test]
fn test_spatial_index_proximity() {
    let mut index = SpatialIndex::new();
    let file = ProjectPath::new("src/main.rs").unwrap();

    for i in 0..100 {
        let loc = SourceLocation::new(file.clone(), i, 1);
        index.insert(loc, FactId::from_u64(i));
    }

    // Query proximity (within 5 lines of line 50)
    let results = index.query_proximity(&file, 50, 5);
    assert!(results.len() >= 10); // lines 45-55
    assert!(results.len() <= 11);
}
```

**Implementation Tasks:**
1. Agregar `rstar = "0.11"` a deps
2. Crear `hodei-ir/src/spatial/r_tree.rs`
3. Implement `LocationPoint` struct
4. Insert/query operations
5. Proximity queries

**Commit**: `feat(ir): add R-Tree spatial indexing for location-based queries`

---

##### US-14: Implementar Correlación Vulnerability + Coverage
**Como** sistema de análisis  
**Quiero** correlacionar vulnerabilidades con coverage  
**Para** identificar high-risk uncovered vulnerabilities

**Criterios de Aceptación:**
- ✅ `CoverageSecurityCorrelator` implementation
- ✅ Find uncovered vulnerabilities
- ✅ Calculate risk scores
- ✅ Integration con existing types

**TDD Tests (RED):**
```rust
#[test]
fn test_uncovered_vulnerability_detection() {
    let correlator = CoverageSecurityCorrelator::new();

    // Create coverage report
    let mut coverage = CoverageReport::new();
    coverage.add_uncovered_line("src/main.rs", 42);

    // Create vulnerability
    let vuln = Vulnerability::new(
        "SQL Injection",
        Location::new("src/main.rs", 42),
        Severity::Critical,
    );

    // Should detect as uncovered
    let uncovered = correlator.find_uncovered_vulnerabilities(&coverage);
    assert_eq!(uncovered.len(), 1);
    assert_eq!(uncovered[0].vulnerability.location.line, 42);
}
```

**Implementation Tasks:**
1. Crear `hodei-core/src/correlation/coverage_security.rs`
2. Implement correlation logic
3. Risk scoring algorithm
4. Integration tests
5. Documentation y examples

**Commit**: `feat(core): implement coverage-security correlation for risk assessment`

---

## 🎯 EPIC 06: SIMD Optimizations
### **Impacto**: Medio | **Complejidad**: Alta | **ROI**: Medio

#### Contexto
SIMD para operaciones data-parallel: string comparison, entropy calculation, filtering.

#### User Stories

---

##### US-15: Implementar SIMD String Comparison
**Como** motor de evaluación  
**Quiero** comparar strings con SIMD  
**Para** acelerar rule evaluation

**Criterios de Aceptación:**
- ✅ AVX2 implementation (x86_64)
- ✅ Fallback a std::simd (portable)
- ✅ 2-4x speedup en benchmarks
- ✅ Feature flags para control

**TDD Tests (RED):**
```rust
#[cfg(target_arch = "x86_64")]
#[test]
fn test_avx2_string_comparison() {
    let s1 = "src/main.rs";
    let s2 = "src/main.rs";
    let s3 = "src/lib.rs";

    assert!(simd_string_eq_avx2(s1, s2));
    assert!(!simd_string_eq_avx2(s1, s3));
}

#[cfg(feature = "portable_simd")]
#[test]
fn test_portable_simd_string_comparison() {
    let s1 = "src/main.rs";
    let s2 = "src/main.rs";

    assert!(simd_string_eq_portable(s1, s2));
}
```

**Implementation Tasks:**
1. Feature flags: `avx2`, `portable_simd`
2. AVX2 implementation con `std::arch`
3. Portable SIMD con `std::simd` (nightly)
4. CPU feature detection
5. Benchmarks

**Commit**: `perf(core): add SIMD string comparison optimizations`

---

## 📊 Estimated Timeline

| Epic | Duración | Dependencias | Prioridad |
|------|----------|--------------|-----------|
| EPIC 01: String Interning | 2 semanas | - | 1 |
| EPIC 02: Cap'n Proto | 3 semanas | EPIC 01 | 1 |
| EPIC 03: DSL Security | 4 semanas | - | 1 |
| EPIC 04: Incremental Analysis | 3 semanas | EPIC 01, 02 | 1 |
| EPIC 05: Spatial Indexing | 3 semanas | EPIC 01 | 2 |
| EPIC 06: SIMD | 2 semanas | EPIC 02 | 3 |

**Total estimado**: 17 semanas (4 meses)

---

## 🔍 Definition of Done (DoD)

Para cada historia de usuario:
- ✅ Test coverage > 90%
- ✅ Benchmarks integrados en CI
- ✅ Documentación KDoc completa
- ✅ Performance targets alcanzados
- ✅ Security review completado
- ✅ Code review aprobado
- ✅ Commit con mensaje Conventional Commits

---

## 🚀 Getting Started

**Epic 01** (String Interning) es el mejor starting point:
1. Quick win (2 semanas)
2. Low complexity
3. High impact
4. Foundation para otras optimizaciones

**Next Steps:**
1. Implementar US-01 (String Interner)
2. Run benchmarks
3. Merge y release
4. Continuar con US-02

---

*Este documento sigue el enfoque TDD y se actualizará conforme avancen las implementaciones.*
