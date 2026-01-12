//! MCP Server Module
//!
//! Provides a simplified MCP (Model Context Protocol) server that exposes
//! search capabilities for the Hippos context management service.

pub mod server;

/// MCP Server Configuration
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Whether to enable MCP server mode
    pub enabled: bool,
    /// Server name for MCP identification
    pub name: String,
    /// Server version
    pub version: String,
}

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            name: "hippos-search".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Run the MCP server with stdio transport
pub async fn run_mcp_server() -> Result<(), Box<dyn std::error::Error>> {
    use crate::config::config::DatabaseConfig;
    use crate::index::create_embedding_model;
    use crate::services::retrieval::create_retrieval_service;
    use crate::storage::surrealdb::SurrealPool;
    use rmcp::{ServiceExt, transport::stdio};
    use server::HipposMcpServer;
    use tracing::info;

    info!("Initializing MCP server...");

    // Create services in correct order
    let db_pool = SurrealPool::new(DatabaseConfig::default()).await?;
    let turn_repository = std::sync::Arc::new(crate::storage::repository::TurnRepository::new(
        db_pool.inner().await,
    ));
    let embedding_config = crate::config::config::EmbeddingConfig {
        model_name: "all-MiniLM-L6-v2".into(),
        backend: "simple".into(),
        ..Default::default()
    };
    let embedding_model = create_embedding_model(&embedding_config, 384).await?;
    let retrieval_service = create_retrieval_service(embedding_model, turn_repository);

    // Create AppState-like structure for the MCP server
    let retrieval_service_arc = std::sync::Arc::from(retrieval_service);

    // Create and run the MCP server
    info!("MCP server starting with stdio transport...");

    let mcp_server = HipposMcpServer::new(retrieval_service_arc)
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("MCP server error: {}", e);
        })?;

    // Wait for server to complete
    mcp_server.waiting().await?;

    Ok(())
}
