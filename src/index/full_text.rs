//! 全文索引服务

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::Result;
use surrealdb::{Surreal, engine::any::Any};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FtsMetadata {
    pub session_id: String,
    pub turn_id: String,
    pub turn_number: u64,
    pub timestamp: DateTime<Utc>,
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtsResult {
    pub id: String,
    pub score: f32,
    pub turn_id: String,
    pub gist: String,
    pub metadata: FtsMetadata,
}

#[async_trait]
pub trait FullTextIndex: Send + Sync {
    async fn add(&self, id: &str, content: &str, metadata: FtsMetadata) -> Result<()>;
    async fn search(&self, query: &str, session_id: &str, limit: usize) -> Result<Vec<FtsResult>>;
    async fn delete(&self, id: &str) -> Result<bool>;
    async fn count(&self, session_id: &str) -> Result<u64>;
}

pub struct MemoryFtsIndex {
    documents: dashmap::DashMap<String, (String, FtsMetadata)>,
}

impl MemoryFtsIndex {
    pub fn new() -> Self {
        Self {
            documents: dashmap::DashMap::new(),
        }
    }

    fn matches_query(content: &str, query: &str) -> bool {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let content_lower = content.to_lowercase();

        query_words
            .iter()
            .all(|word| content_lower.contains(&word.to_lowercase()))
    }

    fn calculate_score(content: &str, query: &str) -> f32 {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let content_lower = content.to_lowercase();

        let mut score = 0.0;
        for word in &query_words {
            let word_lower = word.to_lowercase();
            let count = content_lower.matches(&word_lower).count() as f32;
            score += count / (word.len() as f32 + 1.0);
        }

        score
    }
}

#[async_trait]
impl FullTextIndex for MemoryFtsIndex {
    async fn add(&self, id: &str, content: &str, metadata: FtsMetadata) -> Result<()> {
        self.documents
            .insert(id.to_string(), (content.to_string(), metadata));

        Ok(())
    }

    async fn search(&self, query: &str, session_id: &str, limit: usize) -> Result<Vec<FtsResult>> {
        let mut results: Vec<_> = self
            .documents
            .iter()
            .filter(|ref_multi| ref_multi.value().1.session_id == session_id)
            .filter(|ref_multi| {
                let (content, _) = ref_multi.value();
                Self::matches_query(content, query)
            })
            .map(|ref_multi| {
                let (id, (content, meta)) = ref_multi.pair();
                let score = Self::calculate_score(content, query);
                FtsResult {
                    id: id.clone(),
                    score,
                    turn_id: meta.turn_id.clone(),
                    gist: content.to_string(),
                    metadata: meta.clone(),
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results.truncate(limit);

        Ok(results)
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        Ok(self.documents.remove(id).is_some())
    }

    async fn count(&self, session_id: &str) -> Result<u64> {
        let count = self
            .documents
            .iter()
            .filter(|ref_multi| ref_multi.value().1.session_id == session_id)
            .count();
        Ok(count as u64)
    }
}

pub fn create_full_text_index(
    _db: Option<&Surreal<Any>>,
    _use_fts: bool,
) -> Box<dyn FullTextIndex> {
    Box::new(MemoryFtsIndex::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_fts_index_add_and_search() {
        let index = MemoryFtsIndex::new();

        let metadata = FtsMetadata {
            session_id: "session_1".to_string(),
            turn_id: "turn_1".to_string(),
            turn_number: 1,
            timestamp: Utc::now(),
            extra: HashMap::new(),
        };

        index
            .add("doc_1", "hello world rust", metadata)
            .await
            .unwrap();

        let results = index.search("hello", "session_1", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].turn_id, "turn_1");
    }

    #[tokio::test]
    async fn test_memory_fts_index_multi_word_search() {
        let index = MemoryFtsIndex::new();

        let metadata = FtsMetadata {
            session_id: "session_1".to_string(),
            turn_id: "turn_1".to_string(),
            turn_number: 1,
            timestamp: Utc::now(),
            extra: HashMap::new(),
        };

        index
            .add("doc_1", "hello world rust programming", metadata)
            .await
            .unwrap();

        let results = index
            .search("rust programming", "session_1", 10)
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_memory_fts_index_delete() {
        let index = MemoryFtsIndex::new();

        let metadata = FtsMetadata {
            session_id: "session_1".to_string(),
            turn_id: "turn_1".to_string(),
            turn_number: 1,
            timestamp: Utc::now(),
            extra: HashMap::new(),
        };

        index.add("doc_1", "hello world", metadata).await.unwrap();

        let deleted = index.delete("doc_1").await.unwrap();
        assert!(deleted);

        let count = index.count("session_1").await.unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_matches_query() {
        assert!(MemoryFtsIndex::matches_query("hello world rust", "hello"));
        assert!(MemoryFtsIndex::matches_query("hello world rust", "world"));
        assert!(MemoryFtsIndex::matches_query("hello world rust", "rust"));
        assert!(!MemoryFtsIndex::matches_query("hello world rust", "python"));
    }

    #[test]
    fn test_calculate_score() {
        let content = "hello world rust programming";
        let score1 = MemoryFtsIndex::calculate_score(content, "hello");
        let score2 = MemoryFtsIndex::calculate_score(content, "rust programming");

        assert!(score2 > score1);
    }
}
