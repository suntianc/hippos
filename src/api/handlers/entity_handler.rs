//! Entity API Handlers
//!
//! HTTP handlers for Entity and Relationship CRUD operations and graph queries.

use axum::{
    Json,
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::{
    api::{app_state::AppState, dto::entity_dto::*},
    error::AppError,
    models::entity::{Entity, EntityType, GraphQuery, Relationship, RelationshipType},
    models::entity_repository::EntityRepository,
    security::auth::Claims,
};

/// Create a new entity
///
/// POST /api/v1/entities
pub async fn create_entity(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<CreateEntityRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Creating entity: {} for user: {}", request.name, claims.sub);

    if request.name.is_empty() {
        return Err(AppError::Validation("Entity name cannot be empty".to_string()));
    }

    let entity = Entity::new(&request.name, request.entity_type.into());

    let mut entity = entity;
    if let Some(description) = request.description {
        entity.description = Some(description);
    }
    for alias in request.aliases {
        entity.add_alias(&alias);
    }
    for (key, value) in request.properties {
        entity.add_property(&key, value);
    }
    for memory_id in request.source_memory_ids {
        entity.add_source_memory(&memory_id);
    }

    let created_entity = state
        .entity_repository
        .create_entity(&entity)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = EntityResponse::from(created_entity);

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get an entity by ID
///
/// GET /api/v1/entities/:id
pub async fn get_entity(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting entity: {}", id);

    let entity = state
        .entity_repository
        .get_entity_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Entity not found: {}", id)))?;

    let response = EntityResponse::from(entity);

    Ok(Json(response))
}

/// Update an entity
///
/// PUT /api/v1/entities/:id
pub async fn update_entity(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(request): Json<UpdateEntityRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Updating entity: {}", id);

    let mut entity = state
        .entity_repository
        .get_entity_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Entity not found: {}", id)))?;

    if let Some(name) = request.name {
        entity.name = name;
    }
    if let Some(description) = request.description {
        entity.description = Some(description);
    }

    state
        .entity_repository
        .update_entity(&id, &entity)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = UpdateEntityResponse {
        id,
        message: "Entity updated successfully".to_string(),
    };

    Ok(Json(response))
}

/// Delete an entity
///
/// DELETE /api/v1/entities/:id
pub async fn delete_entity(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Deleting entity: {}", id);

    let entity = state
        .entity_repository
        .get_entity_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Entity not found: {}", id)))?;

    state
        .entity_repository
        .delete_entity(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = DeleteEntityResponse {
        id,
        message: "Entity deleted successfully".to_string(),
    };

    Ok(Json(response))
}

/// List entities with pagination
///
/// GET /api/v1/entities
pub async fn list_entities(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<ListEntitiesParams>,
) -> Result<impl IntoResponse, AppError> {
    debug!(
        "Listing entities for user: {}, page: {:?}, page_size: {:?}",
        claims.sub, params.page, params.page_size
    );

    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20).clamp(1, 100) as usize;
    let offset = ((page - 1) * page_size as u32) as usize;

    let entities = state
        .entity_repository
        .list_entities(page_size, offset)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let entity_responses: Vec<EntityResponse> = entities.into_iter().map(EntityResponse::from).collect();

    // For now, use entities.len() as total since we don't have a count method
    let total = entity_responses.len() as u64;

    let response = ListEntitiesResponse {
        entities: entity_responses,
        total,
        page,
        page_size: page_size as u32,
    };

    Ok(Json(response))
}

/// Search entities
///
/// POST /api/v1/entities/search
pub async fn search_entities(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<SearchEntityRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Searching entities for user: {}", claims.sub);

    let start_time = std::time::Instant::now();

    // Build search query - use name_contains for search
    let name = request.name_contains.as_deref().unwrap_or("");
    let entity_type = request.types.first().map(|t| {
        let dt: EntityType = t.clone().into();
        format!("{}", dt)
    });

    let entities = state
        .entity_repository
        .search_entities(name, entity_type.as_deref())
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let total = entities.len() as u64;

    let entity_responses: Vec<EntitySearchResultDto> = entities
        .into_iter()
        .map(|entity| EntitySearchResultDto {
            entity: EntityResponse::from(entity.clone()),
            score: entity.confidence,
            match_type: "name".to_string(),
            matched_content: entity.name,
        })
        .collect();

    let search_time_ms = start_time.elapsed().as_millis() as u64;

    let response = SearchEntitiesResponse {
        entities: entity_responses,
        total,
        search_time_ms,
    };

    Ok(Json(response))
}

/// Query the knowledge graph
///
/// POST /api/v1/entities/graph/query
pub async fn query_graph(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(request): Json<GraphQueryRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Querying graph for entity: {}", request.center_entity_id);

    let graph_query = GraphQuery {
        center_entity_id: request.center_entity_id,
        relationship_types: request
            .relationship_types
            .into_iter()
            .map(|rt| rt.into())
            .collect(),
        entity_types: request
            .entity_types
            .into_iter()
            .map(|et| et.into())
            .collect(),
        max_depth: request.max_depth,
        limit_per_depth: request.limit_per_depth,
        min_strength: request.min_strength,
        include_center: request.include_center,
    };

    let (entities, relationships) = state
        .entity_repository
        .query_graph(&graph_query)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let entity_responses: Vec<EntityResponse> = entities.into_iter().map(EntityResponse::from).collect();
    let relationship_responses: Vec<RelationshipResponse> = relationships.into_iter().map(RelationshipResponse::from).collect();

    let response = GraphQueryResponse {
        entities: entity_responses,
        relationships: relationship_responses,
        paths: Vec::new(), // Paths would need to be computed separately
    };

    Ok(Json(response))
}

/// Get graph statistics
///
/// GET /api/v1/entities/graph/stats
pub async fn get_graph_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting graph stats for user: {}", claims.sub);

    let stats = state
        .entity_repository
        .get_graph_stats()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = GraphStatsResponse {
        total_entities: stats.total_entities,
        person_count: stats.person_count,
        organization_count: stats.organization_count,
        project_count: stats.project_count,
        tool_count: stats.tool_count,
        concept_count: stats.concept_count,
        total_relationships: stats.total_relationships,
        knows_count: stats.knows_count,
        works_on_count: stats.works_on_count,
        uses_count: stats.uses_count,
        depends_on_count: stats.depends_on_count,
        similar_to_count: stats.similar_to_count,
        connected_components: stats.connected_components,
        largest_component_size: stats.largest_component_size,
        density: stats.density,
    };

    Ok(Json(response))
}

/// Create a new relationship
///
/// POST /api/v1/relationships
pub async fn create_relationship(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Json(request): Json<CreateRelationshipRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!(
        "Creating relationship from {} to {}",
        request.source_entity_id, request.target_entity_id
    );

    // Verify source entity exists
    let _source = state
        .entity_repository
        .get_entity_by_id(&request.source_entity_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Source entity not found: {}", request.source_entity_id)))?;

    // Verify target entity exists
    let _target = state
        .entity_repository
        .get_entity_by_id(&request.target_entity_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Target entity not found: {}", request.target_entity_id)))?;

    let relationship = Relationship::new(
        &request.source_entity_id,
        &request.target_entity_id,
        request.relationship_type.into(),
        &request.source_memory_id,
    );

    let mut relationship = relationship;
    relationship.strength = request.strength.clamp(0.0, 1.0);
    if let Some(context) = request.context {
        relationship.context = Some(context);
    }

    let created_relationship = state
        .entity_repository
        .create_relationship(&relationship)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = RelationshipResponse::from(created_relationship);

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a relationship by ID
///
/// GET /api/v1/relationships/:id
pub async fn get_relationship(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting relationship: {}", id);

    let relationship = state
        .entity_repository
        .get_relationship_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Relationship not found: {}", id)))?;

    let response = RelationshipResponse::from(relationship);

    Ok(Json(response))
}

/// Delete a relationship
///
/// DELETE /api/v1/relationships/:id
pub async fn delete_relationship(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Deleting relationship: {}", id);

    let relationship = state
        .entity_repository
        .get_relationship_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Relationship not found: {}", id)))?;

    state
        .entity_repository
        .delete_relationship(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = DeleteRelationshipResponse {
        id,
        message: "Relationship deleted successfully".to_string(),
    };

    Ok(Json(response))
}

/// Get all relationships for an entity
///
/// GET /api/v1/entities/:id/relationships
pub async fn get_entity_relationships(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(entity_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Getting relationships for entity: {}", entity_id);

    // Verify entity exists
    let _entity = state
        .entity_repository
        .get_entity_by_id(&entity_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Entity not found: {}", entity_id)))?;

    let relationships = state
        .entity_repository
        .get_entity_relationships(&entity_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let relationship_responses: Vec<RelationshipResponse> = relationships.into_iter().map(RelationshipResponse::from).collect();

    let total = relationship_responses.len() as u64;
    let page_size = relationship_responses.len() as u32;

    let response = ListRelationshipsResponse {
        relationships: relationship_responses,
        total,
        page: 1,
        page_size,
    };

    Ok(Json(response))
}

/// Add an alias to an entity
///
/// POST /api/v1/entities/:id/aliases
pub async fn add_entity_alias(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(request): Json<AddAliasRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Adding alias {} to entity: {}", request.alias, id);

    let mut entity = state
        .entity_repository
        .get_entity_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Entity not found: {}", id)))?;

    entity.add_alias(&request.alias);

    state
        .entity_repository
        .update_entity(&id, &entity)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = UpdateEntityResponse {
        id,
        message: "Alias added successfully".to_string(),
    };

    Ok(Json(response))
}

/// Add a property to an entity
///
/// POST /api/v1/entities/:id/properties
pub async fn add_entity_property(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(request): Json<AddPropertyRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Adding property {} to entity: {}", request.key, id);

    let mut entity = state
        .entity_repository
        .get_entity_by_id(&id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound(format!("Entity not found: {}", id)))?;

    entity.add_property(&request.key, request.value);

    state
        .entity_repository
        .update_entity(&id, &entity)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let response = UpdateEntityResponse {
        id,
        message: "Property added successfully".to_string(),
    };

    Ok(Json(response))
}

/// Discover entities from text
///
/// POST /api/v1/entities/discover
pub async fn discover_entities(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<DiscoverEntitiesRequest>,
) -> Result<impl IntoResponse, AppError> {
    debug!("Discovering entities from text for user: {}", claims.sub);

    // For now, this is a placeholder that just returns empty results
    // In a full implementation, this would use NLP to extract entities from text

    let response = DiscoverEntitiesResponse {
        entities: Vec::new(),
        relationships: Vec::new(),
        created_count: 0,
        existing_count: 0,
    };

    Ok(Json(response))
}

// DTO conversion implementations

impl From<Entity> for EntityResponse {
    fn from(entity: Entity) -> Self {
        EntityResponse {
            id: entity.id,
            tenant_id: entity.tenant_id,
            name: entity.name,
            entity_type: entity.entity_type.into(),
            description: entity.description,
            properties: entity.properties,
            aliases: entity.aliases,
            confidence: entity.confidence,
            source_memory_ids: entity.source_memory_ids,
            last_verified: entity.last_verified,
            verified: entity.verified,
            frequency: entity.frequency,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            version: entity.version,
        }
    }
}

impl From<Relationship> for RelationshipResponse {
    fn from(relationship: Relationship) -> Self {
        RelationshipResponse {
            id: relationship.id,
            tenant_id: relationship.tenant_id,
            source_entity_id: relationship.source_entity_id,
            target_entity_id: relationship.target_entity_id,
            relationship_type: relationship.relationship_type.into(),
            strength: relationship.strength,
            context: relationship.context,
            source_memory_id: relationship.source_memory_id,
            created_at: relationship.created_at,
            updated_at: relationship.updated_at,
            verified: relationship.verified,
            confidence: relationship.confidence,
            version: relationship.version,
        }
    }
}

impl From<EntityTypeDto> for EntityType {
    fn from(dto: EntityTypeDto) -> Self {
        match dto {
            EntityTypeDto::Person => EntityType::Person,
            EntityTypeDto::Organization => EntityType::Organization,
            EntityTypeDto::Project => EntityType::Project,
            EntityTypeDto::Tool => EntityType::Tool,
            EntityTypeDto::Concept => EntityType::Concept,
            EntityTypeDto::Document => EntityType::Document,
            EntityTypeDto::Event => EntityType::Event,
            EntityTypeDto::Location => EntityType::Location,
            EntityTypeDto::Product => EntityType::Product,
            EntityTypeDto::Other => EntityType::Other,
        }
    }
}

impl From<EntityType> for EntityTypeDto {
    fn from(entity_type: EntityType) -> Self {
        match entity_type {
            EntityType::Person => EntityTypeDto::Person,
            EntityType::Organization => EntityTypeDto::Organization,
            EntityType::Project => EntityTypeDto::Project,
            EntityType::Tool => EntityTypeDto::Tool,
            EntityType::Concept => EntityTypeDto::Concept,
            EntityType::Document => EntityTypeDto::Document,
            EntityType::Event => EntityTypeDto::Event,
            EntityType::Location => EntityTypeDto::Location,
            EntityType::Product => EntityTypeDto::Product,
            EntityType::Other => EntityTypeDto::Other,
        }
    }
}

impl From<RelationshipTypeDto> for RelationshipType {
    fn from(dto: RelationshipTypeDto) -> Self {
        match dto {
            RelationshipTypeDto::Knows => RelationshipType::Knows,
            RelationshipTypeDto::WorksOn => RelationshipType::WorksOn,
            RelationshipTypeDto::PartOf => RelationshipType::PartOf,
            RelationshipTypeDto::Uses => RelationshipType::Uses,
            RelationshipTypeDto::DependsOn => RelationshipType::DependsOn,
            RelationshipTypeDto::BelongsTo => RelationshipType::BelongsTo,
            RelationshipTypeDto::References => RelationshipType::References,
            RelationshipTypeDto::ConflictsWith => RelationshipType::ConflictsWith,
            RelationshipTypeDto::SimilarTo => RelationshipType::SimilarTo,
            RelationshipTypeDto::CreatedBy => RelationshipType::CreatedBy,
            RelationshipTypeDto::Contains => RelationshipType::Contains,
            RelationshipTypeDto::CompetesWith => RelationshipType::CompetesWith,
            RelationshipTypeDto::CollaboratesWith => RelationshipType::CollaboratesWith,
            RelationshipTypeDto::UsedBy => RelationshipType::UsedBy,
            RelationshipTypeDto::DependedBy => RelationshipType::DependedBy,
            RelationshipTypeDto::Owns => RelationshipType::Owns,
            RelationshipTypeDto::ReferencedBy => RelationshipType::ReferencedBy,
            RelationshipTypeDto::HasWorker => RelationshipType::HasWorker,
            RelationshipTypeDto::Created => RelationshipType::Created,
            RelationshipTypeDto::Other => RelationshipType::Other,
        }
    }
}

impl From<RelationshipType> for RelationshipTypeDto {
    fn from(relationship_type: RelationshipType) -> Self {
        match relationship_type {
            RelationshipType::Knows => RelationshipTypeDto::Knows,
            RelationshipType::WorksOn => RelationshipTypeDto::WorksOn,
            RelationshipType::PartOf => RelationshipTypeDto::PartOf,
            RelationshipType::Uses => RelationshipTypeDto::Uses,
            RelationshipType::DependsOn => RelationshipTypeDto::DependsOn,
            RelationshipType::BelongsTo => RelationshipTypeDto::BelongsTo,
            RelationshipType::References => RelationshipTypeDto::References,
            RelationshipType::ConflictsWith => RelationshipTypeDto::ConflictsWith,
            RelationshipType::SimilarTo => RelationshipTypeDto::SimilarTo,
            RelationshipType::CreatedBy => RelationshipTypeDto::CreatedBy,
            RelationshipType::Contains => RelationshipTypeDto::Contains,
            RelationshipType::CompetesWith => RelationshipTypeDto::CompetesWith,
            RelationshipType::CollaboratesWith => RelationshipTypeDto::CollaboratesWith,
            RelationshipType::UsedBy => RelationshipTypeDto::UsedBy,
            RelationshipType::DependedBy => RelationshipTypeDto::DependedBy,
            RelationshipType::Owns => RelationshipTypeDto::Owns,
            RelationshipType::ReferencedBy => RelationshipTypeDto::ReferencedBy,
            RelationshipType::HasWorker => RelationshipTypeDto::HasWorker,
            RelationshipType::Created => RelationshipTypeDto::Created,
            RelationshipType::Other => RelationshipTypeDto::Other,
        }
    }
}

/// Query parameters for listing entities
#[derive(Debug, Deserialize, Default)]
pub struct ListEntitiesParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

/// Response for entity deletion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteEntityResponse {
    pub id: String,
    pub message: String,
}

/// Response for relationship deletion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteRelationshipResponse {
    pub id: String,
    pub message: String,
}

/// Response for entity update
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEntityResponse {
    pub id: String,
    pub message: String,
}
