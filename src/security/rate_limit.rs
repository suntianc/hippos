//! Rate Limiting Module
//!
//! Provides rate limiting functionality using token bucket algorithm.

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Rate limit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RateLimitConfig {
    /// Maximum requests per minute
    pub requests_per_minute: u32,
    /// Maximum requests per hour
    pub requests_per_hour: u32,
    /// Maximum requests per day
    pub requests_per_day: u32,
    /// Burst size (initial tokens)
    pub burst_size: u32,
    /// Window size in seconds for sliding window
    pub window_size_seconds: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            requests_per_day: 10000,
            burst_size: 10,
            window_size_seconds: 60,
        }
    }
}

impl RateLimitConfig {
    /// Create development rate limit config
    pub fn development() -> Self {
        Self {
            requests_per_minute: 100,
            requests_per_hour: 5000,
            requests_per_day: 50000,
            burst_size: 20,
            window_size_seconds: 60,
        }
    }

    /// Create production rate limit config
    pub fn production() -> Self {
        Self::default()
    }

    /// Create strict rate limit config (for sensitive endpoints)
    pub fn strict() -> Self {
        Self {
            requests_per_minute: 20,
            requests_per_hour: 200,
            requests_per_day: 1000,
            burst_size: 5,
            window_size_seconds: 60,
        }
    }
}

/// Rate limit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitResult {
    /// Request is allowed
    Allowed,
    /// Request is rate limited
    Limited {
        /// Seconds until retry is allowed
        retry_after: u64,
        /// Rate limit info
        limit: RateLimitInfo,
    },
    /// Rate limit info for allowed requests
    AllowedWithInfo {
        /// Remaining requests
        remaining: u32,
        /// Reset time
        reset_at: DateTime<Utc>,
        /// Rate limit info
        limit: RateLimitInfo,
    },
}

/// Rate limit information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Limit for the current window
    pub limit: u32,
    /// Remaining requests in current window
    pub remaining: u32,
    /// Window reset time
    pub reset_at: DateTime<Utc>,
    /// Window type
    pub window: String,
}

/// Client identifier for rate limiting
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum RateLimitClient {
    /// API key based client
    ApiKey(String),
    /// IP address based client
    Ip(String),
    /// JWT subject based client
    JwtSubject(String),
    /// Custom client ID
    Custom(String),
}

impl RateLimitClient {
    /// Create from API key
    pub fn from_api_key(key: &str) -> Self {
        RateLimitClient::ApiKey(key.to_string())
    }

    /// Create from IP address
    pub fn from_ip(ip: &str) -> Self {
        RateLimitClient::Ip(ip.to_string())
    }

    /// Create from JWT subject
    pub fn from_jwt_subject(subject: &str) -> Self {
        RateLimitClient::JwtSubject(subject.to_string())
    }

    /// Get client identifier string
    pub fn as_str(&self) -> &str {
        match self {
            RateLimitClient::ApiKey(s) => s.as_str(),
            RateLimitClient::Ip(s) => s.as_str(),
            RateLimitClient::JwtSubject(s) => s.as_str(),
            RateLimitClient::Custom(s) => s.as_str(),
        }
    }
}

/// In-memory rate limiter using sliding window
#[derive(Debug, Clone)]
pub struct RateLimiter {
    /// Rate limit configuration
    config: RateLimitConfig,
    /// Request history (client -> timestamps)
    request_history: Arc<RwLock<HashMap<String, Vec<DateTime<Utc>>>>>,
    /// Whether rate limiting is enabled
    enabled: bool,
}

impl RateLimiter {
    /// Create new rate limiter
    pub fn new(config: RateLimitConfig, enabled: bool) -> Self {
        Self {
            config,
            request_history: Arc::new(RwLock::new(HashMap::new())),
            enabled,
        }
    }

    /// Create development rate limiter
    pub fn development() -> Self {
        Self::new(RateLimitConfig::development(), false)
    }

    /// Create production rate limiter
    pub fn production() -> Self {
        Self::new(RateLimitConfig::production(), true)
    }

    /// Create from security settings
    pub fn from_settings(
        requests_per_minute: u32,
        requests_per_hour: u32,
        burst_size: u32,
        enabled: bool,
    ) -> Self {
        let config = RateLimitConfig {
            requests_per_minute,
            requests_per_hour,
            requests_per_day: requests_per_hour * 24,
            burst_size,
            ..Default::default()
        };
        Self::new(config, enabled)
    }

    /// Check rate limit for a client
    pub async fn check_rate_limit(&self, client: &RateLimitClient) -> RateLimitResult {
        if !self.enabled {
            return RateLimitResult::Allowed;
        }

        let client_id = client.as_str();
        let now = Utc::now();

        // Clean old entries and get recent requests
        let recent_requests = {
            let history = self.request_history.read().await;

            // Get requests from last minute
            let minute_cutoff = now - Duration::minutes(1);
            let hour_cutoff = now - Duration::hours(1);

            let minute_count = history
                .get(client_id)
                .map(|v| v.iter().filter(|t| **t > minute_cutoff).count())
                .unwrap_or(0);

            let hour_count = history
                .get(client_id)
                .map(|v| v.iter().filter(|t| **t > hour_cutoff).count())
                .unwrap_or(0);

            let reset_at = now + Duration::seconds(self.config.window_size_seconds as i64);

            if minute_count >= self.config.requests_per_minute as usize {
                return RateLimitResult::Limited {
                    retry_after: 60,
                    limit: RateLimitInfo {
                        limit: self.config.requests_per_minute,
                        remaining: 0,
                        reset_at,
                        window: "minute".to_string(),
                    },
                };
            }

            if hour_count >= self.config.requests_per_hour as usize {
                return RateLimitResult::Limited {
                    retry_after: 3600,
                    limit: RateLimitInfo {
                        limit: self.config.requests_per_hour,
                        remaining: 0,
                        reset_at,
                        window: "hour".to_string(),
                    },
                };
            }

            (minute_count, hour_count, reset_at)
        };

        // Record this request
        {
            let mut history = self.request_history.write().await;
            history
                .entry(client_id.to_string())
                .or_insert_with(Vec::new)
                .push(now);
        }

        let remaining = self.config.requests_per_minute as i32 - recent_requests.0 as i32;

        RateLimitResult::AllowedWithInfo {
            remaining: remaining as u32,
            reset_at: recent_requests.2,
            limit: RateLimitInfo {
                limit: self.config.requests_per_minute,
                remaining: remaining as u32,
                reset_at: recent_requests.2,
                window: "minute".to_string(),
            },
        }
    }

    /// Record a request for a client
    pub async fn record_request(&self, client: &RateLimitClient) {
        if !self.enabled {
            return;
        }

        let client_id = client.as_str();
        let now = Utc::now();

        let mut history = self.request_history.write().await;
        history
            .entry(client_id.to_string())
            .or_insert_with(Vec::new)
            .push(now);
    }

    /// Get current usage stats for a client
    pub async fn get_usage_stats(&self, client: &RateLimitClient) -> Vec<RateLimitInfo> {
        let client_id = client.as_str();
        let now = Utc::now();
        let history = self.request_history.read().await;

        let minute_cutoff = now - Duration::minutes(1);
        let hour_cutoff = now - Duration::hours(1);
        let day_cutoff = now - Duration::days(1);

        let minute_count = history
            .get(client_id)
            .map(|v| v.iter().filter(|t| **t > minute_cutoff).count())
            .unwrap_or(0);

        let hour_count = history
            .get(client_id)
            .map(|v| v.iter().filter(|t| **t > hour_cutoff).count())
            .unwrap_or(0);

        let day_count = history
            .get(client_id)
            .map(|v| v.iter().filter(|t| **t > day_cutoff).count())
            .unwrap_or(0);

        vec![
            RateLimitInfo {
                limit: self.config.requests_per_minute,
                remaining: self
                    .config
                    .requests_per_minute
                    .saturating_sub(minute_count as u32),
                reset_at: now + Duration::minutes(1),
                window: "minute".to_string(),
            },
            RateLimitInfo {
                limit: self.config.requests_per_hour,
                remaining: self
                    .config
                    .requests_per_hour
                    .saturating_sub(hour_count as u32),
                reset_at: now + Duration::hours(1),
                window: "hour".to_string(),
            },
            RateLimitInfo {
                limit: self.config.requests_per_day,
                remaining: self
                    .config
                    .requests_per_day
                    .saturating_sub(day_count as u32),
                reset_at: now + Duration::days(1),
                window: "day".to_string(),
            },
        ]
    }

    /// Clear rate limit data for a client (for testing/admin)
    pub async fn clear_client(&self, client: &RateLimitClient) {
        let client_id = client.as_str();
        let mut history = self.request_history.write().await;
        history.remove(client_id);
    }

    /// Clear all rate limit data (for testing)
    pub async fn clear_all(&self) {
        let mut history = self.request_history.write().await;
        history.clear();
    }
}

/// Async trait for rate limiters (allows custom implementations)
#[async_trait]
pub trait AsyncRateLimiter: Send + Sync {
    /// Check rate limit and return result
    async fn check(&self, client: &RateLimitClient) -> RateLimitResult;
    /// Record a request
    async fn record(&self, client: &RateLimitClient);
    /// Get usage stats
    async fn stats(&self, client: &RateLimitClient) -> Vec<RateLimitInfo>;
}

#[async_trait]
impl AsyncRateLimiter for RateLimiter {
    async fn check(&self, client: &RateLimitClient) -> RateLimitResult {
        self.check_rate_limit(client).await
    }

    async fn record(&self, client: &RateLimitClient) {
        self.record_request(client).await
    }

    async fn stats(&self, client: &RateLimitClient) -> Vec<RateLimitInfo> {
        self.get_usage_stats(client).await
    }
}

/// Rate limit middleware helper
pub struct RateLimitMiddleware;

impl RateLimitMiddleware {
    /// Extract client identifier from request
    pub fn extract_client_id<B>(
        req: &axum::http::Request<B>,
        claims: Option<&crate::security::auth::Claims>,
    ) -> RateLimitClient {
        // Try to get client ID from claims first
        if let Some(claims) = claims {
            return RateLimitClient::from_jwt_subject(&claims.sub);
        }

        // Fall back to API key header
        if let Some(api_key) = req.headers().get("X-API-Key") {
            if let Ok(key) = api_key.to_str() {
                return RateLimitClient::from_api_key(key);
            }
        }

        // Fall back to Authorization header
        if let Some(auth) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth.to_str() {
                if auth_str.starts_with("ApiKey ") {
                    let key = &auth_str[7..];
                    return RateLimitClient::from_api_key(key);
                }
            }
        }

        // Finally, use IP address
        if let Some(ip) = req.headers().get("X-Forwarded-For") {
            if let Ok(ip_str) = ip.to_str() {
                return RateLimitClient::from_ip(ip_str.split(',').next().unwrap_or(ip_str).trim());
            }
        }

        if let Some(ip) = req.headers().get("X-Real-IP") {
            if let Ok(ip_str) = ip.to_str() {
                return RateLimitClient::from_ip(ip_str);
            }
        }

        // Use peer address if available (try extensions first)
        if let Some(peer) = req.extensions().get::<std::net::SocketAddr>() {
            return RateLimitClient::from_ip(&peer.to_string());
        }

        // Generate a random client ID as last resort
        RateLimitClient::Custom(format!("unknown-{}", uuid::Uuid::new_v4()))
    }
}
