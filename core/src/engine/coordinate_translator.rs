//! Coordinate translation for the revolutionary mapping pipeline.
//!
//! The CoordinateTranslator provides fast scancode-to-physical-position mapping
//! for devices with known definitions. It caches translation maps per device
//! to meet the <20μs lookup target for the event processing hot path.

use crate::definitions::library::DeviceDefinitionLibrary;
use crate::identity::DeviceIdentity;
use crate::registry::profile::PhysicalPosition;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::debug;

/// Error type for coordinate translation
#[derive(Debug, Error)]
pub enum CoordinateTranslatorError {
    #[error("Device definition not found for {vendor_id:04x}:{product_id:04x}")]
    DeviceDefinitionNotFound { vendor_id: u16, product_id: u16 },

    #[error(
        "Scancode {scancode} not mapped in device definition for {vendor_id:04x}:{product_id:04x}"
    )]
    ScancodeNotMapped {
        scancode: u16,
        vendor_id: u16,
        product_id: u16,
    },
}

/// Translation map for a single device (scancode -> PhysicalPosition)
type TranslationMap = HashMap<u16, PhysicalPosition>;

/// Cache key for translation maps (vendor_id, product_id)
type CacheKey = (u16, u16);

/// Translates scancodes to physical positions using device definitions.
///
/// The CoordinateTranslator wraps a DeviceDefinitionLibrary and maintains
/// a per-device cache of translation maps for O(1) lookups in the hot path.
///
/// # Performance Characteristics
///
/// - First translation for a device: <1ms (build translation map from definition)
/// - Subsequent translations: <20μs (cached HashMap lookup)
/// - Cache invalidation: O(1) removal from cache
///
/// # Cache Strategy
///
/// Translation maps are built lazily on first use and cached by (VID, PID).
/// The cache uses `Arc<HashMap>` for zero-copy sharing across pipeline stages.
pub struct CoordinateTranslator {
    /// Reference to the device definition library
    library: Arc<DeviceDefinitionLibrary>,
    /// Cache of translation maps keyed by (vendor_id, product_id)
    cache: Arc<RwLock<HashMap<CacheKey, Arc<TranslationMap>>>>,
}

impl CoordinateTranslator {
    /// Create a new CoordinateTranslator wrapping the given library.
    pub fn new(library: Arc<DeviceDefinitionLibrary>) -> Self {
        Self {
            library,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Translate a scancode to a physical position for a specific device.
    ///
    /// This method uses a per-device translation map cache:
    /// - First call for a device builds the map from the definition (cold: ~1ms)
    /// - Subsequent calls use the cached map (hot: <20μs)
    ///
    /// # Arguments
    ///
    /// * `device_identity` - The device to translate for
    /// * `scancode` - The scancode to translate
    ///
    /// # Returns
    ///
    /// The physical position (row, col) for the scancode.
    ///
    /// # Errors
    ///
    /// Returns `DeviceDefinitionNotFound` if no definition exists for the device.
    /// Returns `ScancodeNotMapped` if the scancode is not in the device's matrix_map.
    ///
    /// # Performance
    ///
    /// This method meets the <20μs latency target for cached translation maps.
    /// Cold loads may take up to 1ms but occur only once per device.
    pub async fn translate(
        &self,
        device_identity: &DeviceIdentity,
        scancode: u16,
    ) -> Result<PhysicalPosition, CoordinateTranslatorError> {
        let cache_key = (device_identity.vendor_id, device_identity.product_id);

        // Try cache first (fast path)
        {
            let cache = self.cache.read().await;
            if let Some(map) = cache.get(&cache_key) {
                return map.get(&scancode).copied().ok_or(
                    CoordinateTranslatorError::ScancodeNotMapped {
                        scancode,
                        vendor_id: device_identity.vendor_id,
                        product_id: device_identity.product_id,
                    },
                );
            }
        }

        // Cache miss - build translation map (slow path)
        self.build_and_cache_translation_map(device_identity, cache_key)
            .await?;

        // Try again with cached map
        let cache = self.cache.read().await;
        // SAFETY: We just built and cached the map, so it must exist
        let map = match cache.get(&cache_key) {
            Some(m) => m,
            None => {
                // This should never happen, but return an error instead of panicking
                return Err(CoordinateTranslatorError::DeviceDefinitionNotFound {
                    vendor_id: device_identity.vendor_id,
                    product_id: device_identity.product_id,
                });
            }
        };

        map.get(&scancode)
            .copied()
            .ok_or(CoordinateTranslatorError::ScancodeNotMapped {
                scancode,
                vendor_id: device_identity.vendor_id,
                product_id: device_identity.product_id,
            })
    }

    /// Build and cache a translation map for a device.
    ///
    /// This is called on the first translation request for a device.
    /// The translation map is extracted from the device definition's matrix_map
    /// and cached for subsequent lookups.
    async fn build_and_cache_translation_map(
        &self,
        device_identity: &DeviceIdentity,
        cache_key: CacheKey,
    ) -> Result<(), CoordinateTranslatorError> {
        // Find the device definition
        let definition = self
            .library
            .find_definition(device_identity.vendor_id, device_identity.product_id)
            .ok_or(CoordinateTranslatorError::DeviceDefinitionNotFound {
                vendor_id: device_identity.vendor_id,
                product_id: device_identity.product_id,
            })?;

        // Clone the matrix_map to create our translation map
        // This is a one-time cost on first use
        let translation_map: TranslationMap = definition.matrix_map.clone();

        debug!(
            service = "keyrx",
            event = "translation_map_built",
            component = "coordinate_translator",
            vendor_id = format!("{:04x}", device_identity.vendor_id),
            product_id = format!("{:04x}", device_identity.product_id),
            map_size = translation_map.len(),
            "Built translation map for device"
        );

        // Cache the map
        let mut cache = self.cache.write().await;
        cache.insert(cache_key, Arc::new(translation_map));

        Ok(())
    }

    /// Check if a device has a known definition.
    ///
    /// This is a fast check that doesn't build the translation map.
    /// Useful for determining if a device can be used with revolutionary mapping.
    pub fn has_definition(&self, vendor_id: u16, product_id: u16) -> bool {
        self.library
            .find_definition(vendor_id, product_id)
            .is_some()
    }

    /// Get the layout type string for a device, if known.
    ///
    /// Returns the layout type (e.g., "matrix", "standard", "split") from the
    /// device definition, or None if no definition exists.
    pub fn get_layout_type(&self, vendor_id: u16, product_id: u16) -> Option<String> {
        self.library
            .find_definition(vendor_id, product_id)
            .map(|def| def.layout_type_str().to_string())
    }

    /// Invalidate the cached translation map for a specific device.
    ///
    /// Call this if device definitions are reloaded or modified.
    /// The next translation will rebuild the map from the updated definition.
    pub async fn invalidate_cache(&self, vendor_id: u16, product_id: u16) {
        let mut cache = self.cache.write().await;
        cache.remove(&(vendor_id, product_id));

        debug!(
            service = "keyrx",
            event = "translation_cache_invalidated",
            component = "coordinate_translator",
            vendor_id = format!("{:04x}", vendor_id),
            product_id = format!("{:04x}", product_id),
            "Translation cache entry invalidated"
        );
    }

    /// Invalidate all cached translation maps.
    ///
    /// Use this when reloading device definitions from disk.
    pub async fn invalidate_all(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();

        debug!(
            service = "keyrx",
            event = "translation_cache_cleared",
            component = "coordinate_translator",
            "All translation cache entries cleared"
        );
    }

    /// Get the number of cached translation maps.
    ///
    /// Useful for monitoring and diagnostics.
    pub async fn cache_size(&self) -> usize {
        let cache = self.cache.read().await;
        cache.len()
    }

    /// Get a reference to the underlying device definition library.
    pub fn library(&self) -> &Arc<DeviceDefinitionLibrary> {
        &self.library
    }
}

impl std::fmt::Debug for CoordinateTranslator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CoordinateTranslator")
            .field("library", &"DeviceDefinitionLibrary")
            .field("cache_entries", &"<async>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn create_test_definition_toml() -> String {
        r#"
name = "Test Device"
vendor_id = 0x1234
product_id = 0x5678
manufacturer = "Test Manufacturer"

[layout]
layout_type = "matrix"
rows = 3
cols = 3

[matrix_map]
"1" = { row = 0, col = 0 }
"2" = { row = 0, col = 1 }
"3" = { row = 0, col = 2 }
"4" = { row = 1, col = 0 }
"5" = { row = 1, col = 1 }
"6" = { row = 1, col = 2 }
"7" = { row = 2, col = 0 }
"8" = { row = 2, col = 1 }
"9" = { row = 2, col = 2 }

[visual]
key_width = 80
key_height = 80
key_spacing = 4
"#
        .to_string()
    }

    fn setup_test_library() -> (TempDir, Arc<DeviceDefinitionLibrary>) {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.toml");
        std::fs::write(&file_path, create_test_definition_toml()).unwrap();

        let mut library = DeviceDefinitionLibrary::new();
        library.load_from_directory(temp_dir.path()).unwrap();

        (temp_dir, Arc::new(library))
    }

    #[tokio::test]
    #[serial]
    async fn test_translate_known_scancode() {
        let (_temp, library) = setup_test_library();
        let translator = CoordinateTranslator::new(library);

        let device = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());

        // Translate scancode 1 -> should be (0, 0)
        let pos = translator.translate(&device, 1).await.unwrap();
        assert_eq!(pos, PhysicalPosition::new(0, 0));

        // Translate scancode 5 -> should be (1, 1)
        let pos = translator.translate(&device, 5).await.unwrap();
        assert_eq!(pos, PhysicalPosition::new(1, 1));

        // Translate scancode 9 -> should be (2, 2)
        let pos = translator.translate(&device, 9).await.unwrap();
        assert_eq!(pos, PhysicalPosition::new(2, 2));
    }

    #[tokio::test]
    #[serial]
    async fn test_translate_unknown_scancode() {
        let (_temp, library) = setup_test_library();
        let translator = CoordinateTranslator::new(library);

        let device = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());

        // Translate unknown scancode
        let result = translator.translate(&device, 999).await;
        assert!(matches!(
            result,
            Err(CoordinateTranslatorError::ScancodeNotMapped { scancode: 999, .. })
        ));
    }

    #[tokio::test]
    #[serial]
    async fn test_translate_unknown_device() {
        let (_temp, library) = setup_test_library();
        let translator = CoordinateTranslator::new(library);

        let device = DeviceIdentity::new(0x9999, 0x8888, "UNKNOWN".to_string());

        // Translate for unknown device
        let result = translator.translate(&device, 1).await;
        assert!(matches!(
            result,
            Err(CoordinateTranslatorError::DeviceDefinitionNotFound { .. })
        ));
    }

    #[tokio::test]
    #[serial]
    async fn test_caching() {
        let (_temp, library) = setup_test_library();
        let translator = CoordinateTranslator::new(library);

        let device = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());

        // Cache should be empty initially
        assert_eq!(translator.cache_size().await, 0);

        // First translation should build cache
        let _pos = translator.translate(&device, 1).await.unwrap();
        assert_eq!(translator.cache_size().await, 1);

        // Second translation should use cache
        let _pos = translator.translate(&device, 2).await.unwrap();
        assert_eq!(translator.cache_size().await, 1); // Still 1 entry
    }

    #[tokio::test]
    #[serial]
    async fn test_cache_different_devices() {
        let temp_dir = TempDir::new().unwrap();

        // Create two device definitions
        let file1 = temp_dir.path().join("device1.toml");
        std::fs::write(&file1, create_test_definition_toml()).unwrap();

        let file2 = temp_dir.path().join("device2.toml");
        let toml2 = create_test_definition_toml()
            .replace("0x1234", "0x1111")
            .replace("0x5678", "0x2222");
        std::fs::write(&file2, toml2).unwrap();

        let mut library = DeviceDefinitionLibrary::new();
        library.load_from_directory(temp_dir.path()).unwrap();
        let translator = CoordinateTranslator::new(Arc::new(library));

        let device1 = DeviceIdentity::new(0x1234, 0x5678, "DEV1".to_string());
        let device2 = DeviceIdentity::new(0x1111, 0x2222, "DEV2".to_string());

        // Translate for both devices
        let _pos1 = translator.translate(&device1, 1).await.unwrap();
        assert_eq!(translator.cache_size().await, 1);

        let _pos2 = translator.translate(&device2, 1).await.unwrap();
        assert_eq!(translator.cache_size().await, 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_invalidate_cache_single() {
        let (_temp, library) = setup_test_library();
        let translator = CoordinateTranslator::new(library);

        let device = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());

        // Build cache
        let _pos = translator.translate(&device, 1).await.unwrap();
        assert_eq!(translator.cache_size().await, 1);

        // Invalidate
        translator.invalidate_cache(0x1234, 0x5678).await;
        assert_eq!(translator.cache_size().await, 0);

        // Should still work (rebuilds cache)
        let pos = translator.translate(&device, 1).await.unwrap();
        assert_eq!(pos, PhysicalPosition::new(0, 0));
        assert_eq!(translator.cache_size().await, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_invalidate_all() {
        let temp_dir = TempDir::new().unwrap();

        // Create two device definitions
        let file1 = temp_dir.path().join("device1.toml");
        std::fs::write(&file1, create_test_definition_toml()).unwrap();

        let file2 = temp_dir.path().join("device2.toml");
        let toml2 = create_test_definition_toml()
            .replace("0x1234", "0x1111")
            .replace("0x5678", "0x2222");
        std::fs::write(&file2, toml2).unwrap();

        let mut library = DeviceDefinitionLibrary::new();
        library.load_from_directory(temp_dir.path()).unwrap();
        let translator = CoordinateTranslator::new(Arc::new(library));

        let device1 = DeviceIdentity::new(0x1234, 0x5678, "DEV1".to_string());
        let device2 = DeviceIdentity::new(0x1111, 0x2222, "DEV2".to_string());

        // Build cache for both
        let _pos1 = translator.translate(&device1, 1).await.unwrap();
        let _pos2 = translator.translate(&device2, 1).await.unwrap();
        assert_eq!(translator.cache_size().await, 2);

        // Invalidate all
        translator.invalidate_all().await;
        assert_eq!(translator.cache_size().await, 0);

        // Both should still work
        let _pos1 = translator.translate(&device1, 1).await.unwrap();
        let _pos2 = translator.translate(&device2, 1).await.unwrap();
        assert_eq!(translator.cache_size().await, 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_has_definition() {
        let (_temp, library) = setup_test_library();
        let translator = CoordinateTranslator::new(library);

        assert!(translator.has_definition(0x1234, 0x5678));
        assert!(!translator.has_definition(0x9999, 0x8888));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_layout_type() {
        let (_temp, library) = setup_test_library();
        let translator = CoordinateTranslator::new(library);

        let layout = translator.get_layout_type(0x1234, 0x5678);
        assert_eq!(layout, Some("matrix".to_string()));

        let unknown = translator.get_layout_type(0x9999, 0x8888);
        assert_eq!(unknown, None);
    }

    #[tokio::test]
    #[serial]
    async fn test_concurrent_translation() {
        let (_temp, library) = setup_test_library();
        let translator = Arc::new(CoordinateTranslator::new(library));

        let device = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());

        // Spawn multiple concurrent translation operations
        let mut handles = vec![];
        for scancode in 1..=9 {
            let translator_clone = Arc::clone(&translator);
            let device_clone = device.clone();
            let handle =
                tokio::spawn(
                    async move { translator_clone.translate(&device_clone, scancode).await },
                );
            handles.push(handle);
        }

        // All should succeed
        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.await.unwrap();
            assert!(result.is_ok(), "Translation for scancode {} failed", i + 1);
        }

        // Cache should have one entry (all used the same device)
        assert_eq!(translator.cache_size().await, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_translation_all_scancodes() {
        let (_temp, library) = setup_test_library();
        let translator = CoordinateTranslator::new(library);

        let device = DeviceIdentity::new(0x1234, 0x5678, "TEST123".to_string());

        // Test all scancodes in the definition
        let expected = vec![
            (1, PhysicalPosition::new(0, 0)),
            (2, PhysicalPosition::new(0, 1)),
            (3, PhysicalPosition::new(0, 2)),
            (4, PhysicalPosition::new(1, 0)),
            (5, PhysicalPosition::new(1, 1)),
            (6, PhysicalPosition::new(1, 2)),
            (7, PhysicalPosition::new(2, 0)),
            (8, PhysicalPosition::new(2, 1)),
            (9, PhysicalPosition::new(2, 2)),
        ];

        for (scancode, expected_pos) in expected {
            let pos = translator.translate(&device, scancode).await.unwrap();
            assert_eq!(
                pos, expected_pos,
                "Scancode {} should map to {:?}",
                scancode, expected_pos
            );
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_serial_number_ignored_for_caching() {
        let (_temp, library) = setup_test_library();
        let translator = CoordinateTranslator::new(library);

        // Two devices with same VID:PID but different serials
        let device1 = DeviceIdentity::new(0x1234, 0x5678, "SERIAL1".to_string());
        let device2 = DeviceIdentity::new(0x1234, 0x5678, "SERIAL2".to_string());

        // Both should use the same cached translation map
        let _pos1 = translator.translate(&device1, 1).await.unwrap();
        assert_eq!(translator.cache_size().await, 1);

        let _pos2 = translator.translate(&device2, 1).await.unwrap();
        assert_eq!(translator.cache_size().await, 1); // Still 1 entry
    }

    #[tokio::test]
    #[serial]
    async fn test_real_device_definitions() {
        // Test with actual device definitions if they exist
        let device_defs_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("device_definitions");

        if !device_defs_path.exists() {
            // Skip test if definitions directory doesn't exist
            return;
        }

        let mut library = DeviceDefinitionLibrary::new();
        let result = library.load_from_directory(&device_defs_path);
        if result.is_err() || library.is_empty() {
            // Skip if no definitions loaded
            return;
        }

        let translator = CoordinateTranslator::new(Arc::new(library));

        // Test Stream Deck MK.2 if available
        if translator.has_definition(0x0fd9, 0x0080) {
            let device = DeviceIdentity::new(0x0fd9, 0x0080, "TEST".to_string());

            // Stream Deck MK.2 is a 3x5 matrix, button IDs start at 1
            let pos = translator.translate(&device, 1).await;
            assert!(
                pos.is_ok(),
                "Should translate button 1 for Stream Deck MK.2"
            );

            let layout = translator.get_layout_type(0x0fd9, 0x0080);
            assert_eq!(layout, Some("matrix".to_string()));
        }
    }
}
