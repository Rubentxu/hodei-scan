# EPIC-14: Documentation (User & Developer)

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: Hodei Scan v3.2  
**Dependencias**: Todas las épicas técnicas  
**Owner**: Documentation Team  
**Prioridad**: High

---

## 1. Resumen Ejecutivo

Documentación completa para usuarios y desarrolladores: guías, tutoriales, API reference, arquitectura, contributing guide.

### Objetivo
- Onboarding <30 minutos para usuarios.
- Docs API al 100%.
- Guías para custom extractors y reglas.

---

## 2. Estructura de Documentación

```
docs/
├── README.md                       # Landing page
├── user-guide/
│   ├── getting-started.md
│   ├── installation.md
│   ├── basic-usage.md
│   ├── writing-rules.md
│   ├── quality-gates.md
│   ├── ci-cd-integration.md
│   └── troubleshooting.md
├── developer-guide/
│   ├── architecture.md             # ARCHITECTURE-V3.2-FINAL.md
│   ├── contributing.md
│   ├── building-from-source.md
│   ├── custom-extractors.md
│   ├── plugin-development.md
│   └── testing.md
├── reference/
│   ├── cli-reference.md
│   ├── dsl-reference.md
│   ├── ir-schema.md
│   ├── api/                        # rustdoc output
│   └── configuration.md
├── tutorials/
│   ├── first-scan.md
│   ├── writing-your-first-rule.md
│   ├── creating-custom-extractor.md
│   └── integrating-with-github-actions.md
├── epics/                          # Epic documents (EPIC-01..20)
├── decisions/                      # ADRs
└── CHANGELOG.md
```

---

## 3. Documentation Components

### 3.1. User Guide

#### Getting Started
```markdown
# Getting Started with Hodei Scan

## What is Hodei Scan?

Hodei Scan is a unified security and quality scanner that correlates:
- **Taint analysis** (vulnerabilities)
- **Code coverage** (quality)
- **Dependencies** (supply chain)

to find high-priority issues like "vulnerable code with no tests".

## Quick Start

### 1. Install

```bash
curl -sSL https://hodei.dev/install.sh | bash
```

### 2. Run your first scan

```bash
cd /path/to/your/project
hodei check
```

### 3. View results

Hodei will:
- Extract facts from your codebase
- Analyze with built-in rules
- Apply quality gates
- Report findings

Example output:
```
🔍 Extracting facts...
✅ Extraction complete (12,543 facts)

🧠 Analyzing with rules...
✅ Analysis complete (23 findings)

🚦 Evaluating quality gates...
❌ Gate 'Security Critical' failed:
   - Expected FindingsBySeverity(Critical) Equal 0, got 3

📊 Summary:
  🔴 Critical: 3
  🟠 High: 8
  🟡 Medium: 10
  🟢 Low: 2
```

## Next Steps

- [Write custom rules](writing-rules.md)
- [Configure quality gates](quality-gates.md)
- [Integrate with CI/CD](ci-cd-integration.md)
```

#### Writing Rules
```markdown
# Writing Rules

## Rule Anatomy

```hodei
rule RuleName {
    description: "Human-readable description"
    severity: Critical | High | Medium | Low | Info
    tags: ["security", "category"]
    
    match {
        // Pattern matching on facts
        pattern1: FactType and
        pattern2: AnotherFactType
        
        // Optional where clause
        where pattern1.location == pattern2.location
    }
    
    emit Finding {
        message: "Template with {variable}"
        confidence: High | Medium | Low
        metadata: { key: "value" }
    }
}
```

## Example: Vulnerable Uncovered Code

```hodei
rule VulnerableUncoveredCode {
    description: "Taint sink in line without test coverage"
    severity: Critical
    tags: ["security", "coverage"]
    
    match {
        sink: TaintSink and
        uncovered: UncoveredLine
        
        where sink.location == uncovered.location
    }
    
    emit Finding {
        message: "Vulnerable code at {sink.location.file}:{sink.location.start.line} has no tests"
        confidence: High
        metadata: { 
            remediation: "Add test coverage for this code path",
            cwe: "CWE-1004"
        }
    }
}
```

## Built-in FactTypes

- `TaintSource` - User input, untrusted data
- `TaintSink` - Security-sensitive operations (SQL, exec, etc.)
- `DataFlow` - Data flow edge
- `Vulnerability` - Known vulnerability
- `UncoveredLine` - Line without test coverage
- `CoveredLine` - Line with test coverage
- `Dependency` - External dependency
- ... [see full list](../reference/ir-schema.md)

## Built-in Functions

### `reachable(source, sink)`
Check if data can flow from source to sink.

```hodei
rule TaintFlow {
    match {
        source: TaintSource and
        sink: TaintSink
        
        where reachable(source, sink)
    }
    
    emit Finding {
        message: "Data flows from {source.location} to {sink.location}"
        confidence: Medium
    }
}
```

### `distance(loc1, loc2)`
Calculate distance in lines between two locations.

## Testing Rules

```bash
# Validate rule syntax
hodei validate-rules .hodei/rules

# Test rule on fixture
hodei analyze \
    --ir fixtures/test_app.ir \
    --rules .hodei/rules/my-rule.hodei
```
```

### 3.2. Developer Guide

#### Custom Extractors
```markdown
# Creating Custom Extractors

## Extractor Trait

```rust
use hodei_extractors::{Extractor, ExtractionContext, ExtractorError};
use hodei_ir::{Fact, FactType};
use async_trait::async_trait;

pub struct MyExtractor;

#[async_trait]
impl Extractor for MyExtractor {
    fn name(&self) -> &str {
        "my-extractor"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn extract(&self, ctx: &ExtractionContext) -> Result<Vec<Fact>, ExtractorError> {
        let mut facts = Vec::new();
        
        // Your extraction logic here
        for file in walk_files(&ctx.project_root)? {
            // Analyze file
            let analysis_result = analyze_file(&file)?;
            
            // Emit facts
            for result in analysis_result {
                facts.push(Fact {
                    fact_type: FactType::Vulnerability {
                        vuln_type: result.vuln_type,
                        severity: result.severity,
                        description: result.description,
                    },
                    source_location: Some(result.location),
                    confidence: Confidence::new(0.8)?,
                    provenance: Provenance {
                        extractor: self.name().to_string(),
                        version: self.version().to_string(),
                        timestamp: Utc::now(),
                    },
                    ..Default::default()
                });
            }
        }
        
        Ok(facts)
    }
}
```

## Registration

```rust
// In your CLI or plugin loader
let mut runner = ExtractorRunner::new();
runner.register(Box::new(MyExtractor));
```

## Testing

```rust
#[tokio::test]
async fn test_my_extractor() {
    let extractor = MyExtractor;
    let ctx = create_test_context("fixtures/test_project");
    
    let facts = extractor.extract(&ctx).await.unwrap();
    
    assert!(facts.len() > 0);
    assert!(facts.iter().any(|f| matches!(f.fact_type, FactType::Vulnerability { .. })));
}
```

## Best Practices

1. **Atomic Facts Only** - Never emit correlations (e.g., "VulnerableUncovered"). Emit atomic observations; let the rule engine correlate.

2. **Accurate Provenance** - Always set correct `provenance` with extractor name, version, timestamp.

3. **Confidence Scores** - Use appropriate confidence:
   - 1.0: Exact data (e.g., coverage report)
   - 0.8-0.9: High confidence heuristics
   - 0.5-0.7: Medium confidence patterns
   - 0.3-0.5: Low confidence speculation

4. **Error Handling** - Return `ExtractorError` for failures; don't panic.

5. **Performance** - Use async/await for I/O; process files in parallel when possible.
```

### 3.3. API Reference

```markdown
# API Reference

Generated from rustdoc:

- [hodei-ir](api/hodei_ir/index.html)
- [hodei-engine](api/hodei_engine/index.html)
- [hodei-dsl](api/hodei_dsl/index.html)
- [hodei-extractors](api/hodei_extractors/index.html)

## Key Types

### `Fact`
Atomic observation from an extractor.

### `FactType`
Enum of all possible fact types (17 variants).

### `IntermediateRepresentation`
Container for all facts from a scan.

### `Finding`
Correlated result from rule evaluation.

### `RuleEngine`
Evaluator that applies rules to IR and generates findings.

[Full API docs](api/index.html)
```

---

## 4. Documentation Generation

### 4.1. API Docs (rustdoc)

```bash
# Generate rustdoc
cargo doc --no-deps --all-features

# Serve locally
cargo doc --no-deps --all-features --open
```

### 4.2. Book (mdBook)

```toml
# book.toml
[book]
title = "Hodei Scan Documentation"
authors = ["Hodei Team"]
language = "en"
multilingual = false
src = "docs"

[build]
build-dir = "book"

[output.html]
default-theme = "light"
git-repository-url = "https://github.com/hodei-team/hodei-scan"
```

```bash
# Install mdBook
cargo install mdbook

# Serve book locally
mdbook serve

# Build for deployment
mdbook build
```

---

## 5. Plan de Implementación

**Fase 1: User Guide** (Semana 1)
- [ ] Getting Started
- [ ] Installation
- [ ] Basic Usage
- [ ] Writing Rules

**Fase 2: Developer Guide** (Semana 2)
- [ ] Architecture (usar ARCHITECTURE-V3.2)
- [ ] Contributing
- [ ] Custom Extractors
- [ ] Plugin Development

**Fase 3: Reference** (Semana 2)
- [ ] CLI Reference
- [ ] DSL Reference
- [ ] IR Schema
- [ ] Configuration

**Fase 4: Deployment** (Semana 3)
- [ ] Setup mdBook
- [ ] Generate API docs
- [ ] Deploy to GitHub Pages
- [ ] Setup docs versioning

---

## 6. Criterios de Aceptación

- [ ] User guide completa.
- [ ] Developer guide completa.
- [ ] API reference 100%.
- [ ] mdBook deployed.
- [ ] Searchable docs.

---

**Última Actualización**: 2025-01-XX
