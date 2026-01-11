//! Search Routes
//!
//! 定义搜索相关的 API 路由。

use crate::api::handlers::search_handler::*;
use axum::{
    Router,
    routing::{get, post},
};

use crate::api::app_state::AppState;

/// 创建搜索路由器
pub fn create_search_router() -> Router<AppState> {
    Router::new()
        .route("/sessions/:session_id/search", get(hybrid_search))
        .route(
            "/sessions/:session_id/search/semantic",
            post(semantic_search),
        )
        .route(
            "/sessions/:session_id/context/recent",
            get(get_recent_context),
        )
}
