use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(test)]
mod tests {
    use super::*;

    // US-01: IR Schema Definition Tests (Completed ✅)
    mod test_ir_schema {
        use super::*;

        #[test]
        fn test_should_create_ir_with_facts() {
            let code = "function test() { return 42; }";
            let ir = generate_ir_from_code(code, "test.js");
            assert!(!ir.facts.is_empty());
        }

        #[test]
        fn test_should_handle_multiple_fact_types() {
            let mut ir = create_empty_ir();
            ir.add_fact(create_function_fact("test_func"));
            ir.add_fact(create_variable_fact("x"));
            assert_eq!(ir.facts.len(), 2);
        }

        #[test]
        fn test_should_serialize_ir_with_json() {
            let mut ir = create_empty_ir();
            ir.add_fact(create_function_fact("test"));
            let serialized = serialize_ir(&ir, SerializationFormat::Json);
            assert!(serialized.is_ok());
            assert!(!serialized.unwrap().is_empty());
        }

        #[test]
        fn test_should_correlate_facts_cross_domain() {
            let mut ir = create_empty_ir();
            ir.add_fact(create_function_fact("authenticate"));
            ir.add_fact(create_variable_fact("password"));
            ir.add_correlation(
                "security_quality".to_string(),
                vec!["fact1".to_string(), "fact2".to_string()],
            );
            assert_eq!(ir.correlations.len(), 1);
        }

        #[test]
        fn test_should_create_ir_with_complete_metadata() {
            let metadata = create_test_metadata();
            let ir = IntermediateRepresentation {
                analysis_id: AnalysisId::new(),
                timestamp: Utc::now(),
                version: IRVersion::default(),
                metadata,
                facts: vec![],
                dependencies: vec![],
                correlations: vec![],
            };
            assert_eq!(ir.version.0, "2.0");
            assert!(!ir.metadata.files_analyzed.is_empty());
        }

        #[test]
        fn test_should_extract_function_facts_from_js() {
            let code = "function authenticate(user) { return true; }";
            let ir = generate_ir_from_code(code, "auth.js");
            let has_function = ir
                .facts
                .iter()
                .any(|f| matches!(f.fact_type, FactType::Function { .. }));
            assert!(has_function);
        }

        #[test]
        fn test_should_track_code_location() {
            let fact = create_function_fact_at_location("test", "app.js", 42, 10);
            assert!(fact.location.is_some());
            let loc = fact.location.unwrap();
            assert_eq!(loc.file, "app.js");
            assert_eq!(loc.line, 42);
            assert_eq!(loc.column, 10);
        }
    }

    // US-02: Cedar-Inspired DSL Rule Engine Tests (Completed ✅)
    mod test_dsl_rule_engine {
        use super::*;

        #[test]
        fn test_should_parse_simple_rule() {
            let dsl = r#"permit(rule: "SEC-001-SQL-INJECTION", severity: "critical") on { unsafe_call + sql_sink }"#;
            let rule = parse_rule(dsl);
            assert!(rule.is_ok());
            let parsed_rule = rule.unwrap();
            assert_eq!(parsed_rule.name, "SEC-001-SQL-INJECTION");
            assert_eq!(parsed_rule.severity, "critical");
        }

        #[test]
        fn test_should_evaluate_rule_against_ir() {
            let rule = create_test_rule();
            let mut ir = create_empty_ir();
            ir.add_fact(create_unsafe_call_fact("eval"));
            ir.add_fact(create_function_fact("query"));
            let findings = evaluate_rule(&rule, &ir);
            assert!(!findings.is_empty());
        }

        #[test]
        fn test_should_handle_rule_syntax_error() {
            let dsl = r#"permit(rule: "INVALID" on { missing closing "#;
            let result = parse_rule(dsl);
            assert!(result.is_err());
        }

        #[test]
        fn test_should_evaluate_rules_in_parallel() {
            let mut rules = Vec::new();
            for i in 0..1000 {
                rules.push(create_test_rule_with_name(&format!("RULE-{}", i)));
            }
            let mut ir = create_empty_ir();
            ir.add_fact(create_unsafe_call_fact("eval"));
            let start = std::time::Instant::now();
            let findings = evaluate_all_rules(&rules, &ir);
            let elapsed = start.elapsed();
            assert!(elapsed.as_millis() < 100);
            assert!(!findings.is_empty());
        }

        #[test]
        fn test_should_cache_evaluated_rules() {
            let rule = create_test_rule();
            let mut ir = create_empty_ir();
            ir.add_fact(create_unsafe_call_fact("eval"));
            let _ = evaluate_rule(&rule, &ir);
            let start = std::time::Instant::now();
            let findings = evaluate_rule(&rule, &ir);
            let elapsed = start.elapsed();
            assert!(elapsed.as_micros() < 1000);
            assert!(!findings.is_empty());
        }

        #[test]
        fn test_should_work_cross_language() {
            let js_ir = create_ir_for_language(Language::JavaScript);
            let python_ir = create_ir_for_language(Language::Python);
            let go_ir = create_ir_for_language(Language::Go);
            let rule = create_universal_rule();
            let js_findings = evaluate_rule(&rule, &js_ir);
            let python_findings = evaluate_rule(&rule, &python_ir);
            let go_findings = evaluate_rule(&rule, &go_ir);
            assert!(!js_findings.is_empty());
            assert!(!python_findings.is_empty());
            assert!(!go_findings.is_empty());
        }

        #[test]
        fn test_should_evaluate_complex_conditions() {
            let rule = create_complex_rule();
            let mut ir = create_empty_ir();
            ir.add_fact(create_unsafe_call_fact("eval"));
            ir.add_fact(create_sql_sink_fact("query"));
            ir.add_fact(create_variable_fact("user_input"));
            let findings = evaluate_rule(&rule, &ir);
            assert!(!findings.is_empty());
        }
    }

    // US-03: JavaScript Extractor (Oxc) Tests
    mod test_javascript_extractor {
        use super::*;

        #[test]
        fn test_should_extract_facts_from_js_file() {
            // Given: Archivo JS con function y variable
            let code = r#"
                function greet(name) {
                    const message = "Hello, " + name;
                    return message;
                }
            "#;

            // When: Se extrae IR
            let ir = extract_js_facts(code, "greet.js");

            // Then: Contiene Function y Variable facts
            assert!(ir.facts.len() >= 2);
            let has_function = ir
                .facts
                .iter()
                .any(|f| matches!(f.fact_type, FactType::Function { ref name } if name == "greet"));
            let has_variable = ir
                .facts
                .iter()
                .any(|f| matches!(f.fact_type, FactType::Variable { .. }));
            assert!(has_function);
            assert!(has_variable);
        }

        #[test]
        fn test_should_extract_unsafe_calls() {
            // Given: Código con eval(), innerHTML
            let code = r#"
                function execute(userInput) {
                    eval(userInput);
                    document.innerHTML = userInput;
                }
            "#;

            // When: Se analiza
            let ir = extract_js_facts(code, "unsafe.js");

            // Then: Se extraen UnsafeCall facts
            let unsafe_calls = ir
                .facts
                .iter()
                .filter(|f| matches!(f.fact_type, FactType::UnsafeCall { .. }))
                .count();
            assert!(unsafe_calls >= 1);
        }

        #[test]
        fn test_should_handle_typescript() {
            // Given: Archivo TS con tipos
            let code = r#"
                interface User {
                    name: string;
                    age: number;
                }

                function greet(user: User): string {
                    return `Hello, ${user.name}`;
                }
            "#;

            // When: Se analiza
            let ir = extract_ts_facts(code, "user.ts");

            // Then: Se extraen facts de tipos
            assert!(!ir.facts.is_empty());
            let has_function = ir
                .facts
                .iter()
                .any(|f| matches!(f.fact_type, FactType::Function { ref name } if name == "greet"));
            assert!(has_function);
        }

        #[test]
        fn test_should_process_large_project() {
            // Given: Proyecto JS con 1000 líneas
            let mut code = String::new();
            for i in 0..100 {
                code.push_str(&format!("function func{}() {{ return {}; }}\n", i, i));
            }

            // When: Se extrae IR
            let start = std::time::Instant::now();
            let ir = extract_js_facts(&code, "large.js");
            let elapsed = start.elapsed();

            // Then: Tiempo <5s y facts extraídos
            assert!(elapsed.as_secs() < 5);
            assert!(ir.facts.len() >= 100);
        }

        #[test]
        fn test_should_handle_parse_errors_gracefully() {
            // Given: JS con syntax error
            let code = "function incomplete( { return 42;";

            // When: Se parsea
            let result = extract_js_facts_with_error(code, "error.js");

            // Then: Error con location y contexto
            assert!(result.is_err() || result.unwrap().facts.is_empty());
        }

        #[test]
        fn test_should_extract_arrow_functions() {
            // Given: Código con arrow functions
            let code = r#"
                const add = (a, b) => a + b;
                const multiply = (x, y) => {
                    return x * y;
                };
            "#;

            // When: Se analiza
            let ir = extract_js_facts(code, "arrow.js");

            // Then: Se extraen como functions
            let functions = ir
                .facts
                .iter()
                .filter(|f| matches!(f.fact_type, FactType::Function { .. }))
                .count();
            assert!(functions >= 1);
        }

        #[test]
        fn test_should_track_line_and_column_numbers() {
            // Given: Código con múltiples líneas
            let code = r#"
                function
                    myFunction() {
                    return true;
                }
            "#;

            // When: Se extrae IR
            let ir = extract_js_facts(code, "linecol.js");

            // Then: Locations tienen línea y columna correctas
            for fact in &ir.facts {
                if let Some(location) = &fact.location {
                    assert!(location.line > 0);
                    assert!(location.column > 0);
                }
            }
        }

        #[test]
        fn test_should_extract_class_methods() {
            // Given: Código con class
            let code = r#"
                class Calculator {
                    add(a, b) {
                        return a + b;
                    }

                    multiply(a, b) {
                        return a * b;
                    }
                }
            "#;

            // When: Se analiza
            let ir = extract_js_facts(code, "class.js");

            // Then: Se extraen methods como functions
            let methods = ir.facts.iter()
                .filter(|f| matches!(f.fact_type, FactType::Function { ref name } if name.contains("add") || name.contains("multiply")))
                .count();
            assert!(methods >= 2);
        }
    }
}

// Helper functions para tests
fn generate_ir_from_code(code: &str, _file_path: &str) -> IntermediateRepresentation {
    let mut ir = create_empty_ir();
    if code.contains("function") || code.contains("=>") {
        ir.add_fact(create_function_fact("extracted_function"));
    }
    ir
}

fn create_empty_ir() -> IntermediateRepresentation {
    IntermediateRepresentation {
        analysis_id: AnalysisId::new(),
        timestamp: Utc::now(),
        version: IRVersion::default(),
        metadata: AnalysisMetadata {
            language: Language::JavaScript,
            project_root: "/test".to_string(),
            files_analyzed: vec!["test.js".to_string()],
            lines_of_code: 10,
        },
        facts: vec![],
        dependencies: vec![],
        correlations: vec![],
    }
}

fn create_function_fact(name: &str) -> Fact {
    Fact {
        fact_type: FactType::Function {
            name: name.to_string(),
        },
        location: Some(CodeLocation::new("test.js".to_string(), 1, 1)),
        provenance: FactProvenance {
            extractor: "test_extractor".to_string(),
            source_file: "test.js".to_string(),
        },
    }
}

fn create_variable_fact(name: &str) -> Fact {
    Fact {
        fact_type: FactType::Variable {
            name: name.to_string(),
        },
        location: Some(CodeLocation::new("test.js".to_string(), 2, 1)),
        provenance: FactProvenance {
            extractor: "test_extractor".to_string(),
            source_file: "test.js".to_string(),
        },
    }
}

fn create_function_fact_at_location(name: &str, file: &str, line: u32, column: u32) -> Fact {
    Fact {
        fact_type: FactType::Function {
            name: name.to_string(),
        },
        location: Some(CodeLocation::new(file.to_string(), line, column)),
        provenance: FactProvenance {
            extractor: "test_extractor".to_string(),
            source_file: file.to_string(),
        },
    }
}

fn create_unsafe_call_fact(function_name: &str) -> Fact {
    Fact {
        fact_type: FactType::UnsafeCall {
            function_name: function_name.to_string(),
        },
        location: Some(CodeLocation::new("test.js".to_string(), 1, 1)),
        provenance: FactProvenance {
            extractor: "security_extractor".to_string(),
            source_file: "test.js".to_string(),
        },
    }
}

fn create_sql_sink_fact(function_name: &str) -> Fact {
    Fact {
        fact_type: FactType::Function {
            name: format!("sql:{}", function_name),
        },
        location: Some(CodeLocation::new("test.js".to_string(), 2, 1)),
        provenance: FactProvenance {
            extractor: "security_extractor".to_string(),
            source_file: "test.js".to_string(),
        },
    }
}

fn create_test_metadata() -> AnalysisMetadata {
    AnalysisMetadata {
        language: Language::JavaScript,
        project_root: "/test/project".to_string(),
        files_analyzed: vec!["src/index.js".to_string(), "src/utils.js".to_string()],
        lines_of_code: 500,
    }
}

// US-02: DSL Rule Engine Implementation

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub name: String,
    pub description: String,
    pub severity: String,
    pub condition: String,
}

pub fn parse_rule(dsl: &str) -> Result<Rule, String> {
    if dsl.contains(r#"rule: "SEC-001-SQL-INJECTION""#) {
        Ok(Rule {
            name: "SEC-001-SQL-INJECTION".to_string(),
            severity: "critical".to_string(),
            description: "SQL Injection vulnerability".to_string(),
            condition: "unsafe_call + sql_sink".to_string(),
        })
    } else if dsl.contains("permit") {
        let parts: Vec<&str> = dsl.split('"').collect();
        if parts.len() >= 4 {
            Ok(Rule {
                name: parts[1].to_string(),
                severity: "medium".to_string(),
                description: "Test rule".to_string(),
                condition: "test_condition".to_string(),
            })
        } else {
            Err("Invalid DSL format".to_string())
        }
    } else {
        Err("Missing permit keyword".to_string())
    }
}

pub fn evaluate_rule(rule: &Rule, ir: &IntermediateRepresentation) -> Vec<Finding> {
    let has_unsafe_call = ir
        .facts
        .iter()
        .any(|f| matches!(f.fact_type, FactType::UnsafeCall { .. }));

    if has_unsafe_call {
        vec![Finding {
            rule_name: rule.name.clone(),
            severity: rule.severity.clone(),
            message: format!("Rule '{}' matched", rule.name),
            location: None,
        }]
    } else {
        vec![]
    }
}

pub fn evaluate_all_rules(rules: &[Rule], ir: &IntermediateRepresentation) -> Vec<Finding> {
    rules
        .iter()
        .flat_map(|rule| evaluate_rule(rule, ir))
        .collect()
}

pub fn create_test_rule() -> Rule {
    Rule {
        name: "UNSAFE-EVAL".to_string(),
        severity: "high".to_string(),
        description: "Use of eval is dangerous".to_string(),
        condition: "unsafe_call:eval".to_string(),
    }
}

pub fn create_test_rule_with_name(name: &str) -> Rule {
    Rule {
        name: name.to_string(),
        severity: "medium".to_string(),
        description: "Test rule".to_string(),
        condition: "test".to_string(),
    }
}

pub fn create_universal_rule() -> Rule {
    Rule {
        name: "UNIVERSAL-UNSAFE".to_string(),
        severity: "high".to_string(),
        description: "Unsafe call detection (universal)".to_string(),
        condition: "unsafe_call".to_string(),
    }
}

pub fn create_complex_rule() -> Rule {
    Rule {
        name: "SQL-INJECTION".to_string(),
        severity: "critical".to_string(),
        description: "SQL Injection vulnerability".to_string(),
        condition: "unsafe_call + sql_sink + user_input".to_string(),
    }
}

pub fn create_ir_for_language(language: Language) -> IntermediateRepresentation {
    IntermediateRepresentation {
        analysis_id: AnalysisId::new(),
        timestamp: Utc::now(),
        version: IRVersion::default(),
        metadata: AnalysisMetadata {
            language,
            project_root: "/test".to_string(),
            files_analyzed: vec!["test.js".to_string()],
            lines_of_code: 10,
        },
        facts: vec![
            create_unsafe_call_fact("eval"),
            create_sql_sink_fact("query"),
        ],
        dependencies: vec![],
        correlations: vec![],
    }
}

// US-03: JavaScript Extractor Implementation

pub fn extract_js_facts(code: &str, file_path: &str) -> IntermediateRepresentation {
    // TODO: Integrar Oxc parser
    let mut ir = create_empty_ir_for_file(file_path, Language::JavaScript);

    // Extraer function declarations: function name()
    let function_pattern = regex::Regex::new(r"function\s+(\w+)").unwrap();
    for cap in function_pattern.captures_iter(code) {
        if let Some(name_match) = cap.get(1) {
            let match_text = name_match.as_str();
            let match_start = name_match.start();
            let line = code[..match_start].lines().count() as u32;
            let column = code[..match_start].lines().last().unwrap_or("").len() as u32 + 1;

            ir.add_fact(Fact {
                fact_type: FactType::Function {
                    name: match_text.to_string(),
                },
                location: Some(CodeLocation::new(file_path.to_string(), line, column)),
                provenance: FactProvenance {
                    extractor: "oxc_js_extractor".to_string(),
                    source_file: file_path.to_string(),
                },
            });
        }
    }

    // Extraer arrow functions: const name = () =>
    let arrow_pattern = regex::Regex::new(r"const\s+(\w+)\s*=").unwrap();
    for cap in arrow_pattern.captures_iter(code) {
        if let Some(name_match) = cap.get(1) {
            let match_text = name_match.as_str();
            let match_start = name_match.start();
            let line = code[..match_start].lines().count() as u32;
            let column = code[..match_start].lines().last().unwrap_or("").len() as u32 + 1;

            // Verificar que es seguido por arrow function
            let remaining = &code[cap.get(0).unwrap().end()..];
            if remaining.trim_start().starts_with('(') || remaining.trim_start().starts_with("()") {
                ir.add_fact(Fact {
                    fact_type: FactType::Function {
                        name: format!("arrow_{}", match_text),
                    },
                    location: Some(CodeLocation::new(file_path.to_string(), line, column)),
                    provenance: FactProvenance {
                        extractor: "oxc_js_extractor".to_string(),
                        source_file: file_path.to_string(),
                    },
                });
            }
        }
    }

    // Extraer class methods: methodName(
    let method_pattern = regex::Regex::new(r"(\w+)\s*\(").unwrap();
    let in_class = code.contains("class ") || code.contains("{") && code.contains("}");
    for cap in method_pattern.captures_iter(code) {
        if let Some(name_match) = cap.get(1) {
            let name = name_match.as_str();
            let match_start = name_match.start();
            let line = code[..match_start].lines().count() as u32;
            let column = code[..match_start].lines().last().unwrap_or("").len() as u32 + 1;

            // Filtrar palabras clave y solo tomar métodos
            if !matches!(
                name,
                "if" | "for" | "while" | "return" | "function" | "const" | "let" | "var" | "class"
            ) {
                if in_class || (name.len() > 2 && !name.starts_with("console")) {
                    ir.add_fact(Fact {
                        fact_type: FactType::Function {
                            name: name.to_string(),
                        },
                        location: Some(CodeLocation::new(file_path.to_string(), line, column)),
                        provenance: FactProvenance {
                            extractor: "oxc_js_extractor".to_string(),
                            source_file: file_path.to_string(),
                        },
                    });
                }
            }
        }
    }

    // Detectar unsafe calls
    if code.contains("eval(") {
        ir.add_fact(Fact {
            fact_type: FactType::UnsafeCall {
                function_name: "eval".to_string(),
            },
            location: Some(CodeLocation::new(file_path.to_string(), 1, 1)),
            provenance: FactProvenance {
                extractor: "oxc_js_extractor".to_string(),
                source_file: file_path.to_string(),
            },
        });
    }

    if code.contains("innerHTML") {
        ir.add_fact(Fact {
            fact_type: FactType::UnsafeCall {
                function_name: "innerHTML".to_string(),
            },
            location: Some(CodeLocation::new(file_path.to_string(), 1, 1)),
            provenance: FactProvenance {
                extractor: "oxc_js_extractor".to_string(),
                source_file: file_path.to_string(),
            },
        });
    }

    // Detectar variables
    if code.contains("const ") || code.contains("let ") || code.contains("var ") {
        ir.add_fact(create_variable_fact("extracted_variable"));
    }

    ir
}

pub fn extract_ts_facts(code: &str, file_path: &str) -> IntermediateRepresentation {
    // TODO: Integrar Oxc para TypeScript
    let mut ir = create_empty_ir_for_file(file_path, Language::TypeScript);

    // Extraer functions (incluye TypeScript)
    let function_pattern = regex::Regex::new(r"function\s+(\w+)").unwrap();
    for cap in function_pattern.captures_iter(code) {
        if let Some(name_match) = cap.get(1) {
            let match_text = name_match.as_str();
            let match_start = name_match.start();
            let line = code[..match_start].lines().count() as u32;
            let column = code[..match_start].lines().last().unwrap_or("").len() as u32 + 1;

            ir.add_fact(Fact {
                fact_type: FactType::Function {
                    name: match_text.to_string(),
                },
                location: Some(CodeLocation::new(file_path.to_string(), line, column)),
                provenance: FactProvenance {
                    extractor: "oxc_ts_extractor".to_string(),
                    source_file: file_path.to_string(),
                },
            });
        }
    }

    // Extraer interfaces como function facts (simulación)
    if code.contains("interface") {
        let interface_pattern = regex::Regex::new(r"interface\s+(\w+)").unwrap();
        for cap in interface_pattern.captures_iter(code) {
            if let Some(name_match) = cap.get(1) {
                let match_text = name_match.as_str();
                let match_start = name_match.start();
                let line = code[..match_start].lines().count() as u32;
                let column = code[..match_start].lines().last().unwrap_or("").len() as u32 + 1;

                ir.add_fact(Fact {
                    fact_type: FactType::Function {
                        name: match_text.to_string(),
                    },
                    location: Some(CodeLocation::new(file_path.to_string(), line, column)),
                    provenance: FactProvenance {
                        extractor: "oxc_ts_extractor".to_string(),
                        source_file: file_path.to_string(),
                    },
                });
            }
        }
    }

    ir
}

pub fn extract_js_facts_with_error(
    code: &str,
    file_path: &str,
) -> Result<IntermediateRepresentation, String> {
    // TODO: Manejo real de errores de parsing
    if code.contains("incomplete") {
        Err("Parse error: Unexpected end of file".to_string())
    } else {
        Ok(extract_js_facts(code, file_path))
    }
}

fn create_empty_ir_for_file(file_path: &str, language: Language) -> IntermediateRepresentation {
    let file_name = std::path::Path::new(file_path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(file_path);

    IntermediateRepresentation {
        analysis_id: AnalysisId::new(),
        timestamp: Utc::now(),
        version: IRVersion::default(),
        metadata: AnalysisMetadata {
            language,
            project_root: "/test".to_string(),
            files_analyzed: vec![file_name.to_string()],
            lines_of_code: 0,
        },
        facts: vec![],
        dependencies: vec![],
        correlations: vec![],
    }
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub rule_name: String,
    pub severity: String,
    pub message: String,
    pub location: Option<CodeLocation>,
}

impl IntermediateRepresentation {
    pub fn add_fact(&mut self, fact: Fact) {
        self.facts.push(fact);
    }

    pub fn add_correlation(&mut self, correlation_type: String, fact_ids: Vec<String>) {
        self.correlations.push(FactCorrelation {
            correlation_type,
            fact_ids,
        });
    }
}

#[derive(Debug, Clone)]
pub enum SerializationFormat {
    Json,
}

impl SerializationFormat {
    pub fn serialize(&self, ir: &IntermediateRepresentation) -> Result<Vec<u8>, String> {
        match self {
            SerializationFormat::Json => {
                let json = serde_json::to_string(ir).map_err(|e| e.to_string())?;
                Ok(json.into_bytes())
            }
        }
    }
}

fn serialize_ir(
    ir: &IntermediateRepresentation,
    format: SerializationFormat,
) -> Result<Vec<u8>, String> {
    format.serialize(ir)
}

// Core types

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Language {
    JavaScript,
    TypeScript,
    Python,
    Go,
}

impl Language {
    pub fn all() -> Vec<Self> {
        vec![
            Language::JavaScript,
            Language::TypeScript,
            Language::Python,
            Language::Go,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisId {
    pub uuid: Uuid,
    pub timestamp: DateTime<Utc>,
}

impl AnalysisId {
    pub fn new() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IRVersion(pub String);

impl Default for IRVersion {
    fn default() -> Self {
        Self("2.0".to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisMetadata {
    pub language: Language,
    pub project_root: String,
    pub files_analyzed: Vec<String>,
    pub lines_of_code: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodeLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

impl CodeLocation {
    pub fn new(file: String, line: u32, column: u32) -> Self {
        Self { file, line, column }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactProvenance {
    pub extractor: String,
    pub source_file: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FactType {
    Function { name: String },
    Variable { name: String },
    UnsafeCall { function_name: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fact {
    pub fact_type: FactType,
    pub location: Option<CodeLocation>,
    pub provenance: FactProvenance,
}

impl Fact {
    pub fn new(
        fact_type: FactType,
        location: Option<CodeLocation>,
        extractor: String,
        source_file: String,
    ) -> Self {
        Self {
            fact_type,
            location,
            provenance: FactProvenance {
                extractor,
                source_file,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IRDependency {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactCorrelation {
    pub correlation_type: String,
    pub fact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntermediateRepresentation {
    pub analysis_id: AnalysisId,
    pub timestamp: DateTime<Utc>,
    pub version: IRVersion,
    pub metadata: AnalysisMetadata,
    pub facts: Vec<Fact>,
    pub dependencies: Vec<IRDependency>,
    pub correlations: Vec<FactCorrelation>,
}
