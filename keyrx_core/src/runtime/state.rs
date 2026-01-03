//! Device state management with bit vectors
//!
//! This module provides `DeviceState` for tracking modifier and lock state
//! using efficient 255-bit vectors, plus tap-hold processor state.

extern crate alloc;

use arrayvec::ArrayVec;
use bitvec::prelude::*;

use crate::config::{Condition, ConditionItem, KeyCode};
use crate::runtime::tap_hold::{TapHoldProcessor, DEFAULT_MAX_PENDING};

/// Maximum valid modifier/lock ID (0-254, ID 255 is reserved)
const MAX_VALID_ID: u8 = 254;

/// Maximum number of simultaneously pressed keys to track
/// This should cover even the most extreme cases (10-finger roll)
const MAX_PRESSED_KEYS: usize = 32;

/// Maximum number of output keys per input key
/// Covers ModifiedOutput with all 4 modifiers: Shift+Ctrl+Alt+Win+PrimaryKey = 5 keys
const MAX_OUTPUT_KEYS_PER_INPUT: usize = 5;

/// Device state tracking modifier, lock, and pressed key state
///
/// Uses 255-bit vectors for efficient state management:
/// - Modifiers: Temporary state (set on press, clear on release)
/// - Locks: Toggle state (toggle on press, ignore release)
/// - Pressed keys: Maps input keys to multiple output keys for press/release consistency
///
/// Bit layout: IDs 0-254 are valid, ID 255 is reserved and will be rejected.
///
/// # Press/Release Consistency
///
/// When a key press is remapped (e.g., A→Shift+B), we track ALL output keys.
/// When A is released, we release all tracked keys in reverse order,
/// even if the mapping has changed due to modifier state changes. This prevents stuck keys.
///
/// # Example
///
/// ```rust,ignore
/// use keyrx_core::runtime::DeviceState;
///
/// let mut state = DeviceState::new();
/// state.set_modifier(0);
/// assert!(state.is_modifier_active(0));
/// ```
pub struct DeviceState {
    /// Modifier state (255 bits, IDs 0-254)
    modifiers: BitVec<u8, Lsb0>,
    /// Lock state (255 bits, IDs 0-254)
    locks: BitVec<u8, Lsb0>,
    /// Tap-hold processor for dual-function keys
    tap_hold: TapHoldProcessor<DEFAULT_MAX_PENDING>,
    /// Pressed key tracking: (input_key, [output_keys]) pairs
    /// This ensures release events match their corresponding press events
    /// Supports multiple output keys per input (e.g., Shift+Z generates 2 keys)
    pressed_keys:
        ArrayVec<(KeyCode, ArrayVec<KeyCode, MAX_OUTPUT_KEYS_PER_INPUT>), MAX_PRESSED_KEYS>,
}

impl DeviceState {
    /// Creates a new device state with all bits cleared
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let state = DeviceState::new();
    /// assert!(!state.is_modifier_active(0));
    /// assert!(!state.is_lock_active(0));
    /// ```
    pub fn new() -> Self {
        Self {
            modifiers: bitvec![u8, Lsb0; 0; 255],
            locks: bitvec![u8, Lsb0; 0; 255],
            tap_hold: TapHoldProcessor::new(),
            pressed_keys: ArrayVec::new(),
        }
    }

    /// Validates that a modifier/lock ID is in valid range (0-254)
    ///
    /// Returns true if valid, logs error and returns false if invalid (>254).
    #[inline]
    fn validate_id(id: u8) -> bool {
        if id > MAX_VALID_ID {
            // In production, this would use proper logging
            // For now, we just return false
            false
        } else {
            true
        }
    }

    /// Sets a modifier bit to active
    ///
    /// # Arguments
    ///
    /// * `id` - Modifier ID (0-254)
    ///
    /// # Returns
    ///
    /// Returns `true` if successful, `false` if ID is invalid (>254)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut state = DeviceState::new();
    /// assert!(state.set_modifier(0));
    /// assert!(state.is_modifier_active(0));
    /// assert!(!state.set_modifier(255)); // Invalid ID
    /// ```
    pub fn set_modifier(&mut self, id: u8) -> bool {
        if !Self::validate_id(id) {
            return false;
        }
        self.modifiers.set(id as usize, true);
        true
    }

    /// Clears a modifier bit to inactive
    ///
    /// # Arguments
    ///
    /// * `id` - Modifier ID (0-254)
    ///
    /// # Returns
    ///
    /// Returns `true` if successful, `false` if ID is invalid (>254)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut state = DeviceState::new();
    /// state.set_modifier(0);
    /// assert!(state.clear_modifier(0));
    /// assert!(!state.is_modifier_active(0));
    /// ```
    pub fn clear_modifier(&mut self, id: u8) -> bool {
        if !Self::validate_id(id) {
            return false;
        }
        self.modifiers.set(id as usize, false);
        true
    }

    /// Toggles a lock bit (OFF→ON or ON→OFF)
    ///
    /// # Arguments
    ///
    /// * `id` - Lock ID (0-254)
    ///
    /// # Returns
    ///
    /// Returns `true` if successful, `false` if ID is invalid (>254)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut state = DeviceState::new();
    /// assert!(state.toggle_lock(0)); // OFF → ON
    /// assert!(state.is_lock_active(0));
    /// assert!(state.toggle_lock(0)); // ON → OFF
    /// assert!(!state.is_lock_active(0));
    /// ```
    pub fn toggle_lock(&mut self, id: u8) -> bool {
        if !Self::validate_id(id) {
            return false;
        }
        let current = self.locks[id as usize];
        self.locks.set(id as usize, !current);
        true
    }

    /// Checks if a modifier is active
    ///
    /// # Arguments
    ///
    /// * `id` - Modifier ID (0-254)
    ///
    /// # Returns
    ///
    /// Returns `true` if modifier is active, `false` if inactive or ID is invalid
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut state = DeviceState::new();
    /// assert!(!state.is_modifier_active(0));
    /// state.set_modifier(0);
    /// assert!(state.is_modifier_active(0));
    /// ```
    pub fn is_modifier_active(&self, id: u8) -> bool {
        if !Self::validate_id(id) {
            return false;
        }
        self.modifiers[id as usize]
    }

    /// Checks if a lock is active
    ///
    /// # Arguments
    ///
    /// * `id` - Lock ID (0-254)
    ///
    /// # Returns
    ///
    /// Returns `true` if lock is active, `false` if inactive or ID is invalid
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut state = DeviceState::new();
    /// assert!(!state.is_lock_active(0));
    /// state.toggle_lock(0);
    /// assert!(state.is_lock_active(0));
    /// ```
    pub fn is_lock_active(&self, id: u8) -> bool {
        if !Self::validate_id(id) {
            return false;
        }
        self.locks[id as usize]
    }

    /// Evaluates a condition against the current device state
    ///
    /// This is a convenience method that calls `evaluate_condition_with_device`
    /// with `device_id = None`. Use this for conditions that don't involve
    /// device matching (ModifierActive, LockActive, AllActive, NotActive).
    ///
    /// Note: DeviceMatches conditions will always return false when called
    /// without a device_id. Use `evaluate_condition_with_device` for those.
    ///
    /// # Arguments
    ///
    /// * `condition` - The condition to evaluate
    ///
    /// # Returns
    ///
    /// Returns `true` if the condition is satisfied, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use keyrx_core::runtime::DeviceState;
    /// use keyrx_core::config::{Condition, ConditionItem};
    ///
    /// let mut state = DeviceState::new();
    /// state.set_modifier(0);
    ///
    /// // Single modifier active
    /// assert!(state.evaluate_condition(&Condition::ModifierActive(0)));
    ///
    /// // All conditions must be true
    /// state.toggle_lock(1);
    /// let all_cond = Condition::AllActive(vec![
    ///     ConditionItem::ModifierActive(0),
    ///     ConditionItem::LockActive(1),
    /// ]);
    /// assert!(state.evaluate_condition(&all_cond));
    ///
    /// // Not active
    /// let not_cond = Condition::NotActive(vec![ConditionItem::ModifierActive(2)]);
    /// assert!(state.evaluate_condition(&not_cond)); // MD_02 is not active
    /// ```
    pub fn evaluate_condition(&self, condition: &Condition) -> bool {
        self.evaluate_condition_with_device(condition, None)
    }

    /// Evaluates a condition against the current device state and optional device ID
    ///
    /// This is the full version of condition evaluation that supports device matching.
    /// For conditions that don't involve device matching, you can use `evaluate_condition()`.
    ///
    /// # Arguments
    ///
    /// * `condition` - The condition to evaluate
    /// * `device_id` - Optional device ID from the current event
    ///
    /// # Returns
    ///
    /// Returns `true` if the condition is satisfied, `false` otherwise
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use keyrx_core::runtime::DeviceState;
    /// use keyrx_core::config::Condition;
    ///
    /// let state = DeviceState::new();
    ///
    /// // Device matching condition
    /// let cond = Condition::DeviceMatches("numpad".to_string());
    /// assert!(state.evaluate_condition_with_device(&cond, Some("numpad")));
    /// assert!(!state.evaluate_condition_with_device(&cond, Some("keyboard")));
    /// assert!(!state.evaluate_condition_with_device(&cond, None));
    /// ```
    pub fn evaluate_condition_with_device(
        &self,
        condition: &Condition,
        device_id: Option<&str>,
    ) -> bool {
        match condition {
            // Single modifier active
            Condition::ModifierActive(id) => self.is_modifier_active(*id),

            // Single lock active
            Condition::LockActive(id) => self.is_lock_active(*id),

            // All conditions must be true (AND logic)
            Condition::AllActive(items) => {
                items.iter().all(|item| self.evaluate_condition_item(item))
            }

            // All conditions must be false (NOT logic)
            Condition::NotActive(items) => {
                items.iter().all(|item| !self.evaluate_condition_item(item))
            }

            // Device ID matches pattern
            Condition::DeviceMatches(pattern) => Self::matches_device_pattern(device_id, pattern),
        }
    }

    /// Matches a device ID against a pattern
    ///
    /// Supports simple glob patterns with `*` wildcard:
    /// - Exact match: "device-123" matches only "device-123"
    /// - Prefix: "usb-*" matches "usb-keyboard", "usb-numpad", etc.
    /// - Suffix: "*-keyboard" matches "usb-keyboard", "bt-keyboard", etc.
    /// - Contains: "*numpad*" matches "usb-numpad-123", "my-numpad", etc.
    ///
    /// Returns false if device_id is None.
    fn matches_device_pattern(device_id: Option<&str>, pattern: &str) -> bool {
        let Some(id) = device_id else {
            return false;
        };

        // Handle glob patterns with *
        if pattern.contains('*') {
            let parts: alloc::vec::Vec<&str> = pattern.split('*').collect();
            match parts.len() {
                1 => {
                    // No actual * (shouldn't happen but handle it)
                    id == pattern
                }
                2 => {
                    // Single * - either prefix, suffix, or empty on one side
                    let (prefix, suffix) = (parts[0], parts[1]);
                    if prefix.is_empty() && suffix.is_empty() {
                        // Pattern is just "*" - matches everything
                        true
                    } else if prefix.is_empty() {
                        // *suffix
                        id.ends_with(suffix)
                    } else if suffix.is_empty() {
                        // prefix*
                        id.starts_with(prefix)
                    } else {
                        // prefix*suffix
                        id.starts_with(prefix) && id.ends_with(suffix)
                    }
                }
                3 => {
                    // Two *s - typically *contains*
                    let (prefix, middle, suffix) = (parts[0], parts[1], parts[2]);
                    if prefix.is_empty() && suffix.is_empty() {
                        // *middle*
                        id.contains(middle)
                    } else {
                        // More complex pattern - do simple check
                        id.starts_with(prefix) && id.ends_with(suffix) && id.contains(middle)
                    }
                }
                _ => {
                    // Complex pattern with multiple * - just check if all parts exist in order
                    // This is a simplified implementation
                    let mut remaining = id;
                    for (i, part) in parts.iter().enumerate() {
                        if part.is_empty() {
                            continue;
                        }
                        if i == 0 {
                            // First part must be prefix
                            if !remaining.starts_with(part) {
                                return false;
                            }
                            remaining = &remaining[part.len()..];
                        } else if i == parts.len() - 1 {
                            // Last part must be suffix
                            if !remaining.ends_with(part) {
                                return false;
                            }
                        } else {
                            // Middle parts must exist somewhere
                            if let Some(pos) = remaining.find(part) {
                                remaining = &remaining[pos + part.len()..];
                            } else {
                                return false;
                            }
                        }
                    }
                    true
                }
            }
        } else {
            // Exact match
            id == pattern
        }
    }

    /// Evaluates a single condition item
    ///
    /// Helper method for evaluating ConditionItem in composite conditions.
    fn evaluate_condition_item(&self, item: &ConditionItem) -> bool {
        match item {
            ConditionItem::ModifierActive(id) => self.is_modifier_active(*id),
            ConditionItem::LockActive(id) => self.is_lock_active(*id),
        }
    }

    /// Returns a mutable reference to the tap-hold processor
    ///
    /// The processor manages the state machine for dual-function (tap-hold) keys.
    pub fn tap_hold_processor(&mut self) -> &mut TapHoldProcessor<DEFAULT_MAX_PENDING> {
        &mut self.tap_hold
    }

    /// Returns an immutable reference to the tap-hold processor
    pub fn tap_hold_processor_ref(&self) -> &TapHoldProcessor<DEFAULT_MAX_PENDING> {
        &self.tap_hold
    }

    /// Records that an input key was pressed and remapped to output key(s)
    ///
    /// This ensures that when the input key is released, we release ALL output keys,
    /// even if the mapping has changed due to modifier state changes.
    ///
    /// # Arguments
    ///
    /// * `input` - The physical key that was pressed
    /// * `outputs` - The keys that were sent to the OS (e.g., [LShift, Z] for Shift+Z)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // User pressed A, but MD_02 was active, so we sent Shift+B
    /// state.record_press(KeyCode::A, &[KeyCode::LShift, KeyCode::B]);
    /// // Later, when A is released, we'll release B then LShift (even if MD_02 is now inactive)
    /// ```
    pub fn record_press(&mut self, input: KeyCode, outputs: &[KeyCode]) {
        // If this input key is already tracked, update its outputs
        // This handles the case where the same key is pressed multiple times
        if let Some(entry) = self.pressed_keys.iter_mut().find(|(k, _)| *k == input) {
            entry.1.clear();
            for &output in outputs {
                let _ = entry.1.try_push(output);
            }
            return;
        }

        // Add new tracking entry
        let mut output_vec = ArrayVec::new();
        for &output in outputs {
            let _ = output_vec.try_push(output);
        }

        // Ignore if array is full - unlikely scenario
        let _ = self.pressed_keys.try_push((input, output_vec));
    }

    /// Gets the output keys that should be released for a given input key
    ///
    /// Returns the tracked output keys if found, otherwise returns the input key itself.
    /// This ensures press/release consistency even when mappings change.
    ///
    /// # Arguments
    ///
    /// * `input` - The physical key that is being released
    ///
    /// # Returns
    ///
    /// The output keys that should be released (either tracked keys or input itself)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// state.record_press(KeyCode::A, &[KeyCode::LShift, KeyCode::B]);
    /// let outputs = state.get_release_key(KeyCode::A); // Returns [LShift, B]
    /// state.clear_press(KeyCode::A);
    /// ```
    pub fn get_release_key(&self, input: KeyCode) -> ArrayVec<KeyCode, MAX_OUTPUT_KEYS_PER_INPUT> {
        if let Some((_, outputs)) = self.pressed_keys.iter().find(|(k, _)| *k == input) {
            outputs.clone()
        } else {
            let mut result = ArrayVec::new();
            let _ = result.try_push(input);
            result
        }
    }

    /// Clears the press tracking for an input key after it's been released
    ///
    /// # Arguments
    ///
    /// * `input` - The physical key that was released
    pub fn clear_press(&mut self, input: KeyCode) {
        self.pressed_keys.retain(|(k, _)| *k != input);
    }

    /// Clears all pressed key tracking (for testing or emergency reset)
    pub fn clear_all_pressed(&mut self) {
        self.pressed_keys.clear();
    }
}

impl Default for DeviceState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::vec;

    #[test]
    fn test_new_creates_zeroed_state() {
        let state = DeviceState::new();
        assert!(!state.is_modifier_active(0));
        assert!(!state.is_modifier_active(127));
        assert!(!state.is_modifier_active(254));
        assert!(!state.is_lock_active(0));
        assert!(!state.is_lock_active(127));
        assert!(!state.is_lock_active(254));
    }

    #[test]
    fn test_set_modifier_valid_ids() {
        let mut state = DeviceState::new();

        // Test ID 0
        assert!(state.set_modifier(0));
        assert!(state.is_modifier_active(0));

        // Test ID 127 (middle)
        assert!(state.set_modifier(127));
        assert!(state.is_modifier_active(127));

        // Test ID 254 (max valid)
        assert!(state.set_modifier(254));
        assert!(state.is_modifier_active(254));
    }

    #[test]
    fn test_set_modifier_invalid_id() {
        let mut state = DeviceState::new();

        // ID 255 should be rejected
        assert!(!state.set_modifier(255));
        // Modifier should not be set
        assert!(!state.is_modifier_active(255));
    }

    #[test]
    fn test_clear_modifier() {
        let mut state = DeviceState::new();

        // Set then clear
        state.set_modifier(0);
        assert!(state.is_modifier_active(0));
        assert!(state.clear_modifier(0));
        assert!(!state.is_modifier_active(0));

        // Clear invalid ID
        assert!(!state.clear_modifier(255));
    }

    #[test]
    fn test_toggle_lock_cycles() {
        let mut state = DeviceState::new();

        // OFF → ON
        assert!(state.toggle_lock(0));
        assert!(state.is_lock_active(0));

        // ON → OFF
        assert!(state.toggle_lock(0));
        assert!(!state.is_lock_active(0));

        // OFF → ON again
        assert!(state.toggle_lock(0));
        assert!(state.is_lock_active(0));
    }

    #[test]
    fn test_toggle_lock_invalid_id() {
        let mut state = DeviceState::new();

        // ID 255 should be rejected
        assert!(!state.toggle_lock(255));
        assert!(!state.is_lock_active(255));
    }

    #[test]
    fn test_evaluate_condition_modifier_active() {
        let mut state = DeviceState::new();
        state.set_modifier(0);

        let cond = Condition::ModifierActive(0);
        assert!(state.evaluate_condition(&cond));

        let cond_inactive = Condition::ModifierActive(1);
        assert!(!state.evaluate_condition(&cond_inactive));
    }

    #[test]
    fn test_evaluate_condition_lock_active() {
        let mut state = DeviceState::new();
        state.toggle_lock(1);

        let cond = Condition::LockActive(1);
        assert!(state.evaluate_condition(&cond));

        let cond_inactive = Condition::LockActive(2);
        assert!(!state.evaluate_condition(&cond_inactive));
    }

    #[test]
    fn test_evaluate_condition_all_active() {
        let mut state = DeviceState::new();
        state.set_modifier(0);
        state.toggle_lock(1);

        // Both conditions true
        let cond = Condition::AllActive(vec![
            ConditionItem::ModifierActive(0),
            ConditionItem::LockActive(1),
        ]);
        assert!(state.evaluate_condition(&cond));

        // One condition false
        let cond_partial = Condition::AllActive(vec![
            ConditionItem::ModifierActive(0),
            ConditionItem::LockActive(2), // Not active
        ]);
        assert!(!state.evaluate_condition(&cond_partial));

        // All conditions false
        let cond_none = Condition::AllActive(vec![
            ConditionItem::ModifierActive(10),
            ConditionItem::LockActive(11),
        ]);
        assert!(!state.evaluate_condition(&cond_none));
    }

    #[test]
    fn test_evaluate_condition_not_active() {
        let mut state = DeviceState::new();
        state.set_modifier(0);

        // NOT(inactive) = true
        let cond_true = Condition::NotActive(vec![ConditionItem::ModifierActive(1)]);
        assert!(state.evaluate_condition(&cond_true));

        // NOT(active) = false
        let cond_false = Condition::NotActive(vec![ConditionItem::ModifierActive(0)]);
        assert!(!state.evaluate_condition(&cond_false));

        // NOT(MD_00 AND LK_01) with MD_00 active, LK_01 inactive = false (not all inactive)
        let cond_mixed = Condition::NotActive(vec![
            ConditionItem::ModifierActive(0), // Active
            ConditionItem::LockActive(1),     // Inactive
        ]);
        assert!(!state.evaluate_condition(&cond_mixed));

        // NOT(MD_10 AND LK_11) with both inactive = true
        let cond_both_inactive = Condition::NotActive(vec![
            ConditionItem::ModifierActive(10),
            ConditionItem::LockActive(11),
        ]);
        assert!(state.evaluate_condition(&cond_both_inactive));
    }

    #[test]
    fn test_multiple_modifiers_independent() {
        let mut state = DeviceState::new();

        state.set_modifier(0);
        state.set_modifier(1);
        state.set_modifier(254);

        assert!(state.is_modifier_active(0));
        assert!(state.is_modifier_active(1));
        assert!(state.is_modifier_active(254));

        state.clear_modifier(1);
        assert!(state.is_modifier_active(0));
        assert!(!state.is_modifier_active(1));
        assert!(state.is_modifier_active(254));
    }

    #[test]
    fn test_multiple_locks_independent() {
        let mut state = DeviceState::new();

        state.toggle_lock(0); // ON
        state.toggle_lock(1); // ON
        state.toggle_lock(2); // ON

        assert!(state.is_lock_active(0));
        assert!(state.is_lock_active(1));
        assert!(state.is_lock_active(2));

        state.toggle_lock(1); // OFF
        assert!(state.is_lock_active(0));
        assert!(!state.is_lock_active(1));
        assert!(state.is_lock_active(2));
    }

    // Property-based tests
    //
    // These tests verify state management invariants using proptest to generate
    // random test cases, ensuring correctness across a wide range of inputs.
    #[cfg(not(target_arch = "wasm32"))]
    mod proptests {
        use super::*;
        use proptest::prelude::*;

        // Property test: modifier state is always valid (only bits 0-254 can be set)
        //
        // This test verifies the invariant that IDs >254 are always rejected
        // and never modify the state, while valid IDs (0-254) work correctly.
        proptest! {
            #[test]
            fn prop_modifier_state_valid(id in 0u8..=255) {
                let mut state = DeviceState::new();

                if id <= MAX_VALID_ID {
                    // Valid IDs should be settable
                    assert!(state.set_modifier(id), "set_modifier({}) should succeed", id);
                    assert!(state.is_modifier_active(id), "modifier {} should be active after set", id);

                    // Should be clearable
                    assert!(state.clear_modifier(id), "clear_modifier({}) should succeed", id);
                    assert!(!state.is_modifier_active(id), "modifier {} should be inactive after clear", id);
                } else {
                    // Invalid IDs (255) should be rejected
                    assert!(!state.set_modifier(id), "set_modifier({}) should fail for invalid ID", id);
                    assert!(!state.is_modifier_active(id), "modifier {} should never be active for invalid ID", id);

                    assert!(!state.clear_modifier(id), "clear_modifier({}) should fail for invalid ID", id);
                }
            }
        }

        // Property test: lock toggle cycles correctly (OFF→ON→OFF→...)
        //
        // This test verifies that toggle_lock correctly cycles state based on
        // the number of toggles: even toggles = OFF, odd toggles = ON.
        proptest! {
            #[test]
            fn prop_lock_toggle_cycles(id in 0u8..=MAX_VALID_ID, toggle_count in 0usize..=20) {
                let mut state = DeviceState::new();

                // Initial state should be OFF
                assert!(!state.is_lock_active(id));

                // Apply toggles
                for _ in 0..toggle_count {
                    assert!(state.toggle_lock(id), "toggle_lock({}) should succeed", id);
                }

                // Final state should match parity: odd toggles = ON, even toggles = OFF
                let expected_active = toggle_count % 2 == 1;
                assert_eq!(
                    state.is_lock_active(id),
                    expected_active,
                    "After {} toggles, lock {} should be {} (got {})",
                    toggle_count,
                    id,
                    if expected_active { "ON" } else { "OFF" },
                    if state.is_lock_active(id) { "ON" } else { "OFF" }
                );
            }
        }

        // Property test: invalid lock IDs are always rejected
        //
        // This test verifies that ID 255 can never be toggled.
        proptest! {
            #[test]
            fn prop_lock_invalid_id_rejected(toggle_count in 0usize..=10) {
                let mut state = DeviceState::new();

                // Apply toggles to invalid ID 255
                for _ in 0..toggle_count {
                    assert!(!state.toggle_lock(255), "toggle_lock(255) should fail");
                }

                // Lock 255 should never be active
                assert!(!state.is_lock_active(255), "lock 255 should never be active");
            }
        }

        // Property test: set/clear operations are independent
        //
        // This test verifies that setting/clearing one modifier doesn't affect others.
        proptest! {
            #[test]
            fn prop_modifiers_independent(
                id1 in 0u8..=MAX_VALID_ID,
                id2 in 0u8..=MAX_VALID_ID,
                id3 in 0u8..=MAX_VALID_ID
            ) {
                // Use distinct IDs
                prop_assume!(id1 != id2 && id2 != id3 && id1 != id3);

                let mut state = DeviceState::new();

                // Set all three
                state.set_modifier(id1);
                state.set_modifier(id2);
                state.set_modifier(id3);

                assert!(state.is_modifier_active(id1));
                assert!(state.is_modifier_active(id2));
                assert!(state.is_modifier_active(id3));

                // Clear middle one
                state.clear_modifier(id2);

                // id2 should be inactive, others still active
                assert!(state.is_modifier_active(id1));
                assert!(!state.is_modifier_active(id2));
                assert!(state.is_modifier_active(id3));
            }
        }

        // Property test: locks are independent
        //
        // This test verifies that toggling one lock doesn't affect others.
        proptest! {
            #[test]
            fn prop_locks_independent(
                id1 in 0u8..=MAX_VALID_ID,
                id2 in 0u8..=MAX_VALID_ID,
                id3 in 0u8..=MAX_VALID_ID
            ) {
                // Use distinct IDs
                prop_assume!(id1 != id2 && id2 != id3 && id1 != id3);

                let mut state = DeviceState::new();

                // Toggle all three ON
                state.toggle_lock(id1);
                state.toggle_lock(id2);
                state.toggle_lock(id3);

                assert!(state.is_lock_active(id1));
                assert!(state.is_lock_active(id2));
                assert!(state.is_lock_active(id3));

                // Toggle middle one OFF
                state.toggle_lock(id2);

                // id2 should be inactive, others still active
                assert!(state.is_lock_active(id1));
                assert!(!state.is_lock_active(id2));
                assert!(state.is_lock_active(id3));
            }
        }
    }

    #[test]
    fn test_tap_hold_processor_accessors() {
        use crate::config::KeyCode;
        use crate::runtime::tap_hold::TapHoldConfig;

        let mut state = DeviceState::new();

        // Test mutable accessor - register a tap-hold config
        let config = TapHoldConfig::new(KeyCode::Escape, 0, 200_000);
        let added = state
            .tap_hold_processor()
            .register_tap_hold(KeyCode::CapsLock, config);
        assert!(added);

        // Test immutable accessor - verify the config was registered
        assert!(state
            .tap_hold_processor_ref()
            .is_tap_hold_key(KeyCode::CapsLock));
        assert!(!state.tap_hold_processor_ref().is_tap_hold_key(KeyCode::A));
    }

    // Device pattern matching tests
    mod device_pattern_tests {
        use super::*;
        use alloc::string::String;

        #[test]
        fn test_exact_match() {
            // Exact match should work
            assert!(DeviceState::matches_device_pattern(
                Some("usb-numpad-123"),
                "usb-numpad-123"
            ));
            // Different string should not match
            assert!(!DeviceState::matches_device_pattern(
                Some("usb-keyboard-456"),
                "usb-numpad-123"
            ));
            // None device_id should not match
            assert!(!DeviceState::matches_device_pattern(None, "usb-numpad-123"));
        }

        #[test]
        fn test_wildcard_matches_all() {
            // Pattern "*" matches everything
            assert!(DeviceState::matches_device_pattern(Some("anything"), "*"));
            assert!(DeviceState::matches_device_pattern(Some(""), "*"));
            assert!(DeviceState::matches_device_pattern(
                Some("usb-numpad-123"),
                "*"
            ));
            // But still not None
            assert!(!DeviceState::matches_device_pattern(None, "*"));
        }

        #[test]
        fn test_prefix_pattern() {
            // Pattern "usb-*" matches anything starting with "usb-"
            assert!(DeviceState::matches_device_pattern(
                Some("usb-numpad"),
                "usb-*"
            ));
            assert!(DeviceState::matches_device_pattern(
                Some("usb-keyboard"),
                "usb-*"
            ));
            assert!(DeviceState::matches_device_pattern(Some("usb-"), "usb-*"));
            // But not things that don't start with "usb-"
            assert!(!DeviceState::matches_device_pattern(
                Some("bt-keyboard"),
                "usb-*"
            ));
            assert!(!DeviceState::matches_device_pattern(
                Some("keyboard-usb"),
                "usb-*"
            ));
        }

        #[test]
        fn test_suffix_pattern() {
            // Pattern "*-keyboard" matches anything ending with "-keyboard"
            assert!(DeviceState::matches_device_pattern(
                Some("usb-keyboard"),
                "*-keyboard"
            ));
            assert!(DeviceState::matches_device_pattern(
                Some("bt-keyboard"),
                "*-keyboard"
            ));
            assert!(DeviceState::matches_device_pattern(
                Some("-keyboard"),
                "*-keyboard"
            ));
            // But not things that don't end with "-keyboard"
            assert!(!DeviceState::matches_device_pattern(
                Some("keyboard-usb"),
                "*-keyboard"
            ));
            assert!(!DeviceState::matches_device_pattern(
                Some("usb-numpad"),
                "*-keyboard"
            ));
        }

        #[test]
        fn test_contains_pattern() {
            // Pattern "*numpad*" matches anything containing "numpad"
            assert!(DeviceState::matches_device_pattern(
                Some("usb-numpad-123"),
                "*numpad*"
            ));
            assert!(DeviceState::matches_device_pattern(
                Some("numpad"),
                "*numpad*"
            ));
            assert!(DeviceState::matches_device_pattern(
                Some("my-numpad-device"),
                "*numpad*"
            ));
            // But not things that don't contain "numpad"
            assert!(!DeviceState::matches_device_pattern(
                Some("usb-keyboard"),
                "*numpad*"
            ));
            assert!(!DeviceState::matches_device_pattern(
                Some("numpd"),
                "*numpad*"
            ));
        }

        #[test]
        fn test_prefix_and_suffix_pattern() {
            // Pattern "usb-*-keyboard" matches "usb-...-keyboard"
            assert!(DeviceState::matches_device_pattern(
                Some("usb-logitech-keyboard"),
                "usb-*-keyboard"
            ));
            assert!(DeviceState::matches_device_pattern(
                Some("usb-keyboard"),
                "usb-*-keyboard"
            ));
            // But not mismatched
            assert!(!DeviceState::matches_device_pattern(
                Some("bt-logitech-keyboard"),
                "usb-*-keyboard"
            ));
            assert!(!DeviceState::matches_device_pattern(
                Some("usb-logitech-numpad"),
                "usb-*-keyboard"
            ));
        }

        #[test]
        fn test_evaluate_condition_device_matches() {
            let state = DeviceState::new();

            // Exact match condition
            let cond = Condition::DeviceMatches(String::from("usb-numpad-123"));
            assert!(state.evaluate_condition_with_device(&cond, Some("usb-numpad-123")));
            assert!(!state.evaluate_condition_with_device(&cond, Some("usb-keyboard")));
            assert!(!state.evaluate_condition_with_device(&cond, None));

            // Wildcard pattern condition
            let cond_wildcard = Condition::DeviceMatches(String::from("*numpad*"));
            assert!(state.evaluate_condition_with_device(&cond_wildcard, Some("usb-numpad-123")));
            assert!(state.evaluate_condition_with_device(&cond_wildcard, Some("my-numpad")));
            assert!(!state.evaluate_condition_with_device(&cond_wildcard, Some("keyboard")));

            // evaluate_condition (without device) always returns false for DeviceMatches
            assert!(!state.evaluate_condition(&cond));
            assert!(!state.evaluate_condition(&cond_wildcard));
        }

        #[test]
        fn test_device_pattern_prefix() {
            let state = DeviceState::new();

            // Prefix pattern (usb-*)
            let cond = Condition::DeviceMatches(String::from("usb-*"));
            assert!(state.evaluate_condition_with_device(&cond, Some("usb-keyboard")));
            assert!(state.evaluate_condition_with_device(&cond, Some("usb-numpad-123")));
            assert!(state.evaluate_condition_with_device(&cond, Some("usb-")));
            assert!(!state.evaluate_condition_with_device(&cond, Some("serial-usb-device")));
            assert!(!state.evaluate_condition_with_device(&cond, Some("usb"))); // No hyphen
        }

        #[test]
        fn test_device_pattern_suffix() {
            let state = DeviceState::new();

            // Suffix pattern (*-keyboard)
            let cond = Condition::DeviceMatches(String::from("*-keyboard"));
            assert!(state.evaluate_condition_with_device(&cond, Some("usb-keyboard")));
            assert!(state.evaluate_condition_with_device(&cond, Some("at-translated-keyboard")));
            assert!(state.evaluate_condition_with_device(&cond, Some("-keyboard")));
            assert!(!state.evaluate_condition_with_device(&cond, Some("keyboard-usb")));
            assert!(!state.evaluate_condition_with_device(&cond, Some("keyboard")));
            // No hyphen
        }

        #[test]
        fn test_device_pattern_contains() {
            let state = DeviceState::new();

            // Contains pattern (*numpad*)
            let cond = Condition::DeviceMatches(String::from("*numpad*"));
            assert!(state.evaluate_condition_with_device(&cond, Some("usb-numpad-123")));
            assert!(state.evaluate_condition_with_device(&cond, Some("numpad")));
            assert!(state.evaluate_condition_with_device(&cond, Some("my-numpad-device")));
            assert!(state.evaluate_condition_with_device(&cond, Some("anumpadb"))); // Contains substring
            assert!(!state.evaluate_condition_with_device(&cond, Some("keyboard")));
        }

        #[test]
        fn test_device_pattern_case_sensitive() {
            let state = DeviceState::new();

            // Pattern matching should be case-sensitive
            let cond = Condition::DeviceMatches(String::from("USB-Keyboard"));
            assert!(state.evaluate_condition_with_device(&cond, Some("USB-Keyboard")));
            assert!(!state.evaluate_condition_with_device(&cond, Some("usb-keyboard")));
            assert!(!state.evaluate_condition_with_device(&cond, Some("USB-KEYBOARD")));
            assert!(!state.evaluate_condition_with_device(&cond, Some("Usb-Keyboard")));
        }

        #[test]
        fn test_device_pattern_empty_and_whitespace() {
            let state = DeviceState::new();

            // Empty pattern should only match empty device_id
            let cond_empty = Condition::DeviceMatches(String::from(""));
            assert!(state.evaluate_condition_with_device(&cond_empty, Some("")));
            assert!(!state.evaluate_condition_with_device(&cond_empty, Some("any")));
            assert!(!state.evaluate_condition_with_device(&cond_empty, None));

            // Whitespace should be treated literally
            let cond_space = Condition::DeviceMatches(String::from(" "));
            assert!(state.evaluate_condition_with_device(&cond_space, Some(" ")));
            assert!(!state.evaluate_condition_with_device(&cond_space, Some("")));
            assert!(!state.evaluate_condition_with_device(&cond_space, Some("space")));
        }

        #[test]
        fn test_device_pattern_wildcard_only() {
            let state = DeviceState::new();

            // Single wildcard should match everything
            let cond = Condition::DeviceMatches(String::from("*"));
            assert!(state.evaluate_condition_with_device(&cond, Some("any-device")));
            assert!(state.evaluate_condition_with_device(&cond, Some("")));
            assert!(state.evaluate_condition_with_device(&cond, Some("usb-keyboard")));
            assert!(!state.evaluate_condition_with_device(&cond, None)); // Still requires Some()
        }

        #[test]
        fn test_device_pattern_special_characters() {
            let state = DeviceState::new();

            // Special characters should work (path-like patterns)
            let cond = Condition::DeviceMatches(String::from("usb-0000:00:14.0-1/input0"));
            assert!(state.evaluate_condition_with_device(&cond, Some("usb-0000:00:14.0-1/input0")));
            assert!(!state.evaluate_condition_with_device(&cond, Some("usb-0000:00:14.0-2/input0")));

            // Pattern with special chars and wildcard
            let cond_wildcard = Condition::DeviceMatches(String::from("usb-*:00:14.0-*/input0"));
            assert!(state
                .evaluate_condition_with_device(&cond_wildcard, Some("usb-0000:00:14.0-1/input0")));
            assert!(state
                .evaluate_condition_with_device(&cond_wildcard, Some("usb-1234:00:14.0-5/input0")));
            assert!(!state
                .evaluate_condition_with_device(&cond_wildcard, Some("usb-0000:00:14.0-1/input1")));
        }

        #[test]
        fn test_device_pattern_unicode() {
            let state = DeviceState::new();

            // Unicode characters should work
            let cond = Condition::DeviceMatches(String::from("キーボード-日本語"));
            assert!(state.evaluate_condition_with_device(&cond, Some("キーボード-日本語")));
            assert!(!state.evaluate_condition_with_device(&cond, Some("keyboard")));

            // Unicode with wildcard
            let cond_wildcard = Condition::DeviceMatches(String::from("*キーボード*"));
            assert!(
                state.evaluate_condition_with_device(&cond_wildcard, Some("usb-キーボード-123"))
            );
            assert!(state.evaluate_condition_with_device(&cond_wildcard, Some("キーボード")));
            assert!(!state.evaluate_condition_with_device(&cond_wildcard, Some("keyboard")));
        }

        #[test]
        fn test_device_pattern_multiple_wildcards() {
            let state = DeviceState::new();

            // Multiple wildcards in pattern
            let cond = Condition::DeviceMatches(String::from("*usb*keyboard*"));
            assert!(state.evaluate_condition_with_device(&cond, Some("my-usb-fancy-keyboard-123")));
            assert!(state.evaluate_condition_with_device(&cond, Some("usb-keyboard")));
            assert!(state.evaluate_condition_with_device(&cond, Some("usbkeyboard123"))); // No separators needed
            assert!(!state.evaluate_condition_with_device(&cond, Some("keyboard-usb"))); // Wrong order
            assert!(!state.evaluate_condition_with_device(&cond, Some("usb-only")));
        }

        #[test]
        fn test_device_pattern_edge_cases_none() {
            let state = DeviceState::new();

            // None device_id should never match any pattern
            let patterns = vec!["*", "usb-*", "*-keyboard", "*numpad*", "", "exact-match"];

            for pattern_str in patterns {
                let cond = Condition::DeviceMatches(String::from(pattern_str));
                assert!(
                    !state.evaluate_condition_with_device(&cond, None),
                    "Pattern '{}' should not match None",
                    pattern_str
                );
            }
        }

        #[test]
        fn test_device_pattern_realistic_device_ids() {
            let state = DeviceState::new();

            // Test with realistic device IDs from actual hardware

            // Linux evdev path
            let linux_pattern = Condition::DeviceMatches(String::from("/dev/input/event*"));
            assert!(state.evaluate_condition_with_device(&linux_pattern, Some("/dev/input/event0")));
            assert!(
                state.evaluate_condition_with_device(&linux_pattern, Some("/dev/input/event15"))
            );

            // Windows HID path
            let windows_pattern = Condition::DeviceMatches(String::from("*VID_046D&PID_C52B*"));
            assert!(state.evaluate_condition_with_device(
                &windows_pattern,
                Some("\\\\?\\HID#VID_046D&PID_C52B#7&1a2b3c4d&0&0000#{884b96c3-56ef-11d1-bc8c-00a0c91405dd}")
            ));

            // Serial number match
            let serial_pattern = Condition::DeviceMatches(String::from("*SN12345*"));
            assert!(state
                .evaluate_condition_with_device(&serial_pattern, Some("USB-Keyboard-SN12345-v2")));
        }
    }
}
