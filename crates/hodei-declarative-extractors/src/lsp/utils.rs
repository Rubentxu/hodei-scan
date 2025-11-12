//! Utility functions for LSP implementation
//!
//! This module provides helper functions for LSP features like
//! position mapping, range calculation, and document parsing.

use lsp_types::*;

/// Convert byte offset to LSP position
pub fn byte_offset_to_position(content: &str, byte_offset: usize) -> Position {
    let lines: Vec<&str> = content.lines().collect();
    let mut current_offset = 0;
    let mut line = 0;
    let mut character = 0;

    for (i, line_text) in lines.iter().enumerate() {
        let line_end = current_offset + line_text.len() + 1; // +1 for newline

        if byte_offset >= current_offset && byte_offset < line_end {
            line = i as u32;
            character = (byte_offset - current_offset) as u32;
            break;
        }

        current_offset = line_end;
    }

    Position { line, character }
}

/// Convert LSP position to byte offset
pub fn position_to_byte_offset(content: &str, position: Position) -> usize {
    let lines: Vec<&str> = content.lines().collect();

    if position.line as usize >= lines.len() {
        return content.len();
    }

    let mut byte_offset = 0;

    for i in 0..position.line as usize {
        byte_offset += lines[i].len() + 1; // +1 for newline
    }

    byte_offset += position.character as usize;
    byte_offset
}

/// Check if a position is within a range
pub fn is_position_in_range(position: Position, range: Range) -> bool {
    if position.line < range.start.line || position.line > range.end.line {
        return false;
    }

    if position.line == range.start.line && position.character < range.start.character {
        return false;
    }

    if position.line == range.end.line && position.character > range.end.character {
        return false;
    }

    true
}

/// Calculate line and column from byte offset
pub fn byte_offset_to_line_column(content: &str, byte_offset: usize) -> (u32, u32) {
    let lines: Vec<&str> = content.lines().collect();
    let mut current_offset = 0;
    let mut line = 0;
    let mut column = 0;

    for (i, line_text) in lines.iter().enumerate() {
        let line_end = current_offset + line_text.len() + 1;

        if byte_offset >= current_offset && byte_offset < line_end {
            line = i as u32;
            column = (byte_offset - current_offset) as u32;
            break;
        }

        current_offset = line_end;
    }

    (line, column)
}

/// Get text in a range
pub fn get_text_in_range(content: &str, range: Range) -> String {
    let lines: Vec<&str> = content.lines().collect();

    if range.start.line == range.end.line {
        let line = lines[range.start.line as usize];
        let start = range.start.character as usize;
        let end = range.end.character as usize;
        return line[start..end].to_string();
    }

    let mut result = String::new();

    // First line
    let first_line = lines[range.start.line as usize];
    result.push_str(&first_line[range.start.character as usize..]);
    result.push('\n');

    // Middle lines
    for i in (range.start.line + 1)..range.end.line {
        result.push_str(lines[i as usize]);
        result.push('\n');
    }

    // Last line
    let last_line = lines[range.end.line as usize];
    result.push_str(&last_line[..range.end.character as usize]);

    result
}

/// Parse YAML document and return structured data
pub fn parse_yaml_document(content: &str) -> Result<serde_yaml::Value, String> {
    serde_yaml::from_str(content).map_err(|e| format!("YAML parse error: {}", e))
}

/// Extract rule ID from YAML
pub fn extract_rule_id(yaml: &serde_yaml::Value) -> Option<String> {
    yaml.get("rule")?.get("id")?.as_str().map(|s| s.to_string())
}

/// Extract supported languages from YAML
pub fn extract_languages(yaml: &serde_yaml::Value) -> Option<Vec<String>> {
    let langs = yaml.get("rule")?.get("languages")?;

    let mut result = Vec::new();
    for lang in langs.as_sequence()? {
        if let Some(s) = lang.as_str() {
            result.push(s.to_string());
        }
    }
    Some(result)
}

/// Format severity level
pub fn format_severity(severity: &str) -> String {
    match severity.to_uppercase().as_str() {
        "INFO" => "ℹ Info".to_string(),
        "MINOR" => "⚠ Minor".to_string(),
        "MAJOR" => "⚠ Major".to_string(),
        "CRITICAL" => "🔴 Critical".to_string(),
        "BLOCKER" => "⛔ Blocker".to_string(),
        _ => format!("{}", severity),
    }
}

/// Get completion trigger characters for YAML
pub fn get_yaml_trigger_characters() -> Vec<String> {
    vec![
        "\"".to_string(), // String value
        ":".to_string(),  // Key-value separator
        "-".to_string(),  // List item
        "$".to_string(),  // Metavariable
        "[".to_string(),  // List start
        "{".to_string(),  // Mapping start
    ]
}
