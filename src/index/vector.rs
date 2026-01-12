//! 向量索引服务

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Result;
use surrealdb::{Surreal, engine::any::Any};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorMetadata {
    pub session_id: String,
    pub turn_id: String,
    pub turn_number: u64,
    pub timestamp: DateTime<Utc>,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub id: String,
    pub score: f32,
    pub turn_id: String,
    pub metadata: VectorMetadata,
}

#[async_trait]
pub trait VectorIndex: Send + Sync {
    async fn add(&self, id: &str, vector: &[f32], metadata: VectorMetadata) -> Result<()>;
    async fn search(
        &self,
        query: &[f32],
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>>;
    async fn delete(&self, id: &str) -> Result<bool>;
    async fn count(&self, session_id: &str) -> Result<u64>;
    async fn exists(&self, id: &str) -> Result<bool>;
}

pub struct MemoryVectorIndex {
    vectors: dashmap::DashMap<String, (Vec<f32>, VectorMetadata)>,
    dimension: usize,
}

impl MemoryVectorIndex {
    pub fn new(dimension: usize) -> Self {
        Self {
            vectors: dashmap::DashMap::new(),
            dimension,
        }
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len());

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

#[async_trait]
impl VectorIndex for MemoryVectorIndex {
    async fn add(&self, id: &str, vector: &[f32], metadata: VectorMetadata) -> Result<()> {
        assert_eq!(vector.len(), self.dimension);

        self.vectors
            .insert(id.to_string(), (vector.to_vec(), metadata));

        Ok(())
    }

    async fn search(
        &self,
        query: &[f32],
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>> {
        assert_eq!(query.len(), self.dimension);

        let mut results: Vec<_> = self
            .vectors
            .iter()
            .filter(|ref_multi| ref_multi.value().1.session_id == session_id)
            .map(|ref_multi| {
                let (id, (vector, meta)) = ref_multi.pair();
                let score = Self::cosine_similarity(query, vector);
                VectorSearchResult {
                    id: id.clone(),
                    score,
                    turn_id: meta.turn_id.clone(),
                    metadata: meta.clone(),
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);

        Ok(results)
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        Ok(self.vectors.remove(id).is_some())
    }

    async fn count(&self, session_id: &str) -> Result<u64> {
        let count = self
            .vectors
            .iter()
            .filter(|ref_multi| ref_multi.value().1.session_id == session_id)
            .count();
        Ok(count as u64)
    }

    async fn exists(&self, id: &str) -> Result<bool> {
        Ok(self.vectors.contains_key(id))
    }
}

pub fn create_vector_index(_db: Option<&Surreal<Any>>, _use_hnsw: bool) -> Box<dyn VectorIndex> {
    Box::new(MemoryVectorIndex::new(384))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_vector_index_add_and_search() {
        let index = MemoryVectorIndex::new(384);

        let metadata = VectorMetadata {
            session_id: "session_1".to_string(),
            turn_id: "turn_1".to_string(),
            turn_number: 1,
            timestamp: Utc::now(),
            extra: HashMap::new(),
        };

        let vector = vec![0.1; 384];
        index.add("vec_1", &vector, metadata).await.unwrap();

        let results = index.search(&vector, "session_1", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].turn_id, "turn_1");
    }

    #[tokio::test]
    async fn test_memory_vector_index_delete() {
        let index = MemoryVectorIndex::new(384);

        let metadata = VectorMetadata {
            session_id: "session_1".to_string(),
            turn_id: "turn_1".to_string(),
            turn_number: 1,
            timestamp: Utc::now(),
            extra: HashMap::new(),
        };

        let vector = vec![0.1; 384];
        index.add("vec_1", &vector, metadata).await.unwrap();

        let deleted = index.delete("vec_1").await.unwrap();
        assert!(deleted);

        let count = index.count("session_1").await.unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];

        assert_eq!(MemoryVectorIndex::cosine_similarity(&a, &b), 1.0);
        assert_eq!(MemoryVectorIndex::cosine_similarity(&a, &c), 0.0);
    }
}
