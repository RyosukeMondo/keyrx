//! Core device state structure and initialization
//!
//! Manages the 255 modifiers + 255 locks state using bit vectors,
//! plus tap-hold processor state and pressed key tracking.

extern crate alloc;

use alloc::sync::Arc;

use arrayvec::ArrayVec;

use crate::config::{ImeState, KeyCode};
use crate::runtime::state::shared::{SharedModifierState, SharedState};
use crate::runtime::tap_hold::{TapHoldProcessor, DEFAULT_MAX_PENDING};

/// Maximum number of simultaneously pressed keys to track
/// This should cover even the most extreme cases (10-finger roll)
const MAX_PRESSED_KEYS: usize = 32;

/// Maximum number of output keys per input key
/// Covers Sequence (up to 8 keys) and ModifiedOutput (Shift+Ctrl+Alt+Win+Key = 5)
const MAX_OUTPUT_KEYS_PER_INPUT: usize = 8;

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
    /// Modifier/lock bits. Shared by reference across every `DeviceState`
    /// created via [`DeviceState::new_sharing`] (see the cross-device state
    /// sharing docs on [`SharedModifierState`]); private and unshared for a
    /// plain [`DeviceState::new`].
    shared: SharedState,
    /// Tap-hold processor for dual-function keys (per-device: a hold on one
    /// device must not be affected by keystrokes on another).
    tap_hold: TapHoldProcessor<DEFAULT_MAX_PENDING>,
    /// Pressed key tracking: (input_key, [output_keys]) pairs (per-device: a
    /// release on one device must never release another device's tracked
    /// outputs).
    /// This ensures release events match their corresponding press events
    /// Supports multiple output keys per input (e.g., Shift+Z generates 2 keys)
    pressed_keys:
        ArrayVec<(KeyCode, ArrayVec<KeyCode, MAX_OUTPUT_KEYS_PER_INPUT>), MAX_PRESSED_KEYS>,
    /// Current IME state, updated by daemon before event processing
    ime_state: ImeState,
}

impl DeviceState {
    /// Creates a new device state with all bits cleared, and its own
    /// private (unshared) modifier/lock state.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let state = DeviceState::new();
    /// assert!(!state.is_modifier_active(0));
    /// assert!(!state.is_lock_active(0));
    /// ```
    pub fn new() -> Self {
        Self::new_sharing(&SharedModifierState::new_handle())
    }

    /// Creates a device state whose modifier/lock bits are SHARED with
    /// every other `DeviceState` created from the same handle - see the
    /// cross-device state sharing docs on [`SharedModifierState`]. Tap-hold
    /// and pressed-key tracking are always private to the new state.
    pub fn new_sharing(shared: &SharedState) -> Self {
        Self {
            shared: Arc::clone(shared),
            tap_hold: TapHoldProcessor::new(),
            pressed_keys: ArrayVec::new(),
            ime_state: ImeState::default(),
        }
    }

    /// The modifier/lock handle this state reads and writes - clone it into
    /// [`DeviceState::new_sharing`] to make another device share it.
    pub fn shared_handle(&self) -> SharedState {
        Arc::clone(&self.shared)
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
        self.shared.lock().set_modifier(id)
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
        self.shared.lock().clear_modifier(id)
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
        self.shared.lock().toggle_lock(id)
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
        self.shared.lock().is_modifier_active(id)
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
        self.shared.lock().is_lock_active(id)
    }

    /// Returns the current IME state
    pub fn ime_state(&self) -> &ImeState {
        &self.ime_state
    }

    /// Updates the IME state (called by daemon before processing events)
    pub fn set_ime_state(&mut self, state: ImeState) {
        self.ime_state = state;
    }

    /// Checks if IME is currently active
    pub fn is_ime_active(&self) -> bool {
        self.ime_state.active
    }

    /// Returns the current input language tag
    pub fn input_language(&self) -> &str {
        &self.ime_state.language
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
