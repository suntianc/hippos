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
}

/// 会话仓储实现
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

        Ok(created.expect("Session should be created"))
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

    async fn list(&self, limit: usize, _start: usize) -> Result<Vec<Session>> {
        // Simplified list - in production use proper pagination
        let mut all: Vec<Session> = self.db.select("session").await?;
        all.truncate(limit);
        Ok(all)
    }

    async fn count(&self) -> Result<u64> {
        // Simplified count - in production use proper query
        let all: Vec<Session> = self.db.select("session").await?;
        Ok(all.len() as u64)
    }
}

/// 轮次仓储实现
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
}

#[async_trait]
impl Repository<Turn> for TurnRepository {
    async fn create(&self, turn: &Turn) -> Result<Turn> {
        let created: Option<Turn> = self.db.create(("turn", &turn.id)).content(turn).await?;
        Ok(created.expect("Turn should be created"))
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

    async fn list(&self, limit: usize, _start: usize) -> Result<Vec<Turn>> {
        let mut all: Vec<Turn> = self.db.select("turn").await?;
        all.truncate(limit);
        Ok(all)
    }

    async fn count(&self) -> Result<u64> {
        let all: Vec<Turn> = self.db.select("turn").await?;
        Ok(all.len() as u64)
    }
}

/// 索引记录仓储实现
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

        Ok(created.expect("IndexRecord should be created"))
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

    async fn list(&self, limit: usize, _start: usize) -> Result<Vec<IndexRecord>> {
        let mut all: Vec<IndexRecord> = self.db.select("index_record").await?;
        all.truncate(limit);
        Ok(all)
    }

    async fn count(&self) -> Result<u64> {
        let all: Vec<IndexRecord> = self.db.select("index_record").await?;
        Ok(all.len() as u64)
    }
}
