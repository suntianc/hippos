//! Session Routes
//!
//! 定义会话相关的 API 路由。

use crate::api::handlers::session_handler::*;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::api::app_state::AppState;

/// 创建会话路由器
pub fn create_session_router() -> Router<AppState> {
    Router::new()
        .route("/sessions", post(create_session))
        .route("/sessions", get(list_sessions))
        .route("/sessions/:id", get(get_session))
        .route("/sessions/:id", put(update_session))
        .route("/sessions/:id", delete(delete_session))
        .route("/sessions/:id/archive", post(archive_session))
        .route("/sessions/:id/restore", post(restore_session))
}
