//! DSL Input Validation
//!
//! This module provides validation for DSL input to prevent security vulnerabilities
//! such as injection attacks, excessive resource consumption, and malicious code.

use crate::ast::*;
use crate::error::ParseResult;
use std::collections::HashSet;

/// Validator for DSL input with security constraints
#[derive(Debug, Clone)]
pub struct DSLValidator {
    /// Maximum allowed length of a rule in characters
    pub max_rule_length: usize,
    /// Maximum allowed nesting depth in the AST
    pub max_depth: usize,
    /// Set of allowed function names
    pub allowed_functions: HashSet<String>,
    /// Maximum number of facts that can be queried
    pub max_facts_per_query: usize,
    /// Set of allowed operators
    pub allowed_operators: HashSet<String>,
    /// Maximum depth of field path access
    pub max_field_path_depth: usize,
    /// Whether to allow string literals (potential injection risk)
    pub allow_string_literals: bool,
    /// Maximum length of string literals
    pub max_string_literal_length: usize,
}

impl Default for DSLValidator {
    fn default() -> Self {
        let mut allowed_functions = HashSet::new();
        allowed_functions.insert("count".to_string());
        allowed_functions.insert("exists".to_string());
        allowed_functions.insert("regex_match".to_string());
        allowed_functions.insert("starts_with".to_string());
        allowed_functions.insert("ends_with".to_string());

        let mut allowed_operators = HashSet::new();
        allowed_operators.insert("==".to_string());
        allowed_operators.insert("!=".to_string());
        allowed_operators.insert("<".to_string());
        allowed_operators.insert("<=".to_string());
        allowed_operators.insert(">".to_string());
        allowed_operators.insert(">=".to_string());
        allowed_operators.insert("&&".to_string());
        allowed_operators.insert("||".to_string());
        allowed_operators.insert("!".to_string());

        Self {
            max_rule_length: 10_000,
            max_depth: 20,
            allowed_functions,
            max_facts_per_query: 1_000_000,
            allowed_operators,
            max_field_path_depth: 5,
            allow_string_literals: true,
            max_string_literal_length: 1_000,
        }
    }
}

impl DSLValidator {
    /// Create a new validator with default security settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate a complete rule file
    pub fn validate_rule_file(&self, rule_file: &RuleFile) -> ParseResult<()> {
        if rule_file.rules.is_empty() {
            return Err(crate::error::ParseError::msg(
                "Rule file must contain at least one rule",
            ));
        }

        if rule_file.rules.len() > 1000 {
            return Err(crate::error::ParseError::msg(
                "Rule file cannot contain more than 1000 rules",
            ));
        }

        for (i, rule) in rule_file.rules.iter().enumerate() {
            self.validate_rule(rule)
                .map_err(|e| crate::error::ParseError::msg(format!("Rule {}: {}", i + 1, e)))?;
        }

        Ok(())
    }

    /// Validate a single rule
    pub fn validate_rule(&self, rule: &RuleDef) -> ParseResult<()> {
        // Check rule name
        if rule.name.is_empty() {
            return Err(crate::error::ParseError::msg("Rule name cannot be empty"));
        }

        if rule.name.len() > 100 {
            return Err(crate::error::ParseError::msg(
                "Rule name cannot exceed 100 characters",
            ));
        }

        // Check for path traversal in rule name
        if rule.name.contains("..") || rule.name.contains('/') {
            return Err(crate::error::ParseError::msg(
                "Rule name cannot contain path traversal sequences",
            ));
        }

        // Validate match block
        self.validate_match_block(&rule.match_block, 0)?;

        // Validate emit block
        self.validate_emit_block(&rule.emit_block, 0)?;

        Ok(())
    }

    /// Validate match block
    fn validate_match_block(&self, match_block: &MatchBlock, depth: usize) -> ParseResult<()> {
        if depth > self.max_depth {
            return Err(crate::error::ParseError::msg(format!(
                "Maximum nesting depth exceeded (max: {})",
                self.max_depth
            )));
        }

        for pattern in &match_block.patterns {
            self.validate_pattern(pattern, depth + 1)?;
        }

        if let Some(where_clause) = &match_block.where_clause {
            self.validate_expression(where_clause, depth + 1)?;
        }

        Ok(())
    }

    /// Validate emit block
    fn validate_emit_block(&self, emit_block: &EmitBlock, depth: usize) -> ParseResult<()> {
        if depth > self.max_depth {
            return Err(crate::error::ParseError::msg(format!(
                "Maximum nesting depth exceeded (max: {})",
                self.max_depth
            )));
        }

        if emit_block.message_template.len() > 1000 {
            return Err(crate::error::ParseError::msg(
                "Message template cannot exceed 1000 characters",
            ));
        }

        Ok(())
    }

    /// Validate a pattern
    fn validate_pattern(&self, pattern: &Pattern, depth: usize) -> ParseResult<()> {
        // Check binding name
        if pattern.binding.is_empty() {
            return Err(crate::error::ParseError::msg(
                "Pattern binding cannot be empty",
            ));
        }

        if pattern.binding.len() > 50 {
            return Err(crate::error::ParseError::msg(
                "Pattern binding cannot exceed 50 characters",
            ));
        }

        // Check for path traversal in binding
        if pattern.binding.contains("..") || pattern.binding.contains('/') {
            return Err(crate::error::ParseError::msg(
                "Pattern binding cannot contain path traversal sequences",
            ));
        }

        // Check fact type
        if pattern.fact_type.is_empty() {
            return Err(crate::error::ParseError::msg("Fact type cannot be empty"));
        }

        if pattern.fact_type.len() > 100 {
            return Err(crate::error::ParseError::msg(
                "Fact type cannot exceed 100 characters",
            ));
        }

        // Validate conditions
        for condition in &pattern.conditions {
            self.validate_path(&condition.path, depth + 1)?;
            self.validate_literal(&condition.value, depth + 1)?;
        }

        Ok(())
    }

    /// Validate path
    fn validate_path(&self, path: &Path, depth: usize) -> ParseResult<()> {
        if path.segments.len() > self.max_field_path_depth {
            return Err(crate::error::ParseError::msg(format!(
                "Field path depth exceeds maximum (max: {}, found: {})",
                self.max_field_path_depth,
                path.segments.len()
            )));
        }

        for part in &path.segments {
            if part.is_empty() {
                return Err(crate::error::ParseError::msg(
                    "Field path cannot contain empty segments",
                ));
            }

            // Check for path traversal
            if part.contains("..") || part.contains('/') {
                return Err(crate::error::ParseError::msg(
                    "Field path cannot contain path traversal sequences",
                ));
            }

            // Check for SQL injection patterns
            let part_lower = part.to_lowercase();
            if part_lower.contains("select")
                || part_lower.contains("drop")
                || part_lower.contains("delete")
                || part_lower.contains("insert")
            {
                return Err(crate::error::ParseError::msg(
                    "Field path contains potentially dangerous SQL keywords",
                ));
            }
        }

        Ok(())
    }

    /// Validate literal value
    fn validate_literal(&self, literal: &Literal, depth: usize) -> ParseResult<()> {
        match literal {
            Literal::String(s) => {
                if !self.allow_string_literals {
                    return Err(crate::error::ParseError::msg(
                        "String literals are not allowed",
                    ));
                }

                if s.len() > self.max_string_literal_length {
                    return Err(crate::error::ParseError::msg(format!(
                        "String literal exceeds maximum length (max: {})",
                        self.max_string_literal_length
                    )));
                }

                // Check for potential injection patterns
                if s.contains("{{") || s.contains("}}") {
                    return Err(crate::error::ParseError::msg(
                        "String literal contains template syntax which is not allowed",
                    ));
                }

                // Check for command execution patterns
                if s.contains('$') && s.contains('(') {
                    return Err(crate::error::ParseError::msg(
                        "String literal contains potential command execution pattern",
                    ));
                }
            }
            Literal::Number(_) => {
                // Numbers are safe
            }
            Literal::Boolean(_) => {
                // Booleans are safe
            }
            Literal::Null => {
                // Null is safe
            }
        }

        Ok(())
    }

    /// Validate expression
    fn validate_expression(&self, expr: &Expr, depth: usize) -> ParseResult<()> {
        if depth > self.max_depth {
            return Err(crate::error::ParseError::msg(
                "Maximum expression depth exceeded",
            ));
        }

        match expr {
            Expr::Literal(literal) => {
                self.validate_literal(literal, depth)?;
            }
            Expr::Path(path) => {
                self.validate_path(path, depth)?;
            }
            Expr::FunctionCall { name, args } => {
                // Check if function is allowed
                if !self.allowed_functions.contains(name) {
                    return Err(crate::error::ParseError::msg(format!(
                        "Function '{}' is not allowed",
                        name
                    )));
                }

                if args.len() > 10 {
                    return Err(crate::error::ParseError::msg(
                        "Function call cannot have more than 10 arguments",
                    ));
                }

                for arg in args {
                    self.validate_expression(arg, depth + 1)?;
                }
            }
            Expr::Binary { op, left, right } => {
                // Check if operator is allowed
                let op_str = match op {
                    BinaryOp::Eq => "==",
                    BinaryOp::Ne => "!=",
                    BinaryOp::Lt => "<",
                    BinaryOp::Gt => ">",
                    BinaryOp::Le => "<=",
                    BinaryOp::Ge => ">=",
                    BinaryOp::And => "&&",
                    BinaryOp::Or => "||",
                };

                if !self.allowed_operators.contains(op_str) {
                    return Err(crate::error::ParseError::msg(format!(
                        "Operator '{}' is not allowed",
                        op_str
                    )));
                }

                self.validate_expression(left, depth + 1)?;
                self.validate_expression(right, depth + 1)?;
            }
        }

        Ok(())
    }

    /// Validate raw DSL text before parsing
    pub fn validate_raw_input(&self, input: &str) -> ParseResult<()> {
        if input.is_empty() {
            return Err(crate::error::ParseError::msg("Input cannot be empty"));
        }

        if input.len() > self.max_rule_length {
            return Err(crate::error::ParseError::msg(format!(
                "Input exceeds maximum length (max: {} characters)",
                self.max_rule_length
            )));
        }

        // Check for null bytes
        if input.contains('\0') {
            return Err(crate::error::ParseError::msg(
                "Input contains null bytes which are not allowed",
            ));
        }

        // Check for control characters (excluding newlines, tabs, carriage return)
        for (i, c) in input.char_indices() {
            if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
                return Err(crate::error::ParseError::msg(format!(
                    "Input contains disallowed control character at position {}",
                    i
                )));
            }
        }

        // Check for potential code injection patterns
        let suspicious_patterns = [
            ("script", "Script tags are not allowed"),
            ("javascript:", "JavaScript protocol is not allowed"),
            ("data:", "Data URIs are not allowed"),
            ("vbscript:", "VBScript protocol is not allowed"),
            ("..", "Path traversal sequences are not allowed"),
            ("{{", "Template syntax is not allowed"),
            ("}}", "Template syntax is not allowed"),
            ("$(", "Command execution syntax is not allowed"),
        ];

        let input_lower = input.to_lowercase();
        for (pattern, message) in &suspicious_patterns {
            if input_lower.contains(pattern) {
                return Err(crate::error::ParseError::msg(*message));
            }
        }

        Ok(())
    }
}
