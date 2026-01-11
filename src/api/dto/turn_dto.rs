//! 轮次 DTO
//!
//! 定义轮次相关的请求和响应数据结构。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 创建轮次请求
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct CreateTurnRequest {
    /// 原始内容
    pub content: String,
    /// 消息类型
    pub message_type: Option<String>,
    /// 角色
    pub role: Option<String>,
    /// 模型名称
    pub model: Option<String>,
    /// 父轮次 ID
    pub parent_id: Option<String>,
    /// 用户 ID
    pub user_id: Option<String>,
}

impl Default for CreateTurnRequest {
    fn default() -> Self {
        Self {
            content: String::new(),
            message_type: None,
            role: None,
            model: None,
            parent_id: None,
            user_id: None,
        }
    }
}

/// 轮次元数据响应
#[derive(Debug, Serialize)]
#[serde(default)]
pub struct TurnMetadataResponse {
    /// 时间戳
    pub timestamp: DateTime<Utc>,
    /// 用户 ID
    pub user_id: Option<String>,
    /// 消息类型
    pub message_type: String,
    /// 角色
    pub role: Option<String>,
    /// 模型名称
    pub model: Option<String>,
    /// Token 数量
    pub token_count: Option<u64>,
}

/// 脱水数据响应
#[derive(Debug, Serialize)]
#[serde(default)]
pub struct DehydratedDataResponse {
    /// 极简概括
    pub gist: String,
    /// 主题列表
    pub topics: Vec<String>,
    /// 关键词标签
    pub tags: Vec<String>,
    /// 生成时间
    pub generated_at: DateTime<Utc>,
    /// 生成器
    pub generator: Option<String>,
}

/// 轮次响应
#[derive(Debug, Serialize)]
pub struct TurnResponse {
    /// 轮次 ID
    pub id: String,
    /// 会话 ID
    pub session_id: String,
    /// 轮次序号
    pub turn_number: u64,
    /// 原始内容
    pub raw_content: String,
    /// 元数据
    pub metadata: TurnMetadataResponse,
    /// 脱水数据
    pub dehydrated: Option<DehydratedDataResponse>,
    /// 内容状态
    pub status: String,
    /// 父轮次 ID
    pub parent_id: Option<String>,
}

/// 轮次列表响应
#[derive(Debug, Serialize)]
pub struct TurnListResponse {
    /// 轮次列表
    pub turns: Vec<TurnResponse>,
    /// 总数
    pub total: usize,
    /// 当前页
    pub page: usize,
    /// 每页数量
    pub page_size: usize,
}

/// 创建轮次响应
#[derive(Debug, Serialize)]
pub struct CreateTurnResponse {
    /// 轮次 ID
    pub id: String,
    /// 轮次序号
    pub turn_number: u64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 删除轮次响应
#[derive(Debug, Serialize)]
pub struct DeleteTurnResponse {
    /// 轮次 ID
    pub id: String,
    /// 消息
    pub message: String,
}

/// 更新轮次请求
#[derive(Debug, Deserialize, Default)]
pub struct UpdateTurnRequest {
    /// 原始内容
    pub content: Option<String>,
    /// 消息类型
    pub message_type: Option<String>,
    /// 角色
    pub role: Option<String>,
    /// 模型名称
    pub model: Option<String>,
    /// 用户 ID
    pub user_id: Option<String>,
}

/// 更新轮次响应
#[derive(Debug, Serialize)]
pub struct UpdateTurnResponse {
    /// 轮次 ID
    pub id: String,
    /// 消息
    pub message: String,
}
