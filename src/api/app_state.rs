use crate::index::IndexService;
use crate::mcp::sse_server::ConnectionManager;
use crate::models::entity_repository::EntityRepositoryImpl;
use crate::models::memory_repository::MemoryRepositoryImpl;
use crate::models::pattern_repository::PatternRepositoryImpl;
use crate::models::profile_repository::ProfileRepositoryImpl;
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
    /// Memory repository for memory CRUD operations
    pub memory_repository: Arc<MemoryRepositoryImpl>,
    /// Pattern repository for pattern CRUD operations
    pub pattern_repository: Arc<PatternRepositoryImpl>,
    /// Entity repository for entity and relationship CRUD operations
    pub entity_repository: Arc<EntityRepositoryImpl>,
    /// Profile repository for profile CRUD operations
    pub profile_repository: Arc<ProfileRepositoryImpl>,
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
    /// Connection manager for SSE MCP server
    pub connection_manager: Option<Arc<ConnectionManager>>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("db_pool", &"SurrealPool")
            .field("session_repository", &"Arc<SessionRepository>")
            .field("turn_repository", &"Arc<TurnRepository>")
            .field("memory_repository", &"Arc<MemoryRepository>")
            .field("pattern_repository", &"Arc<PatternRepositoryImpl>")
            .field("entity_repository", &"Arc<EntityRepositoryImpl>")
            .field("profile_repository", &"Arc<ProfileRepositoryImpl>")
            .field("session_service", &"Arc<dyn SessionService>")
            .field("turn_service", &"Arc<dyn TurnService>")
            .field("retrieval_service", &"Arc<dyn RetrievalService>")
            .field("dehydration_service", &"Arc<dyn DehydrationService>")
            .field("index_service", &"Arc<dyn IndexService>")
            .field("authenticator", &"Arc<dyn Authenticator>")
            .field("authorizer", &"Arc<dyn Authorizer>")
            .field("rate_limiter", &self.rate_limiter)
            .field(
                "connection_manager",
                &self
                    .connection_manager
                    .as_ref()
                    .map(|_| "Some(ConnectionManager)"),
            )
            .finish()
    }
}

impl AppState {
    pub fn new(
        db_pool: SurrealPool,
        session_repository: SessionRepository,
        turn_repository: TurnRepository,
        memory_repository: MemoryRepositoryImpl,
        pattern_repository: PatternRepositoryImpl,
        entity_repository: EntityRepositoryImpl,
        profile_repository: ProfileRepositoryImpl,
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
            memory_repository: Arc::new(memory_repository),
            pattern_repository: Arc::new(pattern_repository),
            entity_repository: Arc::new(entity_repository),
            profile_repository: Arc::new(profile_repository),
            session_service: Arc::from(session_service),
            turn_service: Arc::from(turn_service),
            retrieval_service: Arc::from(retrieval_service),
            dehydration_service: Arc::from(dehydration_service),
            index_service: Arc::from(index_service),
            authenticator: Arc::from(authenticator),
            authorizer: Arc::from(authorizer),
            rate_limiter: Arc::from(rate_limiter),
            connection_manager: None,
        }
    }

    pub fn init_sse_connection_manager(&mut self, max_connections: usize) {
        self.connection_manager = Some(Arc::new(ConnectionManager::new(max_connections)));
    }

    pub fn development(
        db_pool: SurrealPool,
        session_repository: SessionRepository,
        turn_repository: TurnRepository,
        memory_repository: MemoryRepositoryImpl,
        pattern_repository: PatternRepositoryImpl,
        entity_repository: EntityRepositoryImpl,
        profile_repository: ProfileRepositoryImpl,
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
            memory_repository,
            pattern_repository,
            entity_repository,
            profile_repository,
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
