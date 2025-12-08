use crate::services::{DeviceService, ProfileService, RuntimeService};
use crate::services::device::DeviceView;
use crate::config::models::{VirtualLayout, HardwareProfile, Keymap, RuntimeConfig, ProfileSlot, DeviceInstanceId};
use crate::registry::DeviceRegistry;
use std::sync::Arc;
use tokio::sync::Mutex;
use lazy_static::lazy_static;

// Global services
lazy_static! {
    static ref DEVICE_SERVICE: DeviceService = DeviceService::new(None); // TODO: Hook up live registry
    static ref PROFILE_SERVICE: ProfileService = ProfileService::new();
    static ref RUNTIME_SERVICE: RuntimeService = RuntimeService::new();
}

// Device Service API
pub async fn list_devices() -> anyhow::Result<Vec<DeviceView>> {
    DEVICE_SERVICE.list_devices().await.map_err(Into::into)
}

pub async fn get_device(device_key: String) -> anyhow::Result<DeviceView> {
    DEVICE_SERVICE.get_device(&device_key).await.map_err(Into::into)
}

pub async fn set_device_remap(device_key: String, enabled: bool) -> anyhow::Result<DeviceView> {
    DEVICE_SERVICE.set_remap_enabled(&device_key, enabled).await.map_err(Into::into)
}

pub async fn assign_device_profile(device_key: String, profile_id: String) -> anyhow::Result<DeviceView> {
    DEVICE_SERVICE.assign_profile(&device_key, &profile_id).await.map_err(Into::into)
}

pub async fn unassign_device_profile(device_key: String) -> anyhow::Result<DeviceView> {
    DEVICE_SERVICE.unassign_profile(&device_key).await.map_err(Into::into)
}

pub async fn set_device_label(device_key: String, label: Option<String>) -> anyhow::Result<DeviceView> {
    DEVICE_SERVICE.set_label(&device_key, label).await.map_err(Into::into)
}

// Profile Service API
pub fn list_virtual_layouts() -> anyhow::Result<Vec<VirtualLayout>> {
    PROFILE_SERVICE.list_virtual_layouts().map_err(Into::into)
}

pub fn save_virtual_layout(layout: VirtualLayout) -> anyhow::Result<VirtualLayout> {
    PROFILE_SERVICE.save_virtual_layout(layout).map_err(Into::into)
}

pub fn delete_virtual_layout(id: String) -> anyhow::Result<()> {
    PROFILE_SERVICE.delete_virtual_layout(&id).map_err(Into::into)
}

pub fn list_hardware_profiles() -> anyhow::Result<Vec<HardwareProfile>> {
    PROFILE_SERVICE.list_hardware_profiles().map_err(Into::into)
}

pub fn save_hardware_profile(profile: HardwareProfile) -> anyhow::Result<HardwareProfile> {
    PROFILE_SERVICE.save_hardware_profile(profile).map_err(Into::into)
}

pub fn delete_hardware_profile(id: String) -> anyhow::Result<()> {
    PROFILE_SERVICE.delete_hardware_profile(&id).map_err(Into::into)
}

pub fn list_keymaps() -> anyhow::Result<Vec<Keymap>> {
    PROFILE_SERVICE.list_keymaps().map_err(Into::into)
}

pub fn save_keymap(keymap: Keymap) -> anyhow::Result<Keymap> {
    PROFILE_SERVICE.save_keymap(keymap).map_err(Into::into)
}

pub fn delete_keymap(id: String) -> anyhow::Result<()> {
    PROFILE_SERVICE.delete_keymap(&id).map_err(Into::into)
}

// Runtime Service API
pub fn get_runtime_config() -> anyhow::Result<RuntimeConfig> {
    RUNTIME_SERVICE.get_config().map_err(Into::into)
}

pub fn runtime_add_slot(device: DeviceInstanceId, slot: ProfileSlot) -> anyhow::Result<RuntimeConfig> {
    RUNTIME_SERVICE.add_slot(device, slot).map_err(Into::into)
}

pub fn runtime_remove_slot(device: DeviceInstanceId, slot_id: String) -> anyhow::Result<RuntimeConfig> {
    RUNTIME_SERVICE.remove_slot(device, &slot_id).map_err(Into::into)
}

pub fn runtime_reorder_slot(device: DeviceInstanceId, slot_id: String, new_priority: u32) -> anyhow::Result<RuntimeConfig> {
    RUNTIME_SERVICE.reorder_slot(device, &slot_id, new_priority).map_err(Into::into)
}

pub fn runtime_set_slot_active(device: DeviceInstanceId, slot_id: String, active: bool) -> anyhow::Result<RuntimeConfig> {
    RUNTIME_SERVICE.set_slot_active(device, &slot_id, active).map_err(Into::into)
}
