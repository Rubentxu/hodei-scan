# EPIC-11: IR Schema Evolution - Custom FactTypes & Plugin Registry

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: hodei-scan v3.3  
**Dependencias**: EPIC-06 (RuleEngine), EPIC-10 (Extractor Ecosystem)  
**Owner**: Core Platform Team  
**Prioridad**: Critical Path

---

## 1. Resumen Ejecutivo

Evolucionar el esquema del IR para soportar **Custom FactTypes** sin recompilación del core, creando un sistema híbrido que combine tipos core optimizados con extensibilidad infinita para plugins de terceros.

### Objetivo de Negocio
Permitir que desarrolladores de plugins definan sus propios tipos de hechos (ej. `InsecureS3Bucket` para Terraform, `KubernetesMisconfiguration` para K8s) sin solicitar permisos al equipo de hodei-scan, fomentando la innovación y adopción del ecosistema.

### Métricas de Éxito
- **Extensibilidad**: 100+ Custom FactTypes sin recompilación
- **Performance**: <5% overhead vs tipos nativos para Custom types
- **Validación**: 100% de Custom types validados contra schema
- **Backwards Compatibility**: Todos los IRs v3.2 siguen funcionando

---

## 2. Contexto Técnico

### 2.1. Problema Actual
El `enum FactType` está cerrado:
```rust
pub enum FactType {
    TaintSource { /* ... */ },
    Vulnerability { /* ... */ },
    Function { /* ... */ },
    // ❌ No se puede extender sin modificar core
}
```

### 2.2. Solución: Esquema Híbrido

```rust
pub enum FactType {
    // ✅ Tipos core optimizados
    TaintSource { /* ... */ },
    Vulnerability { /* ... */ },
    
    // ✅ Tipos custom extensibles
    Custom {
        discriminant: String,        // ej: "terraform::aws::insecure_s3_bucket"
        data: HashMap<String, FactValue>,  // ej: {"acl": "public-read", "public_access": true}
    },
}
```

---

## 3. Arquitectura Detallada

### 3.1. FactValue System

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FactValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<FactValue>),
    Object(HashMap<String, FactValue>),
}

impl FactValue {
    pub fn get_type(&self) -> FactValueType {
        match self {
            FactValue::String(_) => FactValueType::String,
            FactValue::Number(_) => FactValueType::Number,
            FactValue::Boolean(_) => FactValueType::Boolean,
            FactValue::Array(_) => FactValueType::Array,
            FactValue::Object(_) => FactValueType::Object,
        }
    }
}
```

### 3.2. Plugin Schema Registry

```rust
pub struct PluginSchemaRegistry {
    schemas: HashMap<String, CustomFactSchema>,
}

pub struct CustomFactSchema {
    pub name: String,
    pub version: String,
    pub fields: HashMap<String, FactValueType>,
    pub required_fields: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl PluginSchemaRegistry {
    /// Register a new custom fact type schema
    pub fn register_schema(&mut self, schema: CustomFactSchema) -> Result<(), SchemaError> {
        // Validate schema
        // Check for conflicts
        // Store in registry
        Ok(())
    }
    
    /// Validate a Custom fact against its schema
    pub fn validate_custom_fact(&self, fact: &FactType::Custom) -> Result<(), ValidationError> {
        let schema = self.schemas.get(&fact.discriminant)
            .ok_or(ValidationError::UnknownFactType)?;
        
        // Check required fields
        // Validate types
        // Custom validation rules
        Ok(())
    }
}
```

### 3.3. IR Validation Pipeline

```
Incoming IR
    ↓
Schema Version Check
    ↓
Core FactType Validation
    ↓
Custom FactType Validation (if PluginRegistry loaded)
    ↓
Cross-Reference Validation (FlowIds, etc.)
    ↓
✅ Validated IR
```

---

## 4. User Stories

### US-11.01: Implementar Custom FactType en IR Schema

**Como:** Desarrollador Plugin  
**Quiero:** Crear tipos de hechos personalizados en IR  
**Para:** Representar conceptos específicos de mi herramienta

**Criterios de Aceptación:**
- [ ] IR schema incluye variante `Custom`
- [ ] `FactValue` enum soporta tipos básicos y estructuras
- [ ] Serialización/deserialización Cap'n Proto
- [ ] Equipos y comparaciones funcionan
- [ ] Performance profiling completado

**TDD - Red:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn create_custom_fact() {
        let custom = FactType::Custom {
            discriminant: "terraform::aws::insecure_s3_bucket".to_string(),
            data: hashmap! {
                "bucket_name".to_string() => FactValue::String("my-bucket".to_string()),
                "acl".to_string() => FactValue::String("public-read".to_string()),
                "public_access".to_string() => FactValue::Boolean(true),
            },
        };
        
        assert!(custom.discriminant().unwrap().contains("terraform"));
        assert_eq!(custom.get_field("acl").unwrap(), "public-read");
    }
}
```

**TDD - Green:**
```rust
pub enum FactType {
    // ... existing variants
    
    Custom {
        discriminant: String,
        data: HashMap<String, FactValue>,
    },
}

impl FactType {
    pub fn discriminant(&self) -> Option<&String> {
        match self {
            FactType::Custom { discriminant, .. } => Some(discriminant),
            _ => None,
        }
    }
    
    pub fn get_field(&self, key: &str) -> Option<&FactValue> {
        match self {
            FactType::Custom { data, .. } => data.get(key),
            _ => None,
        }
    }
}
```

**Conventional Commit:**
`feat(ir): add Custom FactType variant with FactValue system`

---

### US-11.02: Implementar PluginSchemaRegistry

**Como:** Desarrollador Core  
**Quiero:** Registrar y validar schemas de Custom types  
**Para:** Asegurar consistencia y prevenir errores

**Criterios de Aceptación:**
- [ ] Registry guarda schemas por discriminant
- [ ] Validación de required fields
- [ ] Type checking para valores
- [ ] Error messages descriptivas
- [ ] Thread-safe operations

**TDD - Red:**
```rust
#[test]
fn register_and_validate_schema() {
    let mut registry = PluginSchemaRegistry::new();
    
    let schema = CustomFactSchema {
        name: "insecure_s3_bucket".to_string(),
        version: "1.0.0".to_string(),
        fields: hashmap! {
            "bucket_name".to_string() => FactValueType::String,
            "public_access".to_string() => FactValueType::Boolean,
        },
        required_fields: vec!["bucket_name".to_string()],
        metadata: hashmap! {},
    };
    
    registry.register_schema(schema).unwrap();
    
    // Validate custom fact
    let custom_fact = create_test_custom_fact();
    assert!(registry.validate_custom_fact(&custom_fact).is_ok());
}
```

**TDD - Green:**
```rust
pub struct PluginSchemaRegistry {
    schemas: Arc<RwLock<HashMap<String, CustomFactSchema>>>,
}

impl PluginSchemaRegistry {
    pub fn new() -> Self {
        Self {
            schemas: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn register_schema(&self, schema: CustomFactSchema) -> Result<(), SchemaError> {
        // Check for version conflicts
        // Validate schema structure
        let mut schemas = self.schemas.write().unwrap();
        schemas.insert(schema.name.clone(), schema);
        Ok(())
    }
    
    pub fn validate_custom_fact(&self, fact: &FactType::Custom) -> Result<(), ValidationError> {
        let schemas = self.schemas.read().unwrap();
        let schema = schemas.get(&fact.discriminant)
            .ok_or(ValidationError::UnknownFactType(fact.discriminant.clone()))?;
        
        // Validate required fields
        for required in &schema.required_fields {
            if !fact.data.contains_key(required) {
                return Err(ValidationError::MissingRequiredField(required.clone()));
            }
        }
        
        // Validate types
        for (key, value) in &fact.data {
            if let Some(&expected_type) = schema.fields.get(key) {
                if value.get_type() != expected_type {
                    return Err(ValidationError::TypeMismatch {
                        field: key.clone(),
                        expected: expected_type,
                        actual: value.get_type(),
                    });
                }
            }
        }
        
        Ok(())
    }
}
```

**Conventional Commit:**
`feat(ir): implement PluginSchemaRegistry for Custom FactType validation`

---

### US-11.03: Serialización Cap'n Proto para Custom Types

**Como:** Desarrollador Core  
**Quiero:** Serializar Custom FactTypes con Cap'n Proto  
**Para:** Interoperabilidad entre extractores y core

**Criterios de Aceptación:**
- [ ] Cap'n Proto schema incluye Custom type
- [ ] Serialización bidireccional (Fact → bytes → Fact)
- [ ] Preservar type information
- [ ] Round-trip tests (1000+ iterations)
- [ ] Performance <2x de tipos nativos

**Cap'n Proto Schema:**
```capnp
# ir_schema.capnp

struct Fact {
    id @0 :UInt64;
    factType @1 :FactType;
    location @2 :SourceLocation;
    provenance @3 :Provenance;
}

union FactType {
    taintSource @0 :TaintSource;
    vulnerability @1 :Vulnerability;
    custom @2 :CustomFact;
}

struct CustomFact {
    discriminant @0 :Text;
    data @1 :Data;  # JSON-serialized HashMap<String, FactValue>
}
```

**TDD - Red:**
```rust
#[test]
fn capnp_roundtrip_custom_fact() {
    let original = create_custom_fact();
    let bytes = serialize_capnp(&original).unwrap();
    let deserialized: Fact = deserialize_capnp(&bytes).unwrap();
    
    assert_eq!(original.id, deserialized.id);
    match &deserialized.fact_type {
        FactType::Custom { discriminant, data } => {
            assert_eq!(discriminant, "test::custom");
            assert_eq!(data.get("field"), Some(&FactValue::String("value".to_string())));
        }
        _ => panic!("Expected Custom fact type"),
    }
}
```

**TDD - Green:**
```rust
pub fn serialize_capnp(fact: &Fact) -> Result<Vec<u8>, CapnpError> {
    let mut message = Builder::new_default();
    {
        let mut fact_builder = message.init_root::<fact_capnp::Fact::Builder>();
        fact_builder.set_id(fact.id.0);
        
        match &fact.fact_type {
            FactType::Custom { discriminant, data } => {
                let mut custom = fact_builder.init_custom();
                custom.set_discriminant(discriminant);
                
                // Serialize data HashMap to JSON
                let json = serde_json::to_string(data)?;
                custom.set_data(json.into());
            }
            // ... other variants
        }
    }
    
    let mut buf = Vec::new();
    write_message(&mut buf, &message)?;
    Ok(buf)
}
```

**Conventional Commit:**
`feat(serialization): add Cap'n Proto support for Custom FactTypes`

---

### US-11.04: RuleEngine Integration con Custom Types

**Como:** Desarrollador Reglas  
**Quiero:** Escribir reglas que matcheen Custom FactTypes  
**Para:** Analizar findings de plugins externos

**Criterios de Aceptación:**
- [ ] Pattern matching reconoce Custom types
- [ ] DSL permite referenciar campos Custom
- [ ] Template interpolation funciona con Custom data
- [ ] WHERE clauses acceden a campos Custom
- [ ] Performance: 1000 Custom facts <100ms

**DSL Examples:**
```hodei
rule "Insecure S3 Bucket" {
    match {
        s3: Custom("terraform::aws::insecure_s3_bucket")
        where s3.public_access == true
    }
    emit Finding {
        message: "S3 bucket {s3.bucket_name} has public access: {s3.acl}"
    }
}
```

**TDD - Red:**
```rust
#[test]
fn pattern_match_custom_fact() {
    let custom_fact = create_custom_fact();
    let store = IndexedFactStore::new(vec![custom_fact]);
    
    let pattern = Pattern {
        binding: "s3".to_string(),
        fact_type: "Custom".to_string(),
        conditions: vec![Condition {
            path: "data.public_access".to_string(),
            op: ComparisonOp::Eq,
            value: Literal::Boolean(true),
        }],
    };
    
    let matcher = PatternMatcher::new(store);
    let results = matcher.match_patterns(&[pattern]).unwrap();
    
    assert_eq!(results.len(), 1);
}
```

**TDD - Green:**
```rust
impl PatternMatcher {
    fn matches_custom_pattern(&self, pattern: &Pattern, fact: &Fact) -> bool {
        if let FactType::Custom { discriminant, data } = &fact.fact_type {
            // Check discriminant
            if !discriminant.starts_with(&pattern.fact_type) {
                return false;
            }
            
            // Check conditions
            for condition in &pattern.conditions {
                let value = self.resolve_custom_field_path(data, &condition.path);
                if !self.evaluate_condition(value, &condition.op, &condition.value) {
                    return false;
                }
            }
            
            true
        } else {
            false
        }
    }
}
```

**Conventional Commit:**
`feat(rule-engine): add Custom FactType pattern matching support`

---

### US-11.05: Migration Tool para IR v3.2

**Como:** Usuario  
**Quiero:** Migrar IRs existentes a v3.3  
**Para:** Backwards compatibility

**Criterios de Aceptación:**
- [ ] Detecta version del IR (Cap'n Proto)
- [ ] Migración automática v3.2 → v3.3
- [ ] Preserve todos los facts existentes
- [ ] Validación post-migración
- [ ] Rollback capability

**TDD - Red:**
```rust
#[test]
fn migrate_ir_v32_to_v33() {
    let ir_v32 = create_ir_v32_with_core_facts();
    let ir_v33 = migrate_ir_version(&ir_v32).unwrap();
    
    assert_eq!(ir_v33.schema_version, SchemaVersion::V33);
    assert_eq!(ir_v33.facts.len(), ir_v32.facts.len());
    
    // All original facts preserved
    for fact in &ir_v32.facts {
        assert!(ir_v33.facts.contains(fact));
    }
}
```

**TDD - Green:**
```rust
pub fn migrate_ir_version(ir: &IntermediateRepresentation) -> Result<IntermediateRepresentation, MigrationError> {
    match ir.schema_version {
        SchemaVersion::V32 => {
            // Core facts remain the same, no changes needed
            // Just bump version number
            Ok(IntermediateRepresentation {
                schema_version: SchemaVersion::V33,
                ..ir.clone()
            })
        }
        SchemaVersion::V33 => Ok(ir.clone()),
        _ => Err(MigrationError::UnsupportedVersion(ir.schema_version)),
    }
}
```

**Conventional Commit:**
`feat(migration): add IR v3.2 to v3.3 migration tool`

---

## 5. Testing Strategy

### 5.1. Unit Tests
- Custom FactType creation and manipulation
- PluginSchemaRegistry validation logic
- Cap'n Proto serialization round-trips
- Performance benchmarks

### 5.2. Integration Tests
- Custom FactType en RuleEngine workflows
- Cross-extractor communication with Custom types
- Multi-version IR compatibility

### 5.3. Property-Based Tests
- Arbitrary Custom FactType generation
- Round-trip serialization invariants
- Schema validation properties

---

## 6. Benchmarks

```rust
// benches/custom_fact_bench.rs
fn bench_custom_fact_creation(c: &mut Criterion) {
    c.bench_function("custom_fact_creation", |b| {
        b.iter(|| {
            FactType::Custom {
                discriminant: "test::fact".to_string(),
                data: generate_test_data(100),
            }
        });
    });
}

fn bench_serialization(c: &mut Criterion) {
    let fact = create_large_custom_fact();
    c.bench_function("custom_fact_serialization", |b| {
        b.iter(|| serialize_capnp(&fact).unwrap());
    });
}
```

---

## 7. Riesgos y Mitigaciones

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| Schema evolution breaking changes | Alto | Alto | Versioned schemas + migration tools |
| Performance regression with Custom types | Medio | Medio | Optimized serialization + benchmarks |
| Type safety erosion | Alto | Medio | Strong typing + validation rules |
| Plugin registry corruption | Alto | Bajo | Thread-safe ops + atomic updates |

---

## 8. Definition of Done

- [ ] Custom FactType implemented in IR
- [ ] PluginSchemaRegistry with validation
- [ ] Cap'n Proto serialization working
- [ ] RuleEngine Custom type support
- [ ] Migration tool for v3.2 IRs
- [ ] Benchmarks: Custom type overhead <5%
- [ ] Tests: >90% coverage + property-based
- [ ] Documentation: Schema evolution guide
- [ ] Performance regression tests green

---

**Estimación Total**: 3 Sprints (6 semanas)  
**Commit Messages**:  
- `feat(ir): add Custom FactType variant with FactValue system`  
- `feat(ir): implement PluginSchemaRegistry for validation`  
- `feat(serialization): add Cap'n Proto support for Custom types`  
- `feat(rule-engine): add Custom FactType pattern matching`  
- `feat(migration): add IR v3.2 to v3.3 migration tool`

---

**Referencias Técnicas:**
- Cap'n Proto: https://capnproto.org/
- Serde: https://serde.rs/
- Property-based testing: https://proptest-rs.github.io/proptest/
