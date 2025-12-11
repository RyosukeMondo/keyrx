//! Service trait abstractions for dependency injection.
//!
//! This module defines trait contracts for all services, enabling:
//! - Dependency injection for flexible service composition
//! - Mock implementations for fast, isolated unit testing
//! - Clear API contracts between service implementations

use async_trait::async_trait;

use super::device::{DeviceServiceError, DeviceView};

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
