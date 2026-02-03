//! 记忆 DTO
//!
//! API 请求和响应的数据传输对象

use crate::models::{Memory, MemoryQuery, MemorySource, MemoryStatus, MemoryType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 创建记忆请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMemoryRequest {
    /// 记忆类型
    pub memory_type: MemoryType,

    /// 记忆内容
    pub content: String,

    /// 来源类型
    pub source: MemorySource,

    /// 原始来源 ID
    pub source_id: Option<String>,

    /// 父记忆 ID
    pub parent_id: Option<String>,

    /// 标签
    pub tags: Vec<String>,

    /// 主题
    pub topics: Vec<String>,

    /// 过期时间
    pub expires_at: Option<DateTime<Utc>>,
}

/// 更新记忆请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemoryRequest {
    /// 内容
    pub content: Option<String>,

    /// 摘要
    pub gist: Option<String>,

    /// 完整摘要
    pub full_summary: Option<String>,

    /// 重要性评分 (0.0-1.0)
    pub importance: Option<f32>,

    /// 标签
    pub tags: Option<Vec<String>>,

    /// 主题
    pub topics: Option<Vec<String>>,

    /// 状态
    pub status: Option<MemoryStatus>,

    /// 相关记忆 ID
    pub related_ids: Option<Vec<String>>,
}

/// 记忆搜索请求
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchMemoryRequest {
    /// 查询关键词
    pub query: Option<String>,

    /// 记忆类型筛选
    pub memory_types: Vec<MemoryType>,

    /// 标签筛选
    pub tags: Vec<String>,

    /// 主题筛选
    pub topics: Vec<String>,

    /// 开始时间
    pub created_after: Option<DateTime<Utc>>,

    /// 结束时间
    pub created_before: Option<DateTime<Utc>>,

    /// 最小重要性
    pub min_importance: Option<f32>,

    /// 最大重要性
    pub max_importance: Option<f32>,

    /// 来源筛选
    pub sources: Vec<MemorySource>,

    /// 分页
    pub page: u32,
    pub page_size: u32,
}

impl SearchMemoryRequest {
    pub fn to_query(&self, user_id: &str) -> MemoryQuery {
        MemoryQuery::new()
            .for_user(user_id)
            .with_types(&self.memory_types)
            .with_tags(&self.tags.iter().map(|s| s.as_str()).collect::<Vec<_>>())
            .with_time_range(self.created_after, self.created_before)
            .with_min_importance(self.min_importance.unwrap_or(0.0))
            .with_pagination(self.page, self.page_size)
    }
}

/// 记忆搜索响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMemoryResponse {
    /// 记忆列表
    pub memories: Vec<MemoryResponse>,

    /// 总数
    pub total: u64,

    /// 页码
    pub page: u32,

    /// 每页大小
    pub page_size: u32,

    /// 总页数
    pub total_pages: u64,

    /// 搜索耗时（毫秒）
    pub search_time_ms: u64,
}

/// 记忆响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryResponse {
    /// ID
    pub id: String,

    /// 类型
    pub memory_type: MemoryType,

    /// 用户 ID
    pub user_id: String,

    /// 内容
    pub content: String,

    /// 摘要
    pub gist: String,

    /// 完整摘要
    pub full_summary: Option<String>,

    /// 重要性
    pub importance: f32,

    /// 来源
    pub source: MemorySource,

    /// 标签
    pub tags: Vec<String>,

    /// 主题
    pub topics: Vec<String>,

    /// 状态
    pub status: MemoryStatus,

    /// 版本
    pub version: u32,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 相关记忆数
    pub related_count: usize,
}

impl From<Memory> for MemoryResponse {
    fn from(memory: Memory) -> Self {
        Self {
            id: memory.id,
            memory_type: memory.memory_type,
            user_id: memory.user_id,
            content: memory.content,
            gist: memory.gist,
            full_summary: memory.full_summary,
            importance: memory.importance,
            source: memory.source,
            tags: memory.tags,
            topics: memory.topics,
            status: memory.status,
            version: memory.version,
            created_at: memory.created_at,
            updated_at: memory.updated_at,
            related_count: memory.related_ids.len(),
        }
    }
}

/// 批量创建记忆请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateMemoryRequest {
    /// 记忆列表
    pub memories: Vec<CreateMemoryRequest>,
}

/// 批量创建响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateMemoryResponse {
    /// 成功创建的数
    pub successful: usize,

    /// 失败的索引
    pub failed_indices: Vec<usize>,

    /// 错误信息
    pub errors: Vec<String>,

    /// 创建的记忆 ID
    pub memory_ids: Vec<String>,
}

/// 记忆统计请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatsRequest {
    /// 是否包含详细信息
    pub include_details: Option<bool>,
}

/// 记忆统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStatsResponse {
    /// 总数
    pub total_count: u64,

    /// 按类型统计
    pub episodic_count: u64,
    pub semantic_count: u64,
    pub procedural_count: u64,
    pub profile_count: u64,

    /// 按状态统计
    pub active_count: u64,
    pub archived_count: u64,

    /// 重要性分布
    pub high_importance_count: u64,
    pub avg_importance: f32,

    /// 存储大小（字节）
    pub storage_size_bytes: u64,
}

/// 记忆版本历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryVersionResponse {
    /// 版本号
    pub version: u32,

    /// 内容快照
    pub content: String,

    /// 重要性
    pub importance: f32,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 变更原因
    pub change_reason: Option<String>,
}

/// 记忆导出请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMemoryRequest {
    /// 记忆类型筛选
    pub memory_types: Vec<MemoryType>,

    /// 开始时间
    pub created_after: Option<DateTime<Utc>>,

    /// 结束时间
    pub created_before: Option<DateTime<Utc>>,

    /// 格式
    pub format: ExportFormat,
}

/// 导出格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    #[serde(rename = "json")]
    Json,

    #[serde(rename = "jsonl")]
    Jsonl,

    #[serde(rename = "markdown")]
    Markdown,
}

/// 导出响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMemoryResponse {
    /// 导出数据
    pub data: String,

    /// 格式
    pub format: ExportFormat,

    /// 总数
    pub total_count: u64,

    /// 文件大小（字节）
    pub file_size_bytes: u64,
}
