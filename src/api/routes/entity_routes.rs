//! Entity Routes
//!
//! 定义实体和关系相关的 API 路由。

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::api::app_state::AppState;
use crate::api::handlers::entity_handler::*;

/// 创建实体路由器
pub fn create_entity_router() -> Router<AppState> {
    Router::new()
        // Entity CRUD routes
        .route("/entities", get(list_entities))
        .route("/entities", post(create_entity))
        .route("/entities/:id", get(get_entity))
        .route("/entities/:id", put(update_entity))
        .route("/entities/:id", delete(delete_entity))
        // Entity search and discovery routes
        .route("/entities/search", post(search_entities))
        .route("/entities/discover", post(discover_entities))
        // Entity relationships routes
        .route("/entities/:id/relationships", get(get_entity_relationships))
        .route("/entities/:id/aliases", post(add_entity_alias))
        .route("/entities/:id/properties", post(add_entity_property))
        // Graph routes
        .route("/entities/graph", post(query_graph))
        .route("/entities/graph/stats", get(get_graph_stats))
}

/// 创建关系路由器
pub fn create_relationship_router() -> Router<AppState> {
    Router::new()
        .route("/relationships", post(create_relationship))
        .route("/relationships/:id", get(get_relationship))
        .route("/relationships/:id", delete(delete_relationship))
}
