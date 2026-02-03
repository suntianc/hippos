//! Memory 仓储
//!
//! 提供 Memory 数据持久化服务

use async_trait::async_trait;
use std::marker::PhantomData;
use crate::error::Result;
use crate::models::memory::{Memory, MemoryQuery, MemoryStats};
use crate::storage::surrealdb::SurrealPool;

/// Memory 仓储 trait
#[async_trait]
pub trait MemoryRepository {
    /// 创建记忆
    async fn create(&self, memory: &Memory) -> Result<Memory>;

    /// 根据 ID 获取记忆
    async fn get_by_id(&self, id: &str) -> Result<Option<Memory>>;

    /// 更新记忆
    async fn update(&self, id: &str, memory: &Memory) -> Result<Option<Memory>>;

    /// 删除记忆
    async fn delete(&self, id: &str) -> Result<bool>;

    /// 列出记忆
    async fn list(&self, limit: usize, start: usize) -> Result<Vec<Memory>>;

    /// 统计数量
    async fn count(&self) -> Result<u64>;

    /// 根据用户 ID 列出记忆
    async fn list_by_user(
        &self,
        user_id: &str,
        memory_type: Option<&str>,
        limit: usize,
        start: usize,
    ) -> Result<Vec<Memory>>;

    /// 统计用户记忆数量
    async fn count_by_user(&self, user_id: &str) -> Result<u64>;

    /// 搜索记忆
    async fn search(&self, query: &MemoryQuery) -> Result<Vec<Memory>>;

    /// 获取记忆统计
    async fn get_stats(&self, user_id: &str) -> Result<MemoryStats>;
}

/// Memory 仓储实现
#[derive(Clone)]
pub struct MemoryRepositoryImpl {
    pool: SurrealPool,
    _marker: PhantomData<Memory>,
}

impl MemoryRepositoryImpl {
    pub fn new(pool: SurrealPool) -> Self {
        Self {
            pool,
            _marker: PhantomData,
        }
    }

    /// 执行 SurrealDB 查询
    async fn execute_query(&self, query: &str) -> Result<Vec<serde_json::Value>> {
        let config = self.pool.config();
        let url = format!(
            "{}/sql",
            config.url.replace("ws://", "http://").replace("/rpc", "")
        );

        tracing::debug!("Executing query: {}", query);

        let response = self
            .pool
            .http_client()
            .post(&url)
            .header("surreal-ns", &config.namespace)
            .header("surreal-db", &config.database)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(&config.username, Some(&config.password))
            .body(query.to_string())
            .send()
            .await
            .map_err(|e| crate::error::AppError::Database(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::error::AppError::Database(format!(
                "SurrealDB error: {}",
                error_text
            )));
        }

        let response_text = response.text().await.unwrap_or_default();
        let results: Vec<serde_json::Value> =
            serde_json::from_str(&response_text).map_err(|e| {
                crate::error::AppError::Database(format!("Failed to parse response: {}", e))
            })?;

        Ok(results)
    }

    /// 从查询结果解析
    fn parse_results(&self, results: &[serde_json::Value]) -> Vec<Memory> {
        let mut memories = Vec::new();
        for item in results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    for memory_json in result {
                        match serde_json::from_value(memory_json.clone()) {
                            Ok(memory) => memories.push(memory),
                            Err(e) => tracing::warn!("Failed to deserialize memory: {}", e),
                        }
                    }
                }
            }
        }
        memories
    }
}

#[async_trait]
impl MemoryRepository for MemoryRepositoryImpl {
    async fn create(&self, memory: &Memory) -> Result<Memory> {
        let memory = memory.clone();
        let memory_json = serde_json::to_string(&memory).unwrap_or_else(|_| "{}".to_string());

        let query = format!(
            "CREATE memory SET id = '{}', tenant_id = '{}', user_id = '{}', memory_type = '{}', content = '{}', gist = '{}', embedding = {}, importance = {}, status = '{}', version = {}, created_at = '{}', updated_at = '{}'",
            memory.id,
            memory.tenant_id,
            memory.user_id,
            memory.memory_type,
            memory.content.replace("'", "\\'"),
            memory.gist.replace("'", "\\'"),
            "[]", // embedding will be set separately
            memory.importance,
            memory.status,
            memory.version,
            memory.created_at.to_rfc3339(),
            memory.updated_at.to_rfc3339(),
        );

        self.execute_query(&query).await?;
        Ok(memory)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Memory>> {
        let query = format!("SELECT * FROM memory WHERE id = {}", id);
        let results = self.execute_query(&query).await?;

        for item in &results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    if let Some(memory_json) = result.first() {
                        let memory = serde_json::from_value(memory_json.clone()).map_err(|e| {
                            crate::error::AppError::Database(format!(
                                "Failed to deserialize memory: {}",
                                e
                            ))
                        })?;
                        return Ok(Some(memory));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn update(&self, id: &str, memory: &Memory) -> Result<Option<Memory>> {
        let memory = memory.clone();

        let query = format!(
            "UPDATE memory SET content = '{}', gist = '{}', importance = {}, status = '{}', version = {} WHERE id = '{}'",
            memory.content.replace("'", "\\'"),
            memory.gist.replace("'", "\\'"),
            memory.importance,
            memory.status,
            memory.version,
            id,
        );

        self.execute_query(&query).await?;
        Ok(Some(memory))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let query = format!("DELETE FROM memory WHERE id = {}", id);
        let results = self.execute_query(&query).await?;

        for item in &results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    return Ok(result.len() > 0);
                }
            }
        }

        Ok(false)
    }

    async fn list(&self, limit: usize, start: usize) -> Result<Vec<Memory>> {
        let query = format!(
            "SELECT * FROM memory ORDER BY created_at DESC LIMIT {} START {}",
            limit, start
        );
        let results = self.execute_query(&query).await?;
        Ok(self.parse_results(&results))
    }

    async fn count(&self) -> Result<u64> {
        let query = "SELECT count() FROM memory GROUP ALL";
        let results = self.execute_query(&query).await?;

        for item in &results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    if let Some(count_json) = result.first() {
                        if let Some(count) = count_json.get("count").and_then(|v| v.as_u64()) {
                            return Ok(count);
                        }
                    }
                }
            }
        }

        Ok(0)
    }

    async fn list_by_user(
        &self,
        user_id: &str,
        memory_type: Option<&str>,
        limit: usize,
        start: usize,
    ) -> Result<Vec<Memory>> {
        let type_filter = match memory_type {
            Some(t) => format!("AND memory_type = '{}'", t),
            None => String::new(),
        };

        let query = format!(
            "SELECT * FROM memory WHERE user_id = '{}' {} ORDER BY created_at DESC LIMIT {} START {}",
            user_id, type_filter, limit, start
        );

        let results = self.execute_query(&query).await?;
        Ok(self.parse_results(&results))
    }

    async fn count_by_user(&self, user_id: &str) -> Result<u64> {
        let query = format!(
            "SELECT count() FROM memory WHERE user_id = '{}' GROUP ALL",
            user_id
        );
        let results = self.execute_query(&query).await?;

        for item in &results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    if let Some(count_json) = result.first() {
                        if let Some(count) = count_json.get("count").and_then(|v| v.as_u64()) {
                            return Ok(count);
                        }
                    }
                }
            }
        }

        Ok(0)
    }

    async fn search(&self, query: &MemoryQuery) -> Result<Vec<Memory>> {
        // 构建查询条件
        let mut conditions = Vec::new();

        if let Some(user_id) = &query.user_id {
            conditions.push(format!("user_id = '{}'", user_id));
        }

        if !query.memory_types.is_empty() {
            let types: Vec<String> = query
                .memory_types
                .iter()
                .map(|t| format!("'{}'", format!("{}", t)))
                .collect();
            conditions.push(format!("memory_type IN [{}]", types.join(",")));
        }

        if !query.sources.is_empty() {
            let sources: Vec<String> = query
                .sources
                .iter()
                .map(|s| format!("'{}'", format!("{}", s)))
                .collect();
            conditions.push(format!("source IN [{}]", sources.join(",")));
        }

        if let Some(min_importance) = query.min_importance {
            conditions.push(format!("importance >= {}", min_importance));
        }

        if !query.statuses.is_empty() {
            let statuses: Vec<String> = query
                .statuses
                .iter()
                .map(|s| format!("'{}'", format!("{}", s)))
                .collect();
            conditions.push(format!("status IN [{}]", statuses.join(",")));
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let limit = query.page_size;
        let start = (query.page - 1) * query.page_size;

        let sql = format!(
            "SELECT * FROM memory {} ORDER BY created_at DESC LIMIT {} START {}",
            where_clause, limit, start
        );

        let results = self.execute_query(&sql).await?;
        Ok(self.parse_results(&results))
    }

    async fn get_stats(&self, user_id: &str) -> Result<MemoryStats> {
        // 获取各类型数量
        let episodic_count = self.count_by_type(user_id, "episodic").await?;
        let semantic_count = self.count_by_type(user_id, "semantic").await?;
        let procedural_count = self.count_by_type(user_id, "procedural").await?;
        let profile_count = self.count_by_type(user_id, "profile").await?;

        // 获取总数量
        let total_count = self.count_by_user(user_id).await?;

        // 获取活跃数量
        let active_count = self.count_by_status(user_id, "active").await?;

        // 计算平均重要性
        let avg_importance = self.avg_importance(user_id).await?;

        // 获取高重要性数量
        let high_importance_count = self.count_high_importance(user_id).await?;

        Ok(MemoryStats {
            user_id: user_id.to_string(),
            total_count,
            episodic_count,
            semantic_count,
            procedural_count,
            profile_count,
            active_count,
            archived_count: total_count.saturating_sub(active_count),
            avg_importance,
            high_importance_count,
            storage_size_bytes: 0,
        })
    }
}

impl MemoryRepositoryImpl {
    async fn count_by_type(&self, user_id: &str, memory_type: &str) -> Result<u64> {
        let query = format!(
            "SELECT count() FROM memory WHERE user_id = '{}' AND memory_type = '{}' GROUP ALL",
            user_id, memory_type
        );
        let results = self.execute_query(&query).await?;

        for item in &results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    if let Some(count_json) = result.first() {
                        if let Some(count) = count_json.get("count").and_then(|v| v.as_u64()) {
                            return Ok(count);
                        }
                    }
                }
            }
        }

        Ok(0)
    }

    async fn count_by_status(&self, user_id: &str, status: &str) -> Result<u64> {
        let query = format!(
            "SELECT count() FROM memory WHERE user_id = '{}' AND status = '{}' GROUP ALL",
            user_id, status
        );
        let results = self.execute_query(&query).await?;

        for item in &results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    if let Some(count_json) = result.first() {
                        if let Some(count) = count_json.get("count").and_then(|v| v.as_u64()) {
                            return Ok(count);
                        }
                    }
                }
            }
        }

        Ok(0)
    }

    async fn avg_importance(&self, user_id: &str) -> Result<f32> {
        let query = format!(
            "SELECT avg(importance) FROM memory WHERE user_id = '{}' GROUP ALL",
            user_id
        );
        let results = self.execute_query(&query).await?;

        for item in &results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    if let Some(avg_json) = result.first() {
                        if let Some(avg) = avg_json.get("avg").and_then(|v| v.as_f64()) {
                            return Ok(avg as f32);
                        }
                    }
                }
            }
        }

        Ok(0.0)
    }

    async fn count_high_importance(&self, user_id: &str) -> Result<u64> {
        let query = format!(
            "SELECT count() FROM memory WHERE user_id = '{}' AND importance > 0.7 GROUP ALL",
            user_id
        );
        let results = self.execute_query(&query).await?;

        for item in &results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    if let Some(count_json) = result.first() {
                        if let Some(count) = count_json.get("count").and_then(|v| v.as_u64()) {
                            return Ok(count);
                        }
                    }
                }
            }
        }

        Ok(0)
    }
}
