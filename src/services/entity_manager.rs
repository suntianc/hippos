//! Entity Manager Service
//!
//! Provides comprehensive entity and relationship management including:
//! - Entity CRUD operations
//! - Relationship management
//! - Knowledge graph operations
//! - Entity discovery from text
//! - Entity disambiguation and merging

use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::models::entity::{
    Entity, EntityType, Relationship, RelationshipType,
    GraphQuery, GraphResult, GraphStats, GraphPath,
    EntitySearchResult,
};
use crate::models::entity_repository::EntityRepository;

/// Entity updates input
#[derive(Debug, Clone, Default)]
pub struct EntityUpdates {
    /// Name update
    pub name: Option<String>,

    /// Description update
    pub description: Option<String>,

    /// Entity type update
    pub entity_type: Option<EntityType>,

    /// Properties to update (merge with existing)
    pub properties: Option<std::collections::HashMap<String, serde_json::Value>>,

    /// Aliases to add
    pub add_aliases: Option<Vec<String>>,

    /// Aliases to remove
    pub remove_aliases: Option<Vec<String>>,

    /// Confidence update
    pub confidence: Option<f32>,
}

/// Discovery result containing discovered entities and relationships
#[derive(Debug, Clone, Default)]
pub struct DiscoveryResult {
    /// Discovered entities
    pub entities: Vec<Entity>,

    /// Discovered relationships
    pub relationships: Vec<Relationship>,

    /// Entities that were already known
    pub existing_entities: Vec<Entity>,

    /// Total processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Graph traversal result with paths
#[derive(Debug, Clone)]
pub struct GraphTraversalResult {
    /// Entities found in traversal
    pub entities: Vec<Entity>,

    /// Relationships found
    pub relationships: Vec<Relationship>,

    /// Paths from center to discovered entities
    pub paths: Vec<GraphPath>,

    /// Maximum depth reached
    pub max_depth_reached: u32,

    /// Total entities found
    pub total_entities: usize,
}

/// Entity similarity for disambiguation
#[derive(Debug, Clone)]
pub struct EntitySimilarity {
    /// Source entity
    pub source_entity: Entity,

    /// Target entity
    pub target_entity: Entity,

    /// Similarity score (0.0 - 1.0)
    pub score: f32,

    /// Matching fields
    pub matching_fields: Vec<String>,
}

/// Entity Manager Service
///
/// Orchestrates entity and relationship operations with business logic:
/// - Creates, retrieves, updates, and deletes entities
/// - Manages relationships between entities
/// - Queries and traverses knowledge graph
/// - Discovers entities from text content
/// - Handles entity disambiguation and merging
#[derive(Clone)]
pub struct EntityManager {
    entity_repo: Arc<dyn EntityRepository>,
}

impl EntityManager {
    /// Create a new EntityManager
    pub fn new(entity_repo: Arc<dyn EntityRepository>) -> Self {
        Self { entity_repo }
    }

    /// Create a new entity
    ///
    /// Persists a new entity to the repository.
    pub async fn create_entity(&self, entity: &Entity) -> Result<Entity> {
        tracing::info!("Creating entity: {} (type: {})", entity.name, entity.entity_type);

        let entity = entity.clone();
        self.entity_repo.create_entity(&entity).await
    }

    /// Get entity by ID
    ///
    /// Retrieves an entity by its unique identifier.
    pub async fn get_entity(&self, entity_id: &str) -> Result<Option<Entity>> {
        tracing::debug!("Getting entity: {}", entity_id);

        self.entity_repo.get_entity_by_id(entity_id).await
    }

    /// Search entities by name
    ///
    /// Performs a fuzzy search for entities matching the given name.
    pub async fn search_entities(
        &self,
        name: &str,
        entity_type: Option<&str>,
    ) -> Result<Vec<Entity>> {
        tracing::info!("Searching entities: {} (type: {:?})", name, entity_type);

        self.entity_repo.search_entities(name, entity_type).await
    }

    /// Update entity
    ///
    /// Applies updates to an existing entity and returns the updated version.
    pub async fn update_entity(
        &self,
        entity_id: &str,
        updates: &EntityUpdates,
    ) -> Result<Option<Entity>> {
        tracing::info!("Updating entity: {}", entity_id);

        let mut entity = self
            .entity_repo
            .get_entity_by_id(entity_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Entity not found: {}", entity_id)))?;
        if let Some(name) = &updates.name {
            entity.name = name.clone();
        }
        if let Some(description) = &updates.description {
            entity.description = Some(description.clone());
        }
        if let Some(entity_type) = &updates.entity_type {
            entity.entity_type = entity_type.clone();
        }
        if let Some(properties) = &updates.properties {
            for (key, value) in properties {
                entity.properties.insert(key.clone(), value.clone());
            }
        }
        if let Some(aliases) = &updates.add_aliases {
            for alias in aliases {
                entity.add_alias(alias);
            }
        }
        if let Some(aliases) = &updates.remove_aliases {
            entity.aliases.retain(|a| !aliases.contains(a));
        }
        if let Some(confidence) = updates.confidence {
            entity.confidence = confidence.clamp(0.0, 1.0);
        }

        entity.updated_at = Utc::now();
        entity.version += 1;

        self.entity_repo.update_entity(entity_id, &entity).await
    }

    /// Delete entity
    ///
    /// Removes an entity and all its relationships from the graph.
    pub async fn delete_entity(&self, entity_id: &str) -> Result<bool> {
        tracing::info!("Deleting entity: {}", entity_id);

        self.entity_repo.delete_entity(entity_id).await
    }

    /// Create a relationship
    ///
    /// Establishes a new relationship between two entities.
    pub async fn create_relationship(&self, relationship: &Relationship) -> Result<Relationship> {
        tracing::info!(
            "Creating relationship: {} -> {} (type: {})",
            relationship.source_entity_id,
            relationship.target_entity_id,
            relationship.relationship_type
        );

        let relationship = relationship.clone();
        self.entity_repo.create_relationship(&relationship).await
    }

    /// Get entity relationships
    ///
    /// Retrieves all relationships for a given entity (both outgoing and incoming).
    pub async fn get_entity_relationships(&self, entity_id: &str) -> Result<Vec<Relationship>> {
        tracing::debug!("Getting relationships for entity: {}", entity_id);

        self.entity_repo.get_entity_relationships(entity_id).await
    }

    /// Delete relationship
    ///
    /// Removes a specific relationship from the graph.
    pub async fn delete_relationship(&self, relationship_id: &str) -> Result<bool> {
        tracing::info!("Deleting relationship: {}", relationship_id);

        self.entity_repo.delete_relationship(relationship_id).await
    }

    /// Query knowledge graph
    ///
    /// Traverses the graph from a center entity to discover connected entities.
    pub async fn query_graph(
        &self,
        center_entity_id: &str,
        max_depth: u32,
        limit_per_depth: u32,
    ) -> Result<GraphResult> {
        tracing::info!(
            "Querying graph from entity: {} (depth: {}, limit: {})",
            center_entity_id,
            max_depth,
            limit_per_depth
        );

        let query = GraphQuery {
            center_entity_id: center_entity_id.to_string(),
            relationship_types: Vec::new(),
            entity_types: Vec::new(),
            max_depth,
            limit_per_depth,
            min_strength: 0.0,
            include_center: true,
        };

        let (entities, relationships) = self.entity_repo.query_graph(&query).await?;
        let paths = self.build_graph_paths(center_entity_id, &entities, &relationships);

        Ok(GraphResult {
            entities,
            relationships,
            paths,
        })
    }

    /// Discover entities from text
    ///
    /// Analyzes text content to extract and create entities and relationships.
    pub async fn discover_entities(
        &self,
        text: &str,
        source_memory_id: &str,
    ) -> Result<DiscoveryResult> {
        tracing::info!("Discovering entities from text (memory: {})", source_memory_id);

        let start_time = Utc::now();
        let mut result = DiscoveryResult::default();

        let extracted_names = self.extract_entity_names(text);

        for name in extracted_names {
            if let Some(existing) = self.entity_repo.discover_entity(&name, "other").await? {
                result.existing_entities.push(existing);
                continue;
            }

            let mut entity = Entity::new(&name, EntityType::Other);
            entity.add_source_memory(source_memory_id);
            entity.confidence = self.calculate_entity_confidence(&name, text);

            match self.entity_repo.create_entity(&entity).await {
                Ok(new_entity) => {
                    result.entities.push(new_entity);
                }
                Err(e) => {
                    tracing::warn!("Failed to create entity '{}': {}", name, e);
                }
            }
        }

        let relationships = self.extract_relationships(text, source_memory_id);
        for relationship in relationships {
            let source_exists = self.entity_repo.get_entity_by_id(&relationship.source_entity_id).await?.is_some();
            let target_exists = self.entity_repo.get_entity_by_id(&relationship.target_entity_id).await?.is_some();

            if source_exists && target_exists {
                match self.entity_repo.create_relationship(&relationship).await {
                    Ok(rel) => result.relationships.push(rel),
                    Err(e) => tracing::warn!("Failed to create relationship: {}", e),
                }
            }
        }

        result.processing_time_ms = (Utc::now() - start_time).num_milliseconds() as u64;

        Ok(result)
    }

    /// Merge entities (disambiguation)
    ///
    /// Combines a source entity into a target entity, resolving conflicts.
    pub async fn merge_entities(&self, target_id: &str, source_id: &str) -> Result<Entity> {
        tracing::info!("Merging entity {} into {}", source_id, target_id);

        // Get both entities
        let target = self
            .entity_repo
            .get_entity_by_id(target_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Target entity not found: {}", target_id)))?;

        let source = self
            .entity_repo
            .get_entity_by_id(source_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Source entity not found: {}", source_id)))?;

        let mut merged = if target.confidence >= source.confidence {
            target.clone()
        } else {
            source.clone()
        };
        merged.id = target_id.to_string();

        for alias in source.aliases {
            if !merged.aliases.contains(&alias) {
                merged.aliases.push(alias);
            }
        }

        for (key, value) in source.properties {
            merged.properties.insert(key, value);
        }

        for memory_id in source.source_memory_ids {
            if !merged.source_memory_ids.contains(&memory_id) {
                merged.source_memory_ids.push(memory_id);
            }
        }

        merged.frequency = merged.frequency.saturating_add(source.frequency);
        merged.confidence = (target.confidence + source.confidence) / 2.0;

        merged.updated_at = Utc::now();
        merged.version += 1;

        let result = self.entity_repo.update_entity(target_id, &merged).await?
            .ok_or_else(|| crate::error::AppError::Database("Failed to update merged entity".to_string()))?;

        self.entity_repo.delete_entity(source_id).await?;
        self.redirect_relationships(source_id, target_id).await?;

        tracing::info!("Successfully merged entity {} into {}", source_id, target_id);

        Ok(result)
    }

    /// Get graph statistics
    ///
    /// Returns statistics about the knowledge graph.
    pub async fn get_graph_stats(&self) -> Result<GraphStats> {
        tracing::debug!("Getting graph statistics");

        self.entity_repo.get_graph_stats().await
    }

    /// Find similar entities for disambiguation
    ///
    /// Searches for entities that might be the same entity with different names.
    pub async fn find_similar_entities(&self, entity_id: &str) -> Result<Vec<EntitySimilarity>> {
        tracing::info!("Finding similar entities for: {}", entity_id);

        let entity = self
            .entity_repo
            .get_entity_by_id(entity_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Entity not found: {}", entity_id)))?;

        // Search for entities with similar names
        let similar = self.entity_repo.search_entities(&entity.name, None).await?;

        let mut similarities = Vec::new();

        for other in similar {
            if other.id == entity_id {
                continue;
            }

            let score = self.calculate_similarity(&entity, &other);
            let matching_fields = self.get_matching_fields(&entity, &other);

            if score > 0.5 {
                similarities.push(EntitySimilarity {
                    source_entity: entity.clone(),
                    target_entity: other,
                    score,
                    matching_fields,
                });
            }
        }

        similarities.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        Ok(similarities)
    }

    /// Get connected entities (neighbors)
    ///
    /// Returns entities directly connected to the given entity.
    pub async fn get_connected_entities(
        &self,
        entity_id: &str,
        relationship_types: Option<Vec<RelationshipType>>,
        min_strength: Option<f32>,
    ) -> Result<Vec<(Entity, Relationship)>> {
        tracing::debug!("Getting connected entities for: {}", entity_id);

        let relationships = self.entity_repo.get_entity_relationships(entity_id).await?;

        let mut connected = Vec::new();
        let min_strength = min_strength.unwrap_or(0.0);

        for rel in relationships {
            if rel.strength < min_strength {
                continue;
            }
            if let Some(ref types) = relationship_types {
                if !types.contains(&rel.relationship_type) {
                    continue;
                }
            }

            let connected_id = if rel.source_entity_id == entity_id {
                &rel.target_entity_id
            } else {
                &rel.source_entity_id
            };

            if let Some(connected_entity) = self.entity_repo.get_entity_by_id(connected_id).await? {
                connected.push((connected_entity, rel));
            }
        }

        Ok(connected)
    }

    /// Verify relationship
    ///
    /// Marks a relationship as verified.
    pub async fn verify_relationship(&self, relationship_id: &str) -> Result<bool> {
        tracing::info!("Verifying relationship: {}", relationship_id);

        let mut relationship = self
            .entity_repo
            .get_relationship_by_id(relationship_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Relationship not found: {}", relationship_id)))?;

        relationship.verify();

        self.entity_repo
            .update_relationship(relationship_id, &relationship)
            .await?;

        Ok(true)
    }

    /// Get entity by name (exact match)
    ///
    /// Retrieves an entity by its exact name.
    pub async fn get_entity_by_name(&self, name: &str) -> Result<Option<Entity>> {
        tracing::debug!("Getting entity by name: {}", name);

        self.entity_repo.discover_entity(name, "all").await
    }

    /// Add alias to entity
    ///
    /// Adds an alternative name for an entity.
    pub async fn add_alias(&self, entity_id: &str, alias: &str) -> Result<bool> {
        tracing::info!("Adding alias '{}' to entity: {}", alias, entity_id);

        let mut entity = self
            .entity_repo
            .get_entity_by_id(entity_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Entity not found: {}", entity_id)))?;

        entity.add_alias(alias);

        self.entity_repo.update_entity(entity_id, &entity).await?;
        Ok(true)
    }

    /// Increment entity frequency
    ///
    /// Increases the frequency count for an entity (used for importance ranking).
    pub async fn increment_frequency(&self, entity_id: &str) -> Result<bool> {
        tracing::debug!("Incrementing frequency for entity: {}", entity_id);

        let mut entity = self
            .entity_repo
            .get_entity_by_id(entity_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Entity not found: {}", entity_id)))?;

        entity.increment_frequency();

        self.entity_repo.update_entity(entity_id, &entity).await?;
        Ok(true)
    }

    /// Helper: Build graph paths from entities and relationships
    fn build_graph_paths(
        &self,
        center_id: &str,
        entities: &[Entity],
        relationships: &[Relationship],
    ) -> Vec<GraphPath> {
        let mut paths = Vec::new();

        for entity in entities {
            if entity.id == center_id {
                continue;
            }

            // Find shortest path to this entity
            if let Some(path) = self.find_shortest_path(center_id, &entity.id, relationships) {
                paths.push(path);
            }
        }

        paths
    }

    /// Helper: Find shortest path between two entities
    fn find_shortest_path(
        &self,
        from_id: &str,
        to_id: &str,
        relationships: &[Relationship],
    ) -> Option<GraphPath> {
        // Simple BFS for shortest path
        let mut queue = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut came_from = std::collections::HashMap::new();
        let mut rel_from = std::collections::HashMap::new();

        queue.push(from_id.to_string());
        visited.insert(from_id.to_string());

        while let Some(current) = queue.pop() {
            if current == to_id {
                let mut entity_ids = Vec::new();
                let mut relationship_ids = Vec::new();
                let mut current_id = to_id.to_string();
                let mut strength = 1.0;

                while current_id != from_id {
                    entity_ids.insert(0, current_id.clone());
                    if let Some(rel_id) = rel_from.get(&current_id) {
                        relationship_ids.insert(0, rel_id.clone());
                        if let Some(rel) = relationships.iter().find(|r| r.id == *rel_id) {
                            strength = strength.min(rel.strength);
                        }
                    }
                    current_id = came_from.get(&current_id)?.clone();
                }
                entity_ids.insert(0, from_id.to_string());

                return Some(GraphPath {
                    entity_ids,
                    relationship_ids,
                    length: relationship_ids.len() as u32,
                    strength,
                });
            }

            for rel in relationships {
                let neighbor = if rel.source_entity_id == current {
                    &rel.target_entity_id
                } else if rel.target_entity_id == current {
                    &rel.source_entity_id
                } else {
                    continue
                };

                if !visited.contains(neighbor) {
                    visited.insert(neighbor.clone());
                    came_from.insert(neighbor.clone(), current.clone());
                    rel_from.insert(neighbor.clone(), rel.id.clone());
                    queue.push(neighbor.clone());
                }
            }
        }

        None
    }

    /// Helper: Extract entity names from text
    fn extract_entity_names(&self, text: &str) -> Vec<String> {
        let mut names = Vec::new();

        let re = regex::Regex::new(r"[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*").unwrap();
        for mat in re.find_iter(text) {
            let name = mat.as_str().to_string();
            if name.len() > 2 && !names.contains(&name) {
                names.push(name);
            }
        }

        names
    }

    /// Helper: Extract relationships from text
    fn extract_relationships(&self, text: &str, source_memory_id: &str) -> Vec<Relationship> {
        let mut relationships = Vec::new();

        let patterns = vec![
            ("uses", RelationshipType::Uses),
            ("depends on", RelationshipType::DependsOn),
            ("works on", RelationshipType::WorksOn),
            ("part of", RelationshipType::PartOf),
            ("belongs to", RelationshipType::BelongsTo),
        ];

        for (pattern, rel_type) in patterns {
            if text.contains(pattern) {
                let re = regex::Regex::new(&format!(r"(\w+(?:\s+\w+)*)\s+{}\s+(\w+(?:\s+\w+)*)", pattern)).unwrap();
                for mat in re.find_iter(text) {
                    let caps = re.captures(mat.as_str()).unwrap();
                    if let (Some(source), Some(target)) = (caps.get(1), caps.get(2)) {
                        let source_entity = self.entity_repo.discover_entity(source.as_str(), "all").await;
                        let target_entity = self.entity_repo.discover_entity(target.as_str(), "all").await;

                        if let (Ok(Some(source_ent)), Ok(Some(target_ent))) = (source_entity, target_entity) {
                            let rel = Relationship::new(
                                &source_ent.id,
                                &target_ent.id,
                                rel_type,
                                source_memory_id,
                            );
                            relationships.push(rel);
                        }
                    }
                }
            }
        }

        relationships
    }

    /// Helper: Calculate entity confidence from text
    fn calculate_entity_confidence(&self, name: &str, text: &str) -> f32 {
        let count = text.matches(name).count();
        let base_confidence = (count as f32 / 10.0).clamp(0.0, 1.0);

        let mut boost = 0.0;
        if text.starts_with(name) {
            boost = 0.1;
        }

        (base_confidence + boost).clamp(0.1, 0.9)
    }

    /// Helper: Calculate similarity between two entities
    fn calculate_similarity(&self, a: &Entity, b: &Entity) -> f32 {
        let mut score = 0.0;
        let mut weights = 0.0;

        let name_sim = string_similarity(&a.name, &b.name);
        score += name_sim * 0.4;
        weights += 0.4;

        let a_aliases: std::collections::HashSet<_> = a.aliases.iter().collect();
        let b_aliases: std::collections::HashSet<_> = b.aliases.iter().collect();
        let alias_overlap = if !a_aliases.is_empty() && !b_aliases.is_empty() {
            let intersection: std::collections::HashSet<_> = a_aliases.intersection(&b_aliases).collect();
            intersection.len() as f32 / (a_aliases.len().max(b_aliases.len()) as f32)
        } else {
            0.0
        };
        score += alias_overlap * 0.3;
        weights += 0.3;

        let a_props: std::collections::HashSet<_> = a.properties.keys().collect();
        let b_props: std::collections::HashSet<_> = b.properties.keys().collect();
        let prop_overlap = if !a_props.is_empty() && !b_props.is_empty() {
            let intersection: std::collections::HashSet<_> = a_props.intersection(&b_props).collect();
            intersection.len() as f32 / (a_props.len().max(b_props.len()) as f32)
        } else {
            0.0
        };
        score += prop_overlap * 0.2;
        weights += 0.2;

        if a.entity_type == b.entity_type {
            score += 0.1;
            weights += 0.1;
        }

        if weights > 0.0 {
            score / weights
        } else {
            0.0
        }
    }

    /// Helper: Get matching fields between two entities
    fn get_matching_fields(&self, a: &Entity, b: &Entity) -> Vec<String> {
        let mut fields = Vec::new();

        if string_similarity(&a.name, &b.name) > 0.8 {
            fields.push("name".to_string());
        }

        let a_aliases: std::collections::HashSet<_> = a.aliases.iter().collect();
        let b_aliases: std::collections::HashSet<_> = b.aliases.iter().collect();
        if !a_aliases.is_empty() && !b_aliases.is_empty() {
            let intersection: std::collections::HashSet<_> = a_aliases.intersection(&b_aliases).collect();
            if !intersection.is_empty() {
                fields.push("aliases".to_string());
            }
        }

        if a.entity_type == b.entity_type {
            fields.push("entity_type".to_string());
        }

        fields
    }

    /// Helper: Redirect relationships from source to target
    async fn redirect_relationships(&self, source_id: &str, target_id: &str) -> Result<()> {
        let relationships = self.entity_repo.get_entity_relationships(source_id).await?;

        for rel in relationships {
            let mut updated = rel.clone();

            if rel.source_entity_id == source_id {
                updated.source_entity_id = target_id.to_string();
            }
            if rel.target_entity_id == source_id {
                updated.target_entity_id = target_id.to_string();
            }

            updated.updated_at = Utc::now();
            updated.version += 1;

            self.entity_repo.delete_relationship(&rel.id).await?;
            self.entity_repo.create_relationship(&updated).await?;
        }

        Ok(())
    }
}

/// Simple string similarity (Jaccard on character n-grams)
fn string_similarity(a: &str, b: &str) -> f32 {
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }

    let a_lower = a.to_lowercase();
    let b_lower = b.to_lowercase();

    if a_lower == b_lower {
        return 1.0;
    }

    // Character n-grams (n=2)
    let a_ngrams: std::collections::HashSet<String> = (0..a_lower.len().saturating_sub(1))
        .map(|i| (&a_lower[i..i+2]).to_string())
        .collect();

    let b_ngrams: std::collections::HashSet<String> = (0..b_lower.len().saturating_sub(1))
        .map(|i| (&b_lower[i..i+2]).to_string())
        .collect();

    let intersection: std::collections::HashSet<_> = a_ngrams.intersection(&b_ngrams).collect();
    let union: std::collections::HashSet<_> = a_ngrams.union(&b_ngrams).collect();

    if union.is_empty() {
        0.0
    } else {
        intersection.len() as f32 / union.len() as f32
    }
}

/// Create an EntityManager service
pub fn create_entity_manager(
    entity_repo: Arc<dyn EntityRepository>,
) -> EntityManager {
    EntityManager::new(entity_repo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::entity_repository::EntityRepository;
    use async_trait::async_trait;

    #[derive(Clone)]
    struct MockEntityRepository;

    #[async_trait]
    impl EntityRepository for MockEntityRepository {
        async fn create_entity(&self, entity: &Entity) -> Result<Entity> {
            Ok(entity.clone())
        }

        async fn get_entity_by_id(&self, id: &str) -> Result<Option<Entity>> {
            if id == "existing_entity" {
                let entity = Entity::new("Test Entity", EntityType::Person);
                return Ok(Some(entity));
            }
            Ok(None)
        }

        async fn update_entity(&self, id: &str, entity: &Entity) -> Result<Option<Entity>> {
            Ok(Some(entity.clone()))
        }

        async fn delete_entity(&self, _id: &str) -> Result<bool> {
            Ok(true)
        }

        async fn list_entities(&self, _limit: usize, _start: usize) -> Result<Vec<Entity>> {
            Ok(vec![])
        }

        async fn search_entities(&self, name: &str, _entity_type: Option<&str>) -> Result<Vec<Entity>> {
            if name == "Test" {
                let entity = Entity::new("Test Entity", EntityType::Person);
                return Ok(vec![entity]);
            }
            Ok(vec![])
        }

        async fn create_relationship(&self, relationship: &Relationship) -> Result<Relationship> {
            Ok(relationship.clone())
        }

        async fn get_relationship_by_id(&self, _id: &str) -> Result<Option<Relationship>> {
            Ok(None)
        }

        async fn update_relationship(&self, _id: &str, relationship: &Relationship) -> Result<Option<Relationship>> {
            Ok(Some(relationship.clone()))
        }

        async fn delete_relationship(&self, _id: &str) -> Result<bool> {
            Ok(true)
        }

        async fn get_entity_relationships(&self, _entity_id: &str) -> Result<Vec<Relationship>> {
            Ok(vec![])
        }

        async fn query_graph(&self, _query: &GraphQuery) -> Result<(Vec<Entity>, Vec<Relationship>)> {
            Ok((vec![], vec![]))
        }

        async fn get_graph_stats(&self) -> Result<GraphStats> {
            Ok(GraphStats {
                total_entities: 0,
                person_count: 0,
                organization_count: 0,
                project_count: 0,
                tool_count: 0,
                concept_count: 0,
                total_relationships: 0,
                knows_count: 0,
                works_on_count: 0,
                uses_count: 0,
                depends_on_count: 0,
                similar_to_count: 0,
                connected_components: 0,
                largest_component_size: 0,
                density: 0.0,
            })
        }

        async fn discover_entity(&self, name: &str, _entity_type: &str) -> Result<Option<Entity>> {
            if name == "Existing" {
                let entity = Entity::new("Existing Entity", EntityType::Person);
                return Ok(Some(entity));
            }
            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_create_entity() {
        let repo = Arc::new(MockEntityRepository);
        let manager = EntityManager::new(repo);

        let entity = Entity::new("Test Entity", EntityType::Person);
        let result = manager.create_entity(&entity).await.unwrap();

        assert_eq!(result.name, "Test Entity");
        assert_eq!(result.entity_type, EntityType::Person);
    }

    #[tokio::test]
    async fn test_get_entity_existing() {
        let repo = Arc::new(MockEntityRepository);
        let manager = EntityManager::new(repo);

        let result = manager.get_entity("existing_entity").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Test Entity");
    }

    #[tokio::test]
    async fn test_get_entity_not_found() {
        let repo = Arc::new(MockEntityRepository);
        let manager = EntityManager::new(repo);

        let result = manager.get_entity("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_search_entities() {
        let repo = Arc::new(MockEntityRepository);
        let manager = EntityManager::new(repo);

        let result = manager.search_entities("Test", None).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "Test Entity");
    }

    #[tokio::test]
    async fn test_update_entity() {
        let repo = Arc::new(MockEntityRepository);
        let manager = EntityManager::new(repo);

        let updates = EntityUpdates {
            name: Some("Updated Name".to_string()),
            description: Some("New description".to_string()),
            ..Default::default()
        };

        let result = manager.update_entity("existing_entity", &updates).await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "Updated Name");
    }

    #[tokio::test]
    async fn test_delete_entity() {
        let repo = Arc::new(MockEntityRepository);
        let manager = EntityManager::new(repo);

        let result = manager.delete_entity("entity_123").await.unwrap();
        assert!(result);
    }

    #[tokio::test]
    async fn test_get_graph_stats() {
        let repo = Arc::new(MockEntityRepository);
        let manager = EntityManager::new(repo);

        let stats = manager.get_graph_stats().await.unwrap();
        assert_eq!(stats.total_entities, 0);
    }

    #[tokio::test]
    async fn test_string_similarity() {
        assert!((string_similarity("Rust", "rust") - 1.0).abs() < 0.01);
        assert!((string_similarity("TypeScript", "Typescript") - 1.0).abs() < 0.01);
        assert!(string_similarity("Rust", "Python") < 0.5);
        assert!(string_similarity("Rust", "Rust") - 1.0 < 0.01);
    }

    #[tokio::test]
    async fn test_add_alias() {
        let repo = Arc::new(MockEntityRepository);
        let manager = EntityManager::new(repo);

        let result = manager.add_alias("existing_entity", "te").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_increment_frequency() {
        let repo = Arc::new(MockEntityRepository);
        let manager = EntityManager::new(repo);

        let result = manager.increment_frequency("existing_entity").await;
        assert!(result.is_ok());
    }
}
