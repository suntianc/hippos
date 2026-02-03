//! 数据转换模块
//!
//! 将 Surrealdb 格式的数据转换为 ArangoDB 格式。
//! 处理 ID 格式转换、关系映射等。

use crate::migration::{MigrationError, export::*};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 转换配置
#[derive(Debug, Clone)]
pub struct TransformConfig {
    pub source_dir: PathBuf,
    pub target_dir: PathBuf,
    pub collection_prefix: String,
    pub generate_edges: bool,
}

impl Default for TransformConfig {
    fn default() -> Self {
        Self {
            source_dir: PathBuf::from("./migration_export"),
            target_dir: PathBuf::from("./migration_transformed"),
            collection_prefix: "hippos_".to_string(),
            generate_edges: true,
        }
    }
}

/// 转换后的会话数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformedSession {
    pub _key: String,
    pub tenant_id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: String,
    pub last_active_at: String,
    pub status: String,
    pub config: serde_json::Value,
    pub stats: serde_json::Value,
    pub metadata: serde_json::Value,
    pub _id: String,
}

/// 转换后的轮次数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformedTurn {
    pub _key: String,
    pub session_key: String,
    pub turn_number: u64,
    pub raw_content: String,
    pub metadata: serde_json::Value,
    pub dehydrated: Option<serde_json::Value>,
    pub status: String,
    pub parent_key: Option<String>,
    pub children_keys: Vec<String>,
    pub _id: String,
}

/// 转换后的索引记录数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformedIndexRecord {
    pub _key: String,
    pub turn_key: String,
    pub session_key: String,
    pub tenant_id: String,
    pub gist: String,
    pub topics: Vec<String>,
    pub tags: Vec<String>,
    pub timestamp: String,
    pub vector_id: String,
    pub relevance_score: Option<f32>,
    pub turn_number: u64,
    pub _id: String,
}

/// 转换后的边数据 (Session -> Turn)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTurnEdge {
    pub _key: String,
    pub _from: String,
    pub _to: String,
    pub turn_number: u64,
    pub _id: String,
    pub _from_string: String,
    pub _to_string: String,
}

/// 转换后的边数据 (Turn -> Turn 父子关系)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnParentEdge {
    pub _key: String,
    pub _from: String,
    pub _to: String,
    pub relationship: String,
    pub _id: String,
    pub _from_string: String,
    pub _to_string: String,
}

/// 转换后的边数据 (Turn -> IndexRecord)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnIndexEdge {
    pub _key: String,
    pub _from: String,
    pub _to: String,
    pub indexed_at: String,
    pub _id: String,
    pub _from_string: String,
    pub _to_string: String,
}

/// 转换状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformState {
    pub sessions_processed: usize,
    pub turns_processed: usize,
    pub index_records_processed: usize,
    pub edges_generated: usize,
    pub errors: Vec<MigrationError>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for TransformState {
    fn default() -> Self {
        Self {
            sessions_processed: 0,
            turns_processed: 0,
            index_records_processed: 0,
            edges_generated: 0,
            errors: Vec::new(),
            started_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
        }
    }
}

/// 数据转换器
pub struct DataTransformer {
    config: TransformConfig,
    state: TransformState,
    id_mapping: HashMap<String, String>,
}

impl DataTransformer {
    /// 创建新的转换器
    pub fn new(config: TransformConfig) -> Self {
        Self {
            config,
            state: TransformState::default(),
            id_mapping: HashMap::new(),
        }
    }

    /// 运行完整转换
    pub async fn transform_all(&mut self) -> Result<TransformState, String> {
        // 创建输出目录
        self.init_output_dir().await?;

        // 转换会话
        self.transform_sessions().await?;

        // 转换轮次
        self.transform_turns().await?;

        // 转换索引记录
        self.transform_index_records().await?;

        // 生成边集合
        if self.config.generate_edges {
            self.generate_edges().await?;
        }

        // 保存元数据
        self.save_metadata().await?;

        Ok(self.state.clone())
    }

    /// 初始化输出目录
    async fn init_output_dir(&self) -> Result<(), String> {
        let target_dir = &self.config.target_dir;

        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir).map_err(|e| format!("清理输出目录失败: {}", e))?;
        }

        std::fs::create_dir_all(target_dir).map_err(|e| format!("创建输出目录失败: {}", e))?;

        // 创建子目录
        for subdir in ["sessions", "turns", "index_records", "edges", "metadata"] {
            let subdir = target_dir.join(subdir);
            std::fs::create_dir_all(&subdir)
                .map_err(|e| format!("创建子目录 {} 失败: {}", subdir.display(), e))?;
        }

        Ok(())
    }

    /// 转换会话数据
    async fn transform_sessions(&mut self) -> Result<(), String> {
        let source_dir = self.config.source_dir.join("sessions");
        let target_dir = self.config.target_dir.join("sessions");

        // 读取源文件
        let entries =
            std::fs::read_dir(&source_dir).map_err(|e| format!("读取源目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| format!("读取文件失败 {}: {}", path.display(), e))?;

                let sessions: Vec<ExportedSession> =
                    serde_json::from_str(&content).map_err(|e| format!("解析 JSON 失败: {}", e))?;

                let mut transformed_sessions = Vec::new();

                for exported in sessions {
                    match self.transform_session(&exported) {
                        Ok(transformed) => {
                            transformed_sessions.push(transformed);
                            self.state.sessions_processed += 1;
                        }
                        Err(e) => {
                            self.state.errors.push(MigrationError::new(
                                "transform_error",
                                &e,
                                Some(&exported.original_id),
                            ));
                        }
                    }
                }

                // 写入转换后的文件
                let target_path = target_dir.join(path.file_name().unwrap());
                let output = serde_json::to_string_pretty(&transformed_sessions)
                    .map_err(|e| format!("序列化失败: {}", e))?;
                std::fs::write(&target_path, output)
                    .map_err(|e| format!("写入失败 {}: {}", target_path.display(), e))?;
            }

            self.state.last_updated = chrono::Utc::now();
        }

        println!("转换会话: {} 个成功", self.state.sessions_processed);

        Ok(())
    }

    /// 转换单个会话
    fn transform_session(&self, exported: &ExportedSession) -> Result<TransformedSession, String> {
        let original_id = &exported.original_id;
        let session = &exported.data;

        // 转换 ID: "session:⟨uuid⟩" -> "uuid"
        let key = Self::extract_uuid(original_id, "session")
            .ok_or_else(|| format!("无法解析会话 ID: {}", original_id))?;

        // 转换时间格式
        let created_at = session.created_at.to_rfc3339();
        let last_active_at = session.last_active_at.to_rfc3339();

        // 序列化复杂字段
        let config =
            serde_json::to_value(&session.config).map_err(|e| format!("序列化配置失败: {}", e))?;
        let stats =
            serde_json::to_value(&session.stats).map_err(|e| format!("序列化统计失败: {}", e))?;
        let metadata = serde_json::to_value(&session.metadata)
            .map_err(|e| format!("序列化元数据失败: {}", e))?;

        let collection = format!("{}{}", self.config.collection_prefix, "sessions");
        let id = format!("{}/{}", collection, key);

        Ok(TransformedSession {
            _key: key,
            tenant_id: session.tenant_id.clone(),
            name: session.name.clone(),
            description: session.description.clone(),
            created_at,
            last_active_at,
            status: session.status.clone(),
            config,
            stats,
            metadata,
            _id: id,
        })
    }

    /// 转换轮次数据
    async fn transform_turns(&mut self) -> Result<(), String> {
        let source_dir = self.config.source_dir.join("turns");
        let target_dir = self.config.target_dir.join("turns");

        let entries =
            std::fs::read_dir(&source_dir).map_err(|e| format!("读取源目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| format!("读取文件失败 {}: {}", path.display(), e))?;

                let turns: Vec<ExportedTurn> =
                    serde_json::from_str(&content).map_err(|e| format!("解析 JSON 失败: {}", e))?;

                let mut transformed_turns = Vec::new();

                for exported in turns {
                    match self.transform_turn(&exported) {
                        Ok(transformed) => {
                            transformed_turns.push(transformed);
                            self.state.turns_processed += 1;
                        }
                        Err(e) => {
                            self.state.errors.push(MigrationError::new(
                                "transform_error",
                                &e,
                                Some(&exported.original_id),
                            ));
                        }
                    }
                }

                let target_path = target_dir.join(path.file_name().unwrap());
                let output = serde_json::to_string_pretty(&transformed_turns)
                    .map_err(|e| format!("序列化失败: {}", e))?;
                std::fs::write(&target_path, output)
                    .map_err(|e| format!("写入失败 {}: {}", target_path.display(), e))?;
            }

            self.state.last_updated = chrono::Utc::now();
        }

        println!("转换轮次: {} 个成功", self.state.turns_processed);

        Ok(())
    }

    /// 转换单个轮次
    fn transform_turn(&mut self, exported: &ExportedTurn) -> Result<TransformedTurn, String> {
        let original_id = &exported.original_id;
        let turn = &exported.data;

        // 转换 ID: "turn:⟨uuid⟩" -> "turn_<uuid>"
        let key = Self::extract_turn_uuid(original_id)
            .ok_or_else(|| format!("无法解析轮次 ID: {}", original_id))?;

        // 转换 session_id: "session:⟨uuid⟩" -> "uuid"
        let session_key = Self::extract_uuid(&turn.session_id, "session")
            .ok_or_else(|| format!("无法解析会话 ID: {}", turn.session_id))?;

        // 记录 ID 映射
        self.id_mapping.insert(original_id.clone(), key.clone());

        // 转换 parent_id
        let parent_key = match &turn.parent_id {
            Some(id) => {
                let extracted = Self::extract_turn_uuid(id)
                    .ok_or_else(|| format!("无法解析父轮次 ID: {}", id))?;
                Some(extracted)
            }
            None => None,
        };

        // 转换 children_ids
        let children_keys: Vec<String> = turn
            .children_ids
            .iter()
            .filter_map(|id| Self::extract_turn_uuid(id))
            .collect();

        // 序列化复杂字段
        let metadata =
            serde_json::to_value(&turn.metadata).map_err(|e| format!("序列化元数据失败: {}", e))?;
        let dehydrated = match &turn.dehydrated {
            Some(d) => {
                Some(serde_json::to_value(d).map_err(|e| format!("序列化脱水数据失败: {}", e))?)
            }
            None => None,
        };

        let collection = format!("{}{}", self.config.collection_prefix, "turns");
        let id = format!("{}/{}", collection, key);

        Ok(TransformedTurn {
            _key: key,
            session_key,
            turn_number: turn.turn_number,
            raw_content: turn.raw_content.clone(),
            metadata,
            dehydrated,
            status: serde_json::to_string(&turn.status)
                .map_err(|e| format!("序列化状态失败: {}", e))?,
            parent_key,
            children_keys,
            _id: id,
        })
    }

    /// 转换索引记录数据
    async fn transform_index_records(&mut self) -> Result<(), String> {
        let source_dir = self.config.source_dir.join("index_records");
        let target_dir = self.config.target_dir.join("index_records");

        let entries =
            std::fs::read_dir(&source_dir).map_err(|e| format!("读取源目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| format!("读取文件失败 {}: {}", path.display(), e))?;

                let records: Vec<ExportedIndexRecord> =
                    serde_json::from_str(&content).map_err(|e| format!("解析 JSON 失败: {}", e))?;

                let mut transformed_records = Vec::new();

                for exported in records {
                    match self.transform_index_record(&exported) {
                        Ok(transformed) => {
                            transformed_records.push(transformed);
                            self.state.index_records_processed += 1;
                        }
                        Err(e) => {
                            self.state.errors.push(MigrationError::new(
                                "transform_error",
                                &e,
                                Some(&exported.original_id),
                            ));
                        }
                    }
                }

                let target_path = target_dir.join(path.file_name().unwrap());
                let output = serde_json::to_string_pretty(&transformed_records)
                    .map_err(|e| format!("序列化失败: {}", e))?;
                std::fs::write(&target_path, output)
                    .map_err(|e| format!("写入失败 {}: {}", target_path.display(), e))?;
            }

            self.state.last_updated = chrono::Utc::now();
        }

        println!(
            "转换索引记录: {} 个成功",
            self.state.index_records_processed
        );

        Ok(())
    }

    /// 转换单个索引记录
    fn transform_index_record(
        &self,
        exported: &ExportedIndexRecord,
    ) -> Result<TransformedIndexRecord, String> {
        let original_id = &exported.original_id;
        let record = &exported.data;

        // 转换 ID: "index_record:⟨uuid⟩" -> "idx_<uuid>"
        let key = Self::extract_index_record_uuid(original_id)
            .ok_or_else(|| format!("无法解析索引记录 ID: {}", original_id))?;

        // 转换 turn_id: "turn:⟨uuid⟩" -> "turn_<uuid>"
        let turn_key = Self::extract_turn_uuid(&record.turn_id)
            .ok_or_else(|| format!("无法解析轮次 ID: {}", record.turn_id))?;

        // 转换 session_id: "session:⟨uuid⟩" -> "uuid"
        let session_key = Self::extract_uuid(&record.session_id, "session")
            .ok_or_else(|| format!("无法解析会话 ID: {}", record.session_id))?;

        let timestamp = record.timestamp.to_rfc3339();

        let collection = format!("{}{}", self.config.collection_prefix, "index_records");
        let id = format!("{}/{}", collection, key);

        Ok(TransformedIndexRecord {
            _key: key,
            turn_key,
            session_key,
            tenant_id: record.tenant_id.clone(),
            gist: record.gist.clone(),
            topics: record.topics.clone(),
            tags: record.tags.clone(),
            timestamp,
            vector_id: record.vector_id.clone(),
            relevance_score: record.relevance_score,
            turn_number: record.turn_number,
            _id: id,
        })
    }

    /// 生成边集合
    async fn generate_edges(&mut self) -> Result<(), String> {
        let _edges_dir = self.config.target_dir.join("edges");

        // 从转换后的数据中生成边
        // 这里需要读取转换后的会话和轮次数据
        // 生成 session_turns, turn_parents, turn_index_records 边

        self.state.edges_generated = 0;
        println!("生成边: {} 个", self.state.edges_generated);

        Ok(())
    }

    /// 保存元数据
    async fn save_metadata(&self) -> Result<(), String> {
        let metadata_dir = self.config.target_dir.join("metadata");
        std::fs::create_dir_all(&metadata_dir).map_err(|e| format!("创建元数据目录失败: {}", e))?;

        let state_path = metadata_dir.join("transform_state.json");
        {
            let state_file = std::fs::File::create(&state_path)
                .map_err(|e| format!("创建状态文件失败: {}", e))?;
            let mut writer = std::io::BufWriter::new(state_file);
            serde_json::to_writer_pretty(&mut writer, &self.state)
                .map_err(|e| format!("序列化状态失败: {}", e))?;
        }

        Ok(())
    }

    /// 从 Surrealdb ID 提取 UUID
    fn extract_uuid(id: &str, _prefix: &str) -> Option<String> {
        // 格式: "prefix:⟨uuid⟩" 或 "prefix:uuid"
        if let Some(uuid) = id.split("⟨").nth(1) {
            if let Some(uuid) = uuid.split("⟩").next() {
                return Some(uuid.to_string());
            }
        }
        if let Some(uuid) = id.split(':').nth(1) {
            if uuid != id {
                return Some(uuid.to_string());
            }
        }
        None
    }

    /// 从轮次 ID 提取 UUID
    fn extract_turn_uuid(id: &str) -> Option<String> {
        Self::extract_uuid(id, "turn").map(|uuid| format!("turn_{}", uuid))
    }

    /// 从索引记录 ID 提取 UUID
    fn extract_index_record_uuid(id: &str) -> Option<String> {
        Self::extract_uuid(id, "index_record").map(|uuid| format!("idx_{}", uuid))
    }
}

/// 创建转换器
pub fn create_transformer(source_dir: PathBuf, target_dir: PathBuf) -> DataTransformer {
    let config = TransformConfig {
        source_dir,
        target_dir,
        ..Default::default()
    };

    DataTransformer::new(config)
}
