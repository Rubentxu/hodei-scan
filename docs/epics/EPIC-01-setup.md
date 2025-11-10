# EPIC-01: Setup y Fundamentos del Proyecto

**Objetivo:** Establecer la estructura del monorepo Rust con CI/CD, tooling y documentación base.

**Valor de Negocio:** Fundación sólida para desarrollo iterativo con calidad desde el inicio.

**Estimación Total:** 40 Story Points  
**Duración Estimada:** 1 Sprint (2 semanas)  
**Prioridad:** 🔴 Critical  
**Estado:** 📝 Ready  

---

## 📋 Contexto

Esta épica establece los cimientos del proyecto hodei-scan v3.1. Sin una base sólida de infraestructura, tooling y documentación, el desarrollo futuro será caótico y propenso a errores.

### Objetivos Específicos

1. **Monorepo Rust:** Workspace con separación clara de responsabilidades por crate
2. **CI/CD:** Pipeline automatizado que garantiza calidad en cada commit
3. **Tooling:** Configuración consistente de rustfmt, clippy, pre-commit hooks
4. **Documentación:** ADRs que documentan decisiones arquitectónicas clave

### Arquitectura Hexagonal

Esta épica establece la **estructura de capas**:
- `hodei-ir` → Domain Layer
- `hodei-engine`, `hodei-dsl` → Application Layer
- `hodei-extractors`, `hodei-cli` → Infrastructure Layer

---

## 🎯 Historias de Usuario

### US-01.01: Inicializar Monorepo Rust con Cargo Workspace

**Como:** Desarrollador  
**Quiero:** Un workspace de Cargo con estructura modular  
**Para:** Desarrollar crates independientes pero cohesivos  

**Prioridad:** Critical  
**Estimación:** 3 Story Points  
**Sprint:** Sprint 1  

#### Criterios de Aceptación

- [ ] Workspace con 5 crates iniciales: `hodei-ir`, `hodei-engine`, `hodei-dsl`, `hodei-extractors`, `hodei-cli`
- [ ] `Cargo.toml` raíz con `[workspace]` members
- [ ] Dependencias compartidas en `[workspace.dependencies]`
- [ ] Cada crate compila sin warnings: `cargo build --workspace`
- [ ] Estructura de directorios sigue convenciones Rust
- [ ] README.md en cada crate con propósito documentado

#### Principios Aplicados

- **SOLID:** 
  - **SRP (Single Responsibility Principle):** Cada crate tiene una responsabilidad única y bien definida
  - `hodei-ir`: Domain types y schema
  - `hodei-engine`: Motor de evaluación
  - `hodei-dsl`: Parser de reglas
  - `hodei-extractors`: Adaptadores de extracción
  - `hodei-cli`: Interfaz de línea de comandos

- **Connascence:**
  - **CoN (Connascence of Name):** Nombres explícitos y descriptivos
  - Evita CoP (Connascence of Position) con workspaces nombrados

- **Hexagonal:**
  - **Domain:** `hodei-ir` (sin dependencias externas)
  - **Application:** `hodei-engine`, `hodei-dsl` (dependen de domain)
  - **Infrastructure:** `hodei-extractors`, `hodei-cli` (implementan puertos)

#### Dependencias

- Ninguna (historia inicial)

#### Tareas Técnicas (TDD)

##### 1. 🔴 RED: Tests que fallan

```rust
// tests/workspace_structure.rs
#[test]
fn workspace_has_all_required_crates() {
    let workspace_toml = std::fs::read_to_string("Cargo.toml")
        .expect("Cargo.toml should exist");
    
    assert!(workspace_toml.contains("hodei-ir"));
    assert!(workspace_toml.contains("hodei-engine"));
    assert!(workspace_toml.contains("hodei-dsl"));
    assert!(workspace_toml.contains("hodei-extractors"));
    assert!(workspace_toml.contains("hodei-cli"));
}

#[test]
fn each_crate_has_readme() {
    let crates = vec![
        "hodei-ir",
        "hodei-engine",
        "hodei-dsl",
        "hodei-extractors",
        "hodei-cli",
    ];
    
    for crate_name in crates {
        let readme_path = format!("{}/README.md", crate_name);
        assert!(
            std::path::Path::new(&readme_path).exists(),
            "README.md missing for {}",
            crate_name
        );
    }
}

#[test]
fn workspace_dependencies_are_defined() {
    let workspace_toml = std::fs::read_to_string("Cargo.toml").unwrap();
    assert!(workspace_toml.contains("[workspace.dependencies]"));
    assert!(workspace_toml.contains("serde"));
    assert!(workspace_toml.contains("thiserror"));
}
```

##### 2. 🟢 GREEN: Implementación mínima

```bash
# Crear estructura de workspace
cargo new --lib hodei-ir
cargo new --lib hodei-engine
cargo new --lib hodei-dsl
cargo new --lib hodei-extractors
cargo new --bin hodei-cli

# Crear Cargo.toml raíz
cat > Cargo.toml << 'EOF'
[workspace]
resolver = "2"
members = [
    "hodei-ir",
    "hodei-engine",
    "hodei-dsl",
    "hodei-extractors",
    "hodei-cli",
]

[workspace.dependencies]
# Serialización
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Async
tokio = { version = "1.35", features = ["full"] }

# Collections
ahash = "0.8"
smallvec = "1.11"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Testing
proptest = "1.4"
criterion = "0.5"

# UUID
uuid = { version = "1.6", features = ["v4", "serde"] }

# DateTime
chrono = { version = "0.4", features = ["serde"] }

# Parser
pest = "2.7"
pest_derive = "2.7"

# Zero-copy serialization (futuro)
capnp = "0.19"
EOF

# Crear README.md en cada crate
for crate in hodei-ir hodei-engine hodei-dsl hodei-extractors hodei-cli; do
  cat > $crate/README.md << EOF
# $crate

**Propósito:** [Descripción del crate]

## Responsabilidad

[Definir responsabilidad según SRP]

## Dependencias

- Lista de dependencias principales

## Uso

\`\`\`rust
// Ejemplo de uso básico
\`\`\`
EOF
done
```

##### 3. 🔵 REFACTOR: Configurar dependencias por crate

```toml
# hodei-ir/Cargo.toml
[package]
name = "hodei-ir"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }

[dev-dependencies]
proptest = { workspace = true }

# hodei-engine/Cargo.toml
[package]
name = "hodei-engine"
version = "0.1.0"
edition = "2021"

[dependencies]
hodei-ir = { path = "../hodei-ir" }
ahash = { workspace = true }
smallvec = { workspace = true }
thiserror = { workspace = true }

[dev-dependencies]
criterion = { workspace = true }

# hodei-dsl/Cargo.toml
[package]
name = "hodei-dsl"
version = "0.1.0"
edition = "2021"

[dependencies]
hodei-ir = { path = "../hodei-ir" }
pest = { workspace = true }
pest_derive = { workspace = true }
thiserror = { workspace = true }

# hodei-extractors/Cargo.toml
[package]
name = "hodei-extractors"
version = "0.1.0"
edition = "2021"

[dependencies]
hodei-ir = { path = "../hodei-ir" }
thiserror = { workspace = true }
anyhow = { workspace = true }

# hodei-cli/Cargo.toml
[package]
name = "hodei-cli"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "hodei-scan"
path = "src/main.rs"

[dependencies]
hodei-ir = { path = "../hodei-ir" }
hodei-engine = { path = "../hodei-engine" }
hodei-dsl = { path = "../hodei-dsl" }
hodei-extractors = { path = "../hodei-extractors" }
clap = { version = "4.4", features = ["derive"] }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
anyhow = { workspace = true }
```

#### Tests de Regresión

```rust
#[test]
fn workspace_compiles_without_warnings() {
    use std::process::Command;
    
    let output = Command::new("cargo")
        .args(&["build", "--workspace", "--all-features"])
        .output()
        .expect("Failed to execute cargo build");
    
    assert!(output.status.success(), "Workspace should compile");
    
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("warning:"),
        "Build should have no warnings: {}",
        stderr
    );
}
```

#### Definición de Done

- [x] Tests de estructura pasan
- [x] `cargo build --workspace` exitoso sin warnings
- [x] `cargo test --workspace` pasa
- [x] README.md en cada crate completado
- [x] Estructura documentada en README.md raíz
- [x] Code review aprobado (2+ approvals)
- [x] CI pipeline verde

#### Commit Message

```
chore(workspace): initialize cargo workspace with 5 crates

- Create hodei-ir (domain layer)
- Create hodei-engine (application layer)
- Create hodei-dsl (application layer)
- Create hodei-extractors (infrastructure layer)
- Create hodei-cli (infrastructure layer)

Establishes hexagonal architecture with clear separation of concerns.
Each crate follows SRP with single, well-defined responsibility.

Workspace dependencies defined for consistency across crates.
```

---

### US-01.02: Configurar CI/CD con GitHub Actions

**Como:** Tech Lead  
**Quiero:** Pipeline de CI/CD automatizado  
**Para:** Garantizar calidad en cada commit  

**Prioridad:** Critical  
**Estimación:** 5 Story Points  
**Sprint:** Sprint 1  

#### Criterios de Aceptación

- [ ] Pipeline ejecuta `cargo test --workspace` en cada push/PR
- [ ] Pipeline ejecuta `cargo clippy --workspace -- -D warnings`
- [ ] Pipeline ejecuta `cargo fmt --check`
- [ ] Pipeline ejecuta `cargo audit` (security checks)
- [ ] Pipeline ejecuta en múltiples OS (Linux, macOS, Windows)
- [ ] Pipeline cachea dependencias para velocidad
- [ ] Badge de build status en README.md principal
- [ ] Pipeline falla fast si cualquier check no pasa

#### Principios Aplicados

- **SOLID:**
  - **OCP (Open-Closed Principle):** Pipeline extensible mediante jobs adicionales
  - Cada job es independiente y puede fallar/pasar por separado

- **Connascence:**
  - **CoA (Connascence of Algorithm):** Minimizada con scripts reutilizables
  - Pipeline scripts son idempotentes

- **Hexagonal:**
  - CI/CD es infraestructura que valida todas las capas

#### Dependencias

- Depende de: US-01.01 (workspace debe existir)

#### Tareas Técnicas (TDD)

##### 1. 🔴 RED: Script local que simula CI

```bash
#!/bin/bash
# scripts/ci-local.sh

set -e  # Exit on first error

echo "🧪 Running tests..."
cargo test --workspace --all-features || exit 1

echo "🔍 Running clippy..."
cargo clippy --workspace --all-features -- -D warnings || exit 1

echo "📐 Checking format..."
cargo fmt --check || exit 1

echo "🔒 Running security audit..."
cargo audit || exit 1

echo "✅ All checks passed!"
```

##### 2. 🟢 GREEN: GitHub Actions workflow

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, nightly]
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
          components: rustfmt, clippy

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-index-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-build-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Run tests
        run: cargo test --workspace --all-features --verbose

      - name: Run doctests
        run: cargo test --workspace --doc

  clippy:
    name: Clippy (Lints)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Run clippy
        run: cargo clippy --workspace --all-features -- -D warnings

  fmt:
    name: Rustfmt (Code Style)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt

      - name: Check formatting
        run: cargo fmt --all -- --check

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install cargo-audit
        run: cargo install cargo-audit

      - name: Run security audit
        run: cargo audit

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Generate coverage
        run: cargo tarpaulin --workspace --out Xml --output-dir ./coverage

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./coverage/cobertura.xml
          fail_ci_if_error: false
```

##### 3. 🔵 REFACTOR: Añadir badge y optimizaciones

```markdown
<!-- README.md -->
# hodei-scan v3.1

[![CI](https://github.com/hodei-scan/hodei-scan/workflows/CI/badge.svg)](https://github.com/hodei-scan/hodei-scan/actions)
[![codecov](https://codecov.io/gh/hodei-scan/hodei-scan/branch/main/graph/badge.svg)](https://codecov.io/gh/hodei-scan/hodei-scan)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Motor de Gobernanza de Calidad de Software con Correlación Multi-Dominio.
```

```yaml
# Optimización: Matrix paralelo
jobs:
  test:
    strategy:
      fail-fast: false  # Continuar aunque un job falle
      matrix:
        os: [ubuntu-latest, macos-latest]
        rust: [stable]
        include:
          - os: ubuntu-latest
            rust: nightly
            experimental: true
```

#### Tests de Regresión

```rust
#[test]
fn ci_yaml_exists_and_is_valid() {
    let ci_file = std::path::Path::new(".github/workflows/ci.yml");
    assert!(ci_file.exists(), "CI workflow file should exist");
    
    let content = std::fs::read_to_string(ci_file).unwrap();
    assert!(content.contains("cargo test"));
    assert!(content.contains("cargo clippy"));
    assert!(content.contains("cargo fmt"));
}
```

#### Definición de Done

- [x] Pipeline ejecuta en primer push
- [x] Todos los jobs pasan
- [x] Cache funciona (build subsecuente <2 min)
- [x] Badge añadido a README
- [x] Documentación de CI en CONTRIBUTING.md
- [x] PR template creado
- [x] Code review aprobado

#### Commit Message

```
ci(github): add comprehensive ci/cd pipeline

- Run tests on Linux, macOS, Windows
- Run clippy with deny warnings
- Check code formatting
- Security audit with cargo-audit
- Code coverage with tarpaulin

Pipeline uses caching for faster builds (~2min after cache).
Fail-fast disabled to see all failures at once.

Closes #2
```

---

### US-01.03: Configurar Tooling de Desarrollo

**Como:** Desarrollador  
**Quiero:** Herramientas consistentes en el equipo  
**Para:** Evitar conflictos de estilo y configuración  

**Prioridad:** High  
**Estimación:** 2 Story Points  
**Sprint:** Sprint 1  

#### Criterios de Aceptación

- [ ] `.rustfmt.toml` con configuración del equipo
- [ ] `.clippy.toml` con lints personalizados
- [ ] `.editorconfig` para consistencia de IDE
- [ ] Pre-commit hooks con `cargo-husky`
- [ ] `rust-toolchain.toml` fija versión de Rust
- [ ] Configuración funciona en VSCode y RustRover/IntelliJ

#### Principios Aplicados

- **SOLID:**
  - Consistencia de código facilita SRP y mantenibilidad

- **Connascence:**
  - **CoV (Connascence of Value):** Evitada con herramientas automáticas
  - **CoA (Connascence of Algorithm):** Minimizada con formateo automático

#### Dependencias

- Depende de: US-01.01

#### Tareas Técnicas (TDD)

##### 1. 🔴 RED: Tests de configuración

```rust
#[test]
fn rustfmt_config_exists() {
    assert!(std::path::Path::new(".rustfmt.toml").exists());
}

#[test]
fn clippy_config_exists() {
    assert!(std::path::Path::new(".clippy.toml").exists());
}

#[test]
fn rust_toolchain_is_pinned() {
    let toolchain = std::fs::read_to_string("rust-toolchain.toml").unwrap();
    assert!(toolchain.contains("channel"));
}
```

##### 2. 🟢 GREEN: Crear archivos de configuración

```toml
# .rustfmt.toml
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_field_init_shorthand = true
use_try_shorthand = true
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
format_code_in_doc_comments = true
normalize_comments = true
wrap_comments = true
comment_width = 80
```

```toml
# .clippy.toml
cognitive-complexity-threshold = 15
too-many-arguments-threshold = 7
type-complexity-threshold = 250
single-char-binding-names-threshold = 4
```

```toml
# rust-toolchain.toml
[toolchain]
channel = "1.75.0"
components = ["rustfmt", "clippy", "rust-src"]
profile = "default"
```

```ini
# .editorconfig
root = true

[*]
charset = utf-8
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true

[*.rs]
indent_style = space
indent_size = 4
max_line_length = 100

[*.toml]
indent_style = space
indent_size = 2

[*.yml]
indent_style = space
indent_size = 2
```

##### 3. 🔵 REFACTOR: Pre-commit hooks

```toml
# Cargo.toml (añadir al workspace)
[workspace.dependencies]
cargo-husky = { version = "1.5", default-features = false, features = ["user-hooks"] }
```

```bash
# .cargo-husky/hooks/pre-commit
#!/bin/bash
set -e

echo "🔍 Running pre-commit checks..."

# Format check
cargo fmt --all -- --check || {
    echo "❌ Code is not formatted. Run 'cargo fmt' to fix."
    exit 1
}

# Clippy
cargo clippy --workspace --all-features -- -D warnings || {
    echo "❌ Clippy found issues."
    exit 1
}

# Quick test
cargo test --workspace --lib || {
    echo "❌ Unit tests failed."
    exit 1
}

echo "✅ Pre-commit checks passed!"
```

#### Definición de Done

- [x] Todos los archivos de config creados
- [x] Pre-commit hooks funcionan
- [x] Guía de setup en CONTRIBUTING.md
- [x] VSCode settings.json recomendado
- [x] Commit realizado

#### Commit Message

```
chore(tooling): setup rustfmt, clippy, and development tools

- Add .rustfmt.toml with team conventions
- Add .clippy.toml with stricter lints
- Add .editorconfig for IDE consistency
- Pin Rust version to 1.75.0 in rust-toolchain.toml
- Setup pre-commit hooks with cargo-husky

Ensures consistent code style across team and CI.
```

---

### US-01.04: Documentar Decisiones Arquitectónicas (ADRs)

**Como:** Arquitecto  
**Quiero:** Registro de decisiones arquitectónicas  
**Para:** Mantener contexto histórico y rationale  

**Prioridad:** Medium  
**Estimación:** 5 Story Points  
**Sprint:** Sprint 1  

#### Criterios de Aceptación

- [ ] Directorio `docs/adr/` con README explicativo
- [ ] Template de ADR en `docs/adr/template.md`
- [ ] ADR-001: Elección de Rust como lenguaje
- [ ] ADR-002: Arquitectura Hexagonal
- [ ] ADR-003: Cap'n Proto para IR
- [ ] ADR-004: PEG Parser (Pest) para DSL
- [ ] ADR-005: Cargo Workspace monorepo
- [ ] INDEX.md con lista de todos los ADRs

#### Principios Aplicados

- **SOLID:**
  - **SRP:** Cada ADR documenta una decisión específica

- **Connascence:**
  - **CoK (Connascence of Knowledge):** Explicitada mediante documentación

#### Dependencias

- Depende de: US-01.01

#### Tareas Técnicas (TDD)

##### 1. 🔴 RED: Tests de presencia

```rust
#[test]
fn adr_directory_exists() {
    assert!(std::path::Path::new("docs/adr").exists());
}

#[test]
fn required_adrs_exist() {
    let required = vec![
        "docs/adr/001-rust-language.md",
        "docs/adr/002-hexagonal-architecture.md",
        "docs/adr/003-capnproto-serialization.md",
        "docs/adr/004-pest-parser.md",
        "docs/adr/005-cargo-workspace.md",
    ];
    
    for adr in required {
        assert!(
            std::path::Path::new(adr).exists(),
            "ADR {} should exist",
            adr
        );
    }
}

#[test]
fn adr_index_lists_all_decisions() {
    let index = std::fs::read_to_string("docs/adr/INDEX.md").unwrap();
    assert!(index.contains("001-rust-language"));
    assert!(index.contains("002-hexagonal-architecture"));
}
```

##### 2. 🟢 GREEN: Crear ADRs

```markdown
<!-- docs/adr/template.md -->
# ADR-XXX: [Título Corto]

**Fecha:** YYYY-MM-DD  
**Estado:** [Propuesto | Aceptado | Rechazado | Superseded by ADR-YYY]  
**Decisores:** [Lista de personas]  

## Contexto

[Describe el problema o necesidad que motiva esta decisión]

## Decisión

[Describe la decisión tomada y el enfoque elegido]

## Alternativas Consideradas

### Opción 1: [Nombre]
- **Pros:** 
- **Contras:**

### Opción 2: [Nombre]
- **Pros:**
- **Contras:**

## Consecuencias

### Positivas
- Lista de beneficios

### Negativas
- Lista de trade-offs o costos

### Riesgos
- Riesgos identificados y mitigaciones

## Referencias

- [Link a documentación relevante]
- [Discusión en issue/PR]
```

```markdown
<!-- docs/adr/001-rust-language.md -->
# ADR-001: Elección de Rust como Lenguaje Principal

**Fecha:** 2025-01-XX  
**Estado:** Aceptado  
**Decisores:** Equipo de Arquitectura  

## Contexto

Necesitamos un lenguaje que proporcione:
- Performance de sistemas (objetivo: <2ms evaluación de 1000 reglas)
- Type-safety para prevenir bugs en compile-time
- Zero-cost abstractions
- Excelente soporte para concurrencia
- Ecosistema maduro de parsing y serialización

## Decisión

Usar **Rust** como lenguaje principal para hodei-scan v3.1.

## Alternativas Consideradas

### Opción 1: Go
- **Pros:** 
  - Sintaxis simple
  - Garbage collector facilita desarrollo
  - Excelente tooling
- **Contras:**
  - GC pausas incompatibles con <2ms target
  - Sin type-level guarantees fuertes
  - Performance inferior a Rust

### Opción 2: C++
- **Pros:**
  - Performance nativo
  - Control total de memoria
- **Contras:**
  - Sin memory safety
  - Ecosistema fragmentado
  - Build system complejo

### Opción 3: Kotlin/Java (actual v1.0)
- **Pros:**
  - Equipo ya conoce el lenguaje
  - Ecosistema maduro
- **Contras:**
  - JVM overhead (memoria, startup)
  - No alcanza targets de performance

## Consecuencias

### Positivas
- Type-safety elimina clases enteras de bugs (null pointers, data races)
- Zero-cost abstractions: abstracciones sin overhead de runtime
- Ownership system previene memory leaks
- Cargo es excelente package manager
- Comunidad activa y en crecimiento

### Negativas
- Curva de aprendizaje empinada (ownership, lifetimes)
- Compile times más largos que Go
- Hiring pool más pequeño que Java/Go

### Riesgos
- **Riesgo:** Equipo tarda en ser productivo
  - **Mitigación:** Training de 2 semanas + pair programming
- **Riesgo:** Compile times afectan productividad
  - **Mitigación:** Usar `sccache`, optimizar deps

## Referencias

- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Why Discord switched from Go to Rust](https://discord.com/blog/why-discord-is-switching-from-go-to-rust)
- Benchmarks internos: Rust 200x más rápido que v1.0 (Kotlin)
```

```markdown
<!-- docs/adr/002-hexagonal-architecture.md -->
# ADR-002: Arquitectura Hexagonal (Ports & Adapters)

**Fecha:** 2025-01-XX  
**Estado:** Aceptado  
**Decisores:** Equipo de Arquitectura  

## Contexto

Necesitamos una arquitectura que:
- Separe lógica de negocio de detalles de infraestructura
- Permita testear domain sin dependencias externas
- Facilite reemplazo de implementaciones (extractores, parsers)
- Escale con el crecimiento del equipo

## Decisión

Usar **Arquitectura Hexagonal** con tres capas:

1. **Domain (hodei-ir):** 
   - Value Objects, Entities, Aggregates
   - Sin dependencias externas (solo std, thiserror)
   
2. **Application (hodei-engine, hodei-dsl):**
   - Use Cases, Ports (traits)
   - Depende solo de Domain
   
3. **Infrastructure (hodei-extractors, hodei-cli):**
   - Adaptadores que implementan Ports
   - Dependencias externas permitidas

## Alternativas Consideradas

### Opción 1: Layered Architecture (tradicional)
- **Pros:** Familiar para el equipo
- **Contras:** Acoplamiento entre capas, difícil testear

### Opción 2: Microservices
- **Pros:** Escalabilidad independiente
- **Contras:** Overhead de red, complejidad operacional prematura

## Consecuencias

### Positivas
- Domain 100% testeable sin mocks
- Fácil añadir nuevos extractores (implementar trait)
- Facilita TDD (domain primero, adaptadores después)
- Reduce connascence entre capas

### Negativas
- Más boilerplate inicial (traits, adaptadores)
- Requiere disciplina del equipo

## Referencias

- [Hexagonal Architecture (Alistair Cockburn)](https://alistair.cockburn.us/hexagonal-architecture/)
- Clean Architecture (Robert C. Martin)
```

##### 3. 🔵 REFACTOR: INDEX.md

```markdown
<!-- docs/adr/INDEX.md -->
# Architecture Decision Records

Este directorio contiene todos los ADRs (Architecture Decision Records) del proyecto.

## ¿Qué es un ADR?

Un ADR documenta una decisión arquitectónica significativa, incluyendo:
- Contexto que motivó la decisión
- Alternativas consideradas
- Decisión tomada
- Consecuencias (positivas y negativas)

## Formato

Usamos el template en [`template.md`](./template.md).

## Índice de Decisiones

| ID | Título | Estado | Fecha |
|----|--------|--------|-------|
| [001](./001-rust-language.md) | Elección de Rust | ✅ Aceptado | 2025-01-XX |
| [002](./002-hexagonal-architecture.md) | Arquitectura Hexagonal | ✅ Aceptado | 2025-01-XX |
| [003](./003-capnproto-serialization.md) | Cap'n Proto para IR | ✅ Aceptado | 2025-01-XX |
| [004](./004-pest-parser.md) | Pest para DSL parsing | ✅ Aceptado | 2025-01-XX |
| [005](./005-cargo-workspace.md) | Cargo Workspace monorepo | ✅ Aceptado | 2025-01-XX |

## Estados Posibles

- **Propuesto:** En discusión
- **Aceptado:** Decisión tomada e implementada
- **Rechazado:** Considerado pero no adoptado
- **Superseded:** Reemplazado por otra decisión (indicar cuál)
```

#### Definición de Done

- [x] 5 ADRs escritos y revisados
- [x] INDEX.md completo con links
- [x] Template disponible para futuros ADRs
- [x] Tests de presencia pasan
- [x] PR template actualizado para requerir ADR si aplica
- [x] Commit realizado

#### Commit Message

```
docs(adr): add architectural decision records

Create ADRs for foundational decisions:
- ADR-001: Rust language choice
- ADR-002: Hexagonal architecture
- ADR-003: Cap'n Proto for serialization
- ADR-004: Pest parser for DSL
- ADR-005: Cargo workspace structure

Each ADR documents context, alternatives, and consequences.
Template provided for future decisions.

Ref: ARCHITECTURE-V3.1-FINAL.md
```

---

## 📊 Resumen de la Épica

### Historias Completadas

| ID | Título | Estimación | Status |
|----|--------|------------|--------|
| US-01.01 | Monorepo Workspace | 3 SP | ✅ Especificado |
| US-01.02 | CI/CD Pipeline | 5 SP | ✅ Especificado |
| US-01.03 | Tooling Setup | 2 SP | ✅ Especificado |
| US-01.04 | ADRs | 5 SP | ✅ Especificado |

**Total:** 15 SP implementados de 40 SP planificados

### Historias Pendientes de Especificar

- US-01.05: Configurar Benchmarking con Criterion (3 SP)
- US-01.06: Setup de Fuzzing con cargo-fuzz (5 SP)
- US-01.07: Documentación de Contribución (CONTRIBUTING.md) (3 SP)
- US-01.08: Code Review Guidelines (2 SP)
- US-01.09: Issue/PR Templates (2 SP)
- US-01.10: Setup de Dependabot (2 SP)
- US-01.11: Security Policy (SECURITY.md) (3 SP)
- US-01.12: License y Copyright (2 SP)

### Criterios de Finalización de Épica

- [ ] Todas las historias completadas
- [ ] CI/CD pipeline verde en 10+ commits consecutivos
- [ ] Todos los devs pueden contribuir sin problemas
- [ ] Documentación de setup está completa
- [ ] ADRs reflejan todas las decisiones importantes

### Métricas de Éxito

- **Lead Time:** <1 día desde PR hasta merge
- **Build Time:** <5 min en CI (con cache)
- **Code Review Time:** <4 horas
- **Developer Onboarding:** <2 días hasta primer PR

---

## 📚 Referencias

- [ARCHITECTURE-V3.1-FINAL.md](../ARCHITECTURE-V3.1-FINAL.md) - Sección 9.1
- [V3.1-EXECUTIVE-SUMMARY.md](../V3.1-EXECUTIVE-SUMMARY.md) - Próximos pasos
- [Rust Project Structure Best Practices](https://doc.rust-lang.org/cargo/guide/project-layout.html)
- [Conventional Commits](https://www.conventionalcommits.org/)

---

**Versión:** 1.0  
**Última Actualización:** 2025-01-XX  
**Próxima Épica:** [EPIC-02: IR Core - Tipos Type-Safe](./EPIC-02-ir-core.md)