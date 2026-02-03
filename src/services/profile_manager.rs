//! Profile Manager Service
//!
//! Provides comprehensive user profile management including:
//! - Profile CRUD operations
//! - Fact management with verification workflow
//! - Preference tracking
//! - Working hours management
//! - Profile merging for multi-device scenarios

use std::sync::Arc;
use chrono::{DateTime, Utc};
use crate::error::Result;
use crate::models::profile::{
    Profile, ProfileFact, ProfileFactCategory, ProfileChange, ProfileChangeType,
    WorkingHours, ProfileComparison,
};
use crate::models::profile_repository::ProfileRepository;

/// Working hours status for profile summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkingHoursStatus {
    /// Whether working hours are configured
    pub is_configured: bool,

    /// Current day of week (0=Monday, 6=Sunday)
    pub current_day: u32,

    /// Current hour (0-23)
    pub current_hour: u32,

    /// Whether current time is within working hours
    pub is_within_working_hours: bool,

    /// Formatted working hours string
    pub working_hours_display: Option<String>,
}

/// Profile updates input
#[derive(Debug, Clone, Default)]
pub struct ProfileUpdates {
    /// Name update
    pub name: Option<String>,

    /// Role update
    pub role: Option<String>,

    /// Organization update
    pub organization: Option<String>,

    /// Location update
    pub location: Option<String>,

    /// Communication style update
    pub communication_style: Option<String>,

    /// Technical level update
    pub technical_level: Option<String>,

    /// Language preference update
    pub language: Option<String>,
}

/// Profile Manager Service
///
/// Orchestrates profile operations with business logic:
/// - Creates or retrieves existing profiles
/// - Manages profile facts with verification
/// - Handles preferences and working hours
/// - Supports profile merging for multi-device scenarios
#[derive(Clone)]
pub struct ProfileManager {
    profile_repo: Arc<dyn ProfileRepository>,
}

impl ProfileManager {
    /// Create a new ProfileManager
    pub fn new(profile_repo: Arc<dyn ProfileRepository>) -> Self {
        Self { profile_repo }
    }

    /// Get or create a profile for a user
    ///
    /// If the user already has a profile, returns it.
    /// Otherwise, creates a new profile with default values.
    pub async fn get_or_create_profile(&self, user_id: &str) -> Result<Profile> {
        tracing::info!("Getting or creating profile for user: {}", user_id);

        // Try to get existing profile
        if let Some(existing_profile) = self.profile_repo.get_by_user_id(user_id).await? {
            tracing::debug!("Found existing profile for user: {}", user_id);
            return Ok(existing_profile);
        }

        // Create new profile
        tracing::debug!("Creating new profile for user: {}", user_id);
        let new_profile = Profile::new(user_id);
        self.profile_repo.create(&new_profile).await
    }

    /// Update basic profile information
    ///
    /// Updates the specified fields and records changes in history.
    pub async fn update_profile(
        &self,
        user_id: &str,
        updates: &ProfileUpdates,
    ) -> Result<Profile> {
        tracing::info!("Updating profile for user: {}", user_id);

        // Get existing profile
        let mut profile = self
            .profile_repo
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Profile not found for user: {}", user_id)))?;

        // Apply updates
        if let Some(name) = &updates.name {
            profile.update_basic_info(Some(name), None, None, Some("profile update"));
        }
        if let Some(role) = &updates.role {
            profile.update_basic_info(None, Some(role), None, Some("profile update"));
        }
        if let Some(organization) = &updates.organization {
            profile.update_basic_info(None, None, Some(organization), Some("profile update"));
        }
        if let Some(location) = &updates.location {
            let change = ProfileChange {
                version: profile.version + 1,
                change_type: ProfileChangeType::Updated,
                field: "location".to_string(),
                old_value: profile.location.clone().map(serde_json::Value::String),
                new_value: Some(serde_json::Value::String(location.clone())),
                reason: Some("profile update".to_string()),
                changed_at: Utc::now(),
            };
            profile.change_history.push(change);
            profile.location = Some(location.clone());
        }
        if let Some(communication_style) = &updates.communication_style {
            let change = ProfileChange {
                version: profile.version + 1,
                change_type: ProfileChangeType::Updated,
                field: "communication_style".to_string(),
                old_value: profile.communication_style.clone().map(serde_json::Value::String),
                new_value: Some(serde_json::Value::String(communication_style.clone())),
                reason: Some("profile update".to_string()),
                changed_at: Utc::now(),
            };
            profile.change_history.push(change);
            profile.communication_style = Some(communication_style.clone());
        }
        if let Some(technical_level) = &updates.technical_level {
            let change = ProfileChange {
                version: profile.version + 1,
                change_type: ProfileChangeType::Updated,
                field: "technical_level".to_string(),
                old_value: profile.technical_level.clone().map(serde_json::Value::String),
                new_value: Some(serde_json::Value::String(technical_level.clone())),
                reason: Some("profile update".to_string()),
                changed_at: Utc::now(),
            };
            profile.change_history.push(change);
            profile.technical_level = Some(technical_level.clone());
        }
        if let Some(language) = &updates.language {
            let change = ProfileChange {
                version: profile.version + 1,
                change_type: ProfileChangeType::Updated,
                field: "language".to_string(),
                old_value: profile.language.clone().map(serde_json::Value::String),
                new_value: Some(serde_json::Value::String(language.clone())),
                reason: Some("profile update".to_string()),
                changed_at: Utc::now(),
            };
            profile.change_history.push(change);
            profile.language = Some(language.clone());
        }

        profile.updated_at = Utc::now();
        profile.version += 1;

        // Save updated profile
        self.profile_repo.update(&profile.id, &profile).await?
            .ok_or_else(|| crate::error::AppError::Database("Failed to update profile".to_string()))
    }

    /// Add a preference for a user
    ///
    /// Stores key-value preferences in the profile.
    pub async fn add_preference(
        &self,
        user_id: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<()> {
        tracing::info!("Adding preference '{}' for user: {}", key, user_id);

        // Get existing profile
        let mut profile = self
            .profile_repo
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Profile not found for user: {}", user_id)))?;

        // Add preference
        profile.add_preference(key, value, Some("user preference"));

        // Save updated profile
        self.profile_repo.update(&profile.id, &profile).await?;
        Ok(())
    }

    /// Add a fact with source tracking
    ///
    /// Stores a user-provided fact with its category and source memory reference.
    pub async fn add_fact(
        &self,
        user_id: &str,
        fact: &str,
        category: &str,
        source_memory_id: Option<&str>,
    ) -> Result<ProfileFact> {
        tracing::info!("Adding fact for user: {}", user_id);

        // Get existing profile
        let mut profile = self
            .profile_repo
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Profile not found for user: {}", user_id)))?;

        // Parse category
        let fact_category = match category.to_lowercase().as_str() {
            "personal" => ProfileFactCategory::Personal,
            "professional" => ProfileFactCategory::Professional,
            "technical" => ProfileFactCategory::Technical,
            "project" => ProfileFactCategory::Project,
            "communication" => ProfileFactCategory::Communication,
            "lifestyle" => ProfileFactCategory::Lifestyle,
            _ => ProfileFactCategory::Other,
        };

        // Add fact with default confidence
        profile.add_fact(fact, fact_category, source_memory_id, 0.7);

        // Save updated profile
        self.profile_repo.update(&profile.id, &profile).await?;

        // Return the newly added fact
        profile.facts.last()
            .cloned()
            .ok_or_else(|| crate::error::AppError::Database("Failed to retrieve added fact".to_string()))
    }

    /// Verify a fact
    ///
    /// Marks a fact as verified with optional verification source.
    pub async fn verify_fact(
        &self,
        user_id: &str,
        fact_id: &str,
        verified_by: Option<&str>,
    ) -> Result<bool> {
        tracing::info!("Verifying fact {} for user: {}", fact_id, user_id);

        // Get existing profile
        let mut profile = self
            .profile_repo
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Profile not found for user: {}", user_id)))?;

        // Verify fact
        let verified = profile.verify_fact(fact_id, verified_by);

        if verified {
            // Add verification change to history
            let change = ProfileChange {
                version: profile.version + 1,
                change_type: ProfileChangeType::Verified,
                field: format!("facts.{}", fact_id),
                old_value: Some(serde_json::json!({ "verified": false })),
                new_value: Some(serde_json::json!({ "verified": true, "verified_by": verified_by })),
                reason: Some("fact verification".to_string()),
                changed_at: Utc::now(),
            };
            profile.change_history.push(change);
            profile.version += 1;

            // Save updated profile
            self.profile_repo.update(&profile.id, &profile).await?;
        }

        Ok(verified)
    }

    /// Update working hours
    ///
    /// Sets or updates the user's preferred working hours.
    pub async fn update_working_hours(
        &self,
        user_id: &str,
        working_hours: &WorkingHours,
    ) -> Result<()> {
        tracing::info!("Updating working hours for user: {}", user_id);

        // Get existing profile
        let mut profile = self
            .profile_repo
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Profile not found for user: {}", user_id)))?;

        // Add change to history
        let old_value = profile.working_hours.clone().map(|wh| {
            serde_json::json!({
                "start_day": wh.start_day,
                "start_hour": wh.start_hour,
                "end_day": wh.end_day,
                "end_hour": wh.end_hour,
                "timezone": wh.timezone,
                "flexible": wh.flexible
            })
        });
        let new_value = serde_json::json!({
            "start_day": working_hours.start_day,
            "start_hour": working_hours.start_hour,
            "end_day": working_hours.end_day,
            "end_hour": working_hours.end_hour,
            "timezone": working_hours.timezone,
            "flexible": working_hours.flexible
        });

        let change = ProfileChange {
            version: profile.version + 1,
            change_type: ProfileChangeType::Updated,
            field: "working_hours".to_string(),
            old_value,
            new_value: Some(new_value),
            reason: Some("working hours update".to_string()),
            changed_at: Utc::now(),
        };
        profile.change_history.push(change);

        // Update working hours
        profile.working_hours = Some(working_hours.clone());
        profile.updated_at = Utc::now();
        profile.version += 1;

        // Save updated profile
        self.profile_repo.update(&profile.id, &profile).await?;
        Ok(())
    }

    /// Get a summary of the user's profile
    ///
    /// Returns a compact view of the profile with key information.
    pub async fn get_profile_summary(&self, user_id: &str) -> Result<ProfileSummary> {
        tracing::debug!("Getting profile summary for user: {}", user_id);

        // Get profile
        let profile = self
            .profile_repo
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Profile not found for user: {}", user_id)))?;

        // Calculate working hours status
        let now = Utc::now();
        let current_time = now.time();
        let current_day = now.weekday().num_days_from_monday(); // 0-6, Monday=0

        let working_hours_status = if let Some(wh) = &profile.working_hours {
            let start_time = chrono::NaiveTime::from_hms_opt(wh.start_hour, 0, 0).unwrap();
            let end_time = chrono::NaiveTime::from_hms_opt(wh.end_hour, 0, 0).unwrap();

            let is_within = current_day >= wh.start_day
                && current_day <= wh.end_day
                && current_time >= start_time
                && current_time <= end_time;

            let working_hours_display = if wh.flexible {
                format!("{} ({}-{}, {} timezone, flexible)",
                    chrono::Weekday::from(current_day).to_string(),
                    wh.start_hour, wh.end_hour, wh.timezone)
            } else {
                format!("{} ({}-{}, {} timezone)",
                    chrono::Weekday::from(current_day).to_string(),
                    wh.start_hour, wh.end_hour, wh.timezone)
            };

            WorkingHoursStatus {
                is_configured: true,
                current_day,
                current_hour: current_time.hour() as u32,
                is_within_working_hours: is_within,
                working_hours_display: Some(working_hours_display),
            }
        } else {
            WorkingHoursStatus {
                is_configured: false,
                current_day,
                current_hour: current_time.hour() as u32,
                is_within_working_hours: false,
                working_hours_display: None,
            }
        };

        // Get top interests (limit to 5)
        let top_interests = profile.interests.iter().take(5).cloned().collect();

        // Count verified facts
        let total_facts = profile.facts.len() as u32;
        let verified_facts = profile.facts.iter().filter(|f| f.verified).count() as u32;

        Ok(ProfileSummary {
            name: profile.name,
            role: profile.role,
            total_facts,
            verified_facts,
            top_interests,
            tools_used: profile.tools_used.clone(),
            communication_style: profile.communication_style.clone(),
            working_hours_status,
        })
    }

    /// Merge profiles (for multi-device scenarios)
    ///
    /// Combines a source profile into a target profile, preserving verified facts
    /// and adding new information from the source.
    pub async fn merge_profiles(
        &self,
        target_user_id: &str,
        source_user_id: &str,
    ) -> Result<ProfileComparison> {
        tracing::info!("Merging profile {} into {}", source_user_id, target_user_id);

        // Get both profiles
        let target = self
            .profile_repo
            .get_by_user_id(target_user_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Target profile not found for user: {}", target_user_id)))?;

        let source = self
            .profile_repo
            .get_by_user_id(source_user_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Source profile not found for user: {}", source_user_id)))?;

        // Merge using repository
        let comparison = self.profile_repo.merge(&target.id, &source.id, "prefer_target").await?;

        tracing::info!("Profile merge completed: {} added, {} conflicts",
            comparison.added_facts.len(), comparison.conflicting_facts.len());

        Ok(comparison)
    }

    /// Add a tool to the user's profile
    ///
    /// Tracks tools the user commonly uses.
    pub async fn add_tool(&self, user_id: &str, tool: &str) -> Result<()> {
        tracing::info!("Adding tool '{}' for user: {}", tool, user_id);

        // Get existing profile
        let mut profile = self
            .profile_repo
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Profile not found for user: {}", user_id)))?;

        // Add tool
        profile.add_tool(tool);

        // Save updated profile
        self.profile_repo.update(&profile.id, &profile).await?;
        Ok(())
    }

    /// Add an interest to the user's profile
    ///
    /// Tracks areas of interest for the user.
    pub async fn add_interest(&self, user_id: &str, interest: &str) -> Result<()> {
        tracing::info!("Adding interest '{}' for user: {}", interest, user_id);

        // Get existing profile
        let mut profile = self
            .profile_repo
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Profile not found for user: {}", user_id)))?;

        // Add interest
        profile.add_interest(interest);

        // Save updated profile
        self.profile_repo.update(&profile.id, &profile).await?;
        Ok(())
    }

    /// Add a common task to the user's profile
    ///
    /// Tracks frequently performed tasks.
    pub async fn add_common_task(&self, user_id: &str, task: &str) -> Result<()> {
        tracing::info!("Adding common task '{}' for user: {}", task, user_id);

        // Get existing profile
        let mut profile = self
            .profile_repo
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Profile not found for user: {}", user_id)))?;

        // Add common task
        profile.add_common_task(task);

        // Save updated profile
        self.profile_repo.update(&profile.id, &profile).await?;
        Ok(())
    }

    /// Get profile by user ID
    ///
    /// Returns the full profile if it exists.
    pub async fn get_profile(&self, user_id: &str) -> Result<Option<Profile>> {
        self.profile_repo.get_by_user_id(user_id).await
    }

    /// Delete a profile
    ///
    /// Removes the profile from the system.
    pub async fn delete_profile(&self, user_id: &str) -> Result<bool> {
        tracing::info!("Deleting profile for user: {}", user_id);

        let profile = self
            .profile_repo
            .get_by_user_id(user_id)
            .await?
            .ok_or_else(|| crate::error::AppError::NotFound(format!("Profile not found for user: {}", user_id)))?;

        self.profile_repo.delete(&profile.id).await
    }
}

/// Create a ProfileManager service
pub fn create_profile_manager(
    profile_repo: Arc<dyn ProfileRepository>,
) -> ProfileManager {
    ProfileManager::new(profile_repo)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::profile_repository::ProfileRepository;
    use async_trait::async_trait;

    #[derive(Clone)]
    struct MockProfileRepository;

    #[async_trait]
    impl ProfileRepository for MockProfileRepository {
        async fn create(&self, profile: &Profile) -> Result<Profile> {
            Ok(profile.clone())
        }

        async fn get_by_id(&self, id: &str) -> Result<Option<Profile>> {
            Ok(None)
        }

        async fn get_by_user_id(&self, user_id: &str) -> Result<Option<Profile>> {
            if user_id == "existing_user" {
                let mut profile = Profile::new(user_id);
                profile.name = Some("Test User".to_string());
                return Ok(Some(profile));
            }
            Ok(None)
        }

        async fn update(&self, id: &str, profile: &Profile) -> Result<Option<Profile>> {
            Ok(Some(profile.clone()))
        }

        async fn delete(&self, _id: &str) -> Result<bool> {
            Ok(true)
        }

        async fn list(&self, _limit: usize, _start: usize) -> Result<Vec<Profile>> {
            Ok(vec![])
        }

        async fn count(&self) -> Result<u64> {
            Ok(0)
        }

        async fn search(&self, _query: &crate::models::profile::ProfileQuery) -> Result<Vec<Profile>> {
            Ok(vec![])
        }

        async fn merge(&self, _target_id: &str, _source_id: &str, _strategy: &str) -> Result<ProfileComparison> {
            Ok(ProfileComparison {
                added_facts: vec![],
                conflicting_facts: vec![],
                consistent_values: vec![],
            })
        }
    }

    #[tokio::test]
    async fn test_get_or_create_profile_existing() {
        let repo = Arc::new(MockProfileRepository);
        let manager = ProfileManager::new(repo);

        let profile = manager.get_or_create_profile("existing_user").await.unwrap();
        assert_eq!(profile.user_id, "existing_user");
        assert_eq!(profile.name, Some("Test User".to_string()));
    }

    #[tokio::test]
    async fn test_get_or_create_profile_new() {
        let repo = Arc::new(MockProfileRepository);
        let manager = ProfileManager::new(repo);

        let profile = manager.get_or_create_profile("new_user").await.unwrap();
        assert_eq!(profile.user_id, "new_user");
        assert!(profile.name.is_none());
    }

    #[tokio::test]
    async fn test_add_preference() {
        let repo = Arc::new(MockProfileRepository);
        let manager = ProfileManager::new(repo);

        let result = manager.add_preference(
            "existing_user",
            "theme",
            serde_json::json!("dark"),
        ).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_fact() {
        let repo = Arc::new(MockProfileRepository);
        let manager = ProfileManager::new(repo);

        let fact = manager.add_fact(
            "existing_user",
            "User prefers TypeScript",
            "technical",
            Some("mem_123"),
        ).await.unwrap();

        assert_eq!(fact.fact, "User prefers TypeScript");
        assert_eq!(fact.category, ProfileFactCategory::Technical);
    }

    #[tokio::test]
    async fn test_verify_fact() {
        let repo = Arc::new(MockProfileRepository);
        let manager = ProfileManager::new(repo);

        let result = manager.verify_fact("existing_user", "fact_123", Some("admin")).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_working_hours() {
        let repo = Arc::new(MockProfileRepository);
        let manager = ProfileManager::new(repo);

        let working_hours = WorkingHours {
            start_day: 1, // Monday
            start_hour: 9,
            end_day: 5, // Friday
            end_hour: 18,
            timezone: "UTC".to_string(),
            flexible: false,
        };

        let result = manager.update_working_hours("existing_user", &working_hours).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_profile_summary() {
        let repo = Arc::new(MockProfileRepository);
        let manager = ProfileManager::new(repo);

        let summary = manager.get_profile_summary("existing_user").await.unwrap();

        assert_eq!(summary.name, Some("Test User".to_string()));
        assert!(summary.tools_used.is_empty());
        assert!(summary.top_interests.is_empty());
    }

    #[tokio::test]
    async fn test_add_tool() {
        let repo = Arc::new(MockProfileRepository);
        let manager = ProfileManager::new(repo);

        let result = manager.add_tool("existing_user", "VSCode").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_interest() {
        let repo = Arc::new(MockProfileRepository);
        let manager = ProfileManager::new(repo);

        let result = manager.add_interest("existing_user", "Rust").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_common_task() {
        let repo = Arc::new(MockProfileRepository);
        let manager = ProfileManager::new(repo);

        let result = manager.add_common_task("existing_user", "Code Review").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_profile() {
        let repo = Arc::new(MockProfileRepository);
        let manager = ProfileManager::new(repo);

        let profile = manager.get_profile("existing_user").await.unwrap();
        assert!(profile.is_some());
        assert_eq!(profile.unwrap().user_id, "existing_user");
    }

    #[tokio::test]
    async fn test_delete_profile() {
        let repo = Arc::new(MockProfileRepository);
        let manager = ProfileManager::new(repo);

        let result = manager.delete_profile("existing_user").await;
        assert!(result.is_ok());
    }
}
