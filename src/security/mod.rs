//! Security Module
//!
//! Provides comprehensive security features for the Hippos API:
//! - Authentication (API Key + JWT)
//! - Authorization (RBAC)
//! - Rate Limiting
//! - Request Validation
//! - Security Middleware

pub mod auth;
pub mod config;
pub mod middleware;
pub mod rate_limit;
pub mod rbac;
pub mod validation;

pub use auth::{ApiKeyAuth, AuthToken, Authenticator, Credentials, JwtAuth, TokenType};
pub use config::SecuritySettings;
pub use rate_limit::{RateLimitConfig, RateLimitResult, RateLimiter};
pub use rbac::{ActionType, Authorizer, Permission, ResourceType, Role};
pub use validation::{RequestValidator, ValidatedRequest};
