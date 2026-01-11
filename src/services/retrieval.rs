//! 检索服务

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::index::{IndexService, SearchOptions, SearchResult};
use crate::models::turn::Turn;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressiveIndex {
    pub turn_id: String,
    pub session_id: String,
    pub gist: String,
    pub timestamp: DateTime<Utc>,
    pub turn_number: u64,
    pub message_type: String,
    pub model: Option<String>,
}

#[async_trait]
pub trait RetrievalService: Send + Sync {
    async fn list_recent(&self, session_id: &str, limit: u32) -> Result<Vec<ProgressiveIndex>>;
    async fn semantic_search(
        &self,
        session_id: &str,
        query: &str,
        limit: u32,
    ) -> Result<Vec<SearchResult>>;
    async fn hybrid_search(
        &self,
        session_id: &str,
        query: &str,
        limit: u32,
    ) -> Result<Vec<SearchResult>>;
    async fn fetch_content(&self, session_id: &str, turn_id: &str) -> Result<Option<Turn>>;
}

pub struct RetrievalServiceImpl {
    index_service: Box<dyn IndexService>,
}

impl RetrievalServiceImpl {
    pub fn new(index_service: Box<dyn IndexService>) -> Self {
        Self { index_service }
    }
}

#[async_trait]
impl RetrievalService for RetrievalServiceImpl {
    async fn list_recent(&self, session_id: &str, limit: u32) -> Result<Vec<ProgressiveIndex>> {
        let indices = self
            .index_service
            .list_indices(session_id, limit as usize, 0)
            .await?;

        let mut progressive_indices = Vec::with_capacity(indices.len());

        for record in indices {
            progressive_indices.push(ProgressiveIndex {
                turn_id: record.turn_id,
                session_id: record.session_id,
                gist: record.gist,
                timestamp: record.timestamp,
                turn_number: record.turn_number,
                message_type: "Unknown".to_string(),
                model: None,
            });
        }

        Ok(progressive_indices)
    }

    async fn semantic_search(
        &self,
        session_id: &str,
        query: &str,
        limit: u32,
    ) -> Result<Vec<SearchResult>> {
        self.index_service
            .search_indices(
                session_id,
                query,
                SearchOptions {
                    limit: limit as usize,
                    offset: 0,
                    use_semantic: true,
                    use_full_text: false,
                    use_hybrid: false,
                    threshold: None,
                },
            )
            .await
    }

    async fn hybrid_search(
        &self,
        session_id: &str,
        query: &str,
        limit: u32,
    ) -> Result<Vec<SearchResult>> {
        self.index_service
            .search_indices(
                session_id,
                query,
                SearchOptions {
                    limit: limit as usize,
                    offset: 0,
                    use_semantic: true,
                    use_full_text: true,
                    use_hybrid: true,
                    threshold: None,
                },
            )
            .await
    }

    async fn fetch_content(&self, _session_id: &str, _turn_id: &str) -> Result<Option<Turn>> {
        Ok(None)
    }
}

pub fn create_retrieval_service(
    embedding_model: Box<dyn crate::index::EmbeddingModel>,
) -> Box<dyn RetrievalService> {
    use crate::index::{create_full_text_index, create_unified_index_service, create_vector_index};

    let vector_index = create_vector_index(None, false);
    let full_text_index = create_full_text_index(None, false);
    let index_service =
        create_unified_index_service(vector_index, full_text_index, embedding_model);

    Box::new(RetrievalServiceImpl::new(index_service))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::embedding::SimpleEmbeddingModel;
    use crate::index::full_text::MemoryFtsIndex;
    use crate::index::vector::MemoryVectorIndex;

    #[tokio::test]
    async fn test_retrieval_service_list_recent() {
        let embedding_model: Box<dyn crate::index::EmbeddingModel> =
            Box::new(SimpleEmbeddingModel::new(384));
        let vector_index: Box<dyn crate::index::VectorIndex> =
            Box::new(MemoryVectorIndex::new(384));
        let full_text_index: Box<dyn crate::index::FullTextIndex> = Box::new(MemoryFtsIndex::new());
        let index_service = crate::index::create_unified_index_service(
            vector_index,
            full_text_index,
            embedding_model,
        );

        let service = RetrievalServiceImpl::new(index_service);

        let results = service.list_recent("session_1", 10).await.unwrap();
        assert!(results.is_empty());
    }
}
