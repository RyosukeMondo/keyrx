//! KeyRx Public API Module
//!
//! This module provides the public API for interacting with KeyRx services.
//! It offers two usage patterns:
//!
//! ## 1. Dependency Injection (Recommended for Testing)
//!
//! Use [`ApiContext`] for explicit dependency injection, which enables:
//! - Fast, isolated unit testing with mock services
//! - Custom service configurations
//! - Clear dependency relationships
//!
//! ```rust,ignore
//! use std::sync::Arc;
//! use keyrx_core::api::ApiContext;
//! use keyrx_core::services::{MockDeviceService, MockProfileService, MockRuntimeService};
//!
//! // Create API with mock services for testing
//! let api = ApiContext::new(
//!     Arc::new(MockDeviceService::new().with_devices(vec![test_device])),
//!     Arc::new(MockProfileService::new()),
//!     Arc::new(MockRuntimeService::new()),
//! );
//!
//! // Use API methods
//! let devices = api.list_devices().await?;
//! ```
//!
//! ## 2. Global Functions (Backward Compatible)
//!
//! Standalone functions like [`list_devices()`] delegate to a global [`ApiContext`]
//! with default production services. These are convenient for simple usage:
//!
//! ```rust,ignore
//! use keyrx_core::api::{list_devices, save_keymap};
//!
//! // Simple global function calls
//! let devices = list_devices().await?;
//! let keymap = save_keymap(my_keymap)?;
//! ```
//!
//! ## Service Categories
//!
//! The API is organized into three service categories:
//!
//! - **Device API**: Device discovery, configuration, profile assignment
//! - **Profile API**: Virtual layouts, hardware profiles, keymaps (CRUD)
//! - **Runtime API**: Runtime configuration, slot management
//!
//! ## Observability
//!
//! The module also provides observability functions:
//! - [`init_logger()`]: Initialize the structured logger
//! - [`create_log_stream()`]: Create a callback-based log stream for Flutter integration

use std::sync::Arc;

use crate::config::models::{
    DeviceInstanceId, HardwareProfile, Keymap, ProfileSlot, RuntimeConfig, VirtualLayout,
};
use crate::observability::bridge::GLOBAL_LOG_BRIDGE;
use crate::observability::entry::LogEntry;
use crate::observability::logger::{OutputFormat, StructuredLogger};
use crate::services::device::DeviceView;
use crate::services::traits::{DeviceServiceTrait, ProfileServiceTrait, RuntimeServiceTrait};
use crate::services::{DeviceService, ProfileService, RuntimeService};
use lazy_static::lazy_static;

/// API context providing dependency-injected access to all services.
///
/// `ApiContext` is the primary entry point for interacting with KeyRx services.
/// It enables:
/// - Dependency injection for flexible service composition
/// - Mock implementations for fast, isolated unit testing
/// - Clear separation between API layer and service implementations
///
/// # Production Usage
///
/// ```rust,ignore
/// let api = ApiContext::with_defaults();
/// let devices = api.list_devices().await?;
/// ```
///
/// # Testing Usage
///
/// ```rust,ignore
/// use std::sync::Arc;
/// use keyrx_core::api::ApiContext;
/// use keyrx_core::services::{MockDeviceService, MockProfileService, MockRuntimeService};
///
/// #[tokio::test]
/// async fn test_list_devices() {
///     // Arrange: Create mock with test data
///     let mock_device = MockDeviceService::new()
///         .with_devices(vec![DeviceView { key: "test".into(), ..Default::default() }]);
///
///     let api = ApiContext::new(
///         Arc::new(mock_device),
///         Arc::new(MockProfileService::new()),
///         Arc::new(MockRuntimeService::new()),
///     );
///
///     // Act
///     let devices = api.list_devices().await.unwrap();
///
///     // Assert
///     assert_eq!(devices.len(), 1);
///     assert_eq!(devices[0].key, "test");
/// }
/// ```
pub struct ApiContext {
    device_service: Arc<dyn DeviceServiceTrait>,
    profile_service: Arc<dyn ProfileServiceTrait>,
    runtime_service: Arc<dyn RuntimeServiceTrait>,
}

impl ApiContext {
    /// Creates a new `ApiContext` with injected service dependencies.
    ///
    /// This constructor enables dependency injection for testing and custom configurations.
    /// Use this when you need to inject mock services for testing or custom service
    /// implementations.
    ///
    /// # Arguments
    ///
    /// * `device_service` - Implementation of [`DeviceServiceTrait`] for device operations
    /// * `profile_service` - Implementation of [`ProfileServiceTrait`] for profile management
    /// * `runtime_service` - Implementation of [`RuntimeServiceTrait`] for runtime configuration
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let api = ApiContext::new(
    ///     Arc::new(DeviceService::with_defaults(None)),
    ///     Arc::new(ProfileService::with_defaults()),
    ///     Arc::new(RuntimeService::with_defaults()),
    /// );
    /// ```
    pub fn new(
        device_service: Arc<dyn DeviceServiceTrait>,
        profile_service: Arc<dyn ProfileServiceTrait>,
        runtime_service: Arc<dyn RuntimeServiceTrait>,
    ) -> Self {
        Self {
            device_service,
            profile_service,
            runtime_service,
        }
    }

    /// Creates a new `ApiContext` with default production services.
    ///
    /// This convenience constructor creates the standard service implementations
    /// suitable for production use. It initializes:
    /// - `DeviceService` with no pre-loaded registry
    /// - `ProfileService` with default configuration paths
    /// - `RuntimeService` with default configuration
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let api = ApiContext::with_defaults();
    /// let devices = api.list_devices().await?;
    /// ```
    pub fn with_defaults() -> Self {
        Self::new(
            Arc::new(DeviceService::with_defaults(None)),
            Arc::new(ProfileService::with_defaults()),
            Arc::new(RuntimeService::with_defaults()),
        )
    }

    // Device Service API methods

    /// Lists all connected devices.
    #[tracing::instrument(skip(self))]
    pub async fn list_devices(&self) -> anyhow::Result<Vec<DeviceView>> {
        self.device_service.list_devices().await.map_err(Into::into)
    }

    /// Gets a specific device by its key.
    #[tracing::instrument(skip(self, device_key))]
    pub async fn get_device(&self, device_key: String) -> anyhow::Result<DeviceView> {
        self.device_service
            .get_device(&device_key)
            .await
            .map_err(Into::into)
    }

    /// Enables or disables remapping for a device.
    #[tracing::instrument(skip(self, device_key))]
    pub async fn set_device_remap(
        &self,
        device_key: String,
        enabled: bool,
    ) -> anyhow::Result<DeviceView> {
        self.device_service
            .set_remap_enabled(&device_key, enabled)
            .await
            .map_err(Into::into)
    }

    /// Assigns a profile to a device.
    #[tracing::instrument(skip(self, device_key, profile_id))]
    pub async fn assign_device_profile(
        &self,
        device_key: String,
        profile_id: String,
    ) -> anyhow::Result<DeviceView> {
        self.device_service
            .assign_profile(&device_key, &profile_id)
            .await
            .map_err(Into::into)
    }

    /// Unassigns the current profile from a device.
    #[tracing::instrument(skip(self, device_key))]
    pub async fn unassign_device_profile(&self, device_key: String) -> anyhow::Result<DeviceView> {
        self.device_service
            .unassign_profile(&device_key)
            .await
            .map_err(Into::into)
    }

    /// Sets or clears a label for a device.
    #[tracing::instrument(skip(self, device_key))]
    pub async fn set_device_label(
        &self,
        device_key: String,
        label: Option<String>,
    ) -> anyhow::Result<DeviceView> {
        self.device_service
            .set_label(&device_key, label)
            .await
            .map_err(Into::into)
    }

    // Profile Service API methods

    /// Lists all virtual layouts.
    #[tracing::instrument(skip(self))]
    pub fn list_virtual_layouts(&self) -> anyhow::Result<Vec<VirtualLayout>> {
        self.profile_service
            .list_virtual_layouts()
            .map_err(Into::into)
    }

    /// Saves a virtual layout.
    #[tracing::instrument(skip(self, layout))]
    pub fn save_virtual_layout(&self, layout: VirtualLayout) -> anyhow::Result<VirtualLayout> {
        self.profile_service
            .save_virtual_layout(layout)
            .map_err(Into::into)
    }

    /// Deletes a virtual layout by ID.
    #[tracing::instrument(skip(self, id))]
    pub fn delete_virtual_layout(&self, id: String) -> anyhow::Result<()> {
        self.profile_service
            .delete_virtual_layout(&id)
            .map_err(Into::into)
    }

    /// Lists all hardware profiles.
    #[tracing::instrument(skip(self))]
    pub fn list_hardware_profiles(&self) -> anyhow::Result<Vec<HardwareProfile>> {
        self.profile_service
            .list_hardware_profiles()
            .map_err(Into::into)
    }

    /// Saves a hardware profile.
    #[tracing::instrument(skip(self, profile))]
    pub fn save_hardware_profile(
        &self,
        profile: HardwareProfile,
    ) -> anyhow::Result<HardwareProfile> {
        self.profile_service
            .save_hardware_profile(profile)
            .map_err(Into::into)
    }

    /// Deletes a hardware profile by ID.
    #[tracing::instrument(skip(self, id))]
    pub fn delete_hardware_profile(&self, id: String) -> anyhow::Result<()> {
        self.profile_service
            .delete_hardware_profile(&id)
            .map_err(Into::into)
    }

    /// Lists all keymaps.
    #[tracing::instrument(skip(self))]
    pub fn list_keymaps(&self) -> anyhow::Result<Vec<Keymap>> {
        self.profile_service.list_keymaps().map_err(Into::into)
    }

    /// Saves a keymap.
    #[tracing::instrument(skip(self, keymap))]
    pub fn save_keymap(&self, keymap: Keymap) -> anyhow::Result<Keymap> {
        self.profile_service.save_keymap(keymap).map_err(Into::into)
    }

    /// Deletes a keymap by ID.
    #[tracing::instrument(skip(self, id))]
    pub fn delete_keymap(&self, id: String) -> anyhow::Result<()> {
        self.profile_service.delete_keymap(&id).map_err(Into::into)
    }

    // Runtime Service API methods

    /// Gets the current runtime configuration.
    #[tracing::instrument(skip(self))]
    pub fn get_runtime_config(&self) -> anyhow::Result<RuntimeConfig> {
        self.runtime_service.get_config().map_err(Into::into)
    }

    /// Adds a profile slot to a device's configuration.
    #[tracing::instrument(skip(self, device, slot))]
    pub fn runtime_add_slot(
        &self,
        device: DeviceInstanceId,
        slot: ProfileSlot,
    ) -> anyhow::Result<RuntimeConfig> {
        self.runtime_service
            .add_slot(device, slot)
            .map_err(Into::into)
    }

    /// Removes a profile slot from a device's configuration.
    #[tracing::instrument(skip(self, device, slot_id))]
    pub fn runtime_remove_slot(
        &self,
        device: DeviceInstanceId,
        slot_id: String,
    ) -> anyhow::Result<RuntimeConfig> {
        self.runtime_service
            .remove_slot(device, &slot_id)
            .map_err(Into::into)
    }

    /// Reorders a profile slot's priority for a device.
    #[tracing::instrument(skip(self, device, slot_id))]
    pub fn runtime_reorder_slot(
        &self,
        device: DeviceInstanceId,
        slot_id: String,
        new_priority: u32,
    ) -> anyhow::Result<RuntimeConfig> {
        self.runtime_service
            .reorder_slot(device, &slot_id, new_priority)
            .map_err(Into::into)
    }

    /// Sets whether a profile slot is active for a device.
    #[tracing::instrument(skip(self, device, slot_id))]
    pub fn runtime_set_slot_active(
        &self,
        device: DeviceInstanceId,
        slot_id: String,
        active: bool,
    ) -> anyhow::Result<RuntimeConfig> {
        self.runtime_service
            .set_slot_active(device, &slot_id, active)
            .map_err(Into::into)
    }
}

// Global API context for backward compatibility with standalone functions
lazy_static! {
    /// Global API instance using default production services.
    ///
    /// This static instance provides backward compatibility for existing code
    /// that uses standalone functions like `list_devices()` instead of
    /// `ApiContext::new().list_devices()`.
    static ref GLOBAL_API: ApiContext = ApiContext::with_defaults();
}

// Device Service API - standalone functions delegating to GLOBAL_API

/// Lists all connected devices.
#[tracing::instrument]
pub async fn list_devices() -> anyhow::Result<Vec<DeviceView>> {
    GLOBAL_API.list_devices().await
}

/// Gets a specific device by its key.
#[tracing::instrument(skip(device_key))]
pub async fn get_device(device_key: String) -> anyhow::Result<DeviceView> {
    GLOBAL_API.get_device(device_key).await
}

/// Enables or disables remapping for a device.
#[tracing::instrument(skip(device_key))]
pub async fn set_device_remap(device_key: String, enabled: bool) -> anyhow::Result<DeviceView> {
    GLOBAL_API.set_device_remap(device_key, enabled).await
}

/// Assigns a profile to a device.
#[tracing::instrument(skip(device_key, profile_id))]
pub async fn assign_device_profile(
    device_key: String,
    profile_id: String,
) -> anyhow::Result<DeviceView> {
    GLOBAL_API
        .assign_device_profile(device_key, profile_id)
        .await
}

/// Unassigns the current profile from a device.
#[tracing::instrument(skip(device_key))]
pub async fn unassign_device_profile(device_key: String) -> anyhow::Result<DeviceView> {
    GLOBAL_API.unassign_device_profile(device_key).await
}

/// Sets or clears a label for a device.
#[tracing::instrument(skip(device_key))]
pub async fn set_device_label(
    device_key: String,
    label: Option<String>,
) -> anyhow::Result<DeviceView> {
    GLOBAL_API.set_device_label(device_key, label).await
}

// Profile Service API - standalone functions delegating to GLOBAL_API

/// Lists all virtual layouts.
#[tracing::instrument]
pub fn list_virtual_layouts() -> anyhow::Result<Vec<VirtualLayout>> {
    GLOBAL_API.list_virtual_layouts()
}

/// Saves a virtual layout.
#[tracing::instrument(skip(layout))]
pub fn save_virtual_layout(layout: VirtualLayout) -> anyhow::Result<VirtualLayout> {
    GLOBAL_API.save_virtual_layout(layout)
}

/// Deletes a virtual layout by ID.
#[tracing::instrument(skip(id))]
pub fn delete_virtual_layout(id: String) -> anyhow::Result<()> {
    GLOBAL_API.delete_virtual_layout(id)
}

/// Lists all hardware profiles.
#[tracing::instrument]
pub fn list_hardware_profiles() -> anyhow::Result<Vec<HardwareProfile>> {
    GLOBAL_API.list_hardware_profiles()
}

/// Saves a hardware profile.
#[tracing::instrument(skip(profile))]
pub fn save_hardware_profile(profile: HardwareProfile) -> anyhow::Result<HardwareProfile> {
    GLOBAL_API.save_hardware_profile(profile)
}

/// Deletes a hardware profile by ID.
#[tracing::instrument(skip(id))]
pub fn delete_hardware_profile(id: String) -> anyhow::Result<()> {
    GLOBAL_API.delete_hardware_profile(id)
}

/// Lists all keymaps.
#[tracing::instrument]
pub fn list_keymaps() -> anyhow::Result<Vec<Keymap>> {
    GLOBAL_API.list_keymaps()
}

/// Saves a keymap.
#[tracing::instrument(skip(keymap))]
pub fn save_keymap(keymap: Keymap) -> anyhow::Result<Keymap> {
    GLOBAL_API.save_keymap(keymap)
}

/// Deletes a keymap by ID.
#[tracing::instrument(skip(id))]
pub fn delete_keymap(id: String) -> anyhow::Result<()> {
    GLOBAL_API.delete_keymap(id)
}

// Runtime Service API - standalone functions delegating to GLOBAL_API

/// Gets the current runtime configuration.
#[tracing::instrument]
pub fn get_runtime_config() -> anyhow::Result<RuntimeConfig> {
    GLOBAL_API.get_runtime_config()
}

/// Adds a profile slot to a device's configuration.
#[tracing::instrument(skip(device, slot))]
pub fn runtime_add_slot(
    device: DeviceInstanceId,
    slot: ProfileSlot,
) -> anyhow::Result<RuntimeConfig> {
    GLOBAL_API.runtime_add_slot(device, slot)
}

/// Removes a profile slot from a device's configuration.
#[tracing::instrument(skip(device, slot_id))]
pub fn runtime_remove_slot(
    device: DeviceInstanceId,
    slot_id: String,
) -> anyhow::Result<RuntimeConfig> {
    GLOBAL_API.runtime_remove_slot(device, slot_id)
}

/// Reorders a profile slot's priority for a device.
#[tracing::instrument(skip(device, slot_id))]
pub fn runtime_reorder_slot(
    device: DeviceInstanceId,
    slot_id: String,
    new_priority: u32,
) -> anyhow::Result<RuntimeConfig> {
    GLOBAL_API.runtime_reorder_slot(device, slot_id, new_priority)
}

/// Sets whether a profile slot is active for a device.
#[tracing::instrument(skip(device, slot_id))]
pub fn runtime_set_slot_active(
    device: DeviceInstanceId,
    slot_id: String,
    active: bool,
) -> anyhow::Result<RuntimeConfig> {
    GLOBAL_API.runtime_set_slot_active(device, slot_id, active)
}

// Observability API

/// Initialize the logger.
///
/// This should be called once at application startup.
pub fn init_logger() -> anyhow::Result<()> {
    StructuredLogger::new()
        .with_format(OutputFormat::Pretty)
        .init()
        .map_err(|e| anyhow::anyhow!("Failed to initialize logger: {}", e))
}

/// Create a log stream for receiving real-time logs from Rust.
///
/// The provided `callback` will receive all log entries generated by the Rust core.
/// This is intended to be used with Flutter Rust Bridge's callback capability.
pub fn create_log_stream(
    callback: impl Fn(LogEntry) + Send + Sync + 'static,
) -> anyhow::Result<()> {
    GLOBAL_LOG_BRIDGE.set_rust_callback(Box::new(callback));
    tracing::info!(
        service = "keyrx",
        component = "api",
        "Log stream established via FRB"
    );
    Ok(())
}
