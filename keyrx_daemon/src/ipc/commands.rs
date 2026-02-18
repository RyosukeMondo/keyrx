//! IPC command handlers for test mode.
//!
//! This module provides command handling logic for IPC requests, including
//! profile activation and daemon status queries.

use super::{IpcRequest, IpcResponse};
use crate::config::profile_manager::ProfileManager;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Handler for IPC commands in test mode.
///
/// This struct manages the execution of IPC commands, coordinating with
/// the ProfileManager and daemon state.
pub struct IpcCommandHandler {
    profile_manager: Arc<ProfileManager>,
    daemon_running: Arc<RwLock<bool>>,
}

impl IpcCommandHandler {
    /// Create a new command handler.
    ///
    /// # Arguments
    ///
    /// * `profile_manager` - Shared ProfileManager for profile operations
    /// * `daemon_running` - Shared flag indicating daemon running state
    pub fn new(profile_manager: Arc<ProfileManager>, daemon_running: Arc<RwLock<bool>>) -> Self {
        Self {
            profile_manager,
            daemon_running,
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
            IpcRequest::GetState => {
                // State query not yet implemented
                IpcResponse::Error {
                    code: 5001,
                    message: "GetState not implemented yet".to_string(),
                }
            }
            IpcRequest::GetLatencyMetrics => {
                // Latency metrics not yet implemented
                IpcResponse::Error {
                    code: 5001,
                    message: "GetLatencyMetrics not implemented yet".to_string(),
                }
            }
            IpcRequest::GetEventsTail { .. } => {
                // Events tail not yet implemented
                IpcResponse::Error {
                    code: 5001,
                    message: "GetEventsTail not implemented yet".to_string(),
                }
            }
        }
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
    async fn test_unimplemented_commands() {
        let (handler, _temp_dir) = setup_test_handler().await;

        // Test GetState
        let response = handler.handle(IpcRequest::GetState).await;
        match response {
            IpcResponse::Error { code, message } => {
                assert_eq!(code, 5001);
                assert!(message.contains("not implemented"));
            }
            _ => panic!("Expected Error response"),
        }

        // Test GetLatencyMetrics
        let response = handler.handle(IpcRequest::GetLatencyMetrics).await;
        match response {
            IpcResponse::Error { code, message } => {
                assert_eq!(code, 5001);
                assert!(message.contains("not implemented"));
            }
            _ => panic!("Expected Error response"),
        }

        // Test GetEventsTail
        let response = handler
            .handle(IpcRequest::GetEventsTail { count: 10 })
            .await;
        match response {
            IpcResponse::Error { code, message } => {
                assert_eq!(code, 5001);
                assert!(message.contains("not implemented"));
            }
            _ => panic!("Expected Error response"),
        }
    }
}
