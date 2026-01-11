use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 索引记录实体
///
/// 脱水处理后的轻量级数据结构，专门为高速检索优化。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "IndexRecordHelper", into = "IndexRecordHelper")]
pub struct IndexRecord {
    /// 关联的对话轮次ID
    pub turn_id: String,

    /// 会话ID（用于快速过滤）
    pub session_id: String,

    /// 摘要文本
    pub gist: String,

    /// 主题列表
    pub topics: Vec<String>,

    /// 标签列表
    pub tags: Vec<String>,

    /// 时间戳
    pub timestamp: DateTime<Utc>,

    /// 向量标识（指向向量存储的具体位置）
    pub vector_id: String,

    /// 检索相关性评分（预计算）
    pub relevance_score: Option<f32>,

    /// 轮次序号（用于排序）
    pub turn_number: u64,
}

impl IndexRecord {
    /// 创建新索引记录
    pub fn new(
        turn_id: &str,
        session_id: &str,
        gist: &str,
        timestamp: DateTime<Utc>,
        turn_number: u64,
    ) -> Self {
        Self {
            turn_id: turn_id.to_string(),
            session_id: session_id.to_string(),
            gist: gist.to_string(),
            topics: Vec::new(),
            tags: Vec::new(),
            timestamp,
            vector_id: format!("vec_{}", turn_id),
            relevance_score: None,
            turn_number,
        }
    }

    /// 添加主题
    pub fn add_topic(&mut self, topic: &str) {
        if !self.topics.contains(&topic.to_string()) {
            self.topics.push(topic.to_string());
        }
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: &str) {
        if !self.tags.contains(&tag.to_string()) {
            self.tags.push(tag.to_string());
        }
    }

    /// 添加多个标签
    pub fn add_tags(&mut self, tags: &[&str]) {
        for tag in tags {
            self.add_tag(tag);
        }
    }
}

/// 索引记录序列化辅助
#[derive(Serialize, Deserialize)]
struct IndexRecordHelper {
    turn_id: String,
    session_id: String,
    gist: String,
    topics: Vec<String>,
    tags: Vec<String>,
    timestamp: DateTime<Utc>,
    vector_id: String,
    relevance_score: Option<f32>,
    turn_number: u64,
}

impl From<IndexRecordHelper> for IndexRecord {
    fn from(helper: IndexRecordHelper) -> Self {
        IndexRecord {
            turn_id: helper.turn_id,
            session_id: helper.session_id,
            gist: helper.gist,
            topics: helper.topics,
            tags: helper.tags,
            timestamp: helper.timestamp,
            vector_id: helper.vector_id,
            relevance_score: helper.relevance_score,
            turn_number: helper.turn_number,
        }
    }
}

impl From<IndexRecord> for IndexRecordHelper {
    fn from(record: IndexRecord) -> Self {
        IndexRecordHelper {
            turn_id: record.turn_id,
            session_id: record.session_id,
            gist: record.gist,
            topics: record.topics,
            tags: record.tags,
            timestamp: record.timestamp,
            vector_id: record.vector_id,
            relevance_score: record.relevance_score,
            turn_number: record.turn_number,
        }
    }
}
