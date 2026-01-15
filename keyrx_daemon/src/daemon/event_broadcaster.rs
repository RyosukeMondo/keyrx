//! Event broadcasting to WebSocket clients.
//!
//! This module provides functionality to broadcast daemon events (state changes,
//! key events, latency metrics) to connected WebSocket clients via the
//! subscription system.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::broadcast;
use tokio::time::interval;

use super::metrics::{LatencyRecorder, MetricsAggregator};
use crate::web::events::{DaemonEvent, DaemonState, KeyEventData, LatencyStats};

/// Broadcaster for daemon events to WebSocket clients
#[derive(Clone)]
pub struct EventBroadcaster {
    event_tx: broadcast::Sender<DaemonEvent>,
}

impl EventBroadcaster {
    /// Create a new event broadcaster
    pub fn new(event_tx: broadcast::Sender<DaemonEvent>) -> Self {
        Self { event_tx }
    }

    /// Broadcast a daemon state change
    ///
    /// This should be called whenever modifier, lock, or layer state changes.
    pub fn broadcast_state(&self, state: DaemonState) {
        if let Err(e) = self.event_tx.send(DaemonEvent::State(state)) {
            log::warn!("Failed to broadcast state event: {}", e);
        }
    }

    /// Broadcast a key event
    ///
    /// This should be called after each key event is processed.
    pub fn broadcast_key_event(&self, event: KeyEventData) {
        if let Err(e) = self.event_tx.send(DaemonEvent::KeyEvent(event)) {
            log::warn!("Failed to broadcast key event: {}", e);
        }
    }

    /// Broadcast latency statistics
    ///
    /// This should be called periodically (e.g., every 1 second) with current metrics.
    pub fn broadcast_latency(&self, stats: LatencyStats) {
        if let Err(e) = self.event_tx.send(DaemonEvent::Latency(stats)) {
            log::warn!("Failed to broadcast latency event: {}", e);
        }
    }

    /// Check if there are any subscribers
    ///
    /// This can be used to avoid expensive event creation when no clients are connected.
    pub fn has_subscribers(&self) -> bool {
        self.event_tx.receiver_count() > 0
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
            DaemonEvent::State(s) => {
                assert_eq!(s.modifiers, vec!["MD_00"]);
                assert_eq!(s.layer, "base");
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
            DaemonEvent::KeyEvent(e) => {
                assert_eq!(e.key_code, "KEY_A");
                assert_eq!(e.latency, 2300);
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
            DaemonEvent::Latency(s) => {
                assert_eq!(s.min, 1200);
                assert_eq!(s.avg, 2300);
                assert_eq!(s.p95, 3800);
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
            DaemonEvent::Latency(stats) => {
                assert!(stats.timestamp > 0);
            }
            _ => panic!("Expected Latency event"),
        }

        // Stop the task
        running.store(false, Ordering::SeqCst);
        let _ = task.await;
    }
}
