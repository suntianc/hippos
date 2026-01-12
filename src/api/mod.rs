//! API 模块
//!
//! 提供 REST API 支持。

#[cfg(test)]
mod api_tests;
pub mod app_state;
pub mod dto;
pub mod handlers;
pub mod routes;

use crate::api::app_state::AppState;
use crate::error::AppError;
use crate::security::middleware::{auth_middleware, security_headers_middleware};
use axum::Router;

pub fn create_router(app_state: AppState) -> Router {
    let authenticator = app_state.authenticator.clone();

    let api = Router::new()
        .merge(routes::session_routes::create_session_router())
        .merge(routes::turn_routes::create_turn_router())
        .merge(routes::search_routes::create_search_router());

    Router::new()
        .nest("/api/v1", api)
        .layer(axum::middleware::from_fn(security_headers_middleware))
        .layer(axum::middleware::from_fn(move |req, next| {
            auth_middleware(req, next, authenticator.clone())
        }))
        .with_state(app_state)
}

pub async fn initialize_api(app_state: AppState) -> Result<Router, AppError> {
    tracing::info!("Initializing API router...");
    Ok(create_router(app_state))
}
