//! Entity 仓储
//!
//! 提供 Entity 和 Relationship 数据持久化服务

use async_trait::async_trait;
use std::marker::PhantomData;
use crate::error::Result;
use crate::models::entity::{Entity, Relationship, GraphQuery, GraphStats};
use crate::storage::surrealdb::SurrealPool;

/// Entity 仓储 trait
#[async_trait]
pub trait EntityRepository {
    /// 创建实体
    async fn create_entity(&self, entity: &Entity) -> Result<Entity>;

    /// 根据 ID 获取实体
    async fn get_entity_by_id(&self, id: &str) -> Result<Option<Entity>>;

    /// 更新实体
    async fn update_entity(&self, id: &str, entity: &Entity) -> Result<Option<Entity>>;

    /// 删除实体
    async fn delete_entity(&self, id: &str) -> Result<bool>;

    /// 列出实体
    async fn list_entities(&self, limit: usize, start: usize) -> Result<Vec<Entity>>;

    /// 搜索实体
    async fn search_entities(&self, name: &str, entity_type: Option<&str>) -> Result<Vec<Entity>>;

    /// 创建关系
    async fn create_relationship(&self, relationship: &Relationship) -> Result<Relationship>;

    /// 根据 ID 获取关系
    async fn get_relationship_by_id(&self, id: &str) -> Result<Option<Relationship>>;

    /// 删除关系
    async fn delete_relationship(&self, id: &str) -> Result<bool>;

    /// 获取实体的关系
    async fn get_entity_relationships(&self, entity_id: &str) -> Result<Vec<Relationship>>;

    /// 查询知识图谱
    async fn query_graph(&self, query: &GraphQuery) -> Result<(Vec<Entity>, Vec<Relationship>)>;

    /// 获取图统计
    async fn get_graph_stats(&self) -> Result<GraphStats>;

    /// 发现实体（根据名称）
    async fn discover_entity(&self, name: &str, entity_type: &str) -> Result<Option<Entity>>;
}

/// Entity 仓储实现
#[derive(Clone)]
pub struct EntityRepositoryImpl {
    pool: SurrealPool,
    _marker: PhantomData<Entity>,
}

impl EntityRepositoryImpl {
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

    /// 从查询结果解析实体
    fn parse_entity_results(&self, results: &[serde_json::Value]) -> Vec<Entity> {
        let mut entities = Vec::new();
        for item in results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    for entity_json in result {
                        match serde_json::from_value(entity_json.clone()) {
                            Ok(entity) => entities.push(entity),
                            Err(e) => tracing::warn!("Failed to deserialize entity: {}", e),
                        }
                    }
                }
            }
        }
        entities
    }

    /// 从查询结果解析关系
    fn parse_relationship_results(&self, results: &[serde_json::Value]) -> Vec<Relationship> {
        let mut relationships = Vec::new();
        for item in results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    for rel_json in result {
                        match serde_json::from_value(rel_json.clone()) {
                            Ok(rel) => relationships.push(rel),
                            Err(e) => tracing::warn!("Failed to deserialize relationship: {}", e),
                        }
                    }
                }
            }
        }
        relationships
    }
}

#[async_trait]
impl EntityRepository for EntityRepositoryImpl {
    async fn create_entity(&self, entity: &Entity) -> Result<Entity> {
        let entity = entity.clone();

        let properties_json =
            serde_json::to_string(&entity.properties).unwrap_or_else(|_| "{}".to_string());

        let aliases_json =
            serde_json::to_string(&entity.aliases).unwrap_or_else(|_| "[]".to_string());

        let source_memory_ids_json =
            serde_json::to_string(&entity.source_memory_ids).unwrap_or_else(|_| "[]".to_string());

        let query = format!(
            "CREATE entity SET id = '{}', tenant_id = '{}', name = '{}', entity_type = '{}', description = {}, properties = {}, aliases = {}, confidence = {}, source_memory_ids = {}, verified = {}, frequency = {}, created_at = '{}', updated_at = '{}', version = {}",
            entity.id,
            entity.tenant_id,
            entity.name.replace("'", "\\'"),
            entity.entity_type,
            entity.description.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            properties_json,
            aliases_json,
            entity.confidence,
            source_memory_ids_json,
            entity.verified,
            entity.frequency,
            entity.created_at.to_rfc3339(),
            entity.updated_at.to_rfc3339(),
            entity.version,
        );

        self.execute_query(&query).await?;
        Ok(entity)
    }

    async fn get_entity_by_id(&self, id: &str) -> Result<Option<Entity>> {
        let query = format!("SELECT * FROM entity WHERE id = {}", id);
        let results = self.execute_query(&query).await?;

        for item in &results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    if let Some(entity_json) = result.first() {
                        let entity = serde_json::from_value(entity_json.clone()).map_err(|e| {
                            crate::error::AppError::Database(format!(
                                "Failed to deserialize entity: {}",
                                e
                            ))
                        })?;
                        return Ok(Some(entity));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn update_entity(&self, id: &str, entity: &Entity) -> Result<Option<Entity>> {
        let entity = entity.clone();

        let properties_json =
            serde_json::to_string(&entity.properties).unwrap_or_else(|_| "{}".to_string());

        let aliases_json =
            serde_json::to_string(&entity.aliases).unwrap_or_else(|_| "[]".to_string());

        let query = format!(
            "UPDATE entity SET name = '{}', description = {}, properties = {}, aliases = {}, confidence = {}, verified = {}, frequency = {}, updated_at = '{}', version = {} WHERE id = '{}'",
            entity.name.replace("'", "\\'"),
            entity.description.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            properties_json,
            aliases_json,
            entity.confidence,
            entity.verified,
            entity.frequency,
            entity.updated_at.to_rfc3339(),
            entity.version,
            id,
        );

        self.execute_query(&query).await?;
        Ok(Some(entity))
    }

    async fn delete_entity(&self, id: &str) -> Result<bool> {
        // 先删除相关的关系
        let delete_rels_query = format!(
            "DELETE FROM relationship WHERE source_entity_id = '{}' OR target_entity_id = '{}'",
            id, id
        );
        self.execute_query(&delete_rels_query).await?;

        // 删除实体
        let query = format!("DELETE FROM entity WHERE id = {}", id);
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

    async fn list_entities(&self, limit: usize, start: usize) -> Result<Vec<Entity>> {
        let query = format!(
            "SELECT * FROM entity ORDER BY frequency DESC LIMIT {} START {}",
            limit, start
        );
        let results = self.execute_query(&query).await?;
        Ok(self.parse_entity_results(&results))
    }

    async fn search_entities(&self, name: &str, entity_type: Option<&str>) -> Result<Vec<Entity>> {
        let mut conditions = Vec::new();
        conditions.push(format!("name CONTAINS '{}' OR aliases CONTAINS '{}'", name, name));

        if let Some(etype) = entity_type {
            conditions.push(format!("entity_type = '{}'", etype));
        }

        let where_clause = format!("WHERE {}", conditions.join(" AND "));

        let query = format!(
            "SELECT * FROM entity {} ORDER BY frequency DESC LIMIT 20",
            where_clause
        );

        let results = self.execute_query(&query).await?;
        Ok(self.parse_entity_results(&results))
    }

    async fn create_relationship(&self, relationship: &Relationship) -> Result<Relationship> {
        let relationship = relationship.clone();

        let query = format!(
            "CREATE relationship SET id = '{}', tenant_id = '{}', source_entity_id = '{}', target_entity_id = '{}', relationship_type = '{}', strength = {}, context = {}, source_memory_id = '{}', created_at = '{}', updated_at = '{}', verified = {}, confidence = {}, version = {}",
            relationship.id,
            relationship.tenant_id,
            relationship.source_entity_id,
            relationship.target_entity_id,
            relationship.relationship_type,
            relationship.strength,
            relationship.context.as_ref().map(|s| format!("'{}'", s.replace("'", "\\'"))).unwrap_or_else(|| "NONE".to_string()),
            relationship.source_memory_id,
            relationship.created_at.to_rfc3339(),
            relationship.updated_at.to_rfc3339(),
            relationship.verified,
            relationship.confidence,
            relationship.version,
        );

        self.execute_query(&query).await?;
        Ok(relationship)
    }

    async fn get_relationship_by_id(&self, id: &str) -> Result<Option<Relationship>> {
        let query = format!("SELECT * FROM relationship WHERE id = {}", id);
        let results = self.execute_query(&query).await?;

        for item in &results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    if let Some(rel_json) = result.first() {
                        let rel = serde_json::from_value(rel_json.clone()).map_err(|e| {
                            crate::error::AppError::Database(format!(
                                "Failed to deserialize relationship: {}",
                                e
                            ))
                        })?;
                        return Ok(Some(rel));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn delete_relationship(&self, id: &str) -> Result<bool> {
        let query = format!("DELETE FROM relationship WHERE id = {}", id);
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

    async fn get_entity_relationships(&self, entity_id: &str) -> Result<Vec<Relationship>> {
        let query = format!(
            "SELECT * FROM relationship WHERE source_entity_id = '{}' OR target_entity_id = '{}' ORDER BY strength DESC",
            entity_id, entity_id
        );
        let results = self.execute_query(&query).await?;
        Ok(self.parse_relationship_results(&results))
    }

    async fn query_graph(&self, query: &GraphQuery) -> Result<(Vec<Entity>, Vec<Relationship>)> {
        // 获取中心实体
        let center_query = format!(
            "SELECT * FROM entity WHERE id = '{}'",
            query.center_entity_id
        );
        let center_results = self.execute_query(&center_query).await?;
        let mut entities = self.parse_entity_results(&center_results);

        if query.include_center && !entities.is_empty() {
            // 中心实体已包含在 entities 中
        }

        // 获取直接关系
        let rel_query = format!(
            "SELECT * FROM relationship WHERE (source_entity_id = '{}' OR target_entity_id = '{}') AND strength >= {} ORDER BY strength DESC LIMIT {}",
            query.center_entity_id,
            query.center_entity_id,
            query.min_strength,
            query.limit_per_depth
        );
        let rel_results = self.execute_query(&rel_query).await?;
        let relationships = self.parse_relationship_results(&rel_results);

        // 获取相关的实体
        let mut related_entity_ids = Vec::new();
        for rel in &relationships {
            if rel.source_entity_id != query.center_entity_id {
                related_entity_ids.push(rel.source_entity_id.clone());
            }
            if rel.target_entity_id != query.center_entity_id {
                related_entity_ids.push(rel.target_entity_id.clone());
            }
        }

        // 去重
        related_entity_ids.sort();
        related_entity_ids.dedup();

        // 获取相关实体
        if !related_entity_ids.is_empty() && query.max_depth > 1 {
            let ids_str = related_entity_ids.join("','");
            let entity_query = format!(
                "SELECT * FROM entity WHERE id IN ['{}'] LIMIT {}",
                ids_str,
                query.limit_per_depth
            );
            let entity_results = self.execute_query(&entity_query).await?;
            let mut more_entities = self.parse_entity_results(&entity_results);
            entities.extend(more_entities);
        }

        Ok((entities, relationships))
    }

    async fn get_graph_stats(&self) -> Result<GraphStats> {
        let entity_count_query = "SELECT count() FROM entity GROUP ALL";
        let rel_count_query = "SELECT count() FROM relationship GROUP ALL";

        let entity_results = self.execute_query(&entity_count_query).await?;
        let rel_results = self.execute_query(&rel_count_query).await?;

        let total_entities = match entity_results.first() {
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

        let total_relationships = match rel_results.first() {
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

        Ok(GraphStats {
            total_entities,
            person_count: 0,
            organization_count: 0,
            project_count: 0,
            tool_count: 0,
            concept_count: 0,
            total_relationships,
            knows_count: 0,
            works_on_count: 0,
            uses_count: 0,
            depends_on_count: 0,
            similar_to_count: 0,
            connected_components: 0,
            largest_component_size: 0,
            density: if total_entities > 1 {
                (2.0 * total_relationships as f32) / (total_entities * (total_entities - 1)) as f32
            } else {
                0.0
            },
        })
    }

    async fn discover_entity(&self, name: &str, entity_type: &str) -> Result<Option<Entity>> {
        let query = format!(
            "SELECT * FROM entity WHERE name = '{}' AND entity_type = '{}' LIMIT 1",
            name.replace("'", "\\'"),
            entity_type
        );
        let results = self.execute_query(&query).await?;

        for item in &results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    if let Some(entity_json) = result.first() {
                        let entity = serde_json::from_value(entity_json.clone()).map_err(|e| {
                            crate::error::AppError::Database(format!(
                                "Failed to deserialize entity: {}",
                                e
                            ))
                        })?;
                        return Ok(Some(entity));
                    }
                }
            }
        }

        Ok(None)
    }
}
