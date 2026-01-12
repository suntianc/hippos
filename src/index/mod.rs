//! 索引模块

pub mod embedding;
pub mod full_text;
pub mod vector;

pub use embedding::{EmbeddingModel, create_embedding_model};
pub use full_text::{FtsMetadata, FtsResult, FullTextIndex, create_full_text_index};
pub use vector::{VectorIndex, VectorMetadata, VectorSearchResult, create_vector_index};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::models::index_record::IndexRecord;
use crate::models::turn::Turn;

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub limit: usize,
    pub offset: usize,
    pub use_semantic: bool,
    pub use_full_text: bool,
    pub use_hybrid: bool,
    pub threshold: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchResultType {
    Semantic,
    FullText,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub turn_id: String,
    pub gist: String,
    pub score: f32,
    pub result_type: SearchResultType,
    pub turn_number: u64,
    pub timestamp: DateTime<Utc>,
    pub sources: Vec<String>,
}

#[async_trait]
pub trait IndexService: Send + Sync {
    async fn index_turn(&self, turn: &Turn) -> Result<IndexRecord>;
    async fn list_indices(
        &self,
        session_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<IndexRecord>>;
    async fn search_indices(
        &self,
        session_id: &str,
        query: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>>;
    async fn delete_index(&self, turn_id: &str) -> Result<bool>;
}

pub struct UnifiedIndexService {
    vector_index: Box<dyn VectorIndex>,
    full_text_index: Box<dyn FullTextIndex>,
    embedding_model: Box<dyn EmbeddingModel>,
}

impl UnifiedIndexService {
    pub fn new(
        vector_index: Box<dyn VectorIndex>,
        full_text_index: Box<dyn FullTextIndex>,
        embedding_model: Box<dyn EmbeddingModel>,
    ) -> Self {
        Self {
            vector_index,
            full_text_index,
            embedding_model,
        }
    }

    fn rrf_fusion(
        vector_results: &[VectorSearchResult],
        fts_results: &[FtsResult],
        k: u64,
    ) -> Vec<SearchResult> {
        let mut scores: std::collections::HashMap<String, (f32, Vec<String>)> =
            std::collections::HashMap::new();

        for (rank, result) in vector_results.iter().enumerate() {
            let rrf_score = 1.0 / (k + rank as u64) as f32;
            let entry = scores
                .entry(result.turn_id.clone())
                .or_insert((0.0, Vec::new()));
            entry.0 += rrf_score;
            if !entry.1.contains(&"vector".to_string()) {
                entry.1.push("vector".to_string());
            }
        }

        for (rank, result) in fts_results.iter().enumerate() {
            let rrf_score = 1.0 / (k + rank as u64) as f32;
            let entry = scores
                .entry(result.turn_id.clone())
                .or_insert((0.0, Vec::new()));
            entry.0 += rrf_score;
            if !entry.1.contains(&"full_text".to_string()) {
                entry.1.push("full_text".to_string());
            }
        }

        let mut results: Vec<_> = scores
            .into_iter()
            .map(|(turn_id, (score, sources))| {
                let gist = fts_results
                    .iter()
                    .find(|r| r.turn_id == turn_id)
                    .map(|r| r.gist.clone())
                    .unwrap_or_default();
                let timestamp = vector_results
                    .iter()
                    .find(|r| r.turn_id == turn_id)
                    .map(|r| r.metadata.timestamp)
                    .or_else(|| {
                        fts_results
                            .iter()
                            .find(|r| r.turn_id == turn_id)
                            .map(|r| r.metadata.timestamp)
                    })
                    .unwrap_or_else(Utc::now);
                let turn_number = vector_results
                    .iter()
                    .find(|r| r.turn_id == turn_id)
                    .map(|r| r.metadata.turn_number)
                    .or_else(|| {
                        fts_results
                            .iter()
                            .find(|r| r.turn_id == turn_id)
                            .map(|r| r.metadata.turn_number)
                    })
                    .unwrap_or(0);

                SearchResult {
                    turn_id,
                    gist,
                    score,
                    result_type: SearchResultType::Hybrid,
                    turn_number,
                    timestamp,
                    sources,
                }
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        results
    }
}

#[async_trait]
impl IndexService for UnifiedIndexService {
    async fn index_turn(&self, turn: &Turn) -> Result<IndexRecord> {
        let turn_id = &turn.id;
        let vector_id = format!("vec_{}", turn_id);

        let vector_exists = self.vector_index.exists(&vector_id).await?;
        let fts_exists = self
            .full_text_index
            .exists(&format!("doc_{}", turn_id))
            .await?;

        if vector_exists || fts_exists {
            return Err(crate::error::AppError::Validation(format!(
                "Turn {} is already indexed",
                turn_id
            )));
        }

        let gist = turn
            .dehydrated
            .as_ref()
            .map(|d| d.gist.clone())
            .unwrap_or_else(|| turn.raw_content.chars().take(100).collect());

        let embedding = if let Some(dehydrated) = &turn.dehydrated {
            if let Some(emb) = &dehydrated.embedding {
                emb.clone()
            } else {
                self.embedding_model.encode(&gist).await?
            }
        } else {
            self.embedding_model.encode(&gist).await?
        };

        let record = IndexRecord::new(
            &turn.id,
            &turn.session_id,
            &gist,
            turn.metadata.timestamp,
            turn.turn_number,
        );

        let vector_metadata = VectorMetadata {
            session_id: turn.session_id.clone(),
            turn_id: turn.id.clone(),
            turn_number: turn.turn_number,
            timestamp: turn.metadata.timestamp,
            extra: std::collections::HashMap::new(),
        };

        self.vector_index
            .add(&format!("vec_{}", turn.id), &embedding, vector_metadata)
            .await?;

        let fts_metadata = FtsMetadata {
            session_id: turn.session_id.clone(),
            turn_id: turn.id.clone(),
            turn_number: turn.turn_number,
            timestamp: turn.metadata.timestamp,
            extra: std::collections::HashMap::new(),
        };

        self.full_text_index
            .add(&format!("doc_{}", turn.id), &gist, fts_metadata)
            .await?;

        Ok(record)
    }

    async fn list_indices(
        &self,
        session_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<IndexRecord>> {
        let vector_results = self
            .vector_index
            .search(&vec![0.0; 384], session_id, limit + offset)
            .await?;

        let mut indices: Vec<IndexRecord> = Vec::with_capacity(limit);
        for (i, result) in vector_results.iter().enumerate() {
            if i >= offset {
                indices.push(IndexRecord::new(
                    &result.turn_id,
                    session_id,
                    "",
                    result.metadata.timestamp,
                    result.metadata.turn_number,
                ));
                if indices.len() >= limit {
                    break;
                }
            }
        }

        if indices.is_empty() {
            let fts_results = self
                .full_text_index
                .search("", session_id, limit + offset)
                .await?;
            for (i, result) in fts_results.iter().enumerate() {
                if i >= offset {
                    indices.push(IndexRecord::new(
                        &result.turn_id,
                        session_id,
                        &result.gist,
                        result.metadata.timestamp,
                        result.metadata.turn_number,
                    ));
                    if indices.len() >= limit {
                        break;
                    }
                }
            }
        }

        Ok(indices)
    }

    async fn search_indices(
        &self,
        session_id: &str,
        query: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let limit = options.limit.max(10);

        let vector_results = if options.use_semantic || options.use_hybrid {
            let query_embedding = self.embedding_model.encode(query).await?;
            Some(
                self.vector_index
                    .search(&query_embedding, session_id, limit)
                    .await?,
            )
        } else {
            None
        };

        let fts_results = if options.use_full_text || options.use_hybrid {
            Some(
                self.full_text_index
                    .search(query, session_id, limit)
                    .await?,
            )
        } else {
            None
        };

        match (vector_results, fts_results) {
            (Some(vr), None) => Ok(vr
                .into_iter()
                .map(|r| SearchResult {
                    turn_id: r.turn_id,
                    gist: "".to_string(),
                    score: r.score,
                    result_type: SearchResultType::Semantic,
                    turn_number: r.metadata.turn_number,
                    timestamp: r.metadata.timestamp,
                    sources: vec!["vector".to_string()],
                })
                .collect()),
            (None, Some(fr)) => Ok(fr
                .into_iter()
                .map(|r| SearchResult {
                    turn_id: r.turn_id,
                    gist: r.gist,
                    score: r.score,
                    result_type: SearchResultType::FullText,
                    turn_number: r.metadata.turn_number,
                    timestamp: r.metadata.timestamp,
                    sources: vec!["full_text".to_string()],
                })
                .collect()),
            (Some(vr), Some(fr)) => Ok(Self::rrf_fusion(&vr, &fr, 60)),
            (None, None) => Ok(vec![]),
        }
    }

    async fn delete_index(&self, turn_id: &str) -> Result<bool> {
        let vector_deleted = self
            .vector_index
            .delete(&format!("vec_{}", turn_id))
            .await?;
        let fts_deleted = self
            .full_text_index
            .delete(&format!("doc_{}", turn_id))
            .await?;
        Ok(vector_deleted || fts_deleted)
    }
}

pub fn create_unified_index_service(
    vector_index: Box<dyn VectorIndex>,
    full_text_index: Box<dyn FullTextIndex>,
    embedding_model: Box<dyn EmbeddingModel>,
) -> Box<dyn IndexService> {
    Box::new(UnifiedIndexService::new(
        vector_index,
        full_text_index,
        embedding_model,
    ))
}
