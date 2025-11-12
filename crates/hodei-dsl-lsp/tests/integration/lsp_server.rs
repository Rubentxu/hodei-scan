//! Integration tests for LSP server
//!
//! Tests the full LSP server with real components

use hodei_dsl_lsp::infrastructure::server::HodeiDslServer;
use hodei_dsl_lsp::domain::models::Document;
use lsp_types::{Url, Position, TextDocumentItem, DidOpenTextDocumentParams};
use std::str::FromStr;

#[tokio::test]
async fn test_server_full_lifecycle() {
    let server = HodeiDslServer::new();
    
    // Simulate document open
    let uri = Url::from_str("file:///test.hodei").unwrap();
    let document = Document {
        uri: uri.to_string(),
        content: r#"
rule test_rule {
  when {
    emit Vulnerability {
      severity: "Critical",
      message: "Test"
    };
  }
}
"#.to_string(),
        version: 1,
    };
    
    server.store_document(document.clone()).await;
    
    // Verify document is stored
    assert!(server.get_document(uri).await.is_some());
    
    // Remove document
    server.remove_document(uri).await;
    
    // Verify document is removed
    assert!(server.get_document(uri).await.is_none());
}

#[tokio::test]
async fn test_server_with_multiple_documents() {
    let server = HodeiDslServer::new();
    
    // Store multiple documents
    for i in 0..5 {
        let uri = Url::from_str(&format!("file:///test{}.hodei", i)).unwrap();
        let document = Document {
            uri: uri.to_string(),
            content: format!("content{}", i),
            version: 1,
        };
        server.store_document(document).await;
    }
    
    // Verify all documents are stored
    for i in 0..5 {
        let uri = Url::from_str(&format!("file:///test{}.hodei", i)).unwrap();
        assert!(server.get_document(uri).await.is_some());
    }
}

#[tokio::test]
async fn test_server_position_conversion() {
    let server = HodeiDslServer::new();
    
    // Test various positions
    let positions = vec![
        (0, 0),
        (10, 5),
        (100, 50),
    ];
    
    for (line, col) in positions {
        let lsp_pos = Position::new(line, col);
        let cursor_pos = server.position_from_lsp(lsp_pos);
        assert_eq!(cursor_pos.line, line);
        assert_eq!(cursor_pos.column, col);
        
        let back_to_lsp = server.position_to_lsp(cursor_pos);
        assert_eq!(back_to_lsp.line, line);
        assert_eq!(back_to_lsp.character, col);
    }
}

#[tokio::test]
async fn test_server_with_complex_document() {
    let server = HodeiDslServer::new();
    
    let complex_content = r#"
// Complex rule with multiple patterns
rule comprehensive_check {
  when {
    function validateInput(input: string): boolean {
      if (matches(input, /^[a-z]+$/)) {
        return contains(input, "test");
      }
      return length_gt(input, 10);
    }
  }
  then {
    emit Vulnerability {
      severity: "Critical",
      message: "Complex validation logic"
    };
    
    emit CodeSmell {
      type: "complex_logic",
      severity: "Minor"
    };
  }
}
"#;
    
    let uri = Url::from_str("file:///complex.hodei").unwrap();
    let document = Document {
        uri: uri.to_string(),
        content: complex_content.to_string(),
        version: 1,
    };
    
    server.store_document(document).await;
    
    let retrieved = server.get_document(uri).await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().content, complex_content);
}

#[tokio::test]
async fn test_server_document_versioning() {
    let server = HodeiDslServer::new();
    
    let uri = Url::from_str("file:///versioned.hodei").unwrap();
    
    // Version 1
    let doc1 = Document {
        uri: uri.to_string(),
        content: "version 1".to_string(),
        version: 1,
    };
    server.store_document(doc1).await;
    
    // Version 2
    let doc2 = Document {
        uri: uri.to_string(),
        content: "version 2".to_string(),
        version: 2,
    };
    server.store_document(doc2).await;
    
    // Version 3
    let doc3 = Document {
        uri: uri.to_string(),
        content: "version 3".to_string(),
        version: 3,
    };
    server.store_document(doc3).await;
    
    // Should have version 3
    let retrieved = server.get_document(uri).await.unwrap();
    assert_eq!(retrieved.version, 3);
    assert_eq!(retrieved.content, "version 3");
}

#[tokio::test]
async fn test_server_edge_cases() {
    let server = HodeiDslServer::new();
    
    // Empty content
    let uri1 = Url::from_str("file:///empty.hodei").unwrap();
    let doc1 = Document {
        uri: uri1.to_string(),
        content: "".to_string(),
        version: 1,
    };
    server.store_document(doc1).await;
    assert!(server.get_document(uri1).await.is_some());
    
    // Very long content
    let uri2 = Url::from_str("file:///long.hodei").unwrap();
    let long_content = "a".repeat(1000000);
    let doc2 = Document {
        uri: uri2.to_string(),
        content: long_content.clone(),
        version: 1,
    };
    server.store_document(doc2).await;
    assert_eq!(server.get_document(uri2).await.unwrap().content, long_content);
    
    // Unicode content
    let uri3 = Url::from_str("file:///unicode.hodei").unwrap();
    let unicode_content = "Hello 世界 🌍 Ñoño";
    let doc3 = Document {
        uri: uri3.to_string(),
        content: unicode_content.to_string(),
        version: 1,
    };
    server.store_document(doc3).await;
    assert_eq!(server.get_document(uri3).await.unwrap().content, unicode_content);
}

#[tokio::test]
async fn test_server_concurrent_operations() {
    let server = HodeiDslServer::new();
    
    // Concurrent reads and writes
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let server_clone = &server;
        let handle = tokio::spawn(async move {
            let uri = Url::from_str(&format!("file:///concurrent{}.hodei", i)).unwrap();
            let doc = Document {
                uri: uri.to_string(),
                content: format!("content{}", i),
                version: 1,
            };
            server_clone.store_document(doc).await;
            server_clone.get_document(uri).await
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_some());
    }
    
    // Verify all documents are stored
    for i in 0..10 {
        let uri = Url::from_str(&format!("file:///concurrent{}.hodei", i)).unwrap();
        assert!(server.get_document(uri).await.is_some());
    }
}

#[tokio::test]
async fn test_server_remove_operations() {
    let server = HodeiDslServer::new();
    
    // Store and remove multiple documents
    let uris: Vec<Url> = (0..10)
        .map(|i| Url::from_str(&format!("file:///remove{}.hodei", i)).unwrap())
        .collect();
    
    // Store all
    for (i, uri) in uris.iter().enumerate() {
        let doc = Document {
            uri: uri.to_string(),
            content: format!("content{}", i),
            version: 1,
        };
        server.store_document(doc).await;
    }
    
    // Verify all are stored
    for uri in &uris {
        assert!(server.get_document(*uri).await.is_some());
    }
    
    // Remove even-indexed documents
    for (i, uri) in uris.iter().enumerate() {
        if i % 2 == 0 {
            server.remove_document(*uri).await;
        }
    }
    
    // Verify removals
    for (i, uri) in uris.iter().enumerate() {
        if i % 2 == 0 {
            assert!(server.get_document(*uri).await.is_none());
        } else {
            assert!(server.get_document(*uri).await.is_some());
        }
    }
}
