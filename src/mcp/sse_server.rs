//! MCP SSE Server Module
//!
//! Custom Server-Sent Events (SSE) transport implementation for MCP protocol.
//! Supports both standalone mode and merged with regular REST API.

use crate::api::app_state::AppState;
use crate::config::config::DatabaseConfig;
use crate::index::create_embedding_model;
use crate::models::turn::TurnMetadata;
use crate::services::retrieval::{RetrievalService, create_retrieval_service};
use crate::services::session::SessionService;
use crate::services::turn::TurnService;
use crate::storage::repository::TurnRepository;
use crate::storage::surrealdb::SurrealPool;
use axum::{
    Json, Router,
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
};
use futures_util::stream::{self, StreamExt};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use tokio_stream::wrappers::{BroadcastStream, IntervalStream};
use tracing::{error, info};
use uuid::Uuid;

/// MCP Tool Configuration - Controls which tools are exposed
#[derive(Debug, Clone)]
pub struct McpToolConfig {
    // Session Management Tools
    pub enable_create_session: bool,
    pub enable_get_session: bool,
    pub enable_list_sessions: bool,
    pub enable_delete_session: bool,
    // Turn Management Tools
    pub enable_add_turn: bool,
    pub enable_list_turns: bool,
    pub enable_get_turn: bool,
    // Search Tools
    pub enable_search: bool,
    pub enable_semantic_search: bool,
}

impl Default for McpToolConfig {
    fn default() -> Self {
        Self {
            // Default: all tools enabled
            enable_create_session: true,
            enable_get_session: true,
            enable_list_sessions: true,
            enable_delete_session: true,
            enable_add_turn: true,
            enable_list_turns: true,
            enable_get_turn: true,
            enable_search: true,
            enable_semantic_search: true,
        }
    }
}

/// SSE Server Configuration
#[derive(Debug, Clone)]
pub struct SseServerConfig {
    pub name: String,
    pub version: String,
    pub sse_path: String,
    pub message_path: String,
    pub max_connections: usize,
    pub heartbeat_interval: u64,
    pub tools: McpToolConfig,
}

impl Default for SseServerConfig {
    fn default() -> Self {
        Self {
            name: "hippos-mcp-sse".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            sse_path: "/mcp".to_string(),
            message_path: "/mcp/message".to_string(),
            max_connections: 1000,
            heartbeat_interval: 30,
            tools: McpToolConfig::default(),
        }
    }
}

/// SSE Connection Manager
#[derive(Clone)]
pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, String>>>,
    count: Arc<AtomicUsize>,
    max_connections: usize,
    tx: broadcast::Sender<String>,
}

impl ConnectionManager {
    pub fn new(max_connections: usize) -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            count: Arc::new(AtomicUsize::new(0)),
            max_connections,
            tx,
        }
    }

    pub async fn add_connection(&self) -> Result<String, String> {
        if self.count.load(Ordering::SeqCst) >= self.max_connections {
            return Err("Maximum connections reached".to_string());
        }
        let conn_id = Uuid::new_v4().to_string();
        self.connections
            .write()
            .await
            .insert(conn_id.clone(), "connected".to_string());
        self.count.fetch_add(1, Ordering::SeqCst);
        let _ = self
            .tx
            .send(json!({ "event": "connected", "id": conn_id }).to_string());
        info!("New SSE connection: {}", conn_id);
        Ok(conn_id)
    }

    pub async fn remove_connection(&self, connection_id: &str) {
        if self
            .connections
            .write()
            .await
            .remove(connection_id)
            .is_some()
        {
            self.count.fetch_sub(1, Ordering::SeqCst);
            let _ = self
                .tx
                .send(json!({ "event": "disconnected", "id": connection_id }).to_string());
            info!("SSE connection removed: {}", connection_id);
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }
}

/// Server state for SSE MCP server (uses AppState)
#[derive(Clone)]
pub struct SseServerState {
    pub config: SseServerConfig,
    pub connection_manager: Arc<ConnectionManager>,
    pub retrieval_service: Arc<dyn RetrievalService>,
    pub session_service: Arc<dyn SessionService>,
    pub turn_service: Arc<dyn TurnService>,
}

impl From<(&AppState, &SseServerConfig)> for SseServerState {
    fn from((app_state, config): (&AppState, &SseServerConfig)) -> Self {
        Self {
            config: config.clone(),
            connection_manager: app_state.connection_manager.clone()
                .expect("ConnectionManager not initialized. Call app_state.init_sse_connection_manager() first."),
            retrieval_service: app_state.retrieval_service.clone(),
            session_service: app_state.session_service.clone(),
            turn_service: app_state.turn_service.clone(),
        }
    }
}

/// SSE event stream handler - uses AppState
async fn sse_handler_app_state(
    State(state): State<Arc<AppState>>,
) -> Sse<impl futures_util::stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let connection_manager = state
        .connection_manager
        .as_ref()
        .expect("ConnectionManager not initialized");

    let connection_id = connection_manager
        .add_connection()
        .await
        .unwrap_or_else(|_| "unknown".to_string());

    let rx = connection_manager.subscribe();
    let broadcast_stream = BroadcastStream::new(rx);

    let config = SseServerConfig::default();
    let heartbeat_interval = tokio::time::interval(Duration::from_secs(config.heartbeat_interval));
    let heartbeat_stream = IntervalStream::new(heartbeat_interval);

    let connection_id_clone = connection_id.clone();
    let manager = connection_manager.clone();

    // Create a stream that combines broadcast events and heartbeat
    let stream = stream::unfold(
        (
            broadcast_stream,
            heartbeat_stream,
            connection_id_clone,
            manager,
        ),
        |(mut broadcast_rx, mut heartbeat_rx, conn_id, conn_manager)| async move {
            tokio::select! {
                // Handle broadcast messages (like connected/disconnected events)
                biased;
                Some(msg_result) = broadcast_rx.next() => {
                    let msg = msg_result.unwrap_or_default();
                    Some((Ok(Event::default().data(msg)), (broadcast_rx, heartbeat_rx, conn_id, conn_manager)))
                }
                // Handle heartbeat tick
                Some(_) = heartbeat_rx.next() => {
                    let heartbeat = format!(
                        r#"{{"event":"heartbeat","timestamp":{}}}"#,
                        chrono::Utc::now().timestamp()
                    );
                    Some((Ok(Event::default().event("heartbeat").data(heartbeat)), (broadcast_rx, heartbeat_rx, conn_id, conn_manager)))
                }
            }
        },
    );

    // Spawn a task to send the initial connection event
    let tx_for_init = connection_manager.tx.clone();
    let init_event = json!({ "event": "connected", "id": connection_id }).to_string();
    tokio::spawn(async move {
        let _ = tx_for_init.send(init_event);
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// SSE event stream handler - standalone mode
async fn sse_handler(
    State(state): State<Arc<SseServerState>>,
) -> Sse<impl futures_util::stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let connection_id = state
        .connection_manager
        .add_connection()
        .await
        .unwrap_or_else(|_| "unknown".to_string());

    let rx = state.connection_manager.subscribe();
    let broadcast_stream = BroadcastStream::new(rx);

    let heartbeat_interval =
        tokio::time::interval(Duration::from_secs(state.config.heartbeat_interval));
    let heartbeat_stream = IntervalStream::new(heartbeat_interval);

    let connection_id_clone = connection_id.clone();
    let manager = state.connection_manager.clone();

    // Create a stream that combines broadcast events and heartbeat
    let stream = stream::unfold(
        (
            broadcast_stream,
            heartbeat_stream,
            connection_id_clone,
            manager,
        ),
        |(mut broadcast_rx, mut heartbeat_rx, conn_id, conn_manager)| async move {
            tokio::select! {
                // Handle broadcast messages (like connected/disconnected events)
                biased;
                Some(msg_result) = broadcast_rx.next() => {
                    let msg = msg_result.unwrap_or_default();
                    Some((Ok(Event::default().data(msg)), (broadcast_rx, heartbeat_rx, conn_id, conn_manager)))
                }
                // Handle heartbeat tick
                Some(_) = heartbeat_rx.next() => {
                    let heartbeat = format!(
                        r#"{{"event":"heartbeat","timestamp":{}}}"#,
                        chrono::Utc::now().timestamp()
                    );
                    Some((Ok(Event::default().event("heartbeat").data(heartbeat)), (broadcast_rx, heartbeat_rx, conn_id, conn_manager)))
                }
            }
        },
    );

    // Spawn a task to send the initial connection event
    let tx_for_init = state.connection_manager.tx.clone();
    let init_event = json!({ "event": "connected", "id": connection_id }).to_string();
    tokio::spawn(async move {
        let _ = tx_for_init.send(init_event);
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Message handler for MCP JSON-RPC requests (uses AppState)
async fn message_handler_app_state(
    State(state): State<Arc<AppState>>,
    Json(request): Json<Value>,
) -> (axum::http::StatusCode, Json<Value>) {
    let config = SseServerConfig::default();
    let response = process_mcp_request_with_app(&state, &config, request).await;
    let status = if response.get("type") == Some(&json!("error")) {
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    } else {
        axum::http::StatusCode::OK
    };
    (status, Json(response))
}

/// Message handler for MCP JSON-RPC requests (standalone mode)
async fn message_handler(
    State(state): State<Arc<SseServerState>>,
    Json(request): Json<Value>,
) -> (axum::http::StatusCode, Json<Value>) {
    let response = process_mcp_request(&state, request).await;
    let status = if response.get("type") == Some(&json!("error")) {
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    } else {
        axum::http::StatusCode::OK
    };
    (status, Json(response))
}

/// Build the tools list based on configuration
fn build_tools_list(config: &SseServerConfig) -> Vec<Value> {
    let mut tools = Vec::new();
    let tc = &config.tools;

    // Session Management Tools
    if tc.enable_create_session {
        tools.push(json!({
            "name": "hippos_create_session",
            "description": "Create a new session",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tenant_id": { "type": "string", "default": "dev-tenant" },
                    "name": { "type": "string" }
                },
                "required": ["name"]
            }
        }));
    }
    if tc.enable_get_session {
        tools.push(json!({
            "name": "hippos_get_session",
            "description": "Get session details by ID",
            "inputSchema": {
                "type": "object",
                "properties": { "session_id": { "type": "string" } },
                "required": ["session_id"]
            }
        }));
    }
    if tc.enable_list_sessions {
        tools.push(json!({
            "name": "hippos_list_sessions",
            "description": "List all sessions for a tenant",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tenant_id": { "type": "string", "default": "dev-tenant" },
                    "page": { "type": "integer", "default": 1 },
                    "page_size": { "type": "integer", "default": 20 }
                }
            }
        }));
    }
    if tc.enable_delete_session {
        tools.push(json!({
            "name": "hippos_delete_session",
            "description": "Delete a session by ID",
            "inputSchema": {
                "type": "object",
                "properties": { "session_id": { "type": "string" } },
                "required": ["session_id"]
            }
        }));
    }

    // Turn Management Tools
    if tc.enable_add_turn {
        tools.push(json!({
            "name": "hippos_add_turn",
            "description": "Add a turn to a session",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string" },
                    "content": { "type": "string" },
                    "role": { "type": "string", "enum": ["user", "assistant", "system"], "default": "user" }
                },
                "required": ["session_id", "content"]
            }
        }));
    }
    if tc.enable_list_turns {
        tools.push(json!({
            "name": "hippos_list_turns",
            "description": "List all turns in a session",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string" },
                    "page": { "type": "integer", "default": 1 },
                    "page_size": { "type": "integer", "default": 50 }
                },
                "required": ["session_id"]
            }
        }));
    }
    if tc.enable_get_turn {
        tools.push(json!({
            "name": "hippos_get_turn",
            "description": "Get a specific turn by ID",
            "inputSchema": {
                "type": "object",
                "properties": { "turn_id": { "type": "string" } },
                "required": ["turn_id"]
            }
        }));
    }

    // Search Tools
    if tc.enable_search {
        tools.push(json!({
            "name": "hippos_search",
            "description": "Hybrid search (semantic + keyword)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string" },
                    "query": { "type": "string" },
                    "limit": { "type": "integer", "default": 10 }
                },
                "required": ["session_id", "query"]
            }
        }));
    }
    if tc.enable_semantic_search {
        tools.push(json!({
            "name": "hippos_semantic_search",
            "description": "Semantic search only",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string" },
                    "query": { "type": "string" },
                    "limit": { "type": "integer", "default": 10 }
                },
                "required": ["session_id", "query"]
            }
        }));
    }

    tools
}

/// Check if a tool is enabled based on configuration
fn is_tool_enabled(config: &SseServerConfig, tool_name: &str) -> bool {
    let tc = &config.tools;
    match tool_name {
        "hippos_create_session" => tc.enable_create_session,
        "hippos_get_session" => tc.enable_get_session,
        "hippos_list_sessions" => tc.enable_list_sessions,
        "hippos_delete_session" => tc.enable_delete_session,
        "hippos_add_turn" => tc.enable_add_turn,
        "hippos_list_turns" => tc.enable_list_turns,
        "hippos_get_turn" => tc.enable_get_turn,
        "hippos_search" => tc.enable_search,
        "hippos_semantic_search" => tc.enable_semantic_search,
        _ => false,
    }
}

/// Process an MCP JSON-RPC request (uses AppState)
async fn process_mcp_request_with_app(
    state: &AppState,
    config: &SseServerConfig,
    request: Value,
) -> Value {
    let id = request.get("id").cloned().unwrap_or(json!(null));
    let method = request
        .get("method")
        .unwrap_or(&json!(null))
        .as_str()
        .unwrap_or("");

    match method {
        "initialize" => {
            json!({ "type": "result", "id": id, "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": config.name, "version": config.version }
            }})
        }
        "tools/list" => {
            json!({ "type": "result", "id": id, "result": {
                "tools": build_tools_list(config)
            }})
        }
        "tools/call" => {
            let params = request.get("params").cloned().unwrap_or(json!({}));
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            if tool_name.is_empty() {
                return json!({ "type": "error", "id": id, "error": { "code": -32600, "message": "Missing tool name" } });
            }

            // Check if tool is enabled
            if !is_tool_enabled(config, tool_name) {
                return json!({ "type": "error", "id": id, "error": {
                    "code": -32601,
                    "message": format!("Tool '{}' is not enabled", tool_name)
                }});
            }

            // Session Management Tools
            match tool_name {
                "hippos_create_session" => {
                    let tenant_id = arguments
                        .get("tenant_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("dev-tenant")
                        .to_string();
                    let name = arguments
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    if name.is_empty() {
                        return json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Missing name parameter" } });
                    }

                    match state.session_service.create(&tenant_id, &name).await {
                        Ok(session) => {
                            json!({ "type": "result", "id": id, "result": {
                                "id": session.id, "tenant_id": session.tenant_id, "name": session.name,
                                "created_at": session.created_at.to_rfc3339()
                            }})
                        }
                        Err(e) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Failed to create session: {}", e) } })
                        }
                    }
                }
                "hippos_get_session" => {
                    let session_id = arguments
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    if session_id.is_empty() {
                        return json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Missing session_id" } });
                    }

                    match state.session_service.get_by_id(&session_id).await {
                        Ok(Some(session)) => {
                            json!({ "type": "result", "id": id, "result": {
                                "id": session.id, "tenant_id": session.tenant_id, "name": session.name,
                                "description": session.description, "status": format!("{:?}", session.status),
                                "created_at": session.created_at.to_rfc3339()
                            }})
                        }
                        Ok(None) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Session not found" } })
                        }
                        Err(e) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Failed to get session: {}", e) } })
                        }
                    }
                }
                "hippos_list_sessions" => {
                    let tenant_id = arguments
                        .get("tenant_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("dev-tenant")
                        .to_string();

                    match state
                        .session_service
                        .list(&tenant_id, Default::default())
                        .await
                    {
                        Ok(sessions) => {
                            let results: Vec<_> = sessions
                                .iter()
                                .map(|s| {
                                    json!({
                                        "id": s.id, "tenant_id": s.tenant_id, "name": s.name,
                                        "created_at": s.created_at.to_rfc3339()
                                    })
                                })
                                .collect();
                            json!({ "type": "result", "id": id, "result": {
                                "sessions": results, "total": results.len()
                            }})
                        }
                        Err(e) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Failed to list sessions: {}", e) } })
                        }
                    }
                }
                "hippos_delete_session" => {
                    let session_id = arguments
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    if session_id.is_empty() {
                        return json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Missing session_id" } });
                    }

                    match state.session_service.delete(&session_id).await {
                        Ok(_) => {
                            json!({ "type": "result", "id": id, "result": { "message": "Session deleted" }})
                        }
                        Err(e) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Failed to delete session: {}", e) } })
                        }
                    }
                }
                // Turn Management Tools
                "hippos_add_turn" => {
                    let session_id = arguments
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let content = arguments
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let role = arguments
                        .get("role")
                        .and_then(|v| v.as_str())
                        .unwrap_or("user")
                        .to_string();

                    if session_id.is_empty() || content.is_empty() {
                        return json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Missing session_id or content" } });
                    }

                    let metadata = TurnMetadata {
                        role: Some(role),
                        ..Default::default()
                    };

                    match state
                        .turn_service
                        .create(&session_id, &content, Some(metadata))
                        .await
                    {
                        Ok(turn) => {
                            json!({ "type": "result", "id": id, "result": {
                                "id": turn.id, "session_id": turn.session_id, "turn_number": turn.turn_number,
                                "created_at": turn.metadata.timestamp.to_rfc3339()
                            }})
                        }
                        Err(e) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Failed to add turn: {}", e) } })
                        }
                    }
                }
                "hippos_list_turns" => {
                    let session_id = arguments
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    if session_id.is_empty() {
                        return json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Missing session_id" } });
                    }

                    match state
                        .turn_service
                        .list_by_session(&session_id, Default::default())
                        .await
                    {
                        Ok(turns) => {
                            let results: Vec<_> = turns.iter().map(|t| {
                                json!({
                                    "id": t.id, "session_id": t.session_id, "turn_number": t.turn_number,
                                    "content": t.raw_content, "created_at": t.metadata.timestamp.to_rfc3339()
                                })
                            }).collect();
                            json!({ "type": "result", "id": id, "result": {
                                "turns": results, "total": results.len()
                            }})
                        }
                        Err(e) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Failed to list turns: {}", e) } })
                        }
                    }
                }
                "hippos_get_turn" => {
                    let turn_id = arguments
                        .get("turn_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    if turn_id.is_empty() {
                        return json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Missing turn_id" } });
                    }

                    match state.turn_service.get_by_id(&turn_id).await {
                        Ok(Some(turn)) => {
                            json!({ "type": "result", "id": id, "result": {
                                "id": turn.id, "session_id": turn.session_id, "turn_number": turn.turn_number,
                                "content": turn.raw_content, "created_at": turn.metadata.timestamp.to_rfc3339()
                            }})
                        }
                        Ok(None) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Turn not found" } })
                        }
                        Err(e) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Failed to get turn: {}", e) } })
                        }
                    }
                }
                // Search Tools
                "hippos_search" | "hippos_semantic_search" => {
                    let session_id = arguments
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let query = arguments
                        .get("query")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let limit = arguments
                        .get("limit")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(10) as u32;

                    if session_id.is_empty() || query.is_empty() {
                        return json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Invalid params" } });
                    }

                    let is_semantic = tool_name == "hippos_semantic_search";
                    let search_result = if is_semantic {
                        state
                            .retrieval_service
                            .semantic_search(&session_id, &query, limit)
                            .await
                    } else {
                        state
                            .retrieval_service
                            .hybrid_search(&session_id, &query, limit)
                            .await
                    };

                    match search_result {
                        Ok(results) => {
                            let response_results: Vec<_> = results
                                .into_iter()
                                .map(|r| {
                                    json!({
                                        "turn_id": r.turn_id, "gist": r.gist, "score": r.score,
                                        "result_type": format!("{:?}", r.result_type).to_lowercase(),
                                        "turn_number": r.turn_number, "timestamp": r.timestamp.to_rfc3339(),
                                        "sources": r.sources
                                    })
                                })
                                .collect();
                            json!({ "type": "result", "id": id, "result": {
                                "results": response_results, "total_results": response_results.len()
                            }})
                        }
                        Err(e) => {
                            error!("Search error: {}", e);
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Search failed: {}", e) } })
                        }
                    }
                }
                _ => {
                    json!({ "type": "error", "id": id, "error": { "code": -32601, "message": format!("Unknown tool: {}", tool_name) } })
                }
            }
        }
        "ping" => json!({ "type": "result", "id": id, "result": {} }),
        _ => {
            json!({ "type": "error", "id": id, "error": { "code": -32601, "message": format!("Unknown method: {}", method) } })
        }
    }
}

/// Process an MCP JSON-RPC request (standalone mode)
async fn process_mcp_request(state: &SseServerState, request: Value) -> Value {
    let id = request.get("id").cloned().unwrap_or(json!(null));
    let method = request
        .get("method")
        .unwrap_or(&json!(null))
        .as_str()
        .unwrap_or("");

    match method {
        "initialize" => {
            json!({ "type": "result", "id": id, "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": state.config.name, "version": state.config.version }
            }})
        }
        "tools/list" => {
            json!({ "type": "result", "id": id, "result": {
                "tools": build_tools_list(&state.config)
            }})
        }
        "tools/call" => {
            let params = request.get("params").cloned().unwrap_or(json!({}));
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            if tool_name.is_empty() {
                return json!({ "type": "error", "id": id, "error": { "code": -32600, "message": "Missing tool name" } });
            }

            // Check if tool is enabled
            if !is_tool_enabled(&state.config, tool_name) {
                return json!({ "type": "error", "id": id, "error": {
                    "code": -32601,
                    "message": format!("Tool '{}' is not enabled", tool_name)
                }});
            }

            // Session Management Tools
            match tool_name {
                "hippos_create_session" => {
                    let tenant_id = arguments
                        .get("tenant_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("dev-tenant")
                        .to_string();
                    let name = arguments
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    if name.is_empty() {
                        return json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Missing name parameter" } });
                    }

                    match state.session_service.create(&tenant_id, &name).await {
                        Ok(session) => {
                            json!({ "type": "result", "id": id, "result": {
                                "id": session.id, "tenant_id": session.tenant_id, "name": session.name,
                                "created_at": session.created_at.to_rfc3339()
                            }})
                        }
                        Err(e) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Failed to create session: {}", e) } })
                        }
                    }
                }
                "hippos_get_session" => {
                    let session_id = arguments
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    if session_id.is_empty() {
                        return json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Missing session_id" } });
                    }

                    match state.session_service.get_by_id(&session_id).await {
                        Ok(Some(session)) => {
                            json!({ "type": "result", "id": id, "result": {
                                "id": session.id, "tenant_id": session.tenant_id, "name": session.name,
                                "description": session.description, "status": format!("{:?}", session.status),
                                "created_at": session.created_at.to_rfc3339()
                            }})
                        }
                        Ok(None) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Session not found" } })
                        }
                        Err(e) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Failed to get session: {}", e) } })
                        }
                    }
                }
                "hippos_list_sessions" => {
                    let tenant_id = arguments
                        .get("tenant_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("dev-tenant")
                        .to_string();

                    match state
                        .session_service
                        .list(&tenant_id, Default::default())
                        .await
                    {
                        Ok(sessions) => {
                            let results: Vec<_> = sessions
                                .iter()
                                .map(|s| {
                                    json!({
                                        "id": s.id, "tenant_id": s.tenant_id, "name": s.name,
                                        "created_at": s.created_at.to_rfc3339()
                                    })
                                })
                                .collect();
                            json!({ "type": "result", "id": id, "result": {
                                "sessions": results, "total": results.len()
                            }})
                        }
                        Err(e) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Failed to list sessions: {}", e) } })
                        }
                    }
                }
                "hippos_delete_session" => {
                    let session_id = arguments
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    if session_id.is_empty() {
                        return json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Missing session_id" } });
                    }

                    match state.session_service.delete(&session_id).await {
                        Ok(_) => {
                            json!({ "type": "result", "id": id, "result": { "message": "Session deleted" }})
                        }
                        Err(e) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Failed to delete session: {}", e) } })
                        }
                    }
                }
                // Turn Management Tools
                "hippos_add_turn" => {
                    let session_id = arguments
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let content = arguments
                        .get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let role = arguments
                        .get("role")
                        .and_then(|v| v.as_str())
                        .unwrap_or("user")
                        .to_string();

                    if session_id.is_empty() || content.is_empty() {
                        return json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Missing session_id or content" } });
                    }

                    let metadata = TurnMetadata {
                        role: Some(role),
                        ..Default::default()
                    };

                    match state
                        .turn_service
                        .create(&session_id, &content, Some(metadata))
                        .await
                    {
                        Ok(turn) => {
                            json!({ "type": "result", "id": id, "result": {
                                "id": turn.id, "session_id": turn.session_id, "turn_number": turn.turn_number,
                                "created_at": turn.metadata.timestamp.to_rfc3339()
                            }})
                        }
                        Err(e) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Failed to add turn: {}", e) } })
                        }
                    }
                }
                "hippos_list_turns" => {
                    let session_id = arguments
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    if session_id.is_empty() {
                        return json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Missing session_id" } });
                    }

                    match state
                        .turn_service
                        .list_by_session(&session_id, Default::default())
                        .await
                    {
                        Ok(turns) => {
                            let results: Vec<_> = turns.iter().map(|t| {
                                json!({
                                    "id": t.id, "session_id": t.session_id, "turn_number": t.turn_number,
                                    "content": t.raw_content, "created_at": t.metadata.timestamp.to_rfc3339()
                                })
                            }).collect();
                            json!({ "type": "result", "id": id, "result": {
                                "turns": results, "total": results.len()
                            }})
                        }
                        Err(e) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Failed to list turns: {}", e) } })
                        }
                    }
                }
                "hippos_get_turn" => {
                    let turn_id = arguments
                        .get("turn_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();

                    if turn_id.is_empty() {
                        return json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Missing turn_id" } });
                    }

                    match state.turn_service.get_by_id(&turn_id).await {
                        Ok(Some(turn)) => {
                            json!({ "type": "result", "id": id, "result": {
                                "id": turn.id, "session_id": turn.session_id, "turn_number": turn.turn_number,
                                "content": turn.raw_content, "created_at": turn.metadata.timestamp.to_rfc3339()
                            }})
                        }
                        Ok(None) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Turn not found" } })
                        }
                        Err(e) => {
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Failed to get turn: {}", e) } })
                        }
                    }
                }
                // Search Tools
                "hippos_search" | "hippos_semantic_search" => {
                    let session_id = arguments
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let query = arguments
                        .get("query")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let limit = arguments
                        .get("limit")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(10) as u32;

                    if session_id.is_empty() || query.is_empty() {
                        return json!({ "type": "error", "id": id, "error": { "code": -32602, "message": "Invalid params" } });
                    }

                    let is_semantic = tool_name == "hippos_semantic_search";
                    let search_result = if is_semantic {
                        state
                            .retrieval_service
                            .semantic_search(&session_id, &query, limit)
                            .await
                    } else {
                        state
                            .retrieval_service
                            .hybrid_search(&session_id, &query, limit)
                            .await
                    };

                    match search_result {
                        Ok(results) => {
                            let response_results: Vec<_> = results
                                .into_iter()
                                .map(|r| {
                                    json!({
                                        "turn_id": r.turn_id, "gist": r.gist, "score": r.score,
                                        "result_type": format!("{:?}", r.result_type).to_lowercase(),
                                        "turn_number": r.turn_number, "timestamp": r.timestamp.to_rfc3339(),
                                        "sources": r.sources
                                    })
                                })
                                .collect();
                            json!({ "type": "result", "id": id, "result": {
                                "results": response_results, "total_results": response_results.len()
                            }})
                        }
                        Err(e) => {
                            error!("Search error: {}", e);
                            json!({ "type": "error", "id": id, "error": { "code": -32603, "message": format!("Search failed: {}", e) } })
                        }
                    }
                }
                _ => {
                    json!({ "type": "error", "id": id, "error": { "code": -32601, "message": format!("Unknown tool: {}", tool_name) } })
                }
            }
        }
        "ping" => json!({ "type": "result", "id": id, "result": {} }),
        _ => {
            json!({ "type": "error", "id": id, "error": { "code": -32601, "message": format!("Unknown method: {}", method) } })
        }
    }
}

/// Create SSE server state
async fn create_sse_server_state(
    config: &SseServerConfig,
) -> Result<SseServerState, Box<dyn std::error::Error>> {
    let db_config = DatabaseConfig {
        url: std::env::var("EXOCORTEX_DATABASE_URL")
            .unwrap_or_else(|_| "ws://localhost:8000".to_string()),
        namespace: std::env::var("EXOCORTEX_DATABASE_NAMESPACE")
            .unwrap_or_else(|_| "hippos".to_string()),
        database: std::env::var("EXOCORTEX_DATABASE_NAME")
            .unwrap_or_else(|_| "sessions".to_string()),
        username: std::env::var("EXOCORTEX_DATABASE_USERNAME")
            .unwrap_or_else(|_| "root".to_string()),
        password: std::env::var("EXOCORTEX_DATABASE_PASSWORD")
            .unwrap_or_else(|_| "root".to_string()),
        ..Default::default()
    };
    let db_pool = SurrealPool::new(db_config).await?;
    let turn_repository = Arc::new(TurnRepository::new(db_pool.inner().await, db_pool.clone()));

    let embedding_config = crate::config::config::EmbeddingConfig {
        model_name: "all-MiniLM-L6-v2".into(),
        backend: "simple".into(),
        ..Default::default()
    };

    let embedding_model = create_embedding_model(&embedding_config, 384).await?;
    let retrieval_service = create_retrieval_service(embedding_model, turn_repository.clone());

    // For standalone mode, create session and turn service
    let session_repository = Arc::new(crate::storage::repository::SessionRepository::new(
        db_pool.clone(),
    ));
    let session_service: Arc<dyn SessionService> =
        Arc::new(crate::services::session::SessionServiceImpl::new(
            session_repository.clone(),
            turn_repository.clone(),
        ));
    let turn_service: Arc<dyn TurnService> = Arc::new(crate::services::turn::TurnServiceImpl::new(
        turn_repository,
        session_repository,
    ));

    Ok(SseServerState {
        config: config.clone(),
        connection_manager: Arc::new(ConnectionManager::new(config.max_connections)),
        retrieval_service: Arc::from(retrieval_service),
        session_service,
        turn_service,
    })
}

/// Create SSE router that can be merged with existing AppState
pub fn create_sse_router(app_state: Arc<AppState>) -> Router {
    let config = SseServerConfig::default();

    Router::new()
        .route(
            &format!("{}/sse", config.sse_path),
            get(sse_handler_app_state),
        )
        .route(&config.message_path, post(message_handler_app_state))
        .with_state(app_state)
}

/// Run the MCP SSE server (standalone mode)
pub async fn run_sse_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let config = SseServerConfig::default();
    let state = create_sse_server_state(&config).await?;
    let state = Arc::new(state);

    info!("Starting MCP SSE server on port {}...", port);

    let router = Router::new()
        .route(&format!("{}/sse", config.sse_path), get(sse_handler))
        .route(&config.message_path, post(message_handler))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("MCP SSE server listening on {}", addr);

    axum::serve(listener, router).await?;

    Ok(())
}
