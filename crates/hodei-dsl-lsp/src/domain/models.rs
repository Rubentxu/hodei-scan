//! Domain models
//!
//! Core data structures representing concepts in the hodei-scan DSL domain

use lsp_types::Position;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a document open in the LSP server
#[derive(Debug, Clone)]
pub struct Document {
    pub uri: String,
    pub content: String,
    pub version: i32,
}

/// Position in a document (line and column)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CursorPosition {
    pub line: u32,
    pub column: u32,
}

impl From<Position> for CursorPosition {
    fn from(pos: Position) -> Self {
        CursorPosition {
            line: pos.line,
            column: pos.character,
        }
    }
}

/// Context for autocompletion
#[derive(Debug, Clone)]
pub struct CompletionContext {
    pub position: CursorPosition,
    pub trigger_character: Option<char>,
    pub trigger_kind: CompletionTriggerKind,
}

/// Trigger kind for completion requests
#[derive(Debug, Clone)]
pub enum CompletionTriggerKind {
    Invoked,          // Manual trigger (Ctrl+Space)
    TriggerCharacter, // Automatic trigger (e.g., '.')
    TriggerForIncompleteCompletions,
}

/// A completion item suggestion
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: String,
    pub additional_text_edits: Vec<TextEdit>,
}

/// Kind of completion item
#[derive(Debug, Clone)]
pub enum CompletionItemKind {
    Class,
    Function,
    Variable,
    Keyword,
    Snippet,
}

/// Text edit to apply
#[derive(Debug, Clone)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

/// Range in a document
#[derive(Debug, Clone, Copy)]
pub struct Range {
    pub start: CursorPosition,
    pub end: CursorPosition,
}

/// Information displayed on hover
#[derive(Debug, Clone)]
pub struct HoverInfo {
    pub contents: String,
    pub range: Option<Range>,
}

/// Severity levels for diagnostics
#[derive(Debug, Clone, PartialEq)]
pub enum DiagnosticSeverity {
    Error = 1,
    Warning = 2,
    Information = 3,
    Hint = 4,
}

/// A diagnostic message (error, warning, etc.)
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: Range,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: String,
}

/// Documentation for a fact type
#[derive(Debug, Clone)]
pub struct FactDocumentation {
    pub name: String,
    pub description: String,
    pub fields: HashMap<String, FieldDocumentation>,
}

/// Documentation for a fact field
#[derive(Debug, Clone)]
pub struct FieldDocumentation {
    pub name: String,
    pub field_type: String,
    pub description: String,
}

/// Documentation for a function
#[derive(Debug, Clone)]
pub struct FunctionDocumentation {
    pub name: String,
    pub description: String,
    pub usage: String,
    pub parameters: Vec<ParameterDocumentation>,
}

/// Documentation for a function parameter
#[derive(Debug, Clone)]
pub struct ParameterDocumentation {
    pub name: String,
    pub field_type: String,
    pub description: String,
}
