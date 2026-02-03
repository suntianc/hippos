//! MCP Server Module
//!
//! Provides a simplified MCP (Model Context Protocol) server that exposes
//! search capabilities for the Hippos context management service.
//!
//! Supports stdio transport for local MCP clients and SSE transport
//! for remote MCP clients over HTTP.

pub mod server;
pub mod sse_server;

use crate::config::config::DatabaseConfig;
use crate::index::create_embedding_model;
use crate::services::retrieval::create_retrieval_service;
use crate::storage::repository::TurnRepository;
use crate::storage::surrealdb::SurrealPool;
use rmcp::{ServiceExt, transport::stdio};
use server::HipposMcpServer;
use std::sync::Arc;
use tracing::info;

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
    info!("Initializing MCP server...");

    let db_pool = SurrealPool::new(DatabaseConfig::default()).await?;
    let turn_repository = Arc::new(TurnRepository::new(db_pool.inner().await, db_pool.clone()));
    let embedding_config = crate::config::config::EmbeddingConfig {
        model_name: "all-MiniLM-L6-v2".into(),
        backend: "simple".into(),
        ..Default::default()
    };
    let embedding_model = create_embedding_model(&embedding_config, 384).await?;
    let retrieval_service = create_retrieval_service(embedding_model, turn_repository);
    let retrieval_service_arc = Arc::from(retrieval_service);

    info!("MCP server starting with stdio transport...");

    let mcp_server = HipposMcpServer::new(retrieval_service_arc)
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("MCP server error: {}", e);
        })?;

    mcp_server.waiting().await?;

    Ok(())
}
