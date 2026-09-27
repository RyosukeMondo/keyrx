//! Platform-specific daemon runners.
//!
//! This module contains the platform-specific implementations for running the daemon
//! with keyboard capture, web server, and IPC infrastructure.

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(any(target_os = "linux", target_os = "windows"))]
pub use production_ipc::{remove_production_ipc_endpoint, start_production_ipc_server};

#[cfg(any(target_os = "linux", target_os = "windows"))]
mod production_ipc {
    use crate::ipc::commands::IpcCommandHandler;
    use crate::ipc::IpcEndpoint;
    use std::sync::Arc;

    /// Serves `daemon_query` over IPC on the platform's default endpoint so the
    /// `keyrx_daemon status|state|metrics` CLI reports exactly what the web API
    /// reports. Pass the same `daemon_query` the web `AppState` uses.
    ///
    /// Best-effort: if the endpoint cannot be bound (e.g. another daemon owns
    /// the Windows pipe) this logs a warning and the daemon runs without IPC.
    pub fn start_production_ipc_server(
        container: &crate::container::ServiceContainer,
        daemon_query: Arc<crate::services::DaemonQueryService>,
    ) {
        let profile_manager = Arc::clone(container.profile_service().profile_manager());
        let handler = Arc::new(IpcCommandHandler::new(profile_manager, daemon_query));
        let endpoint = IpcEndpoint::default_for_platform();
        match crate::ipc::server::spawn(endpoint.clone(), handler) {
            Ok(()) => log::info!("Production IPC server listening on {endpoint}"),
            Err(e) => log::warn!("Production IPC server disabled ({endpoint}): {e}"),
        }
    }

    /// Removes the production endpoint on exit so the CLI does not find a
    /// stale socket file (no-op for named pipes).
    pub fn remove_production_ipc_endpoint() {
        IpcEndpoint::default_for_platform().remove();
    }
}
