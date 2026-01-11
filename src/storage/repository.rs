use async_trait::async_trait;
use std::marker::PhantomData;
use surrealdb::{Surreal, engine::any::Any};

use crate::error::Result;
use crate::models::index_record::IndexRecord;
use crate::models::session::Session;
use crate::models::turn::Turn;

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

    /// 按会话列出实体
    async fn list_by_session(
        &self,
        _session_id: &str,
        _limit: usize,
        _start: usize,
    ) -> Result<Vec<T>> {
        // 默认实现：子类可覆盖
        Ok(vec![])
    }

    /// 按会话统计数量
    async fn count_by_session(&self, _session_id: &str) -> Result<u64> {
        Ok(0)
    }
}

/// 会话仓储实现
#[derive(Clone)]
pub struct SessionRepository {
    db: Surreal<Any>,
    _marker: PhantomData<Session>,
}

impl SessionRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self {
            db,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl Repository<Session> for SessionRepository {
    async fn create(&self, session: &Session) -> Result<Session> {
        let created: Option<Session> = self
            .db
            .create(("session", &session.id))
            .content(session)
            .await?;

        created.ok_or_else(|| {
            crate::error::AppError::Database(format!("Failed to create session: {}", session.id))
        })
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Session>> {
        let result: Option<Session> = self.db.select(("session", id)).await?;
        Ok(result)
    }

    async fn update(&self, id: &str, session: &Session) -> Result<Option<Session>> {
        let updated: Option<Session> = self.db.update(("session", id)).content(session).await?;
        Ok(updated)
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let result: Option<Session> = self.db.delete(("session", id)).await?;
        Ok(result.is_some())
    }

    async fn list(&self, limit: usize, start: usize) -> Result<Vec<Session>> {
        // Use LIMIT/OFFSET for efficient pagination
        let query = format!(
            "SELECT * FROM session ORDER BY created_at DESC LIMIT {} START {}",
            limit, start
        );
        let result: Vec<Session> = self.db.query(query).await?.take(0)?;
        Ok(result)
    }

    async fn count(&self) -> Result<u64> {
        // Use count() aggregation for efficient counting
        let result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM session GROUP ALL")
            .await?
            .take(0)?;
        if let Some(count_val) = result.first().and_then(|v| v.get("count")) {
            Ok(count_val.as_u64().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    // === 租户过滤实现 ===

    async fn list_by_tenant(
        &self,
        tenant_id: &str,
        limit: usize,
        start: usize,
    ) -> Result<Vec<Session>> {
        // Use parameterized query to prevent SQL injection
        let query = "
            SELECT * FROM session 
            WHERE tenant_id = $tenant_id 
            ORDER BY created_at DESC 
            LIMIT $limit START $start
        ";
        let result: Vec<Session> = self
            .db
            .query(query)
            .bind(("tenant_id", tenant_id))
            .bind(("limit", limit))
            .bind(("start", start))
            .await?
            .take(0)?;
        Ok(result)
    }

    async fn count_by_tenant(&self, tenant_id: &str) -> Result<u64> {
        let query = "
            SELECT count() FROM session 
            WHERE tenant_id = $tenant_id 
            GROUP ALL
        ";
        let result: Vec<serde_json::Value> = self
            .db
            .query(query)
            .bind(("tenant_id", tenant_id))
            .await?
            .take(0)?;
        Ok(result
            .first()
            .and_then(|v| v.get("count"))
            .and_then(|c| c.as_u64())
            .unwrap_or(0))
    }
}

/// 轮次仓储实现
#[derive(Clone)]
pub struct TurnRepository {
    db: Surreal<Any>,
    _marker: PhantomData<Turn>,
}

impl TurnRepository {
    pub fn new(db: Surreal<Any>) -> Self {
        Self {
            db,
            _marker: PhantomData,
        }
    }

    /// 获取指定会话的最大 turn_number
    pub async fn get_max_turn_number(&self, session_id: &str) -> Result<u64> {
        let response: Vec<Turn> = self
            .db
            .query("SELECT * FROM turn WHERE session_id = $session_id")
            .bind(("session_id", session_id))
            .await?
            .take(0)?;

        Ok(response
            .into_iter()
            .map(|t| t.turn_number)
            .max()
            .unwrap_or(0))
    }
}

#[async_trait]
impl Repository<Turn> for TurnRepository {
    async fn create(&self, turn: &Turn) -> Result<Turn> {
        let created: Option<Turn> = self.db.create(("turn", &turn.id)).content(turn).await?;

        created.ok_or_else(|| {
            crate::error::AppError::Database(format!("Failed to create turn: {}", turn.id))
        })
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Turn>> {
        let result: Option<Turn> = self.db.select(("turn", id)).await?;
        Ok(result)
    }

    async fn update(&self, id: &str, turn: &Turn) -> Result<Option<Turn>> {
        let updated: Option<Turn> = self.db.update(("turn", id)).content(turn).await?;
        Ok(updated)
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let result: Option<Turn> = self.db.delete(("turn", id)).await?;
        Ok(result.is_some())
    }

    async fn list(&self, limit: usize, start: usize) -> Result<Vec<Turn>> {
        // Use LIMIT/OFFSET for efficient pagination
        let query = format!(
            "SELECT * FROM turn ORDER BY created_at DESC LIMIT {} START {}",
            limit, start
        );
        let result: Vec<Turn> = self.db.query(query).await?.take(0)?;
        Ok(result)
    }

    async fn count(&self) -> Result<u64> {
        // Use count() aggregation for efficient counting
        let result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM turn GROUP ALL")
            .await?
            .take(0)?;
        if let Some(count_val) = result.first().and_then(|v| v.get("count")) {
            Ok(count_val.as_u64().unwrap_or(0))
        } else {
            Ok(0)
        }
    }

    // === 会话过滤实现 ===

    async fn list_by_session(
        &self,
        session_id: &str,
        limit: usize,
        start: usize,
    ) -> Result<Vec<Turn>> {
        let query = "
            SELECT * FROM turn 
            WHERE session_id = $session_id 
            ORDER BY turn_number ASC 
            LIMIT $limit START $start
        ";
        let result: Vec<Turn> = self
            .db
            .query(query)
            .bind(("session_id", session_id))
            .bind(("limit", limit))
            .bind(("start", start))
            .await?
            .take(0)?;
        Ok(result)
    }

    async fn count_by_session(&self, session_id: &str) -> Result<u64> {
        let query = "
            SELECT count() FROM turn 
            WHERE session_id = $session_id 
            GROUP ALL
        ";
        let result: Vec<serde_json::Value> = self
            .db
            .query(query)
            .bind(("session_id", session_id))
            .await?
            .take(0)?;
        Ok(result
            .first()
            .and_then(|v| v.get("count"))
            .and_then(|c| c.as_u64())
            .unwrap_or(0))
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
        let created: Option<IndexRecord> = self
            .db
            .create(("index_record", &record.turn_id))
            .content(record)
            .await?;

        created.ok_or_else(|| {
            crate::error::AppError::Database(format!(
                "Failed to create index record: {}",
                record.turn_id
            ))
        })
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<IndexRecord>> {
        let result: Option<IndexRecord> = self.db.select(("index_record", id)).await?;
        Ok(result)
    }

    async fn update(&self, id: &str, record: &IndexRecord) -> Result<Option<IndexRecord>> {
        let updated: Option<IndexRecord> =
            self.db.update(("index_record", id)).content(record).await?;
        Ok(updated)
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        let result: Option<IndexRecord> = self.db.delete(("index_record", id)).await?;
        Ok(result.is_some())
    }

    async fn list(&self, limit: usize, start: usize) -> Result<Vec<IndexRecord>> {
        // Use LIMIT/OFFSET for efficient pagination
        let query = format!(
            "SELECT * FROM index_record ORDER BY timestamp DESC LIMIT {} START {}",
            limit, start
        );
        let result: Vec<IndexRecord> = self.db.query(query).await?.take(0)?;
        Ok(result)
    }

    async fn count(&self) -> Result<u64> {
        // Use count() aggregation for efficient counting
        let result: Vec<serde_json::Value> = self
            .db
            .query("SELECT count() FROM index_record GROUP ALL")
            .await?
            .take(0)?;
        if let Some(count_val) = result.first().and_then(|v| v.get("count")) {
            Ok(count_val.as_u64().unwrap_or(0))
        } else {
            Ok(0)
        }
    }
}
