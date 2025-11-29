//! Core type definitions for input/output events.

use serde::{Deserialize, Serialize};

/// Physical key code (hardware scan code).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCode(pub u16);

/// Input event from keyboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputEvent {
    /// Key that was pressed/released.
    pub key: KeyCode,
    /// True if key down, false if key up.
    pub pressed: bool,
    /// Timestamp in microseconds.
    pub timestamp: u64,
}

/// Output action to send to OS.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputAction {
    /// Press a key.
    KeyDown(KeyCode),
    /// Release a key.
    KeyUp(KeyCode),
    /// Press and release a key.
    KeyTap(KeyCode),
    /// Block the original input (consume it).
    Block,
    /// Pass through the original input unchanged.
    PassThrough,
}

impl InputEvent {
    /// Create a new key down event.
    pub fn key_down(key: KeyCode, timestamp: u64) -> Self {
        Self {
            key,
            pressed: true,
            timestamp,
        }
    }

    /// Create a new key up event.
    pub fn key_up(key: KeyCode, timestamp: u64) -> Self {
        Self {
            key,
            pressed: false,
            timestamp,
        }
    }
}
