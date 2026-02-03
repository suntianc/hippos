//! Memory API Handlers
//!
//! HTTP handlers for Memory CRUD operations and search functionality.

use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    api::{app_state::AppState, dto::memory_dto::*},
    error::AppError,
    models::memory::{Memory, MemoryStatus},
    models::memory_repository::MemoryRepository,
    security::auth::Claims,
};

/// Create a new memory
///
/// POST /api/v1/memories
pub async fn create_memory(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateMemoryRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Creating memory for user: {}", claims.sub);

    if request.content.is_empty() {
        return Err(AppError::Validation("Content cannot be empty".to_string()));
    }

    let memory = Memory::new(
        &claims.sub,
        request.memory_type.clone(),
        &request.content,
        request.source.clone(),
    );

    let mut memory = memory;
    if let Some(source_id) = request.source_id {
        memory.source_id = Some(source_id);
    }
    if let Some(parent_id) = request.parent_id {
        memory.parent_id = Some(parent_id);
    }
    for tag in request.tags {
        memory.add_tag(&tag);
    }
    for topic in request.topics {
        memory.add_topic(&topic);
    }
    if let Some(expires_at) = request.expires_at {
        memory.expires_at = Some(expires_at);
    }

    let created_memory = state
        .memory_repository
        .create(&memory)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = MemoryResponse::from(created_memory);

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a memory by ID
///
/// GET /api/v1/memories/:id
pub async fn get_memory(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting memory: {}", id);

    let memory = state
        .memory_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Memory not found: {}", id)))?;

    if memory.user_id != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to memory of another user".to_string(),
        ));
    }

    let response = MemoryResponse::from(memory);

    Ok(Json(response))
}

/// List memories with pagination and filtering
///
/// GET /api/v1/memories
pub async fn list_memories(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListMemoriesParams>,
) -> Result<impl IntoResponse, AppError> {
    debug!(
        "Listing memories for user: {}, page: {:?}, page_size: {:?}",
        claims.sub, params.page, params.page_size
    );

    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100) as usize;
    let offset = ((page - 1) * page_size as u32) as usize;

    let memories = state
        .memory_repository
        .list_by_user(&claims.sub, params.memory_type.as_deref(), page_size, offset)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let total = state
        .memory_repository
        .count_by_user(&claims.sub)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let memory_responses: Vec<MemoryResponse> = memories.into_iter().map(MemoryResponse::from).collect();

    let response = MemoryListResponse {
        memories: memory_responses,
        total,
        page,
        page_size: page_size as u32,
        total_pages: (total as f64 / page_size as f64).ceil() as u32,
    };

    Ok(Json(response))
}

/// Search memories with various filters
///
/// POST /api/v1/memories/search
pub async fn search_memories(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<SearchMemoryRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Searching memories for user: {}", claims.sub);

    let start_time = std::time::Instant::now();

    let query = request.to_query(&claims.sub);

    let memories = state
        .memory_repository
        .search(&query)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let total = memories.len() as u64;

    let memory_responses: Vec<MemoryResponse> = memories.into_iter().map(MemoryResponse::from).collect();

    let search_time_ms = start_time.elapsed().as_millis() as u64;

    let response = SearchMemoryResponse {
        memories: memory_responses,
        total,
        page: request.page,
        page_size: request.page_size,
        total_pages: (total as f64 / request.page_size as f64).ceil() as u64,
        search_time_ms,
    };

    Ok(Json(response))
}

/// Update a memory
///
/// PUT /api/v1/memories/:id
pub async fn update_memory(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(request): Json<UpdateMemoryRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Updating memory: {}", id);

    let mut memory = state
        .memory_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Memory not found: {}", id)))?;

    if memory.user_id != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to memory of another user".to_string(),
        ));
    }

    if let Some(content) = request.content {
        memory.content = content;
    }
    if let Some(gist) = request.gist {
        memory.gist = gist;
    }
    if let Some(full_summary) = request.full_summary {
        memory.full_summary = Some(full_summary);
    }
    if let Some(importance) = request.importance {
        memory.importance = importance.clamp(0.0, 1.0);
    }
    if let Some(tags) = request.tags {
        memory.tags = tags;
    }
    if let Some(topics) = request.topics {
        memory.topics = topics;
    }
    if let Some(status) = request.status {
        memory.status = status;
    }
    if let Some(related_ids) = request.related_ids {
        memory.related_ids = related_ids;
    }

    state
        .memory_repository
        .update(&id, &memory)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = UpdateMemoryResponse {
        id,
        message: "Memory updated successfully".to_string(),
    };

    Ok(Json(response))
}

/// Delete a memory (soft delete)
///
/// DELETE /api/v1/memories/:id
pub async fn delete_memory(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Deleting memory: {}", id);

    let memory = state
        .memory_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Memory not found: {}", id)))?;

    if memory.user_id != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to memory of another user".to_string(),
        ));
    }

    state
        .memory_repository
        .delete(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = DeleteMemoryResponse {
        id,
        message: "Memory deleted successfully".to_string(),
    };

    Ok(Json(response))
}

/// Get memory statistics for the current user
///
/// GET /api/v1/memories/stats
pub async fn get_memory_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting memory stats for user: {}", claims.sub);

    let stats = state
        .memory_repository
        .get_stats(&claims.sub)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = MemoryStatsResponse {
        total_count: stats.total_count,
        episodic_count: stats.episodic_count,
        semantic_count: stats.semantic_count,
        procedural_count: stats.procedural_count,
        profile_count: stats.profile_count,
        active_count: stats.active_count,
        archived_count: stats.archived_count,
        high_importance_count: stats.high_importance_count,
        avg_importance: stats.avg_importance,
        storage_size_bytes: stats.storage_size_bytes,
    };

    Ok(Json(response))
}

/// Get memories by type
///
/// GET /api/v1/memories/type/:type
pub async fn get_memories_by_type(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(memory_type): Path<String>,
    Query(params): Query<ListMemoriesParams>,
) -> Result<impl IntoResponse, AppError> {
    debug!(
        "Listing memories by type: {} for user: {}",
        memory_type, claims.sub
    );

    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100) as usize;
    let offset = ((page - 1) * page_size as u32) as usize;

    let memory_type_str = match memory_type.to_lowercase().as_str() {
        "episodic" => Some("episodic"),
        "semantic" => Some("semantic"),
        "procedural" => Some("procedural"),
        "profile" => Some("profile"),
        _ => None,
    };

    let memories = state
        .memory_repository
        .list_by_user(&claims.sub, memory_type_str, page_size, offset)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let total = state
        .memory_repository
        .count_by_user(&claims.sub)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let memory_responses: Vec<MemoryResponse> = memories.into_iter().map(MemoryResponse::from).collect();

    let response = MemoryListResponse {
        memories: memory_responses,
        total,
        page,
        page_size: page_size as u32,
        total_pages: (total as f64 / page_size as f64).ceil() as u32,
    };

    Ok(Json(response))
}

/// Archive a memory
///
/// POST /api/v1/memories/:id/archive
pub async fn archive_memory(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Archiving memory: {}", id);

    let mut memory = state
        .memory_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Memory not found: {}", id)))?;

    if memory.user_id != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to memory of another user".to_string(),
        ));
    }

    if memory.status == MemoryStatus::Archived {
        return Err(AppError::Validation(
            "Memory is already archived".to_string(),
        ));
    }

    memory.archive();

    state
        .memory_repository
        .update(&id, &memory)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = ArchiveMemoryResponse {
        id,
        status: format!("{:?}", memory.status),
        message: "Memory archived successfully".to_string(),
    };

    Ok(Json(response))
}

/// Restore an archived memory
///
/// POST /api/v1/memories/:id/restore
pub async fn restore_memory(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Restoring memory: {}", id);

    let mut memory = state
        .memory_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Memory not found: {}", id)))?;

    if memory.user_id != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to memory of another user".to_string(),
        ));
    }

    if memory.status != MemoryStatus::Archived {
        return Err(AppError::Validation(
            "Only archived memories can be restored".to_string(),
        ));
    }

    memory.restore();

    state
        .memory_repository
        .update(&id, &memory)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = RestoreMemoryResponse {
        id,
        status: format!("{:?}", memory.status),
        message: "Memory restored successfully".to_string(),
    };

    Ok(Json(response))
}

/// Get a specific version of a memory
///
/// GET /api/v1/memories/:id/versions/:version
pub async fn get_memory_version(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((id, version)): Path<(String, u32)>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting memory version: {} version: {}", id, version);

    let memory = state
        .memory_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Memory not found: {}", id)))?;

    if memory.user_id != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to memory of another user".to_string(),
        ));
    }

    let response = MemoryVersionResponse {
        version: memory.version,
        content: memory.content,
        importance: memory.importance,
        created_at: memory.created_at,
        change_reason: None,
    };

    Ok(Json(response))
}

#[derive(Debug, Deserialize, Default)]
pub struct ListMemoriesParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub memory_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryListResponse {
    pub memories: Vec<MemoryResponse>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveMemoryResponse {
    pub id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreMemoryResponse {
    pub id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMemoryResponse {
    pub id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemoryResponse {
    pub id: String,
    pub message: String,
}
