//! IPC command handlers for test mode.
//!
//! This module provides command handling logic for IPC requests, including
//! profile activation and daemon status queries.

use super::{IpcRequest, IpcResponse};
use crate::config::profile_manager::ProfileManager;
use crate::daemon::metrics::LatencySnapshot;
use crate::daemon::telemetry::{DaemonTelemetry, STATE_BITS};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Handler for IPC commands in test mode.
///
/// This struct manages the execution of IPC commands, coordinating with
/// the ProfileManager and daemon state.
pub struct IpcCommandHandler {
    profile_manager: Arc<ProfileManager>,
    daemon_running: Arc<RwLock<bool>>,
    /// Live runtime telemetry (state/latency/events). `None` in test mode where
    /// no keyboard events are processed — telemetry queries then return defaults.
    telemetry: Option<Arc<DaemonTelemetry>>,
}

impl IpcCommandHandler {
    /// Create a new command handler without live telemetry (test mode).
    ///
    /// Telemetry queries (`GetState`/`GetLatencyMetrics`/`GetEventsTail`) return
    /// well-formed empty/default responses.
    ///
    /// # Arguments
    ///
    /// * `profile_manager` - Shared ProfileManager for profile operations
    /// * `daemon_running` - Shared flag indicating daemon running state
    pub fn new(profile_manager: Arc<ProfileManager>, daemon_running: Arc<RwLock<bool>>) -> Self {
        Self {
            profile_manager,
            daemon_running,
            telemetry: None,
        }
    }

    /// Create a command handler wired to live daemon telemetry (production mode).
    ///
    /// # Arguments
    ///
    /// * `profile_manager` - Shared ProfileManager for profile operations
    /// * `daemon_running` - Shared flag indicating daemon running state
    /// * `telemetry` - Shared live telemetry updated by the event loop
    pub fn with_telemetry(
        profile_manager: Arc<ProfileManager>,
        daemon_running: Arc<RwLock<bool>>,
        telemetry: Arc<DaemonTelemetry>,
    ) -> Self {
        Self {
            profile_manager,
            daemon_running,
            telemetry: Some(telemetry),
        }
    }

    /// Handle an IPC request and return the appropriate response.
    ///
    /// # Arguments
    ///
    /// * `request` - The IPC request to handle
    ///
    /// # Returns
    ///
    /// Returns an IpcResponse containing the result of the request, or an error response.
    pub async fn handle(&self, request: IpcRequest) -> IpcResponse {
        match request {
            IpcRequest::ActivateProfile { name } => self.handle_activate_profile(name).await,
            IpcRequest::GetStatus => self.handle_get_status().await,
            IpcRequest::GetState => self.handle_get_state(),
            IpcRequest::GetLatencyMetrics => self.handle_get_latency(),
            IpcRequest::GetEventsTail { count } => self.handle_get_events(count),
        }
    }

    /// Handle a live-state query.
    ///
    /// Returns the packed 255-bit modifier/lock/layer vector. Without live
    /// telemetry (test mode), returns the all-inactive default.
    fn handle_get_state(&self) -> IpcResponse {
        let state = match self.telemetry {
            Some(ref t) => t.raw_state(),
            None => vec![false; STATE_BITS],
        };
        IpcResponse::State { state }
    }

    /// Handle a latency-metrics query.
    ///
    /// Returns the latest aggregated latency snapshot. Without live telemetry
    /// (test mode), returns zeros.
    fn handle_get_latency(&self) -> IpcResponse {
        let snapshot = match self.telemetry {
            Some(ref t) => t.latency(),
            None => LatencySnapshot::empty(),
        };
        IpcResponse::Latency {
            min_us: snapshot.min_us,
            avg_us: snapshot.avg_us,
            max_us: snapshot.max_us,
            p95_us: snapshot.p95_us,
            p99_us: snapshot.p99_us,
        }
    }

    /// Handle a recent-events query.
    ///
    /// Returns up to `count` most recent event descriptions (oldest first).
    /// Without live telemetry (test mode), returns an empty list.
    fn handle_get_events(&self, count: usize) -> IpcResponse {
        let events = match self.telemetry {
            Some(ref t) => t.recent_events(count),
            None => Vec::new(),
        };
        IpcResponse::Events { events }
    }

    /// Handle profile activation request.
    ///
    /// This activates the specified profile and returns the result.
    /// The activation includes compilation and loading of the profile.
    async fn handle_activate_profile(&self, name: String) -> IpcResponse {
        log::info!("IPC: Activating profile '{}'", name);

        // Attempt to activate the profile
        match self.profile_manager.activate(&name) {
            Ok(result) => {
                if result.success {
                    log::info!(
                        "IPC: Profile '{}' activated successfully (compile: {}ms, reload: {}ms)",
                        name,
                        result.compile_time_ms,
                        result.reload_time_ms
                    );
                    IpcResponse::ProfileActivated { name }
                } else {
                    let error_msg = result.error.unwrap_or_else(|| "Unknown error".to_string());
                    log::error!("IPC: Profile '{}' activation failed: {}", name, error_msg);
                    IpcResponse::Error {
                        code: 5002,
                        message: format!("Profile activation failed: {}", error_msg),
                    }
                }
            }
            Err(e) => {
                log::error!("IPC: Profile '{}' activation error: {}", name, e);
                IpcResponse::Error {
                    code: 5002,
                    message: format!("Profile activation error: {}", e),
                }
            }
        }
    }

    /// Handle daemon status query.
    ///
    /// Returns the current daemon running state along with other status information.
    async fn handle_get_status(&self) -> IpcResponse {
        log::debug!("IPC: Querying daemon status");

        let running = *self.daemon_running.read().await;

        // Get active profile name (ProfileManager.get_active() is immutable, so no unsafe needed)
        let active_profile = self.profile_manager.get_active().ok().flatten();

        // Get device count (in test mode, this is always 0)
        let device_count = 0;

        // Get uptime (for now, just return 0 - we can add proper uptime tracking later)
        let uptime_secs = 0;

        IpcResponse::Status {
            running,
            uptime_secs,
            active_profile,
            device_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::profile_manager::ProfileManager;
    use tempfile::TempDir;

    async fn setup_test_handler() -> (IpcCommandHandler, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let profile_manager = ProfileManager::new(config_dir).unwrap();
        let profile_manager = Arc::new(profile_manager);
        let daemon_running = Arc::new(RwLock::new(true));

        let handler = IpcCommandHandler::new(profile_manager, daemon_running);
        (handler, temp_dir)
    }

    #[tokio::test]
    async fn test_get_status() {
        let (handler, _temp_dir) = setup_test_handler().await;

        let response = handler.handle(IpcRequest::GetStatus).await;

        match response {
            IpcResponse::Status {
                running,
                uptime_secs: _,
                active_profile: _,
                device_count,
            } => {
                assert!(running);
                assert_eq!(device_count, 0);
            }
            _ => panic!("Expected Status response"),
        }
    }

    #[tokio::test]
    async fn test_activate_profile_not_found() {
        let (handler, _temp_dir) = setup_test_handler().await;

        let response = handler
            .handle(IpcRequest::ActivateProfile {
                name: "nonexistent".to_string(),
            })
            .await;

        match response {
            IpcResponse::Error { code, message } => {
                assert_eq!(code, 5002);
                assert!(message.contains("not found") || message.contains("activation"));
            }
            _ => panic!("Expected Error response"),
        }
    }

    #[tokio::test]
    async fn test_telemetry_commands_default_empty() {
        // Without telemetry (test mode), telemetry queries return empty defaults,
        // NOT errors.
        let (handler, _temp_dir) = setup_test_handler().await;

        match handler.handle(IpcRequest::GetState).await {
            IpcResponse::State { state } => {
                assert_eq!(state.len(), STATE_BITS);
                assert!(state.iter().all(|&b| !b));
            }
            other => panic!("Expected State response, got {other:?}"),
        }

        match handler.handle(IpcRequest::GetLatencyMetrics).await {
            IpcResponse::Latency {
                min_us,
                avg_us,
                max_us,
                p95_us,
                p99_us,
            } => {
                assert_eq!((min_us, avg_us, max_us, p95_us, p99_us), (0, 0, 0, 0, 0));
            }
            other => panic!("Expected Latency response, got {other:?}"),
        }

        match handler.handle(IpcRequest::GetEventsTail { count: 10 }).await {
            IpcResponse::Events { events } => assert!(events.is_empty()),
            other => panic!("Expected Events response, got {other:?}"),
        }
    }

    async fn setup_telemetry_handler() -> (IpcCommandHandler, Arc<DaemonTelemetry>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let profile_manager = Arc::new(ProfileManager::new(temp_dir.path().to_path_buf()).unwrap());
        let daemon_running = Arc::new(RwLock::new(true));
        let telemetry = Arc::new(DaemonTelemetry::new());
        let handler = IpcCommandHandler::with_telemetry(
            profile_manager,
            daemon_running,
            Arc::clone(&telemetry),
        );
        (handler, telemetry, temp_dir)
    }

    #[tokio::test]
    async fn test_get_state_reads_telemetry() {
        use crate::daemon::telemetry::TelemetryState;
        let (handler, telemetry, _temp_dir) = setup_telemetry_handler().await;

        let mut s = TelemetryState::empty();
        s.set_modifier(7, true);
        telemetry.update_state(s);

        match handler.handle(IpcRequest::GetState).await {
            IpcResponse::State { state } => {
                assert_eq!(state.len(), STATE_BITS);
                assert!(state[7]);
            }
            other => panic!("Expected State response, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_get_latency_reads_telemetry() {
        let (handler, telemetry, _temp_dir) = setup_telemetry_handler().await;

        let mut snap = LatencySnapshot::empty();
        snap.avg_us = 250;
        snap.p99_us = 900;
        telemetry.update_latency(snap);

        match handler.handle(IpcRequest::GetLatencyMetrics).await {
            IpcResponse::Latency { avg_us, p99_us, .. } => {
                assert_eq!(avg_us, 250);
                assert_eq!(p99_us, 900);
            }
            other => panic!("Expected Latency response, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_get_events_reads_telemetry() {
        let (handler, telemetry, _temp_dir) = setup_telemetry_handler().await;

        telemetry.push_event("press A".to_string());
        telemetry.push_event("release A".to_string());

        match handler.handle(IpcRequest::GetEventsTail { count: 10 }).await {
            IpcResponse::Events { events } => {
                assert_eq!(events, vec!["press A", "release A"]);
            }
            other => panic!("Expected Events response, got {other:?}"),
        }
    }
}
