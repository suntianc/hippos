use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DatabaseConfig {
    /// SurrealDB 连接地址
    pub url: String,
    /// 命名空间
    pub namespace: String,
    /// 数据库名称
    pub database: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 连接池最小大小
    pub min_connections: usize,
    /// 连接池最大大小
    pub max_connections: usize,
    /// 连接超时（秒）
    pub connection_timeout: u64,
    /// 空闲超时（秒）
    pub idle_timeout: u64,
}

/// 向量数据库配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct VectorConfig {
    /// LanceDB 数据目录
    pub data_dir: PathBuf,
    /// 向量维度
    pub dimension: usize,
    /// IVF 聚类数量
    pub nlist: usize,
    /// 查询时搜索的聚类数
    pub nprobe: usize,
    /// 产品量化分段数
    pub pq_m: usize,
    /// 距离计算方式
    pub distance_type: String,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ServerConfig {
    /// 服务地址
    pub host: String,
    /// 服务端口
    pub port: u16,
    /// 工作线程数
    pub workers: usize,
    /// 请求超时（秒）
    pub request_timeout: u64,
    /// 最大请求体大小（字节）
    pub max_request_size: usize,
}

/// 安全配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SecurityConfig {
    /// API 密钥
    pub api_key: String,
    /// Rate limiting 启用
    pub rate_limit_enabled: bool,
    /// 全局限流请求数/秒
    pub global_rate_limit: u32,
    /// 单个会话限流请求数/分钟
    pub per_session_rate_limit: u32,
    /// Redis 地址（用于分布式限流）
    pub redis_url: String,
    /// TLS 启用
    pub tls_enabled: bool,
    /// TLS 证书路径
    pub tls_cert_path: Option<PathBuf>,
    /// TLS 私钥路径
    pub tls_key_path: Option<PathBuf>,
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct LoggingConfig {
    /// 日志级别
    pub level: String,
    /// 结构化日志格式
    pub structured: bool,
    /// 日志文件路径
    pub log_dir: Option<PathBuf>,
    /// 日志文件大小上限（字节）
    pub file_max_size: u64,
    /// 保留日志文件数
    pub file_max_count: u32,
}

/// 嵌入模型配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EmbeddingConfig {
    /// 模型名称
    pub model_name: String,
    /// 模型路径
    pub model_path: Option<PathBuf>,
    /// 批处理大小
    pub batch_size: usize,
    /// 是否使用 GPU
    pub use_gpu: bool,
    /// Embedding 后端类型: "ollama" 或 "simple"
    pub backend: String,
    /// Ollama 服务器地址
    pub ollama_url: String,
    /// Ollama 请求超时（秒）
    pub ollama_timeout: u64,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AppConfig {
    /// 数据库配置
    pub database: DatabaseConfig,
    /// 向量数据库配置
    pub vector: VectorConfig,
    /// 服务器配置
    pub server: ServerConfig,
    /// 安全配置
    pub security: SecurityConfig,
    /// 日志配置
    pub logging: LoggingConfig,
    /// 嵌入模型配置
    pub embedding: EmbeddingConfig,
    /// 应用名称
    pub app_name: String,
    /// 环境
    pub environment: String,
}

impl AppConfig {
    /// 创建开发环境配置
    pub fn development() -> Self {
        Self {
            database: DatabaseConfig {
                url: "ws://localhost:8000".into(),
                namespace: "hippos".into(),
                database: "sessions".into(),
                username: "root".into(),
                password: "root".into(),
                min_connections: 5,
                max_connections: 50,
                connection_timeout: 30,
                idle_timeout: 300,
            },
            vector: VectorConfig {
                data_dir: PathBuf::from("./data/lancedb"),
                dimension: 384,
                nlist: 1024,
                nprobe: 32,
                pq_m: 8,
                distance_type: "cosine".into(),
            },
            server: ServerConfig {
                host: "0.0.0.0".into(),
                port: 8080,
                workers: 4,
                request_timeout: 30,
                max_request_size: 10 * 1024 * 1024,
            },
            security: SecurityConfig {
                api_key: "dev-api-key-change-in-production".into(),
                rate_limit_enabled: false,
                global_rate_limit: 1000,
                per_session_rate_limit: 100,
                redis_url: "redis://localhost:6379".into(),
                tls_enabled: false,
                tls_cert_path: None,
                tls_key_path: None,
            },
            logging: LoggingConfig {
                level: "debug".into(),
                structured: true,
                log_dir: Some(PathBuf::from("./logs")),
                file_max_size: 100 * 1024 * 1024,
                file_max_count: 10,
            },
            embedding: EmbeddingConfig {
                model_name: "all-MiniLM-L6-v2".into(),
                model_path: None,
                batch_size: 32,
                use_gpu: false,
                backend: "simple".into(),
                ollama_url: "http://localhost:11434".into(),
                ollama_timeout: 60,
            },
            app_name: "hippos".into(),
            environment: "development".into(),
        }
    }

    /// 创建生产环境配置
    pub fn production() -> Self {
        let mut config = Self::development();
        config.environment = "production".into();
        config.logging.level = "info".into();
        config.server.workers = std::thread::available_parallelism()
            .map(|p| p.get())
            .unwrap_or(4);
        config.security.rate_limit_enabled = true;
        config
    }
}
