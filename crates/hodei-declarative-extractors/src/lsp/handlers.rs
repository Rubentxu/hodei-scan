//! LSP handlers for Hodei DSL
//!
//! This module implements handlers for LSP features like completion,
//! diagnostics, hover, and document symbols.

use lsp_types::*;

/// Handler for code completion requests
#[derive(Debug, Clone)]
pub struct CompletionHandler;

impl CompletionHandler {
    pub fn new() -> Self {
        Self
    }

    /// Get completions for a given position in the document
    pub fn get_completions(
        &self,
        document: &str,
        position: Position,
    ) -> Result<Vec<CompletionItem>, Box<dyn std::error::Error + Send + Sync>> {
        let mut completions = Vec::new();

        let lines: Vec<&str> = document.lines().collect();
        if position.line as usize >= lines.len() {
            return Ok(completions);
        }

        let current_line = lines[position.line as usize];

        if current_line.trim_start().starts_with("rule:") {
            completions.extend(self.get_rule_field_completions());
        } else if current_line.contains("metadata:") {
            completions.extend(self.get_metadata_completions());
        } else if current_line.contains("languages:") {
            completions.extend(self.get_language_completions());
        } else if current_line.contains("pattern:") {
            completions.extend(self.get_metavariable_completions());
        }

        Ok(completions)
    }

    fn get_rule_field_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "metadata:".to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some("Rule metadata section".to_string()),
                insert_text: Some("metadata:\n  description: \"$1\"".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "languages:".to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some("Supported programming languages".to_string()),
                insert_text: Some("languages: [\"$1\"]".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "patterns:".to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some("List of patterns to match".to_string()),
                insert_text: Some("patterns:\n  - pattern: |\n      $X = $Y".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }

    fn get_metadata_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "description:".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Human-readable description".to_string()),
                insert_text: Some("description: \"$1\"".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "severity:".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("Severity level".to_string()),
                insert_text: Some("severity: \"$1\"".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ]
    }

    fn get_language_completions(&self) -> Vec<CompletionItem> {
        vec!["python", "javascript", "typescript", "rust"]
            .iter()
            .map(|lang| CompletionItem {
                label: format!("\"{}\"", lang),
                kind: Some(CompletionItemKind::CONSTANT),
                insert_text: Some(format!("\"{}\"", lang)),
                ..Default::default()
            })
            .collect()
    }

    fn get_metavariable_completions(&self) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "$X".to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                insert_text: Some("$X".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "$VAR".to_string(),
                kind: Some(CompletionItemKind::VARIABLE),
                insert_text: Some("$VAR".to_string()),
                ..Default::default()
            },
        ]
    }
}

impl Default for CompletionHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Handler for diagnostic requests
#[derive(Debug, Clone)]
pub struct DiagnosticHandler;

impl DiagnosticHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn validate_yaml(
        &self,
        document: &str,
    ) -> Result<Vec<Diagnostic>, Box<dyn std::error::Error + Send + Sync>> {
        let mut diagnostics = Vec::new();

        match serde_yaml::from_str::<serde_yaml::Value>(document) {
            Ok(yaml_value) => {
                if let Some(rule) = yaml_value.get("rule") {
                    if !rule.get("id").is_some() {
                        diagnostics.push(Diagnostic {
                            range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                            severity: Some(DiagnosticSeverity::ERROR),
                            message: "Required field 'id' is missing".to_string(),
                            ..Default::default()
                        });
                    }
                }
            }
            Err(e) => {
                diagnostics.push(Diagnostic {
                    range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Invalid YAML: {}", e),
                    ..Default::default()
                });
            }
        }

        Ok(diagnostics)
    }
}

impl Default for DiagnosticHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Handler for hover requests
#[derive(Debug, Clone)]
pub struct HoverHandler;

impl HoverHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn get_hover(
        &self,
        document: &str,
        position: Position,
    ) -> Result<Option<Hover>, Box<dyn std::error::Error + Send + Sync>> {
        let lines: Vec<&str> = document.lines().collect();
        if position.line as usize >= lines.len() {
            return Ok(None);
        }

        let current_line = lines[position.line as usize];

        let hover_content = if current_line.contains("metadata") {
            "**metadata:** Rule metadata section".to_string()
        } else if current_line.contains("patterns") {
            "**patterns:** Code patterns to match".to_string()
        } else if current_line.contains("languages") {
            "**languages:** Supported languages".to_string()
        } else {
            return Ok(None);
        };

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(hover_content)),
            range: None,
        }))
    }
}

impl Default for HoverHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Handler for document symbols
#[derive(Debug, Clone)]
pub struct SymbolHandler;

impl SymbolHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn get_document_symbols(
        &self,
        document: &str,
    ) -> Result<Vec<DocumentSymbol>, Box<dyn std::error::Error + Send + Sync>> {
        let mut symbols = Vec::new();

        if let Ok(yaml_value) = serde_yaml::from_str::<serde_yaml::Value>(document) {
            if let Some(rule) = yaml_value.get("rule") {
                if let Some(mapping) = rule.as_mapping() {
                    for (k, _) in mapping {
                        if let Some(key) = k.as_str() {
                            symbols.push(DocumentSymbol {
                                name: key.to_string(),
                                kind: SymbolKind::FIELD,
                                range: Range::default(),
                                selection_range: Range::default(),
                                children: None,
                                detail: None,
                                tags: None,
                                deprecated: None,
                            });
                        }
                    }
                }
            }
        }

        Ok(symbols)
    }
}

impl Default for SymbolHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Handler for code actions
#[derive(Debug, Clone)]
pub struct CodeActionHandler;

impl CodeActionHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn get_code_actions(
        &self,
        _document: &str,
    ) -> Result<Vec<CodeAction>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![CodeAction {
            title: "Add fix suggestion".to_string(),
            kind: Some(CodeActionKind::QUICKFIX),
            edit: None,
            command: None,
            diagnostics: None,
            is_preferred: Some(true),
            disabled: None,
            data: None,
        }])
    }
}

impl Default for CodeActionHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Handler for rule templates
#[derive(Debug, Clone)]
pub struct TemplateHandler;

impl TemplateHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn get_rule_templates(
        &self,
    ) -> Result<Vec<CompletionItem>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(vec![CompletionItem {
            label: "OWASP A01 Template".to_string(),
            kind: Some(CompletionItemKind::SNIPPET),
            detail: Some("OWASP A01 vulnerability".to_string()),
            insert_text: Some(
                r#"rule:
  id: "OWASP-A01"
  metadata:
    description: "Broken Access Control"
    severity: "CRITICAL"
  languages: ["python"]
  patterns:
    - pattern: |
        $MODEL.get($X)"#
                    .to_string(),
            ),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            ..Default::default()
        }])
    }
}

impl Default for TemplateHandler {
    fn default() -> Self {
        Self::new()
    }
}
