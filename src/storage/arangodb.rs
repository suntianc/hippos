//! ArangoDB 存储层
//!
//! 使用直接 HTTP API 实现 ArangoDB 操作。

use crate::config::config::DatabaseConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// ArangoDB 配置
#[derive(Clone, Debug)]
pub struct ArangoConfig {
    pub url: String,
    pub database: String,
    pub username: String,
    pub password: String,
    pub collection_prefix: String,
}

impl From<DatabaseConfig> for ArangoConfig {
    fn from(config: DatabaseConfig) -> Self {
        let collection_prefix = if config.collection_prefix.is_empty() {
            "hippos_".to_string()
        } else {
            config.collection_prefix
        };
        Self {
            url: config.url.trim_end_matches("/").to_string(),
            database: config.database,
            username: config.username,
            password: config.password,
            collection_prefix,
        }
    }
}

/// ArangoDB 存储客户端
#[derive(Clone)]
pub struct ArangoStorage {
    /// 配置
    config: ArangoConfig,
    /// HTTP 客户端
    http_client: Arc<reqwest::Client>,
}

impl ArangoStorage {
    /// 创建新的存储客户端
    pub async fn new(config: &DatabaseConfig) -> Result<Self, String> {
        let arango_config = ArangoConfig::from(config.clone());

        let http_client = Arc::new(reqwest::Client::new());

        // 测试连接
        let test_url = format!("{}/_api/version", arango_config.url);
        let response = http_client
            .get(&test_url)
            .basic_auth(&arango_config.username, Some(&arango_config.password))
            .send()
            .await
            .map_err(|e| format!("连接 ArangoDB 失败: {}", e))?;

        if !response.status().is_success() {
            return Err("无法连接到 ArangoDB".to_string());
        }

        Ok(Self {
            config: arango_config,
            http_client,
        })
    }

    /// 获取完整集合名称
    pub fn collection_name(&self, name: &str) -> String {
        format!("{}{}", self.config.collection_prefix, name)
    }

    /// API 基础 URL
    fn api_url(&self) -> String {
        format!("{}/_api", self.config.url)
    }

    /// 执行 AQL 查询
    pub async fn aql<T>(&self, query: &str) -> Result<Vec<T>, String>
    where
        T: for<'de> Deserialize<'de>,
    {
        let url = format!("{}/cursor", self.api_url());

        let payload = serde_json::json!({
            "query": query,
            "batchSize": 100,
            "count": true
        });

        let response = self
            .http_client
            .post(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("AQL 查询失败: {}", e))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| format!("获取错误响应失败: {}", e))?;
            return Err(format!("AQL 错误: {}", error_text));
        }

        let result: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))?;

        let results = result
            .get("result")
            .and_then(|v| v.as_array())
            .ok_or_else(|| "无效的响应格式".to_string())?;

        let parsed: Vec<T> = serde_json::from_value(serde_json::Value::Array(results.clone()))
            .map_err(|e| format!("反序列化失败: {}", e))?;

        Ok(parsed)
    }

    /// 插入文档
    pub async fn insert<T: Serialize>(
        &self,
        collection: &str,
        document: &T,
    ) -> Result<serde_json::Value, String> {
        let url = format!(
            "{}/document/{}",
            self.api_url(),
            self.collection_name(collection)
        );
        let doc_value =
            serde_json::to_value(document).map_err(|e| format!("序列化文档失败: {}", e))?;

        let response = self
            .http_client
            .post(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .json(&doc_value)
            .send()
            .await
            .map_err(|e| format!("插入文档失败: {}", e))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| format!("获取错误响应失败: {}", e))?;
            return Err(format!("插入错误: {}", error_text));
        }

        response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))
    }

    /// 批量插入
    pub async fn insert_many<T: Serialize>(
        &self,
        collection: &str,
        documents: &[T],
    ) -> Result<Vec<serde_json::Value>, String> {
        let url = format!(
            "{}/document/{}",
            self.api_url(),
            self.collection_name(collection)
        );
        let docs: Vec<serde_json::Value> = documents
            .iter()
            .map(|d| serde_json::to_value(d).map_err(|e| format!("序列化失败: {}", e)))
            .collect::<Result<Vec<_>, _>>()?;

        let response = self
            .http_client
            .post(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .json(&docs)
            .send()
            .await
            .map_err(|e| format!("批量插入失败: {}", e))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| format!("获取错误响应失败: {}", e))?;
            return Err(format!("批量插入错误: {}", error_text));
        }

        response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))
    }

    /// 获取文档
    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        collection: &str,
        key: &str,
    ) -> Result<Option<T>, String> {
        let url = format!(
            "{}/document/{}/{}",
            self.api_url(),
            self.collection_name(collection),
            key
        );

        let response = self
            .http_client
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await
            .map_err(|e| format!("获取文档失败: {}", e))?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(None);
        }

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| format!("获取错误响应失败: {}", e))?;
            return Err(format!("获取错误: {}", error_text));
        }

        let doc: T = response
            .json()
            .await
            .map_err(|e| format!("解析文档失败: {}", e))?;

        Ok(Some(doc))
    }

    /// 更新文档
    pub async fn update<T: Serialize>(
        &self,
        collection: &str,
        key: &str,
        document: &T,
    ) -> Result<serde_json::Value, String> {
        let url = format!(
            "{}/document/{}/{}",
            self.api_url(),
            self.collection_name(collection),
            key
        );
        let doc_value =
            serde_json::to_value(document).map_err(|e| format!("序列化文档失败: {}", e))?;

        let response = self
            .http_client
            .patch(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .json(&doc_value)
            .send()
            .await
            .map_err(|e| format!("更新文档失败: {}", e))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| format!("获取错误响应失败: {}", e))?;
            return Err(format!("更新错误: {}", error_text));
        }

        response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))
    }

    /// 删除文档
    pub async fn delete(&self, collection: &str, key: &str) -> Result<serde_json::Value, String> {
        let url = format!(
            "{}/document/{}/{}",
            self.api_url(),
            self.collection_name(collection),
            key
        );

        let response = self
            .http_client
            .delete(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await
            .map_err(|e| format!("删除文档失败: {}", e))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .map_err(|e| format!("获取错误响应失败: {}", e))?;
            return Err(format!("删除错误: {}", error_text));
        }

        response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {}", e))
    }

    /// 创建集合
    pub async fn create_collection(&self, name: &str, is_edge: bool) -> Result<(), String> {
        let url = format!("{}/collection", self.api_url());

        let payload = serde_json::json!({
            "name": self.collection_name(name),
            "type": if is_edge { 3 } else { 2 }
        });

        let response = self
            .http_client
            .post(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("创建集合失败: {}", e))?;

        // 1207 = collection already exists
        if !response.status().is_success() {
            let error: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("解析错误响应失败: {}", e))?;

            if error.get("errorNum").and_then(|v| v.as_i64()) != Some(1207) {
                return Err(format!(
                    "创建集合失败: {}",
                    error.get("errorMessage").unwrap_or(&error)
                ));
            }
        }

        Ok(())
    }

    /// 创建索引
    pub async fn create_index(
        &self,
        collection: &str,
        index_type: &str,
        fields: &[&str],
    ) -> Result<(), String> {
        let url = format!(
            "{}/index/{}",
            self.api_url(),
            self.collection_name(collection)
        );
        let fields_array: Vec<&str> = fields.iter().copied().collect();

        let payload = serde_json::json!({
            "type": index_type,
            "fields": fields_array,
            "unique": false
        });

        let response = self
            .http_client
            .post(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .json(&payload)
            .send()
            .await
            .map_err(|e| format!("创建索引失败: {}", e))?;

        // 1206 = index already exists
        if !response.status().is_success() {
            let error: serde_json::Value = response
                .json()
                .await
                .map_err(|e| format!("解析错误响应失败: {}", e))?;

            if error.get("errorNum").and_then(|v| v.as_i64()) != Some(1206) {
                return Err(format!(
                    "创建索引失败: {}",
                    error.get("errorMessage").unwrap_or(&error)
                ));
            }
        }

        Ok(())
    }

    /// 检查集合是否存在
    pub async fn collection_exists(&self, name: &str) -> Result<bool, String> {
        let url = format!(
            "{}/collection/{}",
            self.api_url(),
            self.collection_name(name)
        );

        let response = self
            .http_client
            .get(&url)
            .basic_auth(&self.config.username, Some(&self.config.password))
            .send()
            .await
            .map_err(|e| format!("检查集合失败: {}", e))?;

        Ok(response.status() == reqwest::StatusCode::OK)
    }

    /// 获取文档数量
    pub async fn count(&self, collection: &str) -> Result<u64, String> {
        let query = format!(
            "RETURN LENGTH(FOR d IN {} RETURN 1)",
            self.collection_name(collection)
        );

        let result = self.aql::<serde_json::Value>(&query).await?;

        if let Some(count) = result.first().and_then(|v| v.as_u64()) {
            Ok(count)
        } else {
            Ok(0)
        }
    }
}

/// 创建 ArangoStorage 的便利函数
pub async fn create_arango_storage(config: &DatabaseConfig) -> Result<ArangoStorage, String> {
    ArangoStorage::new(config).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::config::{DatabaseConfig, DatabaseType};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestDocument {
        _key: String,
        name: String,
        value: i32,
    }

    #[tokio::test]
    async fn test_arango_config_from_database_config() {
        let db_config = DatabaseConfig {
            db_type: DatabaseType::ArangoDB,
            url: "http://localhost:8529".into(),
            namespace: "test_db".into(),
            database: "test_collection".into(),
            username: "test_user".into(),
            password: "test_pass".into(),
            min_connections: 5,
            max_connections: 50,
            connection_timeout: 30,
            idle_timeout: 300,
            collection_prefix: "custom_".into(),
        };

        let arango_config = ArangoConfig::from(db_config);

        assert_eq!(arango_config.url, "http://localhost:8529");
        assert_eq!(arango_config.database, "test_collection");
        assert_eq!(arango_config.username, "test_user");
        assert_eq!(arango_config.password, "test_pass");
        assert_eq!(arango_config.collection_prefix, "custom_");
    }

    #[tokio::test]
    async fn test_arango_config_default_prefix() {
        let db_config = DatabaseConfig {
            db_type: DatabaseType::ArangoDB,
            url: "http://localhost:8529".into(),
            namespace: "test_db".into(),
            database: "test_collection".into(),
            username: "test_user".into(),
            password: "test_pass".into(),
            min_connections: 5,
            max_connections: 50,
            connection_timeout: 30,
            idle_timeout: 300,
            collection_prefix: "".into(),
        };

        let arango_config = ArangoConfig::from(db_config);

        assert_eq!(arango_config.collection_prefix, "hippos_");
    }

    #[tokio::test]
    async fn test_collection_name() {
        let db_config = DatabaseConfig {
            db_type: DatabaseType::ArangoDB,
            url: "http://localhost:8529".into(),
            namespace: "test_db".into(),
            database: "test_collection".into(),
            username: "test_user".into(),
            password: "test_pass".into(),
            min_connections: 5,
            max_connections: 50,
            connection_timeout: 30,
            idle_timeout: 300,
            collection_prefix: "test_".into(),
        };

        let arango_config = ArangoConfig::from(db_config);
        let storage = ArangoStorage {
            config: arango_config,
            http_client: Arc::new(reqwest::Client::new()),
        };

        assert_eq!(storage.collection_name("sessions"), "test_sessions");
        assert_eq!(storage.collection_name("turns"), "test_turns");
        assert_eq!(
            storage.collection_name("index_records"),
            "test_index_records"
        );
    }

    #[tokio::test]
    async fn test_api_url() {
        let db_config = DatabaseConfig {
            db_type: DatabaseType::ArangoDB,
            url: "http://localhost:8529".into(),
            namespace: "test_db".into(),
            database: "test_collection".into(),
            username: "test_user".into(),
            password: "test_pass".into(),
            min_connections: 5,
            max_connections: 50,
            connection_timeout: 30,
            idle_timeout: 300,
            collection_prefix: "test_".into(),
        };

        let arango_config = ArangoConfig::from(db_config);
        let storage = ArangoStorage {
            config: arango_config,
            http_client: Arc::new(reqwest::Client::new()),
        };

        assert_eq!(storage.api_url(), "http://localhost:8529/_api");
    }

    #[tokio::test]
    async fn test_arango_config_url_trimming() {
        let db_config = DatabaseConfig {
            db_type: DatabaseType::ArangoDB,
            url: "http://localhost:8529/".into(),
            namespace: "test_db".into(),
            database: "test_collection".into(),
            username: "test_user".into(),
            password: "test_pass".into(),
            min_connections: 5,
            max_connections: 50,
            connection_timeout: 30,
            idle_timeout: 300,
            collection_prefix: "".into(),
        };

        let arango_config = ArangoConfig::from(db_config);

        // URL should have trailing slash removed
        assert_eq!(arango_config.url, "http://localhost:8529");
    }
}
