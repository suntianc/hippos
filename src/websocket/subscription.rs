//! Subscription Management Module
//!
//! Provides topic-based subscription management for WebSocket connections.
//! Supports wildcard subscriptions (e.g., "memory:*" matches "memory:created")

use std::collections::HashSet;

/// Topic pattern matching result
#[derive(Debug, Clone, PartialEq)]
pub enum MatchResult {
    Exact(String),
    Wildcard(String),
    None,
}

/// Subscription topic with wildcard support
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct SubscriptionTopic {
    topic: String,
    is_wildcard: bool,
    prefix: String,
}

impl SubscriptionTopic {
    /// Create a new subscription topic
    pub fn new(topic: &str) -> Self {
        let is_wildcard = topic.ends_with(":*");
        let prefix = if is_wildcard {
            topic[..topic.len() - 2].to_string()
        } else {
            topic.to_string()
        };

        Self {
            topic: topic.to_string(),
            is_wildcard,
            prefix,
        }
    }

    /// Get the original topic string
    pub fn as_str(&self) -> &str {
        &self.topic
    }

    /// Check if this is a wildcard subscription
    pub fn is_wildcard(&self) -> bool {
        self.is_wildcard
    }

    /// Check if this subscription matches a given topic
    pub fn matches(&self, topic: &str) -> bool {
        if self.is_wildcard {
            topic.starts_with(&self.prefix)
                && topic.len() > self.prefix.len()
                && topic.as_bytes()[self.prefix.len()] == b':'
        } else {
            &self.topic == topic
        }
    }
}

/// Subscription manager for a single WebSocket connection
#[derive(Debug)]
pub struct ConnectionSubscriptionManager {
    connection_id: String,
    subscriptions: HashSet<SubscriptionTopic>,
}

impl ConnectionSubscriptionManager {
    /// Create a new subscription manager for a connection
    pub fn new(connection_id: &str) -> Self {
        Self {
            connection_id: connection_id.to_string(),
            subscriptions: HashSet::new(),
        }
    }

    /// Get the connection ID
    pub fn connection_id(&self) -> &str {
        &self.connection_id
    }

    /// Add a subscription
    pub fn subscribe(&mut self, topic: &str) -> bool {
        let subscription = SubscriptionTopic::new(topic);
        self.subscriptions.insert(subscription)
    }

    /// Remove a subscription
    pub fn unsubscribe(&mut self, topic: &str) -> bool {
        let subscription = SubscriptionTopic::new(topic);
        self.subscriptions.remove(&subscription)
    }

    /// Check if the connection is subscribed to a topic
    pub fn is_subscribed(&self, topic: &str) -> bool {
        self.subscriptions.iter().any(|sub| sub.matches(topic))
    }

    /// Get all matching subscriptions for a topic
    pub fn get_matches(&self, topic: &str) -> Vec<&SubscriptionTopic> {
        self.subscriptions
            .iter()
            .filter(|sub| sub.matches(topic))
            .collect()
    }

    /// Get all subscription topics
    pub fn get_topics(&self) -> Vec<&str> {
        self.subscriptions.iter().map(|t| t.as_str()).collect()
    }

    /// Get the count of active subscriptions
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.len()
    }

    /// Clear all subscriptions
    pub fn clear(&mut self) {
        self.subscriptions.clear();
    }

    /// Check if there are any subscriptions
    pub fn is_empty(&self) -> bool {
        self.subscriptions.is_empty()
    }
}

/// Global subscription registry (for multi-connection scenarios)
#[derive(Debug, Default)]
pub struct GlobalSubscriptionRegistry {
    /// Map of topic to connection IDs
    topic_subscriptions: HashMap<String, HashSet<String>>,
    /// Map of connection ID to subscriptions
    connection_subscriptions: HashMap<String, ConnectionSubscriptionManager>,
}

impl GlobalSubscriptionRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self {
            topic_subscriptions: HashMap::new(),
            connection_subscriptions: HashMap::new(),
        }
    }

    /// Register a new connection
    pub fn register_connection(&mut self, connection_id: &str) {
        self.connection_subscriptions
            .entry(connection_id.to_string())
            .or_insert_with(|| ConnectionSubscriptionManager::new(connection_id));
    }

    /// Unregister a connection and clean up its subscriptions
    pub fn unregister_connection(&mut self, connection_id: &str) {
        if let Some(manager) = self.connection_subscriptions.remove(connection_id) {
            for topic in manager.get_topics() {
                if let Some(connections) = self.topic_subscriptions.get_mut(topic) {
                    connections.remove(connection_id);
                    if connections.is_empty() {
                        self.topic_subscriptions.remove(topic);
                    }
                }
            }
        }
    }

    /// Subscribe a connection to a topic
    pub fn subscribe(&mut self, connection_id: &str, topic: &str) {
        let manager = self
            .connection_subscriptions
            .entry(connection_id.to_string())
            .or_insert_with(|| ConnectionSubscriptionManager::new(connection_id));

        manager.subscribe(topic);

        self.topic_subscriptions
            .entry(topic.to_string())
            .or_insert_with(HashSet::new)
            .insert(connection_id.to_string());
    }

    /// Unsubscribe a connection from a topic
    pub fn unsubscribe(&mut self, connection_id: &str, topic: &str) {
        if let Some(manager) = self.connection_subscriptions.get_mut(connection_id) {
            manager.unsubscribe(topic);
        }

        if let Some(connections) = self.topic_subscriptions.get_mut(topic) {
            connections.remove(connection_id);
            if connections.is_empty() {
                self.topic_subscriptions.remove(topic);
            }
        }
    }

    /// Get all connections subscribed to a topic
    pub fn get_subscribers(&self, topic: &str) -> Vec<&str> {
        if let Some(connections) = self.topic_subscriptions.get(topic) {
            return connections.iter().map(|s| s.as_str()).collect();
        }

        let mut subscribers = Vec::new();
        for (subscribed_topic, connections) in &self.topic_subscriptions {
            let sub = SubscriptionTopic::new(subscribed_topic);
            if sub.matches(topic) {
                for conn_id in connections {
                    if !subscribers.contains(&conn_id.as_str()) {
                        subscribers.push(conn_id.as_str());
                    }
                }
            }
        }

        subscribers
    }

    /// Get connection subscription manager
    pub fn get_connection_manager(
        &self,
        connection_id: &str,
    ) -> Option<&ConnectionSubscriptionManager> {
        self.connection_subscriptions.get(connection_id)
    }

    /// Get total number of connections
    pub fn connection_count(&self) -> usize {
        self.connection_subscriptions.len()
    }

    /// Get total number of subscriptions
    pub fn subscription_count(&self) -> usize {
        self.topic_subscriptions.values().map(|s| s.len()).sum()
    }
}

use std::collections::HashMap;
