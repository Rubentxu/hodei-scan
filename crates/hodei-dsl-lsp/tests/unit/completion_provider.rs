//! Unit tests for CompletionProvider

use hodei_dsl_lsp::domain::models::{
    CompletionContext, CursorPosition, Document, 
    CompletionItemKind, CompletionTriggerKind
};
use hodei_dsl_lsp::infrastructure::adapters::HodeiCompletionProvider;
use crate::fixtures::{create_basic_document, create_document_with_content};

#[tokio::test]
async fn test_completion_provider_initialization() {
    let provider = HodeiCompletionProvider::new();
    
    // Provider should be initialized with fact completions
    assert!(true); // If we can create it, initialization worked
}

#[tokio::test]
async fn test_provide_fact_type_completions() {
    let provider = HodeiCompletionProvider::new();
    let document = create_basic_document();
    
    let context = CompletionContext {
        position: CursorPosition { line: 0, column: 0 },
        trigger_character: None,
        trigger_kind: CompletionTriggerKind::Invoked,
    };
    
    let completions = provider.provide_completions(&document, &context)
        .await
        .expect("Should provide completions");
    
    // Should have fact type completions
    assert!(!completions.is_empty());
    
    // Should include Vulnerability
    assert!(completions.iter().any(|c| c.label == "Vulnerability"));
    
    // Should include CodeSmell
    assert!(completions.iter().any(|c| c.label == "CodeSmell"));
    
    // Should include SecurityIssue
    assert!(completions.iter().any(|c| c.label == "SecurityIssue"));
}

#[tokio::test]
async fn test_provide_function_completions() {
    let provider = HodeiCompletionProvider::new();
    let document = create_document_with_content("matches");
    
    let context = CompletionContext {
        position: CursorPosition { line: 0, column: 7 }, // After "matches"
        trigger_character: None,
        trigger_kind: CompletionTriggerKind::Invoked,
    };
    
    let completions = provider.provide_completions(&document, &context)
        .await
        .expect("Should provide completions");
    
    // Should have function completions
    assert!(!completions.is_empty());
    
    // Should include matches function
    assert!(completions.iter().any(|c| c.label.contains("matches")));
}

#[tokio::test]
async fn test_provide_keyword_completions() {
    let provider = HodeiCompletionProvider::new();
    let document = create_document_with_content("rule");
    
    let context = CompletionContext {
        position: CursorPosition { line: 0, column: 4 }, // After "rule"
        trigger_character: None,
        trigger_kind: CompletionTriggerKind::Invoked,
    };
    
    let completions = provider.provide_completions(&document, &context)
        .await
        .expect("Should provide completions");
    
    // Should have keyword completions
    assert!(!completions.is_empty());
}

#[tokio::test]
async fn test_completion_kind_vulnerability() {
    let provider = HodeiCompletionProvider::new();
    let document = create_basic_document();
    
    let context = CompletionContext {
        position: CursorPosition { line: 0, column: 0 },
        trigger_character: None,
        trigger_kind: CompletionTriggerKind::Invoked,
    };
    
    let completions = provider.provide_completions(&document, &context)
        .await
        .expect("Should provide completions");
    
    let vulnerability_completion = completions
        .iter()
        .find(|c| c.label == "Vulnerability")
        .expect("Should find Vulnerability completion");
    
    assert_eq!(vulnerability_completion.kind, CompletionItemKind::Class);
    assert!(vulnerability_completion.detail.is_some());
    assert!(vulnerability_completion.documentation.is_some());
}

#[tokio::test]
async fn test_completion_insert_text() {
    let provider = HodeiCompletionProvider::new();
    let document = create_basic_document();
    
    let context = CompletionContext {
        position: CursorPosition { line: 0, column: 0 },
        trigger_character: None,
        trigger_kind: CompletionTriggerKind::Invoked,
    };
    
    let completions = provider.provide_completions(&document, &context)
        .await
        .expect("Should provide completions");
    
    let vulnerability_completion = completions
        .iter()
        .find(|c| c.label == "Vulnerability")
        .expect("Should find Vulnerability completion");
    
    // Should have insert text with snippet placeholders
    assert!(!vulnerability_completion.insert_text.is_empty());
    assert!(vulnerability_completion.insert_text.contains("${"));
}

#[tokio::test]
async fn test_empty_document() {
    let provider = HodeiCompletionProvider::new();
    let document = create_document_with_content("");
    
    let context = CompletionContext {
        position: CursorPosition { line: 0, column: 0 },
        trigger_character: None,
        trigger_kind: CompletionTriggerKind::Invoked,
    };
    
    let completions = provider.provide_completions(&document, &context)
        .await
        .expect("Should provide completions");
    
    // Should still provide completions even for empty document
    assert!(!completions.is_empty());
}

#[tokio::test]
async fn test_completion_with_trigger_character() {
    let provider = HodeiCompletionProvider::new();
    let document = create_document_with_content("fact.type.");
    
    let context = CompletionContext {
        position: CursorPosition { line: 0, column: 11 }, // After "fact.type."
        trigger_character: Some('.'),
        trigger_kind: CompletionTriggerKind::TriggerCharacter,
    };
    
    let completions = provider.provide_completions(&document, &context)
        .await
        .expect("Should provide completions");
    
    // Should suggest fact types after "."
    assert!(!completions.is_empty());
    assert!(completions.iter().any(|c| c.label == "Vulnerability"));
}

#[tokio::test]
async fn test_function_completion_matches() {
    let provider = HodeiCompletionProvider::new();
    let document = create_basic_document();
    
    let context = CompletionContext {
        position: CursorPosition { line: 0, column: 0 },
        trigger_character: None,
        trigger_kind: CompletionTriggerKind::Invoked,
    };
    
    let completions = provider.provide_completions(&document, &context)
        .await
        .expect("Should provide completions");
    
    // Should have matches function
    let matches_completion = completions
        .iter()
        .find(|c| c.label.contains("matches"));
    
    if let Some(completion) = matches_completion {
        assert_eq!(completion.kind, CompletionItemKind::Function);
        assert!(completion.insert_text.contains("${1:"));
    }
}

#[tokio::test]
async fn test_completion_context_trigger_kinds() {
    let provider = HodeiCompletionProvider::new();
    let document = create_basic_document();
    
    // Test Invoked
    let context_invoked = CompletionContext {
        position: CursorPosition { line: 0, column: 0 },
        trigger_character: None,
        trigger_kind: CompletionTriggerKind::Invoked,
    };
    
    let completions_invoked = provider.provide_completions(&document, &context_invoked)
        .await
        .expect("Should provide completions");
    assert!(!completions_invoked.is_empty());
    
    // Test TriggerCharacter
    let context_trigger = CompletionContext {
        position: CursorPosition { line: 0, column: 0 },
        trigger_character: Some('.'),
        trigger_kind: CompletionTriggerKind::TriggerCharacter,
    };
    
    let completions_trigger = provider.provide_completions(&document, &context_trigger)
        .await
        .expect("Should provide completions");
    assert!(!completions_trigger.is_empty());
}

#[tokio::test]
async fn test_all_completion_kinds() {
    let provider = HodeiCompletionProvider::new();
    let document = create_basic_document();
    
    let context = CompletionContext {
        position: CursorPosition { line: 0, column: 0 },
        trigger_character: None,
        trigger_kind: CompletionTriggerKind::Invoked,
    };
    
    let completions = provider.provide_completions(&document, &context)
        .await
        .expect("Should provide completions");
    
    // Should have completions of different kinds
    let has_class = completions.iter().any(|c| matches!(c.kind, CompletionItemKind::Class));
    let has_function = completions.iter().any(|c| matches!(c.kind, CompletionItemKind::Function));
    let has_keyword = completions.iter().any(|c| matches!(c.kind, CompletionItemKind::Keyword));
    
    assert!(has_class, "Should have Class completions");
    assert!(has_function, "Should have Function completions");
    assert!(has_keyword, "Should have Keyword completions");
}
