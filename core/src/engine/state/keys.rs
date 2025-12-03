//! Key state tracking for the unified engine state.
//!
//! This module provides the KeyState component which tracks which physical keys
//! are currently pressed and their press timestamps. It's designed to be part of
//! the unified EngineState and supports efficient lookups and iteration.

use std::collections::HashMap;

use crate::engine::KeyCode;

/// Tracks which physical keys are currently pressed and when they were first pressed.
///
/// This is a unified state component that:
/// - Ignores auto-repeat key_down events so state reflects real holds
/// - Maintains press timestamps for tap/hold detection
/// - Provides efficient lookups and iteration
#[derive(Debug, Clone)]
#[allow(dead_code)] // Will be used when EngineState is implemented in Phase 3
pub struct KeyState {
    /// Map of pressed keys to their initial press timestamp (in microseconds).
    pressed: HashMap<KeyCode, u64>,
}

impl Default for KeyState {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(dead_code)] // Will be used when EngineState is implemented in Phase 3
impl KeyState {
    /// Reserve space up front so press/release operations stay allocation-free.
    pub const DEFAULT_CAPACITY: usize = 256;

    /// Create a new KeyState with default capacity.
    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    /// Create a new KeyState with specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pressed: HashMap::with_capacity(capacity),
        }
    }

    /// Check if a key is currently pressed.
    #[inline]
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains_key(&key)
    }

    /// Record a key press. Returns true if the press changed state.
    ///
    /// Auto-repeat key_down events (`is_repeat == true`) are ignored when the
    /// key is already pressed so tap/hold logic sees a single active press.
    ///
    /// # Arguments
    /// * `key` - The key code being pressed
    /// * `timestamp_us` - The timestamp of the press event in microseconds
    /// * `is_repeat` - Whether this is an auto-repeat event
    ///
    /// # Returns
    /// `true` if the key state changed (key was not previously pressed),
    /// `false` if the key was already pressed or is an auto-repeat.
    pub fn press(&mut self, key: KeyCode, timestamp_us: u64, is_repeat: bool) -> bool {
        if self.pressed.contains_key(&key) {
            // Key already pressed - ignore duplicate presses
            return false;
        }

        if is_repeat {
            // Auto-repeat for a key that's not tracked - ignore it
            return false;
        }

        self.pressed.insert(key, timestamp_us);
        true
    }

    /// Record a key release. Returns the original press timestamp if the key
    /// was tracked, or None if it wasn't pressed.
    ///
    /// # Arguments
    /// * `key` - The key code being released
    ///
    /// # Returns
    /// The timestamp when the key was originally pressed, or None if the key
    /// was not being tracked as pressed.
    pub fn release(&mut self, key: KeyCode) -> Option<u64> {
        self.pressed.remove(&key)
    }

    /// Get the timestamp when a key was first pressed.
    ///
    /// # Arguments
    /// * `key` - The key code to query
    ///
    /// # Returns
    /// The press timestamp in microseconds, or None if the key is not pressed.
    #[inline]
    pub fn press_time(&self, key: KeyCode) -> Option<u64> {
        self.pressed.get(&key).copied()
    }

    /// Get an iterator over all currently pressed keys.
    ///
    /// # Returns
    /// An iterator yielding KeyCode values for all pressed keys.
    pub fn pressed_keys(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.pressed.keys().copied()
    }

    /// Get the number of currently pressed keys.
    #[inline]
    pub fn len(&self) -> usize {
        self.pressed.len()
    }

    /// Check if no keys are currently pressed.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pressed.is_empty()
    }

    /// Clear all pressed keys.
    ///
    /// This is useful for reset operations or when external events require
    /// clearing the state.
    pub fn clear(&mut self) {
        self.pressed.clear();
    }

    /// Get all pressed keys and their timestamps as a vector.
    ///
    /// This is useful for inspection and debugging. The order is not guaranteed.
    pub fn all_pressed(&self) -> Vec<(KeyCode, u64)> {
        self.pressed.iter().map(|(&k, &t)| (k, t)).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn new_key_state_is_empty() {
        let state = KeyState::new();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn press_records_key_and_timestamp() {
        let mut state = KeyState::new();
        assert!(state.press(KeyCode::A, 100, false));
        assert!(state.is_pressed(KeyCode::A));
        assert_eq!(state.press_time(KeyCode::A), Some(100));
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn release_removes_key_and_returns_timestamp() {
        let mut state = KeyState::new();
        state.press(KeyCode::A, 100, false);

        let press_time = state.release(KeyCode::A);
        assert_eq!(press_time, Some(100));
        assert!(!state.is_pressed(KeyCode::A));
        assert!(state.is_empty());
    }

    #[test]
    fn release_unknown_key_returns_none() {
        let mut state = KeyState::new();
        assert_eq!(state.release(KeyCode::B), None);
    }

    #[test]
    fn duplicate_press_does_not_change_state() {
        let mut state = KeyState::new();
        assert!(state.press(KeyCode::A, 100, false));
        assert!(!state.press(KeyCode::A, 200, false));
        assert_eq!(state.press_time(KeyCode::A), Some(100));
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn auto_repeat_is_ignored_when_key_pressed() {
        let mut state = KeyState::new();
        assert!(state.press(KeyCode::A, 100, false));
        assert!(!state.press(KeyCode::A, 200, true));
        assert_eq!(state.press_time(KeyCode::A), Some(100));
    }

    #[test]
    fn auto_repeat_ignored_when_key_not_pressed() {
        let mut state = KeyState::new();
        assert!(!state.press(KeyCode::A, 100, true));
        assert!(!state.is_pressed(KeyCode::A));
        assert!(state.is_empty());
    }

    #[test]
    fn pressed_keys_iterator() {
        let mut state = KeyState::new();
        state.press(KeyCode::A, 10, false);
        state.press(KeyCode::LeftShift, 20, false);
        state.press(KeyCode::B, 30, false);

        let keys: HashSet<_> = state.pressed_keys().collect();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&KeyCode::A));
        assert!(keys.contains(&KeyCode::LeftShift));
        assert!(keys.contains(&KeyCode::B));
    }

    #[test]
    fn multiple_keys_independent() {
        let mut state = KeyState::new();
        state.press(KeyCode::A, 100, false);
        state.press(KeyCode::B, 200, false);

        assert!(state.is_pressed(KeyCode::A));
        assert!(state.is_pressed(KeyCode::B));
        assert_eq!(state.len(), 2);

        state.release(KeyCode::A);
        assert!(!state.is_pressed(KeyCode::A));
        assert!(state.is_pressed(KeyCode::B));
        assert_eq!(state.len(), 1);
    }

    #[test]
    fn clear_removes_all_keys() {
        let mut state = KeyState::new();
        state.press(KeyCode::A, 100, false);
        state.press(KeyCode::B, 200, false);

        assert_eq!(state.len(), 2);
        state.clear();

        assert!(state.is_empty());
        assert!(!state.is_pressed(KeyCode::A));
        assert!(!state.is_pressed(KeyCode::B));
    }

    #[test]
    fn all_pressed_returns_all_keys_and_timestamps() {
        let mut state = KeyState::new();
        state.press(KeyCode::A, 100, false);
        state.press(KeyCode::B, 200, false);

        let all = state.all_pressed();
        assert_eq!(all.len(), 2);

        let keys_map: HashMap<_, _> = all.into_iter().collect();
        assert_eq!(keys_map.get(&KeyCode::A), Some(&100));
        assert_eq!(keys_map.get(&KeyCode::B), Some(&200));
    }

    #[test]
    fn default_creates_empty_state() {
        let state = KeyState::default();
        assert!(state.is_empty());
    }

    #[test]
    fn with_capacity_creates_empty_state() {
        let state = KeyState::with_capacity(64);
        assert!(state.is_empty());
    }
}
