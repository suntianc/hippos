//! Session Routes
//!
//! 定义会话相关的 API 路由。

use crate::api::handlers::session_handler::*;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::api::app_state::AppState;

/// 创建会话路由器
pub fn create_session_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_session))
        .route("/", get(list_sessions))
        .route("/:id", get(get_session))
        .route("/:id", put(update_session))
        .route("/:id", delete(delete_session))
}
