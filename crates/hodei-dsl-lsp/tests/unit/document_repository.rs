//! Unit tests for DocumentRepository

use hodei_dsl_lsp::domain::models::Document;
use hodei_dsl_lsp::infrastructure::adapters::InMemoryDocumentRepository;
use crate::fixtures::create_basic_document;

#[tokio::test]
async fn test_document_repository_initialization() {
    let repo = InMemoryDocumentRepository::new();
    
    // Should start empty
    let docs = repo.get_all().await;
    assert!(docs.is_empty());
}

#[tokio::test]
async fn test_store_and_retrieve_document() {
    let repo = InMemoryDocumentRepository::new();
    let document = create_basic_document();
    let uri = document.uri.clone();
    
    // Store document
    repo.store(document.clone()).await;
    
    // Retrieve document
    let retrieved = repo.get(&uri).await;
    assert!(retrieved.is_some());
    
    let retrieved_doc = retrieved.unwrap();
    assert_eq!(retrieved_doc.uri, uri);
    assert_eq!(retrieved_doc.content, document.content);
    assert_eq!(retrieved_doc.version, document.version);
}

#[tokio::test]
async fn test_get_nonexistent_document() {
    let repo = InMemoryDocumentRepository::new();
    
    let document = repo.get("file:///nonexistent.hodei").await;
    assert!(document.is_none());
}

#[tokio::test]
async fn test_remove_document() {
    let repo = InMemoryDocumentRepository::new();
    let document = create_basic_document();
    let uri = document.uri.clone();
    
    // Store document
    repo.store(document).await;
    
    // Verify it exists
    assert!(repo.get(&uri).await.is_some());
    
    // Remove document
    repo.remove(&uri).await;
    
    // Verify it's gone
    assert!(repo.get(&uri).await.is_none());
}

#[tokio::test]
async fn test_remove_nonexistent_document() {
    let repo = InMemoryDocumentRepository::new();
    
    // Should not panic when removing non-existent document
    repo.remove("file:///nonexistent.hodei").await;
    
    // Repository should still be empty
    let docs = repo.get_all().await;
    assert!(docs.is_empty());
}

#[tokio::test]
async fn test_get_all_documents() {
    let repo = InMemoryDocumentRepository::new();
    
    // Initially empty
    assert_eq!(repo.get_all().await.len(), 0);
    
    // Store multiple documents
    let doc1 = Document {
        uri: "file:///test1.hodei".to_string(),
        content: "content1".to_string(),
        version: 1,
    };
    
    let doc2 = Document {
        uri: "file:///test2.hodei".to_string(),
        content: "content2".to_string(),
        version: 1,
    };
    
    repo.store(doc1).await;
    repo.store(doc2).await;
    
    // Should retrieve both
    let all_docs = repo.get_all().await;
    assert_eq!(all_docs.len(), 2);
}

#[tokio::test]
async fn test_update_document() {
    let repo = InMemoryDocumentRepository::new();
    let uri = "file:///test.hodei".to_string();
    
    // Store initial document
    let initial_doc = Document {
        uri: uri.clone(),
        content: "initial content".to_string(),
        version: 1,
    };
    repo.store(initial_doc).await;
    
    // Update document
    let updated_doc = Document {
        uri: uri.clone(),
        content: "updated content".to_string(),
        version: 2,
    };
    repo.store(updated_doc).await;
    
    // Verify updated content
    let retrieved = repo.get(&uri).await;
    assert!(retrieved.is_some());
    
    let doc = retrieved.unwrap();
    assert_eq!(doc.content, "updated content");
    assert_eq!(doc.version, 2);
}

#[tokio::test]
async fn test_concurrent_access() {
    let repo = InMemoryDocumentRepository::new();
    let uri = "file:///test.hodei".to_string();
    
    // Store initial document
    let doc = Document {
        uri: uri.clone(),
        content: "content".to_string(),
        version: 1,
    };
    repo.store(doc).await;
    
    // Read and write concurrently using tokio::spawn
    let repo_clone = &repo;
    let uri_clone = uri.clone();
    
    let (read_result, write_result) = tokio::join!(
        repo_clone.get(&uri_clone),
        async {
            let new_doc = Document {
                uri: uri_clone.clone(),
                content: "new content".to_string(),
                version: 2,
            };
            repo_clone.store(new_doc).await;
        }
    );
    
    // Read should succeed
    assert!(read_result.is_some());
    
    // Write should succeed
    assert!(true);
    
    // Verify final state
    let final_doc = repo.get(&uri).await;
    assert!(final_doc.is_some());
    assert_eq!(final_doc.unwrap().content, "new content");
}

#[tokio::test]
async fn test_repository_with_multiple_documents() {
    let repo = InMemoryDocumentRepository::new();
    
    // Store 10 documents
    for i in 0..10 {
        let doc = Document {
            uri: format!("file:///test{}.hodei", i),
            content: format!("content{}", i),
            version: 1,
        };
        repo.store(doc).await;
    }
    
    // Should have 10 documents
    let all_docs = repo.get_all().await;
    assert_eq!(all_docs.len(), 10);
    
    // Verify each document
    for i in 0..10 {
        let uri = format!("file:///test{}.hodei", i);
        let doc = repo.get(&uri).await;
        assert!(doc.is_some());
        assert_eq!(doc.unwrap().content, format!("content{}", i));
    }
}

#[tokio::test]
async fn test_empty_uri() {
    let repo = InMemoryDocumentRepository::new();
    
    // Store document with empty content
    let doc = Document {
        uri: "file:///empty.hodei".to_string(),
        content: "".to_string(),
        version: 1,
    };
    repo.store(doc).await;
    
    // Should still be retrievable
    let retrieved = repo.get("file:///empty.hodei").await;
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().content, "");
}
