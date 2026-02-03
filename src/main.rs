use hippos::api::{self, app_state::AppState};
use hippos::config::loader::ConfigLoader;
use hippos::index::{create_embedding_model, create_unified_index_service};
use hippos::mcp::sse_server;
use hippos::models::entity_repository::EntityRepositoryImpl;
use hippos::models::memory_repository::MemoryRepositoryImpl;
use hippos::models::pattern_repository::PatternRepositoryImpl;
use hippos::models::profile_repository::ProfileRepositoryImpl;
use hippos::observability::{ObservabilityState, create_observability_router};
use hippos::services::{
    create_dehydration_service, create_retrieval_service, create_session_service,
    create_turn_service,
};
use hippos::storage::repository::{SessionRepository, TurnRepository};
use hippos::storage::surrealdb::SurrealPool;
use std::sync::Arc;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    // Check if we should run in MCP mode
    if std::env::var("HIPPOS_MCP_MODE").is_ok() {
        info!("Starting Hippos in MCP server mode...");

        // Check for SSE mode
        if std::env::var("HIPPOS_MCP_SSE").is_ok() {
            let port = std::env::var("HIPPOS_MCP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse::<u16>()
                .unwrap_or(8080);
            info!(
                "Starting MCP server with SSE transport in COMBINED mode on port {}...",
                port
            );
            return run_combined_server(port).await;
        }

        return hippos::mcp::run_mcp_server().await;
    }

    info!("Starting Hippos...");

    let config = ConfigLoader::load()?;
    info!("Configuration loaded successfully");

    let db_pool = SurrealPool::new(config.database.clone()).await?;
    info!("Database connection pool initialized");

    let session_repository_raw = SessionRepository::new(db_pool.clone());
    let turn_repository_raw = TurnRepository::new(db_pool.clone().inner().await, db_pool.clone());
    let memory_repository_raw = hippos::models::memory_repository::MemoryRepositoryImpl::new(db_pool.clone());
    let pattern_repository_raw = PatternRepositoryImpl::new(db_pool.clone());
    let entity_repository_raw = EntityRepositoryImpl::new(db_pool.clone());
    let profile_repository_raw = ProfileRepositoryImpl::new(db_pool.clone());
    let session_repository = Arc::new(session_repository_raw);
    let turn_repository = Arc::new(turn_repository_raw);
    let memory_repository = Arc::new(memory_repository_raw);
    let pattern_repository = Arc::new(pattern_repository_raw);
    let entity_repository = Arc::new(entity_repository_raw);
    let profile_repository = Arc::new(profile_repository_raw);
    info!("Repositories initialized");

    let embedding_model_for_index =
        create_embedding_model(&config.embedding, config.vector.dimension).await?;
    info!(
        "Embedding model initialized: {} (backend: {})",
        config.embedding.model_name, config.embedding.backend
    );

    let embedding_model_for_retrieval =
        create_embedding_model(&config.embedding, config.vector.dimension).await?;

    let index_service = create_unified_index_service(
        hippos::index::create_vector_index(None, false),
        hippos::index::create_full_text_index(None, false),
        embedding_model_for_index,
    );
    info!("Index service initialized");

    let retrieval_service =
        create_retrieval_service(embedding_model_for_retrieval, turn_repository.clone());
    info!("Retrieval service initialized");

    let dehydration_service = create_dehydration_service(100, 5, 10);
    info!("Dehydration service initialized");

    let session_service =
        create_session_service(session_repository.clone(), turn_repository.clone());
    info!("Session service initialized");

    let turn_service = create_turn_service(turn_repository.clone(), session_repository.clone());
    info!("Turn service initialized");

    let app_state = AppState::new(
        db_pool.clone(),
        (*session_repository).clone(),
        (*turn_repository).clone(),
        (*memory_repository).clone(),
        (*pattern_repository).clone(),
        (*entity_repository).clone(),
        (*profile_repository).clone(),
        session_service as Box<dyn hippos::services::session::SessionService>,
        turn_service as Box<dyn hippos::services::turn::TurnService>,
        retrieval_service as Box<dyn hippos::services::retrieval::RetrievalService>,
        dehydration_service as Box<dyn hippos::services::dehydration::DehydrationService>,
        index_service as Box<dyn hippos::index::IndexService>,
        Box::new(hippos::security::auth::CombinedAuthenticator::development()),
        Box::new(hippos::security::rbac::SimpleAuthorizer::development()),
        hippos::security::rate_limit::RateLimiter::development(),
    );
    info!("Application state created");

    // 创建可观测性状态并集成路由
    let observability_state = Arc::new(ObservabilityState::new("0.1.0".to_string()));
    let api_router = api::create_router(app_state);
    let router = create_observability_router(observability_state).merge(api_router);
    info!("API router created with observability endpoints");

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);

    axum::serve(listener, router).await?;

    Ok(())
}

/// Run the combined server with both REST API and SSE MCP endpoints
async fn run_combined_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    info!("Initializing combined REST API + SSE MCP server...");

    // Load configuration
    let config = ConfigLoader::load()?;
    info!("Configuration loaded successfully");

    let db_pool = SurrealPool::new(config.database.clone()).await?;
    info!("Database connection pool initialized");

    let session_repository_raw = SessionRepository::new(db_pool.clone());
    let turn_repository_raw = TurnRepository::new(db_pool.clone().inner().await, db_pool.clone());
    let memory_repository_raw = hippos::models::memory_repository::MemoryRepositoryImpl::new(db_pool.clone());
    let pattern_repository_raw = PatternRepositoryImpl::new(db_pool.clone());
    let entity_repository_raw = EntityRepositoryImpl::new(db_pool.clone());
    let profile_repository_raw = ProfileRepositoryImpl::new(db_pool.clone());
    let session_repository = Arc::new(session_repository_raw);
    let turn_repository = Arc::new(turn_repository_raw);
    let memory_repository = Arc::new(memory_repository_raw);
    let pattern_repository = Arc::new(pattern_repository_raw);
    let entity_repository = Arc::new(entity_repository_raw);
    let profile_repository = Arc::new(profile_repository_raw);
    info!("Repositories initialized");

    let embedding_model_for_index =
        create_embedding_model(&config.embedding, config.vector.dimension).await?;
    info!(
        "Embedding model initialized: {} (backend: {})",
        config.embedding.model_name, config.embedding.backend
    );

    let embedding_model_for_retrieval =
        create_embedding_model(&config.embedding, config.vector.dimension).await?;

    let index_service = create_unified_index_service(
        hippos::index::create_vector_index(None, false),
        hippos::index::create_full_text_index(None, false),
        embedding_model_for_index,
    );
    info!("Index service initialized");

    let retrieval_service =
        create_retrieval_service(embedding_model_for_retrieval, turn_repository.clone());
    info!("Retrieval service initialized");

    let dehydration_service = create_dehydration_service(100, 5, 10);
    info!("Dehydration service initialized");

    let session_service =
        create_session_service(session_repository.clone(), turn_repository.clone());
    info!("Session service initialized");

    let turn_service = create_turn_service(turn_repository.clone(), session_repository.clone());
    info!("Turn service initialized");

    // Create AppState with SSE ConnectionManager
    let mut app_state = AppState::new(
        db_pool.clone(),
        (*session_repository).clone(),
        (*turn_repository).clone(),
        (*memory_repository).clone(),
        (*pattern_repository).clone(),
        (*entity_repository).clone(),
        (*profile_repository).clone(),
        session_service as Box<dyn hippos::services::session::SessionService>,
        turn_service as Box<dyn hippos::services::turn::TurnService>,
        retrieval_service as Box<dyn hippos::services::retrieval::RetrievalService>,
        dehydration_service as Box<dyn hippos::services::dehydration::DehydrationService>,
        index_service as Box<dyn hippos::index::IndexService>,
        Box::new(hippos::security::auth::CombinedAuthenticator::development()),
        Box::new(hippos::security::rbac::SimpleAuthorizer::development()),
        hippos::security::rate_limit::RateLimiter::development(),
    );

    // Initialize SSE ConnectionManager
    app_state.init_sse_connection_manager(1000);
    info!("SSE ConnectionManager initialized");

    let app_state = Arc::new(app_state);
    info!("Application state created with SSE support");

    // 创建可观测性状态并集成路由
    let observability_state = Arc::new(ObservabilityState::new("0.1.0".to_string()));

    // Create SSE router
    let sse_router = sse_server::create_sse_router(app_state.clone());

    // Create main API router
    let api_router = api::create_router((*app_state).clone());

    // Merge all routers
    let router = create_observability_router(observability_state)
        .merge(api_router)
        .merge(sse_router);

    info!("Combined router created with REST API + SSE MCP endpoints");

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Combined server listening on {}", addr);

    axum::serve(listener, router).await?;

    Ok(())
}
