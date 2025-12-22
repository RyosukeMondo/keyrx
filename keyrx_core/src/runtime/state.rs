//! Device state management with bit vectors
//!
//! This module provides `DeviceState` for tracking modifier and lock state
//! using efficient 255-bit vectors.

/// Device state tracking modifier and lock state
///
/// Uses 255-bit vectors for efficient state management:
/// - Modifiers: Temporary state (set on press, clear on release)
/// - Locks: Toggle state (toggle on press, ignore release)
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
    /// Placeholder - will be implemented in task 2
    _placeholder: u8,
}

impl DeviceState {
    /// Creates a new device state with all bits cleared
    pub fn new() -> Self {
        Self { _placeholder: 0 }
    }
}

impl Default for DeviceState {
    fn default() -> Self {
        Self::new()
    }
}
