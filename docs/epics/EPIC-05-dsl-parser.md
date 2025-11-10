# EPIC-05: DSL Parser (Cedar-like Rule Language)

**Estado**: 📝 Draft  
**Versión**: 1.0  
**Épica padre**: Hodei Scan v3.2  
**Dependencias**: EPIC-02 (IR Core)  
**Owner**: DSL Team  
**Prioridad**: Critical Path

---

## 1. Resumen Ejecutivo

Implementar un **parser formal** para el DSL de reglas de Hodei Scan, usando una gramática PEG (Parsing Expression Grammar) con el crate `pest`. El DSL es declarativo, tipo-safe, inspirado en Cedar (AWS) y diseñado para ser seguro contra inyección.

### Objetivo de Negocio
Permitir que usuarios (security engineers, SREs) escriban reglas personalizadas para detectar vulnerabilidades, deuda técnica y correlaciones calidad-seguridad **sin código**.

### Métricas de Éxito
- **Seguridad**: 0 vulnerabilidades de inyección (gramática formal, sin eval).
- **Usabilidad**: Parsing de reglas simples en <100μs.
- **Expresividad**: Soporte para >90% de casos de uso (spatial joins, flow queries, aggregations).
- **Ergonomía**: Mensajes de error claros (con spans y sugerencias).

---

## 2. Contexto Técnico

### 2.1. Problema
Los usuarios necesitan expresar reglas como:
```
rule VulnerableUncoveredCode {
  description: "Taint sink in uncovered line"
  severity: Critical
  
  match {
    sink: TaintSink and
    uncovered: UncoveredLine
    
    where sink.location == uncovered.location
  }
  
  emit Finding {
    message: "Vulnerable code at {sink.location} has no tests"
    confidence: High
  }
}
```

Sin un parser formal, tendríamos que:
- Usar regex (frágil, inseguro).
- Evaluar código arbitrario (inyección de código).
- Implementar un parser ad-hoc (bug-prone).

### 2.2. Solución: Gramática PEG + pest
- **Gramática formal** (rules.pest) define la sintaxis completa.
- **pest** genera parser seguro con error recovery.
- **AST tipado** (Abstract Syntax Tree) valida semántica.
- **Type checker** verifica tipos antes de evaluación.

### 2.3. Ejemplo de Gramática (Simplificado)
```pest
// rules.pest
rule_def = { "rule" ~ IDENT ~ "{" ~ rule_body ~ "}" }
rule_body = { metadata ~ match_block ~ emit_block }

match_block = { "match" ~ "{" ~ pattern+ ~ where_clause? ~ "}" }
pattern = { IDENT ~ ":" ~ fact_type ~ condition? }
where_clause = { "where" ~ expr }

expr = { term ~ (binary_op ~ term)* }
term = { literal | path | function_call }
path = { IDENT ~ ("." ~ IDENT)* }
```

---

## 3. Alcance

### 3.1. En Alcance (MUST)
1. **Gramática PEG completa** en `rules.pest`.
2. **Parser** con pest que produce AST.
3. **AST Types**: RuleDef, MatchBlock, Pattern, Expr, etc.
4. **Type Checker**: Valida que `sink.location` es `SourceLocation`.
5. **Error Handling**: Mensajes con spans (file:line:col).
6. **Unit Tests**: Tests para cada producción gramatical.

### 3.2. En Alcance (SHOULD)
7. **Language Server**: Autocompletado y diagnósticos en tiempo real (usando tower-lsp).
8. **Linter**: Reglas de estilo (e.g., nombres en CamelCase).
9. **Formatter**: Formato automático de reglas.

### 3.3. Fuera de Alcance
- IDE plugin (VSCode extension) → v3.3.
- Debugger para reglas → v4.0.

---

## 4. Arquitectura Detallada

### 4.1. Gramática PEG (rules.pest)

```pest
// hodei-dsl/grammar/rules.pest
WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
COMMENT = _{ "//" ~ (!"\n" ~ ANY)* ~ "\n" }

// Top-level
rule_file = { SOI ~ rule_def* ~ EOI }
rule_def = {
    "rule" ~ IDENT ~ "{" ~
        metadata_block ~
        match_block ~
        emit_block ~
    "}"
}

// Metadata
metadata_block = { metadata_item* }
metadata_item = {
    "description:" ~ STRING |
    "severity:" ~ severity |
    "tags:" ~ tag_list
}
severity = { "Critical" | "High" | "Medium" | "Low" | "Info" }
tag_list = { "[" ~ (STRING ~ ("," ~ STRING)*)? ~ "]" }

// Match block
match_block = {
    "match" ~ "{" ~
        pattern+ ~
        where_clause? ~
    "}"
}
pattern = { IDENT ~ ":" ~ fact_type ~ ("and" ~ condition)* }
fact_type = {
    "TaintSink" | "TaintSource" | "UncoveredLine" |
    "Vulnerability" | "Dependency" | /* ... */
}
condition = { path ~ comparison_op ~ literal }

where_clause = { "where" ~ expr }

// Expressions
expr = { term ~ (binary_op ~ term)* }
term = { literal | path | function_call | "(" ~ expr ~ ")" }

path = { IDENT ~ ("." ~ IDENT)* }
function_call = { IDENT ~ "(" ~ (expr ~ ("," ~ expr)*)? ~ ")" }

binary_op = { "==" | "!=" | "<" | ">" | "<=" | ">=" | "and" | "or" }
comparison_op = { "==" | "!=" | "contains" | "matches" }

literal = { STRING | NUMBER | BOOLEAN | NULL }
STRING = @{ "\"" ~ (!"\"" ~ ANY)* ~ "\"" }
NUMBER = @{ "-"? ~ ASCII_DIGIT+ ~ ("." ~ ASCII_DIGIT+)? }
BOOLEAN = { "true" | "false" }
NULL = { "null" }

// Emit block
emit_block = {
    "emit" ~ "Finding" ~ "{" ~
        emit_field+ ~
    "}"
}
emit_field = {
    "message:" ~ string_template |
    "confidence:" ~ confidence |
    "metadata:" ~ metadata_map
}
confidence = { "High" | "Medium" | "Low" }
string_template = @{ "\"" ~ (template_var | (!"\"" ~ ANY))* ~ "\"" }
template_var = { "{" ~ path ~ "}" }

metadata_map = { "{" ~ (IDENT ~ ":" ~ literal ~ ("," ~ IDENT ~ ":" ~ literal)*)? ~ "}" }

// Identifiers
IDENT = @{ ASCII_ALPHA ~ (ASCII_ALPHANUMERIC | "_")* }
```

### 4.2. AST Types

```rust
// hodei-dsl/src/ast.rs
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct RuleFile {
    pub rules: Vec<RuleDef>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleDef {
    pub name: String,
    pub metadata: Metadata,
    pub match_block: MatchBlock,
    pub emit_block: EmitBlock,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Metadata {
    pub description: Option<String>,
    pub severity: Severity,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchBlock {
    pub patterns: Vec<Pattern>,
    pub where_clause: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    pub binding: String,       // "sink"
    pub fact_type: FactType,   // TaintSink
    pub conditions: Vec<Condition>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub path: Path,
    pub op: ComparisonOp,
    pub value: Literal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Path(Path),
    FunctionCall {
        name: String,
        args: Vec<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    pub segments: Vec<String>,  // ["sink", "location", "file"]
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOp {
    Eq,
    Ne,
    Contains,
    Matches,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmitBlock {
    pub message_template: String,
    pub confidence: Confidence,
    pub metadata: HashMap<String, Literal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

/// Span para error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}
```

### 4.3. Parser Implementation

```rust
// hodei-dsl/src/parser.rs
use pest::Parser;
use pest_derive::Parser;
use crate::ast::*;
use crate::error::{ParseError, ParseResult};

#[derive(Parser)]
#[grammar = "grammar/rules.pest"]
pub struct RuleParser;

impl RuleParser {
    pub fn parse_file(input: &str) -> ParseResult<RuleFile> {
        let pairs = Self::parse(Rule::rule_file, input)
            .map_err(|e| ParseError::PestError(e))?;
        
        let mut rules = Vec::new();
        
        for pair in pairs {
            match pair.as_rule() {
                Rule::rule_def => {
                    rules.push(Self::parse_rule_def(pair)?);
                }
                Rule::EOI => {},
                _ => unreachable!(),
            }
        }
        
        Ok(RuleFile { rules })
    }
    
    fn parse_rule_def(pair: Pair<Rule>) -> ParseResult<RuleDef> {
        let span = Span::from_pair(&pair);
        let mut inner = pair.into_inner();
        
        let name = inner.next().unwrap().as_str().to_string();
        
        let mut metadata = Metadata::default();
        let mut match_block = None;
        let mut emit_block = None;
        
        for pair in inner {
            match pair.as_rule() {
                Rule::metadata_block => {
                    metadata = Self::parse_metadata_block(pair)?;
                }
                Rule::match_block => {
                    match_block = Some(Self::parse_match_block(pair)?);
                }
                Rule::emit_block => {
                    emit_block = Some(Self::parse_emit_block(pair)?);
                }
                _ => {}
            }
        }
        
        Ok(RuleDef {
            name,
            metadata,
            match_block: match_block.ok_or(ParseError::MissingMatchBlock)?,
            emit_block: emit_block.ok_or(ParseError::MissingEmitBlock)?,
            span,
        })
    }
    
    fn parse_match_block(pair: Pair<Rule>) -> ParseResult<MatchBlock> {
        let mut patterns = Vec::new();
        let mut where_clause = None;
        
        for pair in pair.into_inner() {
            match pair.as_rule() {
                Rule::pattern => {
                    patterns.push(Self::parse_pattern(pair)?);
                }
                Rule::where_clause => {
                    where_clause = Some(Self::parse_expr(pair.into_inner().next().unwrap())?);
                }
                _ => {}
            }
        }
        
        Ok(MatchBlock { patterns, where_clause })
    }
    
    fn parse_expr(pair: Pair<Rule>) -> ParseResult<Expr> {
        // Implementar precedence climbing parser para operadores binarios
        // https://en.wikipedia.org/wiki/Operator-precedence_parser
        todo!("Implement expression parser with precedence")
    }
}
```

### 4.4. Type Checker

```rust
// hodei-dsl/src/type_checker.rs
use crate::ast::*;
use crate::error::{TypeError, TypeResult};
use std::collections::HashMap;

pub struct TypeChecker {
    /// Tipos conocidos de cada FactType
    fact_schemas: HashMap<FactType, FactSchema>,
}

#[derive(Debug, Clone)]
pub struct FactSchema {
    pub fields: HashMap<String, FieldType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    String,
    Number,
    Boolean,
    SourceLocation,
    Confidence,
    Array(Box<FieldType>),
    Optional(Box<FieldType>),
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut fact_schemas = HashMap::new();
        
        // Definir schema de TaintSink
        let mut sink_fields = HashMap::new();
        sink_fields.insert("location".to_string(), FieldType::SourceLocation);
        sink_fields.insert("sink_type".to_string(), FieldType::String);
        sink_fields.insert("confidence".to_string(), FieldType::Confidence);
        fact_schemas.insert(
            FactType::TaintSink,
            FactSchema { fields: sink_fields },
        );
        
        // ... (definir schemas para todos los FactTypes)
        
        Self { fact_schemas }
    }
    
    pub fn check_rule(&self, rule: &RuleDef) -> TypeResult<()> {
        // 1. Construir symbol table con bindings
        let mut symbols = HashMap::new();
        for pattern in &rule.match_block.patterns {
            let schema = self.fact_schemas.get(&pattern.fact_type)
                .ok_or_else(|| TypeError::UnknownFactType(pattern.fact_type.clone()))?;
            symbols.insert(pattern.binding.clone(), schema.clone());
        }
        
        // 2. Type-check where clause
        if let Some(expr) = &rule.match_block.where_clause {
            self.check_expr(expr, &symbols)?;
        }
        
        // 3. Type-check emit block message template
        self.check_string_template(&rule.emit_block.message_template, &symbols)?;
        
        Ok(())
    }
    
    fn check_expr(&self, expr: &Expr, symbols: &HashMap<String, FactSchema>) -> TypeResult<FieldType> {
        match expr {
            Expr::Literal(lit) => Ok(Self::literal_type(lit)),
            
            Expr::Path(path) => {
                self.resolve_path(path, symbols)
            }
            
            Expr::Binary { left, op, right } => {
                let left_ty = self.check_expr(left, symbols)?;
                let right_ty = self.check_expr(right, symbols)?;
                
                // Validar que los tipos son compatibles con el operador
                match op {
                    BinaryOp::Eq | BinaryOp::Ne => {
                        if left_ty != right_ty {
                            return Err(TypeError::TypeMismatch {
                                expected: left_ty,
                                found: right_ty,
                            });
                        }
                        Ok(FieldType::Boolean)
                    }
                    BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                        if left_ty != FieldType::Number || right_ty != FieldType::Number {
                            return Err(TypeError::ExpectedNumber);
                        }
                        Ok(FieldType::Boolean)
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if left_ty != FieldType::Boolean || right_ty != FieldType::Boolean {
                            return Err(TypeError::ExpectedBoolean);
                        }
                        Ok(FieldType::Boolean)
                    }
                }
            }
            
            Expr::FunctionCall { name, args } => {
                // Implementar type checking de funciones builtin
                self.check_function_call(name, args, symbols)
            }
        }
    }
    
    fn resolve_path(&self, path: &Path, symbols: &HashMap<String, FactSchema>) -> TypeResult<FieldType> {
        let first = &path.segments[0];
        let schema = symbols.get(first)
            .ok_or_else(|| TypeError::UndefinedVariable(first.clone()))?;
        
        let mut current_type = FieldType::SourceLocation; // placeholder
        
        for (i, segment) in path.segments.iter().enumerate().skip(1) {
            match &current_type {
                FieldType::SourceLocation => {
                    // SourceLocation tiene campos: file, start, end
                    current_type = match segment.as_str() {
                        "file" => FieldType::String,
                        "start" | "end" => FieldType::SourceLocation, // Position type
                        _ => return Err(TypeError::NoSuchField {
                            ty: "SourceLocation".to_string(),
                            field: segment.clone(),
                        }),
                    };
                }
                _ => {
                    return Err(TypeError::CannotAccessField {
                        ty: format!("{:?}", current_type),
                    });
                }
            }
        }
        
        Ok(current_type)
    }
    
    fn literal_type(lit: &Literal) -> FieldType {
        match lit {
            Literal::String(_) => FieldType::String,
            Literal::Number(_) => FieldType::Number,
            Literal::Boolean(_) => FieldType::Boolean,
            Literal::Null => FieldType::Optional(Box::new(FieldType::String)),
        }
    }
}
```

---

## 5. Plan de Implementación

### 5.1. Fases

**Fase 1: Gramática Base** (Semana 1)
- [ ] Definir gramática PEG en `rules.pest`.
- [ ] Parser básico (rule_def, metadata, match_block simple).
- [ ] Tests: parsear reglas válidas.
- [ ] Error handling: reportar span de errores.

**Fase 2: Expresiones** (Semana 1-2)
- [ ] Parser de expresiones (binary ops, paths, literals).
- [ ] Precedence climbing para operadores.
- [ ] Tests: expresiones complejas.

**Fase 3: Type Checker** (Semana 2)
- [ ] Implementar `FactSchema` para todos los FactTypes.
- [ ] Type checker para expresiones.
- [ ] Tests: rechazar reglas con errores de tipo.

**Fase 4: Emit Block & Templates** (Semana 2-3)
- [ ] Parser de string templates con interpolación.
- [ ] Validar que variables en templates existen.
- [ ] Tests: templates con variables no definidas.

**Fase 5: Error Messages** (Semana 3)
- [ ] Formatear errores con spans y contexto.
- [ ] Sugerencias (e.g., "did you mean 'location'?").
- [ ] Tests: verificar mensajes de error.

**Fase 6: Language Server (opcional)** (Semana 4)
- [ ] Implementar LSP server con tower-lsp.
- [ ] Autocompletado de FactTypes y campos.
- [ ] Diagnósticos en tiempo real.

### 5.2. Dependencias de Crates
```toml
[dependencies]
pest = "2.7"
pest_derive = "2.7"
thiserror = "1.0"

[dev-dependencies]
insta = "1.34"          # Snapshot testing
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
    fn parse_simple_rule() {
        let input = r#"
            rule TestRule {
                description: "Test"
                severity: High
                
                match {
                    sink: TaintSink
                }
                
                emit Finding {
                    message: "Found sink"
                    confidence: High
                }
            }
        "#;
        
        let result = RuleParser::parse_file(input);
        assert!(result.is_ok());
        
        let rule_file = result.unwrap();
        assert_eq!(rule_file.rules.len(), 1);
        assert_eq!(rule_file.rules[0].name, "TestRule");
    }
    
    #[test]
    fn parse_spatial_join() {
        let input = r#"
            rule VulnerableUncovered {
                match {
                    sink: TaintSink and
                    uncovered: UncoveredLine
                    
                    where sink.location == uncovered.location
                }
                
                emit Finding {
                    message: "Vulnerable at {sink.location}"
                    confidence: High
                }
            }
        "#;
        
        let result = RuleParser::parse_file(input);
        assert!(result.is_ok());
    }
    
    #[test]
    fn type_error_undefined_variable() {
        let input = r#"
            rule BadRule {
                match {
                    sink: TaintSink
                    
                    where unknown_var.location == sink.location
                }
                
                emit Finding {
                    message: "Bad"
                    confidence: High
                }
            }
        "#;
        
        let rule_file = RuleParser::parse_file(input).unwrap();
        let checker = TypeChecker::new();
        
        let result = checker.check_rule(&rule_file.rules[0]);
        assert!(matches!(result, Err(TypeError::UndefinedVariable(_))));
    }
}
```

### 6.2. Snapshot Tests (con insta)

```rust
#[test]
fn snapshot_error_messages() {
    let input = r#"
        rule BadSyntax {
            match {
                sink: TaintSink
                where sink.location = uncovered.location  // '=' en vez de '=='
            }
            emit Finding {
                message: "Test"
            }
        }
    "#;
    
    let result = RuleParser::parse_file(input);
    let error_msg = format!("{:?}", result.unwrap_err());
    
    insta::assert_snapshot!(error_msg);
}
```

### 6.3. Fuzzing

```rust
// fuzz/fuzz_targets/parse_rule.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use hodei_dsl::parser::RuleParser;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = RuleParser::parse_file(s);
    }
});
```

---

## 7. Seguridad & Mitigaciones

### 7.1. Threat Model

| Amenaza | Riesgo | Mitigación |
|---------|--------|------------|
| Inyección de código | Critical | Gramática formal (sin eval); todo via AST |
| DoS via regex backtracking | High | PEG no tiene backtracking exponencial |
| Path traversal via templates | Medium | Sanitizar paths en runtime (ProjectPath newtype) |
| ReDoS en string templates | Low | Templates usan parser simple (sin regex) |

### 7.2. Code Review Checklist
- [ ] No hay `eval`, `unsafe`, o ejecución dinámica.
- [ ] Todos los paths sanitizados con ProjectPath.
- [ ] Límites de recursión en expresiones (max depth=100).
- [ ] Fuzzing del parser en CI (cargo-fuzz).

---

## 8. Documentación

### 8.1. Guía de Usuario (DSL)

```markdown
# Hodei DSL Reference

## Rule Structure
```hodei
rule RuleName {
    description: "Human-readable description"
    severity: Critical | High | Medium | Low | Info
    tags: ["security", "taint-analysis"]
    
    match {
        <patterns>
        where <condition>
    }
    
    emit Finding {
        message: "Template with {variable}"
        confidence: High | Medium | Low
        metadata: { key: "value" }
    }
}
```

## Patterns
```hodei
// Simple pattern
sink: TaintSink

// Pattern with condition
sink: TaintSink and sink.confidence == High

// Multiple patterns (spatial join)
sink: TaintSink and
uncovered: UncoveredLine

where sink.location == uncovered.location
```

## Built-in Functions
- `distance(loc1, loc2)` → Number (líneas entre dos locations)
- `reachable(source, sink)` → Boolean (hay flow entre source y sink)
- `count(pattern)` → Number (cuántos hechos matchean)
```

---

## 9. Criterios de Aceptación

- [ ] **Gramática completa**: Todos los features del PRD cubiertos.
- [ ] **Tests**: 100% cobertura en parser; property tests para expr parser.
- [ ] **Type safety**: Type checker rechaza todas las reglas mal tipadas.
- [ ] **Error messages**: Spans precisos; sugerencias útiles.
- [ ] **Seguridad**: 0 vulnerabilidades (audit + fuzzing).
- [ ] **Docs**: User guide completo con ejemplos.
- [ ] **CI**: Fuzzing automatizado (1min en cada PR).

---

## 10. Notas & Referencias

- **pest**: https://pest.rs/
- **Cedar (AWS)**: https://www.cedarpolicy.com/
- **Operator Precedence Parsing**: https://en.wikipedia.org/wiki/Operator-precedence_parser
- **PEG vs CFG**: https://en.wikipedia.org/wiki/Parsing_expression_grammar

---

**Última Actualización**: 2025-01-XX  
**Próxima Revisión**: Después de Fase 3 (Type Checker implementado)
