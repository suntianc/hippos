//! Turn Routes
//!
//! 定义轮次相关的 API 路由。

use crate::api::handlers::turn_handler::*;
use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::api::app_state::AppState;

/// 创建轮次路由器
pub fn create_turn_router() -> Router<AppState> {
    Router::new()
        .route("/sessions/:session_id/turns", post(create_turn))
        .route("/sessions/:session_id/turns", get(list_turns))
        .route("/sessions/:session_id/turns/:turn_id", get(get_turn))
        .route("/sessions/:session_id/turns/:turn_id", put(update_turn))
        .route("/sessions/:session_id/turns/:turn_id", delete(delete_turn))
}
