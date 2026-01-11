use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::debug;

use crate::{
    api::{app_state::AppState, dto::turn_dto::*},
    error::AppError,
};

pub async fn create_turn(
    State(_state): State<AppState>,
    Path(session_id): Path<String>,
    Json(request): Json<CreateTurnRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Creating turn for session: {}", session_id);

    if request.content.is_empty() {
        return Err(AppError::Validation("Content cannot be empty".to_string()));
    }

    let response = CreateTurnResponse {
        id: format!("turn_{}_{}", session_id, uuid::Uuid::new_v4()),
        turn_number: 1,
        created_at: chrono::Utc::now(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_turns(
    State(_state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<ListTurnsParams>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Listing turns for session: {}", session_id);

    let response = TurnListResponse {
        turns: Vec::new(),
        total: 0,
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(50),
    };

    Ok(Json(response))
}

pub async fn get_turn(
    State(_state): State<AppState>,
    Path((session_id, turn_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting turn: {} for session: {}", turn_id, session_id);

    Err::<(), AppError>(AppError::NotFound(format!("Turn not found: {}", turn_id)))
}

pub async fn delete_turn(
    State(_state): State<AppState>,
    Path((session_id, turn_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Deleting turn: {} for session: {}", turn_id, session_id);

    Err::<(), AppError>(AppError::NotFound(format!("Turn not found: {}", turn_id)))
}

#[derive(Debug, Deserialize, Default)]
pub struct ListTurnsParams {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub message_type: Option<String>,
}
