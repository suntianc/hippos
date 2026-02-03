//! Memory Routes
//!
//! 定义记忆相关的 API 路由。

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::api::app_state::AppState;
use crate::api::handlers::memory_handler::*;

/// 创建记忆路由器
pub fn create_memory_router() -> Router<AppState> {
    Router::new()
        .route("/memories", get(list_memories))
        .route("/memories", post(create_memory))
        .route("/memories/:id", get(get_memory))
        .route("/memories/:id", put(update_memory))
        .route("/memories/:id", delete(delete_memory))
        .route("/memories/search", post(search_memories))
        .route("/memories/stats", get(get_memory_stats))
}
