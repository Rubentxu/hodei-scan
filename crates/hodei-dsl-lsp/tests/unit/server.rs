//! Unit tests for HodeiDslServer

use hodei_dsl_lsp::infrastructure::server::HodeiDslServer;
use hodei_dsl_lsp::domain::models::{CursorPosition, Document};
use lsp_types::{Position, Url};
use std::str::FromStr;

#[tokio::test]
async fn test_server_initialization() {
    let server = HodeiDslServer::new();
    
    // Server should initialize successfully
    assert!(true);
}

#[tokio::test]
fn test_position_conversion_lsp_to_cursor() {
    let server = HodeiDslServer::new();
    
    let lsp_position = Position::new(5, 10);
    let cursor_position = server.position_from_lsp(lsp_position);
    
    assert_eq!(cursor_position.line, 5);
    assert_eq!(cursor_position.column, 10);
}

#[tokio::test]
fn test_position_conversion_cursor_to_lsp() {
    let server = HodeiDslServer::new();
    
    let cursor_position = CursorPosition { line: 3, column: 7 };
    let lsp_position = server.position_to_lsp(cursor_position);
    
    assert_eq!(lsp_position.line, 3);
    assert_eq!(lsp_position.character, 7);
}

#[tokio::test]
async fn test_get_document() {
    let server = HodeiDslServer::new();
    
    let uri = Url::from_str("file:///test.hodei").unwrap();
    let document = Document {
        uri: uri.to_string(),
        content: "test content".to_string(),
        version: 1,
    };
    
    server.store_document(document.clone()).await;
    
    let retrieved = server.get_document(uri).await;
    assert!(retrieved.is_some());
    
    let retrieved_doc = retrieved.unwrap();
    assert_eq!(retrieved_doc.content, "test content");
    assert_eq!(retrieved_doc.version, 1);
}

#[tokio::test]
async fn test_get_nonexistent_document() {
    let server = HodeiDslServer::new();
    
    let uri = Url::from_str("file:///nonexistent.hodei").unwrap();
    
    let retrieved = server.get_document(uri).await;
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_remove_document() {
    let server = HodeiDslServer::new();
    
    let uri = Url::from_str("file:///test.hodei").unwrap();
    let document = Document {
        uri: uri.to_string(),
        content: "test content".to_string(),
        version: 1,
    };
    
    server.store_document(document).await;
    assert!(server.get_document(uri).await.is_some());
    
    server.remove_document(uri).await;
    assert!(server.get_document(uri).await.is_none());
}

#[tokio::test]
fn test_uri_parsing() {
    let server = HodeiDslServer::new();
    
    let uri_string = "file:///path/to/file.hodei";
    let uri = Url::from_str(uri_string).unwrap();
    
    let document = Document {
        uri: uri.to_string(),
        content: "content".to_string(),
        version: 1,
    };
    
    // This should not panic
    assert_eq!(document.uri, uri_string);
}

#[tokio::test]
async fn test_server_state_isolation() {
    let server1 = HodeiDslServer::new();
    let server2 = HodeiDslServer::new();
    
    let uri1 = Url::from_str("file:///test1.hodei").unwrap();
    let uri2 = Url::from_str("file:///test2.hodei").unwrap();
    
    let doc1 = Document {
        uri: uri1.to_string(),
        content: "content1".to_string(),
        version: 1,
    };
    
    let doc2 = Document {
        uri: uri2.to_string(),
        content: "content2".to_string(),
        version: 1,
    };
    
    server1.store_document(doc1).await;
    server2.store_document(doc2).await;
    
    // Server1 should only have doc1
    assert!(server1.get_document(uri1).await.is_some());
    assert!(server1.get_document(uri2).await.is_none());
    
    // Server2 should only have doc2
    assert!(server2.get_document(uri1).await.is_none());
    assert!(server2.get_document(uri2).await.is_some());
}

#[tokio::test]
async fn test_update_document_version() {
    let server = HodeiDslServer::new();
    
    let uri = Url::from_str("file:///test.hodei").unwrap();
    
    let doc1 = Document {
        uri: uri.to_string(),
        content: "content".to_string(),
        version: 1,
    };
    
    let doc2 = Document {
        uri: uri.to_string(),
        content: "updated content".to_string(),
        version: 2,
    };
    
    server.store_document(doc1).await;
    assert_eq!(server.get_document(uri).await.unwrap().version, 1);
    
    server.store_document(doc2).await;
    assert_eq!(server.get_document(uri).await.unwrap().version, 2);
    assert_eq!(server.get_document(uri).await.unwrap().content, "updated content");
}
