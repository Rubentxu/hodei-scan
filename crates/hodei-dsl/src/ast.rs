//! Abstract Syntax Tree (AST) types for Hodei DSL

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A complete rule file containing multiple rules
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleFile {
    pub rules: Vec<RuleDef>,
}

/// Definition of a rule with metadata, match patterns, and emit block
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleDef {
    pub name: String,
    pub metadata: Metadata,
    pub match_block: MatchBlock,
    pub emit_block: EmitBlock,
    pub span: Span,
}

/// Metadata for a rule (description, severity, tags)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub description: Option<String>,
    pub severity: Severity,
    pub tags: Vec<String>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            description: None,
            severity: Severity::Medium,
            tags: vec![],
        }
    }
}

/// Severity levels for rules and findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Match block containing patterns and optional where clause
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchBlock {
    pub patterns: Vec<Pattern>,
    pub where_clause: Option<Expr>,
}

/// A pattern matching a specific fact type
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pattern {
    pub binding: String,   // Variable name (e.g., "sink")
    pub fact_type: String, // FactType name (e.g., "TaintSink")
    pub conditions: Vec<Condition>,
    pub span: Span,
}

/// A condition comparing a path to a literal value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Condition {
    pub path: Path,
    pub op: ComparisonOp,
    pub value: Literal,
}

/// Expression tree
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

/// A path accessing a field (e.g., "sink.location.file")
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Path {
    pub segments: Vec<String>,
    pub span: Span,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    Eq,
    Ne,
    Contains,
    Matches,
}

/// Literal values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

/// Emit block defining what to output when rule matches
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmitBlock {
    pub message_template: String,
    pub confidence: Confidence,
    pub metadata: HashMap<String, Literal>,
}

/// Confidence levels for findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

/// Span for error reporting (start and end positions in source)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// Create a new span from a pest pair
    pub fn from_pair(_pair: &impl std::fmt::Debug) -> Self {
        // Simplified - return a zero span for now
        // In production, we'd extract real positions
        Self { start: 0, end: 0 }
    }
}
