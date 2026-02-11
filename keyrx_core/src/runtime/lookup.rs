//! Key lookup table for O(1) mapping resolution
//!
//! This module provides `KeyLookup` for efficient key-to-mapping resolution
//! using a HashMap-based lookup table.

extern crate alloc;
use alloc::vec::Vec;
use hashbrown::HashMap;

use crate::config::{BaseKeyMapping, Condition, DeviceConfig, KeyCode, KeyMapping};
use crate::runtime::state::DeviceState;

/// Entry in the lookup table containing a mapping and optional condition
///
/// Conditional mappings have a Some(condition), unconditional have None.
#[derive(Clone, Debug)]
struct LookupEntry {
    /// The base key mapping
    mapping: BaseKeyMapping,
    /// Optional condition that must be true for this mapping to apply
    condition: Option<Condition>,
}

/// Key lookup table for O(1) mapping resolution
///
/// Groups mappings by input key with conditional mappings ordered before
/// unconditional mappings to ensure correct precedence.
///
/// # Ordering
///
/// Mappings for the same key are stored in order of registration with
/// conditional mappings appearing before unconditional mappings. This ensures
/// that conditional mappings are checked first during lookup.
///
/// # Example
///
/// ```rust,ignore
/// use keyrx_core::runtime::KeyLookup;
/// use keyrx_core::config::DeviceConfig;
///
/// let config: DeviceConfig = /* ... */;
/// let lookup = KeyLookup::from_device_config(&config);
/// ```
pub struct KeyLookup {
    /// HashMap mapping KeyCode to Vec of LookupEntry
    /// Conditional mappings are ordered before unconditional ones
    table: HashMap<KeyCode, Vec<LookupEntry>>,
}

impl KeyLookup {
    /// Adds conditional mappings to the lookup table.
    fn add_conditional_mappings(
        table: &mut HashMap<KeyCode, Vec<LookupEntry>>,
        mapping: &KeyMapping,
    ) {
        if let KeyMapping::Conditional {
            condition,
            mappings,
        } = mapping
        {
            for base_mapping in mappings {
                if let Some(key) = Self::extract_input_key(base_mapping) {
                    table.entry(key).or_insert_with(Vec::new).push(LookupEntry {
                        mapping: base_mapping.clone(),
                        condition: Some(condition.clone()),
                    });
                }
            }
        }
    }

    /// Adds unconditional mappings to the lookup table.
    fn add_unconditional_mappings(
        table: &mut HashMap<KeyCode, Vec<LookupEntry>>,
        mapping: &KeyMapping,
    ) {
        if let KeyMapping::Base(base_mapping) = mapping {
            if let Some(key) = Self::extract_input_key(base_mapping) {
                table.entry(key).or_insert_with(Vec::new).push(LookupEntry {
                    mapping: base_mapping.clone(),
                    condition: None,
                });
            }
        }
    }

    /// Creates a key lookup table from device configuration
    ///
    /// Iterates through all mappings in the config, extracts the input key
    /// from each mapping variant, and groups them in a HashMap. Conditional
    /// mappings are inserted before unconditional mappings to ensure proper
    /// precedence during lookup.
    ///
    /// # Arguments
    ///
    /// * `config` - The device configuration containing key mappings
    ///
    /// # Returns
    ///
    /// A new `KeyLookup` instance with all mappings indexed by input key
    pub fn from_device_config(config: &DeviceConfig) -> Self {
        let mut table: HashMap<KeyCode, Vec<LookupEntry>> = HashMap::new();

        // First pass: collect conditional mappings (higher precedence)
        for mapping in &config.mappings {
            Self::add_conditional_mappings(&mut table, mapping);
        }

        // Second pass: collect unconditional (base) mappings (fallback)
        for mapping in &config.mappings {
            Self::add_unconditional_mappings(&mut table, mapping);
        }

        Self { table }
    }

    /// Finds the appropriate mapping for a key based on current device state
    ///
    /// This is a convenience method that calls `find_mapping_with_device`
    /// without a device_id. Use this for lookups that don't involve
    /// device-specific conditions (DeviceMatches).
    ///
    /// Note: DeviceMatches conditions will never match when using this method.
    /// Use `find_mapping_with_device` for device-aware lookups.
    ///
    /// # Arguments
    ///
    /// * `key` - The input key code to look up
    /// * `state` - The current device state for condition evaluation
    ///
    /// # Returns
    ///
    /// * `Some(&BaseKeyMapping)` - Reference to the first matching mapping
    /// * `None` - No mapping found (key should be passed through)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mapping = lookup.find_mapping(KeyCode::H, &state);
    /// match mapping {
    ///     Some(m) => // Process mapping
    ///     None => // Pass through key unchanged
    /// }
    /// ```
    pub fn find_mapping(&self, key: KeyCode, state: &DeviceState) -> Option<&BaseKeyMapping> {
        self.find_mapping_with_device(key, state, None)
    }

    /// Finds the first matching entry for a key and state.
    fn find_matching_entry<'a>(
        entries: &'a [LookupEntry],
        state: &DeviceState,
        device_id: Option<&str>,
    ) -> Option<&'a LookupEntry> {
        entries
            .iter()
            .find(|entry| Self::entry_matches(entry, state, device_id))
    }

    /// Finds the appropriate mapping for a key based on current device state and device ID
    ///
    /// This is the full version of mapping lookup that supports device-specific
    /// conditions. For lookups that don't involve device matching, you can use
    /// `find_mapping()`.
    ///
    /// Searches for mappings for the given key and evaluates conditions to find
    /// the first matching mapping. Conditional mappings are checked first (in
    /// registration order), followed by unconditional mappings.
    ///
    /// # Arguments
    ///
    /// * `key` - The input key code to look up
    /// * `state` - The current device state for condition evaluation
    /// * `device_id` - Optional device ID from the current event (for DeviceMatches conditions)
    ///
    /// # Returns
    ///
    /// * `Some(&BaseKeyMapping)` - Reference to the first matching mapping
    /// * `None` - No mapping found (key should be passed through)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Lookup with device context
    /// let mapping = lookup.find_mapping_with_device(
    ///     KeyCode::Numpad1,
    ///     &state,
    ///     Some("usb-numpad-123")
    /// );
    /// match mapping {
    ///     Some(m) => // Process mapping (may be device-specific)
    ///     None => // Pass through key unchanged
    /// }
    /// ```
    pub fn find_mapping_with_device(
        &self,
        key: KeyCode,
        state: &DeviceState,
        device_id: Option<&str>,
    ) -> Option<&BaseKeyMapping> {
        let entries = self.table.get(&key)?;
        Self::find_matching_entry(entries, state, device_id).map(|entry| &entry.mapping)
    }

    /// Evaluates a condition against state and optional device ID.
    fn evaluate_condition(
        condition: &Condition,
        state: &DeviceState,
        device_id: Option<&str>,
    ) -> bool {
        state.evaluate_condition_with_device(condition, device_id)
    }

    /// Checks if a mapping entry matches the current state.
    fn entry_matches(entry: &LookupEntry, state: &DeviceState, device_id: Option<&str>) -> bool {
        match &entry.condition {
            Some(condition) => Self::evaluate_condition(condition, state, device_id),
            None => true, // Unconditional mapping always matches
        }
    }

    /// Extracts the input key from a BaseKeyMapping variant
    ///
    /// # Arguments
    ///
    /// * `mapping` - The base key mapping to extract the input key from
    ///
    /// # Returns
    ///
    /// The input KeyCode if the mapping has one, None otherwise
    fn extract_input_key(mapping: &BaseKeyMapping) -> Option<KeyCode> {
        match mapping {
            BaseKeyMapping::Simple { from, .. } => Some(*from),
            BaseKeyMapping::Modifier { from, .. } => Some(*from),
            BaseKeyMapping::Lock { from, .. } => Some(*from),
            BaseKeyMapping::TapHold { from, .. } => Some(*from),
            BaseKeyMapping::ModifiedOutput { from, .. } => Some(*from),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::string::String;
    use alloc::vec;

    use crate::config::{Condition, DeviceIdentifier};

    /// Helper to create a simple test DeviceConfig
    fn create_test_device_config(mappings: Vec<KeyMapping>) -> DeviceConfig {
        DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: String::from("*"),
            },
            mappings,
        }
    }

    #[test]
    fn test_from_device_config_empty() {
        let config = create_test_device_config(vec![]);
        let lookup = KeyLookup::from_device_config(&config);

        // Empty config should produce empty table
        assert!(lookup.table.is_empty());
    }

    #[test]
    fn test_from_device_config_simple_mapping() {
        let config = create_test_device_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
        let lookup = KeyLookup::from_device_config(&config);

        // Should have one entry
        assert_eq!(lookup.table.len(), 1);

        // Entry for key A should exist
        let entries = lookup.table.get(&KeyCode::A).unwrap();
        assert_eq!(entries.len(), 1);
        assert!(entries[0].condition.is_none()); // Unconditional

        // Should be a Simple mapping
        if let BaseKeyMapping::Simple { from, to } = &entries[0].mapping {
            assert_eq!(*from, KeyCode::A);
            assert_eq!(*to, KeyCode::B);
        } else {
            panic!("Expected Simple mapping");
        }
    }

    #[test]
    fn test_from_device_config_conditional_mapping() {
        let config = create_test_device_config(vec![KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            }],
        )]);
        let lookup = KeyLookup::from_device_config(&config);

        // Should have one entry for key H
        let entries = lookup.table.get(&KeyCode::H).unwrap();
        assert_eq!(entries.len(), 1);

        // Should have a condition
        assert!(entries[0].condition.is_some());
        if let Some(Condition::ModifierActive(id)) = &entries[0].condition {
            assert_eq!(*id, 0);
        } else {
            panic!("Expected ModifierActive condition");
        }
    }

    #[test]
    fn test_from_device_config_mixed_mappings() {
        // Create config with both conditional and unconditional for same key
        let config = create_test_device_config(vec![
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            ),
            KeyMapping::simple(KeyCode::H, KeyCode::J), // Unconditional fallback
        ]);
        let lookup = KeyLookup::from_device_config(&config);

        let entries = lookup.table.get(&KeyCode::H).unwrap();
        assert_eq!(entries.len(), 2);

        // First entry should be conditional
        assert!(entries[0].condition.is_some());

        // Second entry should be unconditional
        assert!(entries[1].condition.is_none());
    }

    #[test]
    fn test_find_mapping_no_mapping() {
        let config = create_test_device_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
        let lookup = KeyLookup::from_device_config(&config);
        let state = DeviceState::new();

        // Key Z has no mapping
        let result = lookup.find_mapping(KeyCode::Z, &state);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_mapping_simple() {
        let config = create_test_device_config(vec![KeyMapping::simple(KeyCode::A, KeyCode::B)]);
        let lookup = KeyLookup::from_device_config(&config);
        let state = DeviceState::new();

        // Key A should map to B
        let result = lookup.find_mapping(KeyCode::A, &state);
        assert!(result.is_some());

        if let BaseKeyMapping::Simple { from, to } = result.unwrap() {
            assert_eq!(*from, KeyCode::A);
            assert_eq!(*to, KeyCode::B);
        } else {
            panic!("Expected Simple mapping");
        }
    }

    #[test]
    fn test_find_mapping_conditional_true() {
        let config = create_test_device_config(vec![KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            }],
        )]);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();
        state.set_modifier(0); // Activate modifier

        // Key H should map to Left when modifier active
        let result = lookup.find_mapping(KeyCode::H, &state);
        assert!(result.is_some());

        if let BaseKeyMapping::Simple { from, to } = result.unwrap() {
            assert_eq!(*from, KeyCode::H);
            assert_eq!(*to, KeyCode::Left);
        } else {
            panic!("Expected Simple mapping");
        }
    }

    #[test]
    fn test_find_mapping_conditional_false() {
        let config = create_test_device_config(vec![KeyMapping::conditional(
            Condition::ModifierActive(0),
            vec![BaseKeyMapping::Simple {
                from: KeyCode::H,
                to: KeyCode::Left,
            }],
        )]);
        let lookup = KeyLookup::from_device_config(&config);
        let state = DeviceState::new(); // Modifier not active

        // Key H should have no mapping when modifier not active
        let result = lookup.find_mapping(KeyCode::H, &state);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_mapping_conditional_before_unconditional() {
        // Conditional mapping first, unconditional fallback second
        let config = create_test_device_config(vec![
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            ),
            KeyMapping::simple(KeyCode::H, KeyCode::J),
        ]);
        let lookup = KeyLookup::from_device_config(&config);

        // Test with modifier active - should get conditional mapping
        let mut state = DeviceState::new();
        state.set_modifier(0);

        let result = lookup.find_mapping(KeyCode::H, &state);
        assert!(result.is_some());

        if let BaseKeyMapping::Simple { to, .. } = result.unwrap() {
            assert_eq!(*to, KeyCode::Left); // Conditional result
        } else {
            panic!("Expected Simple mapping");
        }

        // Test with modifier inactive - should get unconditional fallback
        let state2 = DeviceState::new();
        let result2 = lookup.find_mapping(KeyCode::H, &state2);
        assert!(result2.is_some());

        if let BaseKeyMapping::Simple { to, .. } = result2.unwrap() {
            assert_eq!(*to, KeyCode::J); // Unconditional fallback
        } else {
            panic!("Expected Simple mapping");
        }
    }

    #[test]
    fn test_extract_input_key_all_variants() {
        // Test Simple
        let simple = BaseKeyMapping::Simple {
            from: KeyCode::A,
            to: KeyCode::B,
        };
        assert_eq!(KeyLookup::extract_input_key(&simple), Some(KeyCode::A));

        // Test Modifier
        let modifier = BaseKeyMapping::Modifier {
            from: KeyCode::CapsLock,
            modifier_id: 0,
        };
        assert_eq!(
            KeyLookup::extract_input_key(&modifier),
            Some(KeyCode::CapsLock)
        );

        // Test Lock
        let lock = BaseKeyMapping::Lock {
            from: KeyCode::ScrollLock,
            lock_id: 0,
        };
        assert_eq!(
            KeyLookup::extract_input_key(&lock),
            Some(KeyCode::ScrollLock)
        );

        // Test TapHold
        let tap_hold = BaseKeyMapping::TapHold {
            from: KeyCode::Space,
            tap: KeyCode::Space,
            hold_modifier: 0,
            threshold_ms: 200,
        };
        assert_eq!(
            KeyLookup::extract_input_key(&tap_hold),
            Some(KeyCode::Space)
        );

        // Test ModifiedOutput
        let modified = BaseKeyMapping::ModifiedOutput {
            from: KeyCode::A,
            to: KeyCode::A,
            shift: true,
            ctrl: false,
            alt: false,
            win: false,
        };
        assert_eq!(KeyLookup::extract_input_key(&modified), Some(KeyCode::A));
    }

    #[test]
    fn test_find_mapping_with_device_matches() {
        use alloc::string::String;

        // Create config with device-specific mapping and fallback
        let config = create_test_device_config(vec![
            // Device-specific: Numpad1 -> F13 when on numpad device
            KeyMapping::conditional(
                Condition::DeviceMatches(String::from("*numpad*")),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::Numpad1,
                    to: KeyCode::F13,
                }],
            ),
            // Fallback: Numpad1 unchanged (pass through via no mapping)
        ]);
        let lookup = KeyLookup::from_device_config(&config);
        let state = DeviceState::new();

        // Test with matching device - should get device-specific mapping
        let result =
            lookup.find_mapping_with_device(KeyCode::Numpad1, &state, Some("usb-numpad-123"));
        assert!(result.is_some());
        if let BaseKeyMapping::Simple { to, .. } = result.unwrap() {
            assert_eq!(*to, KeyCode::F13);
        } else {
            panic!("Expected Simple mapping");
        }

        // Test with non-matching device - should find no mapping
        let result2 =
            lookup.find_mapping_with_device(KeyCode::Numpad1, &state, Some("usb-keyboard-456"));
        assert!(result2.is_none());

        // Test without device_id - should find no mapping (DeviceMatches never matches)
        let result3 = lookup.find_mapping(KeyCode::Numpad1, &state);
        assert!(result3.is_none());
    }

    #[test]
    fn test_find_mapping_device_with_fallback() {
        use alloc::string::String;

        // Create config with device-specific mapping AND unconditional fallback
        let config = create_test_device_config(vec![
            // Device-specific: Numpad1 -> F13 when on numpad device
            KeyMapping::conditional(
                Condition::DeviceMatches(String::from("*numpad*")),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::Numpad1,
                    to: KeyCode::F13,
                }],
            ),
            // Fallback: Numpad1 -> Numpad1 for all other devices
            KeyMapping::simple(KeyCode::Numpad1, KeyCode::Numpad1),
        ]);
        let lookup = KeyLookup::from_device_config(&config);
        let state = DeviceState::new();

        // Test with matching device - should get device-specific mapping
        let result =
            lookup.find_mapping_with_device(KeyCode::Numpad1, &state, Some("usb-numpad-123"));
        assert!(result.is_some());
        if let BaseKeyMapping::Simple { to, .. } = result.unwrap() {
            assert_eq!(*to, KeyCode::F13);
        } else {
            panic!("Expected Simple mapping");
        }

        // Test with non-matching device - should get fallback
        let result2 =
            lookup.find_mapping_with_device(KeyCode::Numpad1, &state, Some("usb-keyboard-456"));
        assert!(result2.is_some());
        if let BaseKeyMapping::Simple { to, .. } = result2.unwrap() {
            assert_eq!(*to, KeyCode::Numpad1);
        } else {
            panic!("Expected Simple mapping");
        }

        // Test without device_id - should get fallback
        let result3 = lookup.find_mapping(KeyCode::Numpad1, &state);
        assert!(result3.is_some());
        if let BaseKeyMapping::Simple { to, .. } = result3.unwrap() {
            assert_eq!(*to, KeyCode::Numpad1);
        } else {
            panic!("Expected Simple mapping");
        }
    }

    #[test]
    fn test_find_mapping_device_and_modifier_combined() {
        use alloc::string::String;

        // Create config where both device AND modifier must match
        // Note: Currently, conditions are single - for combined conditions,
        // we would need AllActive with ConditionItem::DeviceMatches
        // For now, test that device condition works alongside modifier conditions

        let config = create_test_device_config(vec![
            // Modifier-based: H -> Left when MD_00 active
            KeyMapping::conditional(
                Condition::ModifierActive(0),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::H,
                    to: KeyCode::Left,
                }],
            ),
            // Device-based: Numpad1 -> F13 when on numpad
            KeyMapping::conditional(
                Condition::DeviceMatches(String::from("*numpad*")),
                vec![BaseKeyMapping::Simple {
                    from: KeyCode::Numpad1,
                    to: KeyCode::F13,
                }],
            ),
        ]);
        let lookup = KeyLookup::from_device_config(&config);
        let mut state = DeviceState::new();

        // Test modifier condition (independent of device)
        state.set_modifier(0);
        let result_h = lookup.find_mapping_with_device(KeyCode::H, &state, Some("any-device"));
        assert!(result_h.is_some());
        if let BaseKeyMapping::Simple { to, .. } = result_h.unwrap() {
            assert_eq!(*to, KeyCode::Left);
        }

        // Test device condition (independent of modifiers)
        state.clear_modifier(0);
        let result_numpad =
            lookup.find_mapping_with_device(KeyCode::Numpad1, &state, Some("usb-numpad-123"));
        assert!(result_numpad.is_some());
        if let BaseKeyMapping::Simple { to, .. } = result_numpad.unwrap() {
            assert_eq!(*to, KeyCode::F13);
        }
    }
}
