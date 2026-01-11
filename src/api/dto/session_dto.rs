//! 会话 DTO
//!
//! 定义会话相关的请求和响应数据结构。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 创建会话请求
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct CreateSessionRequest {
    /// 会话名称
    pub name: String,
    /// 会话描述
    pub description: Option<String>,
    /// 最大轮次数
    pub max_turns: Option<u32>,
    /// 保留摘要数量
    pub summary_limit: Option<usize>,
    /// 启用语义搜索
    pub semantic_search_enabled: Option<bool>,
    /// 启用自动摘要
    pub auto_summarize: Option<bool>,
}

impl Default for CreateSessionRequest {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            max_turns: None,
            summary_limit: None,
            semantic_search_enabled: None,
            auto_summarize: None,
        }
    }
}

/// 更新会话请求
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct UpdateSessionRequest {
    /// 会话名称
    pub name: Option<String>,
    /// 会话描述
    pub description: Option<String>,
    /// 最大轮次数
    pub max_turns: Option<u32>,
    /// 会话状态
    pub status: Option<String>,
}

impl Default for UpdateSessionRequest {
    fn default() -> Self {
        Self {
            name: None,
            description: None,
            max_turns: None,
            status: None,
        }
    }
}

/// 会话配置响应
#[derive(Debug, Serialize)]
#[serde(default)]
pub struct SessionConfigResponse {
    /// 保留的摘要数量
    pub summary_limit: usize,
    /// 最大轮次数
    pub max_turns: usize,
    /// 启用语义搜索
    pub semantic_search_enabled: bool,
    /// 启用自动摘要
    pub auto_summarize: bool,
}

/// 会话统计响应
#[derive(Debug, Serialize)]
#[serde(default)]
pub struct SessionStatsResponse {
    /// 总轮次数
    pub total_turns: u64,
    /// 总 Token 数
    pub total_tokens: u64,
    /// 存储大小（字节）
    pub storage_size: u64,
    /// 最后索引时间
    pub last_indexed_at: Option<DateTime<Utc>>,
}

/// 会话响应
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    /// 会话 ID
    pub id: String,
    /// 租户 ID
    pub tenant_id: String,
    /// 会话名称
    pub name: String,
    /// 会话描述
    pub description: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 最后活跃时间
    pub last_active_at: DateTime<Utc>,
    /// 会话状态
    pub status: String,
    /// 配置
    pub config: SessionConfigResponse,
    /// 统计信息
    pub stats: SessionStatsResponse,
}

/// 会话列表响应
#[derive(Debug, Serialize)]
pub struct SessionListResponse {
    /// 会话列表
    pub sessions: Vec<SessionResponse>,
    /// 总数
    pub total: usize,
    /// 当前页
    pub page: usize,
    /// 每页数量
    pub page_size: usize,
}

/// 创建会话响应
#[derive(Debug, Serialize)]
pub struct CreateSessionResponse {
    /// 会话 ID
    pub id: String,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 更新会话响应
#[derive(Debug, Serialize)]
pub struct UpdateSessionResponse {
    /// 会话 ID
    pub id: String,
    /// 消息
    pub message: String,
}

/// 删除会话响应
#[derive(Debug, Serialize)]
pub struct DeleteSessionResponse {
    /// 会话 ID
    pub id: String,
    /// 消息
    pub message: String,
}

/// 归档会话请求
#[derive(Debug, Deserialize, Default)]
pub struct ArchiveSessionRequest {
    /// 归档原因
    pub reason: Option<String>,
}

/// 归档会话响应
#[derive(Debug, Serialize)]
pub struct ArchiveSessionResponse {
    /// 会话 ID
    pub id: String,
    /// 会话状态
    pub status: String,
    /// 归档时间
    pub archived_at: DateTime<Utc>,
    /// 消息
    pub message: String,
}

/// 恢复会话请求
#[derive(Debug, Deserialize, Default)]
pub struct RestoreSessionRequest {
    /// 恢复后的新名称
    pub new_name: Option<String>,
}

/// 恢复会话响应
#[derive(Debug, Serialize)]
pub struct RestoreSessionResponse {
    /// 会话 ID
    pub id: String,
    /// 会话名称
    pub name: String,
    /// 会话状态
    pub status: String,
    /// 恢复时间
    pub restored_at: DateTime<Utc>,
    /// 消息
    pub message: String,
}
