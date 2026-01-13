use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Custom deserializer for SurrealDB record IDs
/// Handles both plain strings and Thing objects (SurrealDB 2.x format)
fn deserialize_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    // First try to deserialize as a plain string
    let value = serde_json::Value::deserialize(deserializer)?;

    // If it's a string, return it
    if let Some(s) = value.as_str() {
        // Handle SurrealDB record ID format "session:⟨uuid⟩"
        if let Some(uuid) = s.split("⟨").nth(1) {
            if let Some(uuid) = uuid.split("⟩").next() {
                return Ok(uuid.to_string());
            }
        }
        return Ok(s.to_string());
    }

    // If it's a map/object, try to extract the id field (Thing format)
    if let Some(map) = value.as_object() {
        if let Some(_tb) = map.get("tb").and_then(|v| v.as_str()) {
            if let Some(id_val) = map.get("id") {
                match id_val {
                    serde_json::Value::String(s) => {
                        if let Some(uuid) = s.split("⟨").nth(1) {
                            if let Some(uuid) = uuid.split("⟩").next() {
                                return Ok(uuid.to_string());
                            }
                        }
                        return Ok(s.clone());
                    }
                    serde_json::Value::Number(n) => return Ok(n.to_string()),
                    _ => return Ok(id_val.to_string()),
                }
            }
        }
    }

    Err(serde::de::Error::custom(
        "Expected string or SurrealDB record ID",
    ))
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
pub struct Session {
    /// 会话唯一标识
    #[serde(deserialize_with = "deserialize_id")]
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

    /// 会话状态 (Active, Paused, Archived, Deleted)
    #[serde(default = "default_status")]
    pub status: String,

    /// 会话配置
    #[serde(default)]
    pub config: SessionConfig,

    /// 统计信息
    #[serde(default)]
    pub stats: SessionStats,

    /// 元数据
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

fn default_status() -> String {
    "Active".to_string()
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
            status: "Active".to_string(),
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

    /// 检查会话是否处于指定状态
    pub fn is_status(&self, status: &str) -> bool {
        self.status == status
    }

    /// 设置会话状态
    pub fn set_status(&mut self, status: &str) {
        self.status = status.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json;

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.summary_limit, 0);
        assert_eq!(config.max_turns, 0);
        assert!(!config.semantic_search_enabled);
        assert!(!config.auto_summarize);
    }

    #[test]
    fn test_session_stats_default() {
        let stats = SessionStats::default();
        assert_eq!(stats.total_turns, 0);
        assert_eq!(stats.total_tokens, 0);
        assert_eq!(stats.storage_size, 0);
        assert!(stats.last_indexed_at.is_none());
    }

    #[test]
    fn test_deserialize_id_with_plain_string() {
        let json = r#"{"id": "session:abc123", "tenant_id": "test", "name": "test", "created_at": "2024-01-15T10:30:00Z", "last_active_at": "2024-01-15T10:30:00Z", "status": "active", "config": {}, "stats": {}}"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let deserialized: Session = serde_json::from_value(value).unwrap();
        assert_eq!(deserialized.id, "session:abc123");
    }

    #[test]
    fn test_deserialize_id_with_thing_format() {
        // SurrealDB 2.x returns IDs in format "session:⟨uuid⟩"
        let json = r#"{"id": "session:⟨abc123-uuid⟩", "tenant_id": "test", "name": "test", "created_at": "2024-01-15T10:30:00Z", "last_active_at": "2024-01-15T10:30:00Z", "status": "active", "config": {}, "stats": {}}"#;
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        let deserialized: Session = serde_json::from_value(value).unwrap();
        // The UUID should be extracted from the angle brackets
        assert!(deserialized.id.contains("abc123"));
    }

    #[test]
    fn test_session_serialization_roundtrip() {
        let session = Session {
            id: "session:test123".to_string(),
            tenant_id: "tenant_1".to_string(),
            name: "Test Session".to_string(),
            description: Some("A test session".to_string()),
            created_at: Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap(),
            last_active_at: Utc.with_ymd_and_hms(2024, 1, 15, 11, 0, 0).unwrap(),
            status: "Active".to_string(),
            config: SessionConfig {
                summary_limit: 10,
                max_turns: 100,
                ..Default::default()
            },
            stats: SessionStats {
                total_turns: 5,
                total_tokens: 1500,
                storage_size: 8192,
                last_indexed_at: None,
            },
            metadata: HashMap::new(),
        };

        let serialized = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&serialized).unwrap();

        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.tenant_id, deserialized.tenant_id);
        assert_eq!(session.name, deserialized.name);
        assert_eq!(session.description, deserialized.description);
        assert_eq!(session.status, deserialized.status);
        assert_eq!(
            session.config.summary_limit,
            deserialized.config.summary_limit
        );
    }

    #[test]
    fn test_session_with_null_description() {
        let json = r#"{
            "id": "session:test",
            "tenant_id": "test",
            "name": "test",
            "description": null,
            "created_at": "2024-01-15T10:30:00Z",
            "last_active_at": "2024-01-15T10:30:00Z",
            "status": "active",
            "config": {},
            "stats": {}
        }"#;
        let deserialized: Session = serde_json::from_str(json).unwrap();
        assert!(deserialized.description.is_none());
    }

    #[test]
    fn test_session_config_serialization() {
        let config = SessionConfig {
            summary_limit: 10,
            index_refresh_interval: 60000,
            semantic_search_enabled: true,
            auto_summarize: false,
            max_turns: 100,
        };

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: SessionConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config.summary_limit, deserialized.summary_limit);
        assert_eq!(
            config.semantic_search_enabled,
            deserialized.semantic_search_enabled
        );
    }

    #[test]
    fn test_session_touch() {
        let mut session = Session::new("tenant_1", "Test");
        let original_active = session.last_active_at;

        // Small delay to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        session.touch();
        assert!(session.last_active_at > original_active);
    }

    #[test]
    fn test_session_increment_turns() {
        let mut session = Session::new("tenant_1", "Test");
        assert_eq!(session.stats.total_turns, 0);
        assert_eq!(session.stats.total_tokens, 0);

        session.increment_turns(100);
        assert_eq!(session.stats.total_turns, 1);
        assert_eq!(session.stats.total_tokens, 100);

        session.increment_turns(50);
        assert_eq!(session.stats.total_turns, 2);
        assert_eq!(session.stats.total_tokens, 150);
    }

    #[test]
    fn test_session_status_methods() {
        let mut session = Session::new("tenant_1", "Test");

        assert!(session.is_status("Active"));
        assert!(!session.is_status("Archived"));

        session.set_status("Paused");
        assert!(session.is_status("Paused"));
        assert!(!session.is_status("Active"));
    }
}
