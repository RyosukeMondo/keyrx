//! Device resolution for input events.
//!
//! The DeviceResolver extracts device identity from input events and looks up
//! the corresponding DeviceState from the DeviceRegistry. This enables
//! per-device configuration and profile assignment in the revolutionary mapping
//! pipeline.

use crate::engine::InputEvent;
use crate::identity::DeviceIdentity;
use crate::registry::{DeviceRegistry, DeviceState};

/// Error type for device resolution
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DeviceResolverError {
    #[error("Failed to extract identity from event: {0}")]
    IdentityExtractionFailed(String),

    #[error("Device not registered: {0}")]
    DeviceNotRegistered(DeviceIdentity),
}

/// Resolves input events to device state.
///
/// The DeviceResolver extracts device identity from input events using
/// platform-specific metadata (device_id and serial_number) and looks up
/// the corresponding DeviceState from the DeviceRegistry.
///
/// This is a critical component of the revolutionary mapping pipeline,
/// with a latency target of <50μs for resolution operations.
#[derive(Debug, Clone)]
pub struct DeviceResolver {
    /// Reference to the device registry
    registry: DeviceRegistry,
}

impl DeviceResolver {
    /// Create a new DeviceResolver with a reference to the registry.
    pub fn new(registry: DeviceRegistry) -> Self {
        Self { registry }
    }

    /// Resolve an input event to its device state.
    ///
    /// This method:
    /// 1. Extracts device identity from the event metadata
    /// 2. Looks up the device in the registry
    /// 3. Returns the device state if found
    ///
    /// Returns None if the device is not registered (e.g., remapping disabled
    /// for this device or device not yet registered). Returns an error if
    /// identity extraction fails.
    ///
    /// # Latency Target
    /// This method must complete in <50μs to meet pipeline requirements.
    pub async fn resolve(
        &self,
        event: &InputEvent,
    ) -> Result<Option<DeviceState>, DeviceResolverError> {
        // Extract identity from event metadata
        let identity = match self.extract_identity(event) {
            Some(id) => id,
            None => {
                // If we can't extract identity, the device might not have proper metadata
                // This is not an error - just means we can't look it up
                return Ok(None);
            }
        };

        // Look up device state in registry
        // This is an O(1) HashMap lookup with RwLock read lock
        let state = self.registry.get_device_state(&identity).await;

        Ok(state)
    }

    /// Extract device identity from an input event.
    ///
    /// This uses the event's metadata fields:
    /// - vendor_id: USB Vendor ID
    /// - product_id: USB Product ID
    /// - serial_number: Device serial number or synthetic ID
    ///
    /// Returns None if the event doesn't have sufficient metadata to
    /// construct a device identity (all three fields required).
    fn extract_identity(&self, event: &InputEvent) -> Option<DeviceIdentity> {
        // All three fields are required for DeviceIdentity
        let vendor_id = event.vendor_id?;
        let product_id = event.product_id?;
        let serial_number = event.serial_number.as_ref()?;

        Some(DeviceIdentity::new(
            vendor_id,
            product_id,
            serial_number.clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::KeyCode;

    fn create_test_identity() -> DeviceIdentity {
        DeviceIdentity::new(0x046D, 0xC52B, "TEST123".to_string())
    }

    fn create_test_event_with_identity() -> InputEvent {
        InputEvent::with_full_identity(
            KeyCode::A,
            true,
            1000,
            Some("/dev/input/event0".to_string()),
            false,
            false,
            30,
            Some("TEST123".to_string()),
            Some(0x046D),
            Some(0xC52B),
        )
    }

    fn create_test_event_without_identity() -> InputEvent {
        InputEvent::key_down(KeyCode::A, 1000)
    }

    #[tokio::test]
    async fn test_resolve_registered_device() {
        let (registry, _rx) = DeviceRegistry::new();
        let resolver = DeviceResolver::new(registry.clone());

        // Register a device
        let identity = create_test_identity();
        registry.register_device(identity.clone()).await;

        // Create event from that device
        let event = create_test_event_with_identity();

        // Resolve should find the device
        let result = resolver.resolve(&event).await;
        assert!(result.is_ok());

        let state = result.unwrap();
        assert!(state.is_some());
        assert_eq!(state.unwrap().identity, identity);
    }

    #[tokio::test]
    async fn test_resolve_unregistered_device() {
        let (registry, _rx) = DeviceRegistry::new();
        let resolver = DeviceResolver::new(registry);

        // Event with identity but device not registered
        let event = create_test_event_with_identity();
        let result = resolver.resolve(&event).await;

        // Should return Ok(None) for unregistered device
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_resolve_no_metadata() {
        let (registry, _rx) = DeviceRegistry::new();
        let resolver = DeviceResolver::new(registry);

        let event = create_test_event_without_identity();

        let result = resolver.resolve(&event).await;

        // Should return Ok(None) when event has no metadata
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_resolve_partial_identity() {
        let (registry, _rx) = DeviceRegistry::new();
        let resolver = DeviceResolver::new(registry);

        // Event with serial but no VID:PID
        let event = InputEvent::with_metadata(
            KeyCode::A,
            true,
            1000,
            Some("/dev/input/event0".to_string()),
            false,
            false,
            30,
            Some("TEST123".to_string()),
        );

        let result = resolver.resolve(&event).await;

        // Should return Ok(None) when identity is incomplete
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_resolve_respects_remap_state() {
        let (registry, _rx) = DeviceRegistry::new();
        let resolver = DeviceResolver::new(registry.clone());

        // Register device and enable remapping
        let identity = create_test_identity();
        registry.register_device(identity.clone()).await;
        registry.set_remap_enabled(&identity, true).await.unwrap();

        // Resolve event
        let event = create_test_event_with_identity();
        let result = resolver.resolve(&event).await;

        assert!(result.is_ok());
        let state = result.unwrap();
        assert!(state.is_some());
        assert!(state.unwrap().remap_enabled);
    }

    #[tokio::test]
    async fn test_resolve_with_profile() {
        let (registry, _rx) = DeviceRegistry::new();
        let resolver = DeviceResolver::new(registry.clone());

        // Register device and assign profile
        let identity = create_test_identity();
        let profile_id = "test-profile".to_string();
        registry.register_device(identity.clone()).await;
        registry
            .assign_profile(&identity, profile_id.clone())
            .await
            .unwrap();

        // Resolve event
        let event = create_test_event_with_identity();
        let result = resolver.resolve(&event).await;

        assert!(result.is_ok());
        let state = result.unwrap();
        assert!(state.is_some());
        assert_eq!(state.unwrap().profile_id, Some(profile_id));
    }

    #[tokio::test]
    async fn test_extract_identity_complete() {
        let (registry, _rx) = DeviceRegistry::new();
        let resolver = DeviceResolver::new(registry);

        let event = create_test_event_with_identity();
        let identity = resolver.extract_identity(&event);

        assert!(identity.is_some());
        let id = identity.unwrap();
        assert_eq!(id.vendor_id, 0x046D);
        assert_eq!(id.product_id, 0xC52B);
        assert_eq!(id.serial_number, "TEST123");
    }

    #[tokio::test]
    async fn test_extract_identity_missing_fields() {
        let (registry, _rx) = DeviceRegistry::new();
        let resolver = DeviceResolver::new(registry);

        // Missing VID:PID
        let event1 = InputEvent::with_metadata(
            KeyCode::A,
            true,
            1000,
            None,
            false,
            false,
            30,
            Some("TEST".to_string()),
        );
        assert!(resolver.extract_identity(&event1).is_none());

        // Missing serial
        let mut event2 = create_test_event_with_identity();
        event2.serial_number = None;
        assert!(resolver.extract_identity(&event2).is_none());
    }

    #[tokio::test]
    async fn test_resolve_concurrent_access() {
        let (registry, _rx) = DeviceRegistry::new();
        let resolver = DeviceResolver::new(registry.clone());

        // Register device
        let identity = create_test_identity();
        registry.register_device(identity.clone()).await;

        // Spawn multiple concurrent resolve operations
        let mut handles = vec![];
        for _ in 0..10 {
            let resolver_clone = resolver.clone();
            let handle = tokio::spawn(async move {
                let event = create_test_event_with_identity();
                resolver_clone.resolve(&event).await
            });
            handles.push(handle);
        }

        // All should succeed
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
            assert!(result.unwrap().is_some());
        }
    }
}
