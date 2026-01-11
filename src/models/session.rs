use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 会话状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    /// 活跃状态
    Active,
    /// 已暂停
    Paused,
    /// 已归档
    Archived,
    /// 已删除
    Deleted,
}

/// 会话配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SessionConfig {
    /// 保留的摘要数量
    pub summary_limit: usize,
    /// 索引刷新间隔（毫秒）
    pub index_refresh_interval: u64,
    /// 是否启用语义搜索
    pub semantic_search_enabled: bool,
    /// 是否启用自动摘要
    pub auto_summarize: bool,
    /// 最大轮次数量（0 表示无限制）
    pub max_turns: usize,
}

/// 会话统计信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SessionStats {
    /// 总轮次数
    pub total_turns: u64,
    /// 总 Token 数
    pub total_tokens: u64,
    /// 存储大小（字节）
    pub storage_size: u64,
    /// 最后索引时间
    pub last_indexed_at: Option<DateTime<Utc>>,
}

/// 会话实体
///
/// 承载会话的元数据信息，是数据访问的根节点。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "SessionHelper", into = "SessionHelper")]
pub struct Session {
    /// 会话唯一标识
    pub id: String,

    /// 所属租户/用户标识
    pub tenant_id: String,

    /// 会话名称
    pub name: String,

    /// 会话描述
    pub description: Option<String>,

    /// 会话创建时间
    pub created_at: DateTime<Utc>,

    /// 最后活跃时间
    pub last_active_at: DateTime<Utc>,

    /// 会话状态
    pub status: SessionStatus,

    /// 会话配置
    pub config: SessionConfig,

    /// 统计信息
    pub stats: SessionStats,

    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl Session {
    /// 创建新会话
    pub fn new(tenant_id: &str, name: &str) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            description: None,
            created_at: now,
            last_active_at: now,
            status: SessionStatus::Active,
            config: SessionConfig::default(),
            stats: SessionStats::default(),
            metadata: HashMap::new(),
        }
    }

    /// 更新最后活跃时间
    pub fn touch(&mut self) {
        self.last_active_at = Utc::now();
    }

    /// 增加轮次统计
    pub fn increment_turns(&mut self, tokens: u64) {
        self.stats.total_turns += 1;
        self.stats.total_tokens += tokens;
        self.touch();
    }

    /// 检查是否允许更多轮次
    pub fn can_add_turn(&self) -> bool {
        self.config.max_turns == 0 || self.stats.total_turns < self.config.max_turns as u64
    }
}

/// 会话序列化辅助
#[derive(Serialize, Deserialize)]
struct SessionHelper {
    id: String,
    tenant_id: String,
    name: String,
    description: Option<String>,
    created_at: DateTime<Utc>,
    last_active_at: DateTime<Utc>,
    status: SessionStatus,
    config: SessionConfig,
    stats: SessionStats,
    metadata: HashMap<String, String>,
}

impl From<SessionHelper> for Session {
    fn from(helper: SessionHelper) -> Self {
        Session {
            id: helper.id,
            tenant_id: helper.tenant_id,
            name: helper.name,
            description: helper.description,
            created_at: helper.created_at,
            last_active_at: helper.last_active_at,
            status: helper.status,
            config: helper.config,
            stats: helper.stats,
            metadata: helper.metadata,
        }
    }
}

impl From<Session> for SessionHelper {
    fn from(session: Session) -> Self {
        SessionHelper {
            id: session.id,
            tenant_id: session.tenant_id,
            name: session.name,
            description: session.description,
            created_at: session.created_at,
            last_active_at: session.last_active_at,
            status: session.status,
            config: session.config,
            stats: session.stats,
            metadata: session.metadata,
        }
    }
}

impl Default for SessionStatus {
    fn default() -> Self {
        SessionStatus::Active
    }
}
