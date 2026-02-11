//! E2E Test Configuration.
//!
//! This module provides test configuration builders for common test scenarios.

use keyrx_core::config::{
    BaseKeyMapping, Condition, ConditionItem, ConfigRoot, DeviceConfig, DeviceIdentifier, KeyCode,
    KeyMapping, Metadata, Version,
};

/// Configuration for an E2E test scenario.
///
/// Provides helper constructors to easily create test configurations for
/// common remapping scenarios. The configuration includes:
///
/// - Device pattern for matching keyboards
/// - Key mappings to apply
///
/// # Example
///
/// ```ignore
/// // Simple A → B remapping
/// let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B);
///
/// // Navigation layer with modifier
/// let config = E2EConfig::with_modifier_layer(
///     KeyCode::CapsLock,
///     0,
///     vec![
///         (KeyCode::H, KeyCode::Left),
///         (KeyCode::J, KeyCode::Down),
///     ],
/// );
/// ```
#[derive(Debug, Clone)]
pub struct E2EConfig {
    /// Device pattern for matching (default: "*" for all devices)
    pub device_pattern: String,
    /// Key mappings to apply
    pub mappings: Vec<KeyMapping>,
}

#[allow(dead_code)]
impl E2EConfig {
    pub fn new(device_pattern: impl Into<String>, mappings: Vec<KeyMapping>) -> Self {
        Self {
            device_pattern: device_pattern.into(),
            mappings,
        }
    }

    /// Creates a configuration with a simple key remapping (A → B).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::simple_remap(KeyCode::CapsLock, KeyCode::Escape);
    /// ```
    pub fn simple_remap(from: KeyCode, to: KeyCode) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::simple(from, to)],
        }
    }

    /// Creates a configuration with multiple simple remappings.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::simple_remaps(vec![
    ///     (KeyCode::A, KeyCode::B),
    ///     (KeyCode::CapsLock, KeyCode::Escape),
    /// ]);
    /// ```
    pub fn simple_remaps(remaps: Vec<(KeyCode, KeyCode)>) -> Self {
        let mappings = remaps
            .into_iter()
            .map(|(from, to)| KeyMapping::simple(from, to))
            .collect();

        Self {
            device_pattern: "*".to_string(),
            mappings,
        }
    }

    /// Creates a configuration with a custom modifier key.
    ///
    /// The modifier key will set internal state when held, but produces no
    /// output events.
    ///
    /// # Arguments
    ///
    /// * `from` - The key that activates the modifier
    /// * `modifier_id` - The modifier ID (0-254)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::modifier(KeyCode::CapsLock, 0);
    /// ```
    pub fn modifier(from: KeyCode, modifier_id: u8) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::modifier(from, modifier_id)],
        }
    }

    /// Creates a configuration with a toggle lock key.
    ///
    /// The lock key toggles internal state on press (no output on release).
    ///
    /// # Arguments
    ///
    /// * `from` - The key that toggles the lock
    /// * `lock_id` - The lock ID (0-254)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::lock(KeyCode::ScrollLock, 0);
    /// ```
    pub fn lock(from: KeyCode, lock_id: u8) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::lock(from, lock_id)],
        }
    }

    /// Creates a configuration with a conditional mapping.
    ///
    /// Maps `from` → `to` only when the specified modifier is active.
    ///
    /// # Arguments
    ///
    /// * `modifier_id` - The modifier that must be active
    /// * `from` - Source key for the mapping
    /// * `to` - Target key for the mapping
    ///
    /// # Example
    ///
    /// ```ignore
    /// // When modifier 0 is active, H → Left
    /// let config = E2EConfig::conditional(0, KeyCode::H, KeyCode::Left);
    /// ```
    pub fn conditional(modifier_id: u8, from: KeyCode, to: KeyCode) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::conditional(
                Condition::ModifierActive(modifier_id),
                vec![BaseKeyMapping::Simple { from, to }],
            )],
        }
    }

    /// Creates a configuration with a modifier key and conditional mappings.
    ///
    /// This is the common pattern for navigation layers (e.g., Vim-style HJKL).
    ///
    /// # Arguments
    ///
    /// * `modifier_key` - The key that activates the layer
    /// * `modifier_id` - The modifier ID for the layer
    /// * `layer_mappings` - List of (from, to) pairs active when layer is held
    ///
    /// # Example
    ///
    /// ```ignore
    /// // CapsLock activates layer, HJKL become arrow keys
    /// let config = E2EConfig::with_modifier_layer(
    ///     KeyCode::CapsLock,
    ///     0,
    ///     vec![
    ///         (KeyCode::H, KeyCode::Left),
    ///         (KeyCode::J, KeyCode::Down),
    ///         (KeyCode::K, KeyCode::Up),
    ///         (KeyCode::L, KeyCode::Right),
    ///     ],
    /// );
    /// ```
    pub fn with_modifier_layer(
        modifier_key: KeyCode,
        modifier_id: u8,
        layer_mappings: Vec<(KeyCode, KeyCode)>,
    ) -> Self {
        let mut mappings = vec![KeyMapping::modifier(modifier_key, modifier_id)];

        for (from, to) in layer_mappings {
            mappings.push(KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(modifier_id)]),
                vec![BaseKeyMapping::Simple { from, to }],
            ));
        }

        Self {
            device_pattern: "*".to_string(),
            mappings,
        }
    }

    /// Creates a configuration with a lock key and conditional mappings.
    ///
    /// Similar to modifier layer but uses toggle lock instead of momentary hold.
    ///
    /// # Arguments
    ///
    /// * `lock_key` - The key that toggles the layer
    /// * `lock_id` - The lock ID for the layer
    /// * `layer_mappings` - List of (from, to) pairs active when lock is on
    ///
    /// # Example
    ///
    /// ```ignore
    /// // ScrollLock toggles layer, number row becomes F-keys
    /// let config = E2EConfig::with_lock_layer(
    ///     KeyCode::ScrollLock,
    ///     0,
    ///     vec![
    ///         (KeyCode::Num1, KeyCode::F1),
    ///         (KeyCode::Num2, KeyCode::F2),
    ///     ],
    /// );
    /// ```
    pub fn with_lock_layer(
        lock_key: KeyCode,
        lock_id: u8,
        layer_mappings: Vec<(KeyCode, KeyCode)>,
    ) -> Self {
        let mut mappings = vec![KeyMapping::lock(lock_key, lock_id)];

        for (from, to) in layer_mappings {
            mappings.push(KeyMapping::conditional(
                Condition::LockActive(lock_id),
                vec![BaseKeyMapping::Simple { from, to }],
            ));
        }

        Self {
            device_pattern: "*".to_string(),
            mappings,
        }
    }

    /// Creates a configuration with a modified output mapping.
    ///
    /// When `from` is pressed, outputs `to` with specified physical modifiers.
    ///
    /// # Arguments
    ///
    /// * `from` - Source key
    /// * `to` - Target key
    /// * `shift` - Include Shift modifier
    /// * `ctrl` - Include Ctrl modifier
    /// * `alt` - Include Alt modifier
    /// * `win` - Include Win/Meta modifier
    ///
    /// # Example
    ///
    /// ```ignore
    /// // A → Shift+1 (outputs '!')
    /// let config = E2EConfig::modified_output(
    ///     KeyCode::A,
    ///     KeyCode::Num1,
    ///     true, false, false, false,
    /// );
    /// ```
    pub fn modified_output(
        from: KeyCode,
        to: KeyCode,
        shift: bool,
        ctrl: bool,
        alt: bool,
        win: bool,
    ) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::modified_output(from, to, shift, ctrl, alt, win)],
        }
    }

    /// Creates a configuration with a tap-hold mapping.
    ///
    /// When the key is tapped (quick press and release), it outputs `tap_key`.
    /// When held beyond `threshold_ms`, it activates `hold_modifier`.
    ///
    /// # Arguments
    ///
    /// * `from` - Source key (e.g., CapsLock)
    /// * `tap_key` - Key to output on tap (e.g., Escape)
    /// * `hold_modifier` - Modifier ID to activate on hold (0-254)
    /// * `threshold_ms` - Time in milliseconds to distinguish tap from hold
    ///
    /// # Example
    ///
    /// ```ignore
    /// // CapsLock: tap=Escape, hold=Ctrl (modifier 0), 200ms threshold
    /// let config = E2EConfig::tap_hold(
    ///     KeyCode::CapsLock,
    ///     KeyCode::Escape,
    ///     0,
    ///     200,
    /// );
    /// ```
    #[allow(dead_code)]
    pub fn tap_hold(from: KeyCode, tap_key: KeyCode, hold_modifier: u8, threshold_ms: u16) -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: vec![KeyMapping::tap_hold(
                from,
                tap_key,
                hold_modifier,
                threshold_ms,
            )],
        }
    }

    /// Creates a configuration with a tap-hold mapping and conditional layer.
    ///
    /// Combines tap-hold with a layer of conditional mappings that activate
    /// when the hold modifier is active.
    ///
    /// # Arguments
    ///
    /// * `from` - Source key for tap-hold
    /// * `tap_key` - Key to output on tap
    /// * `hold_modifier` - Modifier ID to activate on hold
    /// * `threshold_ms` - Time in milliseconds to distinguish tap from hold
    /// * `layer_mappings` - List of (from, to) pairs active when modifier is held
    ///
    /// # Example
    ///
    /// ```ignore
    /// // CapsLock: tap=Escape, hold=navigation layer with HJKL arrows
    /// let config = E2EConfig::tap_hold_with_layer(
    ///     KeyCode::CapsLock,
    ///     KeyCode::Escape,
    ///     0,
    ///     200,
    ///     vec![
    ///         (KeyCode::H, KeyCode::Left),
    ///         (KeyCode::J, KeyCode::Down),
    ///         (KeyCode::K, KeyCode::Up),
    ///         (KeyCode::L, KeyCode::Right),
    ///     ],
    /// );
    /// ```
    #[allow(dead_code)]
    pub fn tap_hold_with_layer(
        from: KeyCode,
        tap_key: KeyCode,
        hold_modifier: u8,
        threshold_ms: u16,
        layer_mappings: Vec<(KeyCode, KeyCode)>,
    ) -> Self {
        let mut mappings = vec![KeyMapping::tap_hold(
            from,
            tap_key,
            hold_modifier,
            threshold_ms,
        )];

        for (layer_from, layer_to) in layer_mappings {
            mappings.push(KeyMapping::conditional(
                Condition::AllActive(vec![ConditionItem::ModifierActive(hold_modifier)]),
                vec![BaseKeyMapping::Simple {
                    from: layer_from,
                    to: layer_to,
                }],
            ));
        }

        Self {
            device_pattern: "*".to_string(),
            mappings,
        }
    }

    /// Adds additional mappings to this configuration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B)
    ///     .with_mappings(vec![
    ///         KeyMapping::simple(KeyCode::C, KeyCode::D),
    ///     ]);
    /// ```
    pub fn with_mappings(mut self, mappings: Vec<KeyMapping>) -> Self {
        self.mappings.extend(mappings);
        self
    }

    /// Sets the device pattern for this configuration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = E2EConfig::simple_remap(KeyCode::A, KeyCode::B)
    ///     .with_device_pattern("USB*");
    /// ```
    pub fn with_device_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.device_pattern = pattern.into();
        self
    }

    /// Converts this E2EConfig to a ConfigRoot for serialization.
    ///
    /// This creates a complete configuration with proper version and metadata.
    pub fn to_config_root(&self) -> ConfigRoot {
        ConfigRoot {
            version: Version::current(),
            devices: vec![DeviceConfig {
                identifier: DeviceIdentifier {
                    pattern: self.device_pattern.clone(),
                },
                mappings: self.mappings.clone(),
            }],
            metadata: Metadata {
                compilation_timestamp: 0,
                compiler_version: "e2e-test".to_string(),
                source_hash: "e2e-test".to_string(),
            },
        }
    }
}

impl Default for E2EConfig {
    fn default() -> Self {
        Self {
            device_pattern: "*".to_string(),
            mappings: Vec::new(),
        }
    }
}
