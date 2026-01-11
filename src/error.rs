//! 错误处理模块
//!
//! 定义应用程序的错误类型和错误处理逻辑。

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// 应用程序错误类型
#[derive(Error, Debug)]
pub enum AppError {
    /// 数据库错误
    #[error("数据库错误: {0}")]
    Database(String),

    /// 连接错误
    #[error("连接错误: {0}")]
    Connection(String),

    /// 认证错误
    #[error("认证失败: {0}")]
    Authentication(String),

    /// 授权错误
    #[error("未授权访问: {0}")]
    Authorization(String),

    /// 资源不存在
    #[error("资源不存在: {0}")]
    NotFound(String),

    /// 参数验证错误
    #[error("参数验证失败: {0}")]
    Validation(String),

    /// 速率限制
    #[error("请求过于频繁，请稍后再试")]
    RateLimited,

    /// 超时错误
    #[error("操作超时: {0}")]
    Timeout(String),

    /// 配置错误
    #[error("配置错误: {0}")]
    Config(String),

    /// 序列化错误
    #[error("序列化错误: {0}")]
    Serialization(String),

    /// 向量索引错误
    #[error("向量索引错误: {0}")]
    VectorIndex(String),

    /// 嵌入模型错误
    #[error("嵌入模型错误: {0}")]
    Embedding(String),

    /// 内部错误
    #[error("内部错误: {0}")]
    Internal(String),

    /// IO 错误
    #[error("IO 错误: {0}")]
    Io(String),
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        AppError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Serialization(e.to_string())
    }
}

impl From<config::ConfigError> for AppError {
    fn from(e: config::ConfigError) -> Self {
        AppError::Config(e.to_string())
    }
}

impl From<surrealdb::Error> for AppError {
    fn from(e: surrealdb::Error) -> Self {
        AppError::Database(e.to_string())
    }
}

/// Axum response implementation for AppError
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = (&self).into();
        let body = Json(ErrorResponse::new(&code, &self.to_string()));
        (
            StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
            body,
        )
            .into_response()
    }
}

/// 错误响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// 错误代码
    pub code: String,
    /// 错误消息
    pub message: String,
    /// 详细信息
    pub details: Option<String>,
    /// 请求 ID
    pub request_id: Option<String>,
}

impl ErrorResponse {
    /// 创建新错误响应
    pub fn new(code: &str, message: &str) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            details: None,
            request_id: None,
        }
    }

    /// 添加详细信息
    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    /// 添加请求 ID
    pub fn with_request_id(mut self, request_id: &str) -> Self {
        self.request_id = Some(request_id.to_string());
        self
    }
}

/// HTTP 状态码映射
impl From<&AppError> for (u16, String) {
    fn from(err: &AppError) -> (u16, String) {
        match err {
            AppError::NotFound(_) => (404, "NOT_FOUND".to_string()),
            AppError::Authentication(_) => (401, "UNAUTHORIZED".to_string()),
            AppError::Authorization(_) => (403, "FORBIDDEN".to_string()),
            AppError::Validation(_) => (400, "BAD_REQUEST".to_string()),
            AppError::RateLimited => (429, "RATE_LIMITED".to_string()),
            AppError::Timeout(_) => (408, "TIMEOUT".to_string()),
            AppError::Connection(_) => (503, "SERVICE_UNAVAILABLE".to_string()),
            AppError::Database(_) => (500, "INTERNAL_ERROR".to_string()),
            AppError::VectorIndex(_) => (500, "INDEX_ERROR".to_string()),
            AppError::Embedding(_) => (500, "EMBEDDING_ERROR".to_string()),
            _ => (500, "INTERNAL_ERROR".to_string()),
        }
    }
}

/// 结果类型别名
pub type Result<T> = std::result::Result<T, AppError>;
