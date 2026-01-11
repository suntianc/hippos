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
    models::turn::Turn,
    services::turn::TurnQuery,
};

pub async fn create_turn(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(request): Json<CreateTurnRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Creating turn for session: {}", session_id);

    if request.content.is_empty() {
        return Err(AppError::Validation("Content cannot be empty".to_string()));
    }

    let turn = state
        .turn_service
        .create(&session_id, &request.content, None)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = CreateTurnResponse {
        id: turn.id,
        turn_number: turn.turn_number,
        created_at: turn.metadata.timestamp,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_turns(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<ListTurnsParams>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Listing turns for session: {}", session_id);

    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(50);

    let query = TurnQuery {
        page,
        page_size,
        message_type: params.message_type.clone(),
    };

    let turns = state
        .turn_service
        .list_by_session(&session_id, query)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let turn_responses: Vec<TurnResponse> = turns
        .into_iter()
        .map(|t| convert_turn_to_response(t))
        .collect();

    let total = turn_responses.len();

    let response = TurnListResponse {
        turns: turn_responses,
        total,
        page,
        page_size,
    };

    Ok(Json(response))
}

pub async fn get_turn(
    State(state): State<AppState>,
    Path((session_id, turn_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting turn: {} for session: {}", turn_id, session_id);

    let turn = state
        .turn_service
        .get_by_id(&turn_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Turn not found: {}", turn_id)))?;

    if turn.session_id != session_id {
        return Err(AppError::NotFound(format!("Turn not found: {}", turn_id)));
    }

    let response = convert_turn_to_response(turn);

    Ok(Json(response))
}

pub async fn delete_turn(
    State(state): State<AppState>,
    Path((session_id, turn_id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Deleting turn: {} for session: {}", turn_id, session_id);

    let turn = state
        .turn_service
        .get_by_id(&turn_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Turn not found: {}", turn_id)))?;

    if turn.session_id != session_id {
        return Err(AppError::NotFound(format!("Turn not found: {}", turn_id)));
    }

    state
        .turn_service
        .delete(&turn_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = DeleteTurnResponse {
        id: turn_id,
        message: "Turn deleted successfully".to_string(),
    };

    Ok(Json(response))
}

fn convert_turn_to_response(turn: Turn) -> TurnResponse {
    let metadata = TurnMetadataResponse {
        timestamp: turn.metadata.timestamp,
        user_id: turn.metadata.user_id,
        message_type: format!("{:?}", turn.metadata.message_type),
        role: turn.metadata.role,
        model: turn.metadata.model,
        token_count: turn.metadata.token_count,
    };

    let dehydrated = turn.dehydrated.map(|d| DehydratedDataResponse {
        gist: d.gist,
        topics: d.topics,
        tags: d.tags,
        generated_at: d.generated_at,
        generator: d.generator,
    });

    TurnResponse {
        id: turn.id,
        session_id: turn.session_id,
        turn_number: turn.turn_number,
        raw_content: turn.raw_content,
        metadata,
        dehydrated,
        status: format!("{:?}", turn.status),
        parent_id: turn.parent_id,
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct ListTurnsParams {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub message_type: Option<String>,
}
