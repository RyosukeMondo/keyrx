//! IPC command handlers.
//!
//! A thin adapter from [`IpcRequest`] to the daemon's single read model,
//! [`DaemonQueryService`]. The handler keeps no state of its own, so what the
//! `keyrx_daemon status|state|metrics` CLI reports is by construction what the
//! REST API reports for the same daemon.

use super::{IpcRequest, IpcResponse};
use crate::config::profile_manager::ProfileManager;
use crate::services::DaemonQueryService;
use std::sync::Arc;

/// Handler for IPC commands (production and test mode alike).
pub struct IpcCommandHandler {
    profile_manager: Arc<ProfileManager>,
    query: Arc<DaemonQueryService>,
}

impl IpcCommandHandler {
    /// Creates a handler over the process's read model.
    ///
    /// * `profile_manager` - performs `ActivateProfile`
    /// * `query` - the same [`DaemonQueryService`] the web API uses
    pub fn new(profile_manager: Arc<ProfileManager>, query: Arc<DaemonQueryService>) -> Self {
        Self {
            profile_manager,
            query,
        }
    }

    /// Handles an IPC request and returns the response.
    pub fn handle(&self, request: IpcRequest) -> IpcResponse {
        match request {
            IpcRequest::ActivateProfile { name } => self.handle_activate_profile(name),
            IpcRequest::GetStatus => self.handle_get_status(),
            IpcRequest::GetState => IpcResponse::State {
                state: self.query.get_state().into_raw(),
            },
            IpcRequest::GetLatencyMetrics => self.handle_get_latency(),
            IpcRequest::GetEventsTail { count } => IpcResponse::Events {
                events: self.query.get_recent_events(count),
            },
            IpcRequest::ClearEvents => IpcResponse::EventsCleared {
                count: self.query.clear_events(),
            },
        }
    }

    fn handle_get_status(&self) -> IpcResponse {
        let status = self.query.get_status();
        log::debug!("IPC: status {status:?}");
        IpcResponse::Status {
            running: status.daemon_running,
            uptime_secs: status.uptime_secs,
            active_profile: status.active_profile,
            device_count: status.device_count,
        }
    }

    fn handle_get_latency(&self) -> IpcResponse {
        let snapshot = self.query.get_latency_snapshot();
        IpcResponse::Latency {
            min_us: snapshot.min_us,
            avg_us: snapshot.avg_us,
            max_us: snapshot.max_us,
            p95_us: snapshot.p95_us,
            p99_us: snapshot.p99_us,
        }
    }

    /// Activates a profile (compile + load), then records it in the read model
    /// so status and the daemon's reload loop see it — same as the REST path.
    fn handle_activate_profile(&self, name: String) -> IpcResponse {
        log::info!("IPC: Activating profile '{}'", name);

        let error = match self.profile_manager.activate(&name) {
            Ok(result) if result.success => {
                log::info!(
                    "IPC: Profile '{}' activated (compile: {}ms, reload: {}ms)",
                    name,
                    result.compile_time_ms,
                    result.reload_time_ms
                );
                self.query.request_profile_activation(&name);
                return IpcResponse::ProfileActivated { name };
            }
            Ok(result) => format!(
                "Profile activation failed: {}",
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ),
            Err(e) => format!("Profile activation error: {}", e),
        };
        log::error!("IPC: Profile '{}': {}", name, error);
        IpcResponse::Error {
            code: 5002,
            message: error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::daemon::telemetry::{DaemonTelemetry, TelemetryState, STATE_BITS};
    use crate::daemon::DaemonSharedState;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicBool;
    use tempfile::TempDir;

    struct Fixture {
        handler: IpcCommandHandler,
        query: Arc<DaemonQueryService>,
        telemetry: Arc<DaemonTelemetry>,
        _dir: TempDir,
    }

    /// A handler over a "running daemon" with profile `work` and 2 devices.
    fn fixture() -> Fixture {
        let dir = TempDir::new().unwrap();
        let profile_manager = Arc::new(ProfileManager::new(dir.path().to_path_buf()).unwrap());
        let telemetry = Arc::new(DaemonTelemetry::new());
        let state = Arc::new(DaemonSharedState::new(
            Arc::new(AtomicBool::new(true)),
            Some("work".to_string()),
            PathBuf::from("work.krx"),
            2,
        ));
        let query = Arc::new(DaemonQueryService::new(state, Arc::clone(&telemetry)));
        Fixture {
            handler: IpcCommandHandler::new(profile_manager, Arc::clone(&query)),
            query,
            telemetry,
            _dir: dir,
        }
    }

    /// Regression: IPC status used to report hardcoded zeros and the on-disk
    /// profile instead of the live daemon state the web API reports.
    #[test]
    fn test_get_status_matches_read_model() {
        let f = fixture();
        let expected = f.query.get_status();

        match f.handler.handle(IpcRequest::GetStatus) {
            IpcResponse::Status {
                running,
                uptime_secs,
                active_profile,
                device_count,
            } => {
                assert!(running);
                assert_eq!(active_profile.as_deref(), Some("work"));
                assert_eq!(device_count, 2);
                assert_eq!(uptime_secs, expected.uptime_secs);
            }
            other => panic!("Expected Status response, got {other:?}"),
        }
    }

    #[test]
    fn test_get_status_without_daemon() {
        let dir = TempDir::new().unwrap();
        let pm = Arc::new(ProfileManager::new(dir.path().to_path_buf()).unwrap());
        let handler = IpcCommandHandler::new(pm, Arc::new(DaemonQueryService::without_daemon()));

        match handler.handle(IpcRequest::GetStatus) {
            IpcResponse::Status {
                running,
                active_profile,
                device_count,
                ..
            } => {
                assert!(!running);
                assert_eq!(active_profile, None);
                assert_eq!(device_count, 0);
            }
            other => panic!("Expected Status response, got {other:?}"),
        }
    }

    #[test]
    fn test_activate_profile_not_found() {
        let f = fixture();
        let response = f.handler.handle(IpcRequest::ActivateProfile {
            name: "nonexistent".to_string(),
        });

        match response {
            IpcResponse::Error { code, message } => {
                assert_eq!(code, 5002);
                assert!(message.contains("not found") || message.contains("activation"));
            }
            other => panic!("Expected Error response, got {other:?}"),
        }
        // A failed activation must not change the reported profile.
        assert_eq!(f.query.get_status().active_profile.as_deref(), Some("work"));
    }

    #[test]
    fn test_get_state_reads_telemetry() {
        let f = fixture();
        let mut s = TelemetryState::empty();
        s.set_modifier(7, true);
        f.telemetry.update_state(s);

        match f.handler.handle(IpcRequest::GetState) {
            IpcResponse::State { state } => {
                assert_eq!(state.len(), STATE_BITS);
                assert!(state[7]);
            }
            other => panic!("Expected State response, got {other:?}"),
        }
    }

    #[test]
    fn test_get_latency_reads_recorder() {
        let f = fixture();
        f.telemetry.latency_recorder().record(250);

        match f.handler.handle(IpcRequest::GetLatencyMetrics) {
            IpcResponse::Latency { min_us, max_us, .. } => {
                assert!(min_us >= 250 && max_us >= 250, "{min_us}..{max_us}");
            }
            other => panic!("Expected Latency response, got {other:?}"),
        }
    }

    #[test]
    fn test_events_tail_and_clear() {
        let f = fixture();
        f.telemetry.push_event("press A".to_string());
        f.telemetry.push_event("release A".to_string());

        match f.handler.handle(IpcRequest::GetEventsTail { count: 10 }) {
            IpcResponse::Events { events } => assert_eq!(events, vec!["press A", "release A"]),
            other => panic!("Expected Events response, got {other:?}"),
        }
        match f.handler.handle(IpcRequest::ClearEvents) {
            IpcResponse::EventsCleared { count } => assert_eq!(count, 2),
            other => panic!("Expected EventsCleared response, got {other:?}"),
        }
        assert!(f.query.get_recent_events(10).is_empty());
    }
}
