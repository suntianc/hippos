//! Pattern Routes
//!
//! 定义模式相关的 API 路由。

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::api::app_state::AppState;
use crate::api::handlers::pattern_handler::*;

/// 创建模式路由器
pub fn create_pattern_router() -> Router<AppState> {
    Router::new()
        .route("/patterns", get(list_patterns))
        .route("/patterns", post(create_pattern))
        .route("/patterns/:id", get(get_pattern))
        .route("/patterns/:id", put(update_pattern))
        .route("/patterns/:id", delete(delete_pattern))
        .route("/patterns/search", post(search_patterns))
        .route("/patterns/:id/usage", post(record_usage))
        .route("/patterns/match", post(match_patterns))
        .route("/patterns/stats", get(get_pattern_stats))
}
