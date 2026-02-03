//! 存储工厂模块
//!
//! 根据配置创建相应的数据库存储实例。

use crate::config::config::{DatabaseConfig, DatabaseType};
use crate::error::{AppError, Result};
use std::sync::Arc;

#[cfg(feature = "surrealdb")]
use crate::storage::surrealdb::SurrealPool;

#[cfg(feature = "arangodb")]
use crate::storage::arangodb::ArangoStorage;

/// 存储实例枚举
pub enum StorageInstance {
    #[cfg(feature = "surrealdb")]
    SurrealDB(Arc<SurrealPool>),
    #[cfg(feature = "arangodb")]
    ArangoDB(ArangoStorage),
}

/// 存储工厂
pub struct StorageFactory;

impl StorageFactory {
    /// 根据配置创建存储实例
    #[cfg(feature = "surrealdb")]
    pub async fn create(config: &DatabaseConfig) -> Result<StorageInstance> {
        match config.db_type {
            DatabaseType::SurrealDB => {
                let pool = SurrealPool::new(config.clone())
                    .await
                    .map_err(|e| AppError::Database(e.to_string()))?;
                Ok(StorageInstance::SurrealDB(Arc::new(pool)))
            }
            #[cfg(feature = "arangodb")]
            DatabaseType::ArangoDB => Self::create_arangodb(config).await,
            #[cfg(not(feature = "arangodb"))]
            DatabaseType::ArangoDB => Err(AppError::Config(
                "ArangoDB feature is not enabled. Enable 'arangodb' feature to use ArangoDB."
                    .into(),
            )),
        }
    }

    #[cfg(not(feature = "surrealdb"))]
    pub async fn create(config: &DatabaseConfig) -> Result<StorageInstance> {
        #[cfg(feature = "arangodb")]
        {
            return ArangoStorage::new(config)
                .await
                .map(StorageInstance::ArangoDB)
                .map_err(AppError::Database);
        }

        #[cfg(not(feature = "arangodb"))]
        Err(AppError::Config(
            "No database feature enabled. Enable either 'surrealdb' or 'arangodb' feature.".into(),
        ))
    }

    /// 创建 ArangoDB 存储实例
    #[cfg(feature = "arangodb")]
    async fn create_arangodb(config: &DatabaseConfig) -> Result<StorageInstance> {
        let storage = ArangoStorage::new(config)
            .await
            .map_err(AppError::Database)?;
        Ok(StorageInstance::ArangoDB(storage))
    }

    /// 检查存储是否可用
    pub async fn health_check(&self, storage: &StorageInstance) -> Result<bool> {
        match storage {
            #[cfg(feature = "surrealdb")]
            StorageInstance::SurrealDB(pool) => {
                let _ = pool.inner().await;
                Ok(true)
            }
            #[cfg(feature = "arangodb")]
            StorageInstance::ArangoDB(storage) => {
                let query = "RETURN 1";
                let result = storage.aql::<i32>(query).await?;
                Ok(result.first().map(|&v| v == 1).unwrap_or(false))
            }
        }
    }
}

#[cfg(feature = "arangodb")]
impl StorageInstance {
    pub fn as_arangodb(&self) -> Option<&ArangoStorage> {
        match self {
            StorageInstance::ArangoDB(storage) => Some(storage),
            _ => None,
        }
    }

    pub fn as_arangodb_mut(&mut self) -> Option<&mut ArangoStorage> {
        match self {
            StorageInstance::ArangoDB(storage) => Some(storage),
            _ => None,
        }
    }
}

#[cfg(feature = "surrealdb")]
impl StorageInstance {
    pub fn as_surrealdb(&self) -> Option<&Arc<SurrealPool>> {
        match self {
            StorageInstance::SurrealDB(pool) => Some(pool),
            _ => None,
        }
    }
}
