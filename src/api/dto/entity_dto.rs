//! 实体和关系 DTO
//!
//! 用于 Entity/Graph API 的请求和响应序列化

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 创建实体请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEntityRequest {
    /// 实体名称
    pub name: String,

    /// 实体类型
    pub entity_type: EntityTypeDto,

    /// 描述
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// 别名列表
    #[serde(default)]
    pub aliases: Vec<String>,

    /// 属性
    #[serde(default)]
    pub properties: HashMap<String, serde_json::Value>,

    /// 来源记忆 ID
    #[serde(default)]
    pub source_memory_ids: Vec<String>,
}

/// 更新实体请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEntityRequest {
    /// 实体名称
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// 描述
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// 添加别名请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddAliasRequest {
    /// 别名
    pub alias: String,
}

/// 添加属性请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPropertyRequest {
    /// 属性键
    pub key: String,

    /// 属性值
    pub value: serde_json::Value,
}

/// 实体查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEntityRequest {
    /// 实体类型筛选
    #[serde(default)]
    pub types: Vec<EntityTypeDto>,

    /// 名称搜索
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_contains: Option<String>,

    /// 别名搜索
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub alias_contains: Option<String>,

    /// 最小置信度
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f32,

    /// 是否只返回已验证
    #[serde(default)]
    pub verified_only: bool,

    /// 关键词属性搜索
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property_key: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property_value: Option<String>,

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

/// 实体响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityResponse {
    /// 实体 ID
    pub id: String,

    /// 租户 ID
    pub tenant_id: String,

    /// 实体定义
    pub name: String,
    pub entity_type: EntityTypeDto,
    pub description: Option<String>,

    /// 属性
    pub properties: HashMap<String, serde_json::Value>,
    pub aliases: Vec<String>,

    /// 元数据
    pub confidence: f32,
    pub source_memory_ids: Vec<String>,
    pub last_verified: Option<DateTime<Utc>>,
    pub verified: bool,
    pub frequency: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub version: u32,
}

/// 实体列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEntitiesResponse {
    /// 实体列表
    pub entities: Vec<EntityResponse>,

    /// 总数
    pub total: u64,

    /// 页码
    pub page: u32,

    /// 每页数量
    pub page_size: u32,
}

/// 搜索实体响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEntitiesResponse {
    /// 匹配的实体
    pub entities: Vec<EntitySearchResultDto>,

    /// 总数
    pub total: u64,

    /// 搜索时间（毫秒）
    pub search_time_ms: u64,
}

/// 实体搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySearchResultDto {
    /// 实体
    pub entity: EntityResponse,

    /// 搜索评分
    pub score: f32,

    /// 匹配类型
    pub match_type: String,

    /// 匹配内容
    pub matched_content: String,
}

/// 实体类型 DTO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntityTypeDto {
    #[serde(rename = "person")]
    Person,

    #[serde(rename = "organization")]
    Organization,

    #[serde(rename = "project")]
    Project,

    #[serde(rename = "tool")]
    Tool,

    #[serde(rename = "concept")]
    Concept,

    #[serde(rename = "document")]
    Document,

    #[serde(rename = "event")]
    Event,

    #[serde(rename = "location")]
    Location,

    #[serde(rename = "product")]
    Product,

    #[serde(rename = "other")]
    Other,
}

/// 创建关系请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRelationshipRequest {
    /// 源实体 ID
    pub source_entity_id: String,

    /// 目标实体 ID
    pub target_entity_id: String,

    /// 关系类型
    pub relationship_type: RelationshipTypeDto,

    /// 关系强度 (0.0-1.0)
    #[serde(default = "default_strength")]
    pub strength: f32,

    /// 关系上下文
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// 来源记忆 ID
    pub source_memory_id: String,
}

fn default_strength() -> f32 {
    0.5
}

/// 更新关系请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRelationshipRequest {
    /// 关系强度
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strength: Option<f32>,

    /// 关系上下文
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// 关系查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRelationshipRequest {
    /// 源实体 ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_entity_id: Option<String>,

    /// 目标实体 ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_entity_id: Option<String>,

    /// 关系类型筛选
    #[serde(default)]
    pub types: Vec<RelationshipTypeDto>,

    /// 最小强度
    #[serde(default = "default_min_strength")]
    pub min_strength: f32,

    /// 是否只返回已验证
    #[serde(default)]
    pub verified_only: bool,

    /// 分页
    #[serde(default = "default_page")]
    pub page: u32,

    #[serde(default = "default_page_size")]
    pub page_size: u32,
}

fn default_min_strength() -> f32 {
    0.0
}

/// 关系响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipResponse {
    /// 关系 ID
    pub id: String,

    /// 租户 ID
    pub tenant_id: String,

    /// 关系定义
    pub source_entity_id: String,
    pub target_entity_id: String,
    pub relationship_type: RelationshipTypeDto,

    /// 元数据
    pub strength: f32,
    pub context: Option<String>,
    pub source_memory_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub verified: bool,
    pub confidence: f32,
    pub version: u32,
}

/// 关系列表响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRelationshipsResponse {
    /// 关系列表
    pub relationships: Vec<RelationshipResponse>,

    /// 总数
    pub total: u64,

    /// 页码
    pub page: u32,

    /// 每页数量
    pub page_size: u32,
}

/// 关系类型 DTO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelationshipTypeDto {
    #[serde(rename = "knows")]
    Knows,

    #[serde(rename = "works_on")]
    WorksOn,

    #[serde(rename = "part_of")]
    PartOf,

    #[serde(rename = "uses")]
    Uses,

    #[serde(rename = "depends_on")]
    DependsOn,

    #[serde(rename = "belongs_to")]
    BelongsTo,

    #[serde(rename = "references")]
    References,

    #[serde(rename = "conflicts_with")]
    ConflictsWith,

    #[serde(rename = "similar_to")]
    SimilarTo,

    #[serde(rename = "created_by")]
    CreatedBy,

    #[serde(rename = "contains")]
    Contains,

    #[serde(rename = "competes_with")]
    CompetesWith,

    #[serde(rename = "collaborates_with")]
    CollaboratesWith,

    #[serde(rename = "used_by")]
    UsedBy,

    #[serde(rename = "depended_by")]
    DependedBy,

    #[serde(rename = "owns")]
    Owns,

    #[serde(rename = "referenced_by")]
    ReferencedBy,

    #[serde(rename = "has_worker")]
    HasWorker,

    #[serde(rename = "created")]
    Created,

    #[serde(rename = "other")]
    Other,
}

/// 知识图谱查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryRequest {
    /// 中心实体 ID
    pub center_entity_id: String,

    /// 关系类型筛选
    #[serde(default)]
    pub relationship_types: Vec<RelationshipTypeDto>,

    /// 实体类型筛选
    #[serde(default)]
    pub entity_types: Vec<EntityTypeDto>,

    /// 最大深度
    #[serde(default = "default_max_depth")]
    pub max_depth: u32,

    /// 每层最大实体数
    #[serde(default = "default_limit_per_depth")]
    pub limit_per_depth: u32,

    /// 最小关系强度
    #[serde(default = "default_min_strength_graph")]
    pub min_strength: f32,

    /// 是否包含中心实体
    #[serde(default)]
    pub include_center: bool,
}

fn default_max_depth() -> u32 {
    2
}

fn default_limit_per_depth() -> u32 {
    10
}

fn default_min_strength_graph() -> f32 {
    0.3
}

/// 知识图谱响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQueryResponse {
    /// 实体映射
    pub entities: Vec<EntityResponse>,

    /// 关系列表
    pub relationships: Vec<RelationshipResponse>,

    /// 路径信息
    pub paths: Vec<GraphPathDto>,
}

/// 图路径
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPathDto {
    /// 路径上的实体 ID
    pub entity_ids: Vec<String>,

    /// 路径上的关系 ID
    pub relationship_ids: Vec<String>,

    /// 路径长度
    pub length: u32,

    /// 路径强度
    pub strength: f32,
}

/// 图统计响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStatsResponse {
    /// 总实体数
    pub total_entities: u64,

    /// 按类型统计
    pub person_count: u64,
    pub organization_count: u64,
    pub project_count: u64,
    pub tool_count: u64,
    pub concept_count: u64,

    /// 总关系数
    pub total_relationships: u64,

    /// 按类型统计
    pub knows_count: u64,
    pub works_on_count: u64,
    pub uses_count: u64,
    pub depends_on_count: u64,
    pub similar_to_count: u64,

    /// 连通性
    pub connected_components: u64,
    pub largest_component_size: u64,

    /// 密度
    pub density: f32,
}

/// 发现实体请求（从文本中提取）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverEntitiesRequest {
    /// 源文本
    pub text: String,

    /// 源记忆 ID
    pub source_memory_id: String,

    /// 期望的实体类型（可选）
    #[serde(default)]
    pub expected_types: Vec<EntityTypeDto>,
}

/// 发现实体响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverEntitiesResponse {
    /// 发现的实体
    pub entities: Vec<EntityResponse>,

    /// 发现的关系
    pub relationships: Vec<RelationshipResponse>,

    /// 新创建的数量
    pub created_count: u64,

    /// 已存在的数量
    pub existing_count: u64,
}

/// 批量创建实体请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateEntitiesRequest {
    /// 实体列表
    pub entities: Vec<CreateEntityRequest>,
}

/// 批量创建实体响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCreateEntitiesResponse {
    /// 创建的实体 ID
    pub entity_ids: Vec<String>,

    /// 失败的数量
    pub failed_count: u32,

    /// 错误信息
    pub errors: Vec<EntityCreateError>,
}

/// 实体创建错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCreateError {
    pub index: u32,
    pub message: String,
}
