//! Hover provider adapter
//!
//! Provides hover documentation for hodei-scan DSL elements

use crate::domain::models::{CursorPosition, Document, HoverInfo};
use crate::domain::ports::HoverProvider;
use std::collections::HashMap;

/// Provider for hover information
pub struct HodeiHoverProvider {
    fact_docs: HashMap<String, FactDoc>,
    function_docs: HashMap<String, FunctionDoc>,
}

#[derive(Clone)]
struct FactDoc {
    name: String,
    description: String,
    fields: Vec<FieldDoc>,
}

#[derive(Clone)]
struct FieldDoc {
    name: String,
    field_type: String,
    description: String,
}

#[derive(Clone)]
struct FunctionDoc {
    name: String,
    description: String,
    usage: String,
    parameters: Vec<ParameterDoc>,
}

#[derive(Clone)]
struct ParameterDoc {
    name: String,
    field_type: String,
    description: String,
}

impl HodeiHoverProvider {
    pub fn new() -> Self {
        let mut provider = Self {
            fact_docs: HashMap::new(),
            function_docs: HashMap::new(),
        };
        
        provider.initialize_fact_docs();
        provider.initialize_function_docs();
        
        provider
    }
    
    fn initialize_fact_docs(&mut self) {
        self.fact_docs.insert(
            "Vulnerability".to_string(),
            FactDoc {
                name: "Vulnerability".to_string(),
                description: "Represents a security vulnerability detected in the code".to_string(),
                fields: vec![
                    FieldDoc {
                        name: "severity".to_string(),
                        field_type: "Severity".to_string(),
                        description: "Severity level of the vulnerability".to_string(),
                    },
                    FieldDoc {
                        name: "message".to_string(),
                        field_type: "String".to_string(),
                        description: "Human-readable description of the vulnerability".to_string(),
                    },
                ],
            },
        );
        
        self.fact_docs.insert(
            "CodeSmell".to_string(),
            FactDoc {
                name: "CodeSmell".to_string(),
                description: "Represents a code quality issue".to_string(),
                fields: vec![
                    FieldDoc {
                        name: "type".to_string(),
                        field_type: "String".to_string(),
                        description: "Type of code smell".to_string(),
                    },
                    FieldDoc {
                        name: "severity".to_string(),
                        field_type: "Severity".to_string(),
                        description: "Impact level of the code smell".to_string(),
                    },
                ],
            },
        );
        
        self.fact_docs.insert(
            "SecurityIssue".to_string(),
            FactDoc {
                name: "SecurityIssue".to_string(),
                description: "Represents a general security issue".to_string(),
                fields: vec![
                    FieldDoc {
                        name: "category".to_string(),
                        field_type: "String".to_string(),
                        description: "Security issue category".to_string(),
                    },
                    FieldDoc {
                        name: "severity".to_string(),
                        field_type: "Severity".to_string(),
                        description: "Severity level of the security issue".to_string(),
                    },
                ],
            },
        );
    }
    
    fn initialize_function_docs(&mut self) {
        self.function_docs.insert(
            "matches".to_string(),
            FunctionDoc {
                name: "matches".to_string(),
                description: "Checks if a string matches a regular expression pattern".to_string(),
                usage: "matches(field, pattern)".to_string(),
                parameters: vec![
                    ParameterDoc {
                        name: "field".to_string(),
                        field_type: "String".to_string(),
                        description: "The string field to check".to_string(),
                    },
                    ParameterDoc {
                        name: "pattern".to_string(),
                        field_type: "String".to_string(),
                        description: "Regular expression pattern".to_string(),
                    },
                ],
            },
        );
        
        self.function_docs.insert(
            "contains".to_string(),
            FunctionDoc {
                name: "contains".to_string(),
                description: "Checks if a string contains a substring".to_string(),
                usage: "contains(field, substring)".to_string(),
                parameters: vec![
                    ParameterDoc {
                        name: "field".to_string(),
                        field_type: "String".to_string(),
                        description: "The string field to check".to_string(),
                    },
                    ParameterDoc {
                        name: "substring".to_string(),
                        field_type: "String".to_string(),
                        description: "The substring to search for".to_string(),
                    },
                ],
            },
        );
        
        self.function_docs.insert(
            "length_gt".to_string(),
            FunctionDoc {
                name: "length_gt".to_string(),
                description: "Checks if a string's length is greater than a value".to_string(),
                usage: "length_gt(field, value)".to_string(),
                parameters: vec![
                    ParameterDoc {
                        name: "field".to_string(),
                        field_type: "String".to_string(),
                        description: "The string field to check".to_string(),
                    },
                    ParameterDoc {
                        name: "value".to_string(),
                        field_type: "Integer".to_string(),
                        description: "Minimum length value".to_string(),
                    },
                ],
            },
        );
    }
}

#[async_trait::async_trait]
impl HoverProvider for HodeiHoverProvider {
    async fn provide_hover(
        &self,
        document: &Document,
        position: CursorPosition,
    ) -> Result<Option<HoverInfo>, String> {
        let content = &document.content;
        
        // Get the token at the cursor position
        let cursor_offset = offset_from_position(content, position);
        let (token_start, token_end, token_text) = get_token_at_position(content, cursor_offset);
        
        // Check if token is a fact type
        if let Some(fact_doc) = self.fact_docs.get(&token_text) {
            return Ok(Some(HoverInfo {
                contents: format_fact_hover(fact_doc),
                range: Some(crate::domain::models::Range {
                    start: position_from_offset(content, token_start),
                    end: position_from_offset(content, token_end),
                }),
            }));
        }
        
        // Check if token is a function
        if let Some(func_doc) = self.function_docs.get(&token_text) {
            return Ok(Some(HoverInfo {
                contents: format_function_hover(func_doc),
                range: Some(crate::domain::models::Range {
                    start: position_from_offset(content, token_start),
                    end: position_from_offset(content, token_end),
                }),
            }));
        }
        
        Ok(None)
    }
}

/// Format fact documentation for hover
fn format_fact_hover(fact: &FactDoc) -> String {
    let fields_md = if !fact.fields.is_empty() {
        format!(
            "\n\n### Fields\n{}",
            fact.fields
                .iter()
                .map(|f| format!("- `{}` ({}) - {}", f.name, f.field_type, f.description))
                .collect::<Vec<_>>()
                .join("\n")
        )
    } else {
        String::new()
    };
    
    format!("# {}\n{}{}", fact.name, fact.description, fields_md)
}

/// Format function documentation for hover
fn format_function_hover(func: &FunctionDoc) -> String {
    let params_md = if !func.parameters.is_empty() {
        format!(
            "\n\n### Parameters\n{}",
            func.parameters
                .iter()
                .map(|p| format!("- `{}` ({}) - {}", p.name, p.field_type, p.description))
                .collect::<Vec<_>>()
                .join("\n")
        )
    } else {
        String::new()
    };
    
    format!(
        "# {}\n\n### Usage\n```\n{}\n```\n\n{}{}",
        func.name, func.usage, func.description, params_md
    )
}

/// Get offset from position
fn offset_from_position(content: &str, position: CursorPosition) -> usize {
    let mut offset = 0;
    let mut current_line = 0;
    let mut current_col = 0;
    
    for (byte_idx, ch) in content.char_indices() {
        if current_line == position.line {
            if current_col == position.column {
                return byte_idx;
            }
            current_col += 1;
        } else if ch == '\n' {
            current_line += 1;
            current_col = 0;
        }
    }
    
    content.len()
}

/// Get token at position
fn get_token_at_position(content: &str, cursor_offset: usize) -> (usize, usize, String) {
    let start = find_token_start(content, cursor_offset);
    let end = find_token_end(content, cursor_offset);
    let token = content[start..end].to_string();
    (start, end, token)
}

/// Find the start of the token at the cursor
fn find_token_start(content: &str, cursor_offset: usize) -> usize {
    let mut start = cursor_offset;
    
    // Move backwards to find start of word
    for i in (0..cursor_offset).rev() {
        let ch = content.chars().nth(i).unwrap();
        if !ch.is_alphanumeric() && ch != '_' && ch != '-' {
            return i + 1;
        }
        start = i;
    }
    
    start
}

/// Find the end of the token at the cursor
fn find_token_end(content: &str, cursor_offset: usize) -> usize {
    let mut end = cursor_offset;
    
    // Move forwards to find end of word
    for (i, ch) in content.char_indices().skip(cursor_offset) {
        if !ch.is_alphanumeric() && ch != '_' && ch != '-' {
            return i;
        }
        end = i + ch.len_utf8();
    }
    
    end
}

/// Convert byte offset to position
fn position_from_offset(content: &str, offset: usize) -> CursorPosition {
    let mut line = 0;
    let mut col = 0;
    
    for (i, ch) in content.char_indices() {
        if i >= offset {
            return CursorPosition { line, column: col };
        }
        if ch == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    
    CursorPosition { line, column: col }
}
