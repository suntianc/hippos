//! 会话服务
//!
//! 提供会话的 CRUD 操作和生命周期管理。

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::{AppError, Result};
use crate::models::session::{Session, SessionStatus};
use crate::storage::repository::{Repository, SessionRepository, TurnRepository};

/// 分页参数
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Pagination {
    /// 页码（从 1 开始）
    pub page: usize,
    /// 每页数量
    pub page_size: usize,
}

impl Pagination {
    /// 创建新分页参数
    pub fn new(page: usize, page_size: usize) -> Self {
        Self { page, page_size }
    }

    /// 计算偏移量
    pub fn offset(&self) -> usize {
        (self.page.saturating_sub(1)) * self.page_size
    }

    /// 检查分页参数是否有效
    pub fn is_valid(&self) -> bool {
        self.page > 0 && self.page_size > 0
    }
}

/// 会话列表查询参数
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SessionQuery {
    /// 分页参数
    pub pagination: Pagination,
    /// 状态过滤
    pub status: Option<SessionStatus>,
}

/// 会话服务 trait
#[async_trait]
pub trait SessionService: Send + Sync {
    /// 创建会话
    async fn create(&self, tenant_id: &str, name: &str) -> Result<Session>;

    /// 根据 ID 获取会话
    async fn get_by_id(&self, id: &str) -> Result<Option<Session>>;

    /// 更新会话
    async fn update(&self, session: &Session) -> Result<Session>;

    /// 删除会话
    async fn delete(&self, id: &str) -> Result<bool>;

    /// 列出会话
    async fn list(&self, tenant_id: &str, query: SessionQuery) -> Result<Vec<Session>>;

    /// 统计会话数量
    async fn count(&self, tenant_id: &str) -> Result<u64>;

    /// 归档会话
    async fn archive(&self, id: &str, reason: Option<String>) -> Result<Session>;

    /// 恢复会话
    async fn restore(&self, id: &str, new_name: Option<String>) -> Result<Session>;

    /// 验证会话访问权限
    async fn validate_access(&self, session_id: &str, user_id: &str) -> Result<bool>;
}

/// 会话服务实现
pub struct SessionServiceImpl {
    repository: Arc<SessionRepository>,
    turn_repository: Arc<TurnRepository>,
}

impl SessionServiceImpl {
    /// 创建新的服务实例
    pub fn new(repository: Arc<SessionRepository>, turn_repository: Arc<TurnRepository>) -> Self {
        Self {
            repository,
            turn_repository,
        }
    }
}

/// 注意：移除了 Default 实现，因为无法在没有数据库连接的情况下创建 Repository
/// 测试时，请使用 SessionServiceImpl::new() 传入适当的 Arc<SessionRepository>

#[async_trait]
impl SessionService for SessionServiceImpl {
    async fn create(&self, tenant_id: &str, name: &str) -> Result<Session> {
        // 检查同名 Session 是否已存在
        let existing = self
            .repository
            .list_by_tenant(tenant_id, 10, 0)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if existing.iter().any(|s| s.name == name) {
            return Err(AppError::Validation(
                "Session with this name already exists".to_string(),
            ));
        }

        let session = Session::new(tenant_id, name);
        self.repository
            .create(&session)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Session>> {
        self.repository
            .get_by_id(id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn update(&self, session: &Session) -> Result<Session> {
        self.repository
            .update(&session.id, session)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("Session not found: {}", session.id)))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        // 1. 验证 Session 存在
        self.get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Session not found: {}", id)))?;

        // 2. 删除所有关联的 Turn（级联删除，使用 while 循环处理大量数据）
        const BATCH_SIZE: usize = 100;
        let mut offset = 0usize;

        loop {
            let turns = self
                .turn_repository
                .list_by_session(id, BATCH_SIZE, offset)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            if turns.is_empty() {
                break;
            }

            for turn in &turns {
                self.turn_repository
                    .delete(&turn.id)
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
            }

            if turns.len() < BATCH_SIZE {
                break;
            }

            offset += turns.len();
        }

        // 3. 删除 Session
        self.repository
            .delete(id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn list(&self, tenant_id: &str, query: SessionQuery) -> Result<Vec<Session>> {
        let offset = query.pagination.offset();
        let limit = query.pagination.page_size;
        self.repository
            .list_by_tenant(tenant_id, limit, offset)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn count(&self, tenant_id: &str) -> Result<u64> {
        self.repository
            .count_by_tenant(tenant_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn archive(&self, id: &str, _reason: Option<String>) -> Result<Session> {
        let mut session = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Session not found: {}", id)))?;

        if session.status == SessionStatus::Archived {
            return Ok(session);
        }

        session.status = SessionStatus::Archived;
        self.update(&session).await
    }

    async fn restore(&self, id: &str, new_name: Option<String>) -> Result<Session> {
        let mut session = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Session not found: {}", id)))?;

        if session.status != SessionStatus::Archived {
            return Err(AppError::Validation(
                "Only archived sessions can be restored".to_string(),
            ));
        }

        session.status = SessionStatus::Active;
        if let Some(name) = new_name {
            session.name = name;
        }
        self.update(&session).await
    }

    async fn validate_access(&self, session_id: &str, _user_id: &str) -> Result<bool> {
        Ok(self.get_by_id(session_id).await?.is_some())
    }
}

/// 会话归档信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArchiveInfo {
    /// 归档时间
    pub archived_at: DateTime<Utc>,
    /// 归档原因
    pub reason: Option<String>,
}

/// 创建会话服务
pub fn create_session_service(
    repository: Arc<SessionRepository>,
    turn_repository: Arc<TurnRepository>,
) -> Box<dyn SessionService> {
    Box::new(SessionServiceImpl::new(repository, turn_repository))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::session::Session;

    #[tokio::test]
    async fn test_pagination_offset() {
        let pagination = Pagination::new(1, 20);
        assert_eq!(pagination.offset(), 0);

        let pagination = Pagination::new(2, 20);
        assert_eq!(pagination.offset(), 20);

        let pagination = Pagination::new(3, 10);
        assert_eq!(pagination.offset(), 20);
    }

    #[tokio::test]
    async fn test_pagination_invalid() {
        let pagination = Pagination::new(0, 20);
        assert!(!pagination.is_valid());

        let pagination = Pagination::new(1, 0);
        assert!(!pagination.is_valid());

        let pagination = Pagination::new(1, 20);
        assert!(pagination.is_valid());
    }

    #[tokio::test]
    async fn test_session_create() {
        let session = Session::new("tenant_1", "Test Session");
        assert_eq!(session.tenant_id, "tenant_1");
        assert_eq!(session.name, "Test Session");
        assert_eq!(session.status, SessionStatus::Active);
    }
}
