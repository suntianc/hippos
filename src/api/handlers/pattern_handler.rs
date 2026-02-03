//! Pattern API Handlers
//!
//! HTTP handlers for Pattern CRUD operations and search functionality.

use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    api::{app_state::AppState, dto::pattern_dto::*},
    error::AppError,
    models::pattern::{Pattern, PatternQuery, PatternType, PatternUsage},
    models::pattern_repository::PatternRepository,
    security::auth::Claims,
};

/// Create a new pattern
///
/// POST /api/v1/patterns
pub async fn create_pattern(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreatePatternRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Creating pattern for user: {}", claims.sub);

    if request.name.is_empty() {
        return Err(AppError::Validation("Name cannot be empty".to_string()));
    }

    if request.problem.is_empty() {
        return Err(AppError::Validation("Problem cannot be empty".to_string()));
    }

    if request.solution.is_empty() {
        return Err(AppError::Validation("Solution cannot be empty".to_string()));
    }

    let pattern_type: PatternType = request.pattern_type.clone().into();

    let mut pattern = Pattern::new(
        &claims.sub,
        pattern_type,
        &request.name,
        &request.problem,
        &request.solution,
    );

    if let Some(description) = request.description {
        pattern.description = description;
    }
    if let Some(trigger) = request.trigger {
        pattern.trigger = trigger;
    }
    if let Some(context) = request.context {
        pattern.context = context;
    }
    if let Some(explanation) = request.explanation {
        pattern.explanation = Some(explanation);
    }
    for tag in request.tags {
        pattern.add_tag(&tag);
    }
    pattern.is_public = request.is_public;

    let created_pattern = state
        .pattern_repository
        .create(&pattern)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = PatternResponse::from(created_pattern);

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a pattern by ID
///
/// GET /api/v1/patterns/:id
pub async fn get_pattern(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting pattern: {}", id);

    let pattern = state
        .pattern_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Pattern not found: {}", id)))?;

    // Check access: user can access if they created it, or if it's public
    if !pattern.is_public && pattern.created_by != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to pattern of another user".to_string(),
        ));
    }

    let response = PatternResponse::from(pattern);

    Ok(Json(response))
}

/// List patterns with pagination
///
/// GET /api/v1/patterns
pub async fn list_patterns(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListPatternsParams>,
) -> Result<impl IntoResponse, AppError> {
    debug!(
        "Listing patterns for user: {}, page: {:?}, page_size: {:?}",
        claims.sub, params.page, params.page_size
    );

    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100) as usize;
    let offset = ((page - 1) * page_size as u32) as usize;

    let patterns = state
        .pattern_repository
        .list(page_size, offset)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Filter to only include patterns the user created or that are public
    let filtered_patterns: Vec<Pattern> = patterns
        .into_iter()
        .filter(|p| p.created_by == claims.sub || p.is_public)
        .collect();

    let total = state
        .pattern_repository
        .count()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let pattern_responses: Vec<PatternResponse> = filtered_patterns
        .into_iter()
        .map(PatternResponse::from)
        .collect();

    let response = ListPatternsResponse {
        patterns: pattern_responses,
        total,
        page,
        page_size: page_size as u32,
    };

    Ok(Json(response))
}

/// Search patterns with various filters
///
/// POST /api/v1/patterns/search
pub async fn search_patterns(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<SearchPatternRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Searching patterns for user: {}", claims.sub);

    let start_time = std::time::Instant::now();

    let query = PatternQuery {
        types: request.types.iter().map(|t| t.clone().into()).collect(),
        tags: request.tags.clone(),
        min_confidence: request.min_confidence,
        min_success_rate: request.min_success_rate,
        keyword: request.keyword.clone(),
        created_by: Some(claims.sub.clone()),
        public_only: request.public_only,
        page: request.page,
        page_size: request.page_size,
    };

    let patterns = state
        .pattern_repository
        .search(&query)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let total = patterns.len() as u64;

    let pattern_responses: Vec<PatternResponse> = patterns
        .into_iter()
        .map(PatternResponse::from)
        .collect();

    let search_time_ms = start_time.elapsed().as_millis() as u64;

    let response = SearchPatternsResponse {
        patterns: pattern_responses,
        scores: vec![],
        total,
        search_time_ms,
    };

    Ok(Json(response))
}

/// Update a pattern
///
/// PUT /api/v1/patterns/:id
pub async fn update_pattern(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(request): Json<UpdatePatternRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Updating pattern: {}", id);

    let mut pattern = state
        .pattern_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Pattern not found: {}", id)))?;

    if pattern.created_by != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to pattern of another user".to_string(),
        ));
    }

    if let Some(name) = request.name {
        pattern.name = name;
    }
    if let Some(description) = request.description {
        pattern.description = description;
    }
    if let Some(trigger) = request.trigger {
        pattern.trigger = trigger;
    }
    if let Some(context) = request.context {
        pattern.context = context;
    }
    if let Some(problem) = request.problem {
        pattern.problem = problem;
    }
    if let Some(solution) = request.solution {
        pattern.solution = solution;
    }
    if let Some(explanation) = request.explanation {
        pattern.explanation = Some(explanation);
    }

    state
        .pattern_repository
        .update(&id, &pattern)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = UpdatePatternResponse {
        id,
        message: "Pattern updated successfully".to_string(),
    };

    Ok(Json(response))
}

/// Delete a pattern
///
/// DELETE /api/v1/patterns/:id
pub async fn delete_pattern(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Deleting pattern: {}", id);

    let pattern = state
        .pattern_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Pattern not found: {}", id)))?;

    if pattern.created_by != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to pattern of another user".to_string(),
        ));
    }

    state
        .pattern_repository
        .delete(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = DeletePatternResponse {
        id,
        message: "Pattern deleted successfully".to_string(),
    };

    Ok(Json(response))
}

/// Record pattern usage
///
/// POST /api/v1/patterns/:id/usage
pub async fn record_usage(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(request): Json<RecordUsageRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Recording usage for pattern: {}", id);

    let pattern = state
        .pattern_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Pattern not found: {}", id)))?;

    if !pattern.is_public && pattern.created_by != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to pattern of another user".to_string(),
        ));
    }

    let usage = PatternUsage {
        id: uuid::Uuid::new_v4().to_string(),
        pattern_id: id.clone(),
        user_id: claims.sub.clone(),
        input: request.input.clone(),
        output: request.output.clone(),
        outcome: request.outcome,
        feedback: request.feedback.clone(),
        used_at: Utc::now(),
        context: request.context.clone(),
    };

    let usage_id = state
        .pattern_repository
        .record_usage(&id, &usage)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = RecordUsageResponse {
        usage_id,
        pattern_id: id,
        message: "Usage recorded successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Match patterns against input text
///
/// POST /api/v1/patterns/match
pub async fn match_patterns(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<MatchPatternRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Matching patterns for user: {}", claims.sub);

    let patterns = state
        .pattern_repository
        .match_patterns(&request.input, request.max_matches)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Filter to only include patterns the user created or that are public
    let filtered_patterns: Vec<Pattern> = patterns
        .into_iter()
        .filter(|p| p.created_by == claims.sub || p.is_public)
        .collect();

    let pattern_responses: Vec<PatternResponse> = filtered_patterns
        .into_iter()
        .map(PatternResponse::from)
        .collect();

    let response = MatchPatternsResponse {
        patterns: pattern_responses,
        input: request.input.clone(),
    };

    Ok(Json(response))
}

/// Get pattern statistics
///
/// GET /api/v1/patterns/stats
pub async fn get_pattern_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting pattern stats for user: {}", claims.sub);

    let stats = state
        .pattern_repository
        .get_stats()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = PatternStatsResponse {
        total_count: stats.total_count,
        problem_solution_count: stats.problem_solution_count,
        workflow_count: stats.workflow_count,
        best_practice_count: stats.best_practice_count,
        common_error_count: stats.common_error_count,
        skill_count: stats.skill_count,
        avg_success_rate: stats.avg_success_rate,
        high_quality_count: stats.high_quality_count,
        total_usages: stats.total_usages,
        most_used_pattern: if !stats.most_used_pattern_id.is_empty() {
            Some(PatternMostUsedDto {
                pattern_id: stats.most_used_pattern_id,
                pattern_name: stats.most_used_pattern_name,
                usage_count: 0,
            })
        } else {
            None
        },
    };

    Ok(Json(response))
}

// Query parameters for listing patterns
#[derive(Debug, Deserialize, Default)]
pub struct ListPatternsParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

// Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePatternResponse {
    pub id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeletePatternResponse {
    pub id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordUsageResponse {
    pub usage_id: String,
    pub pattern_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchPatternsResponse {
    pub patterns: Vec<PatternResponse>,
    pub input: String,
}

// From implementations for DTO conversions
impl From<PatternTypeDto> for PatternType {
    fn from(dto: PatternTypeDto) -> Self {
        match dto {
            PatternTypeDto::ProblemSolution => PatternType::ProblemSolution,
            PatternTypeDto::Workflow => PatternType::Workflow,
            PatternTypeDto::BestPractice => PatternType::BestPractice,
            PatternTypeDto::CommonError => PatternType::CommonError,
            PatternTypeDto::Skill => PatternType::Skill,
        }
    }
}

impl From<PatternType> for PatternTypeDto {
    fn from(pattern_type: PatternType) -> Self {
        match pattern_type {
            PatternType::ProblemSolution => PatternTypeDto::ProblemSolution,
            PatternType::Workflow => PatternTypeDto::Workflow,
            PatternType::BestPractice => PatternTypeDto::BestPractice,
            PatternType::CommonError => PatternTypeDto::CommonError,
            PatternType::Skill => PatternTypeDto::Skill,
        }
    }
}

impl From<Pattern> for PatternResponse {
    fn from(pattern: Pattern) -> Self {
        let examples: Vec<PatternExampleDto> = pattern
            .examples
            .into_iter()
            .map(|e| PatternExampleDto {
                id: e.id,
                input: e.input,
                output: e.output,
                outcome: e.outcome,
                source_memory_id: e.source_memory_id,
                created_at: e.created_at,
            })
            .collect();

        PatternResponse {
            id: pattern.id,
            tenant_id: pattern.tenant_id,
            pattern_type: pattern.pattern_type.into(),
            name: pattern.name,
            description: pattern.description,
            trigger: pattern.trigger,
            context: pattern.context,
            problem: pattern.problem,
            solution: pattern.solution,
            explanation: pattern.explanation,
            examples,
            success_count: pattern.success_count,
            failure_count: pattern.failure_count,
            avg_outcome: pattern.avg_outcome,
            last_used: pattern.last_used,
            tags: pattern.tags,
            created_by: pattern.created_by,
            created_at: pattern.created_at,
            updated_at: pattern.updated_at,
            usage_count: pattern.usage_count,
            is_public: pattern.is_public,
            confidence: pattern.confidence,
            version: pattern.version,
        }
    }
}
