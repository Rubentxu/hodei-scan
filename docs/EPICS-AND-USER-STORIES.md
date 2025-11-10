# hodei-scan v3.1: Épicas e Historias de Usuario
## Product Backlog Completo con Enfoque TDD, Arquitectura Hexagonal y Principios SOLID

**Versión:** 3.1.0  
**Fecha:** 2025-01-XX  
**Metodología:** TDD + Arquitectura Hexagonal + SOLID + Connascence  
**Convención de Commits:** Conventional Commits  

---

## 📋 Índice de Épicas

1. [**EPIC-01:** Setup y Fundamentos del Proyecto](#epic-01-setup-y-fundamentos-del-proyecto)
2. [**EPIC-02:** IR Core - Tipos Type-Safe](#epic-02-ir-core---tipos-type-safe)
3. [**EPIC-03:** IR Schema - FactTypes y Validación](#epic-03-ir-schema---facttypes-y-validación)
4. [**EPIC-04:** Serialización Zero-Copy con Cap'n Proto](#epic-04-serialización-zero-copy-con-capn-proto)
5. [**EPIC-05:** IndexedFactStore - Storage e Indexación](#epic-05-indexedfactstore---storage-e-indexación)
6. [**EPIC-06:** Query Planner y Optimización](#epic-06-query-planner-y-optimización)
7. [**EPIC-07:** RuleEngine - Motor de Evaluación](#epic-07-ruleengine---motor-de-evaluación)
8. [**EPIC-08:** DSL Parser con PEG Grammar](#epic-08-dsl-parser-con-peg-grammar)
9. [**EPIC-09:** Quality Gates y Agregaciones](#epic-09-quality-gates-y-agregaciones)
10. [**EPIC-10:** Sistema de Plugins](#epic-10-sistema-de-plugins)
11. [**EPIC-11:** Extractores Nivel 1 (AST)](#epic-11-extractores-nivel-1-ast)
12. [**EPIC-12:** Extractores Nivel 2 (SAST)](#epic-12-extractores-nivel-2-sast)
13. [**EPIC-13:** CLI y Configuración](#epic-13-cli-y-configuración)
14. [**EPIC-14:** Seguridad y Hardening](#epic-14-seguridad-y-hardening)

---

## 🎯 Estructura de Historia de Usuario

Cada historia sigue este formato:

```
### US-XX.YY: [Título]

**Como:** [Rol]
**Quiero:** [Funcionalidad]
**Para:** [Beneficio]

**Criterios de Aceptación:**
- [ ] Criterio 1
- [ ] Criterio 2

**Principios Aplicados:**
- SOLID: [Principios específicos]
- Connascence: [Tipo de connascence minimizada]
- Hexagonal: [Capa/Puerto/Adaptador]

**Tareas Técnicas (TDD):**
1. 🔴 RED: Escribir test que falla
2. 🟢 GREEN: Implementación mínima
3. 🔵 REFACTOR: Optimización

**Definición de Done:**
- [ ] Tests unitarios pasan (coverage >80%)
- [ ] Tests de integración pasan
- [ ] Documentación actualizada
- [ ] Conventional commit realizado
- [ ] Code review aprobado

**Commit Message Template:**
`feat(scope): descripción breve`
```

---

## EPIC-01: Setup y Fundamentos del Proyecto

**Objetivo:** Establecer la estructura del monorepo Rust con CI/CD y tooling básico.

**Valor de Negocio:** Fundación sólida para desarrollo iterativo y calidad desde el inicio.

**Estimación Total:** 40 Story Points (1 Sprint)

---

### US-01.01: Inicializar Monorepo Rust con Cargo Workspace

**Como:** Desarrollador
**Quiero:** Un workspace de Cargo con estructura modular
**Para:** Desarrollar crates independientes pero cohesivos

**Criterios de Aceptación:**
- [ ] Workspace con 5 crates iniciales (`hodei-ir`, `hodei-engine`, `hodei-dsl`, `hodei-extractors`, `hodei-cli`)
- [ ] `Cargo.toml` raíz con workspace members
- [ ] Dependencias compartidas en workspace
- [ ] Cada crate compila sin warnings

**Principios Aplicados:**
- SOLID: SRP (Single Responsibility) - Un crate por responsabilidad
- Connascence: CoN (Connascence of Name) - Nombres explícitos de módulos
- Hexagonal: Separación de capas por crate

**Tareas Técnicas (TDD):**
1. 🔴 RED: Test que verifica estructura de workspace
   ```rust
   #[test]
   fn workspace_has_all_required_crates() {
       let workspace_toml = std::fs::read_to_string("Cargo.toml").unwrap();
       assert!(workspace_toml.contains("hodei-ir"));
       assert!(workspace_toml.contains("hodei-engine"));
       // ...
   }
   ```

2. 🟢 GREEN: Crear estructura
   ```bash
   cargo new --lib hodei-ir
   cargo new --lib hodei-engine
   cargo new --lib hodei-dsl
   cargo new --lib hodei-extractors
   cargo new --bin hodei-cli
   ```

3. 🔵 REFACTOR: Configurar workspace dependencies
   ```toml
   [workspace.dependencies]
   serde = { version = "1.0", features = ["derive"] }
   thiserror = "1.0"
   ahash = "0.8"
   ```

**Definición de Done:**
- [x] Tests de estructura pasan
- [x] `cargo build --workspace` exitoso
- [x] README.md con estructura documentada
- [x] Commit: `chore(workspace): initialize cargo workspace with 5 crates`

**Estimación:** 3 Story Points

---

### US-01.02: Configurar CI/CD con GitHub Actions

**Como:** Tech Lead
**Quiero:** Pipeline de CI/CD automatizado
**Para:** Garantizar calidad en cada commit

**Criterios de Aceptación:**
- [ ] Pipeline ejecuta `cargo test` en todos los crates
- [ ] Pipeline ejecuta `cargo clippy` con deny warnings
- [ ] Pipeline ejecuta `cargo fmt --check`
- [ ] Pipeline ejecuta `cargo audit` (security)
- [ ] Badge de build status en README

**Principios Aplicados:**
- SOLID: OCP (Open-Closed) - Pipeline extensible
- Connascence: CoA (Connascence of Algorithm) - Minimizada con scripts reutilizables

**Tareas Técnicas (TDD):**
1. 🔴 RED: Test local que simula CI
   ```bash
   #!/bin/bash
   cargo test --workspace || exit 1
   cargo clippy --workspace -- -D warnings || exit 1
   cargo fmt --check || exit 1
   ```

2. 🟢 GREEN: `.github/workflows/ci.yml`
   ```yaml
   name: CI
   on: [push, pull_request]
   jobs:
     test:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v3
         - uses: actions-rs/toolchain@v1
         - run: cargo test --workspace
         - run: cargo clippy --workspace -- -D warnings
         - run: cargo fmt --check
   ```

3. 🔵 REFACTOR: Añadir cache y optimizaciones
   ```yaml
   - uses: actions/cache@v3
     with:
       path: |
         ~/.cargo/bin/
         ~/.cargo/registry/index/
         target/
   ```

**Definición de Done:**
- [x] Pipeline verde en primer push
- [x] Badge en README
- [x] Documentación de pipeline en CONTRIBUTING.md
- [x] Commit: `ci(github): add ci/cd pipeline with test, clippy, fmt`

**Estimación:** 5 Story Points

---

### US-01.03: Setup de Herramientas de Desarrollo

**Como:** Desarrollador
**Quiero:** Tooling consistente en el equipo
**Para:** Evitar conflictos de estilo y configuración

**Criterios de Aceptación:**
- [ ] `.rustfmt.toml` con configuración del equipo
- [ ] `.clippy.toml` con lints adicionales
- [ ] `.editorconfig` para IDEs
- [ ] Pre-commit hooks con `husky` equivalent (cargo-husky)
- [ ] `rust-toolchain.toml` fija versión de Rust

**Principios Aplicados:**
- SOLID: Consistencia como base de SRP
- Connascence: CoV (Connascence of Value) - Evitada con herramientas

**Tareas Técnicas (TDD):**
1. 🔴 RED: Test que valida configuración
   ```rust
   #[test]
   fn rustfmt_config_exists() {
       assert!(std::path::Path::new(".rustfmt.toml").exists());
   }
   ```

2. 🟢 GREEN: Crear archivos de configuración
   ```toml
   # .rustfmt.toml
   edition = "2021"
   max_width = 100
   use_field_init_shorthand = true
   
   # .clippy.toml
   cognitive-complexity-threshold = 15
   ```

3. 🔵 REFACTOR: Pre-commit hooks
   ```bash
   cargo install cargo-husky
   cargo husky install
   ```

**Definición de Done:**
- [x] Todos los archivos de config committeados
- [x] Pre-commit hooks funcionando
- [x] Commit: `chore(tooling): setup rustfmt, clippy, and pre-commit hooks`

**Estimación:** 2 Story Points

---

### US-01.04: Documentación de Arquitectura (ADRs)

**Como:** Arquitecto
**Quiero:** Documentar decisiones arquitectónicas
**Para:** Mantener contexto y rationale

**Criterios de Aceptación:**
- [ ] Directorio `docs/adr/` con plantilla
- [ ] ADR-001: Elección de Rust
- [ ] ADR-002: Arquitectura Hexagonal
- [ ] ADR-003: Cap'n Proto para IR
- [ ] ADR-004: PEG Parser para DSL

**Principios Aplicados:**
- SOLID: Documentación como SRP de conocimiento
- Connascence: CoK (Connascence of Knowledge) - Explícita

**Tareas Técnicas (TDD):**
1. 🔴 RED: Test de presencia de ADRs
   ```rust
   #[test]
   fn adr_directory_has_required_decisions() {
       let adr_dir = std::path::Path::new("docs/adr");
       assert!(adr_dir.join("001-rust-language.md").exists());
       assert!(adr_dir.join("002-hexagonal-architecture.md").exists());
   }
   ```

2. 🟢 GREEN: Crear ADRs con template
   ```markdown
   # ADR-001: Elección de Rust
   
   ## Estado
   Aceptado
   
   ## Contexto
   Necesitamos performance, type-safety y zero-cost abstractions
   
   ## Decisión
   Usar Rust como lenguaje principal
   
   ## Consecuencias
   - Positivas: Performance, safety
   - Negativas: Curva de aprendizaje
   ```

3. 🔵 REFACTOR: Index de ADRs con links

**Definición de Done:**
- [x] 4 ADRs escritos y revisados
- [x] INDEX.md apunta a ADRs
- [x] Commit: `docs(adr): add architectural decision records 001-004`

**Estimación:** 5 Story Points

---

## EPIC-02: IR Core - Tipos Type-Safe

**Objetivo:** Implementar tipos newtype con validación para eliminar Primitive Obsession y CoP.

**Valor de Negocio:** Prevención de bugs en compile-time, base type-safe para todo el sistema.

**Estimación Total:** 55 Story Points (1.5 Sprints)

**Dependencias:** EPIC-01

---

### US-02.01: Implementar Tipo `Confidence` con Validación

**Como:** Desarrollador Core
**Quiero:** Un tipo `Confidence` que garantice valores en [0.0, 1.0]
**Para:** Prevenir valores inválidos en compile-time

**Criterios de Aceptación:**
- [ ] Struct `Confidence` newtype sobre `f32`
- [ ] Constructor `new()` valida rango [0.0, 1.0]
- [ ] Constantes `HIGH`, `MEDIUM`, `LOW`
- [ ] Implementa `PartialOrd` para comparaciones
- [ ] Error type `ConfidenceError` con thiserror

**Principios Aplicados:**
- SOLID: SRP (Confidence tiene una responsabilidad)
- Connascence: CoM → CoT (de significado a tipo)
- Hexagonal: Domain Model (núcleo del dominio)

**Tareas Técnicas (TDD):**

1. 🔴 RED: Test que falla
   ```rust
   // hodei-ir/src/confidence.rs
   #[cfg(test)]
   mod tests {
       use super::*;
   
       #[test]
       fn confidence_rejects_out_of_range() {
           assert!(Confidence::new(-0.1).is_err());
           assert!(Confidence::new(1.1).is_err());
       }
   
       #[test]
       fn confidence_accepts_valid_range() {
           assert!(Confidence::new(0.0).is_ok());
           assert!(Confidence::new(0.5).is_ok());
           assert!(Confidence::new(1.0).is_ok());
       }
   
       #[test]
       fn confidence_constants_are_valid() {
           assert_eq!(Confidence::HIGH.value(), 0.9);
           assert_eq!(Confidence::MEDIUM.value(), 0.6);
           assert_eq!(Confidence::LOW.value(), 0.3);
       }
   
       #[test]
       fn confidence_partial_ord_works() {
           assert!(Confidence::HIGH > Confidence::LOW);
       }
   }
   ```

2. 🟢 GREEN: Implementación mínima
   ```rust
   // hodei-ir/src/confidence.rs
   use thiserror::Error;
   
   #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
   pub struct Confidence(f32);
   
   #[derive(Error, Debug)]
   pub enum ConfidenceError {
       #[error("Confidence value {value} out of range [0.0, 1.0]")]
       OutOfRange { value: f32 },
   }
   
   impl Confidence {
       pub const HIGH: Self = Self(0.9);
       pub const MEDIUM: Self = Self(0.6);
       pub const LOW: Self = Self(0.3);
   
       pub fn new(value: f32) -> Result<Self, ConfidenceError> {
           if !(0.0..=1.0).contains(&value) {
               return Err(ConfidenceError::OutOfRange { value });
           }
           Ok(Self(value))
       }
   
       pub fn value(&self) -> f32 {
           self.0
       }
   }
   
   impl Default for Confidence {
       fn default() -> Self {
           Self::MEDIUM
       }
   }
   ```

3. 🔵 REFACTOR: Añadir traits útiles
   ```rust
   impl std::fmt::Display for Confidence {
       fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
           write!(f, "{:.2}", self.0)
       }
   }
   
   #[cfg(feature = "serde")]
   impl serde::Serialize for Confidence {
       fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
       where
           S: serde::Serializer,
       {
           serializer.serialize_f32(self.0)
       }
   }
   
   #[cfg(feature = "serde")]
   impl<'de> serde::Deserialize<'de> for Confidence {
       fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
       where
           D: serde::Deserializer<'de>,
       {
           let value = f32::deserialize(deserializer)?;
           Self::new(value).map_err(serde::de::Error::custom)
       }
   }
   ```

**Definición de Done:**
- [x] Tests unitarios 100% coverage
- [x] Benchmarks de performance (debe ser zero-cost)
- [x] Documentación con ejemplos
- [x] Commit: `feat(ir): add Confidence newtype with validation`

**Estimación:** 3 Story Points

---

### US-02.02: Implementar Tipo `ProjectPath` con Seguridad

**Como:** Security Engineer
**Quiero:** Un tipo `ProjectPath` que previene path traversal
**Para:** Eliminar vulnerabilidad de seguridad por diseño

**Criterios de Aceptación:**
- [ ] Struct `ProjectPath` newtype sobre `PathBuf`
- [ ] Constructor `new()` canonicaliza y valida confinamiento
- [ ] Rechaza paths fuera del proyecto (../, absolute paths)
- [ ] Error type `PathError` con contexto
- [ ] Tests de seguridad exhaustivos

**Principios Aplicados:**
- SOLID: SRP (validación de paths)
- Connascence: CoM → CoT + Security
- Hexagonal: Value Object en Domain

**Tareas Técnicas (TDD):**

1. 🔴 RED: Tests de seguridad
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       use std::path::PathBuf;
   
       #[test]
       fn rejects_path_traversal_attempts() {
           let root = PathBuf::from("/project");
           
           assert!(ProjectPath::new("../../../etc/passwd", &root).is_err());
           assert!(ProjectPath::new("/etc/passwd", &root).is_err());
           assert!(ProjectPath::new("../../outside", &root).is_err());
       }
   
       #[test]
       fn accepts_valid_project_paths() {
           let root = PathBuf::from("/project");
           
           assert!(ProjectPath::new("src/main.rs", &root).is_ok());
           assert!(ProjectPath::new("./tests/test.rs", &root).is_ok());
       }
   
       #[test]
       fn canonicalizes_paths() {
           let root = PathBuf::from("/project");
           let path = ProjectPath::new("src/../src/main.rs", &root).unwrap();
           
           assert_eq!(path.as_str(), "src/main.rs");
       }
   }
   ```

2. 🟢 GREEN: Implementación
   ```rust
   use std::path::{Path, PathBuf};
   use thiserror::Error;
   
   #[derive(Debug, Clone, PartialEq, Eq, Hash)]
   pub struct ProjectPath(PathBuf);
   
   #[derive(Error, Debug)]
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
   }
   
   impl ProjectPath {
       pub fn new(
           path: impl AsRef<Path>,
           project_root: &Path,
       ) -> Result<Self, PathError> {
           let path = path.as_ref();
           
           // 1. Canonicalizar (resuelve .., symlinks)
           let canonical = path
               .canonicalize()
               .map_err(|e| PathError::Canonicalization {
                   path: path.to_owned(),
                   source: e,
               })?;
           
           // 2. Verificar confinamiento
           if !canonical.starts_with(project_root) {
               return Err(PathError::OutsideProject {
                   attempted: canonical,
                   project_root: project_root.to_owned(),
               });
           }
           
           // 3. Normalizar a relativo
           let relative = canonical
               .strip_prefix(project_root)
               .map_err(|_| PathError::StripPrefixFailed)?
               .to_owned();
           
           Ok(Self(relative))
       }
       
       pub fn as_path(&self) -> &Path {
           &self.0
       }
       
       pub fn as_str(&self) -> &str {
           self.0.to_str().expect("Path validated as UTF-8")
       }
   }
   ```

3. 🔵 REFACTOR: Fuzzing test
   ```rust
   #[cfg(test)]
   mod fuzz_tests {
       use super::*;
       use quickcheck::{Arbitrary, Gen};
       use quickcheck_macros::quickcheck;
   
       #[quickcheck]
       fn fuzz_path_traversal_is_rejected(path: String) -> bool {
           let root = PathBuf::from("/safe/project");
           
           // Si contiene .., debe ser rechazado
           if path.contains("..") {
               ProjectPath::new(&path, &root).is_err()
           } else {
               true // Aceptamos que puede pasar o fallar
           }
       }
   }
   ```

**Definición de Done:**
- [x] Tests de seguridad pasan
- [x] Fuzzing test ejecutado (1M iteraciones)
- [x] Security review aprobado
- [x] Commit: `feat(ir): add ProjectPath with path traversal prevention`

**Estimación:** 5 Story Points

---

### US-02.03: Implementar Tipo `LineNumber` (NonZero)

**Como:** Desarrollador Core
**Quiero:** Un tipo `LineNumber` que no permita línea 0
**Para:** Alinearse con convención de editors (1-indexed)

**Criterios de Aceptación:**
- [ ] Struct `LineNumber` newtype sobre `NonZeroU32`
- [ ] Constructor `new()` rechaza 0
- [ ] Implementa `PartialOrd` para sorting
- [ ] Conversión a/desde `u32`

**Principios Aplicados:**
- SOLID: SRP
- Connascence: CoT (tipo expresa invariante)
- Hexagonal: Value Object

**Tareas Técnicas (TDD):**

1. 🔴 RED: Tests
   ```rust
   #[test]
   fn line_number_rejects_zero() {
       assert!(LineNumber::new(0).is_err());
   }
   
   #[test]
   fn line_number_accepts_positive() {
       assert!(LineNumber::new(1).is_ok());
       assert!(LineNumber::new(1000).is_ok());
   }
   
   #[test]
   fn line_numbers_are_ordered() {
       let line1 = LineNumber::new(1).unwrap();
       let line2 = LineNumber::new(2).unwrap();
       assert!(line1 < line2);
   }
   ```

2. 🟢 GREEN: Implementación
   ```rust
   use std::num::NonZeroU32;
   use thiserror::Error;
   
   #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
   pub struct LineNumber(NonZeroU32);
   
   #[derive(Error, Debug)]
   pub enum LineNumberError {
       #[error("Line number cannot be zero")]
       ZeroLine,
   }
   
   impl LineNumber {
       pub fn new(line: u32) -> Result<Self, LineNumberError> {
           NonZeroU32::new(line)
               .map(Self)
               .ok_or(LineNumberError::ZeroLine)
       }
       
       pub fn get(&self) -> u32 {
           self.0.get()
       }
   }
   ```

3. 🔵 REFACTOR: Traits adicionales
   ```rust
   impl std::fmt::Display for LineNumber {
       fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
           write!(f, "{}", self.0)
       }
   }
   
   impl From<LineNumber> for u32 {
       fn from(ln: LineNumber) -> Self {
           ln.0.get()
       }
   }
   ```

**Definición de Done:**
- [x] Tests pasan
- [x] Property-based tests con quickcheck
- [x] Commit: `feat(ir): add LineNumber newtype (1-indexed, non-zero)`

**Estimación:** 2 Story Points

---

### US-02.04: Implementar Tipo `FlowId` con Factory Scoped

**Como:** SAST Engineer
**Quiero:** Un tipo `FlowId` que garantice unicidad
**Para:** Correlacionar flujos de taint sin colisiones

**Criterios de Aceptación:**
- [ ] Struct `FlowId` newtype sobre `Arc<str>`
- [ ] Factory `new_scoped(ExtractorId, u64)` con namespace
- [ ] Factory `new_uuid()` para máxima unicidad
- [ ] Implementa `Hash` y `Eq` para usar en mapas

**Principios Aplicados:**
- SOLID: SRP (generación de IDs)
- Connascence: CoI (Identidad explícita)
- Hexagonal: Value Object

**Tareas Técnicas (TDD):**

1. 🔴 RED: Tests de unicidad
   ```rust
   #[test]
   fn flow_ids_from_different_extractors_are_unique() {
       let id1 = FlowId::new_scoped(&ExtractorId::TreeSitter, 1);
       let id2 = FlowId::new_scoped(&ExtractorId::OxcParser, 1);
       
       assert_ne!(id1, id2);
   }
   
   #[test]
   fn flow_ids_from_same_extractor_different_sequence_are_unique() {
       let id1 = FlowId::new_scoped(&ExtractorId::TreeSitter, 1);
       let id2 = FlowId::new_scoped(&ExtractorId::TreeSitter, 2);
       
       assert_ne!(id1, id2);
   }
   
   #[test]
   fn uuid_flow_ids_are_unique() {
       let id1 = FlowId::new_uuid();
       let id2 = FlowId::new_uuid();
       
       assert_ne!(id1, id2);
   }
   ```

2. 🟢 GREEN: Implementación
   ```rust
   use std::sync::Arc;
   use uuid::Uuid;
   
   #[derive(Debug, Clone, PartialEq, Eq, Hash)]
   pub struct FlowId(Arc<str>);
   
   impl FlowId {
       pub fn new_scoped(extractor: &ExtractorId, sequence: u64) -> Self {
           Self(format!("{}::{:016x}", extractor.as_str(), sequence).into())
       }
       
       pub fn new_uuid() -> Self {
           Self(Uuid::new_v4().to_string().into())
       }
       
       pub fn from_string(s: String) -> Self {
           Self(s.into())
       }
       
       pub fn as_str(&self) -> &str {
           &self.0
       }
   }
   ```

3. 🔵 REFACTOR: Benchmark de memoria
   ```rust
   #[bench]
   fn bench_flow_id_memory(b: &mut Bencher) {
       b.iter(|| {
           let ids: Vec<_> = (0..1000)
               .map(|i| FlowId::new_scoped(&ExtractorId::TreeSitter, i))
               .collect();
           
           // Arc<str> debe ser cheap clone
           let cloned = ids.clone();
           (ids, cloned)
       });
   }
   ```

**Definición de Done:**
- [x] Tests de unicidad pasan
- [x] Benchmark confirma Arc es eficiente
- [x] Commit: `feat(ir): add FlowId with scoped factory for uniqueness`

**Estimación:** 3 Story Points

---

### US-02.05: Implementar Tipo `SourceLocation` (Composición)

**Como:** Desarrollador Core
**Quiero:** Un tipo `SourceLocation` que agrupe file+line
**Para:** Reducir Data Clumps y facilitar correlación espacial

**Criterios de Aceptación:**
- [ ] Struct `SourceLocation` con `ProjectPath` y `LineNumber`
- [ ] Campos opcionales: `column`, `end_line`, `end_column`
- [ ] Método `span()` calcula líneas abarcadas
- [ ] Implementa `Hash` y `Eq` para indexación

**Principios Aplicados:**
- SOLID: SRP (localización como concepto)
- Connascence: CoP → Composición (elimina Data Clumps)
- Hexagonal: Value Object compuesto

**Tareas Técnicas (TDD):**

1. 🔴 RED: Tests
   ```rust
   #[test]
   fn source_location_has_required_fields() {
       let loc = SourceLocation::new(
           ProjectPath::new("src/main.rs", &root).unwrap(),
           LineNumber::new(42).unwrap(),
       );
       
       assert_eq!(loc.file.as_str(), "src/main.rs");
       assert_eq!(loc.line.get(), 42);
   }
   
   #[test]
   fn span_calculates_correctly() {
       let loc = SourceLocation {
           file: ProjectPath::new("test.rs", &root).unwrap(),
           line: LineNumber::new(10).unwrap(),
           end_line: Some(LineNumber::new(15).unwrap()),
           column: None,
           end_column: None,
       };
       
       assert_eq!(loc.span(), 6); // 15 - 10 + 1
   }
   ```

2. 🟢 GREEN: Implementación
   ```rust
   #[derive(Debug, Clone, PartialEq, Eq, Hash)]
   pub struct SourceLocation {
       pub file: ProjectPath,
       pub line: LineNumber,
       pub column: Option<ColumnNumber>,
       pub end_line: Option<LineNumber>,
       pub end_column: Option<ColumnNumber>,
   }
   
   impl SourceLocation {
       pub fn new(file: ProjectPath, line: LineNumber) -> Self {
           Self {
               file,
               line,
               column: None,
               end_line: None,
               end_column: None,
           }
       }
       
       pub fn span(&self) -> u32 {
           self.end_line
               .map(|end| end.get() - self.line.get() + 1)
               .unwrap_or(1)
       }
   }
   ```

3. 🔵 REFACTOR: Builder pattern opcional
   ```rust
   pub struct SourceLocationBuilder {
       file: ProjectPath,
       line: LineNumber,
       column: Option<ColumnNumber>,
       end_line: Option<LineNumber>,
       end_column: Option<ColumnNumber>,
   }
   
   impl SourceLocationBuilder {
       pub fn new(file: ProjectPath, line: LineNumber) -> Self {
           Self {
               file,
               line,
               column: None,
               end_line: None,
               end_column: None,
           }
       }
       
       pub fn column(mut self, col: ColumnNumber) -> Self {
           self.column = Some(col);
           self
       }
       
       pub fn span_to(mut self, end: LineNumber) -> Self {
           self.end_line = Some(end);
           self
       }
       
       pub fn build(self) -> SourceLocation {
           SourceLocation {
               file: self.file,
               line: self.line,
               column: self.column,
               end_line: self.end_line,
               end_column: self.end_column,
           }
       }
   }
   ```

**Definición de Done:**
- [x] Tests pasan
- [x] Documentación con ejemplos de builder
- [x] Commit: `feat(ir): add SourceLocation value object with builder`

**Estimación:** 3 Story Points

---

## EPIC-03: IR Schema - FactTypes y Validación

**Objetivo:** Implementar enum `FactType` completo con todas las variantes y validación.

**Valor de Negocio:** Schema formal que actúa como contrato entre extractores y motor.

**Estimación Total:** 89 Story Points (2.5 Sprints)

**Dependencias:** EPIC-02

---

### US-03.01: Implementar FactType::TaintSource con Builder

**Como:** SAST Engineer
**Quiero:** Variante `TaintSource` con builder pattern
**Para:** Expresar fuentes de datos no confiables sin CoP

**Criterios de Aceptación:**
- [ ] Enum `FactType` con variante `TaintSource`
- [ ] Campos: `var`, `flow_id`, `source_type`, `confidence`
- [ ] Builder `TaintSourceBuilder` elimina CoP
- [ ] Tests exhaustivos de validación

**Principios Aplicados:**
- SOLID: SRP (TaintSource es un concepto)
- Connascence: CoP → CoN (builder)
- Hexagonal: Domain Entity

**Tareas Técnicas (TDD):**

1. 🔴 RED: Tests del builder
   ```rust
   #[test]
   fn taint_source_builder_works() {
       let fact = TaintSourceBuilder::new()
           .var(VariableName::new("$_GET['id']").unwrap())
           .flow_id(FlowId::new_uuid())
           .source_type(TaintSourceType::HttpRequestParam)
           .confidence(Confidence::HIGH)
           .build()
           .unwrap();
       
       match fact {
           FactType::TaintSource { var, confidence, .. } => {
               assert_eq!(var.as_str(), "$_GET['id']");
               assert_eq!(confidence, Confidence::HIGH);
           }
           _ => panic!("Expected TaintSource"),
       }
   }
   
   #[test]
   fn taint_source_builder_rejects_missing_required_fields() {
       let result = TaintSourceBuilder::new()
           .confidence(Confidence::HIGH)
           .build();
       
       assert!(result.is_err());
   }
   ```

2. 🟢 GREEN: Implementación
   ```rust
   #[derive(Debug, Clone)]
   pub enum FactType {
       TaintSource {
           var: VariableName,
           flow_id: FlowId,
           source_type: TaintSourceType,
           confidence: Confidence,
       },
       // ... otras variantes
   }
   
   pub struct TaintSourceBuilder {
       var: Option<VariableName>,
       flow_id: Option<FlowId>,
       source_type: Option<TaintSourceType>,
       confidence: Confidence,
   }
   
   impl TaintSourceBuilder {
       pub fn new() -> Self {
           Self {
               var: None,
               flow_id: None,
               source_type: None,
               confidence: Confidence::MEDIUM,
           }
       }
       
       pub fn var(mut self, var: VariableName) -> Self {
           self.var = Some(var);
           self
       }
       
       pub fn flow_id(mut self, flow_id: FlowId) -> Self {
           self.flow_id = Some(flow_id);
           self
       }
       
       pub fn source_type(mut self, st: TaintSourceType) -> Self {
           self.source_type = Some(st);
           self
       }
       
       pub fn confidence(mut self, conf: Confidence) -> Self {
           self.confidence = conf;
           self
       }
       
       pub fn build(self) -> Result<FactType, BuildError> {
           Ok(FactType::TaintSource {
               var: self.var.ok_or(BuildError::MissingField("var"))?,
               flow_id: self.flow_id.ok_or(BuildError::MissingField("flow_id"))?,
               source_type: self.source_type
                   .ok_or(BuildError::MissingField("source_type"))?,
               confidence: self.confidence,
           })
       }
   }
   ```

3. 🔵 REFACTOR: Macro para reducir boilerplate
   ```rust
   macro_rules! fact_builder {
       ($name:ident, $variant:ident, {
           $( $field:ident: $type:ty $(= $default:expr)? ),* $(,)?
       }) => {
           pub struct $name {
               $( $field: Option<$type> ),*
           }
           
           impl $name {
               pub fn new() -> Self {
                   Self {
                       $( $field: None ),*
                   }
               }
               
               $(
                   pub fn $field(mut self, $field: $type) -> Self {
                       self.$field = Some($field);
                       self
                   }
               )*
               
               pub fn build(self) -> Result<FactType, BuildError> {
                   Ok(FactType::$variant {
                       $(
                           $field: self.$field
                               .ok_or(BuildError::MissingField(stringify!($field)))?,
                       )*
                   })
               }
           }
       };
   }
   ```

**Definición de Done:**
- [x] Tests del builder pasan
- [x] Documentación con ejemplos
- [x] Commit: `feat(ir): add FactType::TaintSource with builder pattern`

**Estimación:** 5 Story Points

---

### US-03.02: Implementar Enum `FactTypeDiscriminant`

**Como:** Motor de Evaluación
**Quiero:** Enum sin datos para identificar tipos de hechos
**Para:** Indexación rápida sin pattern matching pesado

**Criterios de Aceptación:**
- [ ] Enum `FactTypeDiscriminant` con todas las variantes
- [ ] Método `FactType::discriminant() -> FactTypeDiscriminant`
- [ ] Implementa `Hash`, `Eq`, `Copy`
- [ ] Conversión desde/hacia strings para DSL

**Principios Aplicados:**
- SOLID: SRP (discriminante es identidad)
- Connascence: CoM → CoT (enum en lugar de strings)
- Hexagonal: Domain Primitive

**Tareas Técnicas (TDD):**

1. 🔴 RED: Tests
   ```rust
   #[test]
   fn discriminant_is_correct() {
       let fact = FactType::TaintSource { /* ... */ };
       assert_eq!(fact.discriminant(), FactTypeDiscriminant::TaintSource);
   }
   
   #[test]
   fn discriminant_from_string_works() {
       assert_eq!(
           FactTypeDiscriminant::from_str("TaintSource").unwrap(),
           FactTypeDiscriminant::TaintSource
       );
   }
   
   #[test]
   fn discriminant_from_invalid_string_fails() {
       assert!(FactTypeDiscriminant::from_str("InvalidType").is_err());
   }
   
   #[test]
   fn discriminant_to_string_roundtrips() {
       let disc = FactTypeDiscriminant::TaintSource;
       let s = disc.as_str();
       let parsed = FactTypeDiscriminant::from_str(s).unwrap();
       assert_eq!(disc, parsed);
   }
   ```

2. 🟢 GREEN: Implementación
   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
   #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
   pub enum FactTypeDiscriminant {
       TaintSource,
       TaintSink,
       Sanitization,
       UnsafeCall,
       CryptographicOperation,
       Vulnerability,
       Function,
       Variable,
       CodeSmell,
       ComplexityViolation,
       Dependency,
       DependencyVulnerability,
       License,
       UncoveredLine,
       LowTestCoverage,
       CoverageStats,
       VulnerableUncovered,
       SecurityTechnicalDebt,
       QualitySecurityCorrelation,
   }
   
   impl FactTypeDiscriminant {
       pub fn from_str(s: &str) -> Result<Self, ParseError> {
           match s {
               "TaintSource" => Ok(Self::TaintSource),
               "TaintSink" => Ok(Self::TaintSink),
               // ... todas las variantes
               _ => Err(ParseError::UnknownFactType {
                   provided: s.to_string(),
                   available: Self::all_variants(),
               }),
           }
       }
       
       pub fn as_str(&self) -> &'static str {
           match self {
               Self::TaintSource => "TaintSource",
               Self::TaintSink => "TaintSink",
               // ...
           }
       }
       
       pub fn all_variants() -> &'static [&'static str] {
           &[
               "TaintSource",
               "TaintSink",
               // ...
           ]
       }
   }
   
   impl FactType {
       pub fn discriminant(&self) -> FactTypeDiscriminant {
           match self {
               Self::TaintSource { .. } => FactTypeDiscriminant::TaintSource,
               Self::TaintSink { .. } => FactTypeDiscriminant::TaintSink,
               // ...
           }
       }
   }
   ```

3. 🔵 REFACTOR: Macro para DRY
   ```rust
   macro_rules! define_discriminants {
       (
           $(
               $variant:ident => $str:literal
           ),* $(,)?
       ) => {
           #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
           pub enum FactTypeDiscriminant {
               $( $variant ),*
           }
           
           impl FactTypeDiscriminant {
               pub fn from_str(s: &str) -> Result<Self, ParseError> {
                   match s {
                       $( $str => Ok(Self::$variant), )*
                       _ => Err(ParseError::UnknownFactType {
                           provided: s.to_string(),
                           available: Self::all_variants(),
                       }),
                   }
               }
               
               pub fn as_str(&self) -> &'static str {
                   match self {
                       $( Self::$variant => $str, )*
                   }
               }
               
               pub fn all_variants() -> &'static [&'static str] {
                   &[ $( $str ),* ]
               }
           }
       };
   }
   
   define_discriminants! {
       TaintSource => "TaintSource",
       TaintSink => "TaintSink",
       // ...
   }
   ```

**Definición de Done:**
- [x] Tests exhaustivos pasan
- [x] Macro reduce boilerplate a <50 líneas
- [x] Commit: `feat(ir): add FactTypeDiscriminant enum for indexing`

**Estimación:** 3 Story Points

---

### US-03.03: Implementar Todas las Variantes de FactType (Batch)

**Como:** Team Lead
**Quiero:** Todas las 18 variantes de `FactType` implementadas
**Para:** Schema completo según especificación

**Criterios de Aceptación:**
- [ ] 18 variantes implementadas:
  - Security: TaintSource, TaintSink, Sanitization, UnsafeCall, CryptographicOperation, Vulnerability
  - Quality: Function, Variable, CodeSmell, ComplexityViolation
  - SCA: Dependency, DependencyVulnerability, License
  - Coverage: UncoveredLine, LowTestCoverage, CoverageStats
  - Correlations: VulnerableUncovered, SecurityTechnicalDebt, QualitySecurityCorrelation
- [ ] Cada variante con builder
- [ ] Tests para cada variante

**Principios Aplicados:**
- SOLID: SRP (cada variante es un concepto)
- Connascence: CoN (nombres claros)
- Hexagonal: Domain Entities completas

**Tareas Técnicas (TDD):**

1. 🔴 RED: Test matrix
   ```rust
   #[test]
   fn all_fact_types_have_builders() {
       // TaintSource
       assert!(TaintSourceBuilder::new().build().is_ok());
       
       // TaintSink
       assert!(TaintSinkBuilder::new().build().is_ok());
       
       // Function
       assert!(FunctionBuilder::new().build().is_ok());
       
       // ... resto de variantes
   }
   
   #[test]
   fn all_fact_types_have_discriminants() {
       let discriminants = vec![
           FactTypeDiscriminant::TaintSource,
           FactTypeDiscriminant::TaintSink,
           // ... todas
       ];
       
       assert_eq!(discriminants.len(), 18);
   }
   ```

2. 🟢 GREEN: Implementación batch (paralela)
   ```rust
   // Dividir trabajo en 3 PRs paralelos:
   // PR 1: Security facts (6 variantes)
   // PR 2: Quality + SCA facts (7 variantes)
   // PR 3: Coverage + Correlations (5 variantes)
   
   // Ejemplo: Function
   #[derive(Debug, Clone)]
   pub enum FactType {
       Function {
           name: FunctionName,
           visibility: Visibility,
           cyclomatic_complexity: u32,
           cognitive_complexity: u32,
           lines_of_code: u32,
           parameter_count: u32,
       },
       // ...
   }
   
   fact_builder!(FunctionBuilder, Function, {
       name: FunctionName,
       visibility: Visibility = Visibility::Public,
       cyclomatic_complexity: u32 = 0,
       cognitive_complexity: u32 = 0,
       lines_of_code: u32 = 0,
       parameter_count: u32 = 0,
   });
   ```

3. 🔵 REFACTOR: Integration tests
   ```rust
   #[test]
   fn fact_type_serde_roundtrips() {
       let facts = vec![
           TaintSourceBuilder::new()/*..*/.build().unwrap(),
           FunctionBuilder::new()/*..*/.build().unwrap(),
           // ... todas las variantes
       ];
       
       for fact in facts {
           let json = serde_json::to_string(&fact).unwrap();
           let parsed: FactType = serde_json::from_str(&json).unwrap();
           assert_eq!(fact, parsed);
       }
   }
   ```

**Definición de Done:**
- [x] 18 variantes + 18 builders implementados
- [x] Tests unitarios para cada variante
- [x] Integration test de serde
- [x] 3 Commits:
  - `feat(ir): add security fact types (TaintSource, TaintSink, etc)`
  - `feat(ir): add quality and sca fact types`
  - `feat(ir): add coverage and correlation fact types`

**Estimación:** 34 Story Points (trabajo paralelo de 3 devs = ~11 SP cada uno)

---

### US-03.04: Implementar Struct `Fact` con Provenance

**Como:** Desarrollador Core
**Quiero:** Struct `Fact` que envuelva `FactType` con metadata
**Para:** Rastrear origen y confianza de cada hecho

**Criterios de Aceptación:**
- [ ] Struct `Fact` con campos: `id`, `fact_type`, `location`, `provenance`, `extracted_at`, `context`
- [ ] Struct `Provenance` con `extractor`, `version`, `confidence`
- [ ] Enum `ExtractorId` para identificar extractores
- [ ] Método `Fact::new()` con defaults sensibles

**Principios Aplicados:**
- SOLID: SRP (Fact agrega metadata)
- Connascence: CoN (metadata explícita)
- Hexagonal: Domain Entity con metadata

**Tareas Técnicas (TDD):**

1. 🔴 RED: Tests
   ```rust
   #[test]
   fn fact_has_unique_id() {
       let fact1 = Fact::new(FactType::TaintSource { /*...*/ });
       let fact2 = Fact::new(FactType::TaintSource { /*...*/ });
       
       assert_ne!(fact1.id, fact2.id);
   }
   
   #[test]
   fn fact_includes_provenance() {
       let fact = Fact::new(FactType::TaintSource { /*...*/ })
           .with_provenance(Provenance {
               extractor: ExtractorId::TreeSitter,
               version: SemanticVersion::new(1, 0, 0),
               confidence: Confidence::HIGH,
           });
       
       assert_eq!(fact.provenance.extractor, ExtractorId::TreeSitter);
   }
   ```

2. 🟢 GREEN: Implementación
   ```rust
   use chrono::{DateTime, Utc};
   use std::sync::atomic::{AtomicU64, Ordering};
   
   #[derive(Debug, Clone)]
   pub struct Fact {
       pub id: FactId,
       pub fact_type: FactType,
       pub location: Option<SourceLocation>,
       pub provenance: Provenance,
       pub extracted_at: DateTime<Utc>,
       pub context: FactContext,
   }
   
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
   pub struct FactId(pub u64);
   
   impl FactId {
       pub fn new() -> Self {
           static COUNTER: AtomicU64 = AtomicU64::new(0);
           Self(COUNTER.fetch_add(1, Ordering::SeqCst))
       }
   }
   
   #[derive(Debug, Clone)]
   pub struct Provenance {
       pub extractor: ExtractorId,
       pub version: SemanticVersion,
       pub confidence: Confidence,
   }
   
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
       Custom(&'static str),
   }
   
   impl ExtractorId {
       pub fn as_str(&self) -> &str {
           match self {
               Self::TreeSitter => "tree_sitter",
               Self::OxcParser => "oxc_parser",
               // ...
               Self::Custom(name) => name,
           }
       }
   }
   
   #[derive(Debug, Clone, Default)]
   pub struct FactContext {
       pub tags: Vec<String>,
       pub metadata: HashMap<String, serde_json::Value>,
   }
   
   impl Fact {
       pub fn new(fact_type: FactType) -> Self {
           Self {
               id: FactId::new(),
               fact_type,
               location: None,
               provenance: Provenance::default(),
               extracted_at: Utc::now(),
               context: FactContext::default(),
           }
       }
       
       pub fn with_location(mut self, loc: SourceLocation) -> Self {
           self.location = Some(loc);
           self
       }
       
       pub fn with_provenance(mut self, prov: Provenance) -> Self {
           self.provenance = prov;
           self
       }
   }
   ```

3. 🔵 REFACTOR: Builder para Fact
   ```rust
   pub struct FactBuilder {
       fact_type: FactType,
       location: Option<SourceLocation>,
       provenance: Option<Provenance>,
       context: FactContext,
   }
   
   impl FactBuilder {
       pub fn new(fact_type: FactType) -> Self {
           Self {
               fact_type,
               location: None,
               provenance: None,
               context: FactContext::default(),
           }
       }
       
       pub fn at(mut self, loc: SourceLocation) -> Self {
           self.location = Some(loc);
           self
       }
       
       pub fn extracted_by(
           mut self,
           extractor: ExtractorId,
           version: SemanticVersion,
       ) -> Self {
           self.provenance = Some(Provenance {
               extractor,
               version,
               confidence: Confidence::MEDIUM,
           });
           self
       }
       
       pub fn with_confidence(mut self, conf: Confidence) -> Self {
           if let Some(ref mut prov) = self.provenance {
               prov.confidence = conf;
           }
           self
       }
       
       pub fn tag(mut self, tag: impl Into<String>) -> Self {
           self.context.tags.push(tag.into());
           self
       }
       
       pub fn build(self) -> Fact {
           Fact {
               id: FactId::new(),
               fact_type: self.fact_type,
               location: self.location,
               provenance: self.provenance.unwrap_or_default(),
               extracted_at: Utc::now(),
               context: self.context,
           }
       }
   }
   ```

**Definición de Done:**
- [x] Tests pasan
- [x] Builder hace el código más legible
- [x] Commit: `feat(ir): add Fact struct with provenance and metadata`

**Estimación:** 5 Story Points

---

### US-03.05: Implementar IR Container y Validación

**Como:** Arquitecto
**Quiero:** Struct `IntermediateRepresentation` que agrupe todos los hechos
**Para:** Tener un contenedor validado del análisis completo

**Criterios de Aceptación:**
- [ ] Struct `IntermediateRepresentation` con: `analysis_id`, `timestamp`, `metadata`, `facts`, `stats`, `schema_version`
- [ ] Struct `IRValidator` con método `validate()`
- [ ] Validación de referencias (FlowIds existen)
- [ ] Validación de schema version

**Principios Aplicados:**
- SOLID: SRP (IR es agregado)
- Connascence: CoR (Connascence of Reference) validada
- Hexagonal: Aggregate Root del dominio

**Tareas Técnicas (TDD):**

1. 🔴 RED: Tests de validación
   ```rust
   #[test]
   fn ir_validation_detects_dangling_flow_references() {
       let ir = IntermediateRepresentation {
           facts: vec![
               Fact::new(FactType::TaintSink {
                   consumes_flow: FlowId::from_string("nonexistent".into()),
                   // ...
               }),
           ],
           // ...
       };
       
       let validator = IRValidator::new();
       assert!(validator.validate(&ir).is_err());
   }
   
   #[test]
   fn ir_validation_passes_for_valid_references() {
       let flow_id = FlowId::new_uuid();
       
       let ir = IntermediateRepresentation {
           facts: vec![
               Fact::new(FactType::TaintSource {
                   flow_id: flow_id.clone(),
                   // ...
               }),
               Fact::new(FactType::TaintSink {
                   consumes_flow: flow_id,
                   // ...
               }),
           ],
           // ...
       };
       
       let validator = IRValidator::new();
       assert!(validator.validate(&ir).is_ok());
   }
   ```

2. 🟢 GREEN: Implementación
   ```rust
   use uuid::Uuid;
   
   #[derive(Debug, Clone)]
   pub struct IntermediateRepresentation {
       pub analysis_id: AnalysisId,
       pub timestamp: DateTime<Utc>,
       pub metadata: ProjectMetadata,
       pub facts: Vec<Fact>,
       pub stats: AnalysisStats,
       pub schema_version: SchemaVersion,
   }
   
   #[derive(Debug, Clone)]
   pub struct AnalysisId(Uuid);
   
   impl AnalysisId {
       pub fn new() -> Self {
           Self(Uuid::new_v4())
       }
   }
   
   #[derive(Debug, Clone)]
   pub struct ProjectMetadata {
       pub name: String,
       pub version: Option<SemanticVersion>,
       pub root_path: PathBuf,
       pub language: Option<String>,
       pub git_commit: Option<String>,
       pub git_branch: Option<String>,
   }
   
   #[derive(Debug, Clone, Default)]
   pub struct AnalysisStats {
       pub total_facts: u64,
       pub facts_by_type: HashMap<FactTypeDiscriminant, u64>,
       pub extractors_used: Vec<ExtractorId>,
       pub duration: Duration,
   }
   
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub struct SchemaVersion {
       pub major: u32,
       pub minor: u32,
   }
   
   impl SchemaVersion {
       pub const CURRENT: Self = Self { major: 3, minor: 1 };
   }
   
   pub struct IRValidator {
       schema_version: SchemaVersion,
   }
   
   impl IRValidator {
       pub fn new() -> Self {
           Self {
               schema_version: SchemaVersion::CURRENT,
           }
       }
       
       pub fn validate(&self, ir: &IntermediateRepresentation) -> Result<(), ValidationError> {
           // 1. Verificar versión
           if ir.schema_version != self.schema_version {
               return Err(ValidationError::IncompatibleSchema {
                   provided: ir.schema_version,
                   expected: self.schema_version,
               });
           }
           
           // 2. Validar cada hecho
           for fact in &ir.facts {
               self.validate_fact(fact)?;
           }
           
           // 3. Validar referencias
           self.validate_references(ir)?;
           
           Ok(())
       }
       
       fn validate_fact(&self, fact: &Fact) -> Result<(), ValidationError> {
           match &fact.fact_type {
               FactType::TaintSource { confidence, .. } => {
                   if !(0.0..=1.0).contains(&confidence.value()) {
                       return Err(ValidationError::InvalidConfidence {
                           value: confidence.value(),
                       });
                   }
               }
               // ... otras validaciones
               _ => {}
           }
           
           Ok(())
       }
       
       fn validate_references(&self, ir: &IntermediateRepresentation) -> Result<(), ValidationError> {
           let mut flow_ids = HashSet::new();
           
           // Recolectar FlowIds de sources
           for fact in &ir.facts {
               if let FactType::TaintSource { flow_id, .. } = &fact.fact_type {
                   flow_ids.insert(flow_id.clone());
               }
           }
           
           // Verificar que sinks referencian flows existentes
           for fact in &ir.facts {
               if let FactType::TaintSink { consumes_flow, .. } = &fact.fact_type {
                   if !flow_ids.contains(consumes_flow) {
                       return Err(ValidationError::DanglingFlowReference {
                           flow_id: consumes_flow.clone(),
                       });
                   }
               }
           }
           
           Ok(())
       }
   }
   
   #[derive(Error, Debug)]
   pub enum ValidationError {
       #[error("Incompatible schema version: got {provided:?}, expected {expected:?}")]
       IncompatibleSchema {
           provided: SchemaVersion,
           expected: SchemaVersion,
       },
       
       #[error("Invalid confidence value: {value}")]
       InvalidConf