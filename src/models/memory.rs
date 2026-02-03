//! 记忆数据模型
//!
//! 支持多类型记忆存储：EPISODIC, SEMANTIC, PROCEDURAL, PROFILE
//! 用于 OpenClaw Agent 的"无线记忆引擎"

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 记忆类型枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryType {
    /// 事件/对话记忆 - 记录具体事件、对话、重要时刻
    #[serde(rename = "episodic")]
    Episodic,

    /// 事实/知识记忆 - 抽象的事实、知识、概念
    #[serde(rename = "semantic")]
    Semantic,

    /// 技能/流程记忆 - Agent 学到的技能、流程、工作模式
    #[serde(rename = "procedural")]
    Procedural,

    /// 用户画像记忆 - 用户基本信息、偏好、习惯、重要事实
    #[serde(rename = "profile")]
    Profile,
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryType::Episodic => write!(f, "episodic"),
            MemoryType::Semantic => write!(f, "semantic"),
            MemoryType::Procedural => write!(f, "procedural"),
            MemoryType::Profile => write!(f, "profile"),
        }
    }
}

/// 记忆来源枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemorySource {
    /// 对话交互 - Agent 与用户的对话、用户主动输入
    #[serde(rename = "conversation")]
    Conversation,

    /// 研究结果 - Agent 主动查询搜索引擎、知识库获取的信息
    #[serde(rename = "research")]
    Research,

    /// 执行经验 - Agent 执行任务后的经验总结、错误教训
    #[serde(rename = "execution")]
    Execution,

    /// 用户配置 - 用户主动提供的背景信息、设定文件
    #[serde(rename = "user_config")]
    UserConfig,
}

impl std::fmt::Display for MemorySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemorySource::Conversation => write!(f, "conversation"),
            MemorySource::Research => write!(f, "research"),
            MemorySource::Execution => write!(f, "execution"),
            MemorySource::UserConfig => write!(f, "user_config"),
        }
    }
}

/// 记忆状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryStatus {
    /// 活跃记忆 - 可被检索和使用
    #[serde(rename = "active")]
    Active,

    /// 已归档 - 重要但较少使用
    #[serde(rename = "archived")]
    Archived,

    /// 已删除 - 标记删除（软删除）
    #[serde(rename = "deleted")]
    Deleted,
}

impl std::fmt::Display for MemoryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MemoryStatus::Active => write!(f, "active"),
            MemoryStatus::Archived => write!(f, "archived"),
            MemoryStatus::Deleted => write!(f, "deleted"),
        }
    }
}

/// 核心记忆结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// 记忆唯一标识
    pub id: String,

    /// 记忆类型
    pub memory_type: MemoryType,

    /// 租户隔离 ID
    pub tenant_id: String,

    /// 用户/Agent ID
    pub user_id: String,

    /// === 内容字段 ===
    /// 原始内容
    pub content: String,

    /// 摘要（脱水后）- 50-100字
    pub gist: String,

    /// 完整摘要 - 深度摘要
    pub full_summary: Option<String>,

    /// 向量表示 - 用于语义搜索
    pub embedding: Option<Vec<f32>>,

    /// === 元数据字段 ===
    /// 重要性评分 (0.0-1.0)
    pub importance: f32,

    /// 置信度
    pub confidence: f32,

    /// 来源类型
    pub source: MemorySource,

    /// 原始来源 ID（如对话 ID、文档 ID）
    pub source_id: Option<String>,

    /// === 关系字段 ===
    /// 父记忆 ID（用于记忆链）
    pub parent_id: Option<String>,

    /// 相关记忆 ID 列表
    pub related_ids: Vec<String>,

    /// 标签
    pub tags: Vec<String>,

    /// 主题
    pub topics: Vec<String>,

    /// === 时间字段 ===
    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 更新时间
    pub updated_at: DateTime<Utc>,

    /// 最后访问时间
    pub accessed_at: DateTime<Utc>,

    /// 过期时间（可选）
    pub expires_at: Option<DateTime<Utc>>,

    /// === 状态字段 ===
    /// 当前状态
    pub status: MemoryStatus,

    /// 版本号（乐观锁）
    pub version: u32,

    /// === 检索相关 ===
    /// 关键词（用于快速检索）
    pub keywords: Vec<String>,
}

impl Memory {
    /// 创建新记忆
    pub fn new(
        user_id: &str,
        memory_type: MemoryType,
        content: &str,
        source: MemorySource,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            memory_type,
            tenant_id: "default".to_string(), // TODO: 从认证中获取
            user_id: user_id.to_string(),
            content: content.to_string(),
            gist: String::new(),
            full_summary: None,
            embedding: None,
            importance: 0.5, // 默认中等重要性
            confidence: 0.5,
            source,
            source_id: None,
            parent_id: None,
            related_ids: Vec::new(),
            tags: Vec::new(),
            topics: Vec::new(),
            created_at: now,
            updated_at: now,
            accessed_at: now,
            expires_at: None,
            status: MemoryStatus::Active,
            version: 1,
            keywords: Vec::new(),
        }
    }

    /// 标记为已访问
    pub fn mark_accessed(&mut self) {
        self.accessed_at = Utc::now();
    }

    /// 标记为已归档
    pub fn archive(&mut self) {
        self.status = MemoryStatus::Archived;
        self.updated_at = Utc::now();
        self.version += 1;
    }

    /// 恢复归档的记忆
    pub fn restore(&mut self) {
        self.status = MemoryStatus::Active;
        self.updated_at = Utc::now();
        self.version += 1;
    }

    /// 软删除
    pub fn soft_delete(&mut self) {
        self.status = MemoryStatus::Deleted;
        self.updated_at = Utc::now();
        self.version += 1;
    }

    /// 永久删除（硬删除）
    pub fn is_deleted(&self) -> bool {
        self.status == MemoryStatus::Deleted
    }

    /// 记忆是否过期
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            return Utc::now() > expires_at;
        }
        false
    }

    /// 添加相关记忆
    pub fn add_related(&mut self, memory_id: &str) {
        if !self.related_ids.contains(&memory_id.to_string()) {
            self.related_ids.push(memory_id.to_string());
        }
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: &str) {
        let tag = tag.to_lowercase();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// 添加主题
    pub fn add_topic(&mut self, topic: &str) {
        let topic = topic.to_lowercase();
        if !self.topics.contains(&topic) {
            self.topics.push(topic);
        }
    }

    /// 检查记忆是否可检索
    pub fn is_retrievable(&self) -> bool {
        self.status == MemoryStatus::Active && !self.is_expired()
    }
}

/// 记忆查询条件
#[derive(Debug, Clone, Default)]
pub struct MemoryQuery {
    /// 用户 ID
    pub user_id: Option<String>,

    /// 记忆类型筛选
    pub memory_types: Vec<MemoryType>,

    /// 标签筛选
    pub tags: Vec<String>,

    /// 主题筛选
    pub topics: Vec<String>,

    /// 时间范围 - 开始
    pub created_after: Option<DateTime<Utc>>,

    /// 时间范围 - 结束
    pub created_before: Option<DateTime<Utc>>,

    /// 最小重要性
    pub min_importance: Option<f32>,

    /// 状态筛选
    pub statuses: Vec<MemoryStatus>,

    /// 来源筛选
    pub sources: Vec<MemorySource>,

    /// 关键词搜索
    pub keyword: Option<String>,

    /// 分页
    pub page: u32,
    pub page_size: u32,
}

impl MemoryQuery {
    /// 创建新查询
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置用户筛选
    pub fn for_user(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    /// 设置记忆类型筛选
    pub fn with_types(mut self, types: &[MemoryType]) -> Self {
        self.memory_types = types.to_vec();
        self
    }

    /// 设置标签筛选
    pub fn with_tags(mut self, tags: &[&str]) -> Self {
        self.tags = tags.iter().map(|s| s.to_string()).collect();
        self
    }

    /// 设置时间范围
    pub fn with_time_range(
        mut self,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
    ) -> Self {
        self.created_after = start;
        self.created_before = end;
        self
    }

    /// 设置最小重要性
    pub fn with_min_importance(mut self, importance: f32) -> Self {
        self.min_importance = Some(importance);
        self
    }

    /// 设置分页
    pub fn with_pagination(mut self, page: u32, page_size: u32) -> Self {
        self.page = page.max(1);
        self.page_size = page_size.clamp(1, 100);
        self
    }

    /// 获取偏移量
    pub fn offset(&self) -> u64 {
        ((self.page - 1) * self.page_size) as u64
    }
}

/// 记忆搜索结果
#[derive(Debug, Clone)]
pub struct MemorySearchResult {
    /// 记忆
    pub memory: Memory,

    /// 综合评分
    pub score: f32,

    /// 语义相似度评分
    pub semantic_score: Option<f32>,

    /// 时序评分
    pub temporal_score: f32,

    /// 上下文评分
    pub context_score: Option<f32>,

    /// 匹配原因
    pub match_reasons: Vec<String>,
}

/// 记忆统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// 用户 ID
    pub user_id: String,

    /// 总记忆数
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
    pub avg_importance: f32,
    pub high_importance_count: u64, // > 0.7

    /// 存储大小（字节）
    pub storage_size_bytes: u64,
}

/// 记忆版本历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryVersion {
    /// 版本号
    pub version: u32,

    /// 内容快照
    pub content: String,

    /// 重要性评分
    pub importance: f32,

    /// 创建时间
    pub created_at: DateTime<Utc>,

    /// 变更原因
    pub change_reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_creation() {
        let memory = Memory::new(
            "user_123",
            MemoryType::Episodic,
            "用户说他们喜欢 Rust 编程",
            MemorySource::Conversation,
        );

        assert_eq!(memory.user_id, "user_123");
        assert_eq!(memory.memory_type, MemoryType::Episodic);
        assert_eq!(memory.source, MemorySource::Conversation);
        assert_eq!(memory.status, MemoryStatus::Active);
        assert!(!memory.id.is_empty());
    }

    #[test]
    fn test_memory_operations() {
        let mut memory = Memory::new(
            "user_123",
            MemoryType::Episodic,
            "测试内容",
            MemorySource::Conversation,
        );

        // 测试归档
        memory.archive();
        assert_eq!(memory.status, MemoryStatus::Archived);
        assert!(memory.version > 1);

        // 测试恢复
        memory.restore();
        assert_eq!(memory.status, MemoryStatus::Active);

        // 测试软删除
        memory.soft_delete();
        assert!(memory.is_deleted());

        // 测试标签和主题
        memory.add_tag("rust");
        memory.add_tag("rust"); // 重复添加
        memory.add_topic("编程");

        assert_eq!(memory.tags.len(), 1);
        assert_eq!(memory.tags[0], "rust");
        assert_eq!(memory.topics.len(), 1);
    }

    #[test]
    fn test_memory_query() {
        let query = MemoryQuery::new()
            .for_user("user_123")
            .with_types(&[MemoryType::Episodic])
            .with_tags(&["rust", "编程"])
            .with_min_importance(0.5)
            .with_pagination(1, 20);

        assert_eq!(query.user_id, Some("user_123".to_string()));
        assert_eq!(query.memory_types.len(), 1);
        assert_eq!(query.tags.len(), 2);
        assert_eq!(query.page, 1);
        assert_eq!(query.page_size, 20);
        assert_eq!(query.offset(), 0);
    }
}
