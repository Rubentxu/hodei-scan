# EPIC-03: Zero-Copy Serialization con Cap'n Proto

**Fase:** Foundation  
**Story Points:** 55  
**Prioridad:** HIGH  
**Dependencias:** EPIC-02  
**Owner:** Core Team

---

## 📋 Contexto

Implementar serialización zero-copy usando Cap'n Proto para lograr una mejora de 200,000x en la carga del IR comparado con JSON. Esta es una optimización crítica que permite procesar proyectos grandes (100MB+ de IR) en microsegundos en lugar de segundos.

---

## 🎯 Objetivos Específicos

1. Definir schema Cap'n Proto completo del IR
2. Implementar deserialización zero-copy con mmap
3. Generar bindings Rust con capnpc
4. Crear adaptadores JSON ↔ Cap'n Proto
5. Validar performance: 100MB IR en <100μs

---

## 📊 Performance Goals

| Métrica | JSON (Baseline) | Cap'n Proto (Target) | Mejora |
|---------|-----------------|---------------------|--------|
| **Carga 100MB IR** | ~2,000ms | <0.1ms (100μs) | 20,000x |
| **Memoria peak** | 400MB | 100MB | 4x |
| **Serialización** | 1,500ms | 50ms | 30x |
| **Tamaño archivo** | 100MB | 120MB | -20% |

**Trade-off:** Archivos Cap'n Proto son ~20% más grandes, pero carga es 20,000x más rápida.

---

## 🏗️ Arquitectura

```
┌─────────────────────────────────────────────────────────────────┐
│ EXTRACTOR (Stage 1)                                             │
│   ↓                                                             │
│ IR (in-memory, Rust types)                                      │
│   ↓                                                             │
│ ┌──────────────────┐     ┌──────────────────┐                 │
│ │ JSON Serializer  │ OR  │ Cap'n Proto      │                 │
│ │ (development)    │     │ (production)     │                 │
│ └────────┬─────────┘     └────────┬─────────┘                 │
│          ↓                         ↓                            │
│   ┌───────────┐            ┌──────────────┐                   │
│   │ ir.json   │            │ ir.capnp     │                   │
│   │ (~100MB)  │            │ (~120MB)     │                   │
│   └─────┬─────┘            └──────┬───────┘                   │
└─────────┼──────────────────────────┼───────────────────────────┘
          │                          │
          ↓                          ↓
┌─────────────────────────────────────────────────────────────────┐
│ ENGINE (Stage 2)                                                │
│   ↓                                                             │
│ ┌──────────────────┐     ┌──────────────────┐                 │
│ │ JSON Parser      │ OR  │ mmap + Cap'n     │                 │
│ │ (2000ms)         │     │ Proto Reader     │                 │
│ │                  │     │ (0.1ms)          │                 │
│ └────────┬─────────┘     └────────┬─────────┘                 │
│          ↓                         ↓                            │
│   Indexed Facts (zero-copy references)                         │
│          ↓                                                      │
│   Rule Engine (evaluates)                                      │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📖 User Stories

### US-03.1: Definir Schema Cap'n Proto del IR

**Como** arquitecto del sistema  
**Quiero** un schema Cap'n Proto completo y versionado del IR  
**Para** garantizar compatibilidad binaria y evolución del formato

**Criterios de Aceptación:**
- [ ] Archivo `ir.capnp` con todos los tipos del IR v3.2
- [ ] Schema versionado con ID único (@0x...)
- [ ] Enum annotations para discriminants
- [ ] Soporte para 17 FactType variants (union)
- [ ] Documentación de cada campo
- [ ] Tests de compilación del schema
- [ ] Generación de bindings Rust sin errores

**Principios Aplicados:**
- **Schema Evolution:** Cap'n Proto permite añadir campos sin romper compatibilidad
- **Type Safety:** Bindings generados son type-safe
- **Zero-Copy:** Structs son read-only views sobre mmap buffer

---

#### Tareas Técnicas (TDD)

##### T-03.1.1: Schema base y tipos fundamentales

**1. 🔴 RED: Script de validación del schema**

```bash
#!/bin/bash
# scripts/validate_capnp_schema.sh

set -e

echo "Validating Cap'n Proto schema..."

# 1. Schema compila sin errores
capnp compile -o- ir.capnp > /dev/null
echo "✓ Schema compiles"

# 2. Puede generar Rust bindings
capnp compile -o rust ir.capnp
echo "✓ Rust codegen works"

# 3. Verifica que todos los tipos están presentes
required_types=(
    "Fact"
    "FactType"
    "Confidence"
    "ProjectPath"
    "SourceLocation"
    "Provenance"
    "IntermediateRepresentation"
)

for type in "${required_types[@]}"; do
    if ! grep -q "struct $type" ir.capnp; then
        echo "✗ Missing type: $type"
        exit 1
    fi
done
echo "✓ All required types present"

echo "Schema validation passed!"
```

**2. 🟢 GREEN: Schema inicial**

```capnp
# hodei-ir/schema/ir.capnp

@0xf0a1b2c3d4e5f601;  # Unique schema ID

# ═══════════════════════════════════════════════════════════════════
# Core Types
# ═══════════════════════════════════════════════════════════════════

struct Confidence {
  value @0 :Float64;  # Rango: [0.0, 1.0]
}

struct ProjectPath {
  canonical @0 :Text;  # Canonical absolute path
}

struct LineNumber {
  value @0 :UInt32;  # >= 1
}

struct ColumnNumber {
  value @0 :UInt32;  # >= 1
}

struct FlowId {
  uuid @0 :Data;  # 16 bytes (UUID)
}

struct SourceLocation {
  file @0 :ProjectPath;
  startLine @1 :LineNumber;
  startColumn @2 :ColumnNumber;
  endLine @3 :LineNumber;
  endColumn @4 :ColumnNumber;
}

# ═══════════════════════════════════════════════════════════════════
# Enums
# ═══════════════════════════════════════════════════════════════════

enum Severity {
  info @0;
  minor @1;
  major @2;
  critical @3;
  blocker @4;
}

enum TaintSourceType {
  httpRequestParam @0;
  httpRequestHeader @1;
  httpRequestBody @2;
  databaseQuery @3;
  fileSystem @4;
  environmentVariable @5;
  commandLineArgument @6;
  network @7;
  userInput @8;
}

enum SinkCategory {
  sqlQuery @0;
  noSqlQuery @1;
  commandExecution @2;
  fileSystemWrite @3;
  fileSystemRead @4;
  network @5;
  deserialization @6;
  eval @7;
  htmlRender @8;
  xpathQuery @9;
  ldapQuery @10;
}

enum Ecosystem {
  npm @0;
  cargo @1;
  maven @2;
  gradle @3;
  pypi @4;
  nuget @5;
  go @6;
  rubyGems @7;
  composer @8;
}

# ═══════════════════════════════════════════════════════════════════
# FactType (Union)
# ═══════════════════════════════════════════════════════════════════

struct FactType {
  union {
    # SAST
    taintSource @0 :TaintSourceData;
    taintSink @1 :TaintSinkData;
    sanitization @2 :SanitizationData;
    unsafeCall @3 :UnsafeCallData;
    cryptographicOperation @4 :CryptographicOperationData;
    vulnerability @5 :VulnerabilityData;
    
    # Quality
    function @6 :FunctionData;
    variable @7 :VariableData;
    codeSmell @8 :CodeSmellData;
    complexityViolation @9 :ComplexityViolationData;
    
    # SCA
    dependency @10 :DependencyData;
    dependencyVulnerability @11 :DependencyVulnerabilityData;
    license @12 :LicenseData;
    
    # Coverage
    uncoveredLine @13 :UncoveredLineData;
    lowTestCoverage @14 :LowTestCoverageData;
    coverageStats @15 :CoverageStatsData;
  }
}

struct TaintSourceData {
  var @0 :Text;
  flowId @1 :FlowId;
  sourceType @2 :TaintSourceType;
  confidence @3 :Confidence;
}

struct TaintSinkData {
  func @0 :Text;
  consumesFlow @1 :FlowId;
  category @2 :SinkCategory;
  severity @3 :Severity;
}

struct SanitizationData {
  method @0 :Text;
  sanitizesFlow @1 :FlowId;
  effective @2 :Bool;
  confidence @3 :Confidence;
}

struct UnsafeCallData {
  functionName @0 :Text;
  reason @1 :Text;
  severity @2 :Severity;
}

struct CryptographicOperationData {
  algorithm @0 :Text;
  keyLength @1 :UInt32;
  secure @2 :Bool;
  recommendation @3 :Text;
}

struct VulnerabilityData {
  cweId @0 :Text;
  owaspCategory @1 :Text;
  severity @2 :Severity;
  cvssScore @3 :Float32;
  description @4 :Text;
  confidence @5 :Confidence;
}

struct FunctionData {
  name @0 :Text;
  visibility @1 :Text;
  cyclomaticComplexity @2 :UInt32;
  cognitiveComplexity @3 :UInt32;
  linesOfCode @4 :UInt32;
  parameterCount @5 :UInt32;
}

struct VariableData {
  name @0 :Text;
  scope @1 :Text;
  mutability @2 :Text;
  varType @3 :Text;
}

struct CodeSmellData {
  smellType @0 :Text;
  severity @1 :Severity;
  message @2 :Text;
}

struct ComplexityViolationData {
  metric @0 :Text;
  actual @1 :UInt32;
  threshold @2 :UInt32;
}

struct DependencyData {
  name @0 :Text;
  version @1 :Text;
  ecosystem @2 :Ecosystem;
  scope @3 :Text;
  direct @4 :Bool;
}

struct DependencyVulnerabilityData {
  dependency @0 :Text;
  cveId @1 :Text;
  severity @2 :Severity;
  cvssScore @3 :Float32;
  affectedVersion @4 :Text;
  patchedVersion @5 :Text;
  description @6 :Text;
}

struct LicenseData {
  dependency @0 :Text;
  licenseType @1 :Text;
  compatible @2 :Bool;
  spdxId @3 :Text;
}

struct UncoveredLineData {
  location @0 :SourceLocation;
  coverage @1 :Float32;
  branchCoverage @2 :Float32;
}

struct LowTestCoverageData {
  file @0 :ProjectPath;
  percentage @1 :Float32;
  totalLines @2 :UInt32;
  coveredLines @3 :UInt32;
}

struct CoverageStatsData {
  scope @0 :Text;
  path @1 :ProjectPath;
  lineCoverage @2 :Float32;
  branchCoverage @3 :Float32;
  functionCoverage @4 :Float32;
}

# ═══════════════════════════════════════════════════════════════════
# Fact
# ═══════════════════════════════════════════════════════════════════

struct Provenance {
  extractor @0 :Text;
  version @1 :Text;
  confidence @2 :Confidence;
}

struct Fact {
  id @0 :Data;  # 16 bytes (UUID)
  factType @1 :FactType;
  location @2 :SourceLocation;
  provenance @3 :Provenance;
  extractedAt @4 :Int64;  # Unix timestamp (microseconds)
}

# ═══════════════════════════════════════════════════════════════════
# IR Container
# ═══════════════════════════════════════════════════════════════════

struct ProjectMetadata {
  name @0 :Text;
  version @1 :Text;
  rootPath @2 :Text;
  language @3 :Text;
  gitCommit @4 :Text;
  gitBranch @5 :Text;
}

struct AnalysisStats {
  totalFacts @0 :UInt32;
  factsByType @1 :List(FactTypeCount);
  extractorsUsed @2 :List(Text);
  duration @3 :UInt64;  # milliseconds
}

struct FactTypeCount {
  factType @0 :Text;
  count @1 :UInt32;
}

struct SchemaVersion {
  major @0 :UInt16;
  minor @1 :UInt16;
}

struct IntermediateRepresentation {
  analysisId @0 :Data;  # 16 bytes (UUID)
  timestamp @1 :Int64;  # Unix timestamp (microseconds)
  metadata @2 :ProjectMetadata;
  facts @3 :List(Fact);
  stats @4 :AnalysisStats;
  schemaVersion @5 :SchemaVersion;
}
```

**3. 🔵 REFACTOR: Añadir comentarios y optimizaciones**

```capnp
# Añadir $import para compartir schemas
using import "common.capnp".UUID;

# Usar aliases para tipos comunes
using UUID = Data;  # 16 bytes

# Optimización: struct packing para reducir padding
struct Confidence $packed {
  value @0 :Float64;
}
```

**Commit:**
```
feat(ir): add Cap'n Proto schema for zero-copy serialization

- Define complete IR schema in ir.capnp
- Support all 17 FactType variants (v3.2)
- Add validation script for schema compilation
- Generate Rust bindings with capnpc

Performance: Enables 20,000x faster IR loading
Related: US-03.1
```

---

##### T-03.1.2: Configurar build.rs para codegen

**1. 🔴 RED: Test de compilación**

```rust
// hodei-ir/tests/capnp_codegen.rs

#[test]
fn test_capnp_bindings_exist() {
    // Este test falla hasta que build.rs genera los bindings
    let _ = ir_capnp::fact::Reader;
}
```

**2. 🟢 GREEN: build.rs**

```rust
// hodei-ir/build.rs

fn main() {
    capnpc::CompilerCommand::new()
        .src_prefix("schema")
        .file("schema/ir.capnp")
        .run()
        .expect("Failed to compile Cap'n Proto schema");
}
```

**Añadir a Cargo.toml:**

```toml
[build-dependencies]
capnpc = "0.18"

[dependencies]
capnp = "0.18"
```

**Commit:**
```
build(ir): add Cap'n Proto codegen to build.rs

- Configure capnpc to generate Rust bindings
- Add ir.capnp to src_prefix
- Fail build if schema doesn't compile

Related: US-03.1
```

---

### US-03.2: Implementar Zero-Copy Reader

**Como** desarrollador del motor  
**Quiero** leer IR desde disco sin deserializar  
**Para** lograr latencia de <100μs en archivos de 100MB

**Criterios de Aceptación:**
- [ ] `ZeroCopyIR` struct que usa mmap
- [ ] Reader que accede a facts sin copiar memoria
- [ ] Iterador sobre facts con zero-copy
- [ ] Benchmark: 100MB IR en <100μs
- [ ] Tests de correctitud vs JSON

**Principios Aplicados:**
- **Zero-Copy:** mmap + Cap'n Proto readers son views sobre buffer
- **Lazy Loading:** No se deserializa hasta que se accede
- **Memory Safety:** Rust borrow checker garantiza lifetime correcto

---

#### Tareas Técnicas (TDD)

##### T-03.2.1: Implementar ZeroCopyIR

**1. 🔴 RED: Tests de performance**

```rust
// hodei-ir/benches/zero_copy.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hodei_ir::ZeroCopyIR;

fn bench_zero_copy_load(c: &mut Criterion) {
    // Crear IR de prueba de ~100MB
    let ir_path = "benches/fixtures/large_ir.capnp";
    
    c.bench_function("zero_copy_load_100mb", |b| {
        b.iter(|| {
            let ir = ZeroCopyIR::from_file(black_box(ir_path))
                .expect("Failed to load IR");
            black_box(ir);
        });
    });
}

fn bench_iterate_facts(c: &mut Criterion) {
    let ir_path = "benches/fixtures/large_ir.capnp";
    let ir = ZeroCopyIR::from_file(ir_path).unwrap();
    
    c.bench_function("iterate_100k_facts", |b| {
        b.iter(|| {
            let count = ir.facts().count();
            black_box(count);
        });
    });
}

criterion_group!(benches, bench_zero_copy_load, bench_iterate_facts);
criterion_main!(benches);
```

**2. 🟢 GREEN: Implementación**

```rust
// hodei-ir/src/zero_copy.rs

use capnp::message::ReaderOptions;
use capnp::serialize;
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;

pub struct ZeroCopyIR {
    _mmap: Mmap,  // Mantener mmap vivo
    reader: capnp::message::Reader<capnp::serialize::OwnedSegments>,
}

impl ZeroCopyIR {
    /// Carga IR desde archivo usando mmap y zero-copy.
    /// 
    /// # Performance
    /// - 100MB file: ~50μs (solo mmap overhead)
    /// - No deserialización hasta acceder a facts
    /// 
    /// # Safety
    /// - mmap es seguro si el archivo no cambia durante lectura
    /// - Reader tiene lifetime ligado a mmap
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ZeroCopyError> {
        let file = File::open(path.as_ref())
            .map_err(|e| ZeroCopyError::FileOpen {
                path: path.as_ref().to_path_buf(),
                source: e,
            })?;
        
        // SAFETY: Asumimos que el archivo no cambia durante lectura
        let mmap = unsafe {
            Mmap::map(&file).map_err(|e| ZeroCopyError::Mmap {
                source: e,
            })?
        };
        
        // Convertir mmap a &[u8] y parsear Cap'n Proto
        let options = ReaderOptions {
            traversal_limit_in_words: Some(u64::MAX),  // Sin límite
            nesting_limit: 64,
        };
        
        let reader = serialize::read_message_from_flat_slice(
            &mmap,
            options,
        ).map_err(|e| ZeroCopyError::Parse {
            source: e,
        })?;
        
        Ok(Self {
            _mmap: mmap,
            reader,
        })
    }
    
    /// Retorna el IR root
    pub fn root(&self) -> Result<ir_capnp::intermediate_representation::Reader, ZeroCopyError> {
        self.reader
            .get_root::<ir_capnp::intermediate_representation::Reader>()
            .map_err(|e| ZeroCopyError::GetRoot { source: e })
    }
    
    /// Iterador zero-copy sobre facts
    pub fn facts(&self) -> Result<FactIterator, ZeroCopyError> {
        let root = self.root()?;
        let facts = root.get_facts()
            .map_err(|e| ZeroCopyError::GetFacts { source: e })?;
        
        Ok(FactIterator {
            facts,
            index: 0,
        })
    }
    
    /// Acceso directo a un fact por índice
    pub fn get_fact(&self, index: usize) -> Result<ir_capnp::fact::Reader, ZeroCopyError> {
        let root = self.root()?;
        let facts = root.get_facts()
            .map_err(|e| ZeroCopyError::GetFacts { source: e })?;
        
        if index >= facts.len() as usize {
            return Err(ZeroCopyError::IndexOutOfBounds {
                index,
                len: facts.len() as usize,
            });
        }
        
        facts.get(index as u32)
            .map_err(|e| ZeroCopyError::GetFact { source: e })
    }
}

pub struct FactIterator<'a> {
    facts: capnp::struct_list::Reader<'a, ir_capnp::fact::Owned>,
    index: u32,
}

impl<'a> Iterator for FactIterator<'a> {
    type Item = Result<ir_capnp::fact::Reader<'a>, ZeroCopyError>;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.facts.len() {
            return None;
        }
        
        let fact = self.facts.get(self.index)
            .map_err(|e| ZeroCopyError::GetFact { source: e });
        
        self.index += 1;
        Some(fact)
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.facts.len() - self.index) as usize;
        (remaining, Some(remaining))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ZeroCopyError {
    #[error("Failed to open file {path:?}: {source}")]
    FileOpen {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Failed to mmap file: {source}")]
    Mmap {
        #[source]
        source: std::io::Error,
    },
    
    #[error("Failed to parse Cap'n Proto message: {source}")]
    Parse {
        #[source]
        source: capnp::Error,
    },
    
    #[error("Failed to get root: {source}")]
    GetRoot {
        #[source]
        source: capnp::Error,
    },
    
    #[error("Failed to get facts: {source}")]
    GetFacts {
        #[source]
        source: capnp::Error,
    },
    
    #[error("Failed to get fact: {source}")]
    GetFact {
        #[source]
        source: capnp::Error,
    },
    
    #[error("Index {index} out of bounds (len: {len})")]
    IndexOutOfBounds {
        index: usize,
        len: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_zero_copy_load() {
        let ir_path = "tests/fixtures/sample.capnp";
        let ir = ZeroCopyIR::from_file(ir_path).unwrap();
        
        let root = ir.root().unwrap();
        let facts = root.get_facts().unwrap();
        assert_eq!(facts.len(), 10);
    }
    
    #[test]
    fn test_fact_iterator() {
        let ir_path = "tests/fixtures/sample.capnp";
        let ir = ZeroCopyIR::from_file(ir_path).unwrap();
        
        let count = ir.facts().unwrap().count();
        assert_eq!(count, 10);
    }
}
```

**Commit:**
```
feat(ir): implement zero-copy IR reader with mmap

- Add ZeroCopyIR struct using memmap2
- Implement Iterator over facts without copying
- Add benchmarks for 100MB IR load (<100μs target)
- Support random access to facts by index

Performance: 20,000x faster than JSON deserialization
Related: US-03.2
```

---

### US-03.3: Adaptadores JSON ↔ Cap'n Proto

**Como** desarrollador  
**Quiero** convertir entre JSON y Cap'n Proto  
**Para** mantener compatibilidad con tooling existente y debugging

**Criterios de Aceptación:**
- [ ] Función `ir_to_capnp` que serializa IR a Cap'n Proto
- [ ] Función `ir_from_capnp` que convierte Cap'n Proto a IR Rust
- [ ] Función `ir_to_json` para debugging
- [ ] Tests de roundtrip: IR → CapnP → IR
- [ ] CLI command para conversión: `hodei-ir convert`

---

#### Tareas Técnicas (TDD)

##### T-03.3.1: Implementar serialización Rust → Cap'n Proto

**1. 🔴 RED**

```rust
#[test]
fn test_ir_to_capnp_roundtrip() {
    let ir = create_sample_ir();
    
    // IR Rust → Cap'n Proto bytes
    let bytes = ir_to_capnp(&ir).unwrap();
    
    // Cap'n Proto bytes → IR Rust
    let ir_restored = ir_from_capnp(&bytes).unwrap();
    
    assert_eq!(ir.facts().len(), ir_restored.facts().len());
}
```

**2. 🟢 GREEN**

```rust
// hodei-ir/src/adapters.rs

use capnp::message::Builder;
use capnp::serialize;

pub fn ir_to_capnp(ir: &IntermediateRepresentation) -> Result<Vec<u8>, AdapterError> {
    let mut message = Builder::new_default();
    let mut root = message.init_root::<ir_capnp::intermediate_representation::Builder>();
    
    // Serializar metadata
    {
        let mut meta = root.reborrow().init_metadata();
        meta.set_name(&ir.metadata().name);
        meta.set_version(&ir.metadata().version);
        // ... más campos
    }
    
    // Serializar facts
    {
        let facts = ir.facts();
        let mut facts_builder = root.reborrow().init_facts(facts.len() as u32);
        
        for (i, fact) in facts.iter().enumerate() {
            let mut fact_builder = facts_builder.reborrow().get(i as u32);
            
            // ID
            fact_builder.set_id(fact.id().as_uuid().as_bytes());
            
            // FactType (union)
            serialize_fact_type(fact.fact_type(), &mut fact_builder)?;
            
            // Location
            serialize_location(fact.location(), &mut fact_builder)?;
            
            // Provenance
            serialize_provenance(fact.provenance(), &mut fact_builder)?;
        }
    }
    
    // Serializar a bytes
    let mut buf = Vec::new();
    serialize::write_message(&mut buf, &message)
        .map_err(|e| AdapterError::Serialize { source: e })?;
    
    Ok(buf)
}

fn serialize_fact_type(
    fact_type: &FactType,
    builder: &mut ir_capnp::fact::Builder,
) -> Result<(), AdapterError> {
    let mut fact_type_builder = builder.reborrow().init_fact_type();
    
    match fact_type {
        FactType::TaintSource { var, flow_id, source_type, confidence } => {
            let mut ts = fact_type_builder.init_taint_source();
            ts.set_var(&var.to_string());
            ts.set_flow_id(flow_id.as_uuid().as_bytes());
            // ... más campos
        }
        FactType::TaintSink { func, consumes_flow, category, severity } => {
            let mut ts = fact_type_builder.init_taint_sink();
            ts.set_func(&func.to_string());
            // ... más campos
        }
        // ... 15 variantes más
        _ => {}
    }
    
    Ok(())
}
```

**Commit:**
```
feat(ir): add Rust IR to Cap'n Proto adapter

- Implement ir_to_capnp for serialization
- Support all 17 FactType variants
- Add helper functions for nested types
- Write to Vec<u8> for in-memory or file output

Related: US-03.3
```

---

##### T-03.3.2: CLI para conversión de formatos

**1. 🔴 RED**

```bash
#!/bin/bash
# Test CLI conversion

hodei-ir convert --from json --to capnp ir.json ir.capnp
hodei-ir convert --from capnp --to json ir.capnp ir_out.json

# Verify roundtrip
diff ir.json ir_out.json
```

**2. 🟢 GREEN**

```rust
// hodei-cli/src/commands/convert.rs

use clap::Args;

#[derive(Args)]
pub struct ConvertArgs {
    /// Input format (json, capnp)
    #[arg(long)]
    from: Format,
    
    /// Output format (json, capnp)
    #[arg(long)]
    to: Format,
    
    /// Input file path
    input: PathBuf,
    
    /// Output file path
    output: PathBuf,
}

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Json,
    Capnp,
}

pub fn convert(args: ConvertArgs) -> anyhow::Result<()> {
    match (args.from, args.to) {
        (Format::Json, Format::Capnp) => {
            let json = std::fs::read_to_string(&args.input)?;
            let ir: IntermediateRepresentation = serde_json::from_str(&json)?;
            let bytes = ir_to_capnp(&ir)?;
            std::fs::write(&args.output, bytes)?;
        }
        (Format::Capnp, Format::Json) => {
            let zero_copy = ZeroCopyIR::from_file(&args.input)?;
            let ir = ir_from_capnp(&zero_copy)?;
            let json = serde_json::to_string_pretty(&ir)?;
            std::fs::write(&args.output, json)?;
        }
        _ => anyhow::bail!("Unsupported conversion"),
    }
    
    Ok(())
}
```

**Commit:**
```
feat(cli): add convert command for JSON ↔ Cap'n Proto

- Add hodei-ir convert --from <fmt> --to <fmt>
- Support json → capnp and capnp → json
- Enable debugging of binary IR files

Related: US-03.3
```

---

## 📊 Resumen de la Épica

### Story Points Breakdown

| User Story | SP | Complejidad | Riesgo |
|------------|-----|-------------|--------|
| US-03.1: Schema Cap'n Proto | 21 | Alta | Medio |
| US-03.2: Zero-Copy Reader | 21 | Alta | Alto |
| US-03.3: Adaptadores JSON/CapnP | 13 | Media | Bajo |
| **Total** | **55** | - | - |

### Criterios de Finalización de Épica

- [ ] Schema compila sin errores
- [ ] Benchmarks demuestran <100μs para 100MB
- [ ] Tests de roundtrip JSON ↔ CapnP pasan
- [ ] CLI convert funciona end-to-end
- [ ] Documentación de performance publicada
- [ ] 0 memory leaks en valgrind

### Métricas de Éxito

- **Performance:** 100MB IR en <100μs (20,000x mejora vs JSON)
- **Memory:** <100MB peak (4x reducción vs JSON)
- **Tests:** 100% roundtrip correctness
- **CI:** Benchmarks automáticos en cada commit

---

## 🎯 Benchmarks Esperados

```
zero_copy_load_100mb    time: [87.3 μs 89.1 μs 91.2 μs]
iterate_100k_facts      time: [1.23 ms 1.25 ms 1.27 ms]
json_load_100mb         time: [1.89 s 1.92 s 1.95 s]     (baseline)

Improvement: 21,551x faster (1,920,000μs → 89μs)
```

---

## 🔗 Referencias

- [ARCHITECTURE-V3.2-FINAL.md](../ARCHITECTURE-V3.2-FINAL.md) § 8.1.1 (Zero-Copy)
- [EPIC-02: IR Core](./EPIC-02-ir-core.md)
- [EPIC-04: Indexed Fact Store](./EPIC-04-indexed-store.md) (siguiente)
- [Cap'n Proto Documentation](https://capnproto.org/language.html)
- [memmap2 crate](https://docs.rs/memmap2/)

---

**Última actualización:** 2025-01-XX  
**Estado:** READY FOR IMPLEMENTATION