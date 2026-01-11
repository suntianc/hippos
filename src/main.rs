use hippos::api::{self, app_state::AppState};
use hippos::config::loader::ConfigLoader;
use hippos::index::{create_embedding_model, create_unified_index_service};
use hippos::services::{create_dehydration_service, create_retrieval_service};
use hippos::storage::surrealdb::SurrealPool;
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

    let embedding_model_for_index =
        create_embedding_model(&config.embedding.model_name, config.vector.dimension).await?;
    info!(
        "Embedding model initialized: {}",
        config.embedding.model_name
    );

    let embedding_model_for_retrieval =
        create_embedding_model(&config.embedding.model_name, config.vector.dimension).await?;

    let _index_service = create_unified_index_service(
        hippos::index::create_vector_index(None, false),
        hippos::index::create_full_text_index(None, false),
        embedding_model_for_index,
    );
    info!("Index service initialized");

    let retrieval_service = create_retrieval_service(embedding_model_for_retrieval);
    info!("Retrieval service initialized");

    let dehydration_service = create_dehydration_service(100, 5, 10);
    info!("Dehydration service initialized");

    let app_state = AppState::new(
        db_pool,
        retrieval_service as Box<dyn hippos::services::retrieval::RetrievalService>,
        dehydration_service as Box<dyn hippos::services::dehydration::DehydrationService>,
        Box::new(hippos::security::auth::CombinedAuthenticator::development()),
        Box::new(hippos::security::rbac::SimpleAuthorizer::development()),
        hippos::security::rate_limit::RateLimiter::development(),
    );
    info!("Application state created");

    let router = api::create_router(app_state);
    info!("API router created");

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);

    axum::serve(listener, router).await?;

    Ok(())
}
