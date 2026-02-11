//! Web server factory for creating configured Axum servers.
//!
//! This module provides centralized web server initialization with dependency injection.

use crate::container::ServiceContainer;
use crate::web::AppState;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

/// Factory for creating web server instances.
pub struct WebServerFactory {
    addr: SocketAddr,
    service_container: Arc<ServiceContainer>,
    test_mode_socket: Option<PathBuf>,
}

impl WebServerFactory {
    /// Create a new web server factory.
    ///
    /// # Arguments
    ///
    /// * `addr` - Socket address to bind to
    /// * `service_container` - Service container with all dependencies
    pub fn new(addr: SocketAddr, service_container: Arc<ServiceContainer>) -> Self {
        Self {
            addr,
            service_container,
            test_mode_socket: None,
        }
    }

    /// Set test mode socket path.
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Path to IPC socket for test mode
    pub fn with_test_mode(mut self, socket_path: PathBuf) -> Self {
        self.test_mode_socket = Some(socket_path);
        self
    }

    /// Build and start the web server.
    ///
    /// # Returns
    ///
    /// Returns a future that resolves when the server stops.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Server fails to bind to the address
    /// - Server encounters a runtime error
    pub async fn serve(
        self,
        event_tx: broadcast::Sender<crate::web::events::EventMessage>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Create RPC event broadcaster
        let (rpc_event_tx, _) = broadcast::channel(1000);

        // Build AppState from service container
        let app_state = if let Some(socket_path) = self.test_mode_socket {
            Arc::new(AppState::new_with_test_mode(
                self.service_container.macro_recorder(),
                self.service_container.profile_service(),
                self.service_container.device_service(),
                self.service_container.config_service(),
                self.service_container.settings_service(),
                self.service_container.simulation_service(),
                self.service_container.subscription_manager(),
                rpc_event_tx,
                socket_path,
            ))
        } else {
            Arc::new(AppState::new(
                self.service_container.macro_recorder(),
                self.service_container.profile_service(),
                self.service_container.device_service(),
                self.service_container.config_service(),
                self.service_container.settings_service(),
                self.service_container.simulation_service(),
                self.service_container.subscription_manager(),
                rpc_event_tx,
            ))
        };

        log::info!("Starting web server on http://{}", self.addr);

        // Start the web server
        crate::web::serve(self.addr, event_tx, app_state).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_web_server_factory_new() {
        let temp_dir = tempdir().unwrap();
        let container = Arc::new(
            crate::container::ServiceContainerBuilder::new(temp_dir.path().to_path_buf())
                .build()
                .unwrap(),
        );

        let addr: SocketAddr =
            ([127, 0, 0, 1], crate::daemon_config::DEFAULT_PORT).into();
        let factory = WebServerFactory::new(addr, container);
        assert_eq!(factory.addr, addr);
        assert!(factory.test_mode_socket.is_none());
    }

    #[test]
    fn test_web_server_factory_with_test_mode() {
        let temp_dir = tempdir().unwrap();
        let container = Arc::new(
            crate::container::ServiceContainerBuilder::new(temp_dir.path().to_path_buf())
                .build()
                .unwrap(),
        );

        let addr: SocketAddr =
            ([127, 0, 0, 1], crate::daemon_config::DEFAULT_PORT).into();
        let socket_path = PathBuf::from("/tmp/test.sock");
        let factory = WebServerFactory::new(addr, container).with_test_mode(socket_path.clone());
        assert_eq!(factory.test_mode_socket, Some(socket_path));
    }
}
