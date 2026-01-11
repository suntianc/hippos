use crate::config::config::{AppConfig, DatabaseConfig, VectorConfig};
use figment::{
    Figment,
    providers::{Env, Format, Toml},
};
use std::path::PathBuf;

/// 配置加载器
pub struct ConfigLoader;

impl ConfigLoader {
    /// 从默认路径加载配置
    ///
    /// 搜索路径：
    /// 1. ./config.yaml
    /// 2. 环境变量
    pub fn load() -> Result<AppConfig, figment::Error> {
        let figment = Figment::new()
            .merge(Toml::file("config.yaml"))
            .merge(Env::prefixed("EXOCORTEX_").split("_").global());

        figment.extract()
    }

    /// 从指定路径加载配置
    pub fn load_from(path: PathBuf) -> Result<AppConfig, figment::Error> {
        let figment = Figment::new()
            .merge(Toml::file(path))
            .merge(Env::prefixed("EXOCORTEX_").split("_").global());

        figment.extract()
    }

    /// 加载数据库配置
    pub fn load_database_config() -> Result<DatabaseConfig, figment::Error> {
        let figment = Figment::new()
            .merge(Toml::file("config.yaml"))
            .merge(Env::prefixed("EXOCORTEX_DB_").split("_").global());

        figment.extract()
    }

    /// 加载向量数据库配置
    pub fn load_vector_config() -> Result<VectorConfig, figment::Error> {
        let figment = Figment::new()
            .merge(Toml::file("config.yaml"))
            .merge(Env::prefixed("EXOCORTEX_VECTOR_").split("_").global());

        figment.extract()
    }

    /// 验证配置
    pub fn validate(config: &AppConfig) -> Result<(), ConfigValidationError> {
        if config.server.port == 0 {
            return Err(ConfigValidationError::InvalidPort);
        }

        if config.database.url.is_empty() {
            return Err(ConfigValidationError::MissingDatabaseUrl);
        }

        if config.vector.dimension == 0 {
            return Err(ConfigValidationError::InvalidDimension);
        }

        Ok(())
    }
}

/// 配置验证错误
#[derive(thiserror::Error, Debug)]
pub enum ConfigValidationError {
    #[error("服务端口无效，必须大于 0")]
    InvalidPort,

    #[error("数据库连接 URL 未配置")]
    MissingDatabaseUrl,

    #[error("向量维度无效，必须大于 0")]
    InvalidDimension,

    #[error("配置路径无效: {0}")]
    InvalidPath(String),
}

/// 获取默认配置文件路径
pub fn default_config_path() -> PathBuf {
    PathBuf::from("config.yaml")
}

/// 检查配置文件是否存在
pub fn config_exists() -> bool {
    default_config_path().exists()
}
