//! Mock implementation of DeviceServiceTrait for testing.

// Allow unwrap on mutex locks in mocks - poison panic is acceptable in test infrastructure
#![allow(clippy::unwrap_used)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::services::device::{DeviceServiceError, DeviceView};
use crate::services::traits::DeviceServiceTrait;

/// Mock implementation of [`DeviceServiceTrait`] for testing.
///
/// Provides configurable responses and call tracking for all device operations.
/// All operations are pure in-memory with no I/O.
///
/// # Example
///
/// ```rust,ignore
/// let mock = MockDeviceService::new()
///     .with_devices(vec![
///         DeviceView { key: "device1".into(), ..Default::default() },
///         DeviceView { key: "device2".into(), ..Default::default() },
///     ]);
///
/// // list_devices() will return the configured devices
/// let devices = mock.list_devices().await.unwrap();
/// assert_eq!(devices.len(), 2);
///
/// // get_device() will find by key
/// let device = mock.get_device("device1").await.unwrap();
/// assert_eq!(device.key, "device1");
///
/// // get_device() returns DeviceNotFound for unknown keys
/// let result = mock.get_device("unknown").await;
/// assert!(result.is_err());
/// ```
pub struct MockDeviceService {
    /// Devices to return from list_devices and get_device
    devices: Vec<DeviceView>,
    /// Error to return from list_devices
    list_error: Option<DeviceServiceError>,
    /// Error to return from get_device
    get_error: Option<DeviceServiceError>,
    /// Error to return from set_remap_enabled
    set_remap_error: Option<DeviceServiceError>,
    /// Error to return from assign_profile
    assign_error: Option<DeviceServiceError>,
    /// Error to return from unassign_profile
    unassign_error: Option<DeviceServiceError>,
    /// Error to return from set_label
    set_label_error: Option<DeviceServiceError>,
    /// Tracks method call counts for verification
    call_counts: Arc<Mutex<HashMap<String, usize>>>,
}

impl MockDeviceService {
    /// Creates a new empty MockDeviceService.
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            list_error: None,
            get_error: None,
            set_remap_error: None,
            assign_error: None,
            unassign_error: None,
            set_label_error: None,
            call_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Configures the devices to return from list_devices and get_device.
    pub fn with_devices(mut self, devices: Vec<DeviceView>) -> Self {
        self.devices = devices;
        self
    }

    /// Configures an error to return from list_devices.
    pub fn with_list_error(mut self, error: DeviceServiceError) -> Self {
        self.list_error = Some(error);
        self
    }

    /// Configures an error to return from get_device.
    pub fn with_get_error(mut self, error: DeviceServiceError) -> Self {
        self.get_error = Some(error);
        self
    }

    /// Configures an error to return from set_remap_enabled.
    pub fn with_set_remap_error(mut self, error: DeviceServiceError) -> Self {
        self.set_remap_error = Some(error);
        self
    }

    /// Configures an error to return from assign_profile.
    pub fn with_assign_error(mut self, error: DeviceServiceError) -> Self {
        self.assign_error = Some(error);
        self
    }

    /// Configures an error to return from unassign_profile.
    pub fn with_unassign_error(mut self, error: DeviceServiceError) -> Self {
        self.unassign_error = Some(error);
        self
    }

    /// Configures an error to return from set_label.
    pub fn with_set_label_error(mut self, error: DeviceServiceError) -> Self {
        self.set_label_error = Some(error);
        self
    }

    /// Returns the number of times a method was called.
    pub fn get_call_count(&self, method: &str) -> usize {
        self.call_counts
            .lock()
            .unwrap()
            .get(method)
            .copied()
            .unwrap_or(0)
    }

    fn increment_call(&self, method: &str) {
        let mut counts = self.call_counts.lock().unwrap();
        *counts.entry(method.to_string()).or_insert(0) += 1;
    }

    fn find_device(&self, device_key: &str) -> Option<DeviceView> {
        self.devices.iter().find(|d| d.key == device_key).cloned()
    }
}

impl Default for MockDeviceService {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to create IO errors from strings (since DeviceServiceError variants aren't Clone)
fn make_io_error(msg: &str) -> DeviceServiceError {
    DeviceServiceError::Io(std::io::Error::other(msg.to_string()))
}

#[async_trait]
impl DeviceServiceTrait for MockDeviceService {
    async fn list_devices(&self) -> Result<Vec<DeviceView>, DeviceServiceError> {
        self.increment_call("list_devices");
        if let Some(ref error) = self.list_error {
            return Err(make_io_error(&error.to_string()));
        }
        Ok(self.devices.clone())
    }

    async fn get_device(&self, device_key: &str) -> Result<DeviceView, DeviceServiceError> {
        self.increment_call("get_device");
        if let Some(ref error) = self.get_error {
            return Err(make_io_error(&error.to_string()));
        }
        self.find_device(device_key)
            .ok_or_else(|| DeviceServiceError::DeviceNotFound(device_key.to_string()))
    }

    async fn set_remap_enabled(
        &self,
        device_key: &str,
        _enabled: bool,
    ) -> Result<DeviceView, DeviceServiceError> {
        self.increment_call("set_remap_enabled");
        if let Some(ref error) = self.set_remap_error {
            return Err(make_io_error(&error.to_string()));
        }
        self.find_device(device_key)
            .ok_or_else(|| DeviceServiceError::DeviceNotFound(device_key.to_string()))
    }

    async fn assign_profile(
        &self,
        device_key: &str,
        _profile_id: &str,
    ) -> Result<DeviceView, DeviceServiceError> {
        self.increment_call("assign_profile");
        if let Some(ref error) = self.assign_error {
            return Err(make_io_error(&error.to_string()));
        }
        self.find_device(device_key)
            .ok_or_else(|| DeviceServiceError::DeviceNotFound(device_key.to_string()))
    }

    async fn unassign_profile(&self, device_key: &str) -> Result<DeviceView, DeviceServiceError> {
        self.increment_call("unassign_profile");
        if let Some(ref error) = self.unassign_error {
            return Err(make_io_error(&error.to_string()));
        }
        self.find_device(device_key)
            .ok_or_else(|| DeviceServiceError::DeviceNotFound(device_key.to_string()))
    }

    async fn set_label(
        &self,
        device_key: &str,
        _label: Option<String>,
    ) -> Result<DeviceView, DeviceServiceError> {
        self.increment_call("set_label");
        if let Some(ref error) = self.set_label_error {
            return Err(make_io_error(&error.to_string()));
        }
        self.find_device(device_key)
            .ok_or_else(|| DeviceServiceError::DeviceNotFound(device_key.to_string()))
    }
}
