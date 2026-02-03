//! Profile API Handlers
//!
//! HTTP handlers for Profile CRUD operations and fact management.

use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    api::{app_state::AppState, dto::profile_dto::*},
    error::AppError,
    models::profile::{Profile, ProfileFactCategory},
    models::profile_repository::ProfileRepository,
    security::auth::Claims,
};

/// Create a new profile
///
/// POST /api/v1/profiles
pub async fn create_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateProfileRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Creating profile for user: {}", claims.sub);

    // Check if profile already exists for this user
    let existing = state
        .profile_repository
        .get_by_user_id(&claims.sub)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if existing.is_some() {
        return Err(AppError::Conflict(
            "Profile already exists for this user".to_string(),
        ));
    }

    let mut profile = Profile::new(&claims.sub);

    if let Some(name) = request.name {
        profile.name = Some(name);
    }
    if let Some(role) = request.role {
        profile.role = Some(role);
    }
    if let Some(organization) = request.organization {
        profile.organization = Some(organization);
    }
    if let Some(location) = request.location {
        profile.location = Some(location);
    }
    if let Some(language) = request.language {
        profile.language = Some(language);
    }
    for tool in request.tools_used {
        profile.add_tool(&tool);
    }
    for interest in request.interests {
        profile.add_interest(&interest);
    }

    let created_profile = state
        .profile_repository
        .create(&profile)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = ProfileResponse::from(created_profile);

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a profile by ID
///
/// GET /api/v1/profiles/:id
pub async fn get_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting profile: {}", id);

    let profile = state
        .profile_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Profile not found: {}", id)))?;

    // Verify ownership
    if profile.user_id != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to profile of another user".to_string(),
        ));
    }

    let response = ProfileResponse::from(profile);

    Ok(Json(response))
}

/// Get current user's profile
///
/// GET /api/v1/profiles/me
pub async fn get_my_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting profile for user: {}", claims.sub);

    let profile = state
        .profile_repository
        .get_by_user_id(&claims.sub)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Profile not found for current user".to_string()))?;

    let response = ProfileResponse::from(profile);

    Ok(Json(response))
}

/// Update a profile
///
/// PUT /api/v1/profiles/:id
pub async fn update_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Updating profile: {}", id);

    let mut profile = state
        .profile_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Profile not found: {}", id)))?;

    if profile.user_id != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to profile of another user".to_string(),
        ));
    }

    if let Some(name) = request.name {
        profile.name = Some(name);
    }
    if let Some(role) = request.role {
        profile.role = Some(role);
    }
    if let Some(organization) = request.organization {
        profile.organization = Some(organization);
    }
    if let Some(location) = request.location {
        profile.location = Some(location);
    }
    if let Some(communication_style) = request.communication_style {
        profile.communication_style = Some(communication_style);
    }
    if let Some(technical_level) = request.technical_level {
        profile.technical_level = Some(technical_level);
    }
    if let Some(language) = request.language {
        profile.language = Some(language);
    }

    profile.updated_at = chrono::Utc::now();
    profile.version += 1;

    state
        .profile_repository
        .update(&id, &profile)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = UpdateProfileResponse {
        id,
        message: "Profile updated successfully".to_string(),
    };

    Ok(Json(response))
}

/// List profiles with pagination
///
/// GET /api/v1/profiles
pub async fn list_profiles(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListProfilesParams>,
) -> Result<impl IntoResponse, AppError> {
    debug!(
        "Listing profiles for user: {}, page: {:?}, page_size: {:?}",
        claims.sub, params.page, params.page_size
    );

    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100) as usize;
    let offset = ((page - 1) * page_size as u32) as usize;

    let profiles = state
        .profile_repository
        .list(page_size, offset)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Filter to only include profiles owned by the current user
    let user_profiles: Vec<Profile> = profiles
        .into_iter()
        .filter(|p| p.user_id == claims.sub)
        .collect();

    let total = user_profiles.len() as u64;

    let profile_responses: Vec<ProfileResponse> = user_profiles.into_iter().map(ProfileResponse::from).collect();

    let response = ListProfilesResponse {
        profiles: profile_responses,
        total,
        page,
        page_size: page_size as u32,
    };

    Ok(Json(response))
}

/// Delete a profile
///
/// DELETE /api/v1/profiles/:id
pub async fn delete_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Deleting profile: {}", id);

    let profile = state
        .profile_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Profile not found: {}", id)))?;

    if profile.user_id != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to profile of another user".to_string(),
        ));
    }

    state
        .profile_repository
        .delete(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = DeleteProfileResponse {
        id,
        message: "Profile deleted successfully".to_string(),
    };

    Ok(Json(response))
}

/// Add a fact to a profile
///
/// POST /api/v1/profiles/:id/facts
pub async fn add_fact(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(request): Json<AddFactRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Adding fact to profile: {}", id);

    if request.fact.is_empty() {
        return Err(AppError::Validation("Fact cannot be empty".to_string()));
    }

    let mut profile = state
        .profile_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Profile not found: {}", id)))?;

    if profile.user_id != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to profile of another user".to_string(),
        ));
    }

    let category: ProfileFactCategory = request.category.into();

    profile.add_fact(
        &request.fact,
        category,
        request.source_memory_id.as_deref(),
        request.confidence,
    );

    state
        .profile_repository
        .update(&id, &profile)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = AddFactResponse {
        fact_id: profile.facts.last().map(|f| f.id.clone()).unwrap_or_default(),
        message: "Fact added successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Verify a fact
///
/// POST /api/v1/profiles/:id/facts/:fact_id/verify
pub async fn verify_fact(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((id, fact_id)): Path<(String, String)>,
    Json(request): Json<VerifyFactRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Verifying fact: {} for profile: {}", fact_id, id);

    let mut profile = state
        .profile_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Profile not found: {}", id)))?;

    if profile.user_id != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to profile of another user".to_string(),
        ));
    }

    let verified = profile.verify_fact(&fact_id, request.verified_by.as_deref());

    if !verified {
        return Err(AppError::NotFound(format!("Fact not found: {}", fact_id)));
    }

    state
        .profile_repository
        .update(&id, &profile)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = VerifyFactResponse {
        fact_id,
        verified: true,
        verified_at: profile.last_verified.map(|t| t.to_rfc3339()),
        message: "Fact verified successfully".to_string(),
    };

    Ok(Json(response))
}

/// Get profile statistics
///
/// GET /api/v1/profiles/:id/stats
pub async fn get_profile_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting profile stats: {}", id);

    let profile = state
        .profile_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Profile not found: {}", id)))?;

    if profile.user_id != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to profile of another user".to_string(),
        ));
    }

    let verified_facts_count = profile.verified_facts_count() as u64;
    let total_facts_count = profile.facts.len() as u64;

    // Calculate average confidence
    let avg_confidence = if profile.facts.is_empty() {
        0.0
    } else {
        profile.facts.iter().map(|f| f.confidence).sum::<f32>() / profile.facts.len() as f32
    };

    // Calculate category stats
    let mut category_counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    for fact in &profile.facts {
        let category_name = format!("{:?}", fact.category).to_lowercase();
        *category_counts.entry(category_name).or_insert(0) += 1;
    }

    let category_stats: Vec<ProfileCategoryStat> = category_counts
        .into_iter()
        .map(|(category, count)| ProfileCategoryStat { category, count })
        .collect();

    let response = ProfileStatsResponse {
        total_count: total_facts_count,
        avg_confidence,
        verified_facts_count,
        category_stats,
    };

    Ok(Json(response))
}

/// Add a preference to a profile
///
/// POST /api/v1/profiles/:id/preferences
pub async fn add_preference(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(request): Json<AddPreferenceRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Adding preference to profile: {}", id);

    if request.key.is_empty() {
        return Err(AppError::Validation("Preference key cannot be empty".to_string()));
    }

    let mut profile = state
        .profile_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Profile not found: {}", id)))?;

    if profile.user_id != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to profile of another user".to_string(),
        ));
    }

    profile.add_preference(&request.key, request.value, request.reason.as_deref());

    state
        .profile_repository
        .update(&id, &profile)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = AddPreferenceResponse {
        key: request.key,
        message: "Preference added successfully".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Update working hours
///
/// PUT /api/v1/profiles/:id/working-hours
pub async fn update_working_hours(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(request): Json<UpdateWorkingHoursRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Updating working hours for profile: {}", id);

    let mut profile = state
        .profile_repository
        .get_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Profile not found: {}", id)))?;

    if profile.user_id != claims.sub {
        return Err(AppError::Authorization(
            "Access denied to profile of another user".to_string(),
        ));
    }

    profile.working_hours = Some(crate::models::profile::WorkingHours {
        start_day: request.start_day,
        start_hour: request.start_hour,
        end_day: request.end_day,
        end_hour: request.end_hour,
        timezone: request.timezone,
        flexible: request.flexible,
    });

    profile.updated_at = chrono::Utc::now();
    profile.version += 1;

    state
        .profile_repository
        .update(&id, &profile)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = UpdateWorkingHoursResponse {
        id,
        message: "Working hours updated successfully".to_string(),
    };

    Ok(Json(response))
}

// Helper conversions

impl From<ProfileFactCategoryDto> for ProfileFactCategory {
    fn from(dto: ProfileFactCategoryDto) -> Self {
        match dto {
            ProfileFactCategoryDto::Personal => ProfileFactCategory::Personal,
            ProfileFactCategoryDto::Professional => ProfileFactCategory::Professional,
            ProfileFactCategoryDto::Technical => ProfileFactCategory::Technical,
            ProfileFactCategoryDto::Project => ProfileFactCategory::Project,
            ProfileFactCategoryDto::Communication => ProfileFactCategory::Communication,
            ProfileFactCategoryDto::Lifestyle => ProfileFactCategory::Lifestyle,
            ProfileFactCategoryDto::Other => ProfileFactCategory::Other,
        }
    }
}

impl From<ProfileFactCategory> for ProfileFactCategoryDto {
    fn from(category: ProfileFactCategory) -> Self {
        match category {
            ProfileFactCategory::Personal => ProfileFactCategoryDto::Personal,
            ProfileFactCategory::Professional => ProfileFactCategoryDto::Professional,
            ProfileFactCategory::Technical => ProfileFactCategoryDto::Technical,
            ProfileFactCategory::Project => ProfileFactCategoryDto::Project,
            ProfileFactCategory::Communication => ProfileFactCategoryDto::Communication,
            ProfileFactCategory::Lifestyle => ProfileFactCategoryDto::Lifestyle,
            ProfileFactCategory::Other => ProfileFactCategoryDto::Other,
        }
    }
}

impl From<Profile> for ProfileResponse {
    fn from(profile: Profile) -> Self {
        let facts_dto: Vec<ProfileFactDto> = profile.facts.into_iter().map(|f| f.into()).collect();

        let working_hours_dto = profile.working_hours.map(|wh| WorkingHoursDto {
            start_day: wh.start_day,
            start_hour: wh.start_hour,
            end_day: wh.end_day,
            end_hour: wh.end_hour,
            timezone: wh.timezone,
            flexible: wh.flexible,
        });

        ProfileResponse {
            id: profile.id,
            tenant_id: profile.tenant_id,
            user_id: profile.user_id,
            name: profile.name,
            role: profile.role,
            organization: profile.organization,
            location: profile.location,
            preferences: profile.preferences,
            communication_style: profile.communication_style,
            technical_level: profile.technical_level,
            language: profile.language,
            facts: facts_dto,
            interests: profile.interests,
            working_hours: working_hours_dto,
            common_tasks: profile.common_tasks,
            tools_used: profile.tools_used,
            created_at: profile.created_at,
            updated_at: profile.updated_at,
            confidence: profile.confidence,
            last_verified: profile.last_verified,
            version: profile.version,
        }
    }
}

impl From<crate::models::profile::ProfileFact> for ProfileFactDto {
    fn from(fact: crate::models::profile::ProfileFact) -> Self {
        ProfileFactDto {
            id: fact.id,
            fact: fact.fact,
            category: fact.category.into(),
            source_memory_id: fact.source_memory_id,
            confidence: fact.confidence,
            verified: fact.verified,
            verified_at: fact.verified_at,
            verified_by: fact.verified_by,
            created_at: fact.created_at,
        }
    }
}

// Query params

#[derive(Debug, Deserialize, Default)]
pub struct ListProfilesParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

// Response types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateProfileResponse {
    pub id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteProfileResponse {
    pub id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddFactResponse {
    pub fact_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyFactResponse {
    pub fact_id: String,
    pub verified: bool,
    pub verified_at: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddPreferenceResponse {
    pub key: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkingHoursResponse {
    pub id: String,
    pub message: String,
}
