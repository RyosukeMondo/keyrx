//! Keyboard event types and processing logic
//!
//! This module provides:
//! - `KeyEvent`: Type-safe keyboard event representation with timestamps and device ID
//! - `KeyEventType`: Enum for press/release event types
//! - `process_event`: Core event processing function

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::config::KeyCode;
use crate::runtime::tap_hold::{TapHoldConfig, TapHoldOutput};
use crate::runtime::{DeviceState, KeyLookup};
use serde::{Deserialize, Serialize};

/// Type of keyboard event (press or release)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyEventType {
    /// Key press event
    Press,
    /// Key release event
    Release,
}

/// Keyboard event representing a key press or release with timestamp and optional device ID
///
/// The timestamp is in microseconds and is used for timing-based decisions
/// such as tap-hold functionality. A timestamp of 0 indicates no timestamp
/// is available (legacy compatibility).
///
/// The device_id is optional and allows discrimination between multiple input
/// devices (e.g., laptop keyboard vs USB numpad). When None, the event is
/// treated as coming from the default device (backward compatible).
///
/// # Example
///
/// ```rust,ignore
/// use keyrx_core::runtime::KeyEvent;
/// use keyrx_core::config::KeyCode;
///
/// // Create a press event with timestamp
/// let event = KeyEvent::press(KeyCode::A).with_timestamp(1000);
/// assert_eq!(event.keycode(), KeyCode::A);
/// assert!(event.is_press());
/// assert_eq!(event.timestamp_us(), 1000);
///
/// // Create event with device ID for multi-device support
/// let event = KeyEvent::press(KeyCode::A)
///     .with_device_id("usb-NumericKeypad-123".to_string());
/// assert_eq!(event.device_id(), Some("usb-NumericKeypad-123"));
///
/// // Legacy style (shorthand constructors)
/// let press = KeyEvent::Press(KeyCode::A);
/// let release = KeyEvent::Release(KeyCode::A);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyEvent {
    /// The type of event (press or release)
    event_type: KeyEventType,
    /// The keycode for this event
    keycode: KeyCode,
    /// Timestamp in microseconds (0 = no timestamp)
    timestamp_us: u64,
    /// Optional device identifier for multi-device support
    /// When None, event is treated as coming from default device
    device_id: Option<String>,
}

impl KeyEvent {
    /// Creates a new key press event
    ///
    /// The timestamp defaults to 0 (no timestamp) and device_id defaults to None.
    /// Use `with_timestamp()` and `with_device_id()` to set specific values.
    #[must_use]
    pub fn press(keycode: KeyCode) -> Self {
        Self {
            event_type: KeyEventType::Press,
            keycode,
            timestamp_us: 0,
            device_id: None,
        }
    }

    /// Creates a new key release event
    ///
    /// The timestamp defaults to 0 (no timestamp) and device_id defaults to None.
    /// Use `with_timestamp()` and `with_device_id()` to set specific values.
    #[must_use]
    pub fn release(keycode: KeyCode) -> Self {
        Self {
            event_type: KeyEventType::Release,
            keycode,
            timestamp_us: 0,
            device_id: None,
        }
    }

    /// Legacy constructor for press events (enum-style syntax)
    #[must_use]
    #[allow(non_snake_case)]
    pub fn Press(keycode: KeyCode) -> Self {
        Self::press(keycode)
    }

    /// Legacy constructor for release events (enum-style syntax)
    #[must_use]
    #[allow(non_snake_case)]
    pub fn Release(keycode: KeyCode) -> Self {
        Self::release(keycode)
    }

    /// Creates a new event with the specified timestamp
    #[must_use]
    pub fn with_timestamp(mut self, timestamp_us: u64) -> Self {
        self.timestamp_us = timestamp_us;
        self
    }

    /// Creates a new event with the specified device ID
    ///
    /// The device ID allows discrimination between multiple input devices
    /// (e.g., "usb-NumericKeypad-123" vs "platform-keyboard-0").
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let event = KeyEvent::press(KeyCode::A)
    ///     .with_device_id("usb-NumericKeypad-123".to_string());
    /// assert_eq!(event.device_id(), Some("usb-NumericKeypad-123"));
    /// ```
    #[must_use]
    pub fn with_device_id(mut self, device_id: String) -> Self {
        self.device_id = Some(device_id);
        self
    }

    /// Returns the keycode for this event
    #[must_use]
    pub const fn keycode(&self) -> KeyCode {
        self.keycode
    }

    /// Returns the event type (Press or Release)
    #[must_use]
    pub const fn event_type(&self) -> KeyEventType {
        self.event_type
    }

    /// Returns the timestamp in microseconds (0 = no timestamp)
    #[must_use]
    pub const fn timestamp_us(&self) -> u64 {
        self.timestamp_us
    }

    /// Returns the device ID if set, or None for default device
    ///
    /// The device ID allows scripts and handlers to apply different
    /// remapping rules based on which physical device generated the event.
    #[must_use]
    pub fn device_id(&self) -> Option<&str> {
        self.device_id.as_deref()
    }

    /// Returns true if this is a press event
    #[must_use]
    pub const fn is_press(&self) -> bool {
        matches!(self.event_type, KeyEventType::Press)
    }

    /// Returns true if this is a release event
    #[must_use]
    pub const fn is_release(&self) -> bool {
        matches!(self.event_type, KeyEventType::Release)
    }

    /// Creates a new event with the same keycode, timestamp, and device_id but opposite type
    #[must_use]
    pub fn opposite(&self) -> Self {
        Self {
            event_type: match self.event_type {
                KeyEventType::Press => KeyEventType::Release,
                KeyEventType::Release => KeyEventType::Press,
            },
            keycode: self.keycode,
            timestamp_us: self.timestamp_us,
            device_id: self.device_id.clone(),
        }
    }

    /// Creates a new event with a different keycode but same type, timestamp, and device_id
    #[must_use]
    pub fn with_keycode(mut self, keycode: KeyCode) -> Self {
        self.keycode = keycode;
        self
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
    event: KeyEvent,
    lookup: &KeyLookup,
    state: &mut DeviceState,
) -> Vec<KeyEvent> {
    use crate::config::BaseKeyMapping;

    // Cache event properties before event is potentially moved
    let is_press = event.is_press();
    let input_keycode = event.keycode();

    // For RELEASE events: Check if we have a tracked press mapping
    // This ensures releases match their presses even if mapping changed
    if !is_press {
        let tracked_outputs = state.get_release_key(input_keycode);

        // Check if we have a real tracking (not just [input_keycode])
        if tracked_outputs.len() == 1 && tracked_outputs[0] == input_keycode {
            // No tracked mapping - proceed with normal lookup
            state.clear_press(input_keycode);
        } else {
            // We have tracked mappings! Release all keys in REVERSE order
            // (If press was [LShift, Z], release should be [Z, LShift])
            state.clear_press(input_keycode);
            let mut result = alloc::vec::Vec::new();
            for &keycode in tracked_outputs.iter().rev() {
                result.push(event.clone().with_keycode(keycode));
            }
            return result;
        }
    }

    // Look up the mapping for this key
    let mapping = lookup.find_mapping(event.keycode(), state);

    // Check for permissive hold: if this is a press event and there are pending
    // tap-hold keys, we need to trigger permissive hold BEFORE processing this key.
    // This ensures the modifier is active when this key is processed.
    let mut prefix_events = Vec::new();
    let mut permissive_hold_triggered = false;
    if event.is_press() {
        // Check if any tap-hold keys are pending and this isn't a tap-hold key itself
        let is_tap_hold_key = matches!(mapping, Some(BaseKeyMapping::TapHold { .. }));
        if !is_tap_hold_key && state.tap_hold_processor_ref().has_pending_keys() {
            // Trigger permissive hold for all pending keys
            let outputs = state
                .tap_hold_processor()
                .process_other_key_press(event.keycode());

            if !outputs.is_empty() {
                permissive_hold_triggered = true;
            }
            prefix_events = convert_tap_hold_outputs(outputs, state, event.timestamp_us());
        }
    }

    // If permissive hold changed the state (e.g. activated a modifier), we need to
    // look up the mapping again to ensure we use the layer that was just activated.
    // This fixes the bug where fast typing (permissive hold) would use the base layer
    // mapping instead of the conditional layer mapping.
    let mapping = if permissive_hold_triggered {
        lookup.find_mapping(event.keycode(), state)
    } else {
        mapping
    };

    // If no mapping found, pass through the original event
    let Some(mapping) = mapping else {
        prefix_events.push(event);
        return prefix_events;
    };

    // Process the mapping based on its type
    let mut result = match mapping {
        BaseKeyMapping::Simple { to, .. } => {
            // Simple remapping: replace keycode while preserving Press/Release and timestamp
            alloc::vec![event.with_keycode(*to)]
        }
        BaseKeyMapping::Modifier { modifier_id, .. } => {
            // Modifier mapping: update state, no output events
            if event.is_press() {
                state.set_modifier(*modifier_id);
            } else {
                state.clear_modifier(*modifier_id);
            }
            Vec::new()
        }
        BaseKeyMapping::Lock { lock_id, .. } => {
            // Lock mapping: toggle on press, ignore release, no output events
            if event.is_press() {
                state.toggle_lock(*lock_id);
            }
            Vec::new()
        }
        BaseKeyMapping::TapHold {
            from,
            tap,
            hold_modifier,
            threshold_ms,
        } => {
            // Register the tap-hold configuration if not already registered
            let processor = state.tap_hold_processor();
            if !processor.is_tap_hold_key(*from) {
                let config = TapHoldConfig::from_ms(*tap, *hold_modifier, *threshold_ms);
                processor.register_tap_hold(*from, config);
            }

            // Process the event through the tap-hold processor
            let timestamp = event.timestamp_us();
            let outputs = if event.is_press() {
                processor.process_press(*from, timestamp)
            } else {
                processor.process_release(*from, timestamp)
            };

            // Convert TapHoldOutput to KeyEvent and apply state changes
            convert_tap_hold_outputs(outputs, state, timestamp)
        }
        BaseKeyMapping::ModifiedOutput {
            to,
            shift,
            ctrl,
            alt,
            win,
            ..
        } => {
            // ModifiedOutput: emit modifier presses, then key, then releases in reverse
            use crate::config::KeyCode;

            let mut events = Vec::new();
            let ts = event.timestamp_us();

            if event.is_press() {
                // Press modifiers first, then the key
                if *shift {
                    events.push(KeyEvent::press(KeyCode::LShift).with_timestamp(ts));
                }
                if *ctrl {
                    events.push(KeyEvent::press(KeyCode::LCtrl).with_timestamp(ts));
                }
                if *alt {
                    events.push(KeyEvent::press(KeyCode::LAlt).with_timestamp(ts));
                }
                if *win {
                    events.push(KeyEvent::press(KeyCode::LMeta).with_timestamp(ts));
                }
                events.push(KeyEvent::press(*to).with_timestamp(ts));
            } else {
                // Release key first, then modifiers in reverse order
                events.push(KeyEvent::release(*to).with_timestamp(ts));
                if *win {
                    events.push(KeyEvent::release(KeyCode::LMeta).with_timestamp(ts));
                }
                if *alt {
                    events.push(KeyEvent::release(KeyCode::LAlt).with_timestamp(ts));
                }
                if *ctrl {
                    events.push(KeyEvent::release(KeyCode::LCtrl).with_timestamp(ts));
                }
                if *shift {
                    events.push(KeyEvent::release(KeyCode::LShift).with_timestamp(ts));
                }
            }

            events
        }
    };

    // For PRESS events: Record the mapping for press/release consistency
    // This must happen AFTER processing, so we know the actual output
    if is_press && !result.is_empty() {
        // Collect ALL press event keycodes from the result
        let output_keys: alloc::vec::Vec<KeyCode> = result
            .iter()
            .filter(|e| e.is_press())
            .map(|e| e.keycode())
            .collect();

        // Only track if outputs differ from just [input_keycode]
        if !(output_keys.is_empty() || (output_keys.len() == 1 && output_keys[0] == input_keycode))
        {
            state.record_press(input_keycode, &output_keys);
        }
    }

    // Prepend prefix events (from permissive hold) to the result
    if !prefix_events.is_empty() {
        prefix_events.append(&mut result);
        prefix_events
    } else {
        result
    }
}

/// Checks for tap-hold timeouts and returns resulting events.
///
/// This function should be called periodically (e.g., every 10-100ms) by the
/// daemon to detect keys that have been held past their threshold time.
/// When a timeout occurs, the key transitions from Pending to Hold state
/// and the associated modifier is activated.
///
/// # Arguments
///
/// * `current_time_us` - Current time in microseconds (same timescale as KeyEvent timestamps)
/// * `state` - Mutable reference to the device state containing the tap-hold processor
///
/// # Returns
///
/// A vector of `KeyEvent`s to inject (typically empty, as timeout only activates modifiers).
/// The state is also updated to activate the hold modifiers.
///
/// # Example
///
/// ```ignore
/// // In daemon event loop, after processing events:
/// let current_time = get_current_time_us();
/// let timeout_events = check_tap_hold_timeouts(current_time, &mut device_state);
/// for event in timeout_events {
///     output.inject_event(event)?;
/// }
/// ```
pub fn check_tap_hold_timeouts(current_time_us: u64, state: &mut DeviceState) -> Vec<KeyEvent> {
    let outputs = state.tap_hold_processor().check_timeouts(current_time_us);
    convert_tap_hold_outputs(outputs, state, current_time_us)
}

/// Converts TapHoldOutput events to KeyEvents and applies state changes
///
/// This helper handles the conversion of tap-hold processor outputs:
/// - KeyEvent outputs are converted to KeyEvent structs
/// - Modifier activation/deactivation updates DeviceState
fn convert_tap_hold_outputs(
    outputs: arrayvec::ArrayVec<TapHoldOutput, 4>,
    state: &mut DeviceState,
    _timestamp: u64,
) -> Vec<KeyEvent> {
    let mut events = Vec::new();

    for output in outputs {
        match output {
            TapHoldOutput::KeyEvent {
                key,
                is_press,
                timestamp_us,
            } => {
                let event = if is_press {
                    KeyEvent::press(key).with_timestamp(timestamp_us)
                } else {
                    KeyEvent::release(key).with_timestamp(timestamp_us)
                };
                events.push(event);
            }
            TapHoldOutput::ActivateModifier { modifier_id } => {
                state.set_modifier(modifier_id);
            }
            TapHoldOutput::DeactivateModifier { modifier_id } => {
                state.clear_modifier(modifier_id);
            }
        }
    }

    events
}
