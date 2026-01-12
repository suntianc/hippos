use async_trait::async_trait;
use std::collections::HashMap;
use std::marker::PhantomData;
use surrealdb::{Surreal, engine::any::Any};

use crate::error::Result;
use crate::models::index_record::IndexRecord;
use crate::models::session::{Session, SessionConfig, SessionStats};
use crate::models::turn::Turn;
use crate::storage::surrealdb::SurrealPool;

/// 仓储 trait
#[async_trait]
pub trait Repository<T: Clone + Send + Sync> {
    /// 创建实体
    async fn create(&self, entity: &T) -> Result<T>;

    /// 根据 ID 获取实体
    async fn get_by_id(&self, id: &str) -> Result<Option<T>>;

    /// 更新实体
    async fn update(&self, id: &str, entity: &T) -> Result<Option<T>>;

    /// 删除实体
    async fn delete(&self, id: &str) -> Result<bool>;

    /// 列出所有实体
    async fn list(&self, limit: usize, start: usize) -> Result<Vec<T>>;

    /// 统计数量
    async fn count(&self) -> Result<u64>;

    // === 租户过滤方法 ===

    async fn list_by_tenant(&self, _tenant_id: &str, limit: usize, start: usize) -> Result<Vec<T>> {
        self.list(limit, start).await
    }

    async fn count_by_tenant(&self, _tenant_id: &str) -> Result<u64> {
        self.count().await
    }

    // === 会话过滤方法（用于 Turn） ===

    async fn list_by_session(
        &self,
        _session_id: &str,
        _limit: usize,
        _start: usize,
    ) -> Result<Vec<T>> {
        Ok(vec![])
    }

    async fn count_by_session(&self, _session_id: &str) -> Result<u64> {
        Ok(0)
    }
}

/// 会话仓储实现
#[derive(Clone)]
pub struct SessionRepository {
    pool: SurrealPool,
    _marker: PhantomData<Session>,
}

impl SessionRepository {
    pub fn new(pool: SurrealPool) -> Self {
        Self {
            pool,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl Repository<Session> for SessionRepository {
    async fn create(&self, session: &Session) -> Result<Session> {
        let session = session.clone();

        // Use HTTP API to create the session (bypasses SDK serialization issues)
        let metadata_str = if session.metadata.is_empty() {
            "{}".to_string()
        } else {
            serde_json::to_string(&session.metadata).unwrap_or_else(|_| "{}".to_string())
        };

        let description_str = match &session.description {
            Some(d) => format!("'{}'", d.replace("'", "\\'")),
            None => "NONE".to_string(),
        };

        let query = format!(
            "CREATE session SET tenant_id = '{}', name = '{}', description = {}, created_at = '{}', last_active_at = '{}', status = '{}', metadata = {}",
            session.tenant_id,
            session.name,
            description_str,
            session.created_at.to_rfc3339(),
            session.last_active_at.to_rfc3339(),
            session.status,
            metadata_str,
        );

        // Execute via HTTP to avoid SDK serialization issues
        let config = self.pool.config();
        let url = format!(
            "{}/sql",
            config.url.replace("ws://", "http://").replace("/rpc", "")
        );

        tracing::debug!(
            "Sending HTTP request to SurrealDB: url={}, query={}",
            url,
            query
        );

        let response = self
            .pool
            .http_client()
            .post(&url)
            .header("surreal-ns", &config.namespace)
            .header("surreal-db", &config.database)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(&config.username, Some(&config.password))
            .body(query.clone())
            .send()
            .await
            .map_err(|e| crate::error::AppError::Database(format!("HTTP request failed: {}", e)))?;

        tracing::debug!("SurrealDB response status: {}", response.status());

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::error::AppError::Database(format!(
                "SurrealDB error: {}",
                error_text
            )));
        }

        // Return the session
        Ok(session)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Session>> {
        let query = format!("SELECT * FROM session WHERE id = {}", id);

        // Use HTTP API to avoid SDK serialization issues
        let config = self.pool.config();
        let url = format!(
            "{}/sql",
            config.url.replace("ws://", "http://").replace("/rpc", "")
        );

        tracing::debug!(
            "Sending HTTP request to SurrealDB: url={}, query={}",
            url,
            query
        );

        let response = self
            .pool
            .http_client()
            .post(&url)
            .header("surreal-ns", &config.namespace)
            .header("surreal-db", &config.database)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(&config.username, Some(&config.password))
            .body(query.clone())
            .send()
            .await
            .map_err(|e| crate::error::AppError::Database(format!("HTTP request failed: {}", e)))?;

        tracing::debug!("SurrealDB response status: {}", response.status());

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

        for item in results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    if let Some(session_json) = result.first() {
                        let session =
                            serde_json::from_value(session_json.clone()).map_err(|e| {
                                crate::error::AppError::Database(format!(
                                    "Failed to deserialize session: {}",
                                    e
                                ))
                            })?;
                        return Ok(Some(session));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn update(&self, id: &str, session: &Session) -> Result<Option<Session>> {
        let session = session.clone();
        let query = format!(
            "UPDATE session SET tenant_id = '{}', name = '{}', description = '{}', last_active_at = '{}', status = '{}' WHERE id = {}",
            session.tenant_id,
            session.name,
            session.description.clone().unwrap_or_default(),
            session.last_active_at.to_rfc3339(),
            session.status,
            id,
        );

        // Use HTTP API to avoid SDK serialization issues
        let config = self.pool.config();
        let url = format!(
            "{}/sql",
            config.url.replace("ws://", "http://").replace("/rpc", "")
        );

        tracing::debug!(
            "Sending HTTP request to SurrealDB: url={}, query={}",
            url,
            query
        );

        let response = self
            .pool
            .http_client()
            .post(&url)
            .header("surreal-ns", &config.namespace)
            .header("surreal-db", &config.database)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(&config.username, Some(&config.password))
            .body(query.clone())
            .send()
            .await
            .map_err(|e| crate::error::AppError::Database(format!("HTTP request failed: {}", e)))?;

        tracing::debug!("SurrealDB response status: {}", response.status());

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::error::AppError::Database(format!(
                "SurrealDB error: {}",
                error_text
            )));
        }

        Ok(Some(session))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let query = format!("DELETE FROM session WHERE id = {}", id);

        // Use HTTP API to avoid SDK serialization issues
        let config = self.pool.config();
        let url = format!(
            "{}/sql",
            config.url.replace("ws://", "http://").replace("/rpc", "")
        );

        tracing::debug!(
            "Sending HTTP request to SurrealDB: url={}, query={}",
            url,
            query
        );

        let response = self
            .pool
            .http_client()
            .post(&url)
            .header("surreal-ns", &config.namespace)
            .header("surreal-db", &config.database)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(&config.username, Some(&config.password))
            .body(query.clone())
            .send()
            .await
            .map_err(|e| crate::error::AppError::Database(format!("HTTP request failed: {}", e)))?;

        tracing::debug!("SurrealDB response status: {}", response.status());

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

        for item in results {
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    return Ok(result.len() > 0);
                }
            }
        }

        Ok(false)
    }

    async fn list(&self, limit: usize, start: usize) -> Result<Vec<Session>> {
        let query = format!(
            "SELECT * FROM session ORDER BY created_at DESC LIMIT {} START {}",
            limit, start
        );

        // Use HTTP API to avoid SDK serialization issues
        let config = self.pool.config();
        let url = format!(
            "{}/sql",
            config.url.replace("ws://", "http://").replace("/rpc", "")
        );

        tracing::debug!(
            "Sending HTTP request to SurrealDB: url={}, query={}",
            url,
            query
        );

        let response = self
            .pool
            .http_client()
            .post(&url)
            .header("surreal-ns", &config.namespace)
            .header("surreal-db", &config.database)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(&config.username, Some(&config.password))
            .body(query.clone())
            .send()
            .await
            .map_err(|e| crate::error::AppError::Database(format!("HTTP request failed: {}", e)))?;

        tracing::debug!("SurrealDB response status: {}", response.status());

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(crate::error::AppError::Database(format!(
                "SurrealDB error: {}",
                error_text
            )));
        }

        let response_text = response.text().await.unwrap_or_default();
        tracing::debug!("SurrealDB response text: {}", response_text);

        let results: Vec<serde_json::Value> =
            serde_json::from_str(&response_text).map_err(|e| {
                crate::error::AppError::Database(format!("Failed to parse response: {}", e))
            })?;

        tracing::debug!("Parsed results count: {}", results.len());

        let mut sessions = Vec::new();
        for item in &results {
            tracing::debug!("Item: {:?}", item);
            if let Some(json) = item.as_object() {
                if let Some(result) = json.get("result").and_then(|r| r.as_array()) {
                    tracing::debug!("Result array length: {}", result.len());
                    for session_json in result {
                        tracing::debug!("Session JSON: {:?}", session_json);
                        match serde_json::from_value(session_json.clone()) {
                            Ok(session) => sessions.push(session),
                            Err(e) => tracing::warn!("Failed to deserialize session: {}", e),
                        }
                    }
                }
            }
        }

        tracing::debug!("Total sessions deserialized: {}", sessions.len());

        Ok(sessions)
    }

    async fn count(&self) -> Result<u64> {
        let query = "SELECT count() FROM session GROUP ALL";

        // Use HTTP API to avoid SDK serialization issues
        let config = self.pool.config();
        let url = format!(
            "{}/sql",
            config.url.replace("ws://", "http://").replace("/rpc", "")
        );

        tracing::debug!(
            "Sending HTTP request to SurrealDB: url={}, query={}",
            url,
            query
        );

        let response = self
            .pool
            .http_client()
            .post(&url)
            .header("surreal-ns", &config.namespace)
            .header("surreal-db", &config.database)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(&config.username, Some(&config.password))
            .body(query.clone())
            .send()
            .await
            .map_err(|e| crate::error::AppError::Database(format!("HTTP request failed: {}", e)))?;

        tracing::debug!("SurrealDB response status: {}", response.status());

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

        for item in results {
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

    async fn count_by_tenant(&self, tenant_id: &str) -> Result<u64> {
        let query = format!(
            "SELECT count() FROM session WHERE tenant_id = '{}' GROUP ALL",
            tenant_id
        );

        // Use HTTP API to avoid SDK serialization issues
        let config = self.pool.config();
        let url = format!(
            "{}/sql",
            config.url.replace("ws://", "http://").replace("/rpc", "")
        );

        tracing::debug!(
            "Sending HTTP request to SurrealDB: url={}, query={}",
            url,
            query
        );

        let response = self
            .pool
            .http_client()
            .post(&url)
            .header("surreal-ns", &config.namespace)
            .header("surreal-db", &config.database)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(&config.username, Some(&config.password))
            .body(query.clone())
            .send()
            .await
            .map_err(|e| crate::error::AppError::Database(format!("HTTP request failed: {}", e)))?;

        tracing::debug!("SurrealDB response status: {}", response.status());

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

        for item in results {
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

/// 轮次仓储实现
#[derive(Clone)]
pub struct TurnRepository {
    db: Surreal<Any>,
    pool: SurrealPool,
    _marker: PhantomData<Turn>,
}

impl TurnRepository {
    pub fn new(db: Surreal<Any>, pool: SurrealPool) -> Self {
        Self {
            db,
            pool,
            _marker: PhantomData,
        }
    }

    /// 获取指定会话的最大 turn_number
    pub async fn get_max_turn_number(&self, session_id: &str) -> Result<u64> {
        let query = format!(
            "SELECT turn_number FROM turn WHERE session_id = '{}' ORDER BY turn_number DESC LIMIT 1",
            session_id
        );
        let mut response = self.db.query(query).await?;
        let results: Vec<serde_json::Value> = response.take(0)?;

        if let Some(json) = results.first() {
            if let Some(turn_number) = json.get("turn_number").and_then(|v| v.as_u64()) {
                return Ok(turn_number);
            }
        }

        Ok(0)
    }

    /// 在事务中创建 turn 并返回分配的 turn_number
    pub async fn create_with_turn_number(&self, session_id: &str, turn: &Turn) -> Result<Turn> {
        let max_turn = self.get_max_turn_number(session_id).await?;
        let turn_number = max_turn + 1;

        let mut turn_with_number = turn.clone();
        turn_with_number.turn_number = turn_number;

        // Create the turn
        let created = self.create(&turn_with_number).await?;
        Ok(created)
    }
}

#[async_trait]
impl Repository<Turn> for TurnRepository {
    async fn create(&self, turn: &Turn) -> Result<Turn> {
        let turn = turn.clone();

        // Use raw SQL to create the turn
        let metadata_json =
            serde_json::to_string(&turn.metadata).unwrap_or_else(|_| "{}".to_string());

        let query = format!(
            "CREATE turn SET id = '{}', session_id = '{}', turn_number = {}, raw_content = '{}', metadata = {}",
            turn.id,
            turn.session_id,
            turn.turn_number,
            turn.raw_content.replace("'", "\\'"),
            metadata_json,
        );

        let _ = self.db.query(query).await?;

        // Return the input turn (with ID we provided)
        Ok(turn)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Turn>> {
        let query = format!("SELECT * FROM turn WHERE id = {}", id);
        let mut response = self.db.query(query).await?;
        let results: Vec<serde_json::Value> = response.take(0)?;

        if let Some(json) = results.first() {
            let turn = serde_json::from_value(json.clone()).map_err(|e| {
                crate::error::AppError::Database(format!("Failed to deserialize turn: {}", e))
            })?;
            return Ok(Some(turn));
        }

        Ok(None)
    }

    async fn update(&self, id: &str, turn: &Turn) -> Result<Option<Turn>> {
        let turn = turn.clone();
        let metadata_json =
            serde_json::to_string(&turn.metadata).unwrap_or_else(|_| "{}".to_string());

        let query = format!(
            "UPDATE turn SET raw_content = '{}', metadata = {} WHERE id = {}",
            turn.raw_content.replace("'", "\\'"),
            metadata_json,
            id,
        );

        // Use HTTP API to avoid SDK serialization issues
        let config = self.pool.config();
        let url = format!(
            "{}/sql",
            config.url.replace("ws://", "http://").replace("/rpc", "")
        );

        let response = self
            .pool
            .http_client()
            .post(&url)
            .header("surreal-ns", &config.namespace)
            .header("surreal-db", &config.database)
            .header("Accept", "application/json")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .basic_auth(&config.username, Some(&config.password))
            .body(query)
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

        Ok(Some(turn))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let query = format!("DELETE FROM turn WHERE id = {}", id);
        let mut response = self.db.query(query).await?;
        let results: Vec<serde_json::Value> = response.take(0)?;

        Ok(results.len() > 0)
    }

    async fn list(&self, limit: usize, start: usize) -> Result<Vec<Turn>> {
        let query = format!(
            "SELECT * FROM turn ORDER BY created_at DESC LIMIT {} START {}",
            limit, start
        );
        let mut response = self.db.query(query).await?;
        let results: Vec<serde_json::Value> = response.take(0)?;

        let mut turns = Vec::new();
        for json in results {
            match serde_json::from_value(json) {
                Ok(turn) => turns.push(turn),
                Err(e) => tracing::warn!("Failed to deserialize turn: {}", e),
            }
        }

        Ok(turns)
    }

    async fn count(&self) -> Result<u64> {
        let query = "SELECT count() FROM turn GROUP ALL";
        let mut response = self.db.query(query).await?;
        let results: Vec<serde_json::Value> = response.take(0)?;

        if let Some(json) = results.first() {
            if let Some(count) = json.get("count").and_then(|v| v.as_u64()) {
                return Ok(count);
            }
        }

        Ok(0)
    }

    async fn list_by_session(
        &self,
        session_id: &str,
        limit: usize,
        start: usize,
    ) -> Result<Vec<Turn>> {
        let query = format!(
            "SELECT * FROM turn WHERE session_id = '{}' ORDER BY turn_number ASC LIMIT {} START {}",
            session_id, limit, start
        );
        let mut response = self.db.query(query).await?;
        let results: Vec<serde_json::Value> = response.take(0)?;

        let mut turns = Vec::new();
        for json in results {
            match serde_json::from_value(json) {
                Ok(turn) => turns.push(turn),
                Err(e) => tracing::warn!("Failed to deserialize turn: {}", e),
            }
        }

        Ok(turns)
    }

    async fn count_by_session(&self, session_id: &str) -> Result<u64> {
        let query = format!(
            "SELECT count() FROM turn WHERE session_id = '{}' GROUP ALL",
            session_id
        );
        let mut response = self.db.query(query).await?;
        let results: Vec<serde_json::Value> = response.take(0)?;

        if let Some(json) = results.first() {
            if let Some(count) = json.get("count").and_then(|v| v.as_u64()) {
                return Ok(count);
            }
        }

        Ok(0)
    }
}

/// 索引记录仓储实现
#[derive(Clone)]
pub struct IndexRecordRepository {
    db: Surreal<Any>,
    _marker: PhantomData<IndexRecord>,
}

impl IndexRecordRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self {
            db,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl Repository<IndexRecord> for IndexRecordRepository {
    async fn create(&self, record: &IndexRecord) -> Result<IndexRecord> {
        let record = record.clone();

        let topics_str = record.topics.join(",");
        let tags_str = record.tags.join(",");

        let query = format!(
            "CREATE index_record SET id = '{}', turn_id = '{}', session_id = '{}', tenant_id = '{}', gist = '{}', topics = '{}', tags = '{}', timestamp = '{}', vector_id = '{}', turn_number = {}",
            record.turn_id,
            record.turn_id,
            record.session_id,
            record.tenant_id,
            record.gist.replace("'", "\\'"),
            topics_str,
            tags_str,
            record.timestamp.to_rfc3339(),
            record.vector_id,
            record.turn_number,
        );

        let _ = self.db.query(query).await?;

        Ok(record)
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<IndexRecord>> {
        let query = format!("SELECT * FROM index_record WHERE id = {}", id);
        let mut response = self.db.query(query).await?;
        let results: Vec<serde_json::Value> = response.take(0)?;

        if let Some(json) = results.first() {
            let record = serde_json::from_value(json.clone()).map_err(|e| {
                crate::error::AppError::Database(format!(
                    "Failed to deserialize index record: {}",
                    e
                ))
            })?;
            return Ok(Some(record));
        }

        Ok(None)
    }

    async fn update(&self, id: &str, record: &IndexRecord) -> Result<Option<IndexRecord>> {
        let record = record.clone();
        let query = format!(
            "UPDATE index_record SET gist = '{}' WHERE id = '{}'",
            record.gist.replace("'", "\\'"),
            id,
        );

        let _ = self.db.query(query).await?;

        Ok(Some(record))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let query = format!("DELETE FROM index_record WHERE id = {}", id);
        let mut response = self.db.query(query).await?;
        let results: Vec<serde_json::Value> = response.take(0)?;

        Ok(results.len() > 0)
    }

    async fn list(&self, limit: usize, start: usize) -> Result<Vec<IndexRecord>> {
        let query = format!(
            "SELECT * FROM index_record ORDER BY timestamp DESC LIMIT {} START {}",
            limit, start
        );
        let mut response = self.db.query(query).await?;
        let results: Vec<serde_json::Value> = response.take(0)?;

        let mut records = Vec::new();
        for json in results {
            match serde_json::from_value(json) {
                Ok(record) => records.push(record),
                Err(e) => tracing::warn!("Failed to deserialize index record: {}", e),
            }
        }

        Ok(records)
    }

    async fn count(&self) -> Result<u64> {
        let query = "SELECT count() FROM index_record GROUP ALL";
        let mut response = self.db.query(query).await?;
        let results: Vec<serde_json::Value> = response.take(0)?;

        if let Some(json) = results.first() {
            if let Some(count) = json.get("count").and_then(|v| v.as_u64()) {
                return Ok(count);
            }
        }

        Ok(0)
    }

    async fn list_by_tenant(
        &self,
        tenant_id: &str,
        limit: usize,
        start: usize,
    ) -> Result<Vec<IndexRecord>> {
        let query = format!(
            "SELECT * FROM index_record WHERE tenant_id = '{}' ORDER BY timestamp DESC LIMIT {} START {}",
            tenant_id, limit, start
        );
        let mut response = self.db.query(query).await?;
        let results: Vec<serde_json::Value> = response.take(0)?;

        let mut records = Vec::new();
        for json in results {
            match serde_json::from_value(json) {
                Ok(record) => records.push(record),
                Err(e) => tracing::warn!("Failed to deserialize index record: {}", e),
            }
        }

        Ok(records)
    }

    async fn count_by_tenant(&self, tenant_id: &str) -> Result<u64> {
        let query = format!(
            "SELECT count() FROM index_record WHERE tenant_id = '{}' GROUP ALL",
            tenant_id
        );
        let mut response = self.db.query(query).await?;
        let results: Vec<serde_json::Value> = response.take(0)?;

        if let Some(json) = results.first() {
            if let Some(count) = json.get("count").and_then(|v| v.as_u64()) {
                return Ok(count);
            }
        }

        Ok(0)
    }
}
