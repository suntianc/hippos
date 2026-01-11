//! MCP Server Implementation
//!
//! Provides the HipposMcpServer with hippos_search and hippos_semantic_search tools.

use crate::error::AppError;
use crate::services::RetrievalService;
use rmcp::{
    ServerHandler,
    handler::server::tool::Parameters,
    model::{
        CallToolResult, Content, ErrorData, Implementation, ProtocolVersion, ServerCapabilities,
        ServerInfo,
    },
    tool, tool_handler, tool_router,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error, info};

/// Search result item for MCP response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSearchResultItem {
    pub turn_id: String,
    pub gist: String,
    pub score: f32,
    pub result_type: String,
    pub turn_number: u64,
    pub timestamp: String,
    pub sources: Vec<String>,
}

/// Search response for MCP tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSearchResponse {
    pub results: Vec<McpSearchResultItem>,
    pub total_results: usize,
    pub took_ms: u64,
}

/// MCP Server implementation for Hippos search capabilities
#[derive(Clone)]
pub struct HipposMcpServer {
    retrieval_service: Arc<dyn RetrievalService>,
    tool_router: rmcp::handler::server::tool::ToolRouter<Self>,
}

impl HipposMcpServer {
    /// Create new HipposMcpServer instance
    pub fn new(retrieval_service: Arc<dyn RetrievalService>) -> Self {
        Self {
            retrieval_service,
            tool_router: Self::tool_router(),
        }
    }

    /// Convert SearchResult to McpSearchResultItem
    fn convert_search_result(result: crate::index::SearchResult) -> McpSearchResultItem {
        let result_type = match result.result_type {
            crate::index::SearchResultType::Semantic => "semantic".to_string(),
            crate::index::SearchResultType::FullText => "full_text".to_string(),
            crate::index::SearchResultType::Hybrid => "hybrid".to_string(),
        };

        McpSearchResultItem {
            turn_id: result.turn_id,
            gist: result.gist,
            score: result.score,
            result_type,
            turn_number: result.turn_number,
            timestamp: result.timestamp.to_rfc3339(),
            sources: result.sources,
        }
    }

    /// Convert SearchResult vector to McpSearchResponse
    fn create_search_response(
        results: Vec<crate::index::SearchResult>,
        took_ms: u64,
    ) -> McpSearchResponse {
        let results: Vec<McpSearchResultItem> = results
            .into_iter()
            .map(Self::convert_search_result)
            .collect();

        McpSearchResponse {
            results: results.clone(),
            total_results: results.len(),
            took_ms,
        }
    }

    /// Execute hybrid search and return formatted response
    async fn execute_hybrid_search(
        &self,
        session_id: String,
        query: String,
        limit: u32,
    ) -> Result<McpSearchResponse, AppError> {
        let start = std::time::Instant::now();
        debug!(
            "Executing hybrid search for session: {}, query: {}, limit: {}",
            session_id, query, limit
        );

        let results = self
            .retrieval_service
            .hybrid_search(&session_id, &query, limit)
            .await?;

        let took_ms = start.elapsed().as_millis() as u64;
        let response = Self::create_search_response(results, took_ms);

        info!(
            "Hybrid search completed: {} results in {}ms",
            response.total_results, response.took_ms
        );

        Ok(response)
    }

    /// Execute semantic search and return formatted response
    async fn execute_semantic_search(
        &self,
        session_id: String,
        query: String,
        limit: u32,
    ) -> Result<McpSearchResponse, AppError> {
        let start = std::time::Instant::now();
        debug!(
            "Executing semantic search for session: {}, query: {}, limit: {}",
            session_id, query, limit
        );

        let results = self
            .retrieval_service
            .semantic_search(&session_id, &query, limit)
            .await?;

        let took_ms = start.elapsed().as_millis() as u64;
        let response = Self::create_search_response(results, took_ms);

        info!(
            "Semantic search completed: {} results in {}ms",
            response.total_results, response.took_ms
        );

        Ok(response)
    }
}

/// Tool parameters for hippos_search
#[derive(Deserialize, JsonSchema)]
pub struct HipposSearchParams {
    pub session_id: String,
    pub query: String,
    pub limit: Option<u32>,
}

/// Tool parameters for hippos_semantic_search
#[derive(Deserialize, JsonSchema)]
pub struct HipposSemanticSearchParams {
    pub session_id: String,
    pub query: String,
    pub limit: Option<u32>,
}

impl From<AppError> for ErrorData {
    fn from(error: AppError) -> Self {
        match error {
            AppError::NotFound(msg) => ErrorData::resource_not_found(msg, None),
            AppError::Validation(msg) => ErrorData::invalid_params(msg, None),
            AppError::Authentication(msg) => ErrorData::invalid_request(msg, None),
            AppError::RateLimited => ErrorData::internal_error("Rate limit exceeded", None),
            _ => ErrorData::internal_error(error.to_string(), None),
        }
    }
}

#[tool_router]
impl HipposMcpServer {
    /// Get server information
    #[tool(description = "Get server information")]
    async fn server_info(&self) -> Result<CallToolResult, ErrorData> {
        let info = json!({
            "name": "hippos-search",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "Hippos Context Management Service - Search Only MCP Server"
        });

        let content = Content::json(info).map_err(|e| {
            ErrorData::internal_error(format!("Failed to create JSON content: {}", e), None)
        })?;
        Ok(CallToolResult::success(vec![content]))
    }

    /// Hybrid search combining semantic and keyword search
    #[tool(description = "Hybrid search combining semantic and keyword search")]
    async fn hippos_search(
        &self,
        params: Parameters<HipposSearchParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let hippos_search_params = params.0;

        // Validate inputs
        if hippos_search_params.session_id.trim().is_empty() {
            return Err(ErrorData::invalid_params(
                "session_id cannot be empty",
                None,
            ));
        }

        if hippos_search_params.query.trim().is_empty() {
            return Err(ErrorData::invalid_params("query cannot be empty", None));
        }

        let limit = hippos_search_params.limit.unwrap_or(10);

        // Execute search
        match self
            .execute_hybrid_search(
                hippos_search_params.session_id,
                hippos_search_params.query,
                limit,
            )
            .await
        {
            Ok(response) => {
                let content = Content::json(json!(response)).map_err(|e| {
                    ErrorData::internal_error(format!("Failed to create JSON content: {}", e), None)
                })?;
                Ok(CallToolResult::success(vec![content]))
            }
            Err(e) => {
                error!("Hippos search error: {}", e);
                Err(ErrorData::from(e))
            }
        }
    }

    /// Pure semantic search using vector similarity
    #[tool(description = "Pure semantic search using vector similarity")]
    async fn hippos_semantic_search(
        &self,
        params: Parameters<HipposSemanticSearchParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let hippos_search_params = params.0;

        // Validate inputs
        if hippos_search_params.session_id.trim().is_empty() {
            return Err(ErrorData::invalid_params(
                "session_id cannot be empty",
                None,
            ));
        }

        if hippos_search_params.query.trim().is_empty() {
            return Err(ErrorData::invalid_params("query cannot be empty", None));
        }

        let limit = hippos_search_params.limit.unwrap_or(10);

        // Execute search
        match self
            .execute_semantic_search(
                hippos_search_params.session_id,
                hippos_search_params.query,
                limit,
            )
            .await
        {
            Ok(response) => {
                let content = Content::json(json!(response)).map_err(|e| {
                    ErrorData::internal_error(format!("Failed to create JSON content: {}", e), None)
                })?;
                Ok(CallToolResult::success(vec![content]))
            }
            Err(e) => {
                error!("Hippos semantic search error: {}", e);
                Err(ErrorData::from(e))
            }
        }
    }
}

#[tool_handler]
impl ServerHandler for HipposMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "hippos-search".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                ..Default::default()
            },
            instructions: Some(
                "Hippos Context Management Service - Search Only MCP Server".to_string(),
            ),
        }
    }
}
