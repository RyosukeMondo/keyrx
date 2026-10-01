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
use crate::web::events::{DaemonState, KeyEventData, LatencyStats};

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
            input_overflows: self.daemon_state.input_overflow_count(),
            config_error: self.daemon_state.get_config_error(),
        }
    }

    /// The keyboards this daemon has captured right now. The one place that
    /// answers "what is the daemon managing" for the IPC `devices list`
    /// (REST/MCP/WS reach the same set through `DeviceService`, which reads
    /// `DaemonSharedState::is_device_active`).
    pub fn get_captured_devices(&self) -> Vec<crate::ipc::CapturedDevice> {
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        {
            let mut devices: Vec<_> = crate::device_manager::enumerate_keyboards()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|kb| {
                    let id = kb.device_id();
                    self.daemon_state
                        .is_device_active(&id)
                        .then(|| crate::ipc::CapturedDevice {
                            id,
                            name: kb.name.clone(),
                            path: kb.path.display().to_string(),
                        })
                })
                .collect();
            devices.sort_by(|a, b| a.id.cmp(&b.id));
            devices
        }
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        {
            Vec::new()
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

    /// Current latency statistics in wire form (REST, WS-RPC, MCP, WS push).
    pub fn get_latency_stats(&self) -> LatencyStats {
        LatencyStats::from_snapshot(&self.telemetry.latency())
    }

    /// Current modifier/lock/layer state plus the loaded profile, in wire form.
    pub fn get_daemon_state(&self) -> DaemonState {
        DaemonState::from_telemetry(
            &self.telemetry.state(),
            self.daemon_state.get_active_profile(),
        )
    }

    /// Up to `count` most recent key events, oldest first.
    pub fn get_recent_events(&self, count: usize) -> Vec<KeyEventData> {
        self.telemetry.recent_events(count)
    }

    /// Clears the recent-events history. Returns the number of events removed.
    pub fn clear_events(&self) -> usize {
        self.telemetry.clear_events()
    }

    /// Asks the daemon to switch to the (already compiled) profile `name`.
    ///
    /// Like [`Self::request_profile_activation`] but waits until the daemon
    /// is actually running `name`. A query service without a running event
    /// loop (test mode, unit tests) reports an error after the timeout.
    ///
    /// # Errors
    ///
    /// Why the daemon did not apply the profile.
    pub fn activate_and_wait(&self, name: &str) -> Result<(), String> {
        self.daemon_state
            .activate_and_wait(name, std::time::Duration::from_secs(5))
    }

    /// Every activation path (REST, MCP, WS-RPC, IPC) calls this. Status
    /// reports `name` once the daemon has actually loaded it.
    pub fn request_profile_activation(&self, name: &str) {
        self.daemon_state.request_activation(name);
    }
}

/// Status information returned by [`DaemonQueryService::get_status`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusInfo {
    pub daemon_running: bool,
    pub uptime_secs: u64,
    pub active_profile: Option<String>,
    pub device_count: usize,
    /// Kernel input-buffer overflows (`SYN_DROPPED`) the daemon recovered from.
    pub input_overflows: u64,
    /// Why the requested configuration is not live (the daemon then grabs
    /// no keyboard, or keeps the previous config); `None` when it is live.
    pub config_error: Option<String>,
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
                input_overflows: 0,
                config_error: None,
            }
        );
    }

    #[test]
    fn test_get_status_reports_config_error() {
        let (svc, _) = make_test_service();
        svc.shared_state().set_config_error(Some("bad".to_string()));
        assert_eq!(svc.get_status().config_error.as_deref(), Some("bad"));
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
        telemetry.push_event(KeyEventData::test_press("A"));
        telemetry.push_event(KeyEventData::test_press("B"));
        let inputs: Vec<_> = svc
            .get_recent_events(10)
            .into_iter()
            .map(|e| e.input)
            .collect();
        assert_eq!(inputs, vec!["A", "B"]);
        assert_eq!(svc.clear_events(), 2);
        assert!(telemetry.recent_events(10).is_empty());
    }

    /// Status reports what the daemon loaded, not what was requested: the
    /// activation is a request the daemon takes when it services the reload.
    #[test]
    fn test_request_profile_activation_is_a_request() {
        let (svc, _) = make_test_service();
        svc.request_profile_activation("gaming");
        assert_eq!(svc.get_status().active_profile.as_deref(), Some("test"));
        let state = svc.shared_state();
        assert!(state.take_reload_request());
        assert_eq!(state.take_pending_activation().as_deref(), Some("gaming"));
        assert_eq!(state.take_pending_activation(), None);
    }
}
