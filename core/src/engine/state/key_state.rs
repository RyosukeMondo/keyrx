use std::collections::HashMap;

use crate::engine::KeyCode;
use crate::traits::KeyStateProvider;

/// Tracks which physical keys are currently pressed and when they were first
/// pressed. Ignores auto-repeat key_down events so state reflects real holds.
pub struct KeyStateTracker {
    pressed: HashMap<KeyCode, u64>,
}

impl KeyStateProvider for KeyStateTracker {
    fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains_key(&key)
    }

    fn press(&mut self, key: KeyCode, timestamp_us: u64, is_repeat: bool) -> bool {
        self.key_down(key, timestamp_us, is_repeat)
    }

    fn release(&mut self, key: KeyCode) -> Option<u64> {
        self.key_up(key)
    }

    fn press_time(&self, key: KeyCode) -> Option<u64> {
        self.pressed.get(&key).copied()
    }

    fn pressed_keys(&self) -> Box<dyn Iterator<Item = KeyCode> + '_> {
        Box::new(self.pressed.keys().copied())
    }
}

impl Default for KeyStateTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyStateTracker {
    /// Reserve space up front so key_down/key_up stay allocation-free.
    pub const DEFAULT_CAPACITY: usize = 256;

    pub fn new() -> Self {
        Self::with_capacity(Self::DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pressed: HashMap::with_capacity(capacity),
        }
    }

    /// Record a key press. Returns true if the press changed state.
    ///
    /// Auto-repeat key_down events (`is_repeat == true`) are ignored when the
    /// key is already pressed so tap/hold logic sees a single active press.
    pub fn key_down(&mut self, key: KeyCode, timestamp_us: u64, is_repeat: bool) -> bool {
        if self.pressed.contains_key(&key) {
            if !is_repeat {
                // Duplicate non-repeat press; keep earliest timestamp.
            }
            return false;
        }

        self.pressed.insert(key, timestamp_us);
        true
    }

    /// Record a key release. Returns the original press timestamp if the key
    /// was tracked, or None if it wasn't pressed.
    pub fn key_up(&mut self, key: KeyCode) -> Option<u64> {
        self.pressed.remove(&key)
    }

    /// Returns true if the key is currently pressed.
    #[inline]
    pub fn is_pressed(&self, key: KeyCode) -> bool {
        self.pressed.contains_key(&key)
    }

    /// Returns the timestamp the key was first pressed.
    #[inline]
    pub fn press_time(&self, key: KeyCode) -> Option<u64> {
        self.pressed.get(&key).copied()
    }

    /// Returns an iterator over currently pressed keys.
    pub fn pressed_keys(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.pressed.keys().copied()
    }

    /// Number of currently pressed keys.
    #[inline]
    pub fn len(&self) -> usize {
        self.pressed.len()
    }

    /// True when no keys are pressed.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.pressed.is_empty()
    }
}
