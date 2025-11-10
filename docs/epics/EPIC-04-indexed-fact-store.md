# EPIC-04: Indexed Fact Store & Query Planner

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: Hodei Scan v3.2  
**Dependencias**: EPIC-02 (IR Core), EPIC-03 (Zero-Copy)  
**Owner**: Core Team  
**Prioridad**: Critical Path

---

## 1. Resumen Ejecutivo

Implementar el **IndexedFactStore**, un almacén de hechos optimizado con múltiples índices especializados (por tipo, espacial, por flujo) y un **QueryPlanner** que elige automáticamente la estrategia de consulta óptima. Objetivo: consultas sobre millones de hechos en microsegundos.

### Objetivo de Negocio
Permitir que el Rule Engine evalúe reglas complejas (con joins espaciales, seguimiento de flujos) sobre IRs grandes (100MB+) con latencia <1ms, habilitando análisis interactivo y gates en CI/CD.

### Métricas de Éxito
- **Rendimiento**: Query simple (by_type) sobre 1M hechos en <10μs.
- **Escalabilidad**: Spatial join (2 tipos, misma ubicación) sobre 100k hechos en <500μs.
- **Complejidad**: Flow join (3+ saltos) sobre 10k flows en <2ms.
- **Memoria**: Overhead de índices <20% del tamaño del IR.

---

## 2. Contexto Técnico

### 2.1. Problema
El Rule Engine necesita evaluar condiciones como:
```rust
// "Encuentra todos los TaintSink en líneas sin cobertura"
facts.by_type(FactType::TaintSink)
  .filter(|sink| {
    facts.by_location(sink.location)
      .any(|f| matches!(f.fact_type, FactType::UncoveredLine))
  })
```

Sin índices, esto requeriría O(N×M) comparaciones (full scan por cada sink). Con 100k sinks y 1M hechos, eso son 100B comparaciones → decenas de segundos.

### 2.2. Solución: Índices Especializados
- **TypeIndex**: HashMap<FactType, Vec<FactId>> → O(1) lookup.
- **SpatialIndex**: R-tree espacial para `by_location(file, line_range)`.
- **FlowIndex**: Grafo dirigido para `follow_flow(source_id)`.
- **QueryPlanner**: Analiza la consulta y elige el plan óptimo (TypeIndexScan, SpatialJoin, FlowJoin).

### 2.3. Diseño de Alto Nivel
```
┌─────────────────────────────────────────────────────┐
│         IndexedFactStore                            │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────┐ │
│  │ TypeIndex    │  │ SpatialIndex │  │FlowIndex │ │
│  │ HashMap<...> │  │ RTree<...>   │  │ DiGraph  │ │
│  └──────────────┘  └──────────────┘  └──────────┘ │
│                                                     │
│  ┌─────────────────────────────────────────────┐  │
│  │          QueryPlanner                       │  │
│  │  - analyzes query predicates                │  │
│  │  - estimates costs (cardinality, selectivity)│ │
│  │  - chooses optimal plan                     │  │
│  └─────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

---

## 3. Alcance

### 3.1. En Alcance (MUST)
1. **TypeIndex**: Índice por FactType.
2. **SpatialIndex**: Índice espacial 2D (file × line_range) usando `rstar` crate.
3. **FlowIndex**: Índice de flujo de datos (grafo dirigido) usando `petgraph`.
4. **QueryPlanner**: Planeador de consultas con estimación de costos.
5. **Query Execution**: Operadores `TypeIndexScan`, `SpatialJoin`, `FlowJoin`, `FullScan`.
6. **Benchmarks**: Suite completa con criterion midiendo cada tipo de consulta.

### 3.2. En Alcance (SHOULD)
7. **Composite Index**: Índice compuesto (tipo + file) para consultas frecuentes.
8. **Statistics**: Colectar estadísticas (cardinality, selectivity) durante indexación.
9. **Parallel Indexing**: Construcción de índices en paralelo con rayon.

### 3.3. Fuera de Alcance
- Persistencia de índices (v3.3).
- Actualización incremental de índices (v3.3).
- Índices de texto completo (v4.0).

---

## 4. Arquitectura Detallada

### 4.1. Tipos Core

```rust
// hodei-engine/src/store/mod.rs
use hodei_ir::{Fact, FactId, FactType, SourceLocation};
use rstar::{RTree, RTreeObject, AABB};
use petgraph::graph::DiGraph;
use std::collections::HashMap;

/// Store con múltiples índices
pub struct IndexedFactStore {
    /// Todos los hechos (slice zero-copy o Vec owned)
    facts: Vec<Fact>,
    
    /// Índice por tipo
    type_index: TypeIndex,
    
    /// Índice espacial
    spatial_index: SpatialIndex,
    
    /// Índice de flujo de datos
    flow_index: FlowIndex,
    
    /// Estadísticas para el planner
    stats: IndexStats,
}

impl IndexedFactStore {
    pub fn new(facts: Vec<Fact>) -> Self {
        let type_index = TypeIndex::build(&facts);
        let spatial_index = SpatialIndex::build(&facts);
        let flow_index = FlowIndex::build(&facts);
        let stats = IndexStats::compute(&facts);
        
        Self { facts, type_index, spatial_index, flow_index, stats }
    }
    
    // API de consulta (high-level)
    pub fn by_type(&self, fact_type: FactType) -> impl Iterator<Item = &Fact> {
        self.type_index.get(fact_type)
            .map(move |ids| ids.iter().map(move |id| &self.facts[id.0]))
            .into_iter()
            .flatten()
    }
    
    pub fn by_location(&self, file: &str, line_range: Range<u32>) 
        -> impl Iterator<Item = &Fact> 
    {
        let bbox = BoundingBox::from_file_lines(file, line_range);
        self.spatial_index.query(bbox)
            .map(move |id| &self.facts[id.0])
    }
    
    pub fn follow_flow(&self, fact_id: FactId) -> Vec<&Fact> {
        self.flow_index.reachable_from(fact_id)
            .into_iter()
            .map(|id| &self.facts[id.0])
            .collect()
    }
}
```

### 4.2. TypeIndex

```rust
// hodei-engine/src/store/type_index.rs
use hodei_ir::{Fact, FactId, FactType};
use std::collections::HashMap;

pub struct TypeIndex {
    index: HashMap<FactType, Vec<FactId>>,
}

impl TypeIndex {
    pub fn build(facts: &[Fact]) -> Self {
        let mut index: HashMap<FactType, Vec<FactId>> = HashMap::new();
        
        for (i, fact) in facts.iter().enumerate() {
            index.entry(fact.fact_type)
                .or_default()
                .push(FactId(i));
        }
        
        // Ordenar por FactId para mejorar cache locality
        for ids in index.values_mut() {
            ids.sort_unstable();
        }
        
        Self { index }
    }
    
    pub fn get(&self, fact_type: FactType) -> Option<&[FactId]> {
        self.index.get(&fact_type).map(|v| v.as_slice())
    }
    
    pub fn cardinality(&self, fact_type: FactType) -> usize {
        self.index.get(&fact_type).map_or(0, |v| v.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodei_ir::test_utils::*;
    
    #[test]
    fn type_index_lookup_is_fast() {
        let facts = generate_facts(1_000_000);
        let index = TypeIndex::build(&facts);
        
        let start = std::time::Instant::now();
        let sinks = index.get(FactType::TaintSink).unwrap();
        let elapsed = start.elapsed();
        
        assert!(elapsed.as_micros() < 10, "Lookup must be <10μs");
        assert!(sinks.len() > 0);
    }
}
```

### 4.3. SpatialIndex

```rust
// hodei-engine/src/store/spatial_index.rs
use hodei_ir::{Fact, FactId, SourceLocation};
use rstar::{RTree, RTreeObject, AABB};
use std::ops::Range;

/// Wrapper para hacer Fact compatible con RTree
#[derive(Clone)]
struct SpatialEntry {
    fact_id: FactId,
    bbox: BoundingBox,
}

#[derive(Clone, Debug)]
pub struct BoundingBox {
    file_hash: u64,  // Hash del path para comparaciones rápidas
    line_start: u32,
    line_end: u32,
    col_start: u32,
    col_end: u32,
}

impl RTreeObject for SpatialEntry {
    type Envelope = AABB<[f64; 3]>;
    
    fn envelope(&self) -> Self::Envelope {
        // Mapeamos (file_hash, line, col) a espacio 3D
        let min = [
            self.bbox.file_hash as f64,
            self.bbox.line_start as f64,
            self.bbox.col_start as f64,
        ];
        let max = [
            self.bbox.file_hash as f64,
            self.bbox.line_end as f64,
            self.bbox.col_end as f64,
        ];
        AABB::from_corners(min, max)
    }
}

pub struct SpatialIndex {
    rtree: RTree<SpatialEntry>,
}

impl SpatialIndex {
    pub fn build(facts: &[Fact]) -> Self {
        let entries: Vec<SpatialEntry> = facts.iter()
            .enumerate()
            .filter_map(|(i, fact)| {
                fact.source_location.as_ref().map(|loc| {
                    SpatialEntry {
                        fact_id: FactId(i),
                        bbox: BoundingBox::from_location(loc),
                    }
                })
            })
            .collect();
        
        let rtree = RTree::bulk_load(entries);
        Self { rtree }
    }
    
    pub fn query(&self, bbox: BoundingBox) -> impl Iterator<Item = FactId> + '_ {
        self.rtree.locate_in_envelope_intersecting(&bbox.envelope())
            .map(|entry| entry.fact_id)
    }
}

impl BoundingBox {
    pub fn from_location(loc: &SourceLocation) -> Self {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        loc.file.hash(&mut hasher);
        let file_hash = hasher.finish();
        
        Self {
            file_hash,
            line_start: loc.start.line.0,
            line_end: loc.end.line.0,
            col_start: loc.start.column.0,
            col_end: loc.end.column.0,
        }
    }
    
    pub fn from_file_lines(file: &str, line_range: Range<u32>) -> Self {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        file.hash(&mut hasher);
        
        Self {
            file_hash: hasher.finish(),
            line_start: line_range.start,
            line_end: line_range.end,
            col_start: 0,
            col_end: u32::MAX,
        }
    }
    
    fn envelope(&self) -> AABB<[f64; 3]> {
        AABB::from_corners(
            [self.file_hash as f64, self.line_start as f64, self.col_start as f64],
            [self.file_hash as f64, self.line_end as f64, self.col_end as f64],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn spatial_join_is_fast() {
        let facts = generate_facts_with_locations(100_000);
        let index = SpatialIndex::build(&facts);
        
        let bbox = BoundingBox::from_file_lines("src/main.rs", 100..200);
        
        let start = std::time::Instant::now();
        let results: Vec<_> = index.query(bbox).collect();
        let elapsed = start.elapsed();
        
        assert!(elapsed.as_micros() < 500, "Spatial query must be <500μs");
        assert!(results.len() > 0);
    }
}
```

### 4.4. FlowIndex

```rust
// hodei-engine/src/store/flow_index.rs
use hodei_ir::{Fact, FactId, FactType};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::dijkstra;
use std::collections::HashMap;

pub struct FlowIndex {
    graph: DiGraph<FactId, ()>,
    fact_to_node: HashMap<FactId, NodeIndex>,
}

impl FlowIndex {
    pub fn build(facts: &[Fact]) -> Self {
        let mut graph = DiGraph::new();
        let mut fact_to_node = HashMap::new();
        
        // 1. Crear nodos para todos los hechos con flow_id
        for (i, fact) in facts.iter().enumerate() {
            if let Some(_flow_id) = &fact.flow_id {
                let fact_id = FactId(i);
                let node = graph.add_node(fact_id);
                fact_to_node.insert(fact_id, node);
            }
        }
        
        // 2. Crear aristas basadas en DataFlow facts
        for (i, fact) in facts.iter().enumerate() {
            if let FactType::DataFlow { from, to } = &fact.fact_type {
                if let (Some(&from_node), Some(&to_node)) = 
                    (fact_to_node.get(from), fact_to_node.get(to)) 
                {
                    graph.add_edge(from_node, to_node, ());
                }
            }
        }
        
        Self { graph, fact_to_node }
    }
    
    /// Encuentra todos los hechos alcanzables desde fact_id siguiendo flujos
    pub fn reachable_from(&self, fact_id: FactId) -> Vec<FactId> {
        let Some(&start_node) = self.fact_to_node.get(&fact_id) else {
            return vec![];
        };
        
        let distances = dijkstra(&self.graph, start_node, None, |_| 1);
        
        distances.keys()
            .map(|&node| self.graph[node])
            .collect()
    }
    
    /// Encuentra el camino más corto entre dos hechos
    pub fn shortest_path(&self, from: FactId, to: FactId) -> Option<Vec<FactId>> {
        let start_node = self.fact_to_node.get(&from)?;
        let end_node = self.fact_to_node.get(&to)?;
        
        petgraph::algo::astar(
            &self.graph,
            *start_node,
            |n| n == *end_node,
            |_| 1,
            |_| 0,
        )
        .map(|(_, path)| path.into_iter().map(|n| self.graph[n]).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn flow_traversal_is_fast() {
        let facts = generate_facts_with_flows(10_000, /* avg_depth */ 5);
        let index = FlowIndex::build(&facts);
        
        let source_id = FactId(0);
        
        let start = std::time::Instant::now();
        let reachable = index.reachable_from(source_id);
        let elapsed = start.elapsed();
        
        assert!(elapsed.as_millis() < 2, "Flow query must be <2ms");
        assert!(reachable.len() > 0);
    }
}
```

### 4.5. QueryPlanner

```rust
// hodei-engine/src/store/planner.rs
use hodei_ir::FactType;
use std::ops::Range;

#[derive(Debug, Clone)]
pub enum QueryPlan {
    /// Full scan (último recurso)
    FullScan,
    
    /// Lookup por tipo
    TypeIndexScan { fact_type: FactType },
    
    /// Join espacial (dos tipos en misma ubicación)
    SpatialJoin {
        left_type: FactType,
        right_type: FactType,
        file: String,
        line_range: Range<u32>,
    },
    
    /// Traversal de flujo
    FlowJoin {
        start_type: FactType,
        max_depth: usize,
    },
}

pub struct QueryPlanner<'a> {
    stats: &'a IndexStats,
}

impl<'a> QueryPlanner<'a> {
    pub fn new(stats: &'a IndexStats) -> Self {
        Self { stats }
    }
    
    /// Elige el plan óptimo basándose en estadísticas
    pub fn plan(&self, query: &Query) -> QueryPlan {
        // Ejemplo simplificado: elige el índice más selectivo
        match query {
            Query::ByType { fact_type } => {
                QueryPlan::TypeIndexScan { fact_type: *fact_type }
            }
            
            Query::SpatialAnd { left_type, right_type, location } => {
                // Estimar cardinality de cada rama
                let left_card = self.stats.cardinality(*left_type);
                let right_card = self.stats.cardinality(*right_type);
                
                // Empezar por el más selectivo
                if left_card < right_card {
                    QueryPlan::SpatialJoin {
                        left_type: *left_type,
                        right_type: *right_type,
                        file: location.file.clone(),
                        line_range: location.line_range.clone(),
                    }
                } else {
                    QueryPlan::SpatialJoin {
                        left_type: *right_type,
                        right_type: *left_type,
                        file: location.file.clone(),
                        line_range: location.line_range.clone(),
                    }
                }
            }
            
            Query::FlowReachable { start_type, max_depth } => {
                QueryPlan::FlowJoin {
                    start_type: *start_type,
                    max_depth: *max_depth,
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct IndexStats {
    type_cardinality: HashMap<FactType, usize>,
    total_facts: usize,
}

impl IndexStats {
    pub fn compute(facts: &[Fact]) -> Self {
        let mut type_cardinality = HashMap::new();
        for fact in facts {
            *type_cardinality.entry(fact.fact_type).or_insert(0) += 1;
        }
        
        Self {
            type_cardinality,
            total_facts: facts.len(),
        }
    }
    
    pub fn cardinality(&self, fact_type: FactType) -> usize {
        self.type_cardinality.get(&fact_type).copied().unwrap_or(0)
    }
    
    pub fn selectivity(&self, fact_type: FactType) -> f64 {
        self.cardinality(fact_type) as f64 / self.total_facts as f64
    }
}
```

---

## 5. Plan de Implementación

### 5.1. Fases

**Fase 1: TypeIndex** (Semana 1)
- [ ] Implementar `TypeIndex` con HashMap.
- [ ] Tests unitarios (lookup, cardinality).
- [ ] Benchmark con 1M hechos.
- [ ] Documentación API.

**Fase 2: SpatialIndex** (Semana 2)
- [ ] Integrar `rstar` crate.
- [ ] Implementar `BoundingBox` y `SpatialEntry`.
- [ ] Implementar `SpatialIndex::build` con bulk load.
- [ ] Tests: query por file+line_range.
- [ ] Benchmark: spatial join 100k hechos.

**Fase 3: FlowIndex** (Semana 2-3)
- [ ] Integrar `petgraph` crate.
- [ ] Implementar `FlowIndex::build` (grafo dirigido).
- [ ] Implementar `reachable_from` y `shortest_path`.
- [ ] Tests: traversal con ciclos.
- [ ] Benchmark: flow query 10k flows, depth=5.

**Fase 4: QueryPlanner** (Semana 3)
- [ ] Implementar `IndexStats`.
- [ ] Implementar `QueryPlanner::plan` (estimación de costos).
- [ ] Tests: verificar que elige el plan óptimo.
- [ ] Benchmark: comparar planes (FullScan vs TypeIndexScan).

**Fase 5: Integración** (Semana 4)
- [ ] Integrar todos los índices en `IndexedFactStore`.
- [ ] Implementar API de alto nivel (`by_type`, `by_location`, `follow_flow`).
- [ ] Tests end-to-end: consultas complejas.
- [ ] Benchmarks completos (suite criterion).
- [ ] Documentación de uso.

### 5.2. Dependencias de Crates
```toml
[dependencies]
rstar = "0.12"          # R-tree espacial
petgraph = "0.6"        # Grafo para flujos
rayon = "1.10"          # Paralelización
ahash = "0.8"           # HashMap más rápido que std

[dev-dependencies]
criterion = "0.5"
proptest = "1.4"
```

---

## 6. Tests & Validación

### 6.1. Tests Unitarios

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn type_index_empty() {
        let facts = vec![];
        let index = TypeIndex::build(&facts);
        assert_eq!(index.get(FactType::TaintSink), None);
    }
    
    #[test]
    fn type_index_multiple_types() {
        let facts = vec![
            Fact { fact_type: FactType::TaintSink, ..default() },
            Fact { fact_type: FactType::UncoveredLine, ..default() },
            Fact { fact_type: FactType::TaintSink, ..default() },
        ];
        let index = TypeIndex::build(&facts);
        
        assert_eq!(index.cardinality(FactType::TaintSink), 2);
        assert_eq!(index.cardinality(FactType::UncoveredLine), 1);
    }
    
    #[test]
    fn spatial_index_same_file_query() {
        let facts = vec![
            fact_at("src/main.rs", 10, 10),
            fact_at("src/main.rs", 50, 50),
            fact_at("src/lib.rs", 10, 10),
        ];
        let index = SpatialIndex::build(&facts);
        
        let bbox = BoundingBox::from_file_lines("src/main.rs", 0..100);
        let results: Vec<_> = index.query(bbox).collect();
        
        assert_eq!(results.len(), 2);
    }
    
    #[test]
    fn flow_index_reachability() {
        let facts = vec![
            fact_with_flow(0, Some(flow_id_1)),  // Source
            fact_data_flow(1, FactId(0), FactId(2)),
            fact_with_flow(2, Some(flow_id_1)),  // Sink
        ];
        let index = FlowIndex::build(&facts);
        
        let reachable = index.reachable_from(FactId(0));
        assert!(reachable.contains(&FactId(2)));
    }
}
```

### 6.2. Property Tests

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn type_index_cardinality_sum_equals_total(facts in vec(fact_strategy(), 0..1000)) {
        let index = TypeIndex::build(&facts);
        let sum: usize = FactType::all_variants()
            .map(|ft| index.cardinality(ft))
            .sum();
        prop_assert_eq!(sum, facts.len());
    }
    
    #[test]
    fn spatial_index_never_misses_facts_in_range(facts in vec(fact_with_loc_strategy(), 0..1000)) {
        let index = SpatialIndex::build(&facts);
        
        for fact in &facts {
            if let Some(loc) = &fact.source_location {
                let bbox = BoundingBox::from_location(loc);
                let results: Vec<_> = index.query(bbox).collect();
                prop_assert!(results.contains(&fact.id));
            }
        }
    }
}
```

### 6.3. Benchmarks

```rust
// benches/indexed_store.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use hodei_engine::store::*;
use hodei_ir::test_utils::*;

fn bench_type_index(c: &mut Criterion) {
    let mut group = c.benchmark_group("TypeIndex");
    
    for size in [1_000, 10_000, 100_000, 1_000_000] {
        let facts = generate_facts(size);
        let index = TypeIndex::build(&facts);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), &index, |b, idx| {
            b.iter(|| {
                black_box(idx.get(FactType::TaintSink))
            });
        });
    }
    group.finish();
}

fn bench_spatial_join(c: &mut Criterion) {
    let mut group = c.benchmark_group("SpatialJoin");
    
    for size in [10_000, 50_000, 100_000] {
        let facts = generate_facts_with_locations(size);
        let index = SpatialIndex::build(&facts);
        let bbox = BoundingBox::from_file_lines("src/main.rs", 0..1000);
        
        group.bench_with_input(BenchmarkId::from_parameter(size), &index, |b, idx| {
            b.iter(|| {
                let results: Vec<_> = idx.query(bbox.clone()).collect();
                black_box(results)
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_type_index, bench_spatial_join);
criterion_main!(benches);
```

---

## 7. Métricas de Rendimiento

### 7.1. Targets

| Operación | Tamaño IR | Target | Medido |
|-----------|-----------|--------|---------|
| TypeIndex lookup | 1M hechos | <10μs | TBD |
| Spatial query | 100k hechos | <500μs | TBD |
| Flow traversal (depth=5) | 10k flows | <2ms | TBD |
| Full index build | 1M hechos | <500ms | TBD |

### 7.2. Overhead de Memoria

```
IR Size: 100 MB
TypeIndex: ~1 MB (HashMap<FactType, Vec<FactId>>)
SpatialIndex: ~15 MB (RTree con N=100k nodos)
FlowIndex: ~10 MB (DiGraph con N=50k nodos, E=100k edges)
Total Overhead: ~26 MB → 26% overhead ✅
```

---

## 8. Seguridad & Mitigaciones

### 8.1. Riesgos

1. **DoS via Query Complexity**
   - Riesgo: Query con spatial join + flow traversal → O(N²M) complejidad.
   - Mitigación: Timeouts en `QueryPlanner`; límites de profundidad en flow queries.

2. **Memory Exhaustion**
   - Riesgo: Índices muy grandes (>1GB).
   - Mitigación: Estimar tamaño antes de construir; opción de skip indexes si memoria insuficiente.

3. **Hash Collision en SpatialIndex**
   - Riesgo: Colisión de file_hash → false positives en spatial queries.
   - Mitigación: Usar 128-bit hash (xxhash); filtrado post-query comparando paths.

---

## 9. Documentación

### 9.1. Guía de Uso

```rust
use hodei_engine::store::IndexedFactStore;
use hodei_ir::{FactType, IntermediateRepresentation};

// 1. Cargar IR
let ir = IntermediateRepresentation::from_capnp_file("app.ir")?;

// 2. Construir índices
let store = IndexedFactStore::new(ir.facts);

// 3. Query simple
for sink in store.by_type(FactType::TaintSink) {
    println!("Found sink: {:?}", sink);
}

// 4. Spatial query
let uncovered = store.by_location("src/auth.rs", 100..200);

// 5. Flow traversal
let reachable = store.follow_flow(sink_fact_id);
```

### 9.2. ADRs Relacionados
- ADR-001: Facts Must Be Atomic (índices solo sobre hechos atómicos).
- ADR-004: Spatial Index with RTree (decisión de usar rstar en vez de custom).

---

## 10. Criterios de Aceptación

- [ ] **Funcional**: Todos los tests unitarios y property tests pasan.
- [ ] **Rendimiento**: Todos los benchmarks cumplen targets (tabla §7.1).
- [ ] **Memoria**: Overhead <30% del tamaño del IR.
- [ ] **Documentación**: README con ejemplos, API docs al 100%.
- [ ] **Seguridad**: Timeouts y límites implementados.
- [ ] **CI**: Benchmarks automáticos en GH Actions; alertas si regresión >10%.

---

## 11. Notas & Referencias

- **rstar**: https://docs.rs/rstar/
- **petgraph**: https://docs.rs/petgraph/
- Paper: "R-Trees: A Dynamic Index Structure for Spatial Searching" (Guttman, 1984).
- Comparativa spatial indexes: https://blog.mapbox.com/a-dive-into-spatial-search-algorithms-ebd0c5e39d2a

---

**Última Actualización**: 2025-01-XX  
**Próxima Revisión**: Después de Fase 2 (SpatialIndex implementado)
