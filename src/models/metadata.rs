use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// 元数据条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetadataEntry {
    /// 键
    pub key: String,
    /// 值
    pub value: String,
    /// 值类型
    pub value_type: MetadataType,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl MetadataEntry {
    /// 创建新元数据条目
    pub fn new(key: &str, value: &str, value_type: MetadataType) -> Self {
        let now = chrono::Utc::now();
        Self {
            key: key.to_string(),
            value: value.to_string(),
            value_type,
            created_at: now,
            updated_at: now,
        }
    }

    /// 更新值
    pub fn update(&mut self, value: &str) {
        self.value = value.to_string();
        self.updated_at = chrono::Utc::now();
    }
}

/// 元数据类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetadataType {
    /// 字符串
    String,
    /// 数字
    Number,
    /// 布尔值
    Boolean,
    /// JSON
    Json,
    /// 数组
    Array,
}

impl Default for MetadataType {
    fn default() -> Self {
        MetadataType::String
    }
}

impl fmt::Display for MetadataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetadataType::String => write!(f, "string"),
            MetadataType::Number => write!(f, "number"),
            MetadataType::Boolean => write!(f, "boolean"),
            MetadataType::Json => write!(f, "json"),
            MetadataType::Array => write!(f, "array"),
        }
    }
}

/// 会话元数据集合
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SessionMetadata {
    /// 元数据条目
    pub entries: HashMap<String, MetadataEntry>,
    /// 版本号
    pub version: u64,
}

impl SessionMetadata {
    /// 创建新会话元数据
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            version: 1,
        }
    }

    /// 添加字符串元数据
    pub fn add_string(&mut self, key: &str, value: &str) {
        let entry = MetadataEntry::new(key, value, MetadataType::String);
        self.entries.insert(key.to_string(), entry);
        self.version += 1;
    }

    /// 添加数字元数据
    pub fn add_number(&mut self, key: &str, value: &str) {
        let entry = MetadataEntry::new(key, value, MetadataType::Number);
        self.entries.insert(key.to_string(), entry);
        self.version += 1;
    }

    /// 添加布尔元数据
    pub fn add_boolean(&mut self, key: &str, value: bool) {
        let entry = MetadataEntry::new(key, &value.to_string(), MetadataType::Boolean);
        self.entries.insert(key.to_string(), entry);
        self.version += 1;
    }

    /// 获取字符串值
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.entries.get(key).map(|e| e.value.as_str())
    }

    /// 获取数字值
    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.entries.get(key).and_then(|e| e.value.parse().ok())
    }

    /// 获取布尔值
    pub fn get_boolean(&self, key: &str) -> Option<bool> {
        self.entries.get(key).and_then(|e| e.value.parse().ok())
    }

    /// 检查键是否存在
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// 移除元数据
    pub fn remove(&mut self, key: &str) -> Option<MetadataEntry> {
        let removed = self.entries.remove(key);
        if removed.is_some() {
            self.version += 1;
        }
        removed
    }

    /// 清空所有元数据
    pub fn clear(&mut self) {
        self.entries.clear();
        self.version += 1;
    }

    /// 获取条目数量
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// 转换为 HashMap（用于导出）
impl From<SessionMetadata> for HashMap<String, String> {
    fn from(meta: SessionMetadata) -> Self {
        meta.entries
            .into_iter()
            .map(|(k, v)| (k, v.value))
            .collect()
    }
}

/// 从 HashMap 导入
impl From<HashMap<String, String>> for SessionMetadata {
    fn from(map: HashMap<String, String>) -> Self {
        let mut meta = SessionMetadata::new();
        for (key, value) in map {
            meta.add_string(&key, &value);
        }
        meta
    }
}
