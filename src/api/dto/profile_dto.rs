//! 用户画像 DTO
//!
//! 用于 Profile API 的请求和响应序列化

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 创建画像请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateProfileRequest {
    /// 用户 ID
    pub user_id: String,

    /// 姓名
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// 角色/职位
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// 组织/公司
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,

    /// 位置
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// 语言偏好
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// 初始工具列表
    #[serde(default)]
    pub tools_used: Vec<String>,

    /// 初始兴趣列表
    #[serde(default)]
    pub interests: Vec<String>,
}

/// 更新画像请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    /// 姓名
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// 角色/职位
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// 组织/公司
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,

    /// 位置
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// 沟通风格偏好
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub communication_style: Option<String>,

    /// 技术水平
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technical_level: Option<String>,

    /// 语言偏好
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    /// 变更原因
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// 添加偏好请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPreferenceRequest {
    /// 偏好键
    pub key: String,

    /// 偏好值
    pub value: serde_json::Value,

    /// 变更原因
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// 添加事实请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFactRequest {
    /// 事实描述
    pub fact: String,

    /// 事实类别
    pub category: ProfileFactCategoryDto,

    /// 来源记忆 ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_memory_id: Option<String>,

    /// 置信度
    #[serde(default = "default_confidence")]
    pub confidence: f32,
}

fn default_confidence() -> f32 {
    0.5
}

/// 验证事实请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyFactRequest {
    /// 验证者
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_by: Option<String>,
}

/// 更新工作时间请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkingHoursRequest {
    /// 开始星期（0=周一）
    pub start_day: u32,

    /// 开始时间（小时）
    pub start_hour: u32,

    /// 结束星期
    pub end_day: u32,

    /// 结束时间（小时）
    pub end_hour: u32,

    /// 时区
    #[serde(default = "default_timezone")]
    pub timezone: String,

    /// 是否灵活工作时间
    #[serde(default)]
    pub flexible: bool,
}

fn default_timezone() -> String {
    "Asia/Shanghai".to_string()
}

/// 画像响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileResponse {
    /// 画像 ID
    pub id: String,

    /// 租户 ID
    pub tenant_id: String,

    /// 用户 ID
    pub user_id: String,

    /// 基本信息
    pub name: Option<String>,
    pub role: Option<String>,
    pub organization: Option<String>,
    pub location: Option<String>,

    /// 偏好
    pub preferences: HashMap<String, serde_json::Value>,
    pub communication_style: Option<String>,
    pub technical_level: Option<String>,
    pub language: Option<String>,

    /// 重要事实
    pub facts: Vec<ProfileFactDto>,

    /// 兴趣
    pub interests: Vec<String>,

    /// 工作模式
    pub working_hours: Option<WorkingHoursDto>,
    pub common_tasks: Vec<String>,
    pub tools_used: Vec<String>,

    /// 元数据
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub confidence: f32,
    pub last_verified: Option<DateTime<Utc>>,
    pub version: u32,
}

/// 画像列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProfilesResponse {
    /// 画像列表
    pub profiles: Vec<ProfileResponse>,

    /// 总数
    pub total: u64,

    /// 页码
    pub page: u32,

    /// 每页数量
    pub page_size: u32,
}

/// 画像统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileStatsResponse {
    /// 总画像数
    pub total_count: u64,

    /// 平均置信度
    pub avg_confidence: f32,

    /// 验证事实数
    pub verified_facts_count: u64,

    /// 类别统计
    pub category_stats: Vec<ProfileCategoryStat>,
}

/// 类别统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileCategoryStat {
    pub category: String,
    pub count: u64,
}

/// 画像事实 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileFactDto {
    /// 事实 ID
    pub id: String,

    /// 事实描述
    pub fact: String,

    /// 类别
    pub category: ProfileFactCategoryDto,

    /// 来源记忆 ID
    pub source_memory_id: Option<String>,

    /// 置信度
    pub confidence: f32,

    /// 是否已验证
    pub verified: bool,

    /// 验证时间
    pub verified_at: Option<DateTime<Utc>>,

    /// 验证来源
    pub verified_by: Option<String>,

    /// 创建时间
    pub created_at: DateTime<Utc>,
}

/// 工作时间 DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHoursDto {
    /// 开始星期
    pub start_day: u32,

    /// 开始时间
    pub start_hour: u32,

    /// 结束星期
    pub end_day: u32,

    /// 结束时间
    pub end_hour: u32,

    /// 时区
    pub timezone: String,

    /// 是否灵活
    pub flexible: bool,
}

/// 事实类别 DTO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProfileFactCategoryDto {
    #[serde(rename = "personal")]
    Personal,

    #[serde(rename = "professional")]
    Professional,

    #[serde(rename = "technical")]
    Technical,

    #[serde(rename = "project")]
    Project,

    #[serde(rename = "communication")]
    Communication,

    #[serde(rename = "lifestyle")]
    Lifestyle,

    #[serde(rename = "other")]
    Other,
}

/// 画像对比结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareProfilesResponse {
    /// 新增的事实
    pub added_facts: Vec<ProfileFactDto>,

    /// 冲突的事实
    pub conflicting_facts: Vec<ConflictFact>,

    /// 一致的值
    pub consistent_values: Vec<ConsistentValue>,
}

/// 冲突事实
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictFact {
    pub existing: ProfileFactDto,
    pub incoming: ProfileFactDto,
}

/// 一致的值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistentValue {
    pub key: String,
    pub value: serde_json::Value,
}

/// 合并画像请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeProfileRequest {
    /// 源画像 ID
    pub source_profile_id: String,

    /// 冲突处理策略
    #[serde(default = "default_conflict_strategy")]
    pub conflict_strategy: String,
}

fn default_conflict_strategy() -> String {
    "keep_existing".to_string()
}
