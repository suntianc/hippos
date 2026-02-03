//! WebSocket Handler Module
//!
//! Provides real-time subscriptions to memory updates via WebSocket protocol.
//! Supports topic-based subscriptions for memory, profile, pattern, and entity events.

use axum::{
    Extension,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use futures_util::stream::{SplitSink, SplitStream};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info};

use crate::api::app_state::AppState;

pub mod subscription;

/// WebSocket message types for subscription control
#[derive(Debug, Serialize, Deserialize)]
pub struct SubscriptionMessage {
    pub action: String,
    pub topics: Vec<String>,
}

/// WebSocket message types for broadcasting events
#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub r#type: String,
    pub topic: String,
    pub data: serde_json::Value,
}

/// WebSocket connection state with topic subscriptions
struct WebSocketConnection {
    id: String,
    subscriptions: HashSet<String>,
    sender: SplitSink<WebSocket, Message>,
}

type WebSocketStream = WebSocket;

impl WebSocketConnection {
    fn new(id: String, sender: SplitSink<WebSocketStream, Message>) -> Self {
        Self {
            id,
            subscriptions: HashSet::new(),
            sender,
        }
    }

    fn matches_topic(&self, topic: &str) -> bool {
        self.subscriptions.iter().any(|pattern| {
            if pattern.ends_with(":*") {
                let prefix = &pattern[..pattern.len() - 2];
                topic.starts_with(prefix)
                    && topic.len() > prefix.len()
                    && topic.as_bytes()[prefix.len()] == b':'
            } else {
                pattern == topic
            }
        })
    }

    fn subscribe(&mut self, topic: String) {
        self.subscriptions.insert(topic);
    }

    fn unsubscribe(&mut self, topic: &str) {
        self.subscriptions.remove(topic);
    }
}

/// WebSocket handler using Axum's WebSocket support
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(state): Extension<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handle the WebSocket connection
async fn handle_socket(ws: WebSocket, state: Arc<AppState>) {
    let (sender, receiver) = ws.split();
    let connection_id = uuid::Uuid::new_v4().to_string();

    info!("New WebSocket connection: {}", connection_id);

    let connection_manager = match state.connection_manager.as_ref() {
        Some(cm) => cm.clone(),
        None => {
            error!("Connection manager not initialized");
            return;
        }
    };

    let mut ws_connection = WebSocketConnection::new(connection_id.clone(), sender);

    let init_event = WebSocketMessage {
        r#type: "connected".to_string(),
        topic: "connection".to_string(),
        data: serde_json::json!({
            "id": connection_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
    };
    if let Err(e) = ws_connection
        .sender
        .send(Message::Text(serde_json::to_string(&init_event).unwrap()))
        .await
    {
        error!("Failed to send init event: {}", e);
        return;
    }

    let _ = connection_manager.add_connection().await;

    let mut rx = connection_manager.subscribe();

    let connection = Arc::new(tokio::sync::Mutex::new(ws_connection));

    let receive_conn = connection.clone();
    let forward_conn = connection.clone();

    // Clone connection_id before moving into tasks
    let connection_id_for_receive = connection_id.clone();
    let connection_id_for_forward = connection_id.clone();

    // Use join instead of spawn to avoid Send bound issues with parking_lot Mutex
    tokio::join! {
        handle_receive(receiver, connection_id_for_receive, receive_conn),
        handle_forward(rx, connection_id_for_forward, forward_conn)
    };

    connection_manager.remove_connection(&connection_id).await;
    debug!("WebSocket connection closed: {}", connection_id);
}

/// Handle incoming WebSocket messages
async fn handle_receive(
    mut receiver: SplitStream<WebSocket>,
    connection_id: String,
    connection: Arc<tokio::sync::Mutex<WebSocketConnection>>,
) {
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = process_message(&text, &connection_id, &connection).await {
                    error!("Failed to process message: {}", e);
                }
            }
            Ok(Message::Close(_)) => {
                debug!("Client initiated close for {}", connection_id);
                break;
            }
            Ok(_) => {}
            Err(e) => {
                error!("WebSocket error for {}: {}", connection_id, e);
                break;
            }
        }
    }
}

/// Process incoming subscription messages
async fn process_message(
    text: &str,
    connection_id: &str,
    connection: &Arc<tokio::sync::Mutex<WebSocketConnection>>,
) -> Result<(), String> {
    let msg: SubscriptionMessage =
        serde_json::from_str(text).map_err(|e| format!("Invalid message format: {}", e))?;

    let mut conn = connection.lock().await;

    match msg.action.as_str() {
        "subscribe" => {
            for topic in msg.topics.clone() {
                conn.subscribe(topic);
                debug!("{} subscribed to topic", connection_id);
            }
            // Release lock before awaiting
            let topics = msg.topics.clone();
            drop(conn);
            send_confirmation_async("subscribed", &topics, connection.clone()).await;
        }
        "unsubscribe" => {
            for topic in &msg.topics {
                conn.unsubscribe(topic);
                debug!("{} unsubscribed from topic: {}", connection_id, topic);
            }
            // Release lock before awaiting
            let topics = msg.topics.clone();
            drop(conn);
            send_confirmation_async("unsubscribed", &topics, connection.clone()).await;
        }
        "ping" => {
            let pong = WebSocketMessage {
                r#type: "pong".to_string(),
                topic: "heartbeat".to_string(),
                data: serde_json::json!({
                    "timestamp": chrono::Utc::now().to_rfc3339()
                }),
            };
            let text = serde_json::to_string(&pong).map_err(|e| format!("JSON error: {}", e))?;
            // Release lock before awaiting
            drop(conn);
            send_message_to_connection(text, connection.clone()).await;
        }
        _ => {
            let error_msg = WebSocketMessage {
                r#type: "error".to_string(),
                topic: "message".to_string(),
                data: serde_json::json!({
                    "message": format!("Unknown action: {}", msg.action)
                }),
            };
            let text =
                serde_json::to_string(&error_msg).map_err(|e| format!("JSON error: {}", e))?;
            // Release lock before awaiting
            drop(conn);
            send_message_to_connection(text, connection.clone()).await;
        }
    }

    Ok(())
}

/// Send a message to the WebSocket connection (releases lock first)
async fn send_message_to_connection(
    text: String,
    connection: Arc<tokio::sync::Mutex<WebSocketConnection>>,
) {
    if let Err(e) = connection
        .lock()
        .await
        .sender
        .send(Message::Text(text))
        .await
    {
        error!("Failed to send message: {}", e);
    }
}

/// Send confirmation asynchronously (releases lock first)
async fn send_confirmation_async(
    action: &str,
    topics: &[String],
    connection: Arc<tokio::sync::Mutex<WebSocketConnection>>,
) {
    let confirmation = WebSocketMessage {
        r#type: action.to_string(),
        topic: "subscription".to_string(),
        data: serde_json::json!({
            "topics": topics,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
    };

    if let Err(e) = connection
        .lock()
        .await
        .sender
        .send(Message::Text(serde_json::to_string(&confirmation).unwrap()))
        .await
    {
        error!("Failed to send confirmation: {}", e);
    }
}

/// Send confirmation message
async fn send_confirmation(
    conn: &mut tokio::sync::MutexGuard<'_, WebSocketConnection>,
    action: &str,
    topics: &[String],
) {
    let confirmation = WebSocketMessage {
        r#type: action.to_string(),
        topic: "subscription".to_string(),
        data: serde_json::json!({
            "topics": topics,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }),
    };

    if let Err(e) = conn
        .sender
        .send(Message::Text(serde_json::to_string(&confirmation).unwrap()))
        .await
    {
        error!("Failed to send confirmation: {}", e);
    }
}

/// Forward broadcast events to WebSocket connections
async fn handle_forward(
    mut rx: broadcast::Receiver<String>,
    connection_id: String,
    connection: Arc<tokio::sync::Mutex<WebSocketConnection>>,
) {
    while let Ok(event_str) = rx.recv().await {
        let event: serde_json::Value = match serde_json::from_str(&event_str) {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to parse event: {}", e);
                continue;
            }
        };

        let topic = event
            .get("event")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let should_forward = {
            let conn = connection.lock().await;
            conn.matches_topic(topic)
        };

        if should_forward {
            let message = WebSocketMessage {
                r#type: "event".to_string(),
                topic: topic.to_string(),
                data: event,
            };

            // Acquire lock, send message, and release lock within the same await point
            if let Err(e) = connection.lock().await
                .sender
                .send(Message::Text(serde_json::to_string(&message).unwrap()))
                .await
            {
                error!("Failed to forward event to {}: {}", connection_id, e);
                break;
            }
        }
    }
}

/// Topics that can be subscribed to
pub mod topics {
    pub const MEMORY_CREATED: &str = "memory:created";
    pub const MEMORY_UPDATED: &str = "memory:updated";
    pub const MEMORY_DELETED: &str = "memory:deleted";
    pub const MEMORY_ALL: &str = "memory:*";
    pub const PROFILE_UPDATED: &str = "profile:updated";
    pub const PATTERN_CREATED: &str = "pattern:created";
    pub const ENTITY_CREATED: &str = "entity:created";
    pub const ALL_EVENTS: &str = "*";
}
