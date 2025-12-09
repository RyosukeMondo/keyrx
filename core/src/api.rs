use crate::config::models::{
    DeviceInstanceId, HardwareProfile, Keymap, ProfileSlot, RuntimeConfig, VirtualLayout,
};
use crate::observability::bridge::GLOBAL_LOG_BRIDGE;
use crate::observability::entry::LogEntry;
use crate::observability::logger::{OutputFormat, StructuredLogger};
use crate::services::device::DeviceView;
use crate::services::{DeviceService, ProfileService, RuntimeService};
use lazy_static::lazy_static;

// Global services
lazy_static! {
    static ref DEVICE_SERVICE: DeviceService = DeviceService::new(None); // TODO: Hook up live registry
    static ref PROFILE_SERVICE: ProfileService = ProfileService::new();
    static ref RUNTIME_SERVICE: RuntimeService = RuntimeService::new();
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
