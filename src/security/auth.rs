//! Authentication Module
//!
//! Provides authentication mechanisms:
//! - API Key authentication
//! - JWT (JSON Web Token) authentication

use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

use crate::error::{AppError, Result};
use crate::security::config::SecuritySettings;

/// Credentials for authentication
#[derive(Debug, Clone)]
pub struct Credentials {
    /// API key (if provided)
    pub api_key: Option<String>,
    /// JWT token (if provided)
    pub jwt_token: Option<String>,
}

impl Credentials {
    /// Create new credentials
    pub fn new(api_key: Option<String>, jwt_token: Option<String>) -> Self {
        Self { api_key, jwt_token }
    }

    /// Try to extract credentials from Authorization header
    pub fn from_authorization_header(auth_header: Option<&str>) -> Self {
        match auth_header {
            Some(header) if header.starts_with("ApiKey ") => {
                Self::new(Some(header[7..].to_string()), None)
            }
            Some(header) if header.starts_with("Bearer ") => {
                Self::new(None, Some(header[7..].to_string()))
            }
            _ => Self::new(None, None),
        }
    }
}

/// Token type enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    /// API Key token
    ApiKey,
    /// Bearer token (JWT)
    Bearer,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::ApiKey => write!(f, "ApiKey"),
            TokenType::Bearer => write!(f, "Bearer"),
        }
    }
}

/// Authentication token result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// The token string
    pub token: String,
    /// Type of token
    pub token_type: TokenType,
    /// Token expiration time
    pub expires_at: DateTime<Utc>,
    /// Associated tenant ID (for API keys)
    pub tenant_id: Option<String>,
}

impl AuthToken {
    /// Create a new authentication token
    pub fn new(
        token: String,
        token_type: TokenType,
        expires_at: DateTime<Utc>,
        tenant_id: Option<String>,
    ) -> Self {
        Self {
            token,
            token_type,
            expires_at,
            tenant_id,
        }
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// JWT Claims structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject (usually user ID)
    pub sub: String,
    /// Tenant ID
    pub tenant_id: String,
    /// User role
    pub role: String,
    /// Token expiration timestamp
    pub exp: usize,
    /// Token not before timestamp
    pub nbf: usize,
    /// Issued at timestamp
    pub iat: usize,
    /// Issuer
    pub iss: String,
    /// Audience
    pub aud: String,
    /// Unique token ID
    pub jti: String,
}

impl Claims {
    /// Create new claims
    pub fn new(
        sub: String,
        tenant_id: String,
        role: String,
        expiry_seconds: u64,
        issuer: String,
        audience: String,
    ) -> Self {
        let now = Utc::now();
        let exp = now.timestamp() as usize + expiry_seconds as usize;
        let iat = now.timestamp() as usize;
        let nbf = iat;

        Self {
            sub,
            tenant_id,
            role,
            exp,
            nbf,
            iat,
            iss: issuer,
            aud: audience,
            jti: Uuid::new_v4().to_string(),
        }
    }

    /// Check if claims are expired
    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() as usize > self.exp
    }
}

/// Authentication trait for different authentication methods
#[async_trait]
pub trait Authenticator: Send + Sync {
    /// Authenticate credentials and return a token
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken>;
    /// Validate a token and return claims
    async fn validate_token(&self, token: &str) -> Result<Claims>;
    /// Get the authenticator type
    fn authenticator_type(&self) -> &'static str;
}

/// API Key based authentication
#[derive(Debug, Clone)]
pub struct ApiKeyAuth {
    /// Valid API keys map (key -> tenant_id)
    valid_keys: HashMap<String, String>,
    /// Whether authentication is enabled
    enabled: bool,
}

impl ApiKeyAuth {
    /// Create new API key authenticator
    pub fn new(api_keys: std::collections::HashSet<String>) -> Self {
        let valid_keys: HashMap<String, String> = api_keys
            .into_iter()
            .map(|key| (key.clone(), key.clone()))
            .collect();

        let enabled = !valid_keys.is_empty();

        Self {
            valid_keys,
            enabled,
        }
    }

    /// Create a development API key authenticator with default key
    pub fn development() -> Self {
        let mut valid_keys = HashMap::new();
        valid_keys.insert("dev-api-key".to_string(), "dev-tenant".to_string());
        Self {
            valid_keys,
            enabled: true,
        }
    }
}

#[async_trait]
impl Authenticator for ApiKeyAuth {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken> {
        if !self.enabled {
            return Err(AppError::Authentication(
                "API key authentication is disabled".to_string(),
            ));
        }

        let api_key = credentials
            .api_key
            .as_ref()
            .ok_or_else(|| AppError::Authentication("No API key provided".to_string()))?;

        let tenant_id = self
            .valid_keys
            .get(api_key)
            .ok_or_else(|| AppError::Authentication("Invalid API key".to_string()))?;

        // API keys don't expire in this simple implementation
        // In production, you might want to add expiration
        let expires_at = Utc.timestamp_opt(2147483647, 0).single().unwrap();

        Ok(AuthToken::new(
            api_key.clone(),
            TokenType::ApiKey,
            expires_at,
            Some(tenant_id.clone()),
        ))
    }

    async fn validate_token(&self, token: &str) -> Result<Claims> {
        if !self.enabled {
            return Err(AppError::Authentication(
                "API key authentication is disabled".to_string(),
            ));
        }

        // For API keys, we just check if it's valid and return basic claims
        let tenant_id = self
            .valid_keys
            .get(token)
            .ok_or_else(|| AppError::Authentication("Invalid API key".to_string()))?;

        Ok(Claims {
            sub: token.to_string(),
            tenant_id: tenant_id.clone(),
            role: "user".to_string(),
            exp: 2147483647,
            nbf: 0,
            iat: Utc::now().timestamp() as usize,
            iss: "hippos".to_string(),
            aud: "hippos-api".to_string(),
            jti: Uuid::new_v4().to_string(),
        })
    }

    fn authenticator_type(&self) -> &'static str {
        "ApiKey"
    }
}

/// JWT based authentication
#[derive(Debug, Clone)]
pub struct JwtAuth {
    /// Secret key for encoding
    _encoding_key: EncodingKey,
    /// Secret key for decoding
    decoding_key: DecodingKey,
    /// JWT issuer
    issuer: String,
    /// JWT audience
    audience: String,
    /// Token expiry time in seconds
    _expiry_seconds: u64,
    /// Whether authentication is enabled
    enabled: bool,
}

impl JwtAuth {
    /// Create new JWT authenticator
    pub fn new(secret: String, issuer: String, audience: String, expiry_seconds: u64) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());

        Self {
            _encoding_key: encoding_key,
            decoding_key,
            issuer,
            audience,
            _expiry_seconds: expiry_seconds,
            enabled: true,
        }
    }

    /// Create a development JWT authenticator
    pub fn development() -> Self {
        Self::new(
            "dev-secret-change-in-production-min-32-chars".to_string(),
            "hippos".to_string(),
            "hippos-api".to_string(),
            3600,
        )
    }
}

#[async_trait]
impl Authenticator for JwtAuth {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken> {
        if !self.enabled {
            return Err(AppError::Authentication(
                "JWT authentication is disabled".to_string(),
            ));
        }

        let jwt_token = credentials
            .jwt_token
            .as_ref()
            .ok_or_else(|| AppError::Authentication("No JWT token provided".to_string()))?;

        self.validate_token(jwt_token).await.map(|claims| {
            let expires_at = Utc.timestamp_opt(claims.exp as i64, 0).single().unwrap();
            AuthToken::new(
                jwt_token.clone(),
                TokenType::Bearer,
                expires_at,
                Some(claims.tenant_id.clone()),
            )
        })
    }

    async fn validate_token(&self, token: &str) -> Result<Claims> {
        if !self.enabled {
            return Err(AppError::Authentication(
                "JWT authentication is disabled".to_string(),
            ));
        }

        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[self.issuer.clone()]);
        validation.set_audience(&[self.audience.clone()]);
        validation.validate_nbf = true;

        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|token_data| token_data.claims)
            .map_err(|e| AppError::Authentication(format!("Invalid JWT token: {}", e)))
    }

    fn authenticator_type(&self) -> &'static str {
        "JWT"
    }
}

/// Combined authenticator that tries multiple methods
#[derive(Debug, Clone)]
pub struct CombinedAuthenticator {
    /// API key authenticator
    api_key_auth: Option<ApiKeyAuth>,
    /// JWT authenticator
    jwt_auth: Option<JwtAuth>,
}

impl CombinedAuthenticator {
    /// Create new combined authenticator
    pub fn new(api_key_auth: Option<ApiKeyAuth>, jwt_auth: Option<JwtAuth>) -> Self {
        Self {
            api_key_auth,
            jwt_auth,
        }
    }

    /// Create a development combined authenticator
    pub fn development() -> Self {
        Self::new(
            Some(ApiKeyAuth::development()),
            Some(JwtAuth::development()),
        )
    }

    /// Create from security settings
    pub fn from_settings(settings: &SecuritySettings) -> Self {
        let api_key_auth = if settings.api_key_auth_enabled {
            Some(ApiKeyAuth::new(settings.api_keys.clone()))
        } else {
            None
        };

        let jwt_auth = if settings.jwt_auth_enabled {
            Some(JwtAuth::new(
                settings.jwt_secret.clone(),
                settings.jwt_issuer.clone(),
                settings.jwt_audience.clone(),
                settings.jwt_expiry_seconds,
            ))
        } else {
            None
        };

        Self::new(api_key_auth, jwt_auth)
    }
}

#[async_trait]
impl Authenticator for CombinedAuthenticator {
    async fn authenticate(&self, credentials: &Credentials) -> Result<AuthToken> {
        // Try API key first if available
        if let Some(api_key_auth) = &self.api_key_auth {
            if credentials.api_key.is_some() {
                return api_key_auth.authenticate(credentials).await;
            }
        }

        // Try JWT if available
        if let Some(jwt_auth) = &self.jwt_auth {
            if credentials.jwt_token.is_some() {
                return jwt_auth.authenticate(credentials).await;
            }
        }

        Err(AppError::Authentication(
            "No valid authentication method provided".to_string(),
        ))
    }

    async fn validate_token(&self, token: &str) -> Result<Claims> {
        // Try API key validation first
        if let Some(api_key_auth) = &self.api_key_auth {
            if api_key_auth.authenticator_type() == "ApiKey" {
                return api_key_auth.validate_token(token).await;
            }
        }

        // Try JWT validation
        if let Some(jwt_auth) = &self.jwt_auth {
            return jwt_auth.validate_token(token).await;
        }

        Err(AppError::Authentication(
            "No authenticator available to validate token".to_string(),
        ))
    }

    fn authenticator_type(&self) -> &'static str {
        "Combined"
    }
}

/// JWT token generation helper
pub struct JwtTokenGenerator {
    encoding_key: EncodingKey,
    issuer: String,
    audience: String,
    expiry_seconds: u64,
}

impl JwtTokenGenerator {
    /// Create new token generator
    pub fn new(secret: String, issuer: String, audience: String, expiry_seconds: u64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            issuer,
            audience,
            expiry_seconds,
        }
    }

    /// Generate a new JWT token
    pub fn generate_token(&self, sub: String, tenant_id: String, role: String) -> Result<String> {
        let claims = Claims::new(
            sub,
            tenant_id,
            role,
            self.expiry_seconds,
            self.issuer.clone(),
            self.audience.clone(),
        );

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| AppError::Authentication(format!("Failed to generate token: {}", e)))
    }
}
