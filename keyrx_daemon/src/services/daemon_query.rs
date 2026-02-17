//! Query service for daemon metrics and state.
//!
//! Provides REST-accessible metrics (latency, events, status) by bridging
//! the lock-free `LatencyRecorder` and `DaemonSharedState` to web handlers.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use serde_json::Value;

use crate::daemon::{DaemonSharedState, LatencyRecorder, LatencySnapshot, MetricsAggregator};

/// Maximum number of events stored in the ring buffer.
const EVENT_LOG_CAPACITY: usize = 1000;

/// Service for querying daemon metrics and state from REST endpoints.
///
/// This bridges the daemon's lock-free metrics infrastructure to the web
/// layer, eliminating platform-specific branching in REST handlers.
pub struct DaemonQueryService {
    latency_recorder: Arc<LatencyRecorder>,
    aggregator: Mutex<MetricsAggregator>,
    daemon_state: Arc<DaemonSharedState>,
    event_log: Arc<RwLock<VecDeque<Value>>>,
}

impl DaemonQueryService {
    /// Creates a new query service.
    pub fn new(
        latency_recorder: Arc<LatencyRecorder>,
        daemon_state: Arc<DaemonSharedState>,
    ) -> Self {
        Self {
            latency_recorder,
            aggregator: Mutex::new(MetricsAggregator::new(Duration::from_secs(60))),
            daemon_state,
            event_log: Arc::new(RwLock::new(VecDeque::with_capacity(EVENT_LOG_CAPACITY))),
        }
    }

    /// Computes a latency statistics snapshot from the recorder.
    pub fn get_latency_snapshot(&self) -> LatencySnapshot {
        let mut aggregator = self.aggregator.lock().expect("aggregator lock poisoned");
        aggregator.compute_snapshot(&self.latency_recorder)
    }

    /// Returns daemon status information.
    pub fn get_status(&self) -> StatusInfo {
        StatusInfo {
            daemon_running: self.daemon_state.is_running(),
            uptime_secs: self.daemon_state.uptime_secs(),
            active_profile: self.daemon_state.get_active_profile(),
            device_count: self.daemon_state.get_device_count(),
        }
    }

    /// Returns the most recent events from the ring buffer.
    pub fn get_event_log(&self, count: usize) -> Vec<Value> {
        let log = self.event_log.read().expect("event_log lock poisoned");
        log.iter().rev().take(count).cloned().collect()
    }

    /// Records an event into the ring buffer.
    pub fn record_event(&self, event: Value) {
        let mut log = self.event_log.write().expect("event_log lock poisoned");
        if log.len() >= EVENT_LOG_CAPACITY {
            log.pop_front();
        }
        log.push_back(event);
    }

    /// Spawns a background task that collects events from a broadcast channel.
    pub fn spawn_event_collector(
        self: &Arc<Self>,
        mut event_rx: tokio::sync::broadcast::Receiver<crate::web::rpc_types::ServerMessage>,
    ) {
        let this = Arc::clone(self);
        tokio::spawn(async move {
            loop {
                match event_rx.recv().await {
                    Ok(msg) => {
                        if let Ok(value) = serde_json::to_value(&msg) {
                            this.record_event(value);
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        log::warn!("Event collector lagged by {} messages", n);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        log::info!("Event collector channel closed");
                        break;
                    }
                }
            }
        });
    }
}

/// Status information returned by `get_status()`.
pub struct StatusInfo {
    pub daemon_running: bool,
    pub uptime_secs: u64,
    pub active_profile: Option<String>,
    pub device_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    fn make_test_service() -> DaemonQueryService {
        let recorder = Arc::new(LatencyRecorder::new());
        let running = Arc::new(AtomicBool::new(true));
        let state = Arc::new(DaemonSharedState::new(
            running,
            Some("test".to_string()),
            std::path::PathBuf::from("/test.krx"),
            1,
        ));
        DaemonQueryService::new(recorder, state)
    }

    #[test]
    fn test_get_latency_snapshot_empty() {
        let svc = make_test_service();
        let snap = svc.get_latency_snapshot();
        assert_eq!(snap.sample_count, 0);
    }

    #[test]
    fn test_get_latency_snapshot_with_data() {
        let svc = make_test_service();
        svc.latency_recorder.record(100);
        svc.latency_recorder.record(200);
        let snap = svc.get_latency_snapshot();
        assert_eq!(snap.sample_count, 2);
        assert!(snap.min_us >= 100);
    }

    #[test]
    fn test_get_status() {
        let svc = make_test_service();
        let status = svc.get_status();
        assert!(status.daemon_running);
        assert_eq!(status.active_profile, Some("test".to_string()));
        assert_eq!(status.device_count, 1);
    }

    #[test]
    fn test_event_log() {
        let svc = make_test_service();
        svc.record_event(serde_json::json!({"type": "key", "code": 65}));
        svc.record_event(serde_json::json!({"type": "key", "code": 66}));

        let events = svc.get_event_log(10);
        assert_eq!(events.len(), 2);
        // Most recent first
        assert_eq!(events[0]["code"], 66);
    }

    #[test]
    fn test_event_log_capacity() {
        let svc = make_test_service();
        for i in 0..1100 {
            svc.record_event(serde_json::json!({"i": i}));
        }
        let events = svc.get_event_log(2000);
        assert_eq!(events.len(), EVENT_LOG_CAPACITY);
    }
}
