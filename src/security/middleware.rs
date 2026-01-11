//! Security Middleware Module
//!
//! Provides Axum middleware for authentication, authorization, rate limiting, and security headers.

use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode, header},
    middleware::Next,
    response::Response,
};
use chrono::{DateTime, Utc};
use std::result::Result as StdResult;
use std::sync::Arc;

use crate::api::app_state::AppState;
use crate::error::AppError;
use crate::security::auth::{Authenticator, Claims, Credentials};
use crate::security::rate_limit::{RateLimitMiddleware, RateLimitResult, RateLimiter};
use crate::security::rbac::{ActionType, Authorizer, Permission, ResourceType};
use crate::security::validation::RequestValidator;

/// Extension trait for adding claims to request extensions
pub trait RequestClaimsExt {
    fn claims(&self) -> Option<&Claims>;
    fn set_claims(&mut self, claims: Claims);
}

impl RequestClaimsExt for Request<Body> {
    fn claims(&self) -> Option<&Claims> {
        self.extensions().get::<Claims>()
    }

    fn set_claims(&mut self, claims: Claims) {
        self.extensions_mut().insert(claims);
    }
}

/// Authentication middleware
pub async fn auth_middleware(
    req: Request<Body>,
    next: Next,
    authenticator: Arc<dyn Authenticator>,
) -> StdResult<Response, StatusCode> {
    let credentials = extract_credentials(&req);

    match authenticator.authenticate(&credentials).await {
        Ok(_token) => {
            let claims = authenticator
                .validate_token(&_token.token)
                .await
                .map_err(|_| StatusCode::UNAUTHORIZED)?;

            let mut req = req;
            req.set_claims(claims);

            Ok(next.run(req).await)
        }
        Err(e) => {
            let status = match e {
                AppError::Authentication(_) => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            Err(status)
        }
    }
}

/// Extract credentials from request headers
fn extract_credentials(req: &Request<Body>) -> Credentials {
    let auth_header = req.headers().get(header::AUTHORIZATION);

    if let Some(auth) = auth_header {
        if let Ok(auth_str) = auth.to_str() {
            return Credentials::from_authorization_header(Some(auth_str));
        }
    }

    if let Some(api_key) = req.headers().get("X-API-Key") {
        if let Ok(key) = api_key.to_str() {
            return Credentials::new(Some(key.to_string()), None);
        }
    }

    Credentials::new(None, None)
}

/// Authorization middleware
pub async fn authorize_middleware(
    req: Request<Body>,
    next: Next,
    authorizer: Arc<dyn Authorizer>,
    resource: ResourceType,
    action: ActionType,
) -> StdResult<Response, StatusCode> {
    let claims = req.claims().ok_or(StatusCode::UNAUTHORIZED)?;

    let permission = Permission::new(resource.clone(), action.clone());

    if authorizer.check_permission(claims, &permission).await {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    req: Request<Body>,
    next: Next,
    rate_limiter: Arc<RateLimiter>,
) -> StdResult<Response, StatusCode> {
    let client = RateLimitMiddleware::extract_client_id(&req, req.claims());

    match rate_limiter.check_rate_limit(&client).await {
        RateLimitResult::Allowed => {
            let response = next.run(req).await;
            Ok(response)
        }
        RateLimitResult::AllowedWithInfo {
            remaining,
            reset_at,
            limit,
        } => {
            let req = req;
            let response = next.run(req).await;
            add_rate_limit_headers(response, remaining, &reset_at, &limit)
        }
        RateLimitResult::Limited { retry_after, limit } => {
            let mut response = Response::new(Body::from("Too Many Requests"));
            *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
            response.headers_mut().insert(
                header::RETRY_AFTER,
                retry_after.to_string().parse().unwrap(),
            );
            add_rate_limit_headers(response, 0, &Utc::now(), &limit)
        }
    }
}

/// Add rate limit headers to response
fn add_rate_limit_headers(
    mut response: Response,
    remaining: u32,
    reset_at: &DateTime<Utc>,
    limit: &crate::security::rate_limit::RateLimitInfo,
) -> StdResult<Response, StatusCode> {
    response.headers_mut().insert(
        "X-RateLimit-Limit",
        limit.limit.to_string().parse().unwrap(),
    );
    response.headers_mut().insert(
        "X-RateLimit-Remaining",
        remaining.to_string().parse().unwrap(),
    );
    response.headers_mut().insert(
        "X-RateLimit-Reset",
        format!("{}", reset_at.timestamp()).parse().unwrap(),
    );
    Ok(response)
}

/// Request validation middleware
pub async fn validation_middleware(
    req: Request<Body>,
    next: Next,
    validator: Arc<RequestValidator>,
    max_body_size: usize,
) -> StdResult<Response, StatusCode> {
    if matches!(req.method(), &Method::POST | &Method::PUT | &Method::PATCH) {
        let content_type = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|h| h.to_str().ok());

        if let Err(e) = validator.validate_content_type(content_type) {
            let mut response = Response::new(Body::from(format!("Validation error: {}", e)));
            *response.status_mut() = StatusCode::BAD_REQUEST;
            return Ok(response);
        }
    }

    if let Some(content_length) = req
        .headers()
        .get(header::CONTENT_LENGTH)
        .and_then(|h| h.to_str().ok())
    {
        if let Ok(size) = content_length.parse::<usize>() {
            if size > max_body_size {
                let mut response = Response::new(Body::from("Request body too large"));
                *response.status_mut() = StatusCode::PAYLOAD_TOO_LARGE;
                return Ok(response);
            }
        }
    }

    Ok(next.run(req).await)
}

/// Security headers middleware
pub async fn security_headers_middleware(
    req: Request<Body>,
    next: Next,
) -> StdResult<Response, StatusCode> {
    let mut response = next.run(req).await;

    response
        .headers_mut()
        .insert("X-Content-Type-Options", "nosniff".parse().unwrap());

    response
        .headers_mut()
        .insert("X-Frame-Options", "DENY".parse().unwrap());

    response
        .headers_mut()
        .insert("X-XSS-Protection", "1; mode=block".parse().unwrap());

    response.headers_mut().insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );

    response.headers_mut().insert(
        "Content-Security-Policy",
        "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'"
            .parse()
            .unwrap(),
    );

    response.headers_mut().insert(
        "Referrer-Policy",
        "strict-origin-when-cross-origin".parse().unwrap(),
    );

    response.headers_mut().insert(
        "Permissions-Policy",
        "geolocation=(), microphone=(), camera=()".parse().unwrap(),
    );

    Ok(response)
}

/// CORS middleware
pub async fn cors_middleware(
    req: Request<Body>,
    next: Next,
    allowed_origins: Vec<String>,
) -> StdResult<Response, StatusCode> {
    let origin = req.headers().get("Origin").cloned();

    let response = next.run(req).await;

    if let Some(origin_value) = origin {
        let origin_str = origin_value.to_str().unwrap_or("");

        if allowed_origins.iter().any(|o| o == "*" || o == origin_str) {
            let mut response = response;
            response
                .headers_mut()
                .insert("Access-Control-Allow-Origin", origin_value);
            response.headers_mut().insert(
                "Access-Control-Allow-Methods",
                "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
            );
            response.headers_mut().insert(
                "Access-Control-Allow-Headers",
                "Content-Type, Authorization, X-API-Key".parse().unwrap(),
            );
            response
                .headers_mut()
                .insert("Access-Control-Max-Age", "86400".parse().unwrap());
            return Ok(response);
        }
    }

    Ok(response)
}

/// Handle OPTIONS preflight requests
pub async fn handle_options_preflight(
    req: Request<Body>,
    allowed_origins: Vec<String>,
) -> Response {
    let origin = req.headers().get("Origin").cloned();
    let access_control_request_method = req.headers().get("Access-Control-Request-Method").cloned();

    let mut response = Response::new(Body::from(""));
    *response.status_mut() = StatusCode::NO_CONTENT;

    if let Some(origin_value) = origin {
        let origin_str = origin_value.to_str().unwrap_or("");

        if allowed_origins.iter().any(|o| o == "*" || o == origin_str) {
            response
                .headers_mut()
                .insert("Access-Control-Allow-Origin", origin_value);
            response.headers_mut().insert(
                "Access-Control-Allow-Methods",
                "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap(),
            );

            if let Some(_method) = access_control_request_method {
                response.headers_mut().insert(
                    "Access-Control-Allow-Headers",
                    "Content-Type, Authorization, X-API-Key".parse().unwrap(),
                );
            }

            response
                .headers_mut()
                .insert("Access-Control-Max-Age", "86400".parse().unwrap());
        }
    }

    response
}

/// Middleware type alias - boxed async function that returns Result
pub type BoxedMiddleware = Box<
    dyn Fn(
            Request<Body>,
            Next,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = StdResult<Response, StatusCode>> + Send>,
        > + Send
        + Sync,
>;

/// Builder for middleware chains
#[derive(Debug, Clone)]
pub struct MiddlewareBuilder {
    app_state: AppState,
    require_auth: bool,
    auth_resource: Option<ResourceType>,
    auth_action: Option<ActionType>,
    enable_rate_limit: bool,
    enable_validation: bool,
    enable_security_headers: bool,
    cors_origins: Vec<String>,
}

impl MiddlewareBuilder {
    pub fn new(app_state: AppState) -> Self {
        Self {
            app_state,
            require_auth: false,
            auth_resource: None,
            auth_action: None,
            enable_rate_limit: false,
            enable_validation: false,
            enable_security_headers: true,
            cors_origins: vec![],
        }
    }

    pub fn with_auth(mut self, resource: ResourceType, action: ActionType) -> Self {
        self.require_auth = true;
        self.auth_resource = Some(resource);
        self.auth_action = Some(action);
        self
    }

    pub fn with_rate_limit(mut self) -> Self {
        self.enable_rate_limit = true;
        self
    }

    pub fn with_validation(mut self) -> Self {
        self.enable_validation = true;
        self
    }

    pub fn with_cors(mut self, origins: Vec<String>) -> Self {
        self.cors_origins = origins;
        self
    }

    pub fn without_security_headers(mut self) -> Self {
        self.enable_security_headers = false;
        self
    }

    /// Build the middleware stack
    pub fn build(self) -> Vec<BoxedMiddleware> {
        let mut middleware: Vec<BoxedMiddleware> = Vec::new();

        if self.enable_security_headers {
            middleware.push(Box::new(|req, next| {
                Box::pin(async move { security_headers_middleware(req, next).await })
            }));
        }

        if !self.cors_origins.is_empty() {
            let cors_origins = self.cors_origins.clone();
            middleware.push(Box::new(move |req, next| {
                let origins = cors_origins.clone();
                Box::pin(async move { cors_middleware(req, next, origins).await })
            }));
        }

        // Rate limiting
        if self.enable_rate_limit {
            let rate_limiter = self.app_state.rate_limiter.clone();
            middleware.push(Box::new(move |req, next| {
                let limiter = rate_limiter.clone();
                Box::pin(async move { rate_limit_middleware(req, next, limiter).await })
            }));
        }

        // Authentication
        if self.require_auth {
            let authenticator = self.app_state.authenticator.clone();
            middleware.push(Box::new(move |req, next| {
                let auth = authenticator.clone();
                Box::pin(async move { auth_middleware(req, next, auth).await })
            }));
        }

        // Authorization
        if self.require_auth && self.auth_resource.is_some() && self.auth_action.is_some() {
            let authorizer = self.app_state.authorizer.clone();
            let resource = self.auth_resource.clone().unwrap();
            let action = self.auth_action.clone().unwrap();
            middleware.push(Box::new(move |req, next| {
                let authz = authorizer.clone();
                let res = resource.clone();
                let act = action.clone();
                Box::pin(async move { authorize_middleware(req, next, authz, res, act).await })
            }));
        }

        // Validation
        if self.enable_validation {
            let validator = Arc::new(RequestValidator::production());
            middleware.push(Box::new(move |req, next| {
                let val = validator.clone();
                Box::pin(
                    async move { validation_middleware(req, next, val, 10 * 1024 * 1024).await },
                )
            }));
        }

        middleware
    }
}
