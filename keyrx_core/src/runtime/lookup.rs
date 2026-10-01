//! Key lookup table for O(1) mapping resolution
//!
//! This module provides `KeyLookup` for efficient key-to-mapping resolution
//! using a HashMap-based lookup table.

extern crate alloc;
use alloc::vec::Vec;
use hashbrown::HashMap;

use crate::config::lint::holds_whenever;
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
                    let entries = table.entry(key).or_insert_with(Vec::new);
                    // A more specific layer beats a more general one declared
                    // earlier: `when(["MD_00","MD_01"])` after `when("MD_00")`
                    // would otherwise never fire (the general entry, tried
                    // first, holds whenever the specific one does).
                    let at = entries
                        .iter()
                        .position(|e| strictly_more_general(e.condition.as_ref(), condition))
                        .unwrap_or(entries.len());
                    entries.insert(
                        at,
                        LookupEntry {
                            mapping: base_mapping.clone(),
                            condition: Some(condition.clone()),
                        },
                    );
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
        identities: &[&str],
    ) -> Option<&'a LookupEntry> {
        entries
            .iter()
            .find(|entry| Self::entry_matches(entry, state, identities))
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
        self.find_mapping_for_identities(key, state, device_id.as_slice())
    }

    /// Like [`find_mapping_with_device`](Self::find_mapping_with_device) for a
    /// device known by several identities (id, name, path, serial): a
    /// `DeviceMatches` condition holds if its pattern matches any of them.
    pub fn find_mapping_for_identities(
        &self,
        key: KeyCode,
        state: &DeviceState,
        identities: &[&str],
    ) -> Option<&BaseKeyMapping> {
        let entries = self.table.get(&key)?;
        Self::find_matching_entry(entries, state, identities).map(|entry| &entry.mapping)
    }

    /// Checks if a mapping entry matches the current state.
    fn entry_matches(entry: &LookupEntry, state: &DeviceState, identities: &[&str]) -> bool {
        match &entry.condition {
            Some(condition) => state.evaluate_condition_for_identities(condition, identities),
            None => true, // Unconditional mapping always matches
        }
    }

    /// Conditions of every mapping, per input key, in the order they are
    /// tried (`None` = unconditional). Used by the dead-mapping lint.
    pub fn conditions_in_lookup_order(
        &self,
    ) -> impl Iterator<Item = (KeyCode, Vec<Option<&Condition>>)> + '_ {
        self.table
            .iter()
            .map(|(key, entries)| (*key, entries.iter().map(|e| e.condition.as_ref()).collect()))
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
        Some(mapping.source_key())
    }
}

/// `general` holds whenever `specific` holds, but not the other way round.
fn strictly_more_general(general: Option<&Condition>, specific: &Condition) -> bool {
    general.is_some()
        && holds_whenever(general, Some(specific))
        && !holds_whenever(Some(specific), general)
}

#[cfg(test)]
#[path = "lookup_tests.rs"]
mod tests;
