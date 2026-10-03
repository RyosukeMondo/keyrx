//! Platform-specific daemon runners.
//!
//! This module contains the platform-specific implementations for running the daemon
//! with keyboard capture, web server, and IPC infrastructure.

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(any(target_os = "linux", target_os = "windows"))]
pub use production_ipc::{ensure_no_other_daemon, start_production_ipc_server, ProductionIpc};

#[cfg(any(target_os = "linux", target_os = "windows"))]
mod production_ipc {
    use crate::ipc::commands::IpcCommandHandler;
    use crate::ipc::IpcEndpoint;
    use std::sync::Arc;

    /// Owns the production IPC endpoint this process bound. Dropping it
    /// removes the endpoint (a socket file on Unix) so the CLI does not find
    /// a stale one - but ONLY if this process bound it: a daemon that lost
    /// the race to another one owns nothing and must never delete the
    /// running daemon's socket on its way out.
    #[must_use = "the endpoint is removed when this guard is dropped; hold it until the daemon exits"]
    pub struct ProductionIpc(Option<IpcEndpoint>);

    impl ProductionIpc {
        /// True when this process is serving the endpoint.
        pub fn is_serving(&self) -> bool {
            self.0.is_some()
        }
    }

    impl Drop for ProductionIpc {
        fn drop(&mut self) {
            if let Some(endpoint) = self.0.take() {
                endpoint.remove();
            }
        }
    }

    /// Refuses to start next to a daemon that is already serving the default
    /// endpoint. A second instance could only fight it for the keyboards
    /// (the first holds the grab) and sit idle holding a virtual device, a
    /// web port and a tray icon - so say so and stop, before touching any
    /// device.
    ///
    /// # Errors
    /// A message naming the endpoint when a live daemon answers on it.
    pub fn ensure_no_other_daemon() -> Result<(), String> {
        let endpoint = IpcEndpoint::default_for_platform();
        if endpoint.probe_live() {
            return Err(format!(
                "another keyrx daemon is already running (it answers on {endpoint}); stop it \
                 first (`keyrx_daemon status` shows it), or give this one its own \
                 XDG_RUNTIME_DIR"
            ));
        }
        Ok(())
    }

    /// Serves `daemon_query` over IPC on the platform's default endpoint so the
    /// `keyrx_daemon status|state|metrics` CLI reports exactly what the web API
    /// reports. Pass the same `daemon_query` the web `AppState` uses.
    ///
    /// Best-effort: if the endpoint cannot be bound (e.g. another daemon owns
    /// it) this logs a warning and the daemon runs without IPC; the returned
    /// guard then owns nothing.
    pub fn start_production_ipc_server(
        container: &crate::container::ServiceContainer,
        daemon_query: Arc<crate::services::DaemonQueryService>,
    ) -> ProductionIpc {
        let profile_manager = Arc::clone(container.profile_service().profile_manager());
        let handler = Arc::new(IpcCommandHandler::new(profile_manager, daemon_query));
        let endpoint = IpcEndpoint::default_for_platform();
        match crate::ipc::server::spawn(endpoint.clone(), handler) {
            Ok(()) => {
                log::info!("Production IPC server listening on {endpoint}");
                ProductionIpc(Some(endpoint))
            }
            Err(e) => {
                log::warn!("Production IPC server disabled ({endpoint}): {e}");
                ProductionIpc(None)
            }
        }
    }

    #[cfg(all(test, unix))]
    mod tests {
        use super::*;

        /// Regression: a second daemon (IPC bind refused) removed the FIRST
        /// daemon's socket when it exited, leaving `status` unable to find a
        /// daemon that was still running.
        #[test]
        fn a_guard_that_bound_nothing_leaves_the_running_daemons_socket() {
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("keyrx-daemon.sock");
            let _listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
            let endpoint = IpcEndpoint::SocketFile(path.clone());
            assert!(endpoint.prepare_bind().is_err(), "live socket is refused");

            drop(ProductionIpc(None)); // what the losing daemon holds
            assert!(path.exists());

            drop(ProductionIpc(Some(endpoint))); // the owner cleans up
            assert!(!path.exists());
        }
    }
}
