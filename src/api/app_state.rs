use crate::security::auth::Authenticator;
use crate::security::rate_limit::RateLimiter;
use crate::security::rbac::Authorizer;
use crate::services::dehydration::DehydrationService;
use crate::services::retrieval::RetrievalService;
use crate::storage::surrealdb::SurrealPool;
use std::sync::Arc;

/// Application state containing all shared services and security components
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool
    pub db_pool: SurrealPool,
    /// Retrieval service for querying context
    pub retrieval_service: Arc<dyn RetrievalService>,
    /// Dehydration service for compressing context
    pub dehydration_service: Arc<dyn DehydrationService>,
    /// Authenticator for API key and JWT validation
    pub authenticator: Arc<dyn Authenticator>,
    /// Authorizer for RBAC permission checks
    pub authorizer: Arc<dyn Authorizer>,
    /// Rate limiter for request throttling
    pub rate_limiter: Arc<RateLimiter>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("db_pool", &"SurrealPool")
            .field("retrieval_service", &"Arc<dyn RetrievalService>")
            .field("dehydration_service", &"Arc<dyn DehydrationService>")
            .field("authenticator", &"Arc<dyn Authenticator>")
            .field("authorizer", &"Arc<dyn Authorizer>")
            .field("rate_limiter", &self.rate_limiter)
            .finish()
    }
}

impl AppState {
    /// Create new application state
    pub fn new(
        db_pool: SurrealPool,
        retrieval_service: Box<dyn RetrievalService>,
        dehydration_service: Box<dyn DehydrationService>,
        authenticator: Box<dyn Authenticator>,
        authorizer: Box<dyn Authorizer>,
        rate_limiter: RateLimiter,
    ) -> Self {
        Self {
            db_pool,
            retrieval_service: Arc::from(retrieval_service),
            dehydration_service: Arc::from(dehydration_service),
            authenticator: Arc::from(authenticator),
            authorizer: Arc::from(authorizer),
            rate_limiter: Arc::from(rate_limiter),
        }
    }

    /// Create development application state with default security components
    pub fn development(
        db_pool: SurrealPool,
        retrieval_service: Box<dyn RetrievalService>,
        dehydration_service: Box<dyn DehydrationService>,
    ) -> Self {
        use crate::security::auth::CombinedAuthenticator;
        use crate::security::rate_limit::RateLimiter;
        use crate::security::rbac::SimpleAuthorizer;

        let authenticator = Box::new(CombinedAuthenticator::development());
        let authorizer = Box::new(SimpleAuthorizer::development());
        let rate_limiter = RateLimiter::development();

        Self::new(
            db_pool,
            retrieval_service,
            dehydration_service,
            authenticator,
            authorizer,
            rate_limiter,
        )
    }
}
