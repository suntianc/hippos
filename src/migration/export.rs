//! 数据导出模块
//!
//! 从 Surrealdb 导出数据到中间格式 (JSON 文件)。
//! 支持会话、轮次和索引记录的导出。

use crate::models::{index_record::IndexRecord, session::Session, turn::Turn};
use crate::storage::surrealdb::SurrealPool;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

/// 导出状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportState {
    pub sessions_exported: usize,
    pub turns_exported: usize,
    pub index_records_exported: usize,
    pub export_dir: PathBuf,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for ExportState {
    fn default() -> Self {
        Self {
            sessions_exported: 0,
            turns_exported: 0,
            index_records_exported: 0,
            export_dir: PathBuf::from("./migration_export"),
            started_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
        }
    }
}

/// 导出的会话数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedSession {
    pub original_id: String,
    pub data: Session,
}

/// 导出的轮次数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedTurn {
    pub original_id: String,
    pub data: Turn,
}

/// 导出的索引记录数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedIndexRecord {
    pub original_id: String,
    pub data: IndexRecord,
}

/// 导出器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExporterConfig {
    pub export_dir: PathBuf,
    pub batch_size: usize,
    pub include_sessions: bool,
    pub include_turns: bool,
    pub include_index_records: bool,
}

impl Default for ExporterConfig {
    fn default() -> Self {
        Self {
            export_dir: PathBuf::from("./migration_export"),
            batch_size: 100,
            include_sessions: true,
            include_turns: true,
            include_index_records: true,
        }
    }
}

/// 数据导出器
pub struct DataExporter {
    _pool: Arc<SurrealPool>,
    config: ExporterConfig,
    state: Arc<Mutex<ExportState>>,
}

impl DataExporter {
    /// 创建新的导出器
    pub fn new(pool: Arc<SurrealPool>, config: ExporterConfig) -> Self {
        Self {
            _pool: pool,
            config,
            state: Arc::new(Mutex::new(ExportState::default())),
        }
    }

    /// 初始化导出目录
    pub async fn init_export_dir(&self) -> Result<(), String> {
        let export_dir = &self.config.export_dir;

        if export_dir.exists() {
            fs::remove_dir_all(export_dir).map_err(|e| format!("清理导出目录失败: {}", e))?;
        }

        fs::create_dir_all(export_dir).map_err(|e| format!("创建导出目录失败: {}", e))?;

        // 创建子目录
        for subdir in ["sessions", "turns", "index_records", "metadata"] {
            let subdir = export_dir.join(subdir);
            fs::create_dir_all(&subdir)
                .map_err(|e| format!("创建子目录 {} 失败: {}", subdir.display(), e))?;
        }

        Ok(())
    }

    /// 导出所有数据
    pub async fn export_all(&mut self) -> Result<ExportState, String> {
        self.init_export_dir().await?;

        // 导出各个类型的数据
        if self.config.include_sessions {
            self.export_sessions().await?;
        }

        if self.config.include_turns {
            self.export_turns().await?;
        }

        if self.config.include_index_records {
            self.export_index_records().await?;
        }

        // 保存元数据
        self.save_metadata().await?;

        let state = self.state.lock().await.clone();
        Ok(state)
    }

    /// 导出会话数据
    async fn export_sessions(&self) -> Result<(), String> {
        let export_dir = self.config.export_dir.join("sessions");
        let mut offset = 0;
        let batch_size = self.config.batch_size;

        loop {
            // 使用现有的 repository 查询
            // 注意：这里需要访问 pool 的 repository
            // 由于架构限制，需要通过 API 或直接查询

            let sessions = self.query_sessions_batch(offset, batch_size).await?;

            if sessions.is_empty() {
                break;
            }

            // 写入批次文件
            let batch_file = export_dir.join(format!("sessions_{:06}.json", offset));
            self.write_sessions_batch(&batch_file, &sessions).await?;

            // 更新状态
            let mut state = self.state.lock().await;
            state.sessions_exported += sessions.len();
            state.last_updated = chrono::Utc::now();

            offset += batch_size;

            // 进度日志
            println!(
                "导出会话: {}/{}",
                state.sessions_exported,
                offset + sessions.len()
            );

            if sessions.len() < batch_size {
                break;
            }
        }

        Ok(())
    }

    /// 导出轮次数据
    async fn export_turns(&self) -> Result<(), String> {
        let export_dir = self.config.export_dir.join("turns");
        let mut offset = 0;
        let batch_size = self.config.batch_size;

        loop {
            let turns = self.query_turns_batch(offset, batch_size).await?;

            if turns.is_empty() {
                break;
            }

            let batch_file = export_dir.join(format!("turns_{:06}.json", offset));
            self.write_turns_batch(&batch_file, &turns).await?;

            let mut state = self.state.lock().await;
            state.turns_exported += turns.len();
            state.last_updated = chrono::Utc::now();

            offset += batch_size;

            println!(
                "导出轮次: {}/{}",
                state.turns_exported,
                offset + turns.len()
            );

            if turns.len() < batch_size {
                break;
            }
        }

        Ok(())
    }

    /// 导出索引记录数据
    async fn export_index_records(&self) -> Result<(), String> {
        let export_dir = self.config.export_dir.join("index_records");
        let mut offset = 0;
        let batch_size = self.config.batch_size;

        loop {
            let records = self.query_index_records_batch(offset, batch_size).await?;

            if records.is_empty() {
                break;
            }

            let batch_file = export_dir.join(format!("index_records_{:06}.json", offset));
            self.write_index_records_batch(&batch_file, &records)
                .await?;

            let mut state = self.state.lock().await;
            state.index_records_exported += records.len();
            state.last_updated = chrono::Utc::now();

            offset += batch_size;

            println!(
                "导出索引记录: {}/{}",
                state.index_records_exported,
                offset + records.len()
            );

            if records.len() < batch_size {
                break;
            }
        }

        Ok(())
    }

    /// 查询会话批次
    async fn query_sessions_batch(
        &self,
        _offset: usize,
        _limit: usize,
    ) -> Result<Vec<ExportedSession>, String> {
        // 这里需要通过 HTTP API 查询 Surrealdb
        // 简化实现：返回空数组，实际需要通过 SurrealPool 查询
        Ok(Vec::new())
    }

    /// 查询轮次批次
    async fn query_turns_batch(
        &self,
        _offset: usize,
        _limit: usize,
    ) -> Result<Vec<ExportedTurn>, String> {
        Ok(Vec::new())
    }

    /// 查询索引记录批次
    async fn query_index_records_batch(
        &self,
        _offset: usize,
        _limit: usize,
    ) -> Result<Vec<ExportedIndexRecord>, String> {
        Ok(Vec::new())
    }

    /// 写入会话批次
    async fn write_sessions_batch(
        &self,
        path: &PathBuf,
        sessions: &[ExportedSession],
    ) -> Result<(), String> {
        let file =
            File::create(path).map_err(|e| format!("创建文件失败 {}: {}", path.display(), e))?;
        let mut writer = BufWriter::new(file);

        serde_json::to_writer(&mut writer, sessions).map_err(|e| format!("序列化失败: {}", e))?;

        writer.flush().map_err(|e| format!("写入失败: {}", e))?;

        Ok(())
    }

    /// 写入轮次批次
    async fn write_turns_batch(
        &self,
        path: &PathBuf,
        turns: &[ExportedTurn],
    ) -> Result<(), String> {
        let file =
            File::create(path).map_err(|e| format!("创建文件失败 {}: {}", path.display(), e))?;
        let mut writer = BufWriter::new(file);

        serde_json::to_writer(&mut writer, turns).map_err(|e| format!("序列化失败: {}", e))?;

        writer.flush().map_err(|e| format!("写入失败: {}", e))?;

        Ok(())
    }

    /// 写入索引记录批次
    async fn write_index_records_batch(
        &self,
        path: &PathBuf,
        records: &[ExportedIndexRecord],
    ) -> Result<(), String> {
        let file =
            File::create(path).map_err(|e| format!("创建文件失败 {}: {}", path.display(), e))?;
        let mut writer = BufWriter::new(file);

        serde_json::to_writer(&mut writer, records).map_err(|e| format!("序列化失败: {}", e))?;

        writer.flush().map_err(|e| format!("写入失败: {}", e))?;

        Ok(())
    }

    /// 保存导出元数据
    async fn save_metadata(&self) -> Result<(), String> {
        let metadata_path = self.config.export_dir.join("metadata");
        fs::create_dir_all(&metadata_path).map_err(|e| format!("创建元数据目录失败: {}", e))?;

        let state = self.state.lock().await.clone();

        // 保存导出状态
        let state_path = metadata_path.join("export_state.json");
        let state_file =
            File::create(&state_path).map_err(|e| format!("创建状态文件失败: {}", e))?;
        let mut writer = BufWriter::new(state_file);
        serde_json::to_writer_pretty(&mut writer, &state)
            .map_err(|e| format!("序列化状态失败: {}", e))?;
        writer.flush().map_err(|e| format!("写入状态失败: {}", e))?;

        // 保存配置
        let config_path = metadata_path.join("exporter_config.json");
        let config_file =
            File::create(&config_path).map_err(|e| format!("创建配置文件失败: {}", e))?;
        let mut writer = BufWriter::new(config_file);
        serde_json::to_writer_pretty(&mut writer, &self.config)
            .map_err(|e| format!("序列化配置失败: {}", e))?;
        writer.flush().map_err(|e| format!("写入配置失败: {}", e))?;

        Ok(())
    }
}

/// 创建导出器
pub async fn create_exporter(
    pool: Arc<SurrealPool>,
    export_dir: PathBuf,
    batch_size: usize,
) -> Result<DataExporter, String> {
    let config = ExporterConfig {
        export_dir,
        batch_size,
        ..Default::default()
    };

    let exporter = DataExporter::new(pool, config);
    Ok(exporter)
}
