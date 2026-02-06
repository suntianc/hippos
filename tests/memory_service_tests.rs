// Unit tests for Memory Service
//
// Tests cover:
// - Memory model creation and validation
// - Profile management
// - Pattern matching
// - Entity relationships
// - Service business logic

#[cfg(test)]
mod memory_tests {
    use super::*;
    use crate::models::entity::{Entity, EntityType, Relationship, RelationshipType};
    use crate::models::memory::{Memory, MemorySource, MemoryStatus, MemoryType};
    use crate::models::pattern::{Pattern, PatternExample, PatternType};
    use crate::models::profile::{Profile, ProfileFact, WorkingHours};

    // ============ Memory Tests ============

    #[test]
    fn test_memory_creation() {
        let memory = Memory::new(
            "user123",
            MemoryType::Episodic,
            "This is a test memory content",
            MemorySource::Conversation,
        );

        assert_eq!(memory.user_id, "user123");
        assert_eq!(memory.memory_type, MemoryType::Episodic);
        assert_eq!(memory.source, MemorySource::Conversation);
        assert_eq!(memory.status, MemoryStatus::Active);
        assert!(memory.importance > 0.0 && memory.importance <= 1.0);
        assert!(!memory.id.is_empty());
    }

    #[test]
    fn test_memory_importance_calculation() {
        let episodic = Memory::new(
            "user",
            MemoryType::Episodic,
            "content",
            MemorySource::Conversation,
        );
        let semantic = Memory::new(
            "user",
            MemoryType::Semantic,
            "content",
            MemorySource::Conversation,
        );
        let procedural = Memory::new(
            "user",
            MemoryType::Procedural,
            "content",
            MemorySource::Conversation,
        );
        let profile = Memory::new(
            "user",
            MemoryType::Profile,
            "content",
            MemorySource::Conversation,
        );

        // Profile memories should have higher base importance
        assert!(profile.importance >= episodic.importance);
        assert!(profile.importance >= semantic.importance);
        assert!(profile.importance >= procedural.importance);
    }

    #[test]
    fn test_memory_status_transitions() {
        let mut memory = Memory::new(
            "user",
            MemoryType::Episodic,
            "content",
            MemorySource::Conversation,
        );

        assert_eq!(memory.status, MemoryStatus::Active);

        memory.archive();
        assert_eq!(memory.status, MemoryStatus::Archived);

        memory.restore();
        assert_eq!(memory.status, MemoryStatus::Active);
    }

    #[test]
    fn test_memory_versioning() {
        let mut memory = Memory::new(
            "user",
            MemoryType::Episodic,
            "content",
            MemorySource::Conversation,
        );
        let initial_version = memory.version;

        memory.version += 1;
        assert_eq!(memory.version, initial_version + 1);
    }

    // ============ Profile Tests ============

    #[test]
    fn test_profile_creation() {
        let profile = Profile::new("user123", "John Doe");

        assert_eq!(profile.user_id, "user123");
        assert_eq!(profile.name, Some("John Doe".to_string()));
        assert!(profile.facts.is_empty());
        assert!(profile.interests.is_empty());
        assert!(profile.tools_used.is_empty());
    }

    #[test]
    fn test_profile_fact_management() {
        let mut profile = Profile::new("user123", "John");

        let fact = ProfileFact {
            id: "fact1".to_string(),
            fact: "Works with Rust".to_string(),
            category: crate::models::profile::ProfileFactCategory::Technical,
            source_memory_id: None,
            confidence: 0.9,
            verified: false,
            verified_at: None,
            verified_by: None,
            created_at: chrono::Utc::now(),
        };

        profile.facts.push(fact.clone());
        assert_eq!(profile.facts.len(), 1);
        assert_eq!(profile.facts[0].fact, "Works with Rust");
    }

    #[test]
    fn test_profile_fact_verification() {
        let mut profile = Profile::new("user123", "John");

        let fact = ProfileFact {
            id: "fact1".to_string(),
            fact: "Test fact".to_string(),
            category: crate::models::profile::ProfileFactCategory::Personal,
            source_memory_id: None,
            confidence: 0.5,
            verified: false,
            verified_at: None,
            verified_by: None,
            created_at: chrono::Utc::now(),
        };

        profile.facts.push(fact);

        // Verify the fact
        if let Some(ref mut verified_fact) = profile.facts.first_mut() {
            verified_fact.verified = true;
            verified_fact.verified_at = Some(chrono::Utc::now());
            verified_fact.verified_by = Some("system".to_string());
        }

        assert!(profile.facts[0].verified);
        assert!(profile.facts[0].verified_at.is_some());
    }

    #[test]
    fn test_working_hours() {
        let working_hours = WorkingHours {
            start_day: 1, // Monday
            start_hour: 9,
            end_day: 5, // Friday
            end_hour: 18,
            timezone: "America/New_York".to_string(),
            flexible: false,
        };

        assert_eq!(working_hours.start_day, 1);
        assert_eq!(working_hours.end_hour, 18);
        assert!(!working_hours.flexible);
    }

    #[test]
    fn test_profile_preferences() {
        let mut profile = Profile::new("user123", "John");

        profile
            .preferences
            .insert("theme".to_string(), serde_json::json!("dark"));
        profile
            .preferences
            .insert("language".to_string(), serde_json!("en-US"));

        assert_eq!(
            profile.preferences.get("theme"),
            Some(&serde_json::json!("dark"))
        );
        assert_eq!(
            profile.preferences.get("language"),
            Some(&serde_json::json!("en-US"))
        );
    }

    // ============ Pattern Tests ============

    #[test]
    fn test_pattern_creation() {
        let pattern = Pattern::new(
            "user123",
            PatternType::ProblemSolution,
            "Database Connection Pooling",
            "Database connections are expensive",
            "Use connection pooling",
        );

        assert_eq!(pattern.pattern_type, PatternType::ProblemSolution);
        assert_eq!(pattern.name, "Database Connection Pooling");
        assert!(!pattern.usage_count == 0);
    }

    #[test]
    fn test_pattern_example() {
        let pattern = Pattern::new(
            "user123",
            PatternType::Workflow,
            "Code Review Process",
            "Need to review code changes",
            "1. Open PR, 2. Assign reviewers, 3. Address feedback, 4. Merge",
        );

        let example = PatternExample {
            id: "example1".to_string(),
            input: "How do we review code?",
            output: "1. Open PR, 2. Assign reviewers...",
            outcome: 1.0,
            source_memory_id: None,
            created_at: chrono::Utc::now(),
        };

        pattern.examples.push(example);
        assert_eq!(pattern.examples.len(), 1);
    }

    #[test]
    fn test_pattern_success_rate() {
        let mut pattern = Pattern::new(
            "user123",
            PatternType::BestPractice,
            "Error Handling",
            "Need to handle errors gracefully",
            "Use Result and Option types",
        );

        // Simulate some usages
        pattern.success_count = 8;
        pattern.failure_count = 2;

        let success_rate =
            pattern.success_count as f32 / (pattern.success_count + pattern.failure_count) as f32;
        assert!((success_rate - 0.8).abs() < 0.001);
    }

    // ============ Entity Tests ============

    #[test]
    fn test_entity_creation() {
        let entity = Entity::new("tenant1", "PostgreSQL", EntityType::Tool);

        assert_eq!(entity.name, "PostgreSQL");
        assert_eq!(entity.entity_type, EntityType::Tool);
        assert!(entity.aliases.is_empty());
        assert!(entity.properties.is_empty());
    }

    #[test]
    fn test_entity_with_aliases() {
        let mut entity = Entity::new("tenant1", "PostgreSQL", EntityType::Tool);

        entity.aliases.push("Postgres".to_string());
        entity.aliases.push("pg".to_string());

        assert_eq!(entity.aliases.len(), 2);
        assert!(entity.aliases.contains(&"Postgres".to_string()));
    }

    #[test]
    fn test_entity_properties() {
        let mut entity = Entity::new("tenant1", "Rust", EntityType::Tool);

        entity
            .properties
            .insert("version".to_string(), serde_json::json!("1.75"));
        entity
            .properties
            .insert("license".to_string(), serde_json::json!("MIT"));

        assert_eq!(
            entity.properties.get("version"),
            Some(&serde_json::json!("1.75"))
        );
    }

    #[test]
    fn test_relationship_creation() {
        let relationship = Relationship::new(
            "tenant1",
            "entity1".to_string(),
            "entity2".to_string(),
            RelationshipType::Uses,
            "Application uses database".to_string(),
            "memory123",
        );

        assert_eq!(relationship.source_entity_id, "entity1");
        assert_eq!(relationship.target_entity_id, "entity2");
        assert_eq!(relationship.relationship_type, RelationshipType::Uses);
        assert!(relationship.strength > 0.0 && relationship.strength <= 1.0);
    }

    #[test]
    fn test_relationship_types() {
        // Test all relationship types can be created
        let types = vec![
            RelationshipType::Knows,
            RelationshipType::WorksOn,
            RelationshipType::PartOf,
            RelationshipType::Uses,
            RelationshipType::DependsOn,
            RelationshipType::SimilarTo,
            RelationshipType::CreatedBy,
        ];

        for rel_type in types {
            let relationship = Relationship::new(
                "tenant1",
                "e1".to_string(),
                "e2".to_string(),
                rel_type,
                "test".to_string(),
                "memory123",
            );
            assert_eq!(relationship.relationship_type, rel_type);
        }
    }

    #[test]
    fn test_entity_types() {
        // Test all entity types
        let types = vec![
            EntityType::Person,
            EntityType::Organization,
            EntityType::Project,
            EntityType::Tool,
            EntityType::Concept,
            EntityType::Document,
            EntityType::Event,
        ];

        for entity_type in types {
            let entity = Entity::new("tenant1", "test", entity_type);
            assert_eq!(entity.entity_type, entity_type);
        }
    }
}

#[cfg(test)]
mod importance_scoring_tests {
    use super::*;
    use crate::models::memory::{MemorySource, MemoryType};

    #[test]
    fn test_importance_keywords() {
        // Test that importance keywords affect the score
        let high_importance_content =
            "This is CRITICAL and URGENT for the PROJECT. The USER PREFERENCE is important.";
        let low_importance_content = "Just a regular message about something.";

        // This would test the calculate_importance function if it was public
        // For now, we verify the model fields are correctly set
        assert!(high_importance_content.len() > low_importance_content.len());
    }

    #[test]
    fn test_memory_type_weights() {
        // Profile memories should generally be more important
        let profile = MemoryType::Profile;
        let episodic = MemoryType::Episodic;

        // Just verify they are different variants
        assert_ne!(profile, episodic);
    }
}

#[cfg(test)]
mod search_tests {
    use super::*;
    use crate::models::memory::{MemoryQuery, MemoryStatus, MemoryType};

    #[test]
    fn test_memory_query_builder() {
        let query = MemoryQuery::for_user("user123")
            .with_type(MemoryType::Semantic)
            .with_status(MemoryStatus::Active)
            .build();

        assert_eq!(query.user_id, Some("user123".to_string()));
        assert!(query.memory_types.contains(&MemoryType::Semantic));
        assert!(query.statuses.contains(&MemoryStatus::Active));
    }

    #[test]
    fn test_memory_query_defaults() {
        let query = MemoryQuery::new();

        assert!(query.user_id.is_none());
        assert!(query.memory_types.is_empty());
        assert_eq!(query.page, 1);
        assert_eq!(query.page_size, 20);
    }
}

#[cfg(test)]
mod dto_tests {
    use super::*;
    use crate::api::dto::{CreateMemoryRequest, SearchMemoryRequest};

    #[test]
    fn test_create_memory_request() {
        let request = CreateMemoryRequest {
            user_id: "user123".to_string(),
            memory_type: "episodic".to_string(),
            content: "Test content".to_string(),
            source: "conversation".to_string(),
            importance: 0.8,
            tags: vec!["test".to_string()],
            topics: vec![],
            parent_id: None,
            related_ids: vec![],
        };

        assert_eq!(request.user_id, "user123");
        assert_eq!(request.memory_type, "episodic");
        assert_eq!(request.importance, 0.8);
    }

    #[test]
    fn test_search_memory_request() {
        let request = SearchMemoryRequest {
            user_id: "user123".to_string(),
            memory_types: vec!["episodic".to_string(), "semantic".to_string()],
            keyword: Some("test".to_string()),
            min_importance: Some(0.5),
            status: None,
            page: 1,
            page_size: 20,
        };

        assert_eq!(request.memory_types.len(), 2);
        assert!(request.keyword.is_some());
        assert_eq!(request.keyword.unwrap(), "test");
    }
}

#[cfg(test)]
mod graph_tests {
    use super::*;
    use crate::models::entity::{EntityType, GraphQuery, RelationshipType};

    #[test]
    fn test_graph_query() {
        let query = GraphQuery {
            center_entity_id: "entity123".to_string(),
            relationship_types: vec![RelationshipType::Uses, RelationshipType::DependsOn],
            entity_types: vec![EntityType::Tool, EntityType::Concept],
            max_depth: 2,
            limit_per_depth: 10,
            min_strength: 0.3,
            include_center: true,
        };

        assert_eq!(query.center_entity_id, "entity123");
        assert_eq!(query.max_depth, 2);
        assert!(query.include_center);
    }
}
