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
/// ```ignore
/// let api = ApiContext::with_defaults();
/// let devices = api.list_devices().await?;
/// ```
///
/// # Testing Usage
/// ```ignore
/// let mock_device = Arc::new(MockDeviceService::new());
/// let mock_profile = Arc::new(MockProfileService::new());
/// let mock_runtime = Arc::new(MockRuntimeService::new());
/// let api = ApiContext::new(mock_device, mock_profile, mock_runtime);
/// ```
pub struct ApiContext {
    device_service: Arc<dyn DeviceServiceTrait>,
    // TODO: Remove #[allow(dead_code)] after implementing profile methods in task 4.3
    #[allow(dead_code)]
    profile_service: Arc<dyn ProfileServiceTrait>,
    // TODO: Remove #[allow(dead_code)] after implementing runtime methods in task 4.4
    #[allow(dead_code)]
    runtime_service: Arc<dyn RuntimeServiceTrait>,
}

impl ApiContext {
    /// Creates a new `ApiContext` with injected service dependencies.
    ///
    /// This constructor enables dependency injection for testing and custom configurations.
    ///
    /// # Arguments
    /// * `device_service` - Implementation of `DeviceServiceTrait`
    /// * `profile_service` - Implementation of `ProfileServiceTrait`
    /// * `runtime_service` - Implementation of `RuntimeServiceTrait`
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
    /// suitable for production use.
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
}

// Global services
lazy_static! {
    static ref DEVICE_SERVICE: DeviceService = DeviceService::with_defaults(None); // TODO: Hook up live registry
    static ref PROFILE_SERVICE: ProfileService = ProfileService::with_defaults();
    static ref RUNTIME_SERVICE: RuntimeService = RuntimeService::with_defaults();
}

// Device Service API
#[tracing::instrument]
pub async fn list_devices() -> anyhow::Result<Vec<DeviceView>> {
    DEVICE_SERVICE.list_devices().await.map_err(Into::into)
}

#[tracing::instrument(skip(device_key))]
pub async fn get_device(device_key: String) -> anyhow::Result<DeviceView> {
    DEVICE_SERVICE
        .get_device(&device_key)
        .await
        .map_err(Into::into)
}

#[tracing::instrument(skip(device_key))]
pub async fn set_device_remap(device_key: String, enabled: bool) -> anyhow::Result<DeviceView> {
    DEVICE_SERVICE
        .set_remap_enabled(&device_key, enabled)
        .await
        .map_err(Into::into)
}

#[tracing::instrument(skip(device_key, profile_id))]
pub async fn assign_device_profile(
    device_key: String,
    profile_id: String,
) -> anyhow::Result<DeviceView> {
    DEVICE_SERVICE
        .assign_profile(&device_key, &profile_id)
        .await
        .map_err(Into::into)
}

#[tracing::instrument(skip(device_key))]
pub async fn unassign_device_profile(device_key: String) -> anyhow::Result<DeviceView> {
    DEVICE_SERVICE
        .unassign_profile(&device_key)
        .await
        .map_err(Into::into)
}

#[tracing::instrument(skip(device_key))]
pub async fn set_device_label(
    device_key: String,
    label: Option<String>,
) -> anyhow::Result<DeviceView> {
    DEVICE_SERVICE
        .set_label(&device_key, label)
        .await
        .map_err(Into::into)
}

// Profile Service API
#[tracing::instrument]
pub fn list_virtual_layouts() -> anyhow::Result<Vec<VirtualLayout>> {
    PROFILE_SERVICE.list_virtual_layouts().map_err(Into::into)
}

#[tracing::instrument(skip(layout))]
pub fn save_virtual_layout(layout: VirtualLayout) -> anyhow::Result<VirtualLayout> {
    PROFILE_SERVICE
        .save_virtual_layout(layout)
        .map_err(Into::into)
}

#[tracing::instrument(skip(id))]
pub fn delete_virtual_layout(id: String) -> anyhow::Result<()> {
    PROFILE_SERVICE
        .delete_virtual_layout(&id)
        .map_err(Into::into)
}

#[tracing::instrument]
pub fn list_hardware_profiles() -> anyhow::Result<Vec<HardwareProfile>> {
    PROFILE_SERVICE.list_hardware_profiles().map_err(Into::into)
}

#[tracing::instrument(skip(profile))]
pub fn save_hardware_profile(profile: HardwareProfile) -> anyhow::Result<HardwareProfile> {
    PROFILE_SERVICE
        .save_hardware_profile(profile)
        .map_err(Into::into)
}

#[tracing::instrument(skip(id))]
pub fn delete_hardware_profile(id: String) -> anyhow::Result<()> {
    PROFILE_SERVICE
        .delete_hardware_profile(&id)
        .map_err(Into::into)
}

#[tracing::instrument]
pub fn list_keymaps() -> anyhow::Result<Vec<Keymap>> {
    PROFILE_SERVICE.list_keymaps().map_err(Into::into)
}

#[tracing::instrument(skip(keymap))]
pub fn save_keymap(keymap: Keymap) -> anyhow::Result<Keymap> {
    PROFILE_SERVICE.save_keymap(keymap).map_err(Into::into)
}

#[tracing::instrument(skip(id))]
pub fn delete_keymap(id: String) -> anyhow::Result<()> {
    PROFILE_SERVICE.delete_keymap(&id).map_err(Into::into)
}

// Runtime Service API
#[tracing::instrument]
pub fn get_runtime_config() -> anyhow::Result<RuntimeConfig> {
    RUNTIME_SERVICE.get_config().map_err(Into::into)
}

#[tracing::instrument(skip(device, slot))]
pub fn runtime_add_slot(
    device: DeviceInstanceId,
    slot: ProfileSlot,
) -> anyhow::Result<RuntimeConfig> {
    RUNTIME_SERVICE.add_slot(device, slot).map_err(Into::into)
}

#[tracing::instrument(skip(device, slot_id))]
pub fn runtime_remove_slot(
    device: DeviceInstanceId,
    slot_id: String,
) -> anyhow::Result<RuntimeConfig> {
    RUNTIME_SERVICE
        .remove_slot(device, &slot_id)
        .map_err(Into::into)
}

#[tracing::instrument(skip(device, slot_id))]
pub fn runtime_reorder_slot(
    device: DeviceInstanceId,
    slot_id: String,
    new_priority: u32,
) -> anyhow::Result<RuntimeConfig> {
    RUNTIME_SERVICE
        .reorder_slot(device, &slot_id, new_priority)
        .map_err(Into::into)
}

#[tracing::instrument(skip(device, slot_id))]
pub fn runtime_set_slot_active(
    device: DeviceInstanceId,
    slot_id: String,
    active: bool,
) -> anyhow::Result<RuntimeConfig> {
    RUNTIME_SERVICE
        .set_slot_active(device, &slot_id, active)
        .map_err(Into::into)
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
