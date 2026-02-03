//! 实体和关系数据模型
//!
//! 用于构建知识图谱，管理实体及其关系

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 实体类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EntityType {
    /// 人物
    #[serde(rename = "person")]
    Person,

    /// 组织/公司
    #[serde(rename = "organization")]
    Organization,

    /// 项目
    #[serde(rename = "project")]
    Project,

    /// 工具/软件
    #[serde(rename = "tool")]
    Tool,

    /// 概念/术语
    #[serde(rename = "concept")]
    Concept,

    /// 文档
    #[serde(rename = "document")]
    Document,

    /// 事件
    #[serde(rename = "event")]
    Event,

    /// 位置
    #[serde(rename = "location")]
    Location,

    /// 产品
    #[serde(rename = "product")]
    Product,

    /// 其他
    #[serde(rename = "other")]
    Other,
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Person => write!(f, "person"),
            EntityType::Organization => write!(f, "organization"),
            EntityType::Project => write!(f, "project"),
            EntityType::Tool => write!(f, "tool"),
            EntityType::Concept => write!(f, "concept"),
            EntityType::Document => write!(f, "document"),
            EntityType::Event => write!(f, "event"),
            EntityType::Location => write!(f, "location"),
            EntityType::Product => write!(f, "product"),
            EntityType::Other => write!(f, "other"),
        }
    }
}

/// 实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// 实体唯一标识
    pub id: String,

    /// 租户隔离 ID
    pub tenant_id: String,

    /// === 实体定义 ===
    /// 实体名称
    pub name: String,

    /// 实体类型
    pub entity_type: EntityType,

    /// 描述
    pub description: Option<String>,

    /// === 属性 ===
    /// 键值属性
    pub properties: HashMap<String, serde_json::Value>,

    /// 别名列表
    pub aliases: Vec<String>,

    /// === 向量表示 ===
    /// 向量嵌入（用于语义搜索）
    pub embedding: Option<Vec<f32>>,

    /// === 元数据 ===
    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 置信度 (0.0-1.0)
    pub confidence: f32,

    /// 来源记忆 ID 列表
    pub source_memory_ids: Vec<String>,

    /// 最后验证时间
    pub last_verified: Option<DateTime<Utc>>,

    /// 是否已验证
    pub verified: bool,

    /// 出现频率（用于重要性排序）
    pub frequency: u32,

    /// 版本号
    pub version: u32,
}

/// 关系类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RelationshipType {
    /// 认识/了解
    #[serde(rename = "knows")]
    Knows,

    /// 工作于
    #[serde(rename = "works_on")]
    WorksOn,

    /// 属于（部分）
    #[serde(rename = "part_of")]
    PartOf,

    /// 使用
    #[serde(rename = "uses")]
    Uses,

    /// 依赖
    #[serde(rename = "depends_on")]
    DependsOn,

    /// 属于（归属）
    #[serde(rename = "belongs_to")]
    BelongsTo,

    /// 引用
    #[serde(rename = "references")]
    References,

    /// 冲突
    #[serde(rename = "conflicts_with")]
    ConflictsWith,

    /// 相似
    #[serde(rename = "similar_to")]
    SimilarTo,

    /// 创建
    #[serde(rename = "created_by")]
    CreatedBy,

    /// 包含
    #[serde(rename = "contains")]
    Contains,

    /// 竞争
    #[serde(rename = "competes_with")]
    CompetesWith,

    /// 合作
    #[serde(rename = "collaborates_with")]
    CollaboratesWith,

    /// 被使用（反向）
    #[serde(rename = "used_by")]
    UsedBy,

    /// 依赖（反向）
    #[serde(rename = "depended_by")]
    DependedBy,

    /// 拥有（反向）
    #[serde(rename = "owns")]
    Owns,

    /// 被引用（反向）
    #[serde(rename = "referenced_by")]
    ReferencedBy,

    /// 工作者（反向）
    #[serde(rename = "has_worker")]
    HasWorker,

    /// 创建（反向）
    #[serde(rename = "created")]
    Created,

    /// 其他
    #[serde(rename = "other")]
    Other,
}

impl std::fmt::Display for RelationshipType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationshipType::Knows => write!(f, "knows"),
            RelationshipType::WorksOn => write!(f, "works_on"),
            RelationshipType::PartOf => write!(f, "part_of"),
            RelationshipType::Uses => write!(f, "uses"),
            RelationshipType::DependsOn => write!(f, "depends_on"),
            RelationshipType::BelongsTo => write!(f, "belongs_to"),
            RelationshipType::References => write!(f, "references"),
            RelationshipType::ConflictsWith => write!(f, "conflicts_with"),
            RelationshipType::SimilarTo => write!(f, "similar_to"),
            RelationshipType::CreatedBy => write!(f, "created_by"),
            RelationshipType::Contains => write!(f, "contains"),
            RelationshipType::CompetesWith => write!(f, "competes_with"),
            RelationshipType::CollaboratesWith => write!(f, "collaborates_with"),
            RelationshipType::UsedBy => write!(f, "used_by"),
            RelationshipType::DependedBy => write!(f, "depended_by"),
            RelationshipType::Owns => write!(f, "owns"),
            RelationshipType::ReferencedBy => write!(f, "referenced_by"),
            RelationshipType::HasWorker => write!(f, "has_worker"),
            RelationshipType::Created => write!(f, "created"),
            RelationshipType::Other => write!(f, "other"),
        }
    }
}

/// 关系
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// 关系唯一标识
    pub id: String,

    /// 租户隔离 ID
    pub tenant_id: String,

    /// === 关系定义 ===
    /// 源实体 ID
    pub source_entity_id: String,

    /// 目标实体 ID
    pub target_entity_id: String,

    /// 关系类型
    pub relationship_type: RelationshipType,

    /// === 元数据 ===
    /// 关系强度 (0.0-1.0)
    pub strength: f32,

    /// 关系上下文（描述此关系成立的条件）
    pub context: Option<String>,

    /// 来源记忆 ID
    pub source_memory_id: String,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 是否已验证
    pub verified: bool,

    /// 置信度
    pub confidence: f32,

    /// 版本号
    pub version: u32,
}

/// 知识图谱查询
#[derive(Debug, Clone)]
pub struct GraphQuery {
    /// 中心实体 ID
    pub center_entity_id: String,

    /// 关系类型筛选
    pub relationship_types: Vec<RelationshipType>,

    /// 实体类型筛选
    pub entity_types: Vec<EntityType>,

    /// 最大深度（跳数）
    pub max_depth: u32,

    /// 每层最大实体数
    pub limit_per_depth: u32,

    /// 最小关系强度
    pub min_strength: f32,

    /// 是否包含中心实体
    pub include_center: bool,
}

/// 知识图谱查询结果
#[derive(Debug, Clone)]
pub struct GraphResult {
    /// 实体映射 ID -> Entity
    pub entities: Vec<Entity>,

    /// 关系列表
    pub relationships: Vec<Relationship>,

    /// 路径信息（用于可视化）
    pub paths: Vec<GraphPath>,
}

/// 图路径
#[derive(Debug, Clone)]
pub struct GraphPath {
    /// 路径上的实体 ID
    pub entity_ids: Vec<String>,

    /// 路径上的关系 ID
    pub relationship_ids: Vec<String>,

    /// 路径长度（边数）
    pub length: u32,

    /// 路径强度（最小关系强度）
    pub strength: f32,
}

/// 实体搜索结果
#[derive(Debug, Clone)]
pub struct EntitySearchResult {
    /// 实体
    pub entity: Entity,

    /// 搜索评分
    pub score: f32,

    /// 匹配类型（name/alias/property）
    pub match_type: String,

    /// 匹配内容
    pub matched_content: String,
}

/// 实体查询条件
#[derive(Debug, Clone, Default)]
pub struct EntityQuery {
    /// 实体类型筛选
    pub types: Vec<EntityType>,

    /// 名称搜索
    pub name_contains: Option<String>,

    /// 别名搜索
    pub alias_contains: Option<String>,

    /// 最小置信度
    pub min_confidence: f32,

    /// 是否只返回已验证
    pub verified_only: bool,

    /// 关键词属性搜索
    pub property_key: Option<String>,
    pub property_value: Option<String>,

    /// 分页
    pub page: u32,
    pub page_size: u32,
}

/// 关系查询条件
#[derive(Debug, Clone, Default)]
pub struct RelationshipQuery {
    /// 源实体 ID
    pub source_entity_id: Option<String>,

    /// 目标实体 ID
    pub target_entity_id: Option<String>,

    /// 关系类型筛选
    pub types: Vec<RelationshipType>,

    /// 最小强度
    pub min_strength: f32,

    /// 是否只返回已验证
    pub verified_only: bool,

    /// 分页
    pub page: u32,
    pub page_size: u32,
}

impl Entity {
    /// 创建新实体
    pub fn new(name: &str, entity_type: EntityType) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            tenant_id: "default".to_string(),
            name: name.to_string(),
            entity_type,
            description: None,
            properties: HashMap::new(),
            aliases: Vec::new(),
            embedding: None,
            created_at: now,
            updated_at: now,
            confidence: 0.5,
            source_memory_ids: Vec::new(),
            last_verified: None,
            verified: false,
            frequency: 1,
            version: 1,
        }
    }

    /// 更新实体
    pub fn update(&mut self, name: Option<&str>, description: Option<&str>) {
        if let Some(name) = name {
            self.name = name.to_string();
        }
        if let Some(description) = description {
            self.description = Some(description.to_string());
        }
        self.updated_at = Utc::now();
        self.version += 1;
    }

    /// 添加别名
    pub fn add_alias(&mut self, alias: &str) {
        let alias = alias.to_lowercase();
        if !self.aliases.contains(&alias) {
            self.aliases.push(alias);
            self.updated_at = Utc::now();
        }
    }

    /// 添加属性
    pub fn add_property(&mut self, key: &str, value: serde_json::Value) {
        self.properties.insert(key.to_string(), value);
        self.updated_at = Utc::now();
    }

    /// 标记已验证
    pub fn verify(&mut self) {
        self.verified = true;
        self.last_verified = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// 增加出现频率
    pub fn increment_frequency(&mut self) {
        self.frequency += 1;
        self.updated_at = Utc::now();
    }

    /// 添加来源记忆
    pub fn add_source_memory(&mut self, memory_id: &str) {
        if !self.source_memory_ids.contains(&memory_id.to_string()) {
            self.source_memory_ids.push(memory_id.to_string());
            self.updated_at = Utc::now();
        }
    }

    /// 检查名称或别名是否匹配
    pub fn matches_name(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();
        if self.name.to_lowercase().contains(&query_lower) {
            return true;
        }
        for alias in &self.aliases {
            if alias.contains(&query_lower) {
                return true;
            }
        }
        false
    }
}

impl Relationship {
    /// 创建新关系
    pub fn new(
        source_entity_id: &str,
        target_entity_id: &str,
        relationship_type: RelationshipType,
        source_memory_id: &str,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            tenant_id: "default".to_string(),
            source_entity_id: source_entity_id.to_string(),
            target_entity_id: target_entity_id.to_string(),
            relationship_type,
            strength: 0.5,
            context: None,
            source_memory_id: source_memory_id.to_string(),
            created_at: now,
            updated_at: now,
            verified: false,
            confidence: 0.5,
            version: 1,
        }
    }

    /// 更新关系
    pub fn update(&mut self, strength: Option<f32>, context: Option<&str>) {
        if let Some(strength) = strength {
            self.strength = strength.clamp(0.0, 1.0);
        }
        if let Some(context) = context {
            self.context = Some(context.to_string());
        }
        self.updated_at = Utc::now();
        self.version += 1;
    }

    /// 标记已验证
    pub fn verify(&mut self) {
        self.verified = true;
        self.updated_at = Utc::now();
    }

    /// 获取反向关系类型
    pub fn get_reverse_type(&self) -> RelationshipType {
        match self.relationship_type {
            RelationshipType::Knows => RelationshipType::Knows,
            RelationshipType::WorksOn => RelationshipType::HasWorker,
            RelationshipType::PartOf => RelationshipType::Contains,
            RelationshipType::Uses => RelationshipType::UsedBy,
            RelationshipType::DependsOn => RelationshipType::DependedBy,
            RelationshipType::BelongsTo => RelationshipType::Owns,
            RelationshipType::References => RelationshipType::ReferencedBy,
            RelationshipType::ConflictsWith => RelationshipType::ConflictsWith,
            RelationshipType::SimilarTo => RelationshipType::SimilarTo,
            RelationshipType::CreatedBy => RelationshipType::Created,
            RelationshipType::Contains => RelationshipType::PartOf,
            RelationshipType::CompetesWith => RelationshipType::CompetesWith,
            RelationshipType::CollaboratesWith => RelationshipType::CollaboratesWith,
            // 反向关系变体返回自身（对称关系）
            RelationshipType::UsedBy => RelationshipType::Uses,
            RelationshipType::DependedBy => RelationshipType::DependsOn,
            RelationshipType::Owns => RelationshipType::BelongsTo,
            RelationshipType::ReferencedBy => RelationshipType::References,
            RelationshipType::HasWorker => RelationshipType::WorksOn,
            RelationshipType::Created => RelationshipType::CreatedBy,
            RelationshipType::Other => RelationshipType::Other,
        }
    }
}

/// 图统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new("Rust", EntityType::Concept);
        assert_eq!(entity.name, "Rust");
        assert_eq!(entity.entity_type, EntityType::Concept);
        assert!(!entity.id.is_empty());
    }

    #[test]
    fn test_entity_operations() {
        let mut entity = Entity::new("VSCode", EntityType::Tool);

        // 更新
        entity.update(Some("Visual Studio Code"), Some("微软的代码编辑器"));

        assert_eq!(entity.name, "Visual Studio Code");

        // 添加别名
        entity.add_alias("vs code");
        entity.add_alias("vscode");
        assert_eq!(entity.aliases.len(), 2);

        // 添加属性
        entity.add_property("developer", serde_json::json!("Microsoft"));
        entity.add_property("license", serde_json::json!("MIT"));

        assert_eq!(entity.properties.len(), 2);

        // 验证
        entity.verify();
        assert!(entity.verified);

        // 频率
        entity.increment_frequency();
        entity.increment_frequency();
        assert_eq!(entity.frequency, 3);

        // 名称匹配
        assert!(entity.matches_name("vscode"));
        assert!(entity.matches_name("VS"));
        assert!(!entity.matches_name("emacs"));
    }

    #[test]
    fn test_relationship_creation() {
        let mut relationship =
            Relationship::new("entity_1", "entity_2", RelationshipType::Uses, "memory_123");

        assert_eq!(relationship.source_entity_id, "entity_1");
        assert_eq!(relationship.target_entity_id, "entity_2");
        assert_eq!(relationship.relationship_type, RelationshipType::Uses);
        assert_eq!(relationship.strength, 0.5);

        // 更新
        relationship.update(Some(0.8), Some("在项目开发中使用"));

        assert_eq!(relationship.strength, 0.8);
        assert_eq!(relationship.context, Some("在项目开发中使用".to_string()));
    }

    #[test]
    fn test_graph_query() {
        let query = GraphQuery {
            center_entity_id: "entity_1".to_string(),
            relationship_types: vec![RelationshipType::Uses, RelationshipType::DependsOn],
            entity_types: vec![EntityType::Tool, EntityType::Concept],
            max_depth: 2,
            limit_per_depth: 10,
            min_strength: 0.5,
            include_center: true,
        };

        assert_eq!(query.max_depth, 2);
        assert_eq!(query.relationship_types.len(), 2);
    }
}
