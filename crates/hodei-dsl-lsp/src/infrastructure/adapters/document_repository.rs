//! Document repository adapter
//!
//! In-memory implementation of DocumentRepository

use crate::domain::models::Document;
use crate::domain::ports::DocumentRepository;
use tokio::sync::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// In-memory document repository
pub struct InMemoryDocumentRepository {
    documents: Arc<RwLock<HashMap<String, Document>>>,
}

impl InMemoryDocumentRepository {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl DocumentRepository for InMemoryDocumentRepository {
    async fn get(&self, uri: &str) -> Option<Document> {
        let documents = self.documents.read().await;
        documents.get(uri).cloned()
    }
    
    async fn store(&self, document: Document) {
        let mut documents = self.documents.write().await;
        documents.insert(document.uri.clone(), document);
    }
    
    async fn remove(&self, uri: &str) {
        let mut documents = self.documents.write().await;
        documents.remove(uri);
    }
    
    async fn get_all(&self) -> Vec<Document> {
        let documents = self.documents.read().await;
        documents.values().cloned().collect()
    }
}
