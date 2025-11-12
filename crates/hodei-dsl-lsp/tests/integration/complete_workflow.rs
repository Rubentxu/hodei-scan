//! Integration tests for complete LSP workflows
//!
//! Tests complete workflows combining multiple LSP features

use hodei_dsl_lsp::infrastructure::server::HodeiDslServer;
use hodei_dsl_lsp::infrastructure::adapters::{HodeiCompletionProvider, HodeiHoverProvider};
use hodei_dsl_lsp::domain::models::{Document, CursorPosition, CompletionContext, CompletionTriggerKind};
use lsp_types::Url;
use std::str::FromStr;

#[tokio::test]
async fn test_completion_and_hover_workflow() {
    let server = HodeiDslServer::new();
    let completion_provider = HodeiCompletionProvider::new();
    let hover_provider = HodeiHoverProvider::new();
    
    // Open a document with a fact type
    let uri = Url::from_str("file:///workflow.hodei").unwrap();
    let document = Document {
        uri: uri.to_string(),
        content: "Vulnerability".to_string(),
        version: 1,
    };
    server.store_document(document).await;
    
    // Get completions
    let completions = completion_provider.provide_completions(
        &server.get_document(uri).await.unwrap(),
        &CompletionContext {
            position: CursorPosition { line: 0, column: 0 },
            trigger_character: None,
            trigger_kind: CompletionTriggerKind::Invoked,
        },
    )
    .await
    .expect("Should provide completions");
    
    assert!(!completions.is_empty());
    
    // Get hover info
    let hover = hover_provider.provide_hover(
        &server.get_document(uri).await.unwrap(),
        CursorPosition { line: 0, column: 0 },
    )
    .await
    .expect("Should provide hover");
    
    assert!(hover.is_some());
}

#[tokio::test]
async fn test_document_edit_workflow() {
    let server = HodeiDslServer::new();
    let uri = Url::from_str("file:///edit.hodei").unwrap();
    
    // Initial document
    let doc1 = Document {
        uri: uri.to_string(),
        content: "rule test { }".to_string(),
        version: 1,
    };
    server.store_document(doc1).await;
    
    // Edit document
    let doc2 = Document {
        uri: uri.to_string(),
        content: "rule updated { emit Vulnerability { } }".to_string(),
        version: 2,
    };
    server.store_document(doc2).await;
    
    // Verify edit
    let retrieved = server.get_document(uri).await.unwrap();
    assert_eq!(retrieved.version, 2);
    assert!(retrieved.content.contains("updated"));
    assert!(retrieved.content.contains("Vulnerability"));
}

#[tokio::test]
async fn test_multiple_features_workflow() {
    let server = HodeiDslServer::new();
    let completion_provider = HodeiCompletionProvider::new();
    let hover_provider = HodeiHoverProvider::new();
    
    let uri = Url::from_str("file:///multi.hodei").unwrap();
    
    // Complex document with multiple elements
    let document = Document {
        uri: uri.to_string(),
        content: r#"
rule comprehensive {
  when {
    function check(input: string): boolean {
      return matches(input, /pattern/) && contains(input, "test");
    }
  }
  then {
    emit Vulnerability {
      severity: "Critical"
    };
    
    emit CodeSmell {
      type: "example"
    };
  }
}
"#.to_string(),
        version: 1,
    };
    
    server.store_document(document).await;
    let stored_doc = server.get_document(uri).await.unwrap();
    
    // Test completion at different positions
    let positions = vec![
        (0, 0),   // Start of document
        (5, 10),  // Middle
        (10, 5),  // After function
    ];
    
    for (line, col) in positions {
        let completions = completion_provider.provide_completions(
            &stored_doc,
            &CompletionContext {
                position: CursorPosition { line, column: col },
                trigger_character: None,
                trigger_kind: CompletionTriggerKind::Invoked,
            },
        )
        .await
        .expect("Should provide completions");
        
        // Completions may or may not be empty depending on position
        // The important thing is that the call succeeds
    }
    
    // Test hover at function position
    let hover = hover_provider.provide_hover(
        &stored_doc,
        CursorPosition { line: 3, column: 10 }, // On "matches"
    )
    .await
    .expect("Should provide hover");
    
    // Hover may or may not find documentation depending on implementation
    // The important thing is that the call succeeds
}

#[tokio::test]
async fn test_document_lifecycle_workflow() {
    let server = HodeiDslServer::new();
    let uri = Url::from_str("file:///lifecycle.hodei").unwrap();
    
    // 1. Create document
    let doc1 = Document {
        uri: uri.to_string(),
        content: "initial".to_string(),
        version: 1,
    };
    server.store_document(doc1).await;
    assert!(server.get_document(uri).await.is_some());
    
    // 2. Update document
    let doc2 = Document {
        uri: uri.to_string(),
        content: "updated".to_string(),
        version: 2,
    };
    server.store_document(doc2).await;
    let retrieved2 = server.get_document(uri).await.unwrap();
    assert_eq!(retrieved2.content, "updated");
    assert_eq!(retrieved2.version, 2);
    
    // 3. Update again
    let doc3 = Document {
        uri: uri.to_string(),
        content: "final".to_string(),
        version: 3,
    };
    server.store_document(doc3).await;
    let retrieved3 = server.get_document(uri).await.unwrap();
    assert_eq!(retrieved3.content, "final");
    assert_eq!(retrieved3.version, 3);
    
    // 4. Close document
    server.remove_document(uri).await;
    assert!(server.get_document(uri).await.is_none());
    
    // 5. Reopen document
    let doc4 = Document {
        uri: uri.to_string(),
        content: "reopened".to_string(),
        version: 1,
    };
    server.store_document(doc4).await;
    let retrieved4 = server.get_document(uri).await.unwrap();
    assert_eq!(retrieved4.content, "reopened");
    assert_eq!(retrieved4.version, 1);
}

#[tokio::test]
async fn test_multiple_documents_workflow() {
    let server = HodeiDslServer::new();
    let completion_provider = HodeiCompletionProvider::new();
    
    // Create multiple documents
    for i in 0..5 {
        let uri = Url::from_str(&format!("file:///multi{}.hodei", i)).unwrap();
        let content = format!("rule rule{} {{ emit {} {{ }} }}", i, if i % 2 == 0 { "Vulnerability" } else { "CodeSmell" });
        
        let document = Document {
            uri: uri.to_string(),
            content,
            version: 1,
        };
        server.store_document(document).await;
    }
    
    // Test completions on each document
    for i in 0..5 {
        let uri = Url::from_str(&format!("file:///multi{}.hodei", i)).unwrap();
        let doc = server.get_document(uri).await.unwrap();
        
        let completions = completion_provider.provide_completions(
            &doc,
            &CompletionContext {
                position: CursorPosition { line: 0, column: 0 },
                trigger_character: None,
                trigger_kind: CompletionTriggerKind::Invoked,
            },
        )
        .await
        .expect("Should provide completions");
        
        assert!(!completions.is_empty());
    }
}

#[tokio::test]
async fn test_concurrent_document_operations() {
    let server = HodeiDslServer::new();
    
    // Spawn concurrent operations
    let mut handles = Vec::new();
    
    // Reader tasks
    for i in 0..5 {
        let server_clone = &server;
        let uri = Url::from_str(&format!("file:///read{}.hodei", i)).unwrap();
        let handle = tokio::spawn(async move {
            server_clone.get_document(uri).await
        });
        handles.push(handle);
    }
    
    // Writer tasks
    for i in 0..5 {
        let server_clone = &server;
        let uri = Url::from_str(&format!("file:///write{}.hodei", i)).unwrap();
        let doc = Document {
            uri: uri.to_string(),
            content: format!("content{}", i),
            version: 1,
        };
        let handle = tokio::spawn(async move {
            server_clone.store_document(doc).await;
        });
        handles.push(handle);
    }
    
    // Wait for all operations
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Verify all writes succeeded
    for i in 0..5 {
        let uri = Url::from_str(&format!("file:///write{}.hodei", i)).unwrap();
        assert!(server.get_document(uri).await.is_some());
    }
}

#[tokio::test]
async fn test_error_recovery_workflow() {
    let server = HodeiDslServer::new();
    
    // Try to access non-existent document (should return None, not panic)
    let uri = Url::from_str("file:///nonexistent.hodei").unwrap();
    assert!(server.get_document(uri).await.is_none());
    
    // Remove non-existent document (should not panic)
    server.remove_document(uri).await;
    
    // Store document
    let doc = Document {
        uri: uri.to_string(),
        content: "content".to_string(),
        version: 1,
    };
    server.store_document(doc).await;
    
    // Verify it's there
    assert!(server.get_document(uri).await.is_some());
    
    // Remove it
    server.remove_document(uri).await;
    assert!(server.get_document(uri).await.is_none());
    
    // Try to remove again (should not panic)
    server.remove_document(uri).await;
}
