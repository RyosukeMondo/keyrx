//! Keyboard event types and processing logic
//!
//! This module provides:
//! - `KeyEvent`: Type-safe keyboard event representation
//! - `process_event`: Core event processing function

extern crate alloc;
use alloc::vec::Vec;

use crate::config::KeyCode;
use crate::runtime::{DeviceState, KeyLookup};

/// Keyboard event representing a key press or release
///
/// # Example
///
/// ```rust,ignore
/// use keyrx_core::runtime::KeyEvent;
/// use keyrx_core::config::KeyCode;
///
/// let event = KeyEvent::Press(KeyCode::A);
/// assert_eq!(event.keycode(), KeyCode::A);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyEvent {
    /// Key press event
    Press(KeyCode),
    /// Key release event
    Release(KeyCode),
}

impl KeyEvent {
    /// Returns the keycode for this event
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let event = KeyEvent::Press(KeyCode::A);
    /// assert_eq!(event.keycode(), KeyCode::A);
    /// ```
    pub fn keycode(&self) -> KeyCode {
        match self {
            KeyEvent::Press(k) => *k,
            KeyEvent::Release(k) => *k,
        }
    }
}

/// Process a keyboard event through the remapping engine
///
/// Returns a vector of output events based on the mapping configuration.
/// May return:
/// - Empty vector (for modifier/lock mappings)
/// - Single event (for simple remapping or passthrough)
/// - Multiple events (for modified output sequences)
///
/// # Arguments
///
/// * `event` - Input keyboard event
/// * `lookup` - Key lookup table for mapping resolution
/// * `state` - Mutable device state for modifier/lock tracking
///
/// # Example
///
/// ```rust,ignore
/// use keyrx_core::runtime::{process_event, KeyEvent, KeyLookup, DeviceState};
///
/// let lookup = KeyLookup::from_device_config(&config);
/// let mut state = DeviceState::new();
/// let input = KeyEvent::Press(KeyCode::A);
/// let outputs = process_event(input, &lookup, &mut state);
/// ```
pub fn process_event(
    _event: KeyEvent,
    _lookup: &KeyLookup,
    _state: &mut DeviceState,
) -> Vec<KeyEvent> {
    // Placeholder - will be implemented in tasks 9-11
    Vec::new()
}
