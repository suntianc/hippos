//! Pattern 仓储
//!
//! 提供 Pattern 数据持久化服务

use async_trait::async_trait;
use std::marker::PhantomData;
use crate::error::Result;
use crate::models::pattern::{Pattern, PatternQuery, PatternStats, PatternUsage};
use crate::storage::surrealdb::SurrealPool;

/// Pattern 仓储 trait
#[async_trait]
pub trait PatternRepository {
    /// 创建模式
    async fn create(&self, pattern: &Pattern) -> Result<Pattern>;

    /// 根据 ID 获取模式
    async fn get_by_id(&self, id: &str) -> Result<Option<Pattern>>;

    /// 更新模式
    async fn update(&self, id: &str, pattern: &Pattern) -> Result<Option<Pattern>>;

    /// 删除模式
    async fn delete(&self, id: &str) -> Result<bool>;

    /// 列出模式
    async fn list(&self, limit: usize, start: usize) -> Result<Vec<Pattern>>;

    /// 统计数量
    async fn count(&self) -> Result<u64>;

    /// 根据条件查询
    async fn search(&self, query: &PatternQuery) -> Result<Vec<Pattern>>;

    /// 记录使用
    async fn record_usage(&self, pattern_id: &str, usage: &PatternUsage) -> Result<String>;

    /// 获取统计信息
    async fn get_stats(&self) -> Result<PatternStats>;

    /// 匹配模式
    async fn match_patterns(&self, input: &str, limit: u32) -> Result<Vec<Pattern>>;
}

/// Pattern 仓储实现
#[derive(Clone)]
pub struct PatternRepositoryImpl {
    pool: SurrealPool,
    _marker: PhantomData<Pattern>,
}

impl PatternRepositoryImpl {
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
    fn parse_results(&self, results: &[serde_json::Value]) -> Vec<Pattern> {
        let mut patterns = Vec::new();
        for item in results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    for pattern_json in result {
                        match serde_json::from_value(pattern_json.clone()) {
                            Ok(pattern) => patterns.push(pattern),
                            Err(e) => tracing::warn!("Failed to deserialize pattern: {}", e),
                        }
                    }
                }
            }
        }
        patterns
    }
}

#[async_trait]
impl PatternRepository for PatternRepositoryImpl {
    async fn create(&self, pattern: &Pattern) -> Result<Pattern> {
        let pattern = pattern.clone();

        let examples_json =
            serde_json::to_string(&pattern.examples).unwrap_or_else(|_| "[]".to_string());

        let tags_json = serde_json::to_string(&pattern.tags).unwrap_or_else(|_| "[]".to_string());

        let query = format!(
            "CREATE pattern SET id = '{}', tenant_id = '{}', pattern_type = '{}', name = '{}', description = '{}', trigger = '{}', context = '{}', problem = '{}', solution = '{}', explanation = {}, examples = {}, success_count = {}, failure_count = {}, avg_outcome = {}, tags = {}, created_by = '{}', created_at = '{}', updated_at = '{}', usage_count = {}, is_public = {}, confidence = {}, version = {}",
            pattern.id,
            pattern.tenant_id,
            pattern.pattern_type,
            pattern.name.replace("'", "\\'"),
            pattern.description.replace("'", "\\'"),
            pattern.trigger.replace("'", "\\'"),
            pattern.context.replace("'", "\\'"),
            pattern.problem.replace("'", "\\'"),
            pattern.solution.replace("'", "\\'"),
            pattern.explanation.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            examples_json,
            pattern.success_count,
            pattern.failure_count,
            pattern.avg_outcome,
            tags_json,
            pattern.created_by,
            pattern.created_at.to_rfc3339(),
            pattern.updated_at.to_rfc3339(),
            pattern.usage_count,
            pattern.is_public,
            pattern.confidence,
            pattern.version,
        );

        self.execute_query(&query).await?;
        Ok(pattern)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Pattern>> {
        let query = format!("SELECT * FROM pattern WHERE id = {}", id);
        let results = self.execute_query(&query).await?;

        for item in &results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    if let Some(pattern_json) = result.first() {
                        let pattern = serde_json::from_value(pattern_json.clone()).map_err(|e| {
                            crate::error::AppError::Database(format!(
                                "Failed to deserialize pattern: {}",
                                e
                            ))
                        })?;
                        return Ok(Some(pattern));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn update(&self, id: &str, pattern: &Pattern) -> Result<Option<Pattern>> {
        let pattern = pattern.clone();

        let examples_json =
            serde_json::to_string(&pattern.examples).unwrap_or_else(|_| "[]".to_string());

        let tags_json = serde_json::to_string(&pattern.tags).unwrap_or_else(|_| "[]".to_string());

        let query = format!(
            "UPDATE pattern SET name = '{}', description = '{}', trigger = '{}', context = '{}', problem = '{}', solution = '{}', explanation = {}, examples = {}, success_count = {}, failure_count = {}, avg_outcome = {}, tags = {}, updated_at = '{}', usage_count = {}, confidence = {}, version = {} WHERE id = '{}'",
            pattern.name.replace("'", "\\'"),
            pattern.description.replace("'", "\\'"),
            pattern.trigger.replace("'", "\\'"),
            pattern.context.replace("'", "\\'"),
            pattern.problem.replace("'", "\\'"),
            pattern.solution.replace("'", "\\'"),
            pattern.explanation.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            examples_json,
            pattern.success_count,
            pattern.failure_count,
            pattern.avg_outcome,
            tags_json,
            pattern.updated_at.to_rfc3339(),
            pattern.usage_count,
            pattern.confidence,
            pattern.version,
            id,
        );

        self.execute_query(&query).await?;
        Ok(Some(pattern))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let query = format!("DELETE FROM pattern WHERE id = {}", id);
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

    async fn list(&self, limit: usize, start: usize) -> Result<Vec<Pattern>> {
        let query = format!(
            "SELECT * FROM pattern ORDER BY usage_count DESC LIMIT {} START {}",
            limit, start
        );
        let results = self.execute_query(&query).await?;
        Ok(self.parse_results(&results))
    }

    async fn count(&self) -> Result<u64> {
        let query = "SELECT count() FROM pattern GROUP ALL";
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

    async fn search(&self, query: &PatternQuery) -> Result<Vec<Pattern>> {
        let mut conditions = Vec::new();

        if !query.types.is_empty() {
            let types: Vec<String> = query.types.iter().map(|t| format!("'{}'", t)).collect();
            conditions.push(format!("pattern_type IN [{}]", types.join(",")));
        }

        if !query.tags.is_empty() {
            for tag in &query.tags {
                conditions.push(format!("tags CONTAINS '{}'", tag));
            }
        }

        if query.min_confidence > 0.0 {
            conditions.push(format!("confidence >= {}", query.min_confidence));
        }

        if let Some(min_rate) = query.min_success_rate {
            conditions.push(format!("(success_count / (success_count + failure_count)) >= {}", min_rate));
        }

        if let Some(keyword) = &query.keyword {
            conditions.push(format!("name CONTAINS '{}' OR description CONTAINS '{}' OR problem CONTAINS '{}'", keyword, keyword, keyword));
        }

        if query.public_only {
            conditions.push("is_public = true".to_string());
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let limit = query.page_size;
        let start = (query.page - 1) * query.page_size;

        let sql = format!(
            "SELECT * FROM pattern {} ORDER BY usage_count DESC LIMIT {} START {}",
            where_clause, limit, start
        );

        let results = self.execute_query(&sql).await?;
        Ok(self.parse_results(&results))
    }

    async fn record_usage(&self, pattern_id: &str, usage: &PatternUsage) -> Result<String> {
        let usage_json = serde_json::to_string(usage).unwrap_or_else(|_| "{}".to_string());

        let query = format!(
            "CREATE pattern_usage SET id = '{}', pattern_id = '{}', user_id = '{}', input = '{}', output = '{}', outcome = {}, feedback = {}, used_at = '{}', context = {}",
            usage.id,
            usage.pattern_id,
            usage.user_id,
            usage.input.replace("'", "\\'"),
            usage.output.replace("'", "\\'"),
            usage.outcome,
            usage.feedback.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            usage.used_at.to_rfc3339(),
            usage.context.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
        );

        self.execute_query(&query).await?;

        // 更新模式的统计信息
        let update_query = format!(
            "UPDATE pattern SET usage_count = usage_count + 1, last_used = '{}', success_count = success_count + {}, failure_count = failure_count + {}, avg_outcome = ((avg_outcome * usage_count) + {}) / (usage_count + 1) WHERE id = '{}'",
            usage.used_at.to_rfc3339(),
            if usage.outcome >= 0.0 { 1 } else { 0 },
            if usage.outcome >= 0.0 { 0 } else { 1 },
            usage.outcome,
            pattern_id,
        );

        self.execute_query(&update_query).await?;

        Ok(usage.id.clone())
    }

    async fn get_stats(&self) -> Result<PatternStats> {
        let query = "SELECT count() FROM pattern GROUP ALL";
        let results = self.execute_query(&query).await?;

        let total_count = match results.first() {
            Some(item) => {
                if let Some(json) = item.as_object() {
                    if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                        if let Some(count_json) = result.first() {
                            count_json.get("count").and_then(|v| v.as_u64()).unwrap_or(0)
                        } else {
                            0
                        }
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            None => 0,
        };

        Ok(PatternStats {
            total_count,
            problem_solution_count: 0,
            workflow_count: 0,
            best_practice_count: 0,
            common_error_count: 0,
            skill_count: 0,
            avg_success_rate: 0.5,
            high_quality_count: 0,
            total_usages: 0,
            most_used_pattern_id: String::new(),
            most_used_pattern_name: String::new(),
        })
    }

    async fn match_patterns(&self, input: &str, limit: u32) -> Result<Vec<Pattern>> {
        let input_lower = input.to_lowercase();
        let keywords: Vec<&str> = input_lower.split_whitespace().collect();

        // 构建触发条件匹配查询
        let conditions: Vec<String> = keywords
            .iter()
            .map(|k| format!("trigger CONTAINS '{}'", k))
            .collect();

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" OR "))
        };

        let sql = format!(
            "SELECT * FROM pattern {} ORDER BY usage_count DESC LIMIT {}",
            where_clause, limit
        );

        let results = self.execute_query(&sql).await?;
        Ok(self.parse_results(&results))
    }
}
