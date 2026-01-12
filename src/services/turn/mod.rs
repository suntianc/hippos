//! 轮次服务
//!
//! 提供对话轮次的 CRUD 操作和批量处理。

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::error::{AppError, Result};
use crate::models::turn::{MessageType, Turn, TurnMetadata};
use crate::storage::repository::{Repository, SessionRepository, TurnRepository};

/// 批量创建结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateResult {
    /// 成功创建的轮次数量
    pub successful: usize,
    /// 失败的轮次索引
    pub failed_indices: Vec<usize>,
    /// 错误信息映射
    pub errors: Vec<String>,
    /// 创建的轮次
    pub turns: Vec<Turn>,
}

/// 轮次分组（用于识别 user-assistant 对）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnGroup {
    /// 分组 ID
    pub group_id: String,
    /// 轮次编号范围
    pub start_turn: u64,
    pub end_turn: u64,
    /// 分组类型
    pub group_type: TurnGroupType,
    /// 包含的轮次
    pub turn_ids: Vec<String>,
}

/// 轮次分组类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TurnGroupType {
    /// 用户消息
    User,
    /// 助手回复
    Assistant,
    /// 系统消息
    System,
    /// 混合分组
    Mixed,
}

/// 轮次查询参数
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TurnQuery {
    /// 页码
    pub page: usize,
    /// 每页数量
    pub page_size: usize,
    /// 消息类型过滤
    pub message_type: Option<String>,
}

/// 轮次服务 trait
#[async_trait]
pub trait TurnService: Send + Sync {
    /// 创建轮次
    async fn create(
        &self,
        session_id: &str,
        content: &str,
        metadata: Option<TurnMetadata>,
    ) -> Result<Turn>;

    /// 根据 ID 获取轮次
    async fn get_by_id(&self, id: &str) -> Result<Option<Turn>>;

    /// 更新轮次
    async fn update(&self, turn: &Turn) -> Result<Turn>;

    /// 删除轮次
    async fn delete(&self, id: &str) -> Result<bool>;

    /// 列出会话的所有轮次
    async fn list_by_session(&self, session_id: &str, query: TurnQuery) -> Result<Vec<Turn>>;

    /// 统计会话的轮次数量
    async fn count_by_session(&self, session_id: &str) -> Result<u64>;

    /// 获取下一个轮次编号
    async fn get_next_turn_number(&self, session_id: &str) -> Result<u64>;

    /// 批量创建轮次
    async fn create_batch(
        &self,
        session_id: &str,
        contents: Vec<&str>,
    ) -> Result<BatchCreateResult>;

    /// 识别轮次分组
    async fn identify_turn_groups(&self, session_id: &str) -> Result<Vec<TurnGroup>>;
}

/// 轮次服务实现
pub struct TurnServiceImpl {
    repository: Arc<TurnRepository>,
    session_repository: Arc<SessionRepository>,
}

impl TurnServiceImpl {
    /// 创建新的服务实例
    pub fn new(
        repository: Arc<TurnRepository>,
        session_repository: Arc<SessionRepository>,
    ) -> Self {
        Self {
            repository,
            session_repository,
        }
    }
}

/// 注意：移除了 Default 实现，因为无法在没有数据库连接的情况下创建 Repository
/// 测试时，请使用 TurnServiceImpl::new() 传入适当的 Arc<TurnRepository>

#[async_trait]
impl TurnService for TurnServiceImpl {
    async fn create(
        &self,
        session_id: &str,
        content: &str,
        metadata: Option<TurnMetadata>,
    ) -> Result<Turn> {
        // 验证 Session 存在
        self.session_repository
            .get_by_id(session_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("Session not found: {}", session_id)))?;

        let turn_number = self.get_next_turn_number(session_id).await?;
        let mut turn = Turn::new(session_id, turn_number, content);
        if let Some(md) = metadata {
            turn.metadata = md;
        }
        self.repository
            .create(&turn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn get_by_id(&self, id: &str) -> Result<Option<Turn>> {
        self.repository
            .get_by_id(id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn update(&self, turn: &Turn) -> Result<Turn> {
        self.repository
            .update(&turn.id, turn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
            .ok_or_else(|| AppError::NotFound(format!("Turn not found: {}", turn.id)))
    }

    async fn delete(&self, id: &str) -> Result<bool> {
        self.repository
            .delete(id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn list_by_session(&self, session_id: &str, query: TurnQuery) -> Result<Vec<Turn>> {
        // 检查页码是否越界
        let total = self
            .count_by_session(session_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if query.page > 0 {
            let max_page = (total as f64 / query.page_size as f64).ceil() as usize;
            if max_page > 0 && query.page > max_page {
                return Err(AppError::Validation(format!(
                    "Page {} exceeds maximum page {} (total: {} items)",
                    query.page, max_page, total
                )));
            }
        }

        let offset = (query.page.saturating_sub(1)) * query.page_size;
        let limit = query.page_size;
        self.repository
            .list_by_session(session_id, limit, offset)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn count_by_session(&self, session_id: &str) -> Result<u64> {
        self.repository
            .count_by_session(session_id)
            .await
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn get_next_turn_number(&self, session_id: &str) -> Result<u64> {
        self.repository
            .get_max_turn_number(session_id)
            .await
            .map(|n| n + 1)
            .map_err(|e| AppError::Database(e.to_string()))
    }

    async fn create_batch(
        &self,
        session_id: &str,
        contents: Vec<&str>,
    ) -> Result<BatchCreateResult> {
        let mut successful = 0;
        let mut failed_indices = Vec::new();
        let mut errors = Vec::new();
        let mut turns = Vec::new();

        for (i, content) in contents.iter().enumerate() {
            match self.create(session_id, content, None).await {
                Ok(turn) => {
                    successful += 1;
                    turns.push(turn);
                }
                Err(e) => {
                    failed_indices.push(i);
                    errors.push(e.to_string());
                }
            }
        }

        Ok(BatchCreateResult {
            successful,
            failed_indices,
            errors,
            turns,
        })
    }

    async fn identify_turn_groups(&self, session_id: &str) -> Result<Vec<TurnGroup>> {
        let session_turns = self
            .repository
            .list_by_session(session_id, 1000, 0)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut groups = Vec::new();
        let mut current_group: Option<TurnGroup> = None;

        for turn in session_turns {
            let group_type = match turn.metadata.message_type {
                MessageType::User => TurnGroupType::User,
                MessageType::Assistant => TurnGroupType::Assistant,
                MessageType::System => TurnGroupType::System,
            };

            if let Some(ref mut group) = current_group {
                if group.group_type == group_type
                    || (group.group_type == TurnGroupType::User
                        && group_type == TurnGroupType::Assistant)
                {
                    group.end_turn = turn.turn_number;
                    group.turn_ids.push(turn.id);
                } else {
                    groups.push(current_group.take().unwrap());
                    current_group = Some(TurnGroup {
                        group_id: format!("group_{}", turn.turn_number),
                        start_turn: turn.turn_number,
                        end_turn: turn.turn_number,
                        group_type,
                        turn_ids: vec![turn.id],
                    });
                }
            } else {
                current_group = Some(TurnGroup {
                    group_id: format!("group_{}", turn.turn_number),
                    start_turn: turn.turn_number,
                    end_turn: turn.turn_number,
                    group_type,
                    turn_ids: vec![turn.id],
                });
            }
        }

        if let Some(group) = current_group {
            groups.push(group);
        }

        Ok(groups)
    }
}

/// 创建轮次服务
pub fn create_turn_service(
    repository: Arc<TurnRepository>,
    session_repository: Arc<SessionRepository>,
) -> Box<dyn TurnService> {
    Box::new(TurnServiceImpl::new(repository, session_repository))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::turn::{MessageType, Turn};

    #[tokio::test]
    async fn test_turn_create() {
        let turn = Turn::new("session_1", 1, "Hello, world!");
        assert_eq!(turn.session_id, "session_1");
        assert_eq!(turn.turn_number, 1);
        assert_eq!(turn.raw_content, "Hello, world!");
        assert_eq!(turn.metadata.message_type, MessageType::User);
    }

    #[tokio::test]
    async fn test_batch_create_result() {
        let result = BatchCreateResult {
            successful: 2,
            failed_indices: vec![1],
            errors: vec!["Error creating turn".to_string()],
            turns: vec![],
        };
        assert_eq!(result.successful, 2);
        assert_eq!(result.failed_indices.len(), 1);
    }

    #[tokio::test]
    async fn test_turn_group() {
        let group = TurnGroup {
            group_id: "group_1".to_string(),
            start_turn: 1,
            end_turn: 2,
            group_type: TurnGroupType::Mixed,
            turn_ids: vec!["turn_1".to_string(), "turn_2".to_string()],
        };
        assert_eq!(group.turn_ids.len(), 2);
    }
}
