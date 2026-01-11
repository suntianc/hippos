//! 搜索 DTO
//!
//! 定义搜索相关的请求和响应数据结构。

use serde::{Deserialize, Serialize};

/// 语义搜索请求
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct SemanticSearchRequest {
    /// 搜索查询
    pub query: String,
    /// 返回结果数量
    pub limit: Option<u32>,
    /// 相似度阈值
    pub threshold: Option<f32>,
}

impl Default for SemanticSearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: None,
            threshold: None,
        }
    }
}

/// 混合搜索请求
#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct HybridSearchRequest {
    /// 搜索查询
    pub query: String,
    /// 返回结果数量
    pub limit: Option<u32>,
}

impl Default for HybridSearchRequest {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: None,
        }
    }
}

/// 搜索结果项
#[derive(Debug, Clone, Serialize)]
pub struct SearchResultItem {
    /// 轮次 ID
    pub turn_id: String,
    /// 极简概括
    pub gist: String,
    /// 相似度分数
    pub score: f32,
    /// 搜索类型
    pub result_type: String,
    /// 轮次序号
    pub turn_number: u64,
    /// 时间戳
    pub timestamp: String,
    /// 来源列表
    pub sources: Vec<String>,
}

/// 搜索响应
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    /// 查询
    pub query: String,
    /// 搜索类型
    pub search_type: String,
    /// 结果列表
    pub results: Vec<SearchResultItem>,
    /// 结果数量
    pub total_results: usize,
    /// 耗时（毫秒）
    pub took_ms: u64,
}

/// 最近上下文响应
#[derive(Debug, Serialize)]
pub struct RecentContextResponse {
    /// 轮次列表
    pub turns: Vec<SearchResultItem>,
    /// 总数
    pub total: usize,
}
