# EPIC-02: IR Core Implementation

**Fase:** Foundation  
**Story Points:** 89  
**Prioridad:** CRITICAL  
**Dependencias:** EPIC-01  
**Owner:** Core Team

---

## 📋 Contexto

Implementar el núcleo del Intermediate Representation (IR) con tipos seguros, validación robusta y soporte para hechos atómicos multi-dominio según la especificación v3.2.

**IMPORTANTE:** Esta épica implementa **SOLO hechos atómicos observables**. Los meta-hechos (VulnerableUncovered, SecurityTechnicalDebt, QualitySecurityCorrelation) fueron eliminados en v3.2 según ADR-001.

---

## 🎯 Objetivos Específicos

1. Implementar newtypes con validación en tiempo de compilación
2. Definir FactType enum con 17 variantes atómicas
3. Crear sistema de proveniencia para rastreo de hechos
4. Implementar IRValidator con integridad referencial
5. Zero-copy preparation (Cap'n Proto schema preparado)

---

## 🏗️ Arquitectura Hexagonal

```
┌─────────────────────────────────────────────────────────────────┐
│ DOMAIN LAYER (hodei-ir)                                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Newtypes (Confidence, ProjectPath, LineNumber, FlowId)        │
│      ↓                                                          │
│  Core Types (Fact, FactId, Provenance, SourceLocation)         │
│      ↓                                                          │
│  FactType enum (17 variantes atómicas)                         │
│      ↓                                                          │
│  IntermediateRepresentation (container)                        │
│      ↓                                                          │
│  IRValidator (integrity checks)                                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Principios:**
- **Make Illegal States Unrepresentable:** Newtypes con validación
- **Parse, Don't Validate:** Constructores que garantizan corrección
- **Domain-Driven Design:** Tipos que reflejan el negocio

---

## 📖 User Stories

### US-02.1: Newtypes Fundamentales

**Como** desarrollador del motor  
**Quiero** tipos opacos para valores primitivos críticos  
**Para** prevenir errores de tipo en tiempo de compilación y eliminar CoM

**Criterios de Aceptación:**
- [ ] `Confidence` (0.0..=1.0) con validación en constructor
- [ ] `LineNumber` (u32, ≥1) rechaza línea 0
- [ ] `ColumnNumber` (u32, ≥1) rechaza columna 0
- [ ] `ProjectPath` canonicaliza y valida confinamiento al project root
- [ ] `FlowId` opaco con factory `new_scoped` y `new_uuid`
- [ ] Cada newtype implementa: `Display`, `Debug`, `Serialize`, `Deserialize`, `Eq`, `Hash`
- [ ] Tests de property-based para cada newtype (proptest)
- [ ] Tests de fuzzing para `ProjectPath` (prevención path traversal)

**Principios Aplicados:**
- **Connascence of Position → Connascence of Type:** De tuplas primitivas a tipos con nombre
- **Type Safety:** Rust type system previene mezclar LineNumber con ColumnNumber
- **Security by Design:** ProjectPath valida en construcción (no en uso)

**Dependencias:**
- EPIC-01 (workspace configurado)
- ADR-001 (atomic facts only)

---

#### Tareas Técnicas (TDD)

##### T-02.1.1: Implementar `Confidence` newtype

**1. 🔴 RED: Tests que fallan**

```rust
// hodei-ir/src/types/confidence.rs
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    #[test]
    fn test_confidence_valid_range() {
        assert!(Confidence::new(0.0).is_ok());
        assert!(Confidence::new(0.5).is_ok());
        assert!(Confidence::new(1.0).is_ok());
    }
    
    #[test]
    fn test_confidence_invalid_range() {
        assert!(Confidence::new(-0.1).is_err());
        assert!(Confidence::new(1.1).is_err());
        assert!(Confidence::new(f64::NAN).is_err());
        assert!(Confidence::new(f64::INFINITY).is_err());
    }
    
    #[test]
    fn test_confidence_constants() {
        assert_eq!(Confidence::HIGH.get(), 0.9);
        assert_eq!(Confidence::MEDIUM.get(), 0.6);
        assert_eq!(Confidence::LOW.get(), 0.3);
    }
    
    proptest! {
        #[test]
        fn prop_confidence_roundtrip(value in 0.0_f64..=1.0) {
            let conf = Confidence::new(value).unwrap();
            assert_eq!(conf.get(), value);
        }
        
        #[test]
        fn prop_confidence_rejects_invalid(value in prop::num::f64::ANY) {
            prop_assume!(value < 0.0 || value > 1.0 || !value.is_finite());
            assert!(Confidence::new(value).is_err());
        }
        
        #[test]
        fn prop_confidence_serialization(value in 0.0_f64..=1.0) {
            let conf = Confidence::new(value).unwrap();
            let json = serde_json::to_string(&conf).unwrap();
            let deserialized: Confidence = serde_json::from_str(&json).unwrap();
            assert_eq!(conf, deserialized);
        }
    }
}
```

**2. 🟢 GREEN: Implementación mínima**

```rust
// hodei-ir/src/types/confidence.rs
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "f64")]
pub struct Confidence(f64);

#[derive(Debug, thiserror::Error)]
pub enum ConfidenceError {
    #[error("Confidence value {0} is out of range [0.0, 1.0]")]
    OutOfRange(f64),
    
    #[error("Confidence value is not finite (NaN or Infinity)")]
    NotFinite,
}

impl Confidence {
    /// Creates a new Confidence value.
    /// 
    /// # Errors
    /// Returns `ConfidenceError::OutOfRange` if value is not in [0.0, 1.0]
    /// Returns `ConfidenceError::NotFinite` if value is NaN or Infinity
    pub fn new(value: f64) -> Result<Self, ConfidenceError> {
        if !value.is_finite() {
            return Err(ConfidenceError::NotFinite);
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(ConfidenceError::OutOfRange(value));
        }
        Ok(Self(value))
    }
    
    /// Returns the confidence value as f64
    pub fn get(&self) -> f64 {
        self.0
    }
    
    /// High confidence (0.9)
    pub const HIGH: Self = Self(0.9);
    
    /// Medium confidence (0.6)
    pub const MEDIUM: Self = Self(0.6);
    
    /// Low confidence (0.3)
    pub const LOW: Self = Self(0.3);
}

impl TryFrom<f64> for Confidence {
    type Error = ConfidenceError;
    
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl Eq for Confidence {}

impl std::hash::Hash for Confidence {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}
```

**3. 🔵 REFACTOR: Optimizaciones y documentación**

```rust
// Añadir conversión segura
impl From<Confidence> for f64 {
    fn from(conf: Confidence) -> f64 {
        conf.0
    }
}

// Añadir comparaciones útiles
impl Confidence {
    pub fn is_high(&self) -> bool {
        self.0 >= 0.8
    }
    
    pub fn is_low(&self) -> bool {
        self.0 < 0.5
    }
}
```

**Commit:**
```
feat(ir): add Confidence newtype with range validation

- Implement Confidence(f64) with 0.0..=1.0 range
- Add constants HIGH, MEDIUM, LOW
- Add property-based tests with proptest
- Reject NaN and Infinity values
- Implement Display, Eq, Hash traits

Security: Prevents invalid confidence values at type level
Related: US-02.1
```

---

##### T-02.1.2: Implementar `ProjectPath` con canonicalización

**1. 🔴 RED: Tests de seguridad**

```rust
// hodei-ir/src/types/project_path.rs
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use proptest::prelude::*;
    
    #[test]
    fn test_project_path_rejects_traversal() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        
        // Intento de path traversal
        let result = ProjectPath::new("../../../etc/passwd", root);
        assert!(matches!(result, Err(PathError::OutsideProject { .. })));
    }
    
    #[test]
    fn test_project_path_rejects_absolute_outside() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        
        let result = ProjectPath::new("/etc/passwd", root);
        assert!(matches!(result, Err(PathError::OutsideProject { .. })));
    }
    
    #[test]
    fn test_project_path_canonicalizes() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        
        let file = root.join("src/main.rs");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "").unwrap();
        
        let path = ProjectPath::new("src/../src/main.rs", root).unwrap();
        assert_eq!(path.relative_to(root), PathBuf::from("src/main.rs"));
    }
    
    #[test]
    fn test_project_path_display() {
        let tmp = TempDir::new().unwrap();
        let root = tmp.path();
        
        let file = root.join("src/lib.rs");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, "").unwrap();
        
        let path = ProjectPath::new("src/lib.rs", root).unwrap();
        let display = format!("{}", path);
        assert!(display.ends_with("src/lib.rs"));
    }
    
    proptest! {
        #[test]
        fn prop_project_path_always_inside_root(
            subpath in "[a-z]{1,10}(/[a-z]{1,10}){0,5}"
        ) {
            let tmp = TempDir::new().unwrap();
            let root = tmp.path();
            
            // Crear archivo para que canonicalize funcione
            let full_path = root.join(&subpath);
            if let Some(parent) = full_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&full_path, "");
            
            if let Ok(path) = ProjectPath::new(&subpath, root) {
                assert!(path.as_path().starts_with(root));
            }
        }
    }
}
```

**2. 🟢 GREEN: Implementación con validación**

```rust
// hodei-ir/src/types/project_path.rs
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct ProjectPath {
    canonical: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum PathError {
    #[error("Failed to canonicalize path {path:?}: {source}")]
    Canonicalization {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    
    #[error("Path {attempted:?} is outside project root {project_root:?}")]
    OutsideProject {
        attempted: PathBuf,
        project_root: PathBuf,
    },
    
    #[error("Failed to strip prefix from path")]
    StripPrefixFailed,
    
    #[error("Invalid root path: {0}")]
    InvalidRoot(#[source] std::io::Error),
}

impl ProjectPath {
    /// Creates a new ProjectPath, ensuring it's within project_root.
    /// 
    /// # Security
    /// This function canonicalizes the path and validates it's confined
    /// to project_root, preventing path traversal attacks (CWE-22).
    /// 
    /// # Errors
    /// - `PathError::Canonicalization`: Path doesn't exist or can't be accessed
    /// - `PathError::OutsideProject`: Path is outside project root
    /// - `PathError::InvalidRoot`: Project root is invalid
    pub fn new(path: impl AsRef<Path>, project_root: impl AsRef<Path>) -> Result<Self, PathError> {
        let path = path.as_ref();
        let root = project_root.as_ref()
            .canonicalize()
            .map_err(PathError::InvalidRoot)?;
        
        let canonical = if path.is_absolute() {
            path.canonicalize()
                .map_err(|e| PathError::Canonicalization {
                    path: path.to_path_buf(),
                    source: e,
                })?
        } else {
            root.join(path)
                .canonicalize()
                .map_err(|e| PathError::Canonicalization {
                    path: path.to_path_buf(),
                    source: e,
                })?
        };
        
        // Security: Prevent path traversal
        if !canonical.starts_with(&root) {
            return Err(PathError::OutsideProject {
                attempted: canonical,
                project_root: root,
            });
        }
        
        Ok(Self { canonical })
    }
    
    /// Returns the path relative to project root
    pub fn relative_to(&self, root: &Path) -> PathBuf {
        self.canonical
            .strip_prefix(root)
            .unwrap_or(&self.canonical)
            .to_path_buf()
    }
    
    /// Returns the canonical path
    pub fn as_path(&self) -> &Path {
        &self.canonical
    }
    
    /// Returns the path as a string
    pub fn as_str(&self) -> &str {
        self.canonical.to_str().unwrap_or("<invalid utf-8>")
    }
}

impl TryFrom<String> for ProjectPath {
    type Error = PathError;
    
    fn try_from(value: String) -> Result<Self, Self::Error> {
        // For deserialization, we need a root. This is a limitation.
        // In practice, deserialization happens in context where root is known.
        Err(PathError::StripPrefixFailed)
    }
}

impl fmt::Display for ProjectPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.canonical.display())
    }
}
```

**Commit:**
```
feat(ir): add ProjectPath with path traversal protection

- Implement ProjectPath newtype with canonicalization
- Validate all paths are confined to project root
- Add fuzzing tests for path traversal attempts
- Support both absolute and relative paths

Security: Mitigates CWE-22 (Path Traversal)
Related: US-02.1
```

---

##### T-02.1.3: Implementar `LineNumber` y `ColumnNumber`

**1. 🔴 RED: Tests de validación**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_line_number_rejects_zero() {
        assert!(LineNumber::new(0).is_err());
    }
    
    #[test]
    fn test_line_number_accepts_positive() {
        assert!(LineNumber::new(1).is_ok());
        assert!(LineNumber::new(100).is_ok());
        assert!(LineNumber::new(u32::MAX).is_ok());
    }
    
    #[test]
    fn test_column_number_rejects_zero() {
        assert!(ColumnNumber::new(0).is_err());
    }
}
```

**2. 🟢 GREEN: Implementación**

```rust
// hodei-ir/src/types/line_number.rs
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u32")]
pub struct LineNumber(u32);

#[derive(Debug, thiserror::Error)]
pub enum LineNumberError {
    #[error("Line number must be >= 1, got 0")]
    ZeroLine,
}

impl LineNumber {
    pub fn new(line: u32) -> Result<Self, LineNumberError> {
        if line == 0 {
            return Err(LineNumberError::ZeroLine);
        }
        Ok(Self(line))
    }
    
    pub fn get(&self) -> u32 {
        self.0
    }
}

impl TryFrom<u32> for LineNumber {
    type Error = LineNumberError;
    
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Display for LineNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Similar para ColumnNumber
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "u32")]
pub struct ColumnNumber(u32);

impl ColumnNumber {
    pub fn new(col: u32) -> Result<Self, LineNumberError> {
        if col == 0 {
            return Err(LineNumberError::ZeroLine);
        }
        Ok(Self(col))
    }
    
    pub fn get(&self) -> u32 {
        self.0
    }
}
```

**Commit:**
```
feat(ir): add LineNumber and ColumnNumber newtypes

- Implement LineNumber and ColumnNumber with >=1 validation
- Add Ord trait for range operations
- Reject zero values at type level

Related: US-02.1
```

---

##### T-02.1.4: Implementar `FlowId` opaco

**1. 🔴 RED: Tests de determinismo y unicidad**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_flow_id_scoped_deterministic() {
        let id1 = FlowId::new_scoped("taint", "user_input_to_sql");
        let id2 = FlowId::new_scoped("taint", "user_input_to_sql");
        assert_eq!(id1, id2);
    }
    
    #[test]
    fn test_flow_id_scoped_different_names() {
        let id1 = FlowId::new_scoped("taint", "flow_a");
        let id2 = FlowId::new_scoped("taint", "flow_b");
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_flow_id_uuid_unique() {
        let id1 = FlowId::new_uuid();
        let id2 = FlowId::new_uuid();
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_flow_id_serialization() {
        let id = FlowId::new_scoped("test", "flow");
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: FlowId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }
}
```

**2. 🟢 GREEN: Implementación con UUIDv5 y UUIDv4**

```rust
// hodei-ir/src/types/flow_id.rs
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FlowId(Uuid);

impl FlowId {
    /// Creates a deterministic FlowId based on namespace and name.
    /// 
    /// Same namespace+name always produces same FlowId (UUIDv5).
    /// Useful for correlating flows across different runs.
    pub fn new_scoped(namespace: &str, name: &str) -> Self {
        let namespace_uuid = Uuid::new_v5(&Uuid::NAMESPACE_OID, namespace.as_bytes());
        Self(Uuid::new_v5(&namespace_uuid, name.as_bytes()))
    }
    
    /// Creates a unique random FlowId (UUIDv4).
    /// 
    /// Use for ad-hoc flows that don't need determinism.
    pub fn new_uuid() -> Self {
        Self(Uuid::new_v4())
    }
    
    /// Returns the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl From<Uuid> for FlowId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl fmt::Display for FlowId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

**Commit:**
```
feat(ir): add FlowId opaque type for dataflow correlation

- Implement FlowId with scoped (deterministic) and UUID (unique) factories
- Use UUIDv5 for scoped IDs (enables correlation across runs)
- Use UUIDv4 for ad-hoc unique IDs
- Add determinism tests

Related: US-02.1
```

---

### US-02.2: FactType Enum Completo (Solo Hechos Atómicos)

**Como** desarrollador de extractores  
**Quiero** un enum exhaustivo de tipos de hechos atómicos  
**Para** modelar todos los dominios de análisis sin correlaciones pre-computadas

**BREAKING CHANGE:** Esta US implementa solo 17 variantes atómicas. Los meta-hechos fueron eliminados en v3.2 (ver ADR-001).

**Criterios de Aceptación:**
- [ ] Enum `FactType` con 17 variantes (NO incluye VulnerableUncovered, SecurityTechnicalDebt, QualitySecurityCorrelation)
- [ ] Cada variante tiene campos tipados (no tuplas primitivas)
- [ ] Soporta: SAST (TaintSource, TaintSink, Sanitization, UnsafeCall, CryptographicOperation, Vulnerability)
- [ ] Soporta: Quality (Function, Variable, CodeSmell, ComplexityViolation)
- [ ] Soporta: SCA (Dependency, DependencyVulnerability, License)
- [ ] Soporta: Coverage (UncoveredLine, LowTestCoverage, CoverageStats)
- [ ] Serialización JSON roundtrip
- [ ] Tests exhaustivos por variante

**Principios Aplicados:**
- **Atomic Facts Only:** No correlaciones, solo observaciones directas del código
- **Separation of Concerns:** Extractores emiten facts, engine deriva correlaciones
- **Type Safety:** Cada variante tiene tipos específicos del dominio

---

#### Tareas Técnicas (TDD)

##### T-02.2.1: Definir variantes SAST

**1. 🔴 RED: Tests de serialización**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_taint_source_roundtrip() {
        let fact_type = FactType::TaintSource {
            var: VariableName::from("user_input"),
            flow_id: FlowId::new_scoped("taint", "test_flow"),
            source_type: TaintSourceType::HttpRequestParam,
            confidence: Confidence::HIGH,
        };
        
        let json = serde_json::to_string(&fact_type).unwrap();
        let deserialized: FactType = serde_json::from_str(&json).unwrap();
        assert_eq!(fact_type, deserialized);
    }
    
    #[test]
    fn test_taint_sink_roundtrip() {
        let fact_type = FactType::TaintSink {
            func: FunctionName::from("execute_query"),
            consumes_flow: FlowId::new_scoped("taint", "sql_flow"),
            category: SinkCategory::SqlQuery,
            severity: Severity::High,
        };
        
        let json = serde_json::to_string(&fact_type).unwrap();
        let deserialized: FactType = serde_json::from_str(&json).unwrap();
        assert_eq!(fact_type, deserialized);
    }
}
```

**2. 🟢 GREEN: Implementación de variantes SAST**

```rust
// hodei-ir/src/fact_type.rs
use crate::types::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum FactType {
    // ═══════════════════════════════════════════════════════
    // SECURITY ANALYSIS (SAST) - Atomic Facts Only
    // ═══════════════════════════════════════════════════════
    
    /// Fuente de datos no confiables (entrada del usuario)
    TaintSource {
        /// Nombre de la variable/parámetro
        var: VariableName,
        
        /// ID del flujo de taint (determinista para correlación)
        flow_id: FlowId,
        
        /// Tipo de fuente
        source_type: TaintSourceType,
        
        /// Confianza del análisis
        confidence: Confidence,
    },
    
    /// Sumidero de datos (operación peligrosa)
    TaintSink {
        /// Nombre de la función/método
        func: FunctionName,
        
        /// ID del flujo consumido
        consumes_flow: FlowId,
        
        /// Categoría del sink
        category: SinkCategory,
        
        /// Severidad si se alcanza
        severity: Severity,
    },
    
    /// Sanitización de datos
    Sanitization {
        /// Método de sanitización
        method: SanitizationMethod,
        
        /// ID del flujo sanitizado
        sanitizes_flow: FlowId,
        
        /// ¿Es efectiva?
        effective: bool,
        
        /// Confianza
        confidence: Confidence,
    },
    
    /// Llamada a función insegura
    UnsafeCall {
        /// Nombre de la función
        function_name: FunctionName,
        
        /// Razón por la que es insegura
        reason: UnsafeReason,
        
        /// Severidad
        severity: Severity,
    },
    
    /// Operación criptográfica
    CryptographicOperation {
        /// Algoritmo usado
        algorithm: CryptoAlgorithm,
        
        /// Longitud de la clave
        key_length: Option<u32>,
        
        /// ¿Es seguro?
        secure: bool,
        
        /// Recomendación si no es seguro
        recommendation: Option<String>,
    },
    
    /// Vulnerabilidad identificada
    Vulnerability {
        /// CWE ID
        cwe_id: Option<CweId>,
        
        /// Categoría OWASP
        owasp_category: Option<OwaspCategory>,
        
        /// Severidad
        severity: Severity,
        
        /// CVSS score
        cvss_score: Option<f32>,
        
        /// Descripción
        description: String,
        
        /// Confianza
        confidence: Confidence,
    },
    
    // Más variantes en tareas siguientes...
}
```

**Commit:**
```
feat(ir): add SAST fact variants (atomic facts only)

- Add TaintSource, TaintSink, Sanitization
- Add UnsafeCall, CryptographicOperation, Vulnerability
- All variants use typed fields (no primitives)
- Add roundtrip serialization tests

Breaking: No meta-facts per ADR-001
Related: US-02.2
```

---

##### T-02.2.2: Añadir variantes Quality, SCA, Coverage

**Tests y implementación similar para:**
- Quality: Function, Variable, CodeSmell, ComplexityViolation
- SCA: Dependency, DependencyVulnerability, License
- Coverage: UncoveredLine, LowTestCoverage, CoverageStats

**Commit:**
```
feat(ir): add Quality, SCA, Coverage fact variants

- Add Function, Variable, CodeSmell, ComplexityViolation
- Add Dependency, DependencyVulnerability, License
- Add UncoveredLine, LowTestCoverage, CoverageStats
- Total: 17 atomic fact variants
- Add supporting types (Ecosystem, CveId, CoveragePercentage, etc.)

Related: US-02.2
```

---

### US-02.3: Fact y Provenance

**Como** desarrollador del motor  
**Quiero** una estructura `Fact` inmutable con metadata de proveniencia  
**Para** rastrear el origen de cada hecho y facilitar debugging/auditoría

**Criterios de Aceptación:**
- [ ] Struct `Fact` con: `id`, `fact_type`, `location`, `provenance`, `confidence`
- [ ] `FactId` opaco (UUID)
- [ ] `Provenance` contiene: `extractor_name`, `extractor_version`, `timestamp`
- [ ] `SourceLocation` con `ProjectPath`, `LineNumber`, `ColumnNumber`
- [ ] Fact es inmutable (no métodos mutadores públicos)
- [ ] Tests de construcción y serialización

---

#### Tareas Técnicas (TDD)

##### T-02.3.1: Implementar `Fact` y `FactId`

**1. 🔴 RED**

```rust
#[test]
fn test_fact_id_unique() {
    let id1 = FactId::new();
    let id2 = FactId::new();
    assert_ne!(id1, id2);
}

#[test]
fn test_fact_construction() {
    let tmp = TempDir::new().unwrap();
    let location = SourceLocation::new(
        ProjectPath::new("src/main.rs", tmp.path()).unwrap(),
        LineNumber::new(10).unwrap(),
        ColumnNumber::new(5).unwrap(),
        LineNumber::new(10).unwrap(),
        ColumnNumber::new(20).unwrap(),
    );
    
    let provenance = Provenance {
        extractor: ExtractorId::TreeSitter,
        version: "1.0.0".to_string(),
        confidence: Confidence::HIGH,
    };
    
    let fact = Fact::new(
        FactType::Vulnerability {
            cwe_id: Some(CweId::SQL_INJECTION),
            owasp_category: Some(OwaspCategory::A03_Injection),
            severity: Severity::High,
            cvss_score: Some(8.5),
            description: "Test vulnerability".to_string(),
            confidence: Confidence::HIGH,
        },
        location,
        provenance,
    );
    
    assert!(fact.id().as_uuid().get_version_num() == 4);
}
```

**2. 🟢 GREEN**

```rust
// hodei-ir/src/fact.rs
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FactId(Uuid);

impl FactId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }
    
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for FactId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fact {
    id: FactId,
    fact_type: FactType,
    location: SourceLocation,
    provenance: Provenance,
    extracted_at: DateTime<Utc>,
}

impl Fact {
    pub fn new(
        fact_type: FactType,
        location: SourceLocation,
        provenance: Provenance,
    ) -> Self {
        Self {
            id: FactId::new(),
            fact_type,
            location,
            provenance,
            extracted_at: Utc::now(),
        }
    }
    
    // Solo getters, sin setters (inmutable)
    pub fn id(&self) -> FactId { self.id }
    pub fn fact_type(&self) -> &FactType { &self.fact_type }
    pub fn location(&self) -> &SourceLocation { &self.location }
    pub fn provenance(&self) -> &Provenance { &self.provenance }
    pub fn extracted_at(&self) -> DateTime<Utc> { self.extracted_at }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: ProjectPath,
    pub start_line: LineNumber,
    pub start_column: ColumnNumber,
    pub end_line: LineNumber,
    pub end_column: ColumnNumber,
}

impl SourceLocation {
    pub fn new(
        file: ProjectPath,
        start_line: LineNumber,
        start_column: ColumnNumber,
        end_line: LineNumber,
        end_column: ColumnNumber,
    ) -> Self {
        Self {
            file,
            start_line,
            start_column,
            end_line,
            end_column,
        }
    }
    
    pub fn span(&self) -> (LineNumber, LineNumber) {
        (self.start_line, self.end_line)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Provenance {
    pub extractor: ExtractorId,
    pub version: String,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExtractorId {
    TreeSitter,
    OxcParser,
    SemgrepTaint,
    DataFlowAnalyzer,
    SymbolicExecutor,
    CargoAudit,
    NpmAudit,
    TrivyScanner,
    JaCoCoParser,
    LcovParser,
    CoberturaParser,
    Custom(String),
}

impl ExtractorId {
    pub fn as_str(&self) -> &str {
        match self {
            Self::TreeSitter => "tree-sitter",
            Self::OxcParser => "oxc",
            Self::SemgrepTaint => "semgrep-taint",
            Self::DataFlowAnalyzer => "dataflow",
            Self::SymbolicExecutor => "symbolic",
            Self::CargoAudit => "cargo-audit",
            Self::NpmAudit => "npm-audit",
            Self::TrivyScanner => "trivy",
            Self::JaCoCoParser => "jacoco",
            Self::LcovParser => "lcov",
            Self::CoberturaParser => "cobertura",
            Self::Custom(name) => name,
        }
    }
}
```

**Commit:**
```
feat(ir): add Fact struct with immutable design

- Implement Fact with FactId, FactType, location, provenance
- Add SourceLocation with typed line/column numbers
- Add Provenance with extractor metadata and timestamp
- Ensure immutability with private fields and getter-only API
- Add tests for uniqueness and immutability

Related: US-02.3
```

---

### US-02.4: IRValidator

**Como** usuario del sistema  
**Quiero** validación exhaustiva del IR antes de procesarlo  
**Para** detectar inconsistencias (FlowId huérfanos, paths inválidos) y fallar rápidamente

**Criterios de Aceptación:**
- [ ] `IRValidator` valida:
  - Todos los `FlowId` en DataFlow tienen facts asociados
  - Todos los `ProjectPath` están dentro del project root
  - No hay `FactId` duplicados
  - Schema version es compatible
- [ ] Retorna `Result<(), ValidationError>` con errores detallados
- [ ] Tests con IRs inválidos

---

#### Tareas Técnicas (TDD)

**Implementación similar a la mostrada anteriormente, enfocándose en:**
- Tests de FactId duplicados
- Tests de FlowId huérfanos
- Tests de ProjectPath fuera de root
- Tests de schema version mismatch

**Commit:**
```
feat(ir): add IRValidator for integrity checking

- Implement IRValidator with checks for:
  - Duplicate FactIds
  - Orphaned FlowIds
  - ProjectPath confinement
  - Schema version compatibility
- Add detailed ValidationError enum
- Add tests for each validation rule

Security: Defense-in-depth for path traversal
Related: US-02.4
```

---

### US-02.5: IntermediateRepresentation Container

**Como** desarrollador del sistema  
**Quiero** un contenedor eficiente para el IR completo  
**Para** almacenar y consultar miles de hechos con bajo overhead

**Criterios de Aceptación:**
- [ ] `IntermediateRepresentation` struct con `Vec<Fact>`
- [ ] Métodos: `add_fact`, `facts`, `fact_count`, `facts_by_type`
- [ ] Metadata: project info, schema version, timestamps
- [ ] Serialización JSON preparada
- [ ] Tests de construcción y queries básicos

---

## 📊 Resumen de la Épica

### Story Points Breakdown

| User Story | SP | Complejidad | Riesgo |
|------------|-----|-------------|--------|
| US-02.1: Newtypes | 21 | Media | Bajo |
| US-02.2: FactType Enum | 34 | Alta | Medio |
| US-02.3: Fact & Provenance | 13 | Media | Bajo |
| US-02.4: IRValidator | 13 | Media | Medio |
| US-02.5: IR Container | 8 | Baja | Bajo |
| **Total** | **89** | - | - |

### Criterios de Finalización de Épica

- [ ] Todos los tests pasan (100% green)
- [ ] Coverage ≥ 85% en módulos críticos
- [ ] Property tests implementados para newtypes
- [ ] Fuzzing básico para ProjectPath ejecutado sin fallos
- [ ] Documentación con ejemplos para cada tipo público
- [ ] Benchmarks baseline ejecutados y documentados
- [ ] ADR-001 implementado (no meta-hechos en IR)

### Métricas de Éxito

- **Tests:** ≥200 tests unitarios, ≥20 property tests
- **Coverage:** ≥85% line coverage, ≥80% branch coverage
- **Performance:** Fact::new() < 100ns, IR validation < 1ms/1000 facts
- **Security:** 0 findings en cargo audit, 0 path traversal vulns

---

## 🔗 Referencias

- [ARCHITECTURE-V3.2-FINAL.md](../ARCHITECTURE-V3.2-FINAL.md) § 3 (IR Schema)
- [ADR-001: Atomic Facts Only](../decisions/ADR-001-atomic-facts-only.md)
- [EPIC-01: Setup](./EPIC-01-setup.md)
- [EPIC-03: Zero-Copy Serialization](./EPIC-03-zero-copy.md) (siguiente)

---

**Última actualización:** 2025-01-XX  
**Estado:** READY FOR IMPLEMENTATION