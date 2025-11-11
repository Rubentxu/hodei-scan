# EPIC-14: Developer Experience Tools - LSP, Testing & Debug

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: hodei-scan v3.3  
**Dependencias**: EPIC-05 (DSL Parser), EPIC-06 (Rule Engine)  
**Owner**: Developer Experience Team  
**Prioridad**: Medium Path

---

## 1. Resumen Ejecutivo

Crear un **ecosistema de herramientas de desarrollo** que haga que la creación, testing y debugging de reglas hodei-scan sea **tan fácil como escribir código**. Esta épica incluye Language Server Protocol (LSP), framework de tests de reglas, y herramientas de debug del IR.

### Objetivo de Negocio
Reducir la **curva de adopción** del potente DSL de correlación de hodei-scan de semanas a **horas**, fomentando una **comunidad activa** de desarrolladores de reglas.

### Métricas de Éxito
- **Usabilidad**: <5 min para crear nueva regla funcional
- **Testing**: 90% de reglas tienen test coverage
- **Debug**: 100% de reglas debuggeables interactivamente
- **Adopción**: 50+ desarrolladores contribuyen reglas en 6 meses

---

## 2. Contexto Técnico

### 2.1. Problema Actual
Desarrollar reglas hodei-scan es **complejo**:
- No hay autocompletado en IDEs
- No hay tests unitarios para reglas
- No hay herramientas de debug del IR
- DSL learning curve es empinada
- Sin feedback en tiempo real

### 2.2. Solución: DX Toolchain

```
┌─────────────────────────────────────────────────────────────────┐
│                   Developer Experience Stack                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────┐    ┌──────────────────┐    ┌────────────┐ │
│  │   hodei-dsl-lsp  │    │   hodei-test     │    │  ir-dump   │ │
│  │   (Language      │    │   (Rule Testing  │    │  (IR Debug │ │
│  │   Server)        │    │   Framework)     │    │  Tool)     │ │
│  │                  │    │                  │    │            │ │
│  │ • Autocompletado │    │ • Unit tests     │    │ • JSON     │ │
│  │ • Validation     │    │ • Snapshots      │    │ • YAML     │ │
│  │ • Hover docs     │    │ • Assertions     │    │ • Visual   │ │
│  │ • Diagnostics    │    │ • CI integration │    │ • Filter   │ │
│  └──────────┬───────┘    └────────┬─────────┘    └─────┬──────┘ │
│             │                     │                      │       │
│             └─────────────┬───────────────────┬──────────┘       │
│                           ▼                   ▼                   │
│                  ┌────────────────────────────────┐              │
│                  │        VS Code Extension        │              │
│                  │   (hodei-scan-dsl-support)      │              │
│                  └────────────────────────────────┘              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. Arquitectura Detallada

### 3.1. hodei-dsl-lsp (Language Server Protocol)

#### Componentes
```rust
pub struct HodeiDslServer {
    parser: HodeiParser,
    analyzer: SemanticAnalyzer,
    completions: CompletionProvider,
    hover_provider: HoverProvider,
}

impl LanguageServer for HodeiDslServer {
    async fn completion(&self, params: CompletionParams) -> Result<CompletionResponse> {
        let document = self.get_document(params.text_document_position.text_document.uri)?;
        let position = params.text_document_position.position;
        
        let completions = self.completions.provide(&document, position)?;
        Ok(CompletionResponse::Array(completions))
    }
    
    async fn hover(&self, params: HoverParams) -> Result<HoverResponse> {
        let document = self.get_document(params.text_document_position.text_document.uri)?;
        let position = params.text_document_position.position;
        
        let hover_info = self.hover_provider.provide(&document, position)?;
        Ok(HoverResponse::Scalar(hover_info))
    }
}
```

#### Features Clave

**1. Autocompletado Inteligente:**
```typescript
// Cuando usuario escribe: fact.type.
const completions = [
  {
    label: "Vulnerability",
    kind: CompletionItemKind.Class,
    detail: "Security vulnerability fact",
    documentation: "Represents a security vulnerability in the code...",
    insertText: "Vulnerability { ${1:smell_type: String}, ${2:severity: Severity} }"
  },
  {
    label: "CodeSmell", 
    kind: CompletionItemKind.Class,
    detail: "Code quality issue"
  }
];
```

**2. Semantic Validation:**
```rust
pub struct SemanticAnalyzer {
    fact_registry: FactTypeRegistry,
    rule_registry: RuleRegistry,
}

impl SemanticAnalyzer {
    pub fn validate(&self, ast: &RuleFile) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();
        
        for rule in &ast.rules {
            // Check fact type exists
            if !self.fact_registry.exists(&rule.fact_type) {
                diagnostics.push(Diagnostic::Error(
                    rule.span,
                    format!("Unknown fact type: {}", rule.fact_type)
                ));
            }
            
            // Check pattern syntax
            if let Some(pattern) = &rule.pattern {
                if let Err(e) = self.validate_pattern(pattern) {
                    diagnostics.push(Diagnostic::Warning(rule.span, e));
                }
            }
        }
        
        diagnostics
    }
}
```

**3. Hover Documentation:**
```rust
pub struct HoverProvider {
    fact_docs: HashMap<String, FactDocumentation>,
    function_docs: HashMap<String, FunctionDocumentation>,
}

impl HoverProvider {
    pub fn provide_hover(
        &self,
        document: &Document,
        position: Position,
    ) -> Option<HoverInfo> {
        let token = document.get_token_at(position)?;
        
        match token.kind {
            TokenKind::FactType => {
                self.fact_docs.get(&token.text).map(|doc| HoverInfo {
                    contents: format!("# {}\n\n{}", doc.name, doc.description),
                    range: token.range,
                })
            }
            TokenKind::Function => {
                self.function_docs.get(&token.text).map(|doc| HoverInfo {
                    contents: format!("# {}\n\nUsage: {}\n\n{}", doc.name, doc.usage, doc.description),
                    range: token.range,
                })
            }
            _ => None,
        }
    }
}
```

### 3.2. hodei-test (Rule Testing Framework)

#### Test Format
```yaml
# test_password_rules.hodei.test
rule: "password_strength.hodei"
language: "hodei-dsl"

cases:
  - name: "Strong password"
    code: |
      function validatePassword(pwd: string): boolean {
        if (pwd.length >= 12 && pwd.matches(/[A-Z]/) && pwd.matches(/[0-9]/)) {
          return true;
        }
        return false;
      }
    expected_findings: []

  - name: "Weak password - too short"
    code: |
      function validatePassword(pwd: string): boolean {
        return pwd.length >= 8;  // Too short!
      }
    expected_findings:
      - type: "CodeSmell"
        severity: "Major"
        message: "Password too weak"

  - name: "Weak password - no numbers"
    code: |
      function validatePassword(pwd: string): boolean {
        return pwd.length >= 12;  // Missing numbers!
      }
    expected_findings:
      - type: "CodeSmell"
        severity: "Major"
        message: "Password missing numbers"
```

#### Test Runner
```rust
pub struct HodeiTestRunner {
    compiler: RuleCompiler,
    extractor: IRExtractor,
    rule_engine: RuleEngine,
}

impl HodeiTestRunner {
    pub async fn run_test_file(&self, test_file: &Path) -> Result<TestResults> {
        let test_config: TestConfig = self.load_test_config(test_file)?;
        
        let mut results = TestResults::new();
        
        for test_case in &test_config.cases {
            let result = self.run_single_test(test_case).await?;
            results.add_result(result);
        }
        
        Ok(results)
    }
    
    async fn run_single_test(&self, test_case: &TestCase) -> Result<TestCaseResult> {
        // 1. Extract IR from code snippet
        let ir = self.extractor.extract_from_snippet(&test_case.code)?;
        
        // 2. Compile rule
        let rule = self.compiler.compile_rule(&test_config.rule)?;
        
        // 3. Run rule against IR
        let findings = self.rule_engine.evaluate(&rule, &ir)?;
        
        // 4. Assert findings match expected
        let assertions = self.assert_findings(&findings, &test_case.expected_findings)?;
        
        Ok(TestCaseResult {
            name: test_case.name.clone(),
            passed: assertions.all_passed(),
            assertions,
        })
    }
}
```

#### Snapshot Testing
```rust
pub struct SnapshotManager {
    snapshot_dir: PathBuf,
}

impl SnapshotManager {
    pub fn update_snapshots(&self, test_results: &TestResults) -> Result<()> {
        for (test_name, snapshot) in test_results.snapshots() {
            let snapshot_path = self.snapshot_dir.join(format!("{}.snap", test_name));
            self.write_snapshot(&snapshot_path, snapshot)?;
        }
        Ok(())
    }
    
    pub fn verify_snapshots(&self, test_results: &TestResults) -> Result<Vec<SnapshotDiff>> {
        let mut diffs = Vec::new();
        
        for (test_name, actual) in test_results.snapshots() {
            let snapshot_path = self.snapshot_dir.join(format!("{}.snap", test_name));
            
            if let Ok(expected) = self.read_snapshot(&snapshot_path) {
                let diff = self.compare_snapshots(&expected, &actual)?;
                if !diff.is_empty() {
                    diffs.push(diff);
                }
            } else {
                // No snapshot exists - create one
                self.write_snapshot(&snapshot_path, &actual)?;
            }
        }
        
        Ok(diffs)
    }
}
```

### 3.3. ir-dump (IR Debug Tool)

#### CLI Interface
```bash
# Convert IR to JSON
hodei-scan ir-dump --input facts.capnp --format json

# Convert IR to YAML
hodei-scan ir-dump --input facts.capnp --format yaml

# Filter IR by fact type
hodei-scan ir-dump --input facts.capnp --filter "type=Vulnerability"

# Interactive explorer
hodei-scan ir-dump --input facts.capnp --interactive

# Compare two IRs
hodei-scan ir-dump --input-1 facts_v1.capnp --input-2 facts_v2.capnp --diff
```

#### Implementation
```rust
pub struct IrDumper {
    ir_reader: IRReader,
    formatter: IRFormatter,
    filter: IRFilter,
}

impl IrDumper {
    pub async fn dump(&self, input: &Path, format: &Format, filters: &[Filter]) -> Result<String> {
        // 1. Read IR
        let ir = self.ir_reader.read(input).await?;
        
        // 2. Apply filters
        let filtered_ir = self.filter.apply(&ir, filters)?;
        
        // 3. Format output
        let output = self.formatter.format(&filtered_ir, format)?;
        
        Ok(output)
    }
}

pub struct IRFormatter;

impl IRFormatter {
    pub fn format_json(&self, ir: &IR) -> Result<String> {
        let json = serde_json::to_string_pretty(ir)?;
        Ok(json)
    }
    
    pub fn format_yaml(&self, ir: &IR) -> Result<String> {
        let yaml = serde_yaml::to_string(ir)?;
        Ok(yaml)
    }
    
    pub fn format_visual(&self, ir: &IR) -> Result<String> {
        // ASCII tree visualization
        let mut output = String::new();
        output.push_str("IR Structure:\n");
        
        for fact in &ir.facts {
            output.push_str(&format!(
                "├── [{}] {} at {}\n",
                fact.fact_type,
                fact.message,
                fact.location
            ));
        }
        
        Ok(output)
    }
}
```

#### Interactive Explorer
```rust
pub struct InteractiveExplorer {
    ir: IR,
    current_fact: usize,
}

impl InteractiveExplorer {
    pub fn start(&mut self) -> Result<()> {
        println!("hodei-scan IR Explorer - Type 'help' for commands\n");
        
        loop {
            let input = self.read_command()?;
            
            match input.as_str() {
                "next" => self.next_fact(),
                "prev" => self.prev_fact(),
                "show" => self.show_current_fact(),
                "filter" => self.apply_filter(),
                "quit" => break,
                _ => self.show_help(),
            }
        }
        
        Ok(())
    }
    
    fn show_current_fact(&self) {
        if let Some(fact) = self.ir.facts.get(self.current_fact) {
            println!("Fact #{}: {}\n", self.current_fact + 1, fact);
            println!("{}", serde_json::to_string_pretty(fact).unwrap());
        }
    }
}
```

---

## 4. Plan de Implementación

### 4.1. Fases

**Fase 1: LSP Core (Semana 1-2)**
- [ ] Implementar Language Server Protocol
- [ ] Basic autocompletion
- [ ] Syntax validation
- [ ] VS Code extension skeleton

**Fase 2: Testing Framework (Semana 3)**
- [ ] Test file format (YAML)
- [ ] Test runner implementation
- [ ] Snapshot testing
- [ ] CI integration

**Fase 3: Debug Tools (Semana 4)**
- [ ] ir-dump CLI
- [ ] JSON/YAML formatters
- [ ] Filter system
- [ ] Interactive explorer

---

## 5. User Stories

### US-14.01: hodei-dsl Language Server (LSP)

**Como:** Developer escribiendo reglas  
**Quiero:** Autocompletado y validación en tiempo real en mi IDE  
**Para:** Escribir reglas más rápido y con menos errores

**Criterios de Aceptación:**
- [ ] LSP server implementa protocolo completo
- [ ] Autocompletado de FactTypes y sus campos
- [ ] Hover documentation para built-in functions
- [ ] Real-time syntax validation
- [ ] VS Code extension published
- [ ] Error squiggles en editor

**TDD - Red:**
```rust
#[test]
fn test_completion_provides_fact_types() {
    let server = HodeiDslServer::new();
    let document = "fact.type.".to_string();
    let position = Position::new(0, 11);
    
    let completions = server.completion(&document, position).unwrap();
    
    assert!(completions.iter().any(|c| c.label == "Vulnerability"));
    assert!(completions.iter().any(|c| c.label == "CodeSmell"));
}
```

**TDD - Green:**
```rust
impl CompletionProvider {
    pub fn provide(&self, document: &str, position: Position) -> Vec<CompletionItem> {
        let offset = document[..position.character].rfind('.').unwrap_or(0);
        let context = &document[offset..];
        
        match context {
            "fact.type." => self.fact_type_completions(),
            "fact.location." => self.location_field_completions(),
            _ => Vec::new(),
        }
    }
}
```

**Conventional Commit:**
`feat(lsp): implement hodei-dsl language server with autocompletion`

---

### US-14.02: Rule Testing Framework

**Como:** Rule Developer  
**Quiero:** Escribir tests unitarios para mis reglas  
**Para:** Asegurar que las reglas son precisas y no tienen falsos positivos

**Criterios de Aceptación:**
- [ ] YAML test file format
- [ ] Test runner (hodei-scan test-rule)
- [ ] Snapshot testing support
- [ ] CI integration
- [ ] Test coverage reporting

**TDD - Red:**
```rust
#[tokio::test]
async fn test_rule_testing() {
    let test_file = Path::new("tests/password_strength.hodei.test");
    let runner = HodeiTestRunner::new();
    
    let results = runner.run_test_file(test_file).await.unwrap();
    
    assert_eq!(results.total_tests, 3);
    assert_eq!(results.passed_tests, 2);
    assert_eq!(results.failed_tests, 1);
}
```

**TDD - Green:**
```rust
pub async fn run_test_file(&self, test_file: &Path) -> Result<TestResults> {
    let test_config = self.parse_test_file(test_file)?;
    let mut results = TestResults::new();
    
    for case in &test_config.cases {
        let result = self.run_single_test(case).await?;
        results.add_case_result(result);
    }
    
    Ok(results)
}
```

**Conventional Commit:**
`feat(test): implement rule testing framework with YAML cases`

---

### US-14.03: IR Debug Tools

**Como:** Rule Developer  
**Quiero:** Inspeccionar IR en formato legible  
**Para:** Debugging cuando reglas no funcionan como esperado

**Criterios de Aceptación:**
- [ ] ir-dump CLI command
- [ ] JSON/YAML output formats
- [ ] Filter by fact type, severity, location
- [ ] Interactive explorer mode
- [ ] IR diff between two snapshots

**TDD - Red:**
```rust
#[test]
fn test_ir_dump_json() {
    let dumper = IrDumper::new();
    let ir_path = Path::new("facts.capnp");
    
    let output = dumper.dump(ir_path, &Format::Json, &[]).unwrap();
    
    assert!(output.starts_with("{"));
    assert!(output.contains("\"facts\""));
}
```

**TDD - Green:**
```rust
impl IrDumper {
    pub fn dump(&self, input: &Path, format: &Format, filters: &[Filter]) -> Result<String> {
        let ir = self.read_ir(input)?;
        let filtered = self.apply_filters(&ir, filters)?;
        
        match format {
            Format::Json => self.format_json(&filtered),
            Format::Yaml => self.format_yaml(&filtered),
        }
    }
}
```

**Conventional Commit:**
`feat(debug): implement ir-dump tool for IR inspection`

---

### US-14.04: VS Code Extension

**Como:** Developer  
**Quiero:** Instalar extensión para soporte completo del DSL en VS Code  
**Para:** Tener la mejor experiencia de desarrollo posible

**Criterios de Aceptación:**
- [ ] Extension published en marketplace
- [ ] Language configuration para .hodei files
- [ ] Integración con LSP server
- [ ] Commands palette integration
- [ ] Snippets para patterns comunes
- [ ] Syntax highlighting

**VS Code Extension Structure:**
```typescript
// package.json
{
  "name": "hodei-scan-dsl-support",
  "publisher": "hodei-scan",
  "engines": {
    "vscode": "^1.74.0"
  },
  "activationEvents": [
    "onLanguage:hodei-dsl"
  ],
  "contributes": {
    "languages": [
      {
        "id": "hodei-dsl",
        "extensions": [".hodei"],
        "configuration": "./language-configuration.json"
      }
    ],
    "commands": [
      {
        "command": "hodei.testRule",
        "title": "hodei-scan: Test Rule"
      },
      {
        "command": "hodei.dumpIR",
        "title": "hodei-scan: Dump IR"
      }
    ]
  }
}
```

**Conventional Commit:**
`feat(vscode): publish VS Code extension for hodei-dsl`

---

### US-14.05: CI Integration

**Como:** DevOps Engineer  
**Quiero:** Integrar testing de reglas en CI/CD  
**Para:** Prevenir regressions en reglas

**CI Workflow:**
```yaml
# .github/workflows/rules-test.yml
name: Test hodei-scan Rules

on: [push, pull_request]

jobs:
  test-rules:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install hodei-scan
        run: cargo install --path crates/hodei-cli
      
      - name: Run Rule Tests
        run: |
          hodei-scan test-rule --all --coverage-report
          
      - name: Upload Coverage
        uses: codecov/codecov-action@v3
        with:
          file: coverage/lcov.info
```

**Conventional Commit:**
`feat(ci): add rule testing to CI pipeline`

---

## 6. Testing Strategy

### 6.1. Unit Tests
- LSP protocol compliance
- Completion provider accuracy
- Test runner logic
- IR formatter correctness

### 6.2. Integration Tests
- VS Code extension + LSP communication
- End-to-end rule test workflow
- ir-dump with real IR files

### 6.3. User Acceptance Tests
- Developer onboarding <30 min
- Create first rule <5 min
- Write first test <3 min

---

## 7. Riesgos y Mitigaciones

| Riesgo | Impacto | Probabilidad | Mitigación |
|--------|---------|--------------|------------|
| LSP protocol complexity | Medio | Medio | Use existing libraries (lsp-types) |
| VS Code extension maintenance | Alto | Bajo | Community-driven extension |
| Performance overhead | Medio | Medio | Lazy loading + caching |
| Cross-editor support | Alto | Alto | Support LSP clients generically |

---

## 8. Definition of Done

- [ ] LSP server fully functional
- [ ] VS Code extension published
- [ ] Testing framework integrated in CI
- [ ] ir-dump tool documented
- [ ] All developer docs written
- [ ] 50+ community developers adopt tools

---

**Estimación Total**: 4 Sprints (8 semanas)  
**Commit Messages**:  
- `feat(lsp): implement language server protocol`  
- `feat(test): implement rule testing framework`  
- `feat(debug): implement ir-dump tool`  
- `feat(vscode): publish VS Code extension`  
- `feat(ci): integrate testing in CI pipeline`  

---

**Referencias Técnicas**:
- Language Server Protocol: https://microsoft.github.io/language-server-protocol/
- lsp-types (Rust): https://github.com/gluon-lang/lsp-types
- VS Code Extension API: https://code.visualstudio.com/api
- Clap (CLI): https://clap.rs/
