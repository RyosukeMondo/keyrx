use crate::ffi::runtime::with_revolutionary_runtime;
use crate::registry::{DeviceBinding, DeviceBindings};
use crate::registry::{DeviceRegistry, DeviceRegistryError, DeviceState};
use crate::DeviceIdentity;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DeviceServiceError {
    #[error("Registry error: {0}")]
    Registry(#[from] DeviceRegistryError),
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// A service to manage devices.
/// It interacts with the live `DeviceRegistry` (if available) AND the persisted `DeviceBindings`.
/// It acts as the SSOT for device operations.
pub struct DeviceService {
    // Optional because the runtime might not be active (e.g. CLI in offline mode)
    registry: Option<DeviceRegistry>,
    bindings_path: PathBuf,
}

impl DeviceService {
    pub fn new(registry: Option<DeviceRegistry>) -> Self {
        Self {
            registry,
            bindings_path: DeviceBindings::default_path(),
        }
    }

    pub fn with_bindings_path(mut self, path: PathBuf) -> Self {
        self.bindings_path = path;
        self
    }

    fn load_bindings(&self) -> Result<DeviceBindings, DeviceServiceError> {
        let mut bindings = DeviceBindings::with_path(self.bindings_path.clone());
        if self.bindings_path.exists() {
            bindings
                .load()
                .map_err(|e| std::io::Error::other(e.to_string()))?;
        }
        Ok(bindings)
    }

    fn get_registry(&self) -> Option<DeviceRegistry> {
        if let Some(reg) = &self.registry {
            return Some(reg.clone());
        }

        // Try global runtime
        let mut registry = None;
        let _ = with_revolutionary_runtime(|rt| {
            registry = Some(rt.device_registry().clone());
            Ok(())
        });
        registry
    }

    pub async fn list_devices(&self) -> Result<Vec<DeviceView>, DeviceServiceError> {
        let bindings = self.load_bindings()?;
        let mut views = Vec::new();
        let mut bound_identities = std::collections::HashSet::new();

        // 1. Get live devices from registry
        if let Some(registry) = self.get_registry() {
            let states = registry.list_devices().await;
            for state in states {
                let binding = bindings.get_binding(&state.identity).cloned();
                bound_identities.insert(state.identity.clone());
                views.push(DeviceView::from_state(&state, binding));
            }
        }

        // 2. Add remaining persisted bindings that are not connected
        for (identity, binding) in bindings.all_bindings() {
            if !bound_identities.contains(identity) {
                views.push(DeviceView::from_binding(identity.clone(), binding.clone()));
            }
        }

        views.sort_by(|a, b| a.key.cmp(&b.key));
        Ok(views)
    }

    pub async fn get_device(&self, device_key: &str) -> Result<DeviceView, DeviceServiceError> {
        let identity =
            DeviceIdentity::from_key(device_key).map_err(DeviceServiceError::DeviceNotFound)?;

        let bindings = self.load_bindings()?;
        let binding = bindings.get_binding(&identity).cloned();

        if let Some(registry) = self.get_registry() {
            if let Some(state) = registry.get_device_state(&identity).await {
                return Ok(DeviceView::from_state(&state, binding));
            }
        }

        if let Some(binding) = binding {
            Ok(DeviceView::from_binding(identity, binding))
        } else {
            Ok(DeviceView::empty(identity))
        }
    }

    pub async fn set_remap_enabled(
        &self,
        device_key: &str,
        enabled: bool,
    ) -> Result<DeviceView, DeviceServiceError> {
        let identity =
            DeviceIdentity::from_key(device_key).map_err(DeviceServiceError::DeviceNotFound)?;

        // 1. Update live registry
        if let Some(registry) = self.get_registry() {
            // We ignore error if device is not found in registry (not connected)
            let _ = registry.set_remap_enabled(&identity, enabled).await;
        }

        // 2. Update persistence
        let mut bindings = self.load_bindings()?;
        let mut binding = bindings
            .get_binding(&identity)
            .cloned()
            .unwrap_or_else(DeviceBinding::new);
        binding.remap_enabled = enabled;
        bindings.set_binding(identity.clone(), binding);
        bindings
            .save()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        self.get_device(device_key).await
    }

    pub async fn assign_profile(
        &self,
        device_key: &str,
        profile_id: &str,
    ) -> Result<DeviceView, DeviceServiceError> {
        let identity =
            DeviceIdentity::from_key(device_key).map_err(DeviceServiceError::DeviceNotFound)?;

        // 1. Update live registry
        if let Some(registry) = self.get_registry() {
            let _ = registry
                .assign_profile(&identity, profile_id.to_string())
                .await;
        }

        // 2. Update persistence
        let mut bindings = self.load_bindings()?;
        let mut binding = bindings
            .get_binding(&identity)
            .cloned()
            .unwrap_or_else(DeviceBinding::new);
        binding.profile_id = Some(profile_id.to_string());
        bindings.set_binding(identity.clone(), binding);
        bindings
            .save()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        self.get_device(device_key).await
    }

    pub async fn unassign_profile(
        &self,
        device_key: &str,
    ) -> Result<DeviceView, DeviceServiceError> {
        let identity =
            DeviceIdentity::from_key(device_key).map_err(DeviceServiceError::DeviceNotFound)?;

        // 1. Update live registry
        if let Some(registry) = self.get_registry() {
            let _ = registry.unassign_profile(&identity).await;
        }

        // 2. Update persistence
        let mut bindings = self.load_bindings()?;
        let mut binding = bindings
            .get_binding(&identity)
            .cloned()
            .unwrap_or_else(DeviceBinding::new);
        binding.profile_id = None;
        bindings.set_binding(identity.clone(), binding);
        bindings
            .save()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        self.get_device(device_key).await
    }

    pub async fn set_label(
        &self,
        device_key: &str,
        label: Option<String>,
    ) -> Result<DeviceView, DeviceServiceError> {
        let identity =
            DeviceIdentity::from_key(device_key).map_err(DeviceServiceError::DeviceNotFound)?;

        // 1. Update live registry
        if let Some(registry) = self.get_registry() {
            let _ = registry.set_user_label(&identity, label.clone()).await;
        }

        // 2. Update persistence
        let mut bindings = self.load_bindings()?;
        let mut binding = bindings
            .get_binding(&identity)
            .cloned()
            .unwrap_or_else(DeviceBinding::new);
        binding.user_label = label;
        bindings.set_binding(identity.clone(), binding);
        bindings
            .save()
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        self.get_device(device_key).await
    }
}

/// Unified view of a device (state + config).
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeviceView {
    pub key: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: String,
    pub label: Option<String>,
    pub remap_enabled: bool,
    pub profile_id: Option<String>,
    pub connected: bool,
}

impl DeviceView {
    fn from_state(state: &DeviceState, binding: Option<DeviceBinding>) -> Self {
        let fallback_label = binding.and_then(|b| b.user_label);
        Self {
            key: state.identity.to_key(),
            vendor_id: state.identity.vendor_id,
            product_id: state.identity.product_id,
            serial_number: state.identity.serial_number.clone(),
            label: state.identity.user_label.clone().or(fallback_label),
            remap_enabled: state.remap_enabled,
            profile_id: state.profile_id.clone(),
            connected: true,
        }
    }

    fn from_binding(identity: DeviceIdentity, binding: DeviceBinding) -> Self {
        Self {
            key: identity.to_key(),
            vendor_id: identity.vendor_id,
            product_id: identity.product_id,
            serial_number: identity.serial_number,
            label: binding.user_label,
            remap_enabled: binding.remap_enabled,
            profile_id: binding.profile_id,
            connected: false,
        }
    }

    fn empty(identity: DeviceIdentity) -> Self {
        let binding = DeviceBinding::new();
        Self::from_binding(identity, binding)
    }
}
