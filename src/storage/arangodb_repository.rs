//! ArangoDB 仓储层
//!
//! 使用 AQL 查询实现数据访问。

use async_trait::async_trait;
use std::marker::PhantomData;

use crate::error::{AppError, Result};
use crate::models::index_record::IndexRecord;
use crate::models::session::Session;
use crate::models::turn::Turn;
use crate::storage::arangodb::ArangoStorage;

/// 仓储 trait
#[async_trait]
pub trait ArangoRepository<T: Clone + Send + Sync> {
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
}

/// 会话仓储实现
#[derive(Clone)]
pub struct ArangoSessionRepository {
    storage: ArangoStorage,
    _marker: PhantomData<Session>,
}

impl ArangoSessionRepository {
    pub fn new(storage: ArangoStorage) -> Self {
        Self {
            storage,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl ArangoRepository<Session> for ArangoSessionRepository {
    async fn create(&self, session: &Session) -> Result<Session> {
        let doc = serde_json::json!({
            "_key": session.id,
            "tenant_id": session.tenant_id,
            "name": session.name,
            "description": session.description,
            "created_at": session.created_at.to_rfc3339(),
            "last_active_at": session.last_active_at.to_rfc3339(),
            "status": session.status,
            "metadata": session.metadata,
            "config": session.config,
            "stats": session.stats
        });

        self.storage
            .insert("sessions", &doc)
            .await
            .map_err(AppError::Database)?;

        Ok(session.clone())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Session>> {
        self.storage
            .get::<Session>("sessions", id)
            .await
            .map_err(AppError::Database)
    }

    async fn update(&self, id: &str, session: &Session) -> Result<Option<Session>> {
        let doc = serde_json::json!({
            "tenant_id": session.tenant_id,
            "name": session.name,
            "description": session.description,
            "last_active_at": session.last_active_at.to_rfc3339(),
            "status": session.status,
            "metadata": session.metadata,
            "config": session.config,
            "stats": session.stats
        });

        self.storage
            .update("sessions", id, &doc)
            .await
            .map_err(AppError::Database)?;

        Ok(Some(session.clone()))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        self.storage
            .delete("sessions", id)
            .await
            .map_err(AppError::Database)?;
        Ok(true)
    }

    async fn list(&self, limit: usize, start: usize) -> Result<Vec<Session>> {
        let query = format!(
            "FOR s IN sessions SORT s.created_at DESC LIMIT {}, {} RETURN s",
            start, limit
        );
        self.storage
            .aql::<Session>(&query)
            .await
            .map_err(AppError::Database)
    }

    async fn count(&self) -> Result<u64> {
        self.storage
            .count("sessions")
            .await
            .map_err(AppError::Database)
    }
}

/// 轮次仓储实现
#[derive(Clone)]
pub struct ArangoTurnRepository {
    storage: ArangoStorage,
    _marker: PhantomData<Turn>,
}

impl ArangoTurnRepository {
    pub fn new(storage: ArangoStorage) -> Self {
        Self {
            storage,
            _marker: PhantomData,
        }
    }

    /// 获取指定会话的最大 turn_number
    async fn get_max_turn_number(&self, session_id: &str) -> Result<u64> {
        let query = format!(
            "FOR t IN turns FILTER t.session_id == '{}' SORT t.turn_number DESC LIMIT 1 RETURN t.turn_number",
            session_id
        );
        let result = self
            .storage
            .aql::<serde_json::Value>(&query)
            .await
            .map_err(AppError::Database)?;
        if let Some(turn_number) = result.first().and_then(|v| v.as_u64()) {
            Ok(turn_number)
        } else {
            Ok(0)
        }
    }
}

#[async_trait]
impl ArangoRepository<Turn> for ArangoTurnRepository {
    async fn create(&self, turn: &Turn) -> Result<Turn> {
        let doc = serde_json::json!({
            "_key": turn.id,
            "session_id": turn.session_id,
            "turn_number": turn.turn_number,
            "raw_content": turn.raw_content,
            "content": turn.raw_content,
            "metadata": {
                "timestamp": turn.metadata.timestamp.to_rfc3339(),
                "user_id": turn.metadata.user_id,
                "message_type": turn.metadata.message_type,
                "role": turn.metadata.role,
                "model": turn.metadata.model,
                "token_count": turn.metadata.token_count,
                "custom": turn.metadata.custom
            },
            "dehydrated": turn.dehydrated,
            "status": turn.status,
            "parent_id": turn.parent_id,
            "children_ids": turn.children_ids
        });

        self.storage
            .insert("turns", &doc)
            .await
            .map_err(AppError::Database)?;

        Ok(turn.clone())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Turn>> {
        self.storage
            .get::<Turn>("turns", id)
            .await
            .map_err(AppError::Database)
    }

    async fn update(&self, id: &str, turn: &Turn) -> Result<Option<Turn>> {
        let doc = serde_json::json!({
            "role": turn.metadata.role,
            "content": turn.raw_content,
            "token_count": turn.metadata.token_count
        });

        self.storage
            .update("turns", id, &doc)
            .await
            .map_err(AppError::Database)?;

        Ok(Some(turn.clone()))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        self.storage
            .delete("turns", id)
            .await
            .map_err(AppError::Database)?;
        Ok(true)
    }

    async fn list(&self, limit: usize, start: usize) -> Result<Vec<Turn>> {
        let query = format!(
            "FOR t IN turns SORT t.created_at DESC LIMIT {}, {} RETURN t",
            start, limit
        );
        self.storage
            .aql::<Turn>(&query)
            .await
            .map_err(AppError::Database)
    }

    async fn count(&self) -> Result<u64> {
        self.storage
            .count("turns")
            .await
            .map_err(AppError::Database)
    }
}

/// 索引记录仓储实现
#[derive(Clone)]
pub struct ArangoIndexRecordRepository {
    storage: ArangoStorage,
    _marker: PhantomData<IndexRecord>,
}

impl ArangoIndexRecordRepository {
    pub fn new(storage: ArangoStorage) -> Self {
        Self {
            storage,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl ArangoRepository<IndexRecord> for ArangoIndexRecordRepository {
    async fn create(&self, record: &IndexRecord) -> Result<IndexRecord> {
        let doc = serde_json::json!({
            "_key": record.turn_id,
            "turn_id": record.turn_id,
            "session_id": record.session_id,
            "tenant_id": record.tenant_id,
            "gist": record.gist,
            "topics": record.topics,
            "tags": record.tags,
            "timestamp": record.timestamp.to_rfc3339(),
            "vector_id": record.vector_id,
            "turn_number": record.turn_number
        });

        self.storage
            .insert("index_records", &doc)
            .await
            .map_err(AppError::Database)?;

        Ok(record.clone())
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<IndexRecord>> {
        self.storage
            .get::<IndexRecord>("index_records", id)
            .await
            .map_err(AppError::Database)
    }

    async fn update(&self, id: &str, record: &IndexRecord) -> Result<Option<IndexRecord>> {
        let doc = serde_json::json!({
            "gist": record.gist,
            "topics": record.topics,
            "tags": record.tags
        });

        self.storage
            .update("index_records", id, &doc)
            .await
            .map_err(AppError::Database)?;

        Ok(Some(record.clone()))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        self.storage
            .delete("index_records", id)
            .await
            .map_err(AppError::Database)?;
        Ok(true)
    }

    async fn list(&self, limit: usize, start: usize) -> Result<Vec<IndexRecord>> {
        let query = format!(
            "FOR r IN index_records SORT r.timestamp DESC LIMIT {}, {} RETURN r",
            start, limit
        );
        self.storage
            .aql::<IndexRecord>(&query)
            .await
            .map_err(AppError::Database)
    }

    async fn count(&self) -> Result<u64> {
        self.storage
            .count("index_records")
            .await
            .map_err(AppError::Database)
    }
}
