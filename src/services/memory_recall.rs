//! 记忆召回服务
//!
//! 提供混合搜索能力：语义搜索 + 时间检索 + 上下文推理
//! 使用 RRF (Reciprocal Rank Fusion) 算法融合多路搜索结果

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::error::Result;
use crate::models::memory::{Memory, MemoryQuery, MemoryStats, MemoryType};
use crate::models::memory_repository::MemoryRepository;
use crate::models::profile_repository::ProfileRepository;
use crate::storage::surrealdb::SurrealPool;

/// RRF 融合权重配置
#[derive(Debug, Clone)]
pub struct RrfWeights {
    pub semantic: f32,
    pub temporal: f32,
    pub context: f32,
}

impl Default for RrfWeights {
    fn default() -> Self {
        Self {
            semantic: 0.6,
            temporal: 0.3,
            context: 0.1,
        }
    }
}

/// 时间范围查询
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

impl TimeRange {
    pub fn new(start: Option<DateTime<Utc>>, end: Option<DateTime<Utc>>) -> Self {
        Self { start, end }
    }

    pub fn recent(hours: i64) -> Self {
        Self {
            start: Some(Utc::now() - Duration::from_secs(hours as u64 * 3600)),
            end: Some(Utc::now()),
        }
    }

    pub fn today() -> Self {
        let now = Utc::now();
        let start_of_day = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .map(|ndt| ndt.and_utc())
            .unwrap_or(now);
        Self {
            start: Some(start_of_day),
            end: Some(now),
        }
    }
}

/// 搜索选项
#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub limit: u32,
    pub offset: u32,
    pub time_range: Option<TimeRange>,
    pub min_importance: Option<f32>,
    pub memory_types: Vec<String>,
    pub include_archived: bool,
    pub rrf_weights: RrfWeights,
}

impl SearchOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    pub fn with_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    pub fn with_time_range(mut self, time_range: TimeRange) -> Self {
        self.time_range = Some(time_range);
        self
    }

    pub fn with_min_importance(mut self, importance: f32) -> Self {
        self.min_importance = Some(importance);
        self
    }

    pub fn with_memory_types(mut self, types: &[&str]) -> Self {
        self.memory_types = types.iter().map(|s| s.to_string()).collect();
        self
    }

    pub fn include_archived(mut self, include: bool) -> Self {
        self.include_archived = include;
        self
    }

    pub fn with_rrf_weights(mut self, weights: RrfWeights) -> Self {
        self.rrf_weights = weights;
        self
    }
}

/// 搜索结果项
#[derive(Debug, Clone)]
pub struct SearchResultItem {
    pub memory: Memory,
    pub combined_score: f32,
    pub semantic_score: Option<f32>,
    pub temporal_score: f32,
    pub context_score: Option<f32>,
    pub rank_semantic: Option<u32>,
    pub rank_temporal: Option<u32>,
    pub rank_context: Option<u32>,
    pub match_reasons: Vec<String>,
}

/// 记忆召回服务
#[derive(Clone)]
pub struct MemoryRecall {
    pool: SurrealPool,
    memory_repo: Arc<dyn MemoryRepository + Send + Sync>,
    profile_repo: Arc<dyn ProfileRepository + Send + Sync>,
}

impl MemoryRecall {
    /// 创建新的 MemoryRecall 实例
    pub fn new(
        pool: SurrealPool,
        memory_repo: Arc<dyn MemoryRepository + Send + Sync>,
        profile_repo: Arc<dyn ProfileRepository + Send + Sync>,
    ) -> Self {
        Self {
            pool,
            memory_repo,
            profile_repo,
        }
    }

    /// 获取数据库连接池
    pub fn pool(&self) -> &SurrealPool {
        &self.pool
    }

    /// 获取记忆仓储
    pub fn memory_repo(&self) -> &Arc<dyn MemoryRepository + Send + Sync> {
        &self.memory_repo
    }

    /// 获取用户画像仓储
    pub fn profile_repo(&self) -> &Arc<dyn ProfileRepository + Send + Sync> {
        &self.profile_repo
    }
}

/// MemoryRecall 服务 trait
#[async_trait]
pub trait MemoryRecallService: Send + Sync {
    /// 混合搜索：语义 + 时间 + 上下文
    async fn hybrid_search(
        &self,
        user_id: &str,
        query: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResultItem>>;

    /// 纯语义搜索
    async fn semantic_search(
        &self,
        user_id: &str,
        query: &str,
        limit: u32,
    ) -> Result<Vec<SearchResultItem>>;

    /// 时间范围检索
    async fn temporal_search(
        &self,
        user_id: &str,
        time_range: TimeRange,
        limit: u32,
    ) -> Result<Vec<Memory>>;

    /// 上下文推理
    async fn contextual_inference(
        &self,
        user_id: &str,
        context: &str,
    ) -> Result<Vec<Memory>>;

    /// 获取最近的记忆
    async fn get_recent_memories(&self, user_id: &str, limit: u32) -> Result<Vec<Memory>>;

    /// 获取记忆统计
    async fn get_memory_stats(&self, user_id: &str) -> Result<MemoryStats>;
}

#[async_trait]
impl MemoryRecallService for MemoryRecall {
    /// 混合搜索：语义 + 时间 + 上下文
    async fn hybrid_search(
        &self,
        user_id: &str,
        query: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResultItem>> {
        let weights = options.rrf_weights.clone();
        let limit = options.limit as usize;

        // 并行执行三路搜索
        let (semantic_results, temporal_results, context_results) = tokio::try_join!(
            self.semantic_search_internal(user_id, query, limit, &options),
            self.temporal_search_internal(user_id, &options),
            self.contextual_inference_internal(user_id, query, limit)
        )?;

        // 使用 RRF 融合结果
        let fused_results = Self::rrf_fusion(
            semantic_results,
            temporal_results,
            context_results,
            &weights,
            limit,
        );

        Ok(fused_results)
    }

    /// 纯语义搜索
    async fn semantic_search(
        &self,
        user_id: &str,
        query: &str,
        limit: u32,
    ) -> Result<Vec<SearchResultItem>> {
        let options = SearchOptions::new().with_limit(limit);
        self.semantic_search_internal(user_id, query, limit as usize, &options)
            .await
    }

    /// 时间范围检索
    async fn temporal_search(
        &self,
        user_id: &str,
        time_range: TimeRange,
        limit: u32,
    ) -> Result<Vec<Memory>> {
        let options = SearchOptions::new()
            .with_limit(limit)
            .with_time_range(time_range);

        self.temporal_search_internal(user_id, &options)
            .await
            .map(|results| results.into_iter().map(|r| r.memory).collect())
    }

    /// 上下文推理
    async fn contextual_inference(
        &self,
        user_id: &str,
        context: &str,
    ) -> Result<Vec<Memory>> {
        self.contextual_inference_internal(user_id, context, 10)
            .await
            .map(|results| results.into_iter().map(|r| r.memory).collect())
    }

    /// 获取最近的记忆
    async fn get_recent_memories(&self, user_id: &str, limit: u32) -> Result<Vec<Memory>> {
        let query = MemoryQuery::new()
            .for_user(user_id)
            .with_pagination(1, limit);

        self.memory_repo.search(&query).await
    }

    /// 获取记忆统计
    async fn get_memory_stats(&self, user_id: &str) -> Result<MemoryStats> {
        self.memory_repo.get_stats(user_id).await
    }
}

impl MemoryRecall {
    /// 内部语义搜索实现
    async fn semantic_search_internal(
        &self,
        user_id: &str,
        query: &str,
        limit: usize,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResultItem>> {
        // 构建查询
        let mut memory_query = MemoryQuery::new()
            .for_user(user_id)
            .with_pagination(1, limit as u32);

        // 应用过滤条件
        if let Some(min_importance) = options.min_importance {
            memory_query = memory_query.with_min_importance(min_importance);
        }

        if !options.memory_types.is_empty() {
            // 将字符串类型转换为 MemoryType 枚举
            let types: Vec<MemoryType> = options
                .memory_types
                .iter()
                .filter_map(|s| match s.to_lowercase().as_str() {
                    "episodic" => Some(MemoryType::Episodic),
                    "semantic" => Some(MemoryType::Semantic),
                    "procedural" => Some(MemoryType::Procedural),
                    "profile" => Some(MemoryType::Profile),
                    _ => None,
                })
                .collect();
            if !types.is_empty() {
                memory_query = memory_query.with_types(&types);
            }
        }

        // 搜索记忆
        let memories = self.memory_repo.search(&memory_query).await?;

        // 计算语义相似度分数
        let mut results: Vec<SearchResultItem> = Vec::with_capacity(memories.len());

        // 简单的关键词匹配作为语义相似度的近似
        // 实际实现应该使用向量嵌入计算余弦相似度
        let query_lower = query.to_lowercase();
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();

        for (rank, memory) in memories.iter().enumerate() {
            let content_lower = memory.content.to_lowercase();
            let gist_lower = memory.gist.to_lowercase();

            // 计算关键词匹配分数
            let mut match_count = 0;
            let mut match_reasons = Vec::new();

            for word in &query_words {
                if content_lower.contains(word) {
                    match_count += 1;
                    if !match_reasons.contains(&"keyword_match".to_string()) {
                        match_reasons.push("keyword_match".to_string());
                    }
                }
            }

            // 检查gist匹配
            for word in &query_words {
                if gist_lower.contains(word) && !match_reasons.contains(&"gist_match".to_string()) {
                    match_reasons.push("gist_match".to_string());
                    break;
                }
            }

            // 检查标签和主题匹配
            for tag in &memory.tags {
                let tag_lower = tag.to_lowercase();
                if query_words.iter().any(|w| tag_lower.contains(w)) {
                    if !match_reasons.contains(&"tag_match".to_string()) {
                        match_reasons.push("tag_match".to_string());
                        break;
                    }
                }
            }

            for topic in &memory.topics {
                let topic_lower = topic.to_lowercase();
                if query_words.iter().any(|w| topic_lower.contains(w)) {
                    if !match_reasons.contains(&"topic_match".to_string()) {
                        match_reasons.push("topic_match".to_string());
                        break;
                    }
                }
            }

            // 计算语义分数 (0-1范围)
            let semantic_score_f = if match_count > 0 {
                (match_count as f32 / query_words.len().max(1) as f32)
                    .min(1.0)
                    .max(0.0)
            } else {
                0.1 // 默认低分
            };

            // 基础分数 = 语义分数 * 重要性
            let base_score = semantic_score_f * memory.importance;

            results.push(SearchResultItem {
                memory: memory.clone(),
                combined_score: base_score,
                semantic_score: Some(base_score),
                temporal_score: 0.0,
                context_score: None,
                rank_semantic: Some(rank as u32 + 1),
                rank_temporal: None,
                rank_context: None,
                match_reasons,
            });
        }

        // 按综合分数排序
        results.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());

        Ok(results)
    }

    /// 内部时间搜索实现
    async fn temporal_search_internal(
        &self,
        user_id: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResultItem>> {
        let limit = options.limit as usize;

        // 构建时间范围查询
        let mut memory_query = MemoryQuery::new()
            .for_user(user_id)
            .with_pagination(1, limit as u32);

        if let Some(time_range) = &options.time_range {
            memory_query = memory_query.with_time_range(time_range.start, time_range.end);
        }

        if let Some(min_importance) = options.min_importance {
            memory_query = memory_query.with_min_importance(min_importance);
        }

        let memories = self.memory_repo.search(&memory_query).await?;

        // 按时间排序 (最新的优先)
        let mut memories = memories;
        memories.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let mut results: Vec<SearchResultItem> = Vec::with_capacity(memories.len());

        for (rank, memory) in memories.iter().enumerate() {
            // 时间分数：越近越高
            let time_diff = Utc::now()
                .signed_duration_since(memory.created_at)
                .num_hours();

            let temporal_score = if time_diff < 1 {
                1.0
            } else if time_diff < 24 {
                0.9
            } else if time_diff < 168 {
                // 一周内
                0.7
            } else if time_diff < 720 {
                // 一个月内
                0.5
            } else {
                0.3
            };

            // 综合分数 = 时间分数 * 重要性
            let combined_score = temporal_score * memory.importance;

            results.push(SearchResultItem {
                memory: memory.clone(),
                combined_score,
                semantic_score: None,
                temporal_score,
                context_score: None,
                rank_semantic: None,
                rank_temporal: Some(rank as u32 + 1),
                rank_context: None,
                match_reasons: vec!["temporal_proximity".to_string()],
            });
        }

        Ok(results)
    }

    /// 内部上下文推理实现
    async fn contextual_inference_internal(
        &self,
        user_id: &str,
        context: &str,
        limit: usize,
    ) -> Result<Vec<SearchResultItem>> {
        // 获取用户画像以了解上下文
        let profile = self.profile_repo.get_by_user_id(user_id).await?;

        // 获取最近的记忆
        let recent_query = MemoryQuery::new()
            .for_user(user_id)
            .with_pagination(1, limit as u32 * 2);

        let recent_memories = self.memory_repo.search(&recent_query).await?;

        // 分析上下文关键词
        let context_lower = context.to_lowercase();
        let context_words: Vec<&str> = context_lower.split_whitespace().collect();

        // 匹配策略：
        // 1. 基于用户画像的兴趣和工具
        // 2. 基于当前上下文关键词
        // 3. 基于最近的对话主题

        let profile_interests = profile
            .as_ref()
            .map(|p| p.interests.clone())
            .unwrap_or_default();

        let profile_tools = profile
            .as_ref()
            .map(|p| p.tools_used.clone())
            .unwrap_or_default();

        let profile_topics = profile_interests;

        let mut results: Vec<SearchResultItem> = Vec::with_capacity(recent_memories.len());

        for (rank, memory) in recent_memories.iter().enumerate() {
            let mut context_score: f32 = 0.0;
            let mut match_reasons: Vec<String> = Vec::new();

            // 检查是否匹配用户画像兴趣
            for interest in &profile_topics {
                if memory.topics.contains(interest) || memory.tags.contains(interest) {
                    context_score += 0.3;
                    if !match_reasons.contains(&"profile_interest".to_string()) {
                        match_reasons.push("profile_interest".to_string());
                    }
                    break;
                }
            }

            // 检查是否匹配用户画像工具
            for tool in &profile_tools {
                if memory.content.to_lowercase().contains(&tool.to_lowercase())
                    || memory.tags.contains(&tool.to_lowercase())
                {
                    context_score += 0.2;
                    if !match_reasons.contains(&"profile_tool".to_string()) {
                        match_reasons.push("profile_tool".to_string());
                    }
                    break;
                }
            }

            // 检查上下文关键词匹配
            let content_lower = memory.content.to_lowercase();
            let gist_lower = memory.gist.to_lowercase();

            for word in &context_words {
                if word.len() > 2 {
                    if content_lower.contains(word) || gist_lower.contains(word) {
                        context_score += 0.15;
                        if !match_reasons.contains(&"context_keyword".to_string()) {
                            match_reasons.push("context_keyword".to_string());
                        }
                    }
                }
            }

            // 检查记忆主题是否与上下文相关
            for topic in &memory.topics {
                let topic_lower = topic.to_lowercase();
                if context_words.iter().any(|w| topic_lower.contains(w)) {
                    context_score += 0.25;
                    if !match_reasons.contains(&"topic_relevance".to_string()) {
                        match_reasons.push("topic_relevance".to_string());
                    }
                    break;
                }
            }

            // 归一化上下文分数
            context_score = context_score.min(1.0);

            // 综合分数
            let combined_score = context_score * memory.importance * 0.8
                + (1.0 - (rank as f32 / recent_memories.len() as f32).min(1.0)) * 0.2;

            if context_score > 0.0 || rank < limit {
                results.push(SearchResultItem {
                    memory: memory.clone(),
                    combined_score,
                    semantic_score: None,
                    temporal_score: 0.0,
                    context_score: Some(context_score),
                    rank_semantic: None,
                    rank_temporal: None,
                    rank_context: Some(rank as u32 + 1),
                    match_reasons,
                });
            }
        }

        // 排序并限制结果
        results.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());
        results.truncate(limit);

        Ok(results)
    }

    /// RRF 融合算法
    fn rrf_fusion(
        semantic_results: Vec<SearchResultItem>,
        temporal_results: Vec<SearchResultItem>,
        context_results: Vec<SearchResultItem>,
        weights: &RrfWeights,
        limit: usize,
    ) -> Vec<SearchResultItem> {
        // RRF 常数
        const K: u64 = 60;

        // 构建结果映射
        let mut result_map: HashMap<String, SearchResultItem> = HashMap::new();

        // 处理语义结果
        for (rank, item) in semantic_results.iter().enumerate() {
            let memory_id = item.memory.id.clone();
            let rrf_score = weights.semantic * (1.0 / (K + rank as u64) as f32);

            let mut reasons = item.match_reasons.clone();
            if !reasons.contains(&"semantic".to_string()) {
                reasons.push("semantic".to_string());
            }

            result_map.insert(
                memory_id.clone(),
                SearchResultItem {
                    memory: item.memory.clone(),
                    combined_score: rrf_score,
                    semantic_score: item.semantic_score,
                    temporal_score: item.temporal_score,
                    context_score: item.context_score,
                    rank_semantic: Some(rank as u32 + 1),
                    rank_temporal: None,
                    rank_context: None,
                    match_reasons: reasons,
                },
            );
        }

        // 处理时间结果
        for (rank, item) in temporal_results.iter().enumerate() {
            let memory_id = item.memory.id.clone();
            let rrf_score = weights.temporal * (1.0 / (K + rank as u64) as f32);

            if let Some(existing) = result_map.get_mut(&memory_id) {
                // 合并分数
                existing.combined_score += rrf_score;
                existing.temporal_score = item.temporal_score;
                if existing.rank_temporal.is_none() {
                    existing.rank_temporal = Some(rank as u32 + 1);
                }
                if !existing.match_reasons.contains(&"temporal".to_string()) {
                    existing.match_reasons.push("temporal".to_string());
                }
            } else {
                let mut reasons = item.match_reasons.clone();
                if !reasons.contains(&"temporal".to_string()) {
                    reasons.push("temporal".to_string());
                }

                result_map.insert(
                    memory_id.clone(),
                    SearchResultItem {
                        memory: item.memory.clone(),
                        combined_score: rrf_score,
                        semantic_score: item.semantic_score,
                        temporal_score: item.temporal_score,
                        context_score: item.context_score,
                        rank_semantic: None,
                        rank_temporal: Some(rank as u32 + 1),
                        rank_context: None,
                        match_reasons: reasons,
                    },
                );
            }
        }

        // 处理上下文结果
        for (rank, item) in context_results.iter().enumerate() {
            let memory_id = item.memory.id.clone();
            let rrf_score = weights.context * (1.0 / (K + rank as u64) as f32);

            if let Some(existing) = result_map.get_mut(&memory_id) {
                // 合并分数
                existing.combined_score += rrf_score;
                if let Some(context_score) = item.context_score {
                    if existing.context_score.unwrap_or(0.0) < context_score {
                        existing.context_score = item.context_score;
                    }
                }
                if existing.rank_context.is_none() {
                    existing.rank_context = Some(rank as u32 + 1);
                }
                if !existing.match_reasons.contains(&"contextual".to_string()) {
                    existing.match_reasons.push("contextual".to_string());
                }
            } else {
                let mut reasons = item.match_reasons.clone();
                if !reasons.contains(&"contextual".to_string()) {
                    reasons.push("contextual".to_string());
                }

                result_map.insert(
                    memory_id.clone(),
                    SearchResultItem {
                        memory: item.memory.clone(),
                        combined_score: rrf_score,
                        semantic_score: item.semantic_score,
                        temporal_score: item.temporal_score,
                        context_score: item.context_score,
                        rank_semantic: None,
                        rank_temporal: None,
                        rank_context: Some(rank as u32 + 1),
                        match_reasons: reasons,
                    },
                );
            }
        }

        // 转换为向量并排序
        let mut results: Vec<_> = result_map.into_values().collect();
        results.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());

        // 限制结果数量
        results.truncate(limit);

        results
    }
}

/// 创建 MemoryRecall 服务
pub fn create_memory_recall_service(
    pool: SurrealPool,
    memory_repo: Arc<dyn MemoryRepository + Send + Sync>,
    profile_repo: Arc<dyn ProfileRepository + Send + Sync>,
) -> MemoryRecall {
    MemoryRecall::new(pool, memory_repo, profile_repo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::memory::{Memory, MemorySource, MemoryStatus, MemoryType};
    use chrono::Utc;

    #[test]
    fn test_rrf_weights_default() {
        let weights = RrfWeights::default();
        assert_eq!(weights.semantic, 0.6);
        assert_eq!(weights.temporal, 0.3);
        assert_eq!(weights.context, 0.1);
    }

    #[test]
    fn test_time_range_recent() {
        let range = TimeRange::recent(24);
        assert!(range.start.is_some());
        assert!(range.end.is_some());
    }

    #[test]
    fn test_time_range_today() {
        let range = TimeRange::today();
        assert!(range.start.is_some());
        assert!(range.end.is_some());
    }

    #[test]
    fn test_search_options_builder() {
        let options = SearchOptions::new()
            .with_limit(10)
            .with_offset(5)
            .with_min_importance(0.7)
            .with_memory_types(&["episodic", "semantic"])
            .include_archived(true);

        assert_eq!(options.limit, 10);
        assert_eq!(options.offset, 5);
        assert_eq!(options.min_importance, Some(0.7));
        assert_eq!(options.memory_types.len(), 2);
        assert!(options.include_archived);
    }

    #[test]
    fn test_memory_creation_for_test() {
        let memory = Memory::new(
            "user_123",
            MemoryType::Episodic,
            "Test memory content",
            MemorySource::Conversation,
        );

        assert_eq!(memory.user_id, "user_123");
        assert_eq!(memory.memory_type, MemoryType::Episodic);
        assert_eq!(memory.status, MemoryStatus::Active);
    }

    #[test]
    fn test_rrf_fusion_scores() {
        // 创建测试数据
        let memory = Memory::new(
            "user_123",
            MemoryType::Episodic,
            "Test content",
            MemorySource::Conversation,
        );

        let semantic_item = SearchResultItem {
            memory: memory.clone(),
            combined_score: 0.0,
            semantic_score: Some(0.8),
            temporal_score: 0.0,
            context_score: None,
            rank_semantic: Some(1),
            rank_temporal: None,
            rank_context: None,
            match_reasons: vec!["semantic".to_string()],
        };

        let temporal_item = SearchResultItem {
            memory: memory.clone(),
            combined_score: 0.0,
            semantic_score: None,
            temporal_score: 0.9,
            context_score: None,
            rank_semantic: None,
            rank_temporal: Some(1),
            rank_context: None,
            match_reasons: vec!["temporal".to_string()],
        };

        let weights = RrfWeights::default();

        // 执行 RRF 融合
        let results = MemoryRecall::rrf_fusion(
            vec![semantic_item],
            vec![temporal_item],
            vec![],
            &weights,
            10,
        );

        assert_eq!(results.len(), 1);
        // 检查分数是否被正确合并
        assert!(results[0].combined_score > 0.0);
        // 检查匹配原因是否合并
        assert!(results[0].match_reasons.contains(&"semantic".to_string()));
        assert!(results[0].match_reasons.contains(&"temporal".to_string()));
    }
}
