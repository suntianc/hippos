use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::debug;

use crate::{
    api::{app_state::AppState, dto::search_dto::*},
    error::AppError,
    security::auth::Claims,
};

#[derive(Deserialize)]
pub struct HybridSearchQueryParams {
    pub q: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Deserialize)]
pub struct RecentContextParams {
    pub limit: Option<u32>,
}

pub async fn semantic_search(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<String>,
    Json(request): Json<SemanticSearchRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!(
        "Semantic search for session: {}, query: {}",
        session_id, request.query
    );

    if request.query.is_empty() {
        return Err(AppError::Validation("Query cannot be empty".to_string()));
    }

    let session = state
        .session_service
        .get_by_id(&session_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Session not found: {}", session_id)))?;

    if session.tenant_id != claims.tenant_id {
        return Err(AppError::Authorization(
            "Access denied to session of another tenant".to_string(),
        ));
    }

    let start_time = std::time::Instant::now();

    let results = state
        .retrieval_service
        .semantic_search(&session_id, &request.query, request.limit.unwrap_or(10))
        .await?;

    let took_ms = start_time.elapsed().as_millis() as u64;

    let search_results: Vec<SearchResultItem> = results
        .into_iter()
        .map(|r| SearchResultItem {
            turn_id: r.turn_id,
            gist: r.gist,
            score: r.score,
            result_type: format!("{:?}", r.result_type).to_lowercase(),
            turn_number: r.turn_number,
            timestamp: r.timestamp.to_rfc3339(),
            sources: r.sources,
        })
        .collect();

    let response = SearchResponse {
        query: request.query.clone(),
        search_type: "semantic".to_string(),
        results: search_results.clone(),
        total_results: search_results.len(),
        took_ms,
    };

    Ok(Json(response))
}

pub async fn hybrid_search(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<String>,
    Query(params): Query<HybridSearchQueryParams>,
) -> Result<impl IntoResponse, AppError> {
    let query = params.q.unwrap_or_default();
    debug!(
        "Hybrid search for session: {}, query: {}",
        session_id, query
    );

    if query.is_empty() {
        return Err(AppError::Validation("Query cannot be empty".to_string()));
    }

    let session = state
        .session_service
        .get_by_id(&session_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Session not found: {}", session_id)))?;

    if session.tenant_id != claims.tenant_id {
        return Err(AppError::Authorization(
            "Access denied to session of another tenant".to_string(),
        ));
    }

    let start_time = std::time::Instant::now();

    let results = state
        .retrieval_service
        .hybrid_search(&session_id, &query, params.limit.unwrap_or(10))
        .await?;

    let took_ms = start_time.elapsed().as_millis() as u64;

    let search_results: Vec<SearchResultItem> = results
        .into_iter()
        .map(|r| SearchResultItem {
            turn_id: r.turn_id,
            gist: r.gist,
            score: r.score,
            result_type: format!("{:?}", r.result_type).to_lowercase(),
            turn_number: r.turn_number,
            timestamp: r.timestamp.to_rfc3339(),
            sources: r.sources,
        })
        .collect();

    let response = SearchResponse {
        query: query.clone(),
        search_type: "hybrid".to_string(),
        results: search_results.clone(),
        total_results: search_results.len(),
        took_ms,
    };

    Ok(Json(response))
}

pub async fn get_recent_context(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(session_id): Path<String>,
    Query(params): Query<RecentContextParams>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting recent context for session: {}", session_id);

    let session = state
        .session_service
        .get_by_id(&session_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Session not found: {}", session_id)))?;

    if session.tenant_id != claims.tenant_id {
        return Err(AppError::Authorization(
            "Access denied to session of another tenant".to_string(),
        ));
    }

    let limit = params.limit.unwrap_or(10);

    let recent = state
        .retrieval_service
        .list_recent(&session_id, limit)
        .await?;

    let turns: Vec<SearchResultItem> = recent
        .into_iter()
        .map(|r| SearchResultItem {
            turn_id: r.turn_id,
            gist: r.gist,
            score: 0.0,
            result_type: "recent".to_string(),
            turn_number: r.turn_number,
            timestamp: r.timestamp.to_rfc3339(),
            sources: vec!["recent".to_string()],
        })
        .collect();

    let response = RecentContextResponse {
        turns: turns.clone(),
        total: turns.len(),
    };

    Ok(Json(response))
}
