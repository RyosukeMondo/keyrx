//! CGKeyCode ↔ KeyCode bidirectional mapping.
//!
//! This module provides conversions between macOS CGKeyCode values and
//! keyrx KeyCode enum values, as well as conversions for rdev and enigo types.

use keyrx_core::config::keys::KeyCode;

/// Converts a CGKeyCode to a keyrx KeyCode.
///
/// # Arguments
///
/// * `cgcode` - macOS virtual key code
///
/// # Returns
///
/// The corresponding [`KeyCode`], or `None` if unmapped.
pub fn cgkeycode_to_keyrx(_cgcode: u16) -> Option<KeyCode> {
    // Placeholder - will be implemented in task 5
    None
}

/// Converts a keyrx KeyCode to a CGKeyCode.
///
/// # Arguments
///
/// * `keycode` - keyrx KeyCode
///
/// # Returns
///
/// The corresponding CGKeyCode, or `None` if unmapped.
pub fn keyrx_to_cgkeycode(_keycode: KeyCode) -> Option<u16> {
    // Placeholder - will be implemented in task 5
    None
}

/// Converts an rdev::Key to a keyrx KeyCode.
///
/// # Arguments
///
/// * `key` - rdev key event
///
/// # Returns
///
/// The corresponding [`KeyCode`], or `None` if unmapped.
pub fn rdev_key_to_keyrx(_key: rdev::Key) -> Option<KeyCode> {
    // Placeholder - will be implemented in task 5
    None
}

/// Converts a keyrx KeyCode to an enigo::Key.
///
/// # Arguments
///
/// * `keycode` - keyrx KeyCode
///
/// # Returns
///
/// The corresponding enigo::Key, or `None` if unmapped.
pub fn keyrx_to_enigo_key(_keycode: KeyCode) -> Option<enigo::Key> {
    // Placeholder - will be implemented in task 5
    None
}
