//! Key code mapping between keyrx, evdev, and uinput formats.
//!
//! This module provides conversion functions between different key code representations:
//! - `KeyCode`: The keyrx internal representation (platform-agnostic)
//! - evdev key codes (u16): Raw Linux input event codes
//! - uinput `Keyboard` variants: Used for event injection via uinput
//!
//! The mapping tables are split by direction/target:
//! - [`to_uinput`]: `KeyCode` -> uinput `Keyboard`
//! - [`from_evdev`]: evdev code (u16) -> `KeyCode`
//! - [`to_evdev`]: `KeyCode` -> evdev code (u16)

mod from_evdev;
mod to_evdev;
mod to_uinput;

#[cfg(test)]
mod exhaustive_test;
#[cfg(test)]
mod tests;

pub use from_evdev::evdev_to_keycode;
pub use to_evdev::keycode_to_evdev;
pub use to_uinput::keycode_to_uinput_key;
