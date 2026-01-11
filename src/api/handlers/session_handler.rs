use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::debug;

use crate::{
    api::{app_state::AppState, dto::session_dto::*},
    error::AppError,
};

pub async fn create_session(
    State(_state): State<AppState>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Creating new session: {}", request.name);

    let response = CreateSessionResponse {
        id: format!("session_{}", uuid::Uuid::new_v4()),
        created_at: chrono::Utc::now(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_sessions(
    State(_state): State<AppState>,
    Query(params): Query<ListSessionsParams>,
) -> Result<impl IntoResponse, AppError> {
    debug!(
        "Listing sessions: page={:?}, page_size={:?}",
        params.page, params.page_size
    );

    let response = SessionListResponse {
        sessions: Vec::new(),
        total: 0,
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(20),
    };

    Ok(Json(response))
}

pub async fn get_session(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting session: {}", id);

    Err::<(), AppError>(AppError::NotFound(format!("Session not found: {}", id)))
}

pub async fn update_session(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(_request): Json<UpdateSessionRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Updating session: {}", id);

    Err::<(), AppError>(AppError::NotFound(format!("Session not found: {}", id)))
}

pub async fn delete_session(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Deleting session: {}", id);

    Err::<(), AppError>(AppError::NotFound(format!("Session not found: {}", id)))
}

#[derive(Debug, Deserialize, Default)]
pub struct ListSessionsParams {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub status: Option<String>,
}
