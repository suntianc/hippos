//! Memory Builder Service
//!
//! Handles building memories from raw content with importance scoring,
//! summarization, entity extraction, and relationship building.

use std::sync::Arc;
use crate::error::Result;
use crate::models::entity::{Entity, EntityType, Relationship, RelationshipType};
use crate::models::memory::{Memory, MemorySource, MemoryType};
use crate::models::memory_repository::MemoryRepository;
use crate::models::entity_repository::EntityRepository;
use crate::services::dehydration::DehydrationService;

/// MemoryBuilder Service
///
/// Orchestrates the creation and transformation of memories:
/// - Generates embeddings (placeholder)
/// - Calculates importance scores
/// - Extracts entities and relationships
/// - Integrates with dehydration service for summarization
#[derive(Clone)]
pub struct MemoryBuilder {
    memory_repo: Arc<dyn MemoryRepository>,
    entity_repo: Arc<dyn EntityRepository>,
    dehydration_service: Arc<dyn DehydrationService>,
    min_importance: f32,
    max_importance: f32,
}

impl MemoryBuilder {
    /// Create a new MemoryBuilder
    pub fn new(
        memory_repo: Arc<dyn MemoryRepository>,
        entity_repo: Arc<dyn EntityRepository>,
        dehydration_service: Arc<dyn DehydrationService>,
    ) -> Self {
        Self {
            memory_repo,
            entity_repo,
            dehydration_service,
            min_importance: 0.0,
            max_importance: 1.0,
        }
    }

    /// Build memory from raw content
    ///
    /// This is the main entry point for creating a new memory:
    /// 1. Generate embedding (placeholder for now)
    /// 2. Calculate importance score
    /// 3. Generate gist/summary using dehydration service
    /// 4. Extract entities and relationships
    /// 5. Create and store memory
    pub async fn build_memory(
        &self,
        user_id: &str,
        content: &str,
        memory_type: MemoryType,
        source: MemorySource,
    ) -> Result<Memory> {
        tracing::info!("Building memory for user: {}, type: {:?}", user_id, memory_type);

        let memory_type = memory_type.clone();
        // Create memory with basic fields
        let mut memory = Memory::new(user_id, memory_type.clone(), content, source);

        // Step 1: Generate embedding (placeholder for now)
        // In production, this would call an embedding service
        memory.embedding = self.generate_embedding(content).await;

        // Step 2: Calculate importance score
        memory.importance = self.calculate_importance(content, memory_type);

        // Step 3: Generate gist/summary using dehydration service
        let dehydrated = self.dehydration_service.generate_summary(content).await?;
        memory.gist = dehydrated.gist.clone();
        memory.keywords = dehydrated.tags.clone();

        // Add topics from dehydration
        for topic in &dehydrated.topics {
            memory.add_topic(topic);
        }

        // Step 4: Extract entities from content
        let entities = self.extract_entities(content, &memory.id).await?;

        // Step 5: Store memory first to get the ID
        let created_memory = self.memory_repo.create(&memory).await?;

        // Step 6: Save extracted entities
        for mut entity in entities {
            entity.add_source_memory(&created_memory.id);
            if let Err(e) = self.entity_repo.create_entity(&entity).await {
                tracing::warn!("Failed to create entity {}: {}", entity.name, e);
            }
        }

        // Step 7: Build relationships with existing memories
        let related_memories = self.find_related_memories(&created_memory).await?;
        let relationships = self.build_relationships(&created_memory, &related_memories).await?;

        // Save relationships
        for relationship in relationships {
            if let Err(e) = self.entity_repo.create_relationship(&relationship).await {
                tracing::warn!("Failed to create relationship: {}", e);
            }
        }

        tracing::info!("Memory built successfully: {}", created_memory.id);

        Ok(created_memory)
    }

    /// Calculate importance score (0.0-1.0)
    ///
    /// Heuristic algorithm based on:
    /// - Content length
    /// - Keywords indicating importance
    /// - Memory type weight
    pub fn calculate_importance(&self, content: &str, memory_type: MemoryType) -> f32 {
        let mut score: f32 = 0.5; // Base score

        // Length factor: longer content tends to be more important
        let word_count = content.split_whitespace().count();
        if word_count > 100 {
            score += 0.15;
        } else if word_count > 50 {
            score += 0.1;
        } else if word_count < 10 {
            score -= 0.1;
        }

        // Character count factor
        let char_count = content.chars().count();
        if char_count > 500 {
            score += 0.1;
        }

        // Keyword-based importance detection
        let high_importance_keywords = [
            "important", "critical", "urgent", "essential", "vital",
            "关键", "重要", "紧急", "必须", "牢记",
            "remember", "never forget", "always", "绝对",
            "preference", "偏好", "喜欢", "讨厌", "hate",
            "allergy", "过敏", "禁忌", "敏感",
            "password", "密码", "secret", "机密", "账号",
        ];

        let medium_importance_keywords = [
            "should", "might", "could", "probably", "usually",
            "也许", "可能", "通常", "一般",
            "project", "项目", "task", "任务",
            "meeting", "会议", "deadline", "截止",
            "schedule", "日程", "plan", "计划",
        ];

        let content_lower = content.to_lowercase();

        for keyword in &high_importance_keywords {
            if content_lower.contains(keyword) {
                score += 0.15;
                break;
            }
        }

        for keyword in &medium_importance_keywords {
            if content_lower.contains(keyword) {
                score += 0.05;
                break;
            }
        }

        // Memory type weight
        let type_weight = match memory_type {
            MemoryType::Profile => 0.15, // User preferences are important
            MemoryType::Procedural => 0.10, // Skills are moderately important
            MemoryType::Episodic => 0.0, // Events vary in importance
            MemoryType::Semantic => 0.05, // Facts are somewhat important
        };
        score += type_weight;

        // Question detection: questions are often less important than statements
        if content.trim_start().starts_with('?')
            || content_lower.contains("how to")
            || content_lower.contains("what is")
            || content_lower.contains("?")
        {
            score -= 0.05;
        }

        // Clamp to valid range
        score.clamp(self.min_importance, self.max_importance)
    }

    /// Generate gist/summary
    ///
    /// Uses the dehydration service to generate a concise summary
    pub async fn generate_gist(&self, content: &str) -> Result<String> {
        let dehydrated = self.dehydration_service.generate_summary(content).await?;
        Ok(dehydrated.gist)
    }

    /// Extract entities from content
    ///
    /// Performs simple NER-like extraction:
    /// - Identifies capitalized words (potential proper nouns)
    /// - Identifies known entity patterns
    /// - Creates Entity records for discovered entities
    pub async fn extract_entities(&self, content: &str, memory_id: &str) -> Result<Vec<Entity>> {
        let mut entities = Vec::new();

        // Extract capitalized phrases (potential entities)
        let capitalized_pattern = regex::Regex::new(r"[A-Z][a-z]+(?:\s+[A-Z][a-z]+)*")?;
        let matches: Vec<_> = capitalized_pattern.find_iter(content).collect();

        for mat in matches {
            let name = mat.as_str().trim();
            if name.len() >= 2 && name.len() <= 50 {
                // Determine entity type based on context
                let entity_type = self.infer_entity_type(content, name);

                // Create entity if not already in list (simple dedup)
                if !entities.iter().any(|e: &Entity| e.name == name) {
                    let entity = Entity::new(name, entity_type);
                    entities.push(entity);
                }
            }
        }

        // Extract technical terms and concepts
        let tech_patterns = [
            (r"([A-Z][a-z]+(?:[A-Z][a-z]+)+)", EntityType::Concept), // CamelCase
            (r"`([^`]+)`", EntityType::Tool), // Code references
            (r#""([^"]+)""#, EntityType::Document), // Quoted terms
        ];

        for (pattern, entity_type) in tech_patterns {
            let re = regex::Regex::new(pattern)?;
            for mat in re.find_iter(content) {
                if let Some(caps) = re.captures(mat.as_str()) {
                    if let Some(term) = caps.get(1) {
                        let name = term.as_str().trim();
                        if !name.is_empty() && name.len() <= 100 {
                            if !entities.iter().any(|e: &Entity| e.name == name) {
                                let mut entity = Entity::new(name, entity_type.clone());
                                entity.description = Some(format!("Extracted from content"));
                                entities.push(entity);
                            }
                        }
                    }
                }
            }
        }

        Ok(entities
            .into_iter()
            .take(20) // Limit entities per memory
            .collect())
    }

    /// Infer entity type from context
    fn infer_entity_type(&self, content: &str, name: &str) -> EntityType {
        let content_lower = content.to_lowercase();
        let name_lower = name.to_lowercase();

        // Check for person indicators
        let person_indicators = ["he", "she", "his", "her", "him", "they", "them", "name is"];
        for indicator in &person_indicators {
            if content_lower.contains(&format!("{} {}", name_lower, indicator))
                || content_lower.contains(&format!("{} {}", indicator, name_lower))
            {
                return EntityType::Person;
            }
        }

        // Check for organization indicators
        let org_indicators = ["company", "team", "organization", "inc", "ltd", "corp", "公司", "团队", "组织"];
        for indicator in &org_indicators {
            if content_lower.contains(indicator) {
                return EntityType::Organization;
            }
        }

        // Check for project indicators
        let project_indicators = ["project", "project", "initiative", "program", "项目", "计划"];
        for indicator in &project_indicators {
            if content_lower.contains(indicator) {
                return EntityType::Project;
            }
        }

        // Check for tool/software indicators
        let tool_indicators = ["software", "tool", "app", "framework", "library", "software", "工具", "软件", "框架"];
        for indicator in &tool_indicators {
            if content_lower.contains(indicator) {
                return EntityType::Tool;
            }
        }

        // Default to concept
        EntityType::Concept
    }

    /// Build relationships between memories
    ///
    /// Analyzes content similarity and creates relationship records
    pub async fn build_relationships(
        &self,
        memory: &Memory,
        related_memories: &[Memory],
    ) -> Result<Vec<Relationship>> {
        let mut relationships = Vec::new();

        for related in related_memories {
            // Skip self
            if related.id == memory.id {
                continue;
            }

            // Calculate similarity
            let similarity = self.calculate_similarity(memory, related);

            // Only create relationship if similarity is above threshold
            if similarity >= 0.3 {
                let relationship_type = self.infer_relationship_type(memory, related);
                let strength = similarity;

                let mut relationship = Relationship::new(
                    &memory.id,
                    &related.id,
                    relationship_type,
                    &memory.id,
                );
                relationship.strength = strength;

                // Add context about why they're related
                relationship.context = Some(format!(
                    "Similarity: {:.2}, Topics: {:?}",
                    similarity,
                    memory.topics.iter().filter(|t| related.topics.contains(t)).collect::<Vec<_>>()
                ));

                relationships.push(relationship);
            }
        }

        Ok(relationships
            .into_iter()
            .take(10) // Limit relationships per memory
            .collect())
    }

    /// Calculate similarity between two memories
    fn calculate_similarity(&self, a: &Memory, b: &Memory) -> f32 {
        let mut score = 0.0;

        // Topic overlap
        let a_topics: std::collections::HashSet<_> = a.topics.iter().collect();
        let b_topics: std::collections::HashSet<_> = b.topics.iter().collect();
        let topic_overlap = if !a_topics.is_empty() {
            a_topics.intersection(&b_topics).count() as f32 / a_topics.len() as f32
        } else {
            0.0
        };
        score += topic_overlap * 0.4;

        // Tag overlap
        let a_tags: std::collections::HashSet<_> = a.tags.iter().collect();
        let b_tags: std::collections::HashSet<_> = b.tags.iter().collect();
        let tag_overlap = if !a_tags.is_empty() {
            a_tags.intersection(&b_tags).count() as f32 / a_tags.len() as f32
        } else {
            0.0
        };
        score += tag_overlap * 0.3;

        // Same type bonus
        if a.memory_type == b.memory_type {
            score += 0.2;
        }

        // Same source bonus
        if a.source == b.source {
            score += 0.1;
        }

        score.clamp(0.0, 1.0)
    }

    /// Infer relationship type between memories
    fn infer_relationship_type(&self, a: &Memory, b: &Memory) -> RelationshipType {
        // If same topic, they're related
        let a_topics: std::collections::HashSet<_> = a.topics.iter().collect();
        let b_topics: std::collections::HashSet<_> = b.topics.iter().collect();

        if !a_topics.is_empty() && a_topics.intersection(&b_topics).next().is_some() {
            return RelationshipType::SimilarTo;
        }

        // If same user preference type
        if a.memory_type == MemoryType::Profile && b.memory_type == MemoryType::Profile {
            return RelationshipType::Knows;
        }

        // Default to references
        RelationshipType::References
    }

    /// Find related memories for a given memory
    async fn find_related_memories(&self, memory: &Memory) -> Result<Vec<Memory>> {
        use crate::models::memory::MemoryQuery;

        let query = MemoryQuery::new()
            .for_user(&memory.user_id)
            .with_min_importance(0.3)
            .with_pagination(1, 20);

        let memories = self.memory_repo.search(&query).await?;

        // Filter out self
        Ok(memories.into_iter().filter(|m| m.id != memory.id).collect())
    }

    /// Generate embedding for content
    ///
    /// Placeholder for embedding generation
    /// In production, this would call an embedding model service
    async fn generate_embedding(&self, content: &str) -> Option<Vec<f32>> {
        // Placeholder: Return None for now
        // In production, this would:
        // 1. Call an embedding model (e.g., OpenAI, local model)
        // 2. Return the vector representation
        None
    }

    /// Rebuild memory with new content
    ///
    /// Updates an existing memory with new content
    pub async fn rebuild_memory(&self, memory_id: &str, new_content: &str) -> Result<Option<Memory>> {
        let existing = self.memory_repo.get_by_id(memory_id).await?;

        if let Some(mut memory) = existing {
            let memory_type = memory.memory_type.clone();

            // Update content
            memory.content = new_content.to_string();

            // Recalculate importance
            memory.importance = self.calculate_importance(new_content, memory_type);

            // Regenerate gist
            let dehydrated = self.dehydration_service.generate_summary(new_content).await?;
            memory.gist = dehydrated.gist;
            memory.keywords = dehydrated.tags;

            // Update topics
            memory.topics.clear();
            for topic in &dehydrated.topics {
                memory.add_topic(topic);
            }

            // Re-extract entities
            let entities = self.extract_entities(new_content, &memory.id).await?;
            for mut entity in entities {
                entity.add_source_memory(&memory.id);
                if let Err(e) = self.entity_repo.create_entity(&entity).await {
                    tracing::warn!("Failed to create entity during rebuild: {}", e);
                }
            }

            // Rebuild relationships
            let related_memories = self.find_related_memories(&memory).await?;
            let relationships = self.build_relationships(&memory, &related_memories).await?;

            // Update memory
            let updated = self.memory_repo.update(memory_id, &memory).await?;

            // Save relationships
            for relationship in relationships {
                if let Err(e) = self.entity_repo.create_relationship(&relationship).await {
                    tracing::warn!("Failed to create relationship during rebuild: {}", e);
                }
            }

            return Ok(updated);
        }

        Ok(None)
    }

    /// Batch build memories from multiple content items
    pub async fn batch_build_memory(
        &self,
        user_id: &str,
        items: Vec<(&str, MemoryType, MemorySource)>,
    ) -> Result<Vec<Memory>> {
        let mut memories = Vec::new();

        for (content, memory_type, source) in items {
            match self.build_memory(user_id, content, memory_type, source).await {
                Ok(memory) => memories.push(memory),
                Err(e) => tracing::error!("Failed to build memory: {}", e),
            }
        }

        Ok(memories)
    }
}

/// Create a MemoryBuilder service
pub fn create_memory_builder(
    memory_repo: Arc<dyn MemoryRepository>,
    entity_repo: Arc<dyn EntityRepository>,
    dehydration_service: Arc<dyn DehydrationService>,
) -> MemoryBuilder {
    MemoryBuilder::new(memory_repo, entity_repo, dehydration_service)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::memory_repository::MemoryRepository;
    use crate::models::entity_repository::EntityRepository;
    use crate::services::dehydration::DehydrationService;
    use crate::models::turn::DehydratedData;
    use async_trait::async_trait;

    #[derive(Clone)]
    struct MockDehydrationService;

    #[async_trait]
    impl DehydrationService for MockDehydrationService {
        async fn generate_summary(&self, content: &str) -> Result<DehydratedData> {
            Ok(DehydratedData {
                gist: content.chars().take(100).collect(),
                topics: vec!["test".to_string()],
                tags: vec!["test".to_string()],
                embedding: None,
                generated_at: chrono::Utc::now(),
                generator: Some("mock".to_string()),
            })
        }

        async fn extract_keywords(&self, content: &str) -> Result<Vec<String>> {
            Ok(vec!["test".to_string()])
        }

        async fn extract_topics(&self, content: &str) -> Result<Vec<String>> {
            Ok(vec!["test".to_string()])
        }
    }

    #[derive(Clone)]
    struct MockMemoryRepository;

    #[async_trait]
    impl MemoryRepository for MockMemoryRepository {
        async fn create(&self, memory: &Memory) -> Result<Memory> {
            Ok(memory.clone())
        }

        async fn get_by_id(&self, _id: &str) -> Result<Option<Memory>> {
            Ok(None)
        }

        async fn update(&self, _id: &str, memory: &Memory) -> Result<Option<Memory>> {
            Ok(Some(memory.clone()))
        }

        async fn delete(&self, _id: &str) -> Result<bool> {
            Ok(true)
        }

        async fn list(&self, _limit: usize, _start: usize) -> Result<Vec<Memory>> {
            Ok(vec![])
        }

        async fn count(&self) -> Result<u64> {
            Ok(0)
        }

        async fn list_by_user(
            &self,
            _user_id: &str,
            _memory_type: Option<&str>,
            _limit: usize,
            _start: usize,
        ) -> Result<Vec<Memory>> {
            Ok(vec![])
        }

        async fn count_by_user(&self, _user_id: &str) -> Result<u64> {
            Ok(0)
        }

        async fn search(&self, _query: &crate::models::memory::MemoryQuery) -> Result<Vec<Memory>> {
            Ok(vec![])
        }

        async fn get_stats(&self, _user_id: &str) -> Result<crate::models::memory::MemoryStats> {
            Ok(crate::models::memory::MemoryStats {
                user_id: _user_id.to_string(),
                total_count: 0,
                episodic_count: 0,
                semantic_count: 0,
                procedural_count: 0,
                profile_count: 0,
                active_count: 0,
                archived_count: 0,
                avg_importance: 0.0,
                high_importance_count: 0,
                storage_size_bytes: 0,
            })
        }
    }

    #[derive(Clone)]
    struct MockEntityRepository;

    #[async_trait]
    impl EntityRepository for MockEntityRepository {
        async fn create_entity(&self, entity: &Entity) -> Result<Entity> {
            Ok(entity.clone())
        }

        async fn get_entity_by_id(&self, _id: &str) -> Result<Option<Entity>> {
            Ok(None)
        }

        async fn update_entity(&self, _id: &str, entity: &Entity) -> Result<Option<Entity>> {
            Ok(Some(entity.clone()))
        }

        async fn delete_entity(&self, _id: &str) -> Result<bool> {
            Ok(true)
        }

        async fn list_entities(&self, _limit: usize, _start: usize) -> Result<Vec<Entity>> {
            Ok(vec![])
        }

        async fn search_entities(&self, _name: &str, _entity_type: Option<&str>) -> Result<Vec<Entity>> {
            Ok(vec![])
        }

        async fn create_relationship(&self, relationship: &Relationship) -> Result<Relationship> {
            Ok(relationship.clone())
        }

        async fn get_relationship_by_id(&self, _id: &str) -> Result<Option<Relationship>> {
            Ok(None)
        }

        async fn delete_relationship(&self, _id: &str) -> Result<bool> {
            Ok(true)
        }

        async fn get_entity_relationships(&self, _entity_id: &str) -> Result<Vec<Relationship>> {
            Ok(vec![])
        }

        async fn query_graph(
            &self,
            _query: &crate::models::entity::GraphQuery,
        ) -> Result<(Vec<Entity>, Vec<Relationship>)> {
            Ok((vec![], vec![]))
        }

        async fn get_graph_stats(&self) -> Result<crate::models::entity::GraphStats> {
            Ok(crate::models::entity::GraphStats {
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

        async fn discover_entity(&self, _name: &str, _entity_type: &str) -> Result<Option<Entity>> {
            Ok(None)
        }
    }

    #[tokio::test]
    async fn test_calculate_importance() {
        let memory_repo = Arc::new(MockMemoryRepository);
        let entity_repo = Arc::new(MockEntityRepository);
        let dehydration_service = Arc::new(MockDehydrationService);

        let builder = MemoryBuilder::new(memory_repo, entity_repo, dehydration_service);

        // Test basic content
        let score = builder.calculate_importance("This is a test message", MemoryType::Episodic);
        assert!(score >= 0.0 && score <= 1.0);

        // Test with important keywords
        let score = builder.calculate_importance(
            "This is important and critical information",
            MemoryType::Profile,
        );
        assert!(score > 0.5);

        // Test with preference keywords
        let score = builder.calculate_importance(
            "I prefer dark mode and I always use Vim",
            MemoryType::Profile,
        );
        assert!(score > 0.5);

        // Test short content
        let score = builder.calculate_importance("Hi", MemoryType::Episodic);
        assert!(score < 0.6);
    }

    #[tokio::test]
    async fn test_generate_gist() {
        let memory_repo = Arc::new(MockMemoryRepository);
        let entity_repo = Arc::new(MockEntityRepository);
        let dehydration_service = Arc::new(MockDehydrationService);

        let builder = MemoryBuilder::new(memory_repo, entity_repo, dehydration_service);

        let gist = builder.generate_gist("This is a longer test message that should be summarized").await.unwrap();
        assert!(!gist.is_empty());
    }

    #[tokio::test]
    async fn test_build_memory() {
        let memory_repo = Arc::new(MockMemoryRepository);
        let entity_repo = Arc::new(MockEntityRepository);
        let dehydration_service = Arc::new(MockDehydrationService);

        let builder = MemoryBuilder::new(memory_repo, entity_repo, dehydration_service);

        let memory = builder
            .build_memory(
                "user_123",
                "This is a test memory about Rust programming",
                MemoryType::Episodic,
                MemorySource::Conversation,
            )
            .await
            .unwrap();

        assert_eq!(memory.user_id, "user_123");
        assert_eq!(memory.memory_type, MemoryType::Episodic);
        assert!(!memory.gist.is_empty());
        assert!(memory.importance >= 0.0 && memory.importance <= 1.0);
    }

    #[tokio::test]
    async fn test_batch_build_memory() {
        let memory_repo = Arc::new(MockMemoryRepository);
        let entity_repo = Arc::new(MockEntityRepository);
        let dehydration_service = Arc::new(MockDehydrationService);

        let builder = MemoryBuilder::new(memory_repo, entity_repo, dehydration_service);

        let items = vec![
            ("Content 1", MemoryType::Episodic, MemorySource::Conversation),
            ("Content 2", MemoryType::Semantic, MemorySource::Research),
            ("Content 3", MemoryType::Procedural, MemorySource::Execution),
        ];

        let memories = builder.batch_build_memory("user_123", items).await.unwrap();
        assert_eq!(memories.len(), 3);
    }

    #[tokio::test]
    async fn test_extract_entities() {
        let memory_repo = Arc::new(MockMemoryRepository);
        let entity_repo = Arc::new(MockEntityRepository);
        let dehydration_service = Arc::new(MockDehydrationService);

        let builder = MemoryBuilder::new(memory_repo, entity_repo, dehydration_service);

        let entities = builder
            .extract_entities("I worked with John on the Rust Project using VSCode", "memory_123")
            .await
            .unwrap();

        // Should extract at least some entities
        assert!(!entities.is_empty());
    }

    #[tokio::test]
    async fn test_similarity_calculation() {
        let memory_repo = Arc::new(MockMemoryRepository);
        let entity_repo = Arc::new(MockEntityRepository);
        let dehydration_service = Arc::new(MockDehydrationService);

        let builder = MemoryBuilder::new(memory_repo, entity_repo, dehydration_service);

        let memory_a = Memory::new("user", MemoryType::Episodic, "content", MemorySource::Conversation);
        let mut memory_b = Memory::new("user", MemoryType::Episodic, "content", MemorySource::Conversation);
        memory_b.add_topic("test");

        let similarity = builder.calculate_similarity(&memory_a, &memory_b);
        assert!(similarity >= 0.0 && similarity <= 1.0);
    }
}
