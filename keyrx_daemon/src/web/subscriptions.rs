//! Subscription Channel Manager for WebSocket real-time updates.
//!
//! This module implements a publish-subscribe system for broadcasting events to WebSocket clients.
//! Clients can subscribe to channels like "daemon-state", "events", and "latency" to receive
//! real-time updates. The SubscriptionManager tracks which clients are subscribed to which channels
//! and handles broadcasting messages to all subscribed clients.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::web::rpc_types::ServerMessage;

/// Unique identifier for a WebSocket client connection
pub type ClientId = usize;

/// Manages pub/sub subscriptions for WebSocket channels
#[derive(Debug, Clone)]
pub struct SubscriptionManager {
    /// Map of channel names to sets of subscribed client IDs
    subscriptions: Arc<Mutex<HashMap<String, HashSet<ClientId>>>>,
    /// Counter for generating unique client IDs
    next_client_id: Arc<Mutex<ClientId>>,
}

impl SubscriptionManager {
    /// Create a new SubscriptionManager
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            next_client_id: Arc::new(Mutex::new(0)),
        }
    }

    /// Generate a new unique client ID
    pub async fn new_client_id(&self) -> ClientId {
        let mut counter = self.next_client_id.lock().await;
        let id = *counter;
        *counter += 1;
        id
    }

    /// Subscribe a client to a channel
    pub async fn subscribe(&self, client_id: ClientId, channel: &str) {
        log::debug!("Client {} subscribing to channel: {}", client_id, channel);

        let mut subs = self.subscriptions.lock().await;
        subs.entry(channel.to_string())
            .or_insert_with(HashSet::new)
            .insert(client_id);
    }

    /// Unsubscribe a client from a channel
    pub async fn unsubscribe(&self, client_id: ClientId, channel: &str) {
        log::debug!(
            "Client {} unsubscribing from channel: {}",
            client_id,
            channel
        );

        let mut subs = self.subscriptions.lock().await;
        if let Some(clients) = subs.get_mut(channel) {
            clients.remove(&client_id);

            // Clean up empty channel
            if clients.is_empty() {
                subs.remove(channel);
            }
        }
    }

    /// Unsubscribe a client from all channels (called on disconnect)
    pub async fn unsubscribe_all(&self, client_id: ClientId) {
        log::debug!("Client {} unsubscribing from all channels", client_id);

        let mut subs = self.subscriptions.lock().await;

        // Remove client from all channels
        let channels_to_remove: Vec<String> = subs
            .iter_mut()
            .filter_map(|(channel, clients)| {
                clients.remove(&client_id);
                if clients.is_empty() {
                    Some(channel.clone())
                } else {
                    None
                }
            })
            .collect();

        // Clean up empty channels
        for channel in channels_to_remove {
            subs.remove(&channel);
        }
    }

    /// Get all clients subscribed to a channel
    pub async fn get_subscribers(&self, channel: &str) -> Vec<ClientId> {
        let subs = self.subscriptions.lock().await;
        subs.get(channel)
            .map(|clients| clients.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Check if a client is subscribed to a channel
    pub async fn is_subscribed(&self, client_id: ClientId, channel: &str) -> bool {
        let subs = self.subscriptions.lock().await;
        subs.get(channel)
            .map(|clients| clients.contains(&client_id))
            .unwrap_or(false)
    }

    /// Get count of active subscriptions for a client
    pub async fn subscription_count(&self, client_id: ClientId) -> usize {
        let subs = self.subscriptions.lock().await;
        subs.values()
            .filter(|clients| clients.contains(&client_id))
            .count()
    }

    /// Broadcast a message to all clients subscribed to a channel
    ///
    /// Note: This returns a list of client IDs that should receive the message.
    /// The actual sending must be handled by the WebSocket connection manager,
    /// which has access to the individual client connections.
    pub async fn broadcast(&self, channel: &str, _message: ServerMessage) -> Vec<ClientId> {
        self.get_subscribers(channel).await
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_new_client_id() {
        let manager = SubscriptionManager::new();
        let id1 = manager.new_client_id().await;
        let id2 = manager.new_client_id().await;
        assert_ne!(id1, id2);
        assert_eq!(id2, id1 + 1);
    }

    #[tokio::test]
    async fn test_subscribe_and_get_subscribers() {
        let manager = SubscriptionManager::new();
        let client1 = 1;
        let client2 = 2;

        manager.subscribe(client1, "test-channel").await;
        manager.subscribe(client2, "test-channel").await;

        let subscribers = manager.get_subscribers("test-channel").await;
        assert_eq!(subscribers.len(), 2);
        assert!(subscribers.contains(&client1));
        assert!(subscribers.contains(&client2));
    }

    #[tokio::test]
    async fn test_unsubscribe() {
        let manager = SubscriptionManager::new();
        let client1 = 1;

        manager.subscribe(client1, "test-channel").await;
        assert!(manager.is_subscribed(client1, "test-channel").await);

        manager.unsubscribe(client1, "test-channel").await;
        assert!(!manager.is_subscribed(client1, "test-channel").await);
    }

    #[tokio::test]
    async fn test_unsubscribe_all() {
        let manager = SubscriptionManager::new();
        let client1 = 1;

        manager.subscribe(client1, "channel1").await;
        manager.subscribe(client1, "channel2").await;
        manager.subscribe(client1, "channel3").await;

        assert_eq!(manager.subscription_count(client1).await, 3);

        manager.unsubscribe_all(client1).await;

        assert_eq!(manager.subscription_count(client1).await, 0);
        assert!(!manager.is_subscribed(client1, "channel1").await);
        assert!(!manager.is_subscribed(client1, "channel2").await);
        assert!(!manager.is_subscribed(client1, "channel3").await);
    }

    #[tokio::test]
    async fn test_empty_channel_cleanup() {
        let manager = SubscriptionManager::new();
        let client1 = 1;

        manager.subscribe(client1, "test-channel").await;
        manager.unsubscribe(client1, "test-channel").await;

        // After unsubscribing the only client, channel should be removed
        let subscribers = manager.get_subscribers("test-channel").await;
        assert_eq!(subscribers.len(), 0);
    }

    #[tokio::test]
    async fn test_multiple_channels() {
        let manager = SubscriptionManager::new();
        let client1 = 1;
        let client2 = 2;

        manager.subscribe(client1, "daemon-state").await;
        manager.subscribe(client1, "events").await;
        manager.subscribe(client2, "latency").await;

        assert!(manager.is_subscribed(client1, "daemon-state").await);
        assert!(manager.is_subscribed(client1, "events").await);
        assert!(!manager.is_subscribed(client1, "latency").await);
        assert!(manager.is_subscribed(client2, "latency").await);
    }

    #[tokio::test]
    async fn test_broadcast_returns_subscribers() {
        let manager = SubscriptionManager::new();
        let client1 = 1;
        let client2 = 2;

        manager.subscribe(client1, "test-channel").await;
        manager.subscribe(client2, "test-channel").await;

        let message = ServerMessage::Event {
            channel: "test-channel".to_string(),
            data: json!({"test": "data"}),
        };

        let recipients = manager.broadcast("test-channel", message).await;
        assert_eq!(recipients.len(), 2);
        assert!(recipients.contains(&client1));
        assert!(recipients.contains(&client2));
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let manager = Arc::new(SubscriptionManager::new());
        let manager1 = Arc::clone(&manager);
        let manager2 = Arc::clone(&manager);

        let handle1 = tokio::spawn(async move {
            for i in 0..100 {
                manager1.subscribe(i, "channel1").await;
            }
        });

        let handle2 = tokio::spawn(async move {
            for i in 100..200 {
                manager2.subscribe(i, "channel2").await;
            }
        });

        handle1.await.expect("Task 1 failed");
        handle2.await.expect("Task 2 failed");

        assert_eq!(manager.get_subscribers("channel1").await.len(), 100);
        assert_eq!(manager.get_subscribers("channel2").await.len(), 100);
    }
}
