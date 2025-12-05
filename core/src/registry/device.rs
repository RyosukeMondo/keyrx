//! Device registry for runtime device state management.
//!
//! This module provides the DeviceRegistry which tracks connected devices,
//! their remap state, assigned profiles, and user labels. It uses thread-safe
//! data structures for concurrent access and emits events for state changes.

use crate::identity::DeviceIdentity;
use crate::registry::ProfileId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Runtime state of a connected device
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceState {
    /// Device identity
    pub identity: DeviceIdentity,

    /// Whether remapping is enabled for this device
    pub remap_enabled: bool,

    /// Assigned profile ID (if any)
    pub profile_id: Option<ProfileId>,

    /// Connection timestamp (ISO 8601)
    pub connected_at: String,

    /// Last update timestamp (ISO 8601)
    pub updated_at: String,
}

impl DeviceState {
    /// Create a new device state with remap disabled
    pub fn new(identity: DeviceIdentity) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            identity,
            remap_enabled: false,
            profile_id: None,
            connected_at: now.clone(),
            updated_at: now,
        }
    }

    /// Update the updated_at timestamp to current time
    pub fn touch(&mut self) {
        self.updated_at = chrono::Utc::now().to_rfc3339();
    }
}

/// Events emitted by the DeviceRegistry
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceEvent {
    /// A device was registered
    Registered { identity: DeviceIdentity },

    /// A device was unregistered
    Unregistered { identity: DeviceIdentity },

    /// Remap state changed for a device
    RemapStateChanged {
        identity: DeviceIdentity,
        enabled: bool,
    },

    /// Profile assigned to a device
    ProfileAssigned {
        identity: DeviceIdentity,
        profile_id: ProfileId,
    },

    /// Profile unassigned from a device
    ProfileUnassigned { identity: DeviceIdentity },

    /// User label changed for a device
    LabelChanged {
        identity: DeviceIdentity,
        label: Option<String>,
    },
}

/// Thread-safe registry for managing runtime device state
///
/// The DeviceRegistry tracks all connected devices and their configuration,
/// including remap state, assigned profiles, and user labels. All operations
/// are thread-safe and emit events for state changes.
#[derive(Debug, Clone)]
pub struct DeviceRegistry {
    /// Device states indexed by DeviceIdentity
    devices: Arc<RwLock<HashMap<DeviceIdentity, DeviceState>>>,

    /// Event channel for broadcasting state changes
    event_tx: mpsc::UnboundedSender<DeviceEvent>,
}

impl DeviceRegistry {
    /// Create a new DeviceRegistry with an event channel
    ///
    /// Returns the registry and a receiver for device events
    pub fn new() -> (Self, mpsc::UnboundedReceiver<DeviceEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let registry = Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        };

        (registry, event_rx)
    }

    /// Register a new device
    ///
    /// If the device is already registered, this is a no-op and returns false.
    /// Returns true if the device was newly registered.
    pub async fn register_device(&self, identity: DeviceIdentity) -> bool {
        let mut devices = self.devices.write().await;

        if devices.contains_key(&identity) {
            return false;
        }

        let state = DeviceState::new(identity.clone());
        devices.insert(identity.clone(), state);

        // Emit event (ignore send errors - no listeners is ok)
        let _ = self.event_tx.send(DeviceEvent::Registered {
            identity: identity.clone(),
        });

        true
    }

    /// Unregister a device
    ///
    /// Returns true if the device was removed, false if it wasn't registered.
    pub async fn unregister_device(&self, identity: &DeviceIdentity) -> bool {
        let mut devices = self.devices.write().await;

        if devices.remove(identity).is_some() {
            // Emit event
            let _ = self.event_tx.send(DeviceEvent::Unregistered {
                identity: identity.clone(),
            });
            true
        } else {
            false
        }
    }

    /// Set remap enabled state for a device
    ///
    /// Returns Ok(()) if successful, Err if device not found.
    pub async fn set_remap_enabled(
        &self,
        identity: &DeviceIdentity,
        enabled: bool,
    ) -> Result<(), DeviceRegistryError> {
        let mut devices = self.devices.write().await;

        let state = devices
            .get_mut(identity)
            .ok_or_else(|| DeviceRegistryError::DeviceNotFound(identity.clone()))?;

        if state.remap_enabled != enabled {
            state.remap_enabled = enabled;
            state.touch();

            // Emit event
            let _ = self.event_tx.send(DeviceEvent::RemapStateChanged {
                identity: identity.clone(),
                enabled,
            });
        }

        Ok(())
    }

    /// Assign a profile to a device
    ///
    /// Returns Ok(()) if successful, Err if device not found.
    pub async fn assign_profile(
        &self,
        identity: &DeviceIdentity,
        profile_id: ProfileId,
    ) -> Result<(), DeviceRegistryError> {
        let mut devices = self.devices.write().await;

        let state = devices
            .get_mut(identity)
            .ok_or_else(|| DeviceRegistryError::DeviceNotFound(identity.clone()))?;

        let changed = state.profile_id.as_ref() != Some(&profile_id);
        state.profile_id = Some(profile_id.clone());
        state.touch();

        if changed {
            // Emit event
            let _ = self.event_tx.send(DeviceEvent::ProfileAssigned {
                identity: identity.clone(),
                profile_id,
            });
        }

        Ok(())
    }

    /// Unassign the profile from a device
    ///
    /// Returns Ok(()) if successful, Err if device not found.
    pub async fn unassign_profile(
        &self,
        identity: &DeviceIdentity,
    ) -> Result<(), DeviceRegistryError> {
        let mut devices = self.devices.write().await;

        let state = devices
            .get_mut(identity)
            .ok_or_else(|| DeviceRegistryError::DeviceNotFound(identity.clone()))?;

        if state.profile_id.is_some() {
            state.profile_id = None;
            state.touch();

            // Emit event
            let _ = self.event_tx.send(DeviceEvent::ProfileUnassigned {
                identity: identity.clone(),
            });
        }

        Ok(())
    }

    /// Set user label for a device
    ///
    /// Updates the label in the device's identity. Pass None to clear the label.
    /// Returns Ok(()) if successful, Err if device not found.
    pub async fn set_user_label(
        &self,
        identity: &DeviceIdentity,
        label: Option<String>,
    ) -> Result<(), DeviceRegistryError> {
        let mut devices = self.devices.write().await;

        let state = devices
            .get_mut(identity)
            .ok_or_else(|| DeviceRegistryError::DeviceNotFound(identity.clone()))?;

        let changed = state.identity.user_label != label;
        state.identity.user_label = label.clone();
        state.touch();

        if changed {
            // Emit event
            let _ = self.event_tx.send(DeviceEvent::LabelChanged {
                identity: identity.clone(),
                label,
            });
        }

        Ok(())
    }

    /// Get the state of a specific device
    ///
    /// Returns None if the device is not registered.
    pub async fn get_device_state(&self, identity: &DeviceIdentity) -> Option<DeviceState> {
        let devices = self.devices.read().await;
        devices.get(identity).cloned()
    }

    /// List all registered devices
    ///
    /// Returns a vector of all device states, sorted by connection time.
    pub async fn list_devices(&self) -> Vec<DeviceState> {
        let devices = self.devices.read().await;
        let mut states: Vec<DeviceState> = devices.values().cloned().collect();

        // Sort by connected_at timestamp for consistent ordering
        states.sort_by(|a, b| a.connected_at.cmp(&b.connected_at));

        states
    }

    /// Get the number of registered devices
    pub async fn device_count(&self) -> usize {
        let devices = self.devices.read().await;
        devices.len()
    }

    /// Check if a device is registered
    pub async fn is_registered(&self, identity: &DeviceIdentity) -> bool {
        let devices = self.devices.read().await;
        devices.contains_key(identity)
    }
}

impl Default for DeviceRegistry {
    fn default() -> Self {
        Self::new().0
    }
}

/// Errors that can occur when interacting with the DeviceRegistry
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DeviceRegistryError {
    #[error("Device not found: {0}")]
    DeviceNotFound(DeviceIdentity),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_identity(serial: &str) -> DeviceIdentity {
        DeviceIdentity::new(0x1234, 0x5678, serial.to_string())
    }

    #[tokio::test]
    async fn test_register_device() {
        let (registry, mut rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");

        let result = registry.register_device(identity.clone()).await;
        assert!(result);

        // Check event
        let event = rx.recv().await.unwrap();
        assert_eq!(
            event,
            DeviceEvent::Registered {
                identity: identity.clone()
            }
        );

        // Verify device is registered
        assert!(registry.is_registered(&identity).await);
        assert_eq!(registry.device_count().await, 1);
    }

    #[tokio::test]
    async fn test_register_device_idempotent() {
        let (registry, _rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");

        let result1 = registry.register_device(identity.clone()).await;
        let result2 = registry.register_device(identity.clone()).await;

        assert!(result1);
        assert!(!result2); // Second registration should return false
        assert_eq!(registry.device_count().await, 1);
    }

    #[tokio::test]
    async fn test_unregister_device() {
        let (registry, mut rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");

        registry.register_device(identity.clone()).await;
        let _ = rx.recv().await; // Consume register event

        let result = registry.unregister_device(&identity).await;
        assert!(result);

        // Check event
        let event = rx.recv().await.unwrap();
        assert_eq!(
            event,
            DeviceEvent::Unregistered {
                identity: identity.clone()
            }
        );

        assert!(!registry.is_registered(&identity).await);
        assert_eq!(registry.device_count().await, 0);
    }

    #[tokio::test]
    async fn test_unregister_nonexistent_device() {
        let (registry, _rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");

        let result = registry.unregister_device(&identity).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn test_set_remap_enabled() {
        let (registry, mut rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");

        registry.register_device(identity.clone()).await;
        let _ = rx.recv().await; // Consume register event

        let result = registry.set_remap_enabled(&identity, true).await;
        assert!(result.is_ok());

        // Check event
        let event = rx.recv().await.unwrap();
        assert_eq!(
            event,
            DeviceEvent::RemapStateChanged {
                identity: identity.clone(),
                enabled: true
            }
        );

        // Verify state
        let state = registry.get_device_state(&identity).await.unwrap();
        assert!(state.remap_enabled);
    }

    #[tokio::test]
    async fn test_set_remap_enabled_no_change() {
        let (registry, mut rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");

        registry.register_device(identity.clone()).await;
        let _ = rx.recv().await; // Consume register event

        // Set to false (already false)
        let result = registry.set_remap_enabled(&identity, false).await;
        assert!(result.is_ok());

        // Should not emit event
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_set_remap_enabled_device_not_found() {
        let (registry, _rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");

        let result = registry.set_remap_enabled(&identity, true).await;
        assert!(matches!(
            result,
            Err(DeviceRegistryError::DeviceNotFound(_))
        ));
    }

    #[tokio::test]
    async fn test_assign_profile() {
        let (registry, mut rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");
        let profile_id = "profile-123".to_string();

        registry.register_device(identity.clone()).await;
        let _ = rx.recv().await; // Consume register event

        let result = registry.assign_profile(&identity, profile_id.clone()).await;
        assert!(result.is_ok());

        // Check event
        let event = rx.recv().await.unwrap();
        assert_eq!(
            event,
            DeviceEvent::ProfileAssigned {
                identity: identity.clone(),
                profile_id: profile_id.clone()
            }
        );

        // Verify state
        let state = registry.get_device_state(&identity).await.unwrap();
        assert_eq!(state.profile_id, Some(profile_id));
    }

    #[tokio::test]
    async fn test_unassign_profile() {
        let (registry, mut rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");
        let profile_id = "profile-123".to_string();

        registry.register_device(identity.clone()).await;
        let _ = rx.recv().await; // Consume register event

        registry
            .assign_profile(&identity, profile_id.clone())
            .await
            .unwrap();
        let _ = rx.recv().await; // Consume assign event

        let result = registry.unassign_profile(&identity).await;
        assert!(result.is_ok());

        // Check event
        let event = rx.recv().await.unwrap();
        assert_eq!(
            event,
            DeviceEvent::ProfileUnassigned {
                identity: identity.clone()
            }
        );

        // Verify state
        let state = registry.get_device_state(&identity).await.unwrap();
        assert_eq!(state.profile_id, None);
    }

    #[tokio::test]
    async fn test_set_user_label() {
        let (registry, mut rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");
        let label = Some("My Keyboard".to_string());

        registry.register_device(identity.clone()).await;
        let _ = rx.recv().await; // Consume register event

        let result = registry.set_user_label(&identity, label.clone()).await;
        assert!(result.is_ok());

        // Check event
        let event = rx.recv().await.unwrap();
        assert_eq!(
            event,
            DeviceEvent::LabelChanged {
                identity: identity.clone(),
                label: label.clone()
            }
        );

        // Verify state
        let state = registry.get_device_state(&identity).await.unwrap();
        assert_eq!(state.identity.user_label, label);
    }

    #[tokio::test]
    async fn test_list_devices() {
        let (registry, _rx) = DeviceRegistry::new();

        let id1 = test_identity("TEST001");
        let id2 = test_identity("TEST002");
        let id3 = test_identity("TEST003");

        registry.register_device(id1.clone()).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        registry.register_device(id2.clone()).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        registry.register_device(id3.clone()).await;

        let devices = registry.list_devices().await;
        assert_eq!(devices.len(), 3);

        // Should be sorted by connection time
        assert_eq!(devices[0].identity, id1);
        assert_eq!(devices[1].identity, id2);
        assert_eq!(devices[2].identity, id3);
    }

    #[tokio::test]
    async fn test_get_device_state() {
        let (registry, _rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");

        // Not registered yet
        assert!(registry.get_device_state(&identity).await.is_none());

        registry.register_device(identity.clone()).await;

        // Now should exist
        let state = registry.get_device_state(&identity).await.unwrap();
        assert_eq!(state.identity, identity);
        assert!(!state.remap_enabled);
        assert_eq!(state.profile_id, None);
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        let (registry, _rx) = DeviceRegistry::new();

        // Spawn multiple tasks that concurrently register devices
        let mut handles = vec![];

        for i in 0..10 {
            let reg = registry.clone();
            let handle = tokio::spawn(async move {
                let identity = test_identity(&format!("TEST{:03}", i));
                reg.register_device(identity).await
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // All 10 devices should be registered
        assert_eq!(registry.device_count().await, 10);
    }

    #[tokio::test]
    async fn test_device_state_timestamps() {
        let (registry, _rx) = DeviceRegistry::new();
        let identity = test_identity("TEST001");

        registry.register_device(identity.clone()).await;
        let state1 = registry.get_device_state(&identity).await.unwrap();

        // Sleep to ensure timestamp changes
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        registry.set_remap_enabled(&identity, true).await.unwrap();
        let state2 = registry.get_device_state(&identity).await.unwrap();

        // connected_at should not change
        assert_eq!(state1.connected_at, state2.connected_at);

        // updated_at should change
        assert_ne!(state1.updated_at, state2.updated_at);
    }
}
