//! Core device state structure and initialization
//!
//! Manages the 255 modifiers + 255 locks state using bit vectors,
//! plus tap-hold processor state and pressed key tracking.

extern crate alloc;

use arrayvec::ArrayVec;
use bitvec::prelude::*;

use crate::config::{KeyCode, MAX_MODIFIER_ID, MODIFIER_COUNT};
use crate::runtime::tap_hold::{TapHoldProcessor, DEFAULT_MAX_PENDING};

/// Maximum valid modifier/lock ID (re-exported from config SSOT)
const MAX_VALID_ID: u8 = MAX_MODIFIER_ID as u8;

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
            modifiers: bitvec![u8, Lsb0; 0; MODIFIER_COUNT],
            locks: bitvec![u8, Lsb0; 0; MODIFIER_COUNT],
            tap_hold: TapHoldProcessor::new(),
            pressed_keys: ArrayVec::new(),
        }
    }

    /// Validates that a modifier/lock ID is in valid range (0-254)
    ///
    /// Returns true if valid, logs error and returns false if invalid (>254).
    #[inline]
    pub(super) fn validate_id(id: u8) -> bool {
        id <= MAX_VALID_ID
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
