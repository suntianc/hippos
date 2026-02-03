//! 问题模式 DTO
//!
//! 用于 Pattern API 的请求和响应序列化

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 创建模式请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePatternRequest {
    /// 模式类型
    pub pattern_type: PatternTypeDto,

    /// 模式名称
    pub name: String,

    /// 模式描述
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 触发条件
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,

    /// 适用场景
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// 问题描述
    pub problem: String,

    /// 解决方案
    pub solution: String,

    /// 详细解释
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,

    /// 标签
    #[serde(default)]
    pub tags: Vec<String>,

    /// 是否公开
    #[serde(default)]
    pub is_public: bool,
}

/// 更新模式请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePatternRequest {
    /// 模式名称
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// 模式描述
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 触发条件
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,

    /// 适用场景
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// 问题描述
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,

    /// 解决方案
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solution: Option<String>,

    /// 详细解释
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

/// 添加示例请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddExampleRequest {
    /// 输入
    pub input: String,

    /// 输出
    pub output: String,

    /// 结果评分 (-1.0 到 1.0)
    #[serde(default)]
    pub outcome: f32,

    /// 来源记忆 ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_memory_id: Option<String>,
}

/// 记录使用请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordUsageRequest {
    /// 使用时的输入
    pub input: String,

    /// 使用的输出
    pub output: String,

    /// 结果评分 (-1.0 到 1.0)
    pub outcome: f32,

    /// 用户反馈
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,

    /// 上下文
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// 模式查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPatternRequest {
    /// 模式类型筛选
    #[serde(default)]
    pub types: Vec<PatternTypeDto>,

    /// 标签筛选
    #[serde(default)]
    pub tags: Vec<String>,

    /// 最小置信度
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f32,

    /// 最小成功率
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_success_rate: Option<f32>,

    /// 关键词搜索
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyword: Option<String>,

    /// 是否只返回公开模式
    #[serde(default)]
    pub public_only: bool,

    /// 分页
    #[serde(default = "default_page")]
    pub page: u32,

    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_min_confidence() -> f32 {
    0.0
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

/// 模式响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternResponse {
    /// 模式 ID
    pub id: String,

    /// 租户 ID
    pub tenant_id: String,

    /// 模式类型
    pub pattern_type: PatternTypeDto,

    /// 模式定义
    pub name: String,
    pub description: String,
    pub trigger: String,
    pub context: String,

    /// 模式内容
    pub problem: String,
    pub solution: String,
    pub explanation: Option<String>,
    pub examples: Vec<PatternExampleDto>,

    /// 效果追踪
    pub success_count: u32,
    pub failure_count: u32,
    pub avg_outcome: f32,
    pub last_used: Option<DateTime<Utc>>,

    /// 元数据
    pub tags: Vec<String>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub usage_count: u32,
    pub is_public: bool,
    pub confidence: f32,
    pub version: u32,
}

/// 模式列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPatternsResponse {
    /// 模式列表
    pub patterns: Vec<PatternResponse>,

    /// 总数
    pub total: u64,

    /// 页码
    pub page: u32,

    /// 每页数量
    pub page_size: u32,
}

/// 搜索模式响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchPatternsResponse {
    /// 匹配的模式
    pub patterns: Vec<PatternResponse>,

    /// 匹配分数
    pub scores: Vec<f32>,

    /// 总数
    pub total: u64,

    /// 搜索时间（毫秒）
    pub search_time_ms: u64,
}

/// 模式推荐响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendPatternsResponse {
    /// 推荐列表
    pub recommendations: Vec<PatternRecommendationDto>,

    /// 推荐数量
    pub count: usize,
}

/// 模式推荐项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRecommendationDto {
    /// 模式
    pub pattern: PatternResponse,

    /// 推荐分数
    pub score: f32,

    /// 推荐原因
    pub reasons: Vec<String>,
}

/// 模式示例 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternExampleDto {
    /// 示例 ID
    pub id: String,

    /// 输入
    pub input: String,

    /// 输出
    pub output: String,

    /// 结果评分
    pub outcome: f32,

    /// 来源记忆 ID
    pub source_memory_id: Option<String>,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 模式类型 DTO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PatternTypeDto {
    #[serde(rename = "problem_solution")]
    ProblemSolution,

    #[serde(rename = "workflow")]
    Workflow,

    #[serde(rename = "best_practice")]
    BestPractice,

    #[serde(rename = "common_error")]
    CommonError,

    #[serde(rename = "skill")]
    Skill,
}

/// 模式统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStatsResponse {
    /// 总模式数
    pub total_count: u64,

    /// 按类型统计
    pub problem_solution_count: u64,
    pub workflow_count: u64,
    pub best_practice_count: u64,
    pub common_error_count: u64,
    pub skill_count: u64,

    /// 效果统计
    pub avg_success_rate: f32,
    pub high_quality_count: u64,

    /// 使用统计
    pub total_usages: u64,
    pub most_used_pattern: Option<PatternMostUsedDto>,
}

/// 最多使用的模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMostUsedDto {
    pub pattern_id: String,
    pub pattern_name: String,
    pub usage_count: u32,
}

/// 批量创建模式请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreatePatternsRequest {
    /// 模式列表
    pub patterns: Vec<CreatePatternRequest>,
}

/// 批量创建模式响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreatePatternsResponse {
    /// 创建的模式 ID
    pub pattern_ids: Vec<String>,

    /// 失败的数量
    pub failed_count: u32,

    /// 错误信息
    pub errors: Vec<PatternCreateError>,
}

/// 模式创建错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCreateError {
    pub index: u32,
    pub message: String,
}

/// 匹配模式请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchPatternRequest {
    /// 输入文本
    pub input: String,

    /// 最大匹配数量
    #[serde(default = "default_max_matches")]
    pub max_matches: u32,
}

fn default_max_matches() -> u32 {
    5
}
