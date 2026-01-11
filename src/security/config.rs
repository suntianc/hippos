//! Security Configuration
//!
//! Security-related configuration settings.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Extended security configuration for the security layer
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct SecuritySettings {
    /// JWT secret key for token validation
    pub jwt_secret: String,
    /// JWT issuer
    pub jwt_issuer: String,
    /// JWT audience
    pub jwt_audience: String,
    /// JWT expiry time in seconds
    pub jwt_expiry_seconds: u64,
    /// Valid API keys (map key -> tenant_id)
    pub api_keys: HashSet<String>,
    /// Rate limit requests per minute
    pub rate_limit_requests_per_minute: u32,
    /// Rate limit requests per hour
    pub rate_limit_requests_per_hour: u32,
    /// Rate limit burst size
    pub rate_limit_burst_size: u32,
    /// Enable rate limiting
    pub rate_limit_enabled: bool,
    /// Enable API key authentication
    pub api_key_auth_enabled: bool,
    /// Enable JWT authentication
    pub jwt_auth_enabled: bool,
    /// CORS allowed origins
    pub cors_allowed_origins: Vec<String>,
    /// Maximum request body size in bytes
    pub max_request_size: usize,
    /// Enable request validation
    pub validation_enabled: bool,
    /// Enable security headers
    pub security_headers_enabled: bool,
}

impl SecuritySettings {
    /// Create development security settings
    pub fn development() -> Self {
        let mut api_keys = HashSet::new();
        api_keys.insert("dev-api-key-change-in-production".to_string());

        Self {
            jwt_secret: "dev-secret-change-in-production-min-32-chars".to_string(),
            jwt_issuer: "hippos".to_string(),
            jwt_audience: "hippos-api".to_string(),
            jwt_expiry_seconds: 3600,
            api_keys,
            rate_limit_requests_per_minute: 60,
            rate_limit_requests_per_hour: 1000,
            rate_limit_burst_size: 10,
            rate_limit_enabled: false,
            api_key_auth_enabled: true,
            jwt_auth_enabled: true,
            cors_allowed_origins: vec!["http://localhost:3000".to_string()],
            max_request_size: 10 * 1024 * 1024,
            validation_enabled: true,
            security_headers_enabled: true,
        }
    }

    /// Create production security settings
    pub fn production() -> Self {
        let mut settings = Self::development();
        settings.rate_limit_enabled = true;
        settings.security_headers_enabled = true;
        settings
    }

    /// Check if a JWT secret is set (indicates production-like environment)
    pub fn has_jwt_secret(&self) -> bool {
        !self.jwt_secret.is_empty()
    }
}
