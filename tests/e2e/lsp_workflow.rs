//! End-to-End tests for LSP workflow

use hodei_dsl_lsp::domain::models::{
    CompletionContext, CompletionTriggerKind, CursorPosition, Document,
};
use hodei_dsl_lsp::infrastructure::{
    adapters::{HodeiCompletionProvider, HodeiHoverProvider},
    server::HodeiDslServer,
};
use lsp_types::Url;
use std::str::FromStr;

#[tokio::test]
async fn test_complete_rule_authoring_workflow() {
    // Simulate a developer writing a complete rule
    let server = HodeiDslServer::new();
    let completion_provider = HodeiCompletionProvider::new();
    let hover_provider = HodeiHoverProvider::new();

    let uri = Url::from_str("file:///password_rule.hodei").unwrap();

    // Step 1: Start with empty document
    let doc1 = Document {
        uri: uri.to_string(),
        content: "".to_string(),
        version: 1,
    };
    server.store_document(doc1).await;

    // Step 2: Start typing "rule"
    let doc2 = Document {
        uri: uri.to_string(),
        content: "rule".to_string(),
        version: 2,
    };
    server.store_document(doc2.clone()).await;

    // Get completions for "rule"
    let completions = completion_provider
        .provide_completions(
            &server.get_document(uri).await.unwrap(),
            &CompletionContext {
                position: CursorPosition { line: 0, column: 4 },
                trigger_character: None,
                trigger_kind: CompletionTriggerKind::Invoked,
            },
        )
        .await
        .unwrap();

    // Should have keyword completions including "rule"
    assert!(!completions.is_empty());

    // Step 3: Complete the rule definition
    let doc3 = Document {
        uri: uri.to_string(),
        content: r#"
rule password_strength {
  when {
    function validatePassword(pwd: string): boolean {
      return pwd.length >= 8;
    }
  }
  then {
    emit CodeSmell {
      type: "weak_password",
      severity: "Major"
    };
  }
}
"#
        .to_string(),
        version: 3,
    };
    server.store_document(doc3).await;

    // Step 4: Get hover info for "Vulnerability" (not in this rule, but test hover)
    let hover = hover_provider
        .provide_hover(
            &server.get_document(uri).await.unwrap(),
            CursorPosition { line: 0, column: 0 },
        )
        .await
        .unwrap();

    // Should find hover info for fact types
    if let Some(info) = hover {
        assert!(info.contents.contains("Vulnerability") || info.contents.contains("CodeSmell"));
    }
}

#[tokio::test]
async fn test_document_edit_and_completion() {
    let server = HodeiDslServer::new();
    let completion_provider = HodeiCompletionProvider::new();

    let uri = Url::from_str("file:///interactive.hodei").unwrap();

    // Create initial document
    let mut doc = Document {
        uri: uri.to_string(),
        content: "fact.".to_string(),
        version: 1,
    };
    server.store_document(doc.clone()).await;

    // After typing "fact.", get completions
    let completions = completion_provider
        .provide_completions(
            &server.get_document(uri).await.unwrap(),
            &CompletionContext {
                position: CursorPosition { line: 0, column: 5 },
                trigger_character: Some('.'),
                trigger_kind: CompletionTriggerKind::TriggerCharacter,
            },
        )
        .await
        .unwrap();

    // Should suggest fact types
    assert!(!completions.is_empty());
    assert!(completions.iter().any(|c| c.label == "Vulnerability"));
    assert!(completions.iter().any(|c| c.label == "CodeSmell"));

    // Now complete with "Vulnerability"
    doc.content = "fact.Vulnerability {".to_string();
    doc.version = 2;
    server.store_document(doc).await;

    // Get completions for fields
    let field_completions = completion_provider
        .provide_completions(
            &server.get_document(uri).await.unwrap(),
            &CompletionContext {
                position: CursorPosition {
                    line: 0,
                    column: 18,
                },
                trigger_character: None,
                trigger_kind: CompletionTriggerKind::Invoked,
            },
        )
        .await
        .unwrap();

    // Should have suggestions (may include fields for Vulnerability)
    assert!(!field_completions.is_empty());
}

#[tokio::test]
async fn test_multiple_documents_lsp_session() {
    let server = HodeiDslServer::new();
    let completion_provider = HodeiCompletionProvider::new();

    // Open multiple documents
    for i in 0..5 {
        let uri = Url::from_str(&format!("file:///doc{}.hodei", i)).unwrap();
        let content = format!(
            "rule rule_{} {{ emit {} {{ }} }}",
            i,
            if i % 2 == 0 {
                "Vulnerability"
            } else {
                "CodeSmell"
            }
        );

        let doc = Document {
            uri: uri.to_string(),
            content,
            version: 1,
        };
        server.store_document(doc).await;
    }

    // Test completions work on each document
    for i in 0..5 {
        let uri = Url::from_str(&format!("file:///doc{}.hodei", i)).unwrap();
        let stored_doc = server.get_document(uri).await.unwrap();

        let completions = completion_provider
            .provide_completions(
                &stored_doc,
                &CompletionContext {
                    position: CursorPosition { line: 0, column: 0 },
                    trigger_character: None,
                    trigger_kind: CompletionTriggerKind::Invoked,
                },
            )
            .await
            .unwrap();

        // Each document should provide completions
        assert!(!completions.is_empty());
    }
}

#[tokio::test]
async fn test_error_detection_workflow() {
    let server = HodeiDslServer::new();

    let uri = Url::from_str("file:///error_test.hodei").unwrap();

    // Document with intentional error (unknown fact type)
    let doc_with_error = Document {
        uri: uri.to_string(),
        content: r#"
rule test_error {
  when {
    emit UnknownFact {
      field: "value"
    };
  }
}
"#
        .to_string(),
        version: 1,
    };
    server.store_document(doc_with_error).await;

    // The document should be stored successfully
    let retrieved = server.get_document(uri).await.unwrap();
    assert!(retrieved.content.contains("UnknownFact"));
}

#[tokio::test]
async fn test_concurrent_document_operations() {
    let server = HodeiDslServer::new();
    let completion_provider = HodeiCompletionProvider::new();

    // Concurrent read/write operations
    let mut handles = Vec::new();

    // Writers
    for i in 0..10 {
        let server_clone = &server;
        let uri = Url::from_str(&format!("file:///write{}.hodei", i)).unwrap();
        let doc = Document {
            uri: uri.to_string(),
            content: format!("rule test{} {{ }}", i),
            version: 1,
        };
        let handle = tokio::spawn(async move {
            server_clone.store_document(doc).await;
        });
        handles.push(handle);
    }

    // Readers
    for i in 0..10 {
        let server_clone = &server;
        let uri = Url::from_str(&format!("file:///write{}.hodei", i)).unwrap();
        let handle = tokio::spawn(async move { server_clone.get_document(uri).await });
        handles.push(handle);
    }

    // Completion requests
    for i in 0..5 {
        let server_clone = &server;
        let doc_repo = server_clone.document_repository.clone();
        let handle = tokio::spawn(async move {
            let doc_repo_read = doc_repo.read().await;
            if let Some(doc) = doc_repo_read.get(&format!("file:///write{}.hodei", i)) {
                let provider = HodeiCompletionProvider::new();
                provider
                    .provide_completions(
                        doc,
                        &CompletionContext {
                            position: CursorPosition { line: 0, column: 0 },
                            trigger_character: None,
                            trigger_kind: CompletionTriggerKind::Invoked,
                        },
                    )
                    .await
            } else {
                Ok(Vec::new())
            }
        });
        handles.push(handle);
    }

    // Wait for all operations
    for handle in handles {
        handle.await.unwrap();
    }
}
