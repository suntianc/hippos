//! Memory Integration Service
//!
//! Provides memory consolidation, re-evaluation, and optimization services.
//! Implements periodic summarization, importance re-scoring, redundancy detection,
//! and relationship updates for the memory system.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info};

use crate::{
    models::{
        memory::{Memory, MemoryStatus, MemoryType},
        memory_repository::MemoryRepository,
        MemoryQuery,
    },
    services::memory_recall::MemoryRecall,
};

/// Configuration for memory integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryIntegrationConfig {
    /// Interval for periodic summarization (in seconds)
    pub summarization_interval: u64,
    /// Interval for importance re-evaluation (in seconds)
    pub importance_interval: u64,
    /// Interval for redundancy detection (in seconds)
    pub redundancy_interval: u64,
    /// Interval for relationship updates (in seconds)
    pub relationship_interval: u64,
    /// Minimum importance threshold to keep a memory
    pub min_importance: f32,
    /// Similarity threshold for redundancy detection
    pub similarity_threshold: f32,
    /// Maximum memories to process per batch
    pub batch_size: usize,
}

impl Default for MemoryIntegrationConfig {
    fn default() -> Self {
        Self {
            summarization_interval: 3600,      // 1 hour
            importance_interval: 1800,         // 30 minutes
            redundancy_interval: 7200,         // 2 hours
            relationship_interval: 3600,       // 1 hour
            min_importance: 0.1,
            similarity_threshold: 0.85,
            batch_size: 100,
        }
    }
}

/// Statistics for memory integration operations
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IntegrationStats {
    pub summarizations: u32,
    pub importance_updates: u32,
    pub merges: u32,
    pub relationship_updates: u32,
    pub archived_memories: u32,
    pub last_run: Option<String>,
}

/// Memory integration result
#[derive(Debug, Serialize, Deserialize)]
pub struct IntegrationResult {
    pub success: bool,
    pub memories_processed: usize,
    pub memories_updated: usize,
    pub memories_archived: usize,
    pub errors: Vec<String>,
}

/// Result of a redundancy check
#[derive(Debug, Serialize, Deserialize)]
pub struct RedundancyCheckResult {
    pub memory_id: String,
    pub similar_memories: Vec<SimilarMemory>,
    pub action: RedundancyAction,
}

/// A similar memory found during redundancy detection
#[derive(Debug, Serialize, Deserialize)]
pub struct SimilarMemory {
    pub id: String,
    pub similarity: f32,
    pub overlap_score: f32,
}

/// Action to take for redundant memories
#[derive(Debug, Serialize, Deserialize)]
pub enum RedundancyAction {
    Keep,
    Merge,
    Archive,
    Replace,
}

/// Memory integration trait
#[async_trait]
pub trait MemoryIntegrationService: Send + Sync {
    /// Run the full integration pipeline
    async fn run_integration(&self) -> IntegrationResult;

    /// Run periodic summarization
    async fn run_summarization(&self) -> IntegrationResult;

    /// Re-evaluate memory importance
    async fn reevaluate_importance(&self) -> IntegrationResult;

    /// Detect and handle redundant memories
    async fn detect_redundancy(&self) -> Vec<RedundancyCheckResult>;

    /// Update memory relationships
    async fn update_relationships(&self) -> IntegrationResult;

    /// Get integration statistics
    fn get_stats(&self) -> IntegrationStats;

    /// Start the background integration task
    async fn start_background_task(&self);
}

/// Memory integration service implementation
pub struct MemoryIntegrator {
    config: MemoryIntegrationConfig,
    memory_repo: Arc<dyn MemoryRepository + Send + Sync>,
    memory_recall: Arc<MemoryRecall>,
    stats: Arc<tokio::sync::Mutex<IntegrationStats>>,
}

impl MemoryIntegrator {
    /// Create a new memory integrator
    pub fn new(
        memory_repo: Arc<dyn MemoryRepository + Send + Sync>,
        memory_recall: Arc<MemoryRecall>,
        config: Option<MemoryIntegrationConfig>,
    ) -> Self {
        Self {
            config: config.unwrap_or_default(),
            memory_repo,
            memory_recall,
            stats: Arc::new(tokio::sync::Mutex::new(IntegrationStats::default())),
        }
    }

    /// Calculate similarity between two memories
    fn calculate_similarity(&self, m1: &Memory, m2: &Memory) -> f32 {
        // If both have embeddings, use cosine similarity
        if let (Some(e1), Some(e2)) = (&m1.embedding, &m2.embedding) {
            return self.cosine_similarity(e1, e2);
        }

        // Fall back to keyword overlap
        self.keyword_overlap(&m1.keywords, &m2.keywords)
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(&self, v1: &[f32], v2: &[f32]) -> f32 {
        if v1.len() != v2.len() {
            return 0.0;
        }

        let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let norm1: f32 = v1.iter().map(|v| v * v).sum::<f32>().sqrt();
        let norm2: f32 = v2.iter().map(|v| v * v).sum::<f32>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0;
        }

        dot_product / (norm1 * norm2)
    }

    /// Calculate keyword overlap score
    fn keyword_overlap(&self, k1: &[String], k2: &[String]) -> f32 {
        if k1.is_empty() || k2.is_empty() {
            return 0.0;
        }

        let set1: std::collections::HashSet<&str> = k1.iter().map(|s| s.as_str()).collect();
        let set2: std::collections::HashSet<&str> = k2.iter().map(|s| s.as_str()).collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
}

#[async_trait]
impl MemoryIntegrationService for MemoryIntegrator {
    async fn run_integration(&self) -> IntegrationResult {
        info!("Starting full memory integration");

        let mut result = IntegrationResult {
            success: true,
            memories_processed: 0,
            memories_updated: 0,
            memories_archived: 0,
            errors: Vec::new(),
        };

        // Run all integration steps
        let summarization = self.run_summarization().await;
        let importance = self.reevaluate_importance().await;
        let redundancy = self.detect_redundancy().await;
        let relationships = self.update_relationships().await;

        // Aggregate results
        result.memories_processed = summarization.memories_processed
            + importance.memories_processed
            + redundancy.len() as usize
            + relationships.memories_processed;

        result.memories_updated = summarization.memories_updated
            + importance.memories_updated
            + relationships.memories_updated;

        result.memories_archived = summarization.memories_archived
            + importance.memories_archived
            + redundancy
                .iter()
                .filter(|r| matches!(r.action, RedundancyAction::Archive))
                .count()
            + relationships.memories_archived;

        result.errors.extend(summarization.errors);
        result.errors.extend(importance.errors);
        result.errors.extend(redundancy.iter().flat_map(|r| {
            if let RedundancyAction::Archive = r.action {
                vec![format!("Archived redundant memory: {}", r.memory_id)]
            } else {
                Vec::new()
            }
        }));
        result.errors.extend(relationships.errors);

        {
            let mut stats = self.stats.lock().await;
            stats.summarizations += summarization.memories_processed as u32;
            stats.importance_updates += importance.memories_updated as u32;
            stats.merges += redundancy
                .iter()
                .filter(|r| matches!(r.action, RedundancyAction::Merge))
                .count() as u32;
            stats.relationship_updates += relationships.memories_updated as u32;
            stats.archived_memories += result.memories_archived as u32;
            stats.last_run = Some(chrono::Utc::now().to_rfc3339());
        }

        info!(
            "Memory integration complete: {} processed, {} updated, {} archived",
            result.memories_processed, result.memories_updated, result.memories_archived
        );

        result
    }

    async fn run_summarization(&self) -> IntegrationResult {
        debug!("Running periodic summarization");

        let mut result = IntegrationResult {
            success: true,
            memories_processed: 0,
            memories_updated: 0,
            memories_archived: 0,
            errors: Vec::new(),
        };

        // Find memories that need summarization (old episodic memories without gist)
        let query = MemoryQuery {
            memory_types: vec![MemoryType::Episodic],
            statuses: vec![MemoryStatus::Active],
            page: 1,
            page_size: self.config.batch_size as u32,
            ..Default::default()
        };

        match self.memory_repo.search(&query).await {
            Ok(memories) => {
                for memory in memories {
                    // Skip if already has a gist
                    if !memory.gist.is_empty() {
                        continue;
                    }

                    // Generate gist (simplified - in real implementation, use LLM)
                    let gist = self.generate_gist(&memory.content).await;

                    let mut updated_memory = memory.clone();
                    updated_memory.gist = gist;

                    match self
                        .memory_repo
                        .update(&memory.id, &updated_memory)
                        .await
                    {
                        Ok(_) => result.memories_updated += 1,
                        Err(e) => result.errors.push(format!("Failed to update memory {}: {}", memory.id, e)),
                    }

                    result.memories_processed += 1;
                }
            }
            Err(e) => {
                result.success = false;
                result.errors.push(format!("Failed to search memories: {}", e));
            }
        }

        result
    }

    async fn reevaluate_importance(&self) -> IntegrationResult {
        debug!("Re-evaluating memory importance");

        let mut result = IntegrationResult {
            success: true,
            memories_processed: 0,
            memories_updated: 0,
            memories_archived: 0,
            errors: Vec::new(),
        };

        // Find all active memories
        let query = MemoryQuery {
            statuses: vec![MemoryStatus::Active],
            page: 1,
            page_size: self.config.batch_size as u32,
            ..Default::default()
        };

        match self.memory_repo.search(&query).await {
            Ok(memories) => {
                for memory in memories {
                    // Re-calculate importance based on current factors
                    let new_importance = self.calculate_importance(&memory);

                    // Archive memories below threshold
                    if new_importance < self.config.min_importance {
                        let mut updated_memory = memory.clone();
                        updated_memory.status = MemoryStatus::Archived;

                        match self
                            .memory_repo
                            .update(&memory.id, &updated_memory)
                            .await
                        {
                            Ok(_) => result.memories_archived += 1,
                            Err(e) => result.errors.push(format!("Failed to archive memory {}: {}", memory.id, e)),
                        }
                    } else if (new_importance - memory.importance).abs() > 0.1 {
                        // Update if importance changed significantly
                        let mut updated_memory = memory.clone();
                        updated_memory.importance = new_importance;

                        match self
                            .memory_repo
                            .update(&memory.id, &updated_memory)
                            .await
                        {
                            Ok(_) => result.memories_updated += 1,
                            Err(e) => result.errors.push(format!("Failed to update memory {}: {}", memory.id, e)),
                        }
                    }

                    result.memories_processed += 1;
                }
            }
            Err(e) => {
                result.success = false;
                result.errors.push(format!("Failed to search memories: {}", e));
            }
        }

        result
    }

    async fn detect_redundancy(&self) -> Vec<RedundancyCheckResult> {
        debug!("Detecting redundant memories");

        let mut results = Vec::new();

        // Get all active memories
        let query = MemoryQuery {
            statuses: vec![MemoryStatus::Active],
            page: 1,
            page_size: (self.config.batch_size * 2) as u32,
            ..Default::default()
        };

        let memories = match self.memory_repo.search(&query).await {
            Ok(m) => m,
            Err(e) => {
                error!("Failed to search memories for redundancy detection: {}", e);
                return results;
            }
        };

        // Compare memories for redundancy
        for i in 0..memories.len() {
            for j in (i + 1)..memories.len() {
                let m1 = &memories[i];
                let m2 = &memories[j];

                let similarity = self.calculate_similarity(m1, m2);

                if similarity >= self.config.similarity_threshold {
                    // Determine action based on factors
                    let action = if m1.importance > m2.importance {
                        RedundancyAction::Archive
                    } else if m1.content.len() > m2.content.len() {
                        RedundancyAction::Replace
                    } else {
                        RedundancyAction::Merge
                    };

                    results.push(RedundancyCheckResult {
                        memory_id: m2.id.clone(),
                        similar_memories: vec![SimilarMemory {
                            id: m1.id.clone(),
                            similarity,
                            overlap_score: self.keyword_overlap(&m1.keywords, &m2.keywords),
                        }],
                        action,
                    });
                }
            }
        }

        // Handle redundancy results
        for result in &results {
            match result.action {
                RedundancyAction::Archive => {
                    if let Ok(Some(memory)) = self.memory_repo.get_by_id(&result.memory_id).await {
                        let mut updated_memory = memory.clone();
                        updated_memory.status = MemoryStatus::Archived;

                        if let Err(e) = self.memory_repo.update(&result.memory_id, &updated_memory).await {
                            error!("Failed to archive memory {}: {}", result.memory_id, e);
                        }
                    }
                }
                RedundancyAction::Merge => {
                    // In a full implementation, merge content and relationships
                    debug!("Would merge memory {} into similar", result.memory_id);
                }
                RedundancyAction::Replace => {
                    if let Err(e) = self.memory_repo.delete(&result.memory_id).await {
                        error!("Failed to delete memory {}: {}", result.memory_id, e);
                    }
                }
                RedundancyAction::Keep => {}
            }
        }

        results
    }

    async fn update_relationships(&self) -> IntegrationResult {
        debug!("Updating memory relationships");

        let mut result = IntegrationResult {
            success: true,
            memories_processed: 0,
            memories_updated: 0,
            memories_archived: 0,
            errors: Vec::new(),
        };

        // Find memories without relationships that might benefit from them
        let query = MemoryQuery {
            statuses: vec![MemoryStatus::Active],
            page: 1,
            page_size: self.config.batch_size as u32,
            ..Default::default()
        };

        match self.memory_repo.search(&query).await {
            Ok(memories) => {
                for memory in memories {
                    // Find related memories based on topics and keywords
                    let related = self.find_related_memories(&memory).await;

                    if !related.is_empty() {
                        let current_related: std::collections::HashSet<String> =
                            memory.related_ids.iter().cloned().collect();

                        let new_related: Vec<String> = related
                            .into_iter()
                            .filter(|id| !current_related.contains(id))
                            .take(5)
                            .collect();

                        if !new_related.is_empty() {
                            let mut updated_related = memory.related_ids.clone();
                            updated_related.extend(new_related);

                            let mut updated_memory = memory.clone();
                            updated_memory.related_ids = updated_related;

                            match self
                                .memory_repo
                                .update(&memory.id, &updated_memory)
                                .await
                            {
                                Ok(_) => result.memories_updated += 1,
                                Err(e) => {
                                    result.errors.push(format!("Failed to update relationships: {}", e))
                                }
                            }
                        }
                    }

                    result.memories_processed += 1;
                }
            }
            Err(e) => {
                result.success = false;
                result.errors.push(format!("Failed to search memories: {}", e));
            }
        }

        result
    }

    fn get_stats(&self) -> IntegrationStats {
        // For now, return default
        IntegrationStats::default()
    }

    async fn start_background_task(&self) {
        let config = self.config.clone();
        let integrator = self.clone();

        tokio::spawn(async move {
            let mut summarization_interval = interval(Duration::from_secs(config.summarization_interval));
            let mut importance_interval = interval(Duration::from_secs(config.importance_interval));
            let mut redundancy_interval = interval(Duration::from_secs(config.redundancy_interval));
            let mut relationship_interval = interval(Duration::from_secs(config.relationship_interval));

            loop {
                tokio::select! {
                    _ = summarization_interval.tick() => {
                        let _ = integrator.run_summarization().await;
                    }
                    _ = importance_interval.tick() => {
                        let _ = integrator.reevaluate_importance().await;
                    }
                    _ = redundancy_interval.tick() => {
                        let _ = integrator.detect_redundancy().await;
                    }
                    _ = relationship_interval.tick() => {
                        let _ = integrator.update_relationships().await;
                    }
                }
            }
        });
    }
}

impl MemoryIntegrator {
    /// Generate a gist/summary for a memory
    async fn generate_gist(&self, content: &str) -> String {
        // Simplified gist generation - in production, use an LLM
        let words: Vec<&str> = content.split_whitespace().take(50).collect();
        words.join(" ")
    }

    /// Calculate importance score for a memory
    fn calculate_importance(&self, memory: &Memory) -> f32 {
        // Base importance from original calculation
        let mut importance = memory.importance;

        // Adjust based on age (older active memories might be less important)
        let age_hours = (chrono::Utc::now()
            .signed_duration_since(memory.created_at)
            .num_seconds() as f32)
            / 3600.0;

        if age_hours > 24.0 && age_hours < 168.0 {
            // 1-7 days old: slight decrease
            importance *= 0.95;
        } else if age_hours >= 168.0 {
            // Older than 7 days: more decrease
            importance *= 0.85;
        }

        // Boost for frequently accessed memories
        let recent_access_hours = (chrono::Utc::now()
            .signed_duration_since(memory.accessed_at)
            .num_seconds() as f32)
            / 3600.0;

        if recent_access_hours < 24.0 {
            importance = (importance + 0.1).min(1.0);
        }

        importance
    }

    /// Find related memories based on topics and keywords
    async fn find_related_memories(&self, memory: &Memory) -> Vec<String> {
        let mut related = Vec::new();

        for topic in &memory.topics {
            let query = MemoryQuery {
                topics: vec![topic.clone()],
                statuses: vec![MemoryStatus::Active],
                page: 1,
                page_size: 5,
                ..Default::default()
            };

            match self.memory_repo.search(&query).await {
                Ok(results) => {
                    for result in results {
                        if result.id != memory.id && !related.contains(&result.id) {
                            related.push(result.id);
                        }
                    }
                }
                Err(e) => {
                    debug!("Failed to search related memories for topic {}: {}", topic, e);
                }
            }
        }

        related
    }
}

impl Clone for MemoryIntegrator {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            memory_repo: self.memory_repo.clone(),
            memory_recall: self.memory_recall.clone(),
            stats: self.stats.clone(),
        }
    }
}
