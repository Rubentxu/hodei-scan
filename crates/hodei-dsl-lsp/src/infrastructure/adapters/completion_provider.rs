//! Completion provider adapter
//!
//! Provides intelligent autocompletion for hodei-scan DSL

use crate::domain::models::{
    CompletionContext, CompletionItem, CursorPosition, Document, 
    CompletionItemKind as DomainCompletionItemKind
};
use crate::domain::ports::CompletionProvider;
use std::collections::HashMap;
use unicode_segmentation::UnicodeSegmentation;

/// Completion provider with contextual intelligence
pub struct HodeiCompletionProvider {
    fact_completions: HashMap<String, CompletionItem>,
    function_completions: HashMap<String, CompletionItem>,
    keyword_completions: HashMap<String, CompletionItem>,
}

impl HodeiCompletionProvider {
    pub fn new() -> Self {
        let mut provider = Self {
            fact_completions: HashMap::new(),
            function_completions: HashMap::new(),
            keyword_completions: HashMap::new(),
        };
        
        provider.initialize_fact_completions();
        provider.initialize_function_completions();
        provider.initialize_keyword_completions();
        
        provider
    }
    
    fn initialize_fact_completions(&mut self) {
        // Vulnerability fact
        self.fact_completions.insert(
            "Vulnerability".to_string(),
            CompletionItem {
                label: "Vulnerability".to_string(),
                kind: DomainCompletionItemKind::Class,
                detail: Some("Security vulnerability fact".to_string()),
                documentation: Some(
                    "Represents a security vulnerability in the code.\n\n\
                     Fields:\n\
                     - severity (Severity): Critical/Major/Minor\n\
                     - message (String): Human-readable description"
                        .to_string(),
                ),
                insert_text: "Vulnerability { ${1:severity: Severity}, ${2:message: String} }".to_string(),
                additional_text_edits: Vec::new(),
            },
        );
        
        // CodeSmell fact
        self.fact_completions.insert(
            "CodeSmell".to_string(),
            CompletionItem {
                label: "CodeSmell".to_string(),
                kind: DomainCompletionItemKind::Class,
                detail: Some("Code quality issue fact".to_string()),
                documentation: Some(
                    "Represents a code quality issue.\n\n\
                     Fields:\n\
                     - type (String): Type of code smell\n\
                     - severity (Severity): Impact level"
                        .to_string(),
                ),
                insert_text: "CodeSmell { ${1:type: String}, ${2:severity: Severity} }".to_string(),
                additional_text_edits: Vec::new(),
            },
        );
        
        // SecurityIssue fact
        self.fact_completions.insert(
            "SecurityIssue".to_string(),
            CompletionItem {
                label: "SecurityIssue".to_string(),
                kind: DomainCompletionItemKind::Class,
                detail: Some("General security issue fact".to_string()),
                documentation: Some(
                    "Represents a general security issue.\n\n\
                     Fields:\n\
                     - category (String): Security category\n\
                     - severity (Severity): Impact level"
                        .to_string(),
                ),
                insert_text: "SecurityIssue { ${1:category: String}, ${2:severity: Severity} }".to_string(),
                additional_text_edits: Vec::new(),
            },
        );
    }
    
    fn initialize_function_completions(&mut self) {
        // Pattern matching functions
        self.function_completions.insert(
            "matches".to_string(),
            CompletionItem {
                label: "matches()".to_string(),
                kind: DomainCompletionItemKind::Function,
                detail: Some("Pattern matching function".to_string()),
                documentation: Some(
                    "Checks if a string matches a regular expression pattern.\n\n\
                     Usage: matches(field, pattern)".to_string(),
                ),
                insert_text: "matches(${1:field}, ${2:pattern})".to_string(),
                additional_text_edits: Vec::new(),
            },
        );
        
        self.function_completions.insert(
            "contains".to_string(),
            CompletionItem {
                label: "contains()".to_string(),
                kind: DomainCompletionItemKind::Function,
                detail: Some("Substring check function".to_string()),
                documentation: Some(
                    "Checks if a string contains a substring.\n\n\
                     Usage: contains(field, substring)".to_string(),
                ),
                insert_text: "contains(${1:field}, ${2:substring})".to_string(),
                additional_text_edits: Vec::new(),
            },
        );
        
        // Length check functions
        self.function_completions.insert(
            "length_gt".to_string(),
            CompletionItem {
                label: "length_gt()".to_string(),
                kind: DomainCompletionItemKind::Function,
                detail: Some("Check if length is greater than".to_string()),
                documentation: Some(
                    "Checks if a string's length is greater than a value.\n\n\
                     Usage: length_gt(field, value)".to_string(),
                ),
                insert_text: "length_gt(${1:field}, ${2:value})".to_string(),
                additional_text_edits: Vec::new(),
            },
        );
    }
    
    fn initialize_keyword_completions(&mut self) {
        self.keyword_completions.insert(
            "rule".to_string(),
            CompletionItem {
                label: "rule".to_string(),
                kind: DomainCompletionItemKind::Keyword,
                detail: Some("Rule declaration keyword".to_string()),
                documentation: Some("Declares a new rule".to_string()),
                insert_text: "rule ${1:name} {\n  ${2:// rule body}\n}".to_string(),
                additional_text_edits: Vec::new(),
            },
        );
        
        self.keyword_completions.insert(
            "when".to_string(),
            CompletionItem {
                label: "when".to_string(),
                kind: DomainCompletionItemKind::Keyword,
                detail: Some("Pattern matching clause".to_string()),
                documentation: Some("Pattern matching clause in a rule".to_string()),
                insert_text: "when {\n  ${1:// condition}\n}".to_string(),
                additional_text_edits: Vec::new(),
            },
        );
    }
}

#[async_trait::async_trait]
impl CompletionProvider for HodeiCompletionProvider {
    async fn provide_completions(
        &self,
        document: &Document,
        context: &CompletionContext,
    ) -> Result<Vec<CompletionItem>, String> {
        let content = &document.content;
        let position = context.position;
        
        // Get text up to cursor position
        let cursor_offset = offset_from_position(content, position);
        let text_before_cursor = &content[..cursor_offset];
        
        // Determine context from text before cursor
        let completions = determine_context_and_completions(
            text_before_cursor,
            self,
        );
        
        Ok(completions)
    }
}

/// Helper function to get byte offset from position
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

/// Determine the current context and return appropriate completions
fn determine_context_and_completions(
    text_before_cursor: &str,
    provider: &HodeiCompletionProvider,
) -> Vec<CompletionItem> {
    // Check if we're after a dot (e.g., "fact.type.")
    let after_dot = text_before_cursor
        .chars()
        .rev()
        .take_while(|&c| c.is_whitespace() || c == '.')
        .collect::<String>()
        .chars()
        .any(|c| c == '.');
    
    if after_dot {
        // After a dot, suggest fact types
        return provider.fact_completions.values().cloned().collect();
    }
    
    // Check if we're typing a keyword or fact type
    // Get the last word
    let last_word = text_before_cursor
        .split_whitespace()
        .last()
        .unwrap_or("");
    
    // If last word looks like a fact type or function, suggest fields
    if provider.fact_completions.contains_key(last_word) {
        return suggest_fact_fields(last_word);
    }
    
    // Default: suggest all completions based on the last word prefix
    let mut all_completions = Vec::new();
    
    // Add fact completions that match the prefix
    for completion in provider.fact_completions.values() {
        if completion.label.starts_with(last_word) {
            all_completions.push(completion.clone());
        }
    }
    
    // Add function completions that match the prefix
    for completion in provider.function_completions.values() {
        if completion.label.starts_with(last_word) {
            all_completions.push(completion.clone());
        }
    }
    
    // Add keyword completions that match the prefix
    for completion in provider.keyword_completions.values() {
        if completion.label.starts_with(last_word) {
            all_completions.push(completion.clone());
        }
    }
    
    // If no matches, show all fact types as default
    if all_completions.is_empty() && last_word.len() <= 1 {
        all_completions.extend(provider.fact_completions.values().cloned());
    }
    
    all_completions
}

/// Suggest fields for a specific fact type
fn suggest_fact_fields(fact_name: &str) -> Vec<CompletionItem> {
    match fact_name {
        "Vulnerability" => vec![
            CompletionItem {
                label: "severity".to_string(),
                kind: DomainCompletionItemKind::Variable,
                detail: Some("Severity level".to_string()),
                documentation: Some("Severity: Critical, Major, or Minor".to_string()),
                insert_text: "severity".to_string(),
                additional_text_edits: Vec::new(),
            },
            CompletionItem {
                label: "message".to_string(),
                kind: DomainCompletionItemKind::Variable,
                detail: Some("Description message".to_string()),
                documentation: Some("Human-readable vulnerability description".to_string()),
                insert_text: "message".to_string(),
                additional_text_edits: Vec::new(),
            },
        ],
        _ => Vec::new(),
    }
}
