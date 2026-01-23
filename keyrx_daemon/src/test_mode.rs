//! Test mode infrastructure for E2E testing without keyboard capture.
//!
//! This module provides the infrastructure to run the daemon in test mode,
//! where keyboard capture is disabled and profile activation is handled via IPC.

use crate::config::ProfileManager;
use crate::ipc::commands::IpcCommandHandler;
use crate::ipc::server::{get_test_socket_path, IpcServer};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Test mode context that holds IPC server and command handler.
pub struct TestModeContext {
    ipc_server: Arc<Mutex<IpcServer>>,
    command_handler: Arc<IpcCommandHandler>,
    daemon_running: Arc<RwLock<bool>>,
}

impl TestModeContext {
    /// Create a new test mode context with IPC infrastructure.
    ///
    /// # Arguments
    ///
    /// * `profile_manager` - Shared ProfileManager for profile operations
    ///
    /// # Returns
    ///
    /// Returns a TestModeContext or an error if IPC server creation fails.
    pub fn new(profile_manager: Arc<RwLock<ProfileManager>>) -> Result<Self, std::io::Error> {
        let socket_path = get_test_socket_path();
        log::info!(
            "Test mode: Creating IPC server at {}",
            socket_path.display()
        );

        let ipc_server = IpcServer::new(socket_path)?;
        let daemon_running = Arc::new(RwLock::new(true));

        let command_handler = Arc::new(IpcCommandHandler::new(
            profile_manager,
            Arc::clone(&daemon_running),
        ));

        Ok(Self {
            ipc_server: Arc::new(Mutex::new(ipc_server)),
            command_handler,
            daemon_running,
        })
    }

    /// Start the IPC server in a background thread.
    ///
    /// This spawns a thread that listens for IPC connections and handles requests.
    pub fn start_ipc_server(&self) -> Result<(), std::io::Error> {
        let server = Arc::clone(&self.ipc_server);
        let handler = Arc::clone(&self.command_handler);

        // Start the server
        {
            let mut server_guard = server.blocking_lock();
            server_guard.start()?;
            log::info!(
                "Test mode: IPC server started at {}",
                server_guard.socket_path().display()
            );
        }

        // Spawn handler thread
        std::thread::spawn(move || {
            let handler_fn = Arc::new(Mutex::new(move |request| {
                // Block on async handler
                let handler = Arc::clone(&handler);
                let rt = tokio::runtime::Runtime::new().unwrap();
                let response = rt.block_on(async { handler.handle(request).await });
                Ok(response)
            }));

            let server_guard = server.blocking_lock();
            if let Err(e) = server_guard.handle_connections(handler_fn) {
                log::error!("IPC server error: {}", e);
            }
        });

        Ok(())
    }

    /// Get the daemon running flag.
    pub fn daemon_running(&self) -> Arc<RwLock<bool>> {
        Arc::clone(&self.daemon_running)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_context_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let profile_manager = ProfileManager::new(config_dir).unwrap();
        let profile_manager = Arc::new(RwLock::new(profile_manager));

        let context = TestModeContext::new(profile_manager);
        assert!(context.is_ok());
    }

    #[test]
    fn test_daemon_running_flag() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let profile_manager = ProfileManager::new(config_dir).unwrap();
        let profile_manager = Arc::new(RwLock::new(profile_manager));

        let context = TestModeContext::new(profile_manager).unwrap();
        let running = context.daemon_running();

        // Verify we can read the flag
        let rt = tokio::runtime::Runtime::new().unwrap();
        let is_running = rt.block_on(async { *running.read().await });
        assert!(is_running);
    }
}
