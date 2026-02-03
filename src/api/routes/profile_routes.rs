//! Profile Routes
//!
//! 定义用户档案相关的 API 路由。

use crate::api::handlers::profile_handler::*;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::api::app_state::AppState;

/// 创建档案路由器
pub fn create_profile_router() -> Router<AppState> {
    Router::new()
        // Profile CRUD operations
        .route("/profiles", post(create_profile))
        .route("/profiles", get(list_profiles))
        .route("/profiles/:id", get(get_profile))
        .route("/profiles/me", get(get_my_profile))
        .route("/profiles/:id", put(update_profile))
        .route("/profiles/:id", delete(delete_profile))
        // Profile facts
        .route("/profiles/:id/facts", post(add_fact))
        .route("/profiles/:id/facts/:fact_id/verify", post(verify_fact))
        // Profile preferences
        .route("/profiles/:id/preferences", post(add_preference))
        // Profile working hours
        .route("/profiles/:id/working-hours", put(update_working_hours))
        // Profile stats
        .route("/profiles/:id/stats", get(get_profile_stats))
}
