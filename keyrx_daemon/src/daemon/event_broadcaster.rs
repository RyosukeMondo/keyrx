//! Event broadcasting to WebSocket clients.
//!
//! This module provides functionality to broadcast daemon events (state changes,
//! key events, latency metrics) to connected WebSocket clients via the
//! subscription system.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use tokio::time::interval;

use super::metrics::{LatencyRecorder, MetricsAggregator};
use crate::web::events::{DaemonEvent, DaemonState, ErrorData, KeyEventData, LatencyStats};

/// Global sequence number for message ordering (WS-004)
static SEQUENCE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Get next sequence number
fn next_sequence() -> u64 {
    SEQUENCE_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Ring buffer for delivered message IDs per subscriber (WS-005)
const DELIVERED_BUFFER_SIZE: usize = 1000;

/// Tracks delivered message IDs for a subscriber
#[derive(Default)]
struct DeliveredMessages {
    ring_buffer: VecDeque<u64>,
}

impl DeliveredMessages {
    fn new() -> Self {
        Self {
            ring_buffer: VecDeque::with_capacity(DELIVERED_BUFFER_SIZE),
        }
    }

    /// Check if message was already delivered
    fn contains(&self, seq: u64) -> bool {
        self.ring_buffer.contains(&seq)
    }

    /// Mark message as delivered
    fn insert(&mut self, seq: u64) {
        if self.ring_buffer.len() >= DELIVERED_BUFFER_SIZE {
            self.ring_buffer.pop_front();
        }
        self.ring_buffer.push_back(seq);
    }
}

/// Broadcaster for daemon events to WebSocket clients
#[derive(Clone)]
pub struct EventBroadcaster {
    event_tx: broadcast::Sender<DaemonEvent>,
    /// WS-003: Track delivered messages per subscriber for deduplication
    delivered_messages: Arc<RwLock<std::collections::HashMap<String, DeliveredMessages>>>,
}

impl EventBroadcaster {
    /// Create a new event broadcaster
    pub fn new(event_tx: broadcast::Sender<DaemonEvent>) -> Self {
        Self {
            event_tx,
            delivered_messages: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Broadcast a daemon state change
    ///
    /// This should be called whenever modifier, lock, or layer state changes.
    pub fn broadcast_state(&self, state: DaemonState) {
        let sequence = next_sequence();
        if let Err(e) = self.event_tx.send(DaemonEvent::State {
            data: state,
            sequence,
        }) {
            log::warn!("Failed to broadcast state event: {}", e);
        }
    }

    /// Broadcast a key event
    ///
    /// This should be called after each key event is processed.
    pub fn broadcast_key_event(&self, event: KeyEventData) {
        let subscribers = self.event_tx.receiver_count();
        log::debug!(
            "Broadcasting key event (subscribers: {}): {:?}",
            subscribers,
            event.key_code
        );

        let sequence = next_sequence();
        match self.event_tx.send(DaemonEvent::KeyEvent {
            data: event,
            sequence,
        }) {
            Ok(receiver_count) => {
                log::debug!(
                    "Successfully broadcast key event to {} receivers",
                    receiver_count
                );
            }
            Err(e) => {
                log::warn!("Failed to broadcast key event (no subscribers?): {}", e);
            }
        }
    }

    /// Broadcast latency statistics
    ///
    /// This should be called periodically (e.g., every 1 second) with current metrics.
    pub fn broadcast_latency(&self, stats: LatencyStats) {
        let sequence = next_sequence();
        if let Err(e) = self.event_tx.send(DaemonEvent::Latency {
            data: stats,
            sequence,
        }) {
            log::warn!("Failed to broadcast latency event: {}", e);
        }
    }

    /// Broadcast an error notification (WS-005)
    ///
    /// This should be called when errors occur that clients should be notified about.
    pub fn broadcast_error(&self, code: String, message: String, context: Option<String>) {
        let sequence = next_sequence();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);

        let error_data = ErrorData {
            code,
            message,
            context,
            timestamp,
        };

        if let Err(e) = self.event_tx.send(DaemonEvent::Error {
            data: error_data,
            sequence,
        }) {
            log::warn!("Failed to broadcast error event: {}", e);
        }
    }

    /// Check if there are any subscribers
    ///
    /// This can be used to avoid expensive event creation when no clients are connected.
    pub fn has_subscribers(&self) -> bool {
        self.event_tx.receiver_count() > 0
    }

    /// WS-005: Check if message was already delivered to subscriber
    pub fn was_delivered(&self, subscriber_id: &str, sequence: u64) -> bool {
        let delivered = self.delivered_messages.read().unwrap();
        delivered
            .get(subscriber_id)
            .map(|d| d.contains(sequence))
            .unwrap_or(false)
    }

    /// WS-005: Mark message as delivered to subscriber
    pub fn mark_delivered(&self, subscriber_id: &str, sequence: u64) {
        let mut delivered = self.delivered_messages.write().unwrap();
        delivered
            .entry(subscriber_id.to_string())
            .or_insert_with(DeliveredMessages::new)
            .insert(sequence);
    }

    /// WS-003: Subscribe a new client
    pub fn subscribe_client(&self, client_id: &str) {
        let mut delivered = self.delivered_messages.write().unwrap();
        delivered.insert(client_id.to_string(), DeliveredMessages::new());
    }

    /// WS-003: Unsubscribe a client
    pub fn unsubscribe_client(&self, client_id: &str) {
        let mut delivered = self.delivered_messages.write().unwrap();
        delivered.remove(client_id);
    }
}

/// Start a background task that periodically broadcasts latency metrics
///
/// This task runs every 1 second and broadcasts latency statistics to all
/// connected WebSocket clients. The task continues until the provided
/// running flag is set to false.
///
/// # Arguments
///
/// * `broadcaster` - The event broadcaster to use
/// * `running` - Atomic flag that controls task lifetime
/// * `latency_recorder` - Optional lock-free latency recorder for real metrics
///
/// # Performance
///
/// Percentile calculation uses HdrHistogram, which is O(log n) for queries
/// but is done off the hot path (every 1 second in this background task).
/// The latency recording itself is lock-free O(1) on the hot path.
///
/// # Example
///
/// ```ignore
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use std::sync::Arc;
///
/// let running = Arc::new(AtomicBool::new(true));
/// let broadcaster = EventBroadcaster::new(event_tx);
/// let latency_recorder = Arc::new(LatencyRecorder::new());
///
/// tokio::spawn(start_latency_broadcast_task(
///     broadcaster,
///     Arc::clone(&running),
///     Some(Arc::clone(&latency_recorder)),
/// ));
///
/// // Later...
/// running.store(false, Ordering::SeqCst);
/// ```
pub async fn start_latency_broadcast_task(
    broadcaster: EventBroadcaster,
    running: Arc<std::sync::atomic::AtomicBool>,
    latency_recorder: Option<Arc<LatencyRecorder>>,
) {
    use std::sync::atomic::Ordering;

    log::info!("Starting latency broadcast task (1 second interval)");

    // Create metrics aggregator for percentile computation
    // This runs in the background task, NOT on the hot path
    let mut aggregator = MetricsAggregator::new(Duration::from_secs(60));

    let mut ticker = interval(Duration::from_secs(1));

    // Skip the first tick (immediate)
    ticker.tick().await;

    while running.load(Ordering::SeqCst) {
        ticker.tick().await;

        // Only broadcast if there are subscribers
        if !broadcaster.has_subscribers() {
            continue;
        }

        // Compute real latency statistics if recorder is available
        let stats = if let Some(ref recorder) = latency_recorder {
            let snapshot = aggregator.compute_snapshot(recorder);

            LatencyStats {
                min: snapshot.min_us,
                avg: snapshot.avg_us,
                max: snapshot.max_us,
                p95: snapshot.p95_us,
                p99: snapshot.p99_us,
                timestamp: snapshot.timestamp_us,
            }
        } else {
            // Fallback to placeholder zeros if no recorder
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_micros() as u64)
                .unwrap_or(0);

            LatencyStats {
                min: 0,
                avg: 0,
                max: 0,
                p95: 0,
                p99: 0,
                timestamp,
            }
        };

        broadcaster.broadcast_latency(stats);
    }

    log::info!("Latency broadcast task stopped");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;

    #[test]
    fn test_event_broadcaster_new() {
        let (event_tx, _) = broadcast::channel(100);
        let _broadcaster = EventBroadcaster::new(event_tx);
    }

    #[tokio::test]
    async fn test_broadcast_state() {
        let (event_tx, mut event_rx) = broadcast::channel(100);
        let broadcaster = EventBroadcaster::new(event_tx);

        let state = DaemonState {
            modifiers: vec!["MD_00".to_string()],
            locks: vec![],
            layer: "base".to_string(),
            active_profile: None,
        };

        broadcaster.broadcast_state(state.clone());

        let received = event_rx.recv().await.unwrap();
        match received {
            DaemonEvent::State { data, sequence } => {
                assert_eq!(data.modifiers, vec!["MD_00"]);
                assert_eq!(data.layer, "base");
                assert!(sequence > 0);
            }
            _ => panic!("Expected State event"),
        }
    }

    #[tokio::test]
    async fn test_broadcast_key_event() {
        let (event_tx, mut event_rx) = broadcast::channel(100);
        let broadcaster = EventBroadcaster::new(event_tx);

        let event = KeyEventData {
            timestamp: 1234567890,
            key_code: "KEY_A".to_string(),
            event_type: "press".to_string(),
            input: "A".to_string(),
            output: "B".to_string(),
            latency: 2300,
            device_id: Some("dev-001".to_string()),
            device_name: Some("USB Keyboard".to_string()),
            mapping_type: Some("simple".to_string()),
            mapping_triggered: true,
        };

        broadcaster.broadcast_key_event(event.clone());

        let received = event_rx.recv().await.unwrap();
        match received {
            DaemonEvent::KeyEvent { data, sequence } => {
                assert_eq!(data.key_code, "KEY_A");
                assert_eq!(data.latency, 2300);
                assert!(sequence > 0);
            }
            _ => panic!("Expected KeyEvent event"),
        }
    }

    #[tokio::test]
    async fn test_broadcast_latency() {
        let (event_tx, mut event_rx) = broadcast::channel(100);
        let broadcaster = EventBroadcaster::new(event_tx);

        let stats = LatencyStats {
            min: 1200,
            avg: 2300,
            max: 4500,
            p95: 3800,
            p99: 4200,
            timestamp: 1234567890,
        };

        broadcaster.broadcast_latency(stats.clone());

        let received = event_rx.recv().await.unwrap();
        match received {
            DaemonEvent::Latency { data, sequence } => {
                assert_eq!(data.min, 1200);
                assert_eq!(data.avg, 2300);
                assert_eq!(data.p95, 3800);
                assert!(sequence > 0);
            }
            _ => panic!("Expected Latency event"),
        }
    }

    #[test]
    fn test_has_subscribers() {
        let (event_tx, _rx1) = broadcast::channel(100);
        let broadcaster = EventBroadcaster::new(event_tx.clone());

        // With one receiver
        assert!(broadcaster.has_subscribers());

        // Subscribe another
        let _rx2 = event_tx.subscribe();
        assert!(broadcaster.has_subscribers());
    }

    #[tokio::test]
    async fn test_latency_broadcast_task_stops_when_running_false() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use tokio::time::timeout;

        let (event_tx, _) = broadcast::channel(100);
        let broadcaster = EventBroadcaster::new(event_tx);
        let running = Arc::new(AtomicBool::new(true));

        let task_running = Arc::clone(&running);
        let task = tokio::spawn(async move {
            start_latency_broadcast_task(broadcaster, task_running, None).await;
        });

        // Let it run for a bit
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Stop the task
        running.store(false, Ordering::SeqCst);

        // Task should complete quickly
        let result = timeout(Duration::from_secs(2), task).await;
        assert!(result.is_ok(), "Task should complete when running=false");
    }

    #[tokio::test]
    async fn test_latency_broadcast_task_sends_events() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let (event_tx, mut event_rx) = broadcast::channel(100);
        let broadcaster = EventBroadcaster::new(event_tx);
        let running = Arc::new(AtomicBool::new(true));

        let task_running = Arc::clone(&running);
        let task = tokio::spawn(async move {
            start_latency_broadcast_task(broadcaster, task_running, None).await;
        });

        // Wait for at least one broadcast (happens every 1 second)
        tokio::time::sleep(Duration::from_millis(1100)).await;

        // Should have received at least one event
        let result = tokio::time::timeout(Duration::from_millis(100), event_rx.recv()).await;
        assert!(result.is_ok(), "Should receive latency event");

        match result.unwrap().unwrap() {
            DaemonEvent::Latency { data, sequence } => {
                assert!(data.timestamp > 0);
                assert!(sequence > 0);
            }
            _ => panic!("Expected Latency event"),
        }

        // Stop the task
        running.store(false, Ordering::SeqCst);
        let _ = task.await;
    }
}
