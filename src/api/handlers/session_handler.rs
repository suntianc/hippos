use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::debug;

use crate::{
    api::{app_state::AppState, dto::session_dto::*},
    error::AppError,
    security::auth::Claims,
    services::session::{Pagination, SessionQuery},
};

/// 从请求扩展中提取 tenant_id
/// 如果没有 claims，使用 "default" 作为默认租户
fn extract_tenant_id(claims: Option<&Claims>) -> String {
    claims
        .map(|c| c.tenant_id.clone())
        .unwrap_or_else(|| "default".to_string())
}

pub async fn create_session(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateSessionRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Creating new session: {}", request.name);

    let tenant_id = extract_tenant_id(Some(&claims));
    let session = state
        .session_service
        .create(&tenant_id, &request.name)
        .await?;

    let response = CreateSessionResponse {
        id: session.id,
        created_at: session.created_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn list_sessions(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListSessionsParams>,
) -> Result<impl IntoResponse, AppError> {
    debug!(
        "Listing sessions: page={:?}, page_size={:?}",
        params.page, params.page_size
    );

    let tenant_id = extract_tenant_id(Some(&claims));
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);

    let query = SessionQuery {
        pagination: Pagination::new(page, page_size),
        status: None,
    };

    let sessions = state
        .session_service
        .list(&tenant_id, query)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let total = state
        .session_service
        .count(&tenant_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let session_responses: Vec<SessionResponse> = sessions
        .into_iter()
        .map(|s| SessionResponse {
            id: s.id,
            tenant_id: s.tenant_id,
            name: s.name,
            description: s.description,
            created_at: s.created_at,
            last_active_at: s.last_active_at,
            status: format!("{:?}", s.status),
            config: SessionConfigResponse {
                summary_limit: s.config.summary_limit,
                max_turns: s.config.max_turns,
                semantic_search_enabled: s.config.semantic_search_enabled,
                auto_summarize: s.config.auto_summarize,
            },
            stats: SessionStatsResponse {
                total_turns: s.stats.total_turns,
                total_tokens: s.stats.total_tokens,
                storage_size: s.stats.storage_size,
                last_indexed_at: s.stats.last_indexed_at,
            },
        })
        .collect();

    let response = SessionListResponse {
        sessions: session_responses,
        total: total as usize,
        page,
        page_size,
    };

    Ok(Json(response))
}

pub async fn get_session(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting session: {}", id);

    let session = state
        .session_service
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Session not found: {}", id)))?;

    // 验证租户隔离
    if session.tenant_id != claims.tenant_id {
        return Err(AppError::Authorization(
            "Access denied to session of another tenant".to_string(),
        ));
    }

    let response = SessionResponse {
        id: session.id,
        tenant_id: session.tenant_id,
        name: session.name,
        description: session.description,
        created_at: session.created_at,
        last_active_at: session.last_active_at,
        status: format!("{:?}", session.status),
        config: SessionConfigResponse {
            summary_limit: session.config.summary_limit,
            max_turns: session.config.max_turns,
            semantic_search_enabled: session.config.semantic_search_enabled,
            auto_summarize: session.config.auto_summarize,
        },
        stats: SessionStatsResponse {
            total_turns: session.stats.total_turns,
            total_tokens: session.stats.total_tokens,
            storage_size: session.stats.storage_size,
            last_indexed_at: session.stats.last_indexed_at,
        },
    };

    Ok(Json(response))
}

pub async fn update_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateSessionRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Updating session: {}", id);

    let mut session = state
        .session_service
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Session not found: {}", id)))?;

    if let Some(name) = request.name {
        session.name = name;
    }
    if let Some(description) = request.description {
        session.description = Some(description);
    }
    if let Some(max_turns) = request.max_turns {
        session.config.max_turns = max_turns as usize;
    }

    session.touch();

    state
        .session_service
        .update(&session)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = UpdateSessionResponse {
        id,
        message: "Session updated successfully".to_string(),
    };

    Ok(Json(response))
}

pub async fn delete_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Deleting session: {}", id);

    state
        .session_service
        .delete(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = DeleteSessionResponse {
        id,
        message: "Session deleted successfully".to_string(),
    };

    Ok(Json(response))
}

#[derive(Debug, Deserialize, Default)]
pub struct ListSessionsParams {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
    pub status: Option<String>,
}
