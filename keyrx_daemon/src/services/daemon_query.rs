//! The daemon's single read model.
//!
//! Every transport that reports on the running daemon — the IPC server behind
//! `keyrx_daemon status|state|metrics`, the REST API, MCP tools and the
//! WebSocket latency feed — reads through one [`DaemonQueryService`]. It is a
//! thin view over the two write-side sources:
//!
//! | Data                          | Source                 | Written by            |
//! |-------------------------------|------------------------|-----------------------|
//! | running / uptime / profile / devices | [`DaemonSharedState`] | daemon + runners  |
//! | modifier/lock/layer state     | [`DaemonTelemetry`]    | event loop            |
//! | latency statistics            | [`DaemonTelemetry`]    | event loop (recorder) |
//! | recent events                 | [`DaemonTelemetry`]    | event loop            |
//!
//! Because there is exactly one instance per process and no transport keeps a
//! copy or a fallback, two transports cannot disagree about the same fact.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::daemon::{DaemonSharedState, DaemonTelemetry, LatencySnapshot, TelemetryState};

/// Read model for daemon status, live state, latency and recent events.
pub struct DaemonQueryService {
    daemon_state: Arc<DaemonSharedState>,
    telemetry: Arc<DaemonTelemetry>,
}

impl DaemonQueryService {
    /// Creates the read model over a running daemon's state and telemetry.
    pub fn new(daemon_state: Arc<DaemonSharedState>, telemetry: Arc<DaemonTelemetry>) -> Self {
        Self {
            daemon_state,
            telemetry,
        }
    }

    /// Creates a read model for a process with no keyboard daemon (test mode,
    /// unit tests): not running, no profile, no devices, empty telemetry.
    pub fn without_daemon() -> Self {
        let state =
            DaemonSharedState::new(Arc::new(AtomicBool::new(false)), None, PathBuf::new(), 0);
        Self::new(Arc::new(state), Arc::new(DaemonTelemetry::new()))
    }

    /// The shared daemon state this read model reports on.
    pub fn shared_state(&self) -> &Arc<DaemonSharedState> {
        &self.daemon_state
    }

    /// Daemon status (running, uptime, active profile, device count).
    pub fn get_status(&self) -> StatusInfo {
        StatusInfo {
            daemon_running: self.daemon_state.is_running(),
            uptime_secs: self.daemon_state.uptime_secs(),
            active_profile: self.daemon_state.get_active_profile(),
            device_count: self.daemon_state.get_device_count(),
        }
    }

    /// Current modifier/lock/layer state.
    pub fn get_state(&self) -> TelemetryState {
        self.telemetry.state()
    }

    /// Current latency statistics.
    pub fn get_latency_snapshot(&self) -> LatencySnapshot {
        self.telemetry.latency()
    }

    /// Up to `count` most recent event descriptions, oldest first.
    pub fn get_recent_events(&self, count: usize) -> Vec<String> {
        self.telemetry.recent_events(count)
    }

    /// Clears the recent-events history. Returns the number of events removed.
    pub fn clear_events(&self) -> usize {
        self.telemetry.clear_events()
    }

    /// Records that `name` was activated and asks the daemon to reload it.
    ///
    /// Every activation path (REST, IPC) must call this so status reflects the
    /// activation regardless of which transport performed it.
    pub fn record_profile_activation(&self, name: &str) {
        self.daemon_state.set_active_profile(Some(name.to_string()));
        self.daemon_state.request_reload();
    }
}

/// Status information returned by [`DaemonQueryService::get_status`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusInfo {
    pub daemon_running: bool,
    pub uptime_secs: u64,
    pub active_profile: Option<String>,
    pub device_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_service() -> (DaemonQueryService, Arc<DaemonTelemetry>) {
        let telemetry = Arc::new(DaemonTelemetry::new());
        let state = Arc::new(DaemonSharedState::new(
            Arc::new(AtomicBool::new(true)),
            Some("test".to_string()),
            PathBuf::from("/test.krx"),
            2,
        ));
        (
            DaemonQueryService::new(state, Arc::clone(&telemetry)),
            telemetry,
        )
    }

    #[test]
    fn test_get_status_reads_shared_state() {
        let (svc, _) = make_test_service();
        assert_eq!(
            svc.get_status(),
            StatusInfo {
                daemon_running: true,
                uptime_secs: 0,
                active_profile: Some("test".to_string()),
                device_count: 2,
            }
        );
    }

    #[test]
    fn test_without_daemon_reports_not_running() {
        let status = DaemonQueryService::without_daemon().get_status();
        assert!(!status.daemon_running);
        assert_eq!(status.active_profile, None);
        assert_eq!(status.device_count, 0);
    }

    #[test]
    fn test_get_state_reads_telemetry() {
        let (svc, telemetry) = make_test_service();
        let mut s = TelemetryState::empty();
        s.set_modifier(4, true);
        telemetry.update_state(s);
        assert_eq!(svc.get_state().modifiers(), vec!["MD_04"]);
    }

    #[test]
    fn test_latency_reads_telemetry_recorder() {
        let (svc, telemetry) = make_test_service();
        telemetry.latency_recorder().record(100);
        telemetry.latency_recorder().record(200);
        let snap = svc.get_latency_snapshot();
        assert_eq!(snap.sample_count, 2);
        assert!(snap.min_us >= 100);
    }

    #[test]
    fn test_events_are_the_telemetry_ring() {
        let (svc, telemetry) = make_test_service();
        telemetry.push_event("press A".to_string());
        telemetry.push_event("release A".to_string());
        assert_eq!(svc.get_recent_events(10), vec!["press A", "release A"]);
        assert_eq!(svc.clear_events(), 2);
        assert!(telemetry.recent_events(10).is_empty());
    }

    #[test]
    fn test_record_profile_activation_updates_status() {
        let (svc, _) = make_test_service();
        svc.record_profile_activation("gaming");
        assert_eq!(svc.get_status().active_profile.as_deref(), Some("gaming"));
    }
}
