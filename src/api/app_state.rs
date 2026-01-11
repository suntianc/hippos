use crate::index::IndexService;
use crate::security::auth::Authenticator;
use crate::security::rate_limit::RateLimiter;
use crate::security::rbac::Authorizer;
use crate::services::dehydration::DehydrationService;
use crate::services::retrieval::RetrievalService;
use crate::services::session::SessionService;
use crate::services::turn::TurnService;
use crate::storage::repository::{SessionRepository, TurnRepository};
use crate::storage::surrealdb::SurrealPool;
use std::sync::Arc;

/// Application state containing all shared services and security components
#[derive(Clone)]
pub struct AppState {
    /// Database connection pool
    pub db_pool: SurrealPool,
    /// Session repository for session CRUD operations
    pub session_repository: Arc<SessionRepository>,
    /// Turn repository for turn CRUD operations
    pub turn_repository: Arc<TurnRepository>,
    /// Session service for session business logic
    pub session_service: Arc<dyn SessionService>,
    /// Turn service for turn business logic
    pub turn_service: Arc<dyn TurnService>,
    /// Retrieval service for querying context
    pub retrieval_service: Arc<dyn RetrievalService>,
    /// Dehydration service for compressing context
    pub dehydration_service: Arc<dyn DehydrationService>,
    /// Index service for search indexing
    pub index_service: Arc<dyn IndexService>,
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
            .field("session_repository", &"Arc<SessionRepository>")
            .field("turn_repository", &"Arc<TurnRepository>")
            .field("session_service", &"Arc<dyn SessionService>")
            .field("turn_service", &"Arc<dyn TurnService>")
            .field("retrieval_service", &"Arc<dyn RetrievalService>")
            .field("dehydration_service", &"Arc<dyn DehydrationService>")
            .field("index_service", &"Arc<dyn IndexService>")
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
        session_repository: SessionRepository,
        turn_repository: TurnRepository,
        session_service: Box<dyn SessionService>,
        turn_service: Box<dyn TurnService>,
        retrieval_service: Box<dyn RetrievalService>,
        dehydration_service: Box<dyn DehydrationService>,
        index_service: Box<dyn IndexService>,
        authenticator: Box<dyn Authenticator>,
        authorizer: Box<dyn Authorizer>,
        rate_limiter: RateLimiter,
    ) -> Self {
        Self {
            db_pool,
            session_repository: Arc::new(session_repository),
            turn_repository: Arc::new(turn_repository),
            session_service: Arc::from(session_service),
            turn_service: Arc::from(turn_service),
            retrieval_service: Arc::from(retrieval_service),
            dehydration_service: Arc::from(dehydration_service),
            index_service: Arc::from(index_service),
            authenticator: Arc::from(authenticator),
            authorizer: Arc::from(authorizer),
            rate_limiter: Arc::from(rate_limiter),
        }
    }

    /// Create development application state with default security components
    pub fn development(
        db_pool: SurrealPool,
        session_repository: SessionRepository,
        turn_repository: TurnRepository,
        session_service: Box<dyn SessionService>,
        turn_service: Box<dyn TurnService>,
        retrieval_service: Box<dyn RetrievalService>,
        dehydration_service: Box<dyn DehydrationService>,
        index_service: Box<dyn IndexService>,
    ) -> Self {
        use crate::security::auth::CombinedAuthenticator;
        use crate::security::rate_limit::RateLimiter;
        use crate::security::rbac::SimpleAuthorizer;

        let authenticator = Box::new(CombinedAuthenticator::development());
        let authorizer = Box::new(SimpleAuthorizer::development());
        let rate_limiter = RateLimiter::development();

        Self::new(
            db_pool,
            session_repository,
            turn_repository,
            session_service,
            turn_service,
            retrieval_service,
            dehydration_service,
            index_service,
            authenticator,
            authorizer,
            rate_limiter,
        )
    }
}
