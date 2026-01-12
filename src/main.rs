use hippos::api::{self, app_state::AppState};
use hippos::config::loader::ConfigLoader;
use hippos::index::{create_embedding_model, create_unified_index_service};
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
        return hippos::mcp::run_mcp_server().await;
    }

    info!("Starting Hippos...");

    let config = ConfigLoader::load()?;
    info!("Configuration loaded successfully");

    let db_pool = SurrealPool::new(config.database.clone()).await?;
    info!("Database connection pool initialized");

    let session_repository_raw = SessionRepository::new(db_pool.clone().inner().await);
    let turn_repository_raw = TurnRepository::new(db_pool.clone().inner().await);
    let session_repository = Arc::new(session_repository_raw);
    let turn_repository = Arc::new(turn_repository_raw);
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
