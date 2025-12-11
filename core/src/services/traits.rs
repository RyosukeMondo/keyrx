//! Service trait abstractions for dependency injection.
//!
//! This module defines trait contracts for all services, enabling:
//! - Dependency injection for flexible service composition
//! - Mock implementations for fast, isolated unit testing
//! - Clear API contracts between service implementations

use async_trait::async_trait;

use crate::config::models::{HardwareProfile, Keymap, VirtualLayout};

use super::device::{DeviceServiceError, DeviceView};
use super::profile::ProfileServiceError;

/// Trait defining the contract for device service operations.
///
/// This trait abstracts device management operations, allowing for:
/// - Real implementations that interact with hardware and persistence
/// - Mock implementations for testing without I/O
///
/// All methods are async and the trait requires `Send + Sync` for thread-safe usage.
#[async_trait]
pub trait DeviceServiceTrait: Send + Sync {
    /// Lists all known devices (both connected and previously bound).
    ///
    /// Returns a combined view of:
    /// - Live devices from the device registry (if runtime is active)
    /// - Persisted device bindings (for disconnected devices)
    ///
    /// # Returns
    /// A vector of `DeviceView` sorted by device key.
    ///
    /// # Errors
    /// Returns `DeviceServiceError` if loading bindings fails.
    async fn list_devices(&self) -> Result<Vec<DeviceView>, DeviceServiceError>;

    /// Retrieves a specific device by its key.
    ///
    /// # Arguments
    /// * `device_key` - The unique device identifier (format: `vendor_id:product_id:serial`)
    ///
    /// # Returns
    /// The `DeviceView` for the requested device. If the device has no binding
    /// and is not connected, returns an empty view with default values.
    ///
    /// # Errors
    /// Returns `DeviceServiceError::DeviceNotFound` if the key format is invalid.
    async fn get_device(&self, device_key: &str) -> Result<DeviceView, DeviceServiceError>;

    /// Enables or disables key remapping for a device.
    ///
    /// Updates both the live registry (if device is connected) and persisted bindings.
    ///
    /// # Arguments
    /// * `device_key` - The unique device identifier
    /// * `enabled` - Whether remapping should be enabled
    ///
    /// # Returns
    /// The updated `DeviceView` reflecting the new remap state.
    ///
    /// # Errors
    /// Returns `DeviceServiceError` if the key is invalid or persistence fails.
    async fn set_remap_enabled(
        &self,
        device_key: &str,
        enabled: bool,
    ) -> Result<DeviceView, DeviceServiceError>;

    /// Assigns a profile to a device.
    ///
    /// Updates both the live registry (if device is connected) and persisted bindings.
    ///
    /// # Arguments
    /// * `device_key` - The unique device identifier
    /// * `profile_id` - The ID of the profile to assign
    ///
    /// # Returns
    /// The updated `DeviceView` with the assigned profile.
    ///
    /// # Errors
    /// Returns `DeviceServiceError` if the key is invalid or persistence fails.
    async fn assign_profile(
        &self,
        device_key: &str,
        profile_id: &str,
    ) -> Result<DeviceView, DeviceServiceError>;

    /// Removes the profile assignment from a device.
    ///
    /// Updates both the live registry (if device is connected) and persisted bindings.
    ///
    /// # Arguments
    /// * `device_key` - The unique device identifier
    ///
    /// # Returns
    /// The updated `DeviceView` with no profile assigned.
    ///
    /// # Errors
    /// Returns `DeviceServiceError` if the key is invalid or persistence fails.
    async fn unassign_profile(&self, device_key: &str) -> Result<DeviceView, DeviceServiceError>;

    /// Sets or clears the user-defined label for a device.
    ///
    /// Updates both the live registry (if device is connected) and persisted bindings.
    ///
    /// # Arguments
    /// * `device_key` - The unique device identifier
    /// * `label` - The label to set, or `None` to clear the existing label
    ///
    /// # Returns
    /// The updated `DeviceView` with the new label.
    ///
    /// # Errors
    /// Returns `DeviceServiceError` if the key is invalid or persistence fails.
    async fn set_label(
        &self,
        device_key: &str,
        label: Option<String>,
    ) -> Result<DeviceView, DeviceServiceError>;
}

/// Trait defining the contract for profile service operations.
///
/// This trait abstracts profile management operations for:
/// - Virtual layouts (keyboard layout definitions)
/// - Hardware profiles (device-specific configurations)
/// - Keymaps (key remapping definitions)
///
/// All methods are synchronous. The trait requires `Send + Sync` for thread-safe usage.
pub trait ProfileServiceTrait: Send + Sync {
    /// Lists all virtual layouts.
    ///
    /// # Returns
    /// A vector of all stored `VirtualLayout` configurations.
    ///
    /// # Errors
    /// Returns `ProfileServiceError::Storage` if loading fails.
    fn list_virtual_layouts(&self) -> Result<Vec<VirtualLayout>, ProfileServiceError>;

    /// Saves a virtual layout.
    ///
    /// Creates a new layout or updates an existing one with the same ID.
    ///
    /// # Arguments
    /// * `layout` - The virtual layout to save
    ///
    /// # Returns
    /// The saved `VirtualLayout`.
    ///
    /// # Errors
    /// Returns `ProfileServiceError::Storage` if saving fails.
    fn save_virtual_layout(
        &self,
        layout: VirtualLayout,
    ) -> Result<VirtualLayout, ProfileServiceError>;

    /// Deletes a virtual layout by ID.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the layout to delete
    ///
    /// # Errors
    /// Returns `ProfileServiceError::Storage` if deletion fails.
    fn delete_virtual_layout(&self, id: &str) -> Result<(), ProfileServiceError>;

    /// Lists all hardware profiles.
    ///
    /// # Returns
    /// A vector of all stored `HardwareProfile` configurations.
    ///
    /// # Errors
    /// Returns `ProfileServiceError::Storage` if loading fails.
    fn list_hardware_profiles(&self) -> Result<Vec<HardwareProfile>, ProfileServiceError>;

    /// Saves a hardware profile.
    ///
    /// Creates a new profile or updates an existing one with the same ID.
    ///
    /// # Arguments
    /// * `profile` - The hardware profile to save
    ///
    /// # Returns
    /// The saved `HardwareProfile`.
    ///
    /// # Errors
    /// Returns `ProfileServiceError::Storage` if saving fails.
    fn save_hardware_profile(
        &self,
        profile: HardwareProfile,
    ) -> Result<HardwareProfile, ProfileServiceError>;

    /// Deletes a hardware profile by ID.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the profile to delete
    ///
    /// # Errors
    /// Returns `ProfileServiceError::Storage` if deletion fails.
    fn delete_hardware_profile(&self, id: &str) -> Result<(), ProfileServiceError>;

    /// Lists all keymaps.
    ///
    /// # Returns
    /// A vector of all stored `Keymap` configurations.
    ///
    /// # Errors
    /// Returns `ProfileServiceError::Storage` if loading fails.
    fn list_keymaps(&self) -> Result<Vec<Keymap>, ProfileServiceError>;

    /// Saves a keymap.
    ///
    /// Creates a new keymap or updates an existing one with the same ID.
    ///
    /// # Arguments
    /// * `keymap` - The keymap to save
    ///
    /// # Returns
    /// The saved `Keymap`.
    ///
    /// # Errors
    /// Returns `ProfileServiceError::Storage` if saving fails.
    fn save_keymap(&self, keymap: Keymap) -> Result<Keymap, ProfileServiceError>;

    /// Deletes a keymap by ID.
    ///
    /// # Arguments
    /// * `id` - The unique identifier of the keymap to delete
    ///
    /// # Errors
    /// Returns `ProfileServiceError::Storage` if deletion fails.
    fn delete_keymap(&self, id: &str) -> Result<(), ProfileServiceError>;
}
