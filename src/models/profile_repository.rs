//! Profile 仓储
//!
//! 提供 User Profile 数据持久化服务

use async_trait::async_trait;
use std::marker::PhantomData;
use crate::error::Result;
use crate::models::profile::{Profile, ProfileQuery, ProfileComparison};
use crate::storage::surrealdb::SurrealPool;

/// Profile 仓储 trait
#[async_trait]
pub trait ProfileRepository {
    /// 创建画像
    async fn create(&self, profile: &Profile) -> Result<Profile>;

    /// 根据 ID 获取画像
    async fn get_by_id(&self, id: &str) -> Result<Option<Profile>>;

    /// 根据用户 ID 获取画像
    async fn get_by_user_id(&self, user_id: &str) -> Result<Option<Profile>>;

    /// 更新画像
    async fn update(&self, id: &str, profile: &Profile) -> Result<Option<Profile>>;

    /// 删除画像
    async fn delete(&self, id: &str) -> Result<bool>;

    /// 列出画像
    async fn list(&self, limit: usize, start: usize) -> Result<Vec<Profile>>;

    /// 统计数量
    async fn count(&self) -> Result<u64>;

    /// 根据条件查询
    async fn search(&self, query: &ProfileQuery) -> Result<Vec<Profile>>;

    /// 合并画像
    async fn merge(&self, target_id: &str, source_id: &str, strategy: &str) -> Result<ProfileComparison>;
}

/// Profile 仓储实现
#[derive(Clone)]
pub struct ProfileRepositoryImpl {
    pool: SurrealPool,
    _marker: PhantomData<Profile>,
}

impl ProfileRepositoryImpl {
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
    fn parse_results(&self, results: &[serde_json::Value]) -> Vec<Profile> {
        let mut profiles = Vec::new();
        for item in results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    for profile_json in result {
                        match serde_json::from_value(profile_json.clone()) {
                            Ok(profile) => profiles.push(profile),
                            Err(e) => tracing::warn!("Failed to deserialize profile: {}", e),
                        }
                    }
                }
            }
        }
        profiles
    }
}

#[async_trait]
impl ProfileRepository for ProfileRepositoryImpl {
    async fn create(&self, profile: &Profile) -> Result<Profile> {
        let profile = profile.clone();

        let preferences_json =
            serde_json::to_string(&profile.preferences).unwrap_or_else(|_| "{}".to_string());

        let facts_json = serde_json::to_string(&profile.facts).unwrap_or_else(|_| "[]".to_string());

        let change_history_json =
            serde_json::to_string(&profile.change_history).unwrap_or_else(|_| "[]".to_string());

        let query = format!(
            "CREATE profile SET id = '{}', tenant_id = '{}', user_id = '{}', name = {}, role = {}, organization = {}, location = {}, preferences = {}, communication_style = {}, technical_level = {}, language = {}, facts = {}, interests = {}, working_hours = {}, common_tasks = {}, tools_used = {}, created_at = '{}', updated_at = '{}', confidence = {}, version = {}, change_history = {}",
            profile.id,
            profile.tenant_id,
            profile.user_id,
            profile.name.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            profile.role.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            profile.organization.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            profile.location.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            preferences_json,
            profile.communication_style.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            profile.technical_level.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            profile.language.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            facts_json,
            serde_json::to_string(&profile.interests).unwrap_or_else(|_| "[]".to_string()),
            serde_json::to_string(&profile.working_hours).unwrap_or_else(|_| "null".to_string()),
            serde_json::to_string(&profile.common_tasks).unwrap_or_else(|_| "[]".to_string()),
            serde_json::to_string(&profile.tools_used).unwrap_or_else(|_| "[]".to_string()),
            profile.created_at.to_rfc3339(),
            profile.updated_at.to_rfc3339(),
            profile.confidence,
            profile.version,
            change_history_json,
        );

        self.execute_query(&query).await?;
        Ok(profile)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Profile>> {
        let query = format!("SELECT * FROM profile WHERE id = {}", id);
        let results = self.execute_query(&query).await?;

        for item in &results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    if let Some(profile_json) = result.first() {
                        let profile = serde_json::from_value(profile_json.clone()).map_err(|e| {
                            crate::error::AppError::Database(format!(
                                "Failed to deserialize profile: {}",
                                e
                            ))
                        })?;
                        return Ok(Some(profile));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn get_by_user_id(&self, user_id: &str) -> Result<Option<Profile>> {
        let query = format!("SELECT * FROM profile WHERE user_id = '{}'", user_id);
        let results = self.execute_query(&query).await?;

        for item in &results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    if let Some(profile_json) = result.first() {
                        let profile = serde_json::from_value(profile_json.clone()).map_err(|e| {
                            crate::error::AppError::Database(format!(
                                "Failed to deserialize profile: {}",
                                e
                            ))
                        })?;
                        return Ok(Some(profile));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn update(&self, id: &str, profile: &Profile) -> Result<Option<Profile>> {
        let profile = profile.clone();

        let preferences_json =
            serde_json::to_string(&profile.preferences).unwrap_or_else(|_| "{}".to_string());

        let facts_json = serde_json::to_string(&profile.facts).unwrap_or_else(|_| "[]".to_string());

        let query = format!(
            "UPDATE profile SET name = {}, role = {}, organization = {}, location = {}, preferences = {}, communication_style = {}, technical_level = {}, language = {}, facts = {}, interests = {}, working_hours = {}, common_tasks = {}, tools_used = {}, updated_at = '{}', confidence = {}, version = {} WHERE id = '{}'",
            profile.name.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            profile.role.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            profile.organization.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            profile.location.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            preferences_json,
            profile.communication_style.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            profile.technical_level.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            profile.language.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            facts_json,
            serde_json::to_string(&profile.interests).unwrap_or_else(|_| "[]".to_string()),
            serde_json::to_string(&profile.working_hours).unwrap_or_else(|_| "null".to_string()),
            serde_json::to_string(&profile.common_tasks).unwrap_or_else(|_| "[]".to_string()),
            serde_json::to_string(&profile.tools_used).unwrap_or_else(|_| "[]".to_string()),
            profile.updated_at.to_rfc3339(),
            profile.confidence,
            profile.version,
            id,
        );

        self.execute_query(&query).await?;
        Ok(Some(profile))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let query = format!("DELETE FROM profile WHERE id = {}", id);
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

    async fn list(&self, limit: usize, start: usize) -> Result<Vec<Profile>> {
        let query = format!(
            "SELECT * FROM profile ORDER BY created_at DESC LIMIT {} START {}",
            limit, start
        );
        let results = self.execute_query(&query).await?;
        Ok(self.parse_results(&results))
    }

    async fn count(&self) -> Result<u64> {
        let query = "SELECT count() FROM profile GROUP ALL";
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

    async fn search(&self, query: &ProfileQuery) -> Result<Vec<Profile>> {
        let mut conditions = Vec::new();

        if let Some(user_id) = &query.user_id {
            conditions.push(format!("user_id = '{}'", user_id));
        }

        if let Some(min_confidence) = query.min_confidence {
            conditions.push(format!("confidence >= {}", min_confidence));
        }

        if !query.tools.is_empty() {
            for tool in &query.tools {
                conditions.push(format!("tools_used CONTAINS '{}'", tool));
            }
        }

        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", conditions.join(" AND "))
        };

        let limit = query.page_size;
        let start = (query.page - 1) * query.page_size;

        let sql = format!(
            "SELECT * FROM profile {} ORDER BY created_at DESC LIMIT {} START {}",
            where_clause, limit, start
        );

        let results = self.execute_query(&sql).await?;
        Ok(self.parse_results(&results))
    }

    async fn merge(&self, target_id: &str, source_id: &str, strategy: &str) -> Result<ProfileComparison> {
        // 获取两个画像
        let target = self.get_by_id(target_id).await?;
        let source = self.get_by_id(source_id).await?;

        match (target, source) {
            (Some(mut target), Some(source)) => {
                let mut comparison = ProfileComparison {
                    added_facts: Vec::new(),
                    conflicting_facts: Vec::new(),
                    consistent_values: Vec::new(),
                };

                // 合并事实
                for fact in source.facts {
                    let exists = target.facts.iter().any(|f| f.fact == fact.fact);
                    if !exists {
                        target.facts.push(fact.clone());
                        comparison.added_facts.push(fact);
                    }
                }

                // 合并工具
                for tool in source.tools_used {
                    if !target.tools_used.contains(&tool) {
                        target.tools_used.push(tool);
                    }
                }

                // 合并兴趣
                for interest in source.interests {
                    if !target.interests.contains(&interest) {
                        target.interests.push(interest);
                    }
                }

                // 更新目标画像
                self.update(target_id, &target).await?;

                Ok(comparison)
            }
            _ => Err(crate::error::AppError::NotFound(
                "Profile not found".to_string(),
            )),
        }
    }
}
