//! Dependency injection container for service management.
//!
//! This module provides a ServiceContainer that manages all application services
//! and their dependencies. It implements the Dependency Inversion Principle by
//! centralizing service creation and wiring.
//!
//! # Architecture
//!
//! The ServiceContainer follows these principles:
//! - **Single Responsibility**: Only manages service lifecycle and wiring
//! - **Dependency Inversion**: Services depend on abstractions (Arc<T>)
//! - **Builder Pattern**: Flexible construction with ServiceContainerBuilder
//! - **Thread Safety**: All services are Send + Sync via Arc
//!
//! # Usage
//!
//! ```no_run
//! use std::path::PathBuf;
//! use keyrx_daemon::container::ServiceContainerBuilder;
//!
//! let config_dir = PathBuf::from("/path/to/config");
//! let container = ServiceContainerBuilder::new(config_dir)
//!     .build()
//!     .expect("Failed to build service container");
//!
//! // Access services via getters
//! let profile_service = container.profile_service();
//! let device_service = container.device_service();
//! ```
//!
//! For complete examples of refactoring main.rs, see `example_usage.rs`.

pub mod example_usage;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

use crate::config::ProfileManager;
use crate::macro_recorder::MacroRecorder;
use crate::services::{
    ConfigService, DeviceService, ProfileService, SettingsService, SimulationService,
};
use crate::web::rpc_types::ServerMessage;
use crate::web::subscriptions::SubscriptionManager;

/// Container for all application services with dependency injection.
///
/// This struct owns all service instances and provides accessor methods
/// for dependency injection. It ensures:
/// - Services are created once with correct dependencies
/// - Thread-safe access via Arc<T>
/// - Consistent service lifecycle management
///
/// # Thread Safety
///
/// ServiceContainer is Send + Sync and can be shared across threads.
/// All services are wrapped in Arc for cheap cloning.
#[derive(Clone)]
pub struct ServiceContainer {
    /// Macro recorder for capturing keyboard event sequences
    macro_recorder: Arc<MacroRecorder>,
    /// Profile service for profile management operations
    profile_service: Arc<ProfileService>,
    /// Device service for device management operations
    device_service: Arc<DeviceService>,
    /// Configuration service for config operations
    config_service: Arc<ConfigService>,
    /// Settings service for daemon settings
    settings_service: Arc<SettingsService>,
    /// Simulation service for event simulation
    simulation_service: Arc<SimulationService>,
    /// Subscription manager for WebSocket pub/sub
    subscription_manager: Arc<SubscriptionManager>,
    /// Event broadcaster for RPC events
    event_broadcaster: broadcast::Sender<ServerMessage>,
}

impl ServiceContainer {
    /// Get macro recorder reference
    pub fn macro_recorder(&self) -> Arc<MacroRecorder> {
        Arc::clone(&self.macro_recorder)
    }

    /// Get profile service reference
    pub fn profile_service(&self) -> Arc<ProfileService> {
        Arc::clone(&self.profile_service)
    }

    /// Get device service reference
    pub fn device_service(&self) -> Arc<DeviceService> {
        Arc::clone(&self.device_service)
    }

    /// Get configuration service reference
    pub fn config_service(&self) -> Arc<ConfigService> {
        Arc::clone(&self.config_service)
    }

    /// Get settings service reference
    pub fn settings_service(&self) -> Arc<SettingsService> {
        Arc::clone(&self.settings_service)
    }

    /// Get simulation service reference
    pub fn simulation_service(&self) -> Arc<SimulationService> {
        Arc::clone(&self.simulation_service)
    }

    /// Get subscription manager reference
    pub fn subscription_manager(&self) -> Arc<SubscriptionManager> {
        Arc::clone(&self.subscription_manager)
    }

    /// Get event broadcaster channel sender
    pub fn event_broadcaster(&self) -> broadcast::Sender<ServerMessage> {
        self.event_broadcaster.clone()
    }
}

/// Builder for ServiceContainer with flexible dependency injection.
///
/// This builder implements the Builder Pattern to provide a fluent API
/// for constructing ServiceContainer instances with optional dependencies.
///
/// # Examples
///
/// ```no_run
/// use std::path::PathBuf;
/// use keyrx_daemon::container::ServiceContainerBuilder;
///
/// // Production mode
/// let container = ServiceContainerBuilder::new(PathBuf::from("~/.config/keyrx"))
///     .build()
///     .expect("Failed to build container");
///
/// // Test mode with IPC socket
/// let test_container = ServiceContainerBuilder::new(PathBuf::from("/tmp/test"))
///     .with_test_mode_socket(PathBuf::from("/tmp/test.sock"))
///     .build()
///     .expect("Failed to build test container");
/// ```
pub struct ServiceContainerBuilder {
    config_dir: PathBuf,
    test_mode_socket: Option<PathBuf>,
    event_channel_size: usize,
}

impl ServiceContainerBuilder {
    /// Create a new builder with the specified config directory
    ///
    /// # Arguments
    ///
    /// * `config_dir` - Path to configuration directory (e.g., ~/.config/keyrx)
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            test_mode_socket: None,
            event_channel_size: 1000, // Default channel size
        }
    }

    /// Enable test mode with IPC socket path
    ///
    /// Test mode allows running the daemon without keyboard capture for testing.
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Path to IPC socket (e.g., /tmp/keyrx-test.sock)
    pub fn with_test_mode_socket(mut self, socket_path: PathBuf) -> Self {
        self.test_mode_socket = Some(socket_path);
        self
    }

    /// Set the event broadcast channel size
    ///
    /// Default is 1000. Increase for high-throughput scenarios.
    ///
    /// # Arguments
    ///
    /// * `size` - Channel buffer size
    pub fn with_event_channel_size(mut self, size: usize) -> Self {
        self.event_channel_size = size;
        self
    }

    /// Build the service container with all dependencies wired
    ///
    /// This method:
    /// 1. Creates ProfileManager (shared dependency)
    /// 2. Creates all services with injected dependencies
    /// 3. Wires event broadcaster
    /// 4. Returns the fully initialized container
    ///
    /// # Errors
    ///
    /// Returns ContainerError if:
    /// - ProfileManager initialization fails
    /// - Config directory is invalid
    /// - Required dependencies cannot be created
    pub fn build(self) -> Result<ServiceContainer, ContainerError> {
        log::debug!(
            "Building ServiceContainer with config_dir: {:?}",
            self.config_dir
        );

        // Create ProfileManager first (shared dependency)
        let profile_manager = Arc::new(
            ProfileManager::new(self.config_dir.clone()).map_err(ContainerError::ProfileManager)?,
        );

        log::debug!("ProfileManager initialized");

        // Create MacroRecorder
        let macro_recorder = Arc::new(MacroRecorder::new());

        // Create services with injected dependencies
        let profile_service = Arc::new(ProfileService::new(Arc::clone(&profile_manager)));
        let device_service = Arc::new(DeviceService::new(self.config_dir.clone()));
        let config_service = Arc::new(ConfigService::new(Arc::clone(&profile_manager)));
        let settings_service = Arc::new(SettingsService::new(self.config_dir.clone()));

        // Create SimulationService with optional event channel
        let (macro_event_tx, macro_event_rx) = if self.test_mode_socket.is_some() {
            let (tx, rx) = tokio::sync::mpsc::channel(self.event_channel_size);
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };

        let simulation_service = Arc::new(SimulationService::new(
            self.config_dir.clone(),
            macro_event_tx,
        ));

        // Create SubscriptionManager
        let subscription_manager = Arc::new(SubscriptionManager::new());

        // Create event broadcaster
        let (event_broadcaster, _event_rx) = broadcast::channel(self.event_channel_size);

        // If test mode, spawn macro recorder event loop
        if let Some(rx) = macro_event_rx {
            let recorder_for_loop = (*macro_recorder).clone();
            tokio::spawn(async move {
                recorder_for_loop.run_event_loop(rx).await;
            });
            log::debug!("Test mode enabled: macro recorder event loop spawned");
        }

        log::info!("ServiceContainer built successfully");

        Ok(ServiceContainer {
            macro_recorder,
            profile_service,
            device_service,
            config_service,
            settings_service,
            simulation_service,
            subscription_manager,
            event_broadcaster,
        })
    }
}

/// Errors that can occur during container construction.
#[derive(Debug, thiserror::Error)]
pub enum ContainerError {
    /// Failed to create ProfileManager
    #[error("Failed to create profile manager: {0}")]
    ProfileManager(#[from] crate::config::profile_manager::ProfileError),

    /// Configuration directory is invalid
    #[error("Invalid config directory: {0}")]
    InvalidConfigDir(String),

    /// Generic I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_service_container_build() {
        let temp_dir = tempdir().unwrap();
        let container = ServiceContainerBuilder::new(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        // Verify all services are initialized
        assert!(Arc::strong_count(&container.macro_recorder()) >= 1);
        assert!(Arc::strong_count(&container.profile_service()) >= 1);
        assert!(Arc::strong_count(&container.device_service()) >= 1);
        assert!(Arc::strong_count(&container.config_service()) >= 1);
        assert!(Arc::strong_count(&container.settings_service()) >= 1);
        assert!(Arc::strong_count(&container.simulation_service()) >= 1);
        assert!(Arc::strong_count(&container.subscription_manager()) >= 1);
    }

    #[tokio::test]
    async fn test_test_mode_enabled() {
        let temp_dir = tempdir().unwrap();
        let socket_path = temp_dir.path().join("test.sock");

        let container = ServiceContainerBuilder::new(temp_dir.path().to_path_buf())
            .with_test_mode_socket(socket_path)
            .build()
            .unwrap();

        // Container should be created successfully with test mode
        assert!(Arc::strong_count(&container.simulation_service()) >= 1);
    }

    #[test]
    fn test_custom_channel_size() {
        let temp_dir = tempdir().unwrap();

        let container = ServiceContainerBuilder::new(temp_dir.path().to_path_buf())
            .with_event_channel_size(5000)
            .build()
            .unwrap();

        // Verify container created with custom channel size
        assert!(Arc::strong_count(&container.macro_recorder()) >= 1);
    }

    #[test]
    fn test_service_cloning() {
        let temp_dir = tempdir().unwrap();
        let container = ServiceContainerBuilder::new(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        // Get multiple references to the same service
        let profile1 = container.profile_service();
        let profile2 = container.profile_service();

        // Both should point to the same Arc
        assert!(Arc::ptr_eq(&profile1, &profile2));
    }

    #[test]
    fn test_container_clone() {
        let temp_dir = tempdir().unwrap();
        let container1 = ServiceContainerBuilder::new(temp_dir.path().to_path_buf())
            .build()
            .unwrap();

        let container2 = container1.clone();

        // Both containers should share the same services
        assert!(Arc::ptr_eq(
            &container1.profile_service(),
            &container2.profile_service()
        ));
    }
}
