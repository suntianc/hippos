use crate::config::config::DatabaseConfig;
use reqwest;
use std::sync::Arc;
use surrealdb::{
    Surreal,
    engine::any::{Any, connect},
    opt::auth::Root,
};
use tokio::sync::Mutex;

/// SurrealDB 连接池
#[derive(Clone)]
pub struct SurrealPool {
    /// 数据库连接
    db: Arc<Mutex<Option<Surreal<Any>>>>,
    /// 连接配置
    config: DatabaseConfig,
    /// HTTP client for raw queries
    http_client: Arc<reqwest::Client>,
}

impl SurrealPool {
    /// 创建新的连接池
    pub async fn new(config: DatabaseConfig) -> Result<Self, surrealdb::Error> {
        let db: Surreal<Any> = connect(&config.url).await?;

        // 认证
        db.signin(Root {
            username: &config.username,
            password: &config.password,
        })
        .await?;

        // 选择命名空间和数据库
        db.use_ns(&config.namespace)
            .use_db(&config.database)
            .await?;

        // Create HTTP client
        let http_client = Arc::new(reqwest::Client::new());

        Ok(Self {
            db: Arc::new(Mutex::new(Some(db))),
            config,
            http_client,
        })
    }

    /// 获取连接
    pub async fn get(&self) -> SurrealPoolConn {
        SurrealPoolConn { pool: self.clone() }
    }

    /// 获取内部数据库实例
    pub async fn inner(&self) -> Surreal<Any> {
        let guard = self.db.lock().await;
        guard.as_ref().expect("Database connection closed").clone()
    }

    /// 获取 HTTP client
    pub fn http_client(&self) -> Arc<reqwest::Client> {
        self.http_client.clone()
    }

    /// 获取数据库配置
    pub fn config(&self) -> &DatabaseConfig {
        &self.config
    }

    /// 关闭连接
    pub async fn close(&self) {
        let mut guard = self.db.lock().await;
        *guard = None;
    }
}

/// 连接包装器
pub struct SurrealPoolConn {
    pool: SurrealPool,
}

impl SurrealPoolConn {
    /// 获取数据库实例
    pub async fn db(&self) -> Surreal<Any> {
        self.pool.inner().await
    }

    /// 获取 HTTP client
    pub fn http_client(&self) -> Arc<reqwest::Client> {
        self.pool.http_client()
    }

    /// 获取数据库配置
    pub fn config(&self) -> &DatabaseConfig {
        self.pool.config()
    }
}
