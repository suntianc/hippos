use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// 消息类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    /// 用户消息
    User,
    /// 助手消息
    Assistant,
    /// 系统消息
    System,
}

/// 内容状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentStatus {
    /// 待处理
    Pending,
    /// 已索引
    Indexed,
    /// 已归档
    Archived,
    /// 处理中
    Processing,
}

/// 元数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct TurnMetadata {
    /// 精确到毫秒的时间戳
    pub timestamp: DateTime<Utc>,

    /// 用户标识
    pub user_id: Option<String>,

    /// 消息类型
    pub message_type: MessageType,

    /// 消息角色
    pub role: Option<String>,

    /// 模型名称（如果是助手消息）
    pub model: Option<String>,

    /// Token 数量
    pub token_count: Option<u64>,

    /// 自定义元数据
    pub custom: HashMap<String, String>,
}

/// 脱水后的摘要信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DehydratedData {
    /// 50-100字的极简概括
    pub gist: String,

    /// 核心讨论主题
    pub topics: Vec<String>,

    /// 关键词标签
    pub tags: Vec<String>,

    /// 语义向量（384或768维度）
    pub embedding: Option<Vec<f32>>,

    /// 摘要生成时间
    pub generated_at: DateTime<Utc>,

    /// 生成摘要的模型
    pub generator: Option<String>,
}

/// 对话轮次实体
///
/// 存储每一轮对话的完整信息，是系统的核心数据实体。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "TurnHelper", into = "TurnHelper")]
pub struct Turn {
    /// 轮次唯一标识
    pub id: String,

    /// 所属会话ID
    pub session_id: String,

    /// 轮次序号
    pub turn_number: u64,

    /// 原始对话内容（Markdown格式）
    pub raw_content: String,

    /// 元数据
    pub metadata: TurnMetadata,

    /// 脱水后的摘要信息
    pub dehydrated: Option<DehydratedData>,

    /// 内容状态
    pub status: ContentStatus,

    /// 父轮次ID（用于对话链）
    pub parent_id: Option<String>,

    /// 子轮次ID列表
    pub children_ids: Vec<String>,
}

impl Turn {
    /// 创建新轮次
    pub fn new(session_id: &str, turn_number: u64, content: &str) -> Self {
        let now = Utc::now();
        Self {
            id: format!("turn_{}_{}", session_id, Uuid::new_v4()),
            session_id: session_id.to_string(),
            turn_number,
            raw_content: content.to_string(),
            metadata: TurnMetadata {
                timestamp: now,
                user_id: None,
                message_type: MessageType::User,
                role: None,
                model: None,
                token_count: None,
                custom: HashMap::new(),
            },
            dehydrated: None,
            status: ContentStatus::Pending,
            parent_id: None,
            children_ids: Vec::new(),
        }
    }

    /// 标记为已索引
    pub fn mark_indexed(&mut self) {
        self.status = ContentStatus::Indexed;
    }

    /// 标记为处理中
    pub fn mark_processing(&mut self) {
        self.status = ContentStatus::Processing;
    }

    /// 获取内容长度
    pub fn content_length(&self) -> usize {
        self.raw_content.len()
    }

    /// 估算 Token 数量（粗略估算）
    pub fn estimated_tokens(&self) -> u64 {
        (self.raw_content.len() / 4) as u64
    }
}

/// 轮次序列化辅助
#[derive(Serialize, Deserialize)]
struct TurnHelper {
    id: String,
    session_id: String,
    turn_number: u64,
    raw_content: String,
    metadata: TurnMetadata,
    dehydrated: Option<DehydratedData>,
    status: ContentStatus,
    parent_id: Option<String>,
    children_ids: Vec<String>,
}

impl From<TurnHelper> for Turn {
    fn from(helper: TurnHelper) -> Self {
        Turn {
            id: helper.id,
            session_id: helper.session_id,
            turn_number: helper.turn_number,
            raw_content: helper.raw_content,
            metadata: helper.metadata,
            dehydrated: helper.dehydrated,
            status: helper.status,
            parent_id: helper.parent_id,
            children_ids: helper.children_ids,
        }
    }
}

impl From<Turn> for TurnHelper {
    fn from(turn: Turn) -> Self {
        TurnHelper {
            id: turn.id,
            session_id: turn.session_id,
            turn_number: turn.turn_number,
            raw_content: turn.raw_content,
            metadata: turn.metadata,
            dehydrated: turn.dehydrated,
            status: turn.status,
            parent_id: turn.parent_id,
            children_ids: turn.children_ids,
        }
    }
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::User
    }
}

impl Default for ContentStatus {
    fn default() -> Self {
        ContentStatus::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use serde_json;

    #[test]
    fn test_message_type_default() {
        assert_eq!(MessageType::default(), MessageType::User);
    }

    #[test]
    fn test_content_status_default() {
        assert_eq!(ContentStatus::default(), ContentStatus::Pending);
    }

    #[test]
    fn test_turn_metadata_default() {
        let metadata = TurnMetadata::default();
        assert!(metadata.user_id.is_none());
        assert!(metadata.role.is_none());
        assert!(metadata.model.is_none());
        assert!(metadata.token_count.is_none());
        assert!(metadata.custom.is_empty());
        assert_eq!(metadata.message_type, MessageType::User);
    }

    #[test]
    fn test_dehydrated_data_default() {
        let data = DehydratedData::default();
        assert!(data.gist.is_empty());
        assert!(data.topics.is_empty());
        assert!(data.tags.is_empty());
        assert!(data.embedding.is_none());
        assert!(data.generator.is_none());
    }

    #[test]
    fn test_turn_serialization_roundtrip() {
        let turn = Turn {
            id: "turn:test123".to_string(),
            session_id: "session:abc".to_string(),
            turn_number: 1,
            raw_content: "Hello, world!".to_string(),
            metadata: TurnMetadata {
                timestamp: Utc::now(),
                user_id: Some("user123".to_string()),
                message_type: MessageType::User,
                role: Some("user".to_string()),
                model: None,
                token_count: Some(50),
                custom: HashMap::new(),
            },
            dehydrated: None,
            status: ContentStatus::Pending,
            parent_id: None,
            children_ids: vec![],
        };

        let serialized = serde_json::to_string(&turn).unwrap();
        let deserialized: Turn = serde_json::from_str(&serialized).unwrap();

        assert_eq!(turn.id, deserialized.id);
        assert_eq!(turn.session_id, deserialized.session_id);
        assert_eq!(turn.turn_number, deserialized.turn_number);
        assert_eq!(turn.raw_content, deserialized.raw_content);
    }

    #[test]
    fn test_turn_with_children() {
        let turn = Turn {
            id: "turn:parent".to_string(),
            session_id: "session:abc".to_string(),
            turn_number: 1,
            raw_content: "Parent turn".to_string(),
            metadata: TurnMetadata::default(),
            dehydrated: None,
            status: ContentStatus::Indexed,
            parent_id: None,
            children_ids: vec!["turn:child1".to_string(), "turn:child2".to_string()],
        };

        assert_eq!(turn.children_ids.len(), 2);
        assert!(turn.children_ids.contains(&"turn:child1".to_string()));
    }

    #[test]
    fn test_turn_helper_conversion() {
        let turn = Turn {
            id: "turn:test".to_string(),
            session_id: "session:abc".to_string(),
            turn_number: 5,
            raw_content: "Test content".to_string(),
            metadata: TurnMetadata::default(),
            dehydrated: Some(DehydratedData {
                gist: "Test gist".to_string(),
                topics: vec!["test".to_string()],
                tags: vec!["tag1".to_string()],
                embedding: None,
                generated_at: Utc::now(),
                generator: Some("test".to_string()),
            }),
            status: ContentStatus::Indexed,
            parent_id: Some("turn:parent".to_string()),
            children_ids: vec!["turn:child".to_string()],
        };

        let helper: TurnHelper = turn.clone().into();
        assert_eq!(helper.id, "turn:test");
        assert_eq!(helper.session_id, "session:abc");
        assert_eq!(helper.turn_number, 5);
        assert!(helper.dehydrated.is_some());
        assert_eq!(helper.parent_id, Some("turn:parent".to_string()));
        assert_eq!(helper.children_ids.len(), 1);
    }

    #[test]
    fn test_message_type_serialization() {
        let user = serde_json::to_string(&MessageType::User).unwrap();
        let assistant = serde_json::to_string(&MessageType::Assistant).unwrap();
        let system = serde_json::to_string(&MessageType::System).unwrap();

        // Enum variants are serialized as lowercase strings
        assert!(user.contains("User") || user.contains("user"));
        assert!(assistant.contains("Assistant") || assistant.contains("assistant"));
        assert!(system.contains("System") || system.contains("system"));
    }

    #[test]
    fn test_content_status_serialization() {
        let pending = serde_json::to_string(&ContentStatus::Pending).unwrap();
        let indexed = serde_json::to_string(&ContentStatus::Indexed).unwrap();
        let archived = serde_json::to_string(&ContentStatus::Archived).unwrap();
        let processing = serde_json::to_string(&ContentStatus::Processing).unwrap();

        // Just verify they serialize to strings
        assert!(pending.starts_with('"'));
        assert!(indexed.starts_with('"'));
        assert!(archived.starts_with('"'));
        assert!(processing.starts_with('"'));
    }
}
