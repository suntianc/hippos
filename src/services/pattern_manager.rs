//! Pattern Manager Service
//!
//! Provides comprehensive pattern/knowledge management including:
//! - Pattern CRUD operations with outcome tracking
//! - Pattern matching and discovery
//! - Recommendations based on user context
//! - Integration with memory repository for context-aware pattern discovery

use std::sync::Arc;
use async_trait::async_trait;
use chrono::Utc;
use crate::error::Result;
use crate::models::pattern::{
    Pattern, PatternType, PatternQuery, PatternStats, PatternUsage,
};
use crate::models::memory::{Memory, MemoryQuery};
use crate::models::pattern_repository::PatternRepository;
use crate::models::memory_repository::MemoryRepository;

/// Pattern updates input
#[derive(Debug, Clone, Default)]
pub struct PatternUpdates {
    /// Name update
    pub name: Option<String>,

    /// Description update
    pub description: Option<String>,

    /// Trigger update (keywords or regex patterns)
    pub trigger: Option<String>,

    /// Context update (applicable scenarios)
    pub context: Option<String>,

    /// Problem description update
    pub problem: Option<String>,

    /// Solution update
    pub solution: Option<String>,

    /// Explanation update
    pub explanation: Option<String>,

    /// Tags update (replaces existing)
    pub tags: Option<Vec<String>>,

    /// Public visibility update
    pub is_public: Option<bool>,

    /// Confidence update
    pub confidence: Option<f32>,
}

/// Pattern recommendation result
#[derive(Debug, Clone)]
pub struct PatternRecommendation {
    /// The recommended pattern
    pub pattern: Pattern,

    /// Recommendation score (0.0 to 1.0)
    pub score: f32,

    /// Reasons for this recommendation
    pub reasons: Vec<String>,

    /// Matched trigger keywords
    pub matched_keywords: Vec<String>,
}

/// Pattern discovery result
#[derive(Debug, Clone)]
pub struct PatternDiscoveryResult {
    /// Discovered patterns
    pub patterns: Vec<Pattern>,

    /// Discovery method used
    pub method: DiscoveryMethod,

    /// Number of memories analyzed
    pub memories_analyzed: usize,

    /// Confidence score for discovery
    pub confidence: f32,

    /// Suggested new patterns to create
    pub suggested_patterns: Vec<PatternSuggestion>,
}

/// Discovery method enum
#[derive(Debug, Clone, PartialEq)]
pub enum DiscoveryMethod {
    /// Discovered from memory analysis
    MemoryAnalysis,

    /// Discovered from usage patterns
    UsagePattern,

    /// Discovered from common errors
    ErrorPattern,

    /// Discovered from similar patterns
    Similarity,

    /// Discovered from user feedback
    Feedback,
}

/// Suggested pattern to create
#[derive(Debug, Clone)]
pub struct PatternSuggestion {
    /// Suggested name
    pub name: String,

    /// Suggested problem description
    pub problem: String,

    /// Suggested solution
    pub solution: String,

    /// Suggested pattern type
    pub pattern_type: PatternType,

    /// Confidence score
    pub confidence: f32,

    /// Supporting evidence
    pub evidence: Vec<String>,
}

/// Pattern generator trait for AI-based pattern extraction
#[async_trait]
pub trait PatternGenerator: Send + Sync {
    /// Generate a pattern create request from a memory
    async fn generate_from_memory(&self, memory: &Memory) -> Result<PatternCreateRequest>;
}

/// Pattern creation request - output from AI analysis
#[derive(Debug, Clone)]
pub struct PatternCreateRequest {
    /// Pattern name
    pub name: String,

    /// Pattern description
    pub description: String,

    /// Trigger keywords (comma-separated)
    pub trigger: String,

    /// Context description
    pub context: String,

    /// Problem description
    pub problem: String,

    /// Solution description
    pub solution: String,

    /// Pattern type
    pub pattern_type: PatternType,

    /// Tags derived from memory
    pub tags: Vec<String>,

    /// Confidence score from analysis
    pub confidence: f32,

    /// Source memory ID
    pub source_memory_id: String,
}

impl PatternCreateRequest {
    /// Create a new pattern from this request
    pub fn to_pattern(&self, created_by: &str) -> Pattern {
        Pattern::new(
            created_by,
            self.pattern_type.clone(),
            &self.name,
            &self.problem,
            &self.solution,
        )
    }
}

/// Outcome recording input
#[derive(Debug, Clone)]
pub struct OutcomeRecord {
    /// User ID who used the pattern
    pub user_id: String,

    /// Input that triggered the pattern
    pub input: String,

    /// Output produced
    pub output: String,

    /// Outcome score (-1.0 to 1.0)
    pub outcome: f32,

    /// User feedback
    pub feedback: Option<String>,

    /// Context information
    pub context: Option<String>,
}

/// Pattern Manager Service
///
/// Orchestrates pattern operations with business logic:
/// - Creates and manages patterns with full lifecycle tracking
/// - Matches patterns against user input
/// - Records outcomes and updates pattern statistics
/// - Discovers new patterns from memory analysis
/// - Provides intelligent recommendations based on context
/// - Auto-generates patterns from high-importance memories using AI
#[derive(Clone)]
pub struct PatternManager {
    pattern_repo: Arc<dyn PatternRepository>,
    memory_repo: Arc<dyn MemoryRepository>,
    /// Optional AI generator for pattern extraction
    ai_generator: Option<Arc<dyn PatternGenerator>>,
}

impl PatternManager {
    /// Create a new PatternManager with optional AI generator
    pub fn new(
        pattern_repo: Arc<dyn PatternRepository>,
        memory_repo: Arc<dyn MemoryRepository>,
        ai_generator: Option<Arc<dyn PatternGenerator>>,
    ) -> Self {
        Self {
            pattern_repo,
            memory_repo,
            ai_generator,
        }
    }

    /// Create a new PatternManager without AI generator
    pub fn new_basic(
        pattern_repo: Arc<dyn PatternRepository>,
        memory_repo: Arc<dyn MemoryRepository>,
    ) -> Self {
        Self::new(pattern_repo, memory_repo, None)
    }

    /// Create a new pattern
    ///
    /// Creates a pattern with the given parameters and stores it in the repository.
    pub async fn create_pattern(
        &self,
        created_by: &str,
        pattern_type: &str,
        name: &str,
        problem: &str,
        solution: &str,
    ) -> Result<Pattern> {
        tracing::info!("Creating pattern '{}' for user: {}", name, created_by);

        // Parse pattern type
        let ptype = match pattern_type.to_lowercase().as_str() {
            "problem_solution" | "problem-solution" => PatternType::ProblemSolution,
            "workflow" => PatternType::Workflow,
            "best_practice" | "best-practice" => PatternType::BestPractice,
            "common_error" | "common-error" => PatternType::CommonError,
            "skill" => PatternType::Skill,
            _ => PatternType::ProblemSolution,
        };

        let pattern = Pattern::new(created_by, ptype, name, problem, solution);
        self.pattern_repo.create(&pattern).await
    }

    /// Get a pattern by ID
    ///
    /// Returns the full pattern details if found.
    pub async fn get_pattern(&self, pattern_id: &str) -> Result<Option<Pattern>> {
        tracing::debug!("Getting pattern: {}", pattern_id);
        self.pattern_repo.get_by_id(pattern_id).await
    }

    /// Update a pattern
    ///
    /// Applies the specified updates to an existing pattern.
    pub async fn update_pattern(
        &self,
        pattern_id: &str,
        updates: &PatternUpdates,
    ) -> Result<Option<Pattern>> {
        tracing::info!("Updating pattern: {}", pattern_id);

        // Get existing pattern
        let mut pattern = self
            .pattern_repo
            .get_by_id(pattern_id)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::NotFound(format!("Pattern not found: {}", pattern_id))
            })?;

        // Apply updates
        if let Some(name) = &updates.name {
            pattern.name = name.clone();
        }
        if let Some(description) = &updates.description {
            pattern.description = description.clone();
        }
        if let Some(trigger) = &updates.trigger {
            pattern.trigger = trigger.clone();
        }
        if let Some(context) = &updates.context {
            pattern.context = context.clone();
        }
        if let Some(problem) = &updates.problem {
            pattern.problem = problem.clone();
        }
        if let Some(solution) = &updates.solution {
            pattern.solution = solution.clone();
        }
        if let Some(explanation) = &updates.explanation {
            pattern.explanation = Some(explanation.clone());
        }
        if let Some(tags) = &updates.tags {
            pattern.tags = tags.clone();
        }
        if let Some(is_public) = &updates.is_public {
            pattern.is_public = *is_public;
        }
        if let Some(confidence) = &updates.confidence {
            pattern.confidence = *confidence;
        }

        pattern.updated_at = Utc::now();
        pattern.version += 1;

        // Save updated pattern
        self.pattern_repo.update(pattern_id, &pattern).await
    }

    /// Record an outcome for a pattern
    ///
    /// Records how a pattern performed when used, updating statistics.
    pub async fn record_outcome(
        &self,
        pattern_id: &str,
        record: &OutcomeRecord,
    ) -> Result<String> {
        tracing::info!("Recording outcome for pattern: {}", pattern_id);

        // Verify pattern exists
        let _pattern = self
            .pattern_repo
            .get_by_id(pattern_id)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::NotFound(format!("Pattern not found: {}", pattern_id))
            })?;

        // Create usage record
        let usage = PatternUsage {
            id: uuid::Uuid::new_v4().to_string(),
            pattern_id: pattern_id.to_string(),
            user_id: record.user_id.clone(),
            input: record.input.clone(),
            output: record.output.clone(),
            outcome: record.outcome,
            feedback: record.feedback.clone(),
            used_at: Utc::now(),
            context: record.context.clone(),
        };

        // Record usage in repository
        let usage_id = self.pattern_repo.record_usage(pattern_id, &usage).await?;

        Ok(usage_id)
    }

    /// Search patterns with various filters
    ///
    /// Returns patterns matching the specified query criteria.
    pub async fn search_patterns(&self, query: &PatternQuery) -> Result<Vec<Pattern>> {
        tracing::debug!("Searching patterns with query: {:?}", query);

        // Set default page size if not specified
        let mut query = query.clone();
        if query.page_size == 0 {
            query.page_size = 20;
        }
        if query.page == 0 {
            query.page = 1;
        }

        self.pattern_repo.search(&query).await
    }

    /// Get pattern recommendations based on context
    ///
    /// Analyzes the current context and recommends relevant patterns.
    pub async fn get_recommendations(
        &self,
        user_id: &str,
        context: &str,
        limit: u32,
    ) -> Result<Vec<PatternRecommendation>> {
        tracing::info!("Getting pattern recommendations for user: {}", user_id);

        // Get user's recent memories for context
        let memory_query = MemoryQuery {
            user_id: Some(user_id.to_string()),
            page: 1,
            page_size: 10,
            ..Default::default()
        };
        let memories = self.memory_repo.search(&memory_query).await?;

        // Get all public patterns or user's patterns
        let pattern_query = PatternQuery {
            public_only: false,
            page: 1,
            page_size: 100,
            ..Default::default()
        };
        let patterns = self.pattern_repo.search(&pattern_query).await?;

        // Score and rank patterns
        let context_lower = context.to_lowercase();
        let mut recommendations: Vec<PatternRecommendation> = patterns
            .into_iter()
            .map(|pattern| {
                let (score, reasons, matched_keywords) = self.score_pattern(
                    &pattern,
                    &context_lower,
                    &memories,
                );

                PatternRecommendation {
                    pattern,
                    score,
                    reasons,
                    matched_keywords,
                }
            })
            .filter(|rec| rec.score > 0.0)
            .collect();

        // Sort by score descending
        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Limit results
        let limit = limit.max(1) as usize;
        recommendations.truncate(limit);

        Ok(recommendations)
    }

    /// Score a pattern against context
    fn score_pattern(
        &self,
        pattern: &Pattern,
        context: &str,
        memories: &[Memory],
    ) -> (f32, Vec<String>, Vec<String>) {
        let mut score = 0.0;
        let mut reasons = Vec::new();
        let mut matched_keywords = Vec::new();

        // Score based on trigger matching
        if !pattern.trigger.is_empty() {
            let trigger_keywords: Vec<&str> = pattern
                .trigger
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();

            let mut trigger_matches = 0;
            for keyword in &trigger_keywords {
                if context.contains(keyword) {
                    trigger_matches += 1;
                    matched_keywords.push(keyword.to_string());
                }
            }

            if !trigger_keywords.is_empty() {
                let trigger_score = trigger_matches as f32 / trigger_keywords.len() as f32;
                score += trigger_score * 0.4;

                if trigger_matches > 0 {
                    reasons.push(format!(
                        "Matches {} of {} trigger keywords",
                        trigger_matches,
                        trigger_keywords.len()
                    ));
                }
            }
        }

        // Score based on pattern quality
        if pattern.is_high_quality() {
            score += 0.2;
            reasons.push("High quality pattern (high confidence + success rate)".to_string());
        }

        // Score based on usage count
        if pattern.usage_count > 10 {
            score += 0.1;
            reasons.push(format!("Well-used pattern ({} uses)", pattern.usage_count));
        }

        // Score based on recent usage
        if let Some(last_used) = pattern.last_used {
            let days_since_use = (Utc::now() - last_used).num_days();
            if days_since_use < 7 {
                score += 0.1;
                reasons.push("Recently used".to_string());
            } else if days_since_use < 30 {
                score += 0.05;
            }
        }

        // Score based on memory relevance
        for memory in memories {
            let gist_lower = memory.gist.to_lowercase();
            let content_lower = memory.content.to_lowercase();
            if context.contains(&gist_lower) || context.contains(&content_lower) {
                score += 0.1;
                if reasons.len() < 3 {
                    reasons.push("Relevant to your recent memories".to_string());
                }
                break;
            }
        }

        // Normalize score
        score = score.min(1.0);

        (score, reasons, matched_keywords)
    }

    /// Match patterns against input
    ///
    /// Finds patterns that match the given input text.
    pub async fn match_patterns(&self, input: &str, limit: u32) -> Result<Vec<Pattern>> {
        tracing::info!("Matching patterns against input (limit: {})", limit);

        self.pattern_repo.match_patterns(input, limit).await
    }

    /// Discover new patterns from memories
    ///
    /// Analyzes user memories to discover potential new patterns.
    pub async fn discover_patterns(
        &self,
        user_id: &str,
        method: DiscoveryMethod,
        limit: u32,
    ) -> Result<PatternDiscoveryResult> {
        tracing::info!(
            "Discovering patterns for user: {} using method: {:?}",
            user_id,
            method
        );

        // Get user memories for analysis
        let memory_query = MemoryQuery {
            user_id: Some(user_id.to_string()),
            page: 1,
            page_size: 50,
            ..Default::default()
        };
        let memories = self.memory_repo.search(&memory_query).await?;

        // Analyze memories based on discovery method
        let (patterns, confidence, suggestions) = match method {
            DiscoveryMethod::MemoryAnalysis => {
                self.analyze_memories_for_patterns(&memories, limit as usize)
            }
            DiscoveryMethod::UsagePattern => {
                self.analyze_usage_patterns(&memories, limit as usize)
            }
            DiscoveryMethod::ErrorPattern => {
                self.analyze_error_patterns(&memories, limit as usize)
            }
            DiscoveryMethod::Similarity => {
                self.analyze_similarity_patterns(&memories, limit as usize)
            }
            DiscoveryMethod::Feedback => {
                self.analyze_feedback_patterns(&memories, limit as usize)
            }
        };

        Ok(PatternDiscoveryResult {
            patterns,
            method,
            memories_analyzed: memories.len(),
            confidence,
            suggested_patterns: suggestions,
        })
    }

    /// Analyze memories to discover potential patterns
    fn analyze_memories_for_patterns(
        &self,
        memories: &[Memory],
        _limit: usize,
    ) -> (Vec<Pattern>, f32, Vec<PatternSuggestion>) {
        let patterns = Vec::new();
        let mut suggestions = Vec::new();
        let mut common_topics: Vec<(String, usize)> = Vec::new();

        // Extract common topics from memories
        for memory in memories {
            let gist_lower = memory.gist.to_lowercase();
            let words: Vec<String> = gist_lower
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            // Simple frequency analysis
            for word in words.iter() {
                if word.len() > 3 {
                    if let Some(existing) = common_topics.iter_mut().find(|(w, _)| w == word) {
                        existing.1 += 1;
                    } else {
                        common_topics.push((word.clone(), 1));
                    }
                }
            }
        }

        // Sort by frequency
        common_topics.sort_by(|a, b| b.1.cmp(&a.1));

        // Generate suggestions for common topics
        for (topic, count) in common_topics.iter().take(5) {
            if *count >= 2 {
                suggestions.push(PatternSuggestion {
                    name: format!("{} pattern", topic),
                    problem: format!("Issues related to {}", topic),
                    solution: String::new(),
                    pattern_type: PatternType::ProblemSolution,
                    confidence: (*count as f32 / memories.len() as f32).min(1.0),
                    evidence: vec![format!("Mentioned {} times in memories", count)],
                });
            }
        }

        let confidence = if !memories.is_empty() {
            (common_topics.len() as f32 / memories.len() as f32).min(1.0)
        } else {
            0.0
        };

        (patterns, confidence, suggestions)
    }

    /// Analyze usage patterns from memories
    fn analyze_usage_patterns(
        &self,
        _memories: &[Memory],
        _limit: usize,
    ) -> (Vec<Pattern>, f32, Vec<PatternSuggestion>) {
        // Placeholder: In a real implementation, this would analyze
        // usage patterns from memory content
        (Vec::new(), 0.0, Vec::new())
    }

    /// Analyze error patterns from memories
    fn analyze_error_patterns(
        &self,
        memories: &[Memory],
        _limit: usize,
    ) -> (Vec<Pattern>, f32, Vec<PatternSuggestion>) {
        let mut suggestions = Vec::new();

        // Look for error-related memories
        for memory in memories {
            let content_lower = memory.content.to_lowercase();
            if content_lower.contains("error")
                || content_lower.contains("failed")
                || content_lower.contains("bug")
            {
                suggestions.push(PatternSuggestion {
                    name: format!("Error handling pattern for {}", memory.gist),
                    problem: memory.gist.clone(),
                    solution: String::new(),
                    pattern_type: PatternType::CommonError,
                    confidence: memory.importance,
                    evidence: vec![format!(
                        "Found error-related memory: {}",
                        memory.content.chars().take(100).collect::<String>()
                    )],
                });
            }
        }

        let confidence = if !memories.is_empty() {
            suggestions.len() as f32 / memories.len() as f32
        } else {
            0.0
        };

        (Vec::new(), confidence, suggestions)
    }

    /// Analyze similarity patterns from memories
    fn analyze_similarity_patterns(
        &self,
        _memories: &[Memory],
        _limit: usize,
    ) -> (Vec<Pattern>, f32, Vec<PatternSuggestion>) {
        // Placeholder: In a real implementation, this would find
        // similar patterns in memories
        (Vec::new(), 0.0, Vec::new())
    }

    /// Analyze feedback patterns from memories
    fn analyze_feedback_patterns(
        &self,
        _memories: &[Memory],
        _limit: usize,
    ) -> (Vec<Pattern>, f32, Vec<PatternSuggestion>) {
        // Placeholder: In a real implementation, this would analyze
        // feedback patterns from memory content
        (Vec::new(), 0.0, Vec::new())
    }

    /// Get pattern statistics
    ///
    /// Returns comprehensive statistics about patterns.
    pub async fn get_pattern_stats(&self) -> Result<PatternStats> {
        tracing::debug!("Getting pattern statistics");

        self.pattern_repo.get_stats().await
    }

    /// Delete a pattern
    ///
    /// Removes a pattern from the system.
    pub async fn delete_pattern(&self, pattern_id: &str) -> Result<bool> {
        tracing::info!("Deleting pattern: {}", pattern_id);

        self.pattern_repo.delete(pattern_id).await
    }

    /// Add an example to a pattern
    ///
    /// Adds a new example to an existing pattern.
    pub async fn add_example(
        &self,
        pattern_id: &str,
        input: &str,
        output: &str,
        outcome: f32,
        source_memory_id: Option<&str>,
    ) -> Result<Option<Pattern>> {
        tracing::info!("Adding example to pattern: {}", pattern_id);

        // Get existing pattern
        let mut pattern = self
            .pattern_repo
            .get_by_id(pattern_id)
            .await?
            .ok_or_else(|| {
                crate::error::AppError::NotFound(format!("Pattern not found: {}", pattern_id))
            })?;

        // Add example
        pattern.add_example(input, output, outcome, source_memory_id);

        // Save updated pattern
        self.pattern_repo.update(pattern_id, &pattern).await
    }

    /// List patterns with pagination
    ///
    /// Returns a paginated list of patterns.
    pub async fn list_patterns(
        &self,
        limit: usize,
        start: usize,
    ) -> Result<Vec<Pattern>> {
        tracing::debug!("Listing patterns (limit: {}, start: {})", limit, start);

        self.pattern_repo.list(limit, start).await
    }

    /// Auto-generate patterns from high-importance memories
    ///
    /// Searches for memories with importance above the threshold and generates
    /// patterns from them using AI analysis.
    pub async fn auto_generate_from_memories(
        &self,
        min_importance: f32,
    ) -> Result<Vec<String>> {
        tracing::info!(
            "Auto-generating patterns from memories with importance >= {}",
            min_importance
        );

        // Check if AI generator is available
        let Some(ref generator) = self.ai_generator else {
            tracing::warn!("No AI generator configured, skipping auto-generation");
            return Ok(Vec::new());
        };

        // Search for high-importance memories
        let memory_query = MemoryQuery {
            min_importance: Some(min_importance),
            statuses: vec![crate::models::memory::MemoryStatus::Active],
            page: 1,
            page_size: 50,
            ..Default::default()
        };

        let memories = self.memory_repo.search(&memory_query).await?;
        tracing::debug!("Found {} high-importance memories", memories.len());

        let mut created_pattern_ids = Vec::new();
        let mut seen_patterns: std::collections::HashSet<String> =
            std::collections::HashSet::new();

        for memory in memories {
            // Check for duplicates
            let pattern_key = format!(
                "{}:{}",
                memory.gist.to_lowercase(),
                memory.content.to_lowercase()
            );

            if seen_patterns.contains(&pattern_key) {
                tracing::debug!("Skipping duplicate memory: {}", memory.id);
                continue;
            }

            // Generate pattern from memory
            match generator.generate_from_memory(&memory).await {
                Ok(request) => {
                    // Create the pattern
                    let pattern = self
                        .create_pattern(
                            &memory.user_id,
                            &request.pattern_type.to_string(),
                            &request.name,
                            &request.problem,
                            &request.solution,
                        )
                        .await?;

                    // Update with additional details
                    let mut updates = PatternUpdates::default();
                    updates.trigger = Some(request.trigger.clone());
                    updates.context = Some(request.context.clone());
                    updates.description = Some(request.description.clone());
                    updates.tags = Some(request.tags.clone());
                    updates.confidence = Some(request.confidence);

                    self.update_pattern(&pattern.id, &updates).await?;

                    // Add example from memory
                    self.add_example(
                        &pattern.id,
                        &memory.content,
                        &request.solution,
                        memory.confidence,
                        Some(&memory.id),
                    )
                    .await?;

                    created_pattern_ids.push(pattern.id.clone());
                    seen_patterns.insert(pattern_key);

                    tracing::info!(
                        "Created pattern '{}' from memory {}",
                        pattern.name,
                        memory.id
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to generate pattern from memory {}: {}",
                        memory.id,
                        e
                    );
                }
            }
        }

        Ok(created_pattern_ids)
    }

    /// Generate a pattern create request from a memory
    ///
    /// Uses AI to analyze the memory and extract pattern components.
    pub async fn generate_pattern_from_memory(
        &self,
        memory: &Memory,
    ) -> Result<PatternCreateRequest> {
        tracing::debug!(
            "Generating pattern request from memory: {}",
            memory.id
        );

        // Check if AI generator is available
        let request = match &self.ai_generator {
            Some(ai_gen) => ai_gen.generate_from_memory(memory).await?,
            None => self.generate_pattern_from_memory_fallback(memory)?,
        };

        Ok(request)
    }

    /// Fallback pattern generation without AI
    fn generate_pattern_from_memory_fallback(
        &self,
        memory: &Memory,
    ) -> Result<PatternCreateRequest> {
        let content_lower = memory.content.to_lowercase();
        let gist_lower = memory.gist.to_lowercase();

        // Extract trigger keywords from content
        let trigger = self.extract_trigger_keywords(&content_lower);

        // Determine pattern type based on content
        let pattern_type = self.detect_pattern_type(&content_lower);

        // Extract problem and solution
        let (problem, solution) = self.extract_problem_solution(&content_lower);

        // Generate name from gist
        let name = if !memory.gist.is_empty() {
            memory.gist.clone()
        } else {
            format!(
                "Pattern from {} memory",
                memory.memory_type.to_string()
            )
        };

        // Generate description
        let description = format!(
            "Auto-generated pattern from memory about {}. Importance: {:.2}",
            name,
            memory.importance
        );

        // Generate context
        let context = format!(
            "This pattern applies when dealing with: {}. Source: {}",
            gist_lower,
            memory.source.to_string()
        );

        // Extract tags from memory
        let mut tags = memory.tags.clone();
        if tags.is_empty() {
            tags = self.extract_tags(&content_lower);
        }

        Ok(PatternCreateRequest {
            name,
            description,
            trigger,
            context,
            problem,
            solution,
            pattern_type,
            tags,
            confidence: memory.importance * 0.8, // Reduce confidence for rule-based
            source_memory_id: memory.id.clone(),
        })
    }

    /// Extract trigger keywords from content
    fn extract_trigger_keywords(&self, content: &str) -> String {
        let keywords = vec![
            "error", "bug", "issue", "problem", "fail", "crash", "slow", "performance",
            "memory", "cpu", "network", "database", "api", "async", "thread", "lock",
            "rust", "python", "javascript", "go", "java", "c++", "tokio",
        ];

        let mut found: Vec<String> = Vec::new();

        for keyword in keywords {
            if content.contains(keyword) && !found.contains(&keyword.to_string()) {
                found.push(keyword.to_string());
            }
        }

        // Add gist words if meaningful
        let gist_words: Vec<String> = self
            .get_gist_words(content)
            .into_iter()
            .take(3)
            .collect();

        for word in gist_words {
            if word.len() > 3 && !found.contains(&word) {
                found.push(word);
            }
        }

        found.join(",")
    }

    /// Get meaningful words from gist
    fn get_gist_words(&self, content: &str) -> Vec<String> {
        let stop_words: std::collections::HashSet<&str> = [
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being",
            "have", "has", "had", "do", "does", "did", "will", "would", "could",
            "should", "may", "might", "must", "shall", "can", "need", "dare",
            "to", "of", "in", "for", "on", "with", "at", "by", "from", "as",
            "into", "through", "during", "before", "after", "above", "below",
            "over", "under",
            "and", "but", "or", "nor", "so", "yet", "both", "either", "neither",
            "not", "only", "just", "also", "very", "too", "quite", "rather",
        ]
        .iter()
        .copied()
        .collect();

        let words: Vec<String> = content
            .split_whitespace()
            .filter(|w| w.len() > 3 && !stop_words.contains(&w.to_lowercase().as_str()))
            .map(|w| w.to_lowercase())
            .collect();

        words
    }

    /// Detect pattern type from content
    fn detect_pattern_type(&self, content: &str) -> PatternType {
        if content.contains("error")
            || content.contains("fail")
            || content.contains("bug")
            || content.contains("exception")
        {
            PatternType::CommonError
        } else if content.contains("step") || content.contains("workflow")
            || content.contains("process") || content.contains("flow")
        {
            PatternType::Workflow
        } else if content.contains("best") || content.contains("practice")
            || content.contains("recommend") || content.contains("should")
        {
            PatternType::BestPractice
        } else if content.contains("how to") || content.contains("tutorial")
            || content.contains("guide")
        {
            PatternType::Skill
        } else {
            PatternType::ProblemSolution
        }
    }

    /// Extract problem and solution from content
    fn extract_problem_solution(&self, content: &str) -> (String, String) {
        // Simple heuristic: look for problem/solution indicators
        let problem_indicators = ["problem", "issue", "error", "fail", "bug", "when"];
        let solution_indicators = ["solution", "fix", "resolve", "use", "try", "instead"];

        let sentences: Vec<&str> = content
            .split(|c| c == '.' || c == '!' || c == '?')
            .filter(|s| !s.trim().is_empty())
            .collect();

        let mut problem_sentences = Vec::new();
        let mut solution_sentences = Vec::new();

        for sentence in sentences {
            let lower = sentence.to_lowercase();
            if problem_indicators.iter().any(|i| lower.contains(i)) {
                problem_sentences.push(sentence.trim());
            } else if solution_indicators.iter().any(|i| lower.contains(i)) {
                solution_sentences.push(sentence.trim());
            }
        }

        let problem = if !problem_sentences.is_empty() {
            problem_sentences.join(". ")
        } else {
            content.to_string()
        };

        let solution = if !solution_sentences.is_empty() {
            solution_sentences.join(". ")
        } else {
            String::new()
        };

        (problem, solution)
    }

    /// Extract tags from content
    fn extract_tags(&self, content: &str) -> Vec<String> {
        let tech_terms = vec![
            "rust", "python", "javascript", "typescript", "go", "java", "c++", "c#",
            "react", "vue", "angular", "node", "deno", "bun",
            "async", "await", "tokio", "actix",
            "database", "sql", "nosql", "mongodb", "postgres", "mysql", "redis",
            "api", "rest", "graphql", "grpc",
            "docker", "kubernetes", "aws", "azure", "gcp",
            "git", "github", "gitlab", "ci", "cd",
        ];

        let content_lower = content.to_lowercase();
        let mut tags = Vec::new();

        for term in tech_terms {
            if content_lower.contains(term) && !tags.contains(&term.to_string()) {
                tags.push(term.to_string());
            }
        }

        tags
    }

    /// Auto-discover patterns periodically
    ///
    /// Searches for high-importance memories and creates patterns from them.
    /// Returns the count of newly created patterns.
    pub async fn auto_discover_patterns(&self) -> Result<u32> {
        tracing::info!("Running auto-discovery of patterns");

        // Check for existing patterns to avoid duplicates
        let existing_patterns = self
            .pattern_repo
            .search(&PatternQuery {
                page: 1,
                page_size: 1000,
                ..Default::default()
            })
            .await?;

        let existing_triggers: std::collections::HashSet<String> = existing_patterns
            .iter()
            .map(|p| p.trigger.to_lowercase())
            .collect();

        // Generate patterns from high-importance memories
        let created_ids = self.auto_generate_from_memories(0.7).await?;

        // Count truly new patterns (not duplicates)
        let new_count = created_ids.len() as u32;

        tracing::info!(
            "Auto-discovery complete. Created {} new patterns",
            new_count
        );

        Ok(new_count)
    }

    /// Get pattern count
    ///
    /// Returns the total number of patterns.
    pub async fn count_patterns(&self) -> Result<u64> {
        self.pattern_repo.count().await
    }
}

/// Create a PatternManager service with optional AI generator
pub fn create_pattern_manager(
    pattern_repo: Arc<dyn PatternRepository>,
    memory_repo: Arc<dyn MemoryRepository>,
    ai_generator: Option<Arc<dyn PatternGenerator>>,
) -> PatternManager {
    PatternManager::new(pattern_repo, memory_repo, ai_generator)
}

/// Create a basic PatternManager without AI generator
pub fn create_pattern_manager_basic(
    pattern_repo: Arc<dyn PatternRepository>,
    memory_repo: Arc<dyn MemoryRepository>,
) -> PatternManager {
    PatternManager::new_basic(pattern_repo, memory_repo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::pattern_repository::PatternRepository;
    use crate::models::memory_repository::MemoryRepository;
    use async_trait::async_trait;

    #[derive(Clone)]
    struct MockPatternRepository;

    #[async_trait]
    impl PatternRepository for MockPatternRepository {
        async fn create(&self, pattern: &Pattern) -> Result<Pattern> {
            Ok(pattern.clone())
        }

        async fn get_by_id(&self, id: &str) -> Result<Option<Pattern>> {
            if id == "existing_pattern" {
                let mut pattern = Pattern::new(
                    "user_123",
                    PatternType::ProblemSolution,
                    "Test Pattern",
                    "Test Problem",
                    "Test Solution",
                );
                pattern.id = id.to_string();
                return Ok(Some(pattern));
            }
            Ok(None)
        }

        async fn update(&self, _id: &str, pattern: &Pattern) -> Result<Option<Pattern>> {
            Ok(Some(pattern.clone()))
        }

        async fn delete(&self, _id: &str) -> Result<bool> {
            Ok(true)
        }

        async fn list(&self, _limit: usize, _start: usize) -> Result<Vec<Pattern>> {
            Ok(vec![])
        }

        async fn count(&self) -> Result<u64> {
            Ok(1)
        }

        async fn search(&self, _query: &PatternQuery) -> Result<Vec<Pattern>> {
            let mut pattern = Pattern::new(
                "user_123",
                PatternType::ProblemSolution,
                "Test Pattern",
                "Test Problem",
                "Test Solution",
            );
            pattern.id = "test_pattern".to_string();
            pattern.confidence = 0.8;
            pattern.success_count = 5;
            pattern.failure_count = 1;
            Ok(vec![pattern])
        }

        async fn record_usage(&self, _pattern_id: &str, _usage: &PatternUsage) -> Result<String> {
            Ok("usage_123".to_string())
        }

        async fn get_stats(&self) -> Result<PatternStats> {
            Ok(PatternStats {
                total_count: 10,
                problem_solution_count: 4,
                workflow_count: 2,
                best_practice_count: 2,
                common_error_count: 1,
                skill_count: 1,
                avg_success_rate: 0.85,
                high_quality_count: 5,
                total_usages: 100,
                most_used_pattern_id: "pattern_1".to_string(),
                most_used_pattern_name: "Most Used Pattern".to_string(),
            })
        }

        async fn match_patterns(&self, _input: &str, _limit: u32) -> Result<Vec<Pattern>> {
            let mut pattern = Pattern::new(
                "user_123",
                PatternType::Skill,
                "Matched Pattern",
                "Matched Problem",
                "Matched Solution",
            );
            pattern.trigger = "rust,async".to_string();
            Ok(vec![pattern])
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

        async fn search(&self, _query: &MemoryQuery) -> Result<Vec<Memory>> {
            let memory = Memory {
                id: "memory_123".to_string(),
                tenant_id: "default".to_string(),
                user_id: "user_123".to_string(),
                memory_type: crate::models::memory::MemoryType::Episodic,
                content: "Test memory content about Rust programming".to_string(),
                gist: "Rust programming".to_string(),
                full_summary: None,
                embedding: Some(Vec::new()),
                importance: 0.8,
                confidence: 0.9,
                source: crate::models::memory::MemorySource::Conversation,
                source_id: None,
                parent_id: None,
                related_ids: vec![],
                tags: vec![],
                topics: vec![],
                created_at: Utc::now(),
                updated_at: Utc::now(),
                accessed_at: Utc::now(),
                expires_at: None,
                status: crate::models::memory::MemoryStatus::Active,
                version: 1,
                keywords: vec![],
            };
            Ok(vec![memory])
        }

        async fn get_stats(&self, _user_id: &str) -> Result<crate::models::memory::MemoryStats> {
            Ok(crate::models::memory::MemoryStats {
                user_id: "user_123".to_string(),
                total_count: 10,
                episodic_count: 5,
                semantic_count: 3,
                procedural_count: 2,
                profile_count: 0,
                active_count: 8,
                archived_count: 2,
                avg_importance: 0.7,
                high_importance_count: 4,
                storage_size_bytes: 1024,
            })
        }
    }

    #[tokio::test]
    async fn test_create_pattern() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let pattern = manager
            .create_pattern(
                "user_123",
                "problem_solution",
                "Test Pattern",
                "Test Problem",
                "Test Solution",
            )
            .await
            .unwrap();

        assert_eq!(pattern.name, "Test Pattern");
        assert_eq!(pattern.problem, "Test Problem");
        assert_eq!(pattern.solution, "Test Solution");
        assert_eq!(pattern.created_by, "user_123");
    }

    #[tokio::test]
    async fn test_get_pattern_existing() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let pattern = manager.get_pattern("existing_pattern").await.unwrap();

        assert!(pattern.is_some());
        assert_eq!(pattern.unwrap().name, "Test Pattern");
    }

    #[tokio::test]
    async fn test_get_pattern_not_found() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let pattern = manager.get_pattern("nonexistent").await.unwrap();

        assert!(pattern.is_none());
    }

    #[tokio::test]
    async fn test_update_pattern() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let updates = PatternUpdates {
            name: Some("Updated Name".to_string()),
            description: Some("Updated description".to_string()),
            ..Default::default()
        };

        let result = manager.update_pattern("existing_pattern", &updates).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_outcome() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let record = OutcomeRecord {
            user_id: "user_123".to_string(),
            input: "Test input".to_string(),
            output: "Test output".to_string(),
            outcome: 0.9,
            feedback: Some("Excellent!".to_string()),
            context: Some("Test context".to_string()),
        };

        let result = manager.record_outcome("existing_pattern", &record).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "usage_123");
    }

    #[tokio::test]
    async fn test_search_patterns() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let query = PatternQuery::default();
        let patterns = manager.search_patterns(&query).await.unwrap();

        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].name, "Test Pattern");
    }

    #[tokio::test]
    async fn test_get_recommendations() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let recommendations = manager
            .get_recommendations("user_123", "rust async programming", 5)
            .await
            .unwrap();

        assert!(!recommendations.is_empty());
    }

    #[tokio::test]
    async fn test_match_patterns() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let patterns = manager.match_patterns("learning rust async", 10).await.unwrap();

        assert!(!patterns.is_empty());
    }

    #[tokio::test]
    async fn test_discover_patterns() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let result = manager
            .discover_patterns("user_123", DiscoveryMethod::MemoryAnalysis, 5)
            .await
            .unwrap();

        assert_eq!(result.method, DiscoveryMethod::MemoryAnalysis);
    }

    #[tokio::test]
    async fn test_get_pattern_stats() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let stats = manager.get_pattern_stats().await.unwrap();

        assert_eq!(stats.total_count, 10);
        assert_eq!(stats.avg_success_rate, 0.85);
    }

    #[tokio::test]
    async fn test_delete_pattern() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let result = manager.delete_pattern("existing_pattern").await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn test_add_example() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let result = manager
            .add_example("existing_pattern", "Input", "Output", 0.8, None)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_patterns() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let patterns = manager.list_patterns(10, 0).await.unwrap();

        assert!(patterns.is_empty());
    }

    #[tokio::test]
    async fn test_count_patterns() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let count = manager.count_patterns().await.unwrap();

        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_pattern_type_parsing() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        // Test various pattern type strings
        let types = vec![
            ("problem_solution", PatternType::ProblemSolution),
            ("problem-solution", PatternType::ProblemSolution),
            ("workflow", PatternType::Workflow),
            ("best_practice", PatternType::BestPractice),
            ("best-practice", PatternType::BestPractice),
            ("common_error", PatternType::CommonError),
            ("skill", PatternType::Skill),
            ("unknown", PatternType::ProblemSolution), // Default
        ];

        for (input, expected) in types {
            let pattern = manager
                .create_pattern("user_123", input, "Name", "Problem", "Solution")
                .await
                .unwrap();

            assert_eq!(pattern.pattern_type, expected);
        }
    }

    #[tokio::test]
    async fn test_generate_pattern_from_memory_fallback() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let memory = Memory {
            id: "memory_test".to_string(),
            tenant_id: "default".to_string(),
            user_id: "user_123".to_string(),
            memory_type: crate::models::memory::MemoryType::Episodic,
            content: "I encountered a Rust async error when using tokio::spawn. The problem was not handling JoinError properly. The solution is to use spawn_with_handle and await the result.".to_string(),
            gist: "Rust async error handling".to_string(),
            full_summary: None,
            embedding: None,
            importance: 0.85,
            confidence: 0.9,
            source: crate::models::memory::MemorySource::Execution,
            source_id: None,
            parent_id: None,
            related_ids: vec![],
            tags: vec!["rust".to_string(), "async".to_string()],
            topics: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            accessed_at: Utc::now(),
            expires_at: None,
            status: crate::models::memory::MemoryStatus::Active,
            version: 1,
            keywords: vec![],
        };

        let request = manager
            .generate_pattern_from_memory(&memory)
            .await
            .unwrap();

        assert!(!request.name.is_empty());
        assert!(request.name.contains("Rust") || request.name.contains("async"));
        assert!(request.trigger.contains("rust") || request.trigger.contains("async"));
        assert_eq!(request.source_memory_id, "memory_test");
        assert!(request.confidence > 0.0);
    }

    #[tokio::test]
    async fn test_extract_trigger_keywords() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let content = "Rust async performance issue with tokio spawn causing memory leak";
        let triggers = manager.extract_trigger_keywords(content);

        assert!(triggers.contains("rust"));
        assert!(triggers.contains("async"));
        assert!(triggers.contains("tokio"));
    }

    #[tokio::test]
    async fn test_detect_pattern_type() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        // Test error pattern
        let error_content = "error occurred when connecting to database";
        let error_type = manager.detect_pattern_type(error_content);
        assert_eq!(error_type, PatternType::CommonError);

        // Test workflow pattern
        let workflow_content = "The process involves these steps: first do A, then B";
        let workflow_type = manager.detect_pattern_type(workflow_content);
        assert_eq!(workflow_type, PatternType::Workflow);

        // Test best practice
        let practice_content = "You should always follow best practices for coding";
        let practice_type = manager.detect_pattern_type(practice_content);
        assert_eq!(practice_type, PatternType::BestPractice);

        // Test skill pattern
        let skill_content = "how to implement async rust patterns";
        let skill_type = manager.detect_pattern_type(skill_content);
        assert_eq!(skill_type, PatternType::Skill);
    }

    #[tokio::test]
    async fn test_extract_problem_solution() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let content = "The problem is slow database queries. The solution is to add proper indexing.";
        let (problem, solution) = manager.extract_problem_solution(content);

        assert!(problem.contains("problem") || problem.contains("slow"));
        assert!(solution.contains("solution") || solution.contains("indexing"));
    }

    #[tokio::test]
    async fn test_extract_tags() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let content = "Using React with TypeScript for API development with Docker and Kubernetes";
        let tags = manager.extract_tags(content);

        assert!(tags.contains(&"react".to_string()));
        assert!(tags.contains(&"typescript".to_string()));
        assert!(tags.contains(&"api".to_string()));
        assert!(tags.contains(&"docker".to_string()));
        assert!(tags.contains(&"kubernetes".to_string()));
    }

    #[tokio::test]
    async fn test_pattern_create_request_to_pattern() {
        let request = PatternCreateRequest {
            name: "Test Pattern".to_string(),
            description: "Test description".to_string(),
            trigger: "rust,async".to_string(),
            context: "Test context".to_string(),
            problem: "Test problem".to_string(),
            solution: "Test solution".to_string(),
            pattern_type: PatternType::ProblemSolution,
            tags: vec!["rust".to_string()],
            confidence: 0.8,
            source_memory_id: "memory_123".to_string(),
        };

        let pattern = request.to_pattern("user_456");

        assert_eq!(pattern.name, "Test Pattern");
        assert_eq!(pattern.problem, "Test problem");
        assert_eq!(pattern.solution, "Test solution");
        assert_eq!(pattern.created_by, "user_456");
        assert_eq!(pattern.pattern_type, PatternType::ProblemSolution);
    }

    #[tokio::test]
    async fn test_auto_discover_patterns_no_ai_generator() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        // Should return empty when no AI generator is configured
        let result = manager.auto_discover_patterns().await.unwrap();

        assert_eq!(result, 0);
    }

    #[tokio::test]
    async fn test_get_gist_words() {
        let pattern_repo = Arc::new(MockPatternRepository);
        let memory_repo = Arc::new(MockMemoryRepository);
        let manager = PatternManager::new_basic(pattern_repo, memory_repo);

        let content = "the quick brown fox jumps over the lazy dog and runs fast";
        let words = manager.get_gist_words(content);

        // Should filter out stop words
        assert!(!words.contains(&"the".to_string()));
        assert!(!words.contains(&"and".to_string()));
        assert!(!words.contains(&"over".to_string()));
        // Should include meaningful words
        assert!(words.contains(&"quick".to_string())
            || words.contains(&"brown".to_string())
            || words.contains(&"jumps".to_string()));
    }
}
