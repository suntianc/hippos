//! 数据导入模块
//!
//! 将转换后的数据导入到 ArangoDB。
//! 支持文档和边集合的批量导入。

use crate::migration::{MigrationError, transform::*};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 导入配置
#[derive(Debug, Clone)]
pub struct ImportConfig {
    pub source_dir: PathBuf,
    pub arango_url: String,
    pub arango_db: String,
    pub arango_user: String,
    pub arango_password: String,
    pub collection_prefix: String,
    pub batch_size: usize,
    pub create_collections: bool,
    pub create_indexes: bool,
}

impl Default for ImportConfig {
    fn default() -> Self {
        Self {
            source_dir: PathBuf::from("./migration_transformed"),
            arango_url: "http://localhost:8529".to_string(),
            arango_db: "hippos".to_string(),
            arango_user: "root".to_string(),
            arango_password: "password".to_string(),
            collection_prefix: "hippos_".to_string(),
            batch_size: 100,
            create_collections: true,
            create_indexes: true,
        }
    }
}

/// 导入状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportState {
    pub sessions_imported: usize,
    pub turns_imported: usize,
    pub index_records_imported: usize,
    pub edges_imported: usize,
    pub errors: Vec<MigrationError>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for ImportState {
    fn default() -> Self {
        Self {
            sessions_imported: 0,
            turns_imported: 0,
            index_records_imported: 0,
            edges_imported: 0,
            errors: Vec::new(),
            started_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
        }
    }
}

/// ArangoDB 导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success: bool,
    pub documents_created: usize,
    pub documents_updated: usize,
    pub errors: Vec<ImportError>,
}

/// 导入错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportError {
    pub error_num: i32,
    pub error_message: String,
    pub document_key: Option<String>,
}

/// 数据导入器
pub struct DataImporter {
    config: ImportConfig,
    state: ImportState,
    http_client: reqwest::Client,
}

impl DataImporter {
    /// 创建新的导入器
    pub fn new(config: ImportConfig) -> Self {
        let http_client = reqwest::Client::new();

        Self {
            config,
            state: ImportState::default(),
            http_client,
        }
    }

    /// 运行完整导入
    pub async fn import_all(&mut self) -> Result<ImportState, String> {
        // 创建集合
        if self.config.create_collections {
            self.create_collections().await?;
        }

        // 导入会话
        self.import_sessions().await?;

        // 导入轮次
        self.import_turns().await?;

        // 导入索引记录
        self.import_index_records().await?;

        // 导入边
        self.import_edges().await?;

        // 创建索引
        if self.config.create_indexes {
            self.create_indexes().await?;
        }

        Ok(self.state.clone())
    }

    /// 创建集合
    async fn create_collections(&self) -> Result<(), String> {
        // 创建文档集合
        let collections = [
            format!("{}sessions", self.config.collection_prefix),
            format!("{}turns", self.config.collection_prefix),
            format!("{}index_records", self.config.collection_prefix),
        ];

        // 创建边集合
        let edge_collections = [
            format!("{}session_turns", self.config.collection_prefix),
            format!("{}turn_parents", self.config.collection_prefix),
            format!("{}turn_index_records", self.config.collection_prefix),
        ];

        // 调用 ArangoDB API 创建集合
        for collection in collections.iter().chain(edge_collections.iter()) {
            self.create_collection(collection, false).await?;
        }

        Ok(())
    }

    /// 创建单个集合
    async fn create_collection(&self, name: &str, is_edge: bool) -> Result<(), String> {
        let url = format!("{}/_api/collection", self.config.arango_url);

        let payload = serde_json::json!({
            "name": name,
            "type": if is_edge { 3 } else { 2 }
        });

        // 忽略 1207 错误（集合已存在）
        let response = self
            .http_client
            .post(&url)
            .basic_auth(&self.config.arango_user, Some(&self.config.arango_password))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("创建集合失败 {}: {}", name, e))?;

        if !response.status().is_success() {
            let error: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("解析错误响应失败: {}", e))?;

            // 1207 = collection already exists
            if error.get("errorNum").and_then(|v| v.as_i64()) != Some(1207) {
                return Err(format!(
                    "创建集合失败 {}: {}",
                    name,
                    error.get("errorMessage").unwrap_or(&error)
                ));
            }
        }

        Ok(())
    }

    /// 导入会话
    async fn import_sessions(&mut self) -> Result<(), String> {
        let source_dir = self.config.source_dir.join("sessions");
        let collection = format!("{}sessions", self.config.collection_prefix);

        let entries =
            std::fs::read_dir(&source_dir).map_err(|e| format!("读取源目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| format!("读取文件失败 {}: {}", path.display(), e))?;

                let sessions: Vec<TransformedSession> =
                    serde_json::from_str(&content).map_err(|e| format!("解析 JSON 失败: {}", e))?;

                let result = self.import_documents(&collection, &sessions).await?;

                self.state.sessions_imported += result.documents_created;
                self.state.last_updated = chrono::Utc::now();

                // 记录错误
                for error in result.errors {
                    self.state.errors.push(MigrationError::new(
                        "import_error",
                        &error.error_message,
                        error.document_key.as_deref(),
                    ));
                }
            }
        }

        println!("导入会话: {} 个成功", self.state.sessions_imported);

        Ok(())
    }

    /// 导入轮次
    async fn import_turns(&mut self) -> Result<(), String> {
        let source_dir = self.config.source_dir.join("turns");
        let collection = format!("{}turns", self.config.collection_prefix);

        let entries =
            std::fs::read_dir(&source_dir).map_err(|e| format!("读取源目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| format!("读取文件失败 {}: {}", path.display(), e))?;

                let turns: Vec<TransformedTurn> =
                    serde_json::from_str(&content).map_err(|e| format!("解析 JSON 失败: {}", e))?;

                let result = self.import_documents(&collection, &turns).await?;

                self.state.turns_imported += result.documents_created;
                self.state.last_updated = chrono::Utc::now();

                for error in result.errors {
                    self.state.errors.push(MigrationError::new(
                        "import_error",
                        &error.error_message,
                        error.document_key.as_deref(),
                    ));
                }
            }
        }

        println!("导入轮次: {} 个成功", self.state.turns_imported);

        Ok(())
    }

    /// 导入索引记录
    async fn import_index_records(&mut self) -> Result<(), String> {
        let source_dir = self.config.source_dir.join("index_records");
        let collection = format!("{}index_records", self.config.collection_prefix);

        let entries =
            std::fs::read_dir(&source_dir).map_err(|e| format!("读取源目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| format!("读取文件失败 {}: {}", path.display(), e))?;

                let records: Vec<TransformedIndexRecord> =
                    serde_json::from_str(&content).map_err(|e| format!("解析 JSON 失败: {}", e))?;

                let result = self.import_documents(&collection, &records).await?;

                self.state.index_records_imported += result.documents_created;
                self.state.last_updated = chrono::Utc::now();

                for error in result.errors {
                    self.state.errors.push(MigrationError::new(
                        "import_error",
                        &error.error_message,
                        error.document_key.as_deref(),
                    ));
                }
            }
        }

        println!("导入索引记录: {} 个成功", self.state.index_records_imported);

        Ok(())
    }

    /// 导入边
    async fn import_edges(&mut self) -> Result<(), String> {
        let edges_dir = self.config.source_dir.join("edges");

        if !edges_dir.exists() {
            println!("边目录不存在，跳过边导入");
            return Ok(());
        }

        // 导入会话-轮次边
        let st_edges_dir = edges_dir.join("session_turns");
        if st_edges_dir.exists() {
            self.import_edge_collection(
                &format!("{}session_turns", self.config.collection_prefix),
                &st_edges_dir,
            )
            .await?;
        }

        // 导入轮次-轮次边
        let tp_edges_dir = edges_dir.join("turn_parents");
        if tp_edges_dir.exists() {
            self.import_edge_collection(
                &format!("{}turn_parents", self.config.collection_prefix),
                &tp_edges_dir,
            )
            .await?;
        }

        // 导入轮次-索引记录边
        let ti_edges_dir = edges_dir.join("turn_index_records");
        if ti_edges_dir.exists() {
            self.import_edge_collection(
                &format!("{}turn_index_records", self.config.collection_prefix),
                &ti_edges_dir,
            )
            .await?;
        }

        println!("导入边: {} 个成功", self.state.edges_imported);

        Ok(())
    }

    /// 导入单个边集合
    async fn import_edge_collection(
        &mut self,
        collection: &str,
        dir: &PathBuf,
    ) -> Result<(), String> {
        let entries = std::fs::read_dir(dir).map_err(|e| format!("读取边目录失败: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("读取条目失败: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| format!("读取文件失败 {}: {}", path.display(), e))?;

                let edges: Vec<serde_json::Value> =
                    serde_json::from_str(&content).map_err(|e| format!("解析 JSON 失败: {}", e))?;

                let result = self.import_documents(collection, &edges).await?;
                self.state.edges_imported += result.documents_created;
            }
        }

        Ok(())
    }

    /// 导入文档到集合
    async fn import_documents<T: Serialize>(
        &self,
        collection: &str,
        documents: &[T],
    ) -> Result<ImportResult, String> {
        let url = format!("{}/_api/document/{}", self.config.arango_url, collection);

        let payload = serde_json::json!(documents);

        let response = self
            .http_client
            .post(&url)
            .basic_auth(&self.config.arango_user, Some(&self.config.arango_password))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("导入文档失败: {}", e))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| format!("获取错误响应失败: {}", e))?;
            return Err(format!("导入失败: {}", error_text));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))?;

        // 解析导入结果
        let documents_created = result.get("created").and_then(|v| v.as_u64()).unwrap_or(0);
        let documents_updated = result.get("updated").and_then(|v| v.as_u64()).unwrap_or(0);

        let mut errors = Vec::new();
        if let Some(error_entries) = result.get("errors").and_then(|v| v.as_array()) {
            for error in error_entries {
                errors.push(ImportError {
                    error_num: error.get("errorNum").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                    error_message: error
                        .get("errorMessage")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error")
                        .to_string(),
                    document_key: error
                        .get("_key")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                });
            }
        }

        Ok(ImportResult {
            success: errors.is_empty(),
            documents_created: documents_created as usize,
            documents_updated: documents_updated as usize,
            errors,
        })
    }

    /// 创建索引
    async fn create_indexes(&self) -> Result<(), String> {
        // 会话集合索引
        let session_collection = format!("{}sessions", self.config.collection_prefix);
        self.create_hash_index(&session_collection, "tenant_id")
            .await?;
        self.create_hash_index(&session_collection, "status")
            .await?;
        self.create_skiplist_index(&session_collection, "created_at")
            .await?;

        // 轮次集合索引
        let turn_collection = format!("{}turns", self.config.collection_prefix);
        self.create_hash_index(&turn_collection, "session_key")
            .await?;
        self.create_skiplist_index(&turn_collection, "turn_number")
            .await?;

        // 索引记录集合索引
        let index_collection = format!("{}index_records", self.config.collection_prefix);
        self.create_hash_index(&index_collection, "session_key")
            .await?;
        self.create_hash_index(&index_collection, "tenant_id")
            .await?;
        self.create_skiplist_index(&index_collection, "timestamp")
            .await?;

        println!("索引创建完成");
        Ok(())
    }

    /// 创建哈希索引
    async fn create_hash_index(&self, collection: &str, fields: &str) -> Result<(), String> {
        let url = format!("{}/_api/index/{}", self.config.arango_url, collection);

        let payload = serde_json::json!({
            "type": "hash",
            "fields": [fields],
            "unique": false
        });

        // 忽略已存在的索引错误
        let response = self
            .http_client
            .post(&url)
            .basic_auth(&self.config.arango_user, Some(&self.config.arango_password))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("创建索引失败: {}", e))?;

        // 成功或索引已存在都算成功
        if !response.status().is_success() {
            let error: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("解析错误响应失败: {}", e))?;

            // 1206 = index already exists
            if error.get("errorNum").and_then(|v| v.as_i64()) != Some(1206) {
                return Err(format!(
                    "创建索引失败 for {} on {}: {}",
                    collection,
                    fields,
                    error.get("errorMessage").unwrap_or(&error)
                ));
            }
        }

        Ok(())
    }

    /// 创建跳表索引
    async fn create_skiplist_index(&self, collection: &str, fields: &str) -> Result<(), String> {
        let url = format!("{}/_api/index/{}", self.config.arango_url, collection);

        let payload = serde_json::json!({
            "type": "skiplist",
            "fields": [fields],
            "unique": false
        });

        let response = self
            .http_client
            .post(&url)
            .basic_auth(&self.config.arango_user, Some(&self.config.arango_password))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("创建索引失败: {}", e))?;

        if !response.status().is_success() {
            let error: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("解析错误响应失败: {}", e))?;

            if error.get("errorNum").and_then(|v| v.as_i64()) != Some(1206) {
                return Err(format!(
                    "创建索引失败 for {} on {}: {}",
                    collection,
                    fields,
                    error.get("errorMessage").unwrap_or(&error)
                ));
            }
        }

        Ok(())
    }
}

/// 创建导入器
pub fn create_importer(source_dir: PathBuf) -> DataImporter {
    let config = ImportConfig {
        source_dir,
        ..Default::default()
    };

    DataImporter::new(config)
}
