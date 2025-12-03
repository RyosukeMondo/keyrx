//! Engine state provider traits for dependency injection.
//!
//! These traits abstract over the concrete state types used by `AdvancedEngine`,
//! enabling unit tests to inject mock implementations without global state.
//!
//! # Traits
//!
//! - [`KeyStateProvider`]: Tracks which keys are currently pressed
//! - [`ModifierProvider`]: Manages active modifiers (Shift, Ctrl, etc.)
//! - [`LayerProvider`]: Controls the layer stack for mapping lookup
//!
//! # Production vs Test
//!
//! In production, the engine uses:
//! - [`KeyStateTracker`](crate::engine::KeyStateTracker)
//! - [`ModifierState`](crate::engine::ModifierState)
//! - [`LayerStack`](crate::engine::LayerStack)
//!
//! In tests, mock implementations can be injected to verify behavior
//! without complex state setup.

use crate::engine::{KeyCode, LayerAction, LayerId, Modifier};

/// Tracks which physical keys are currently pressed.
///
/// This trait abstracts over key state tracking, allowing tests to inject
/// mock implementations that simulate specific key states.
///
/// # Methods
///
/// - `is_pressed`: Check if a key is currently held down
/// - `press`: Record a key press with timestamp
/// - `release`: Record a key release, returning the original press timestamp
/// - `press_time`: Get the timestamp when a key was pressed
/// - `pressed_keys`: Iterate over all currently pressed keys
///
/// # Thread Safety
///
/// Implementations do not need to be thread-safe. The engine processes
/// events sequentially on a single thread.
pub trait KeyStateProvider {
    /// Returns true if the key is currently pressed.
    fn is_pressed(&self, key: KeyCode) -> bool;

    /// Record a key press.
    ///
    /// # Arguments
    /// * `key` - The key being pressed
    /// * `timestamp_us` - Microsecond timestamp of the press
    /// * `is_repeat` - Whether this is an auto-repeat event
    ///
    /// # Returns
    /// `true` if this is a new press (state changed), `false` if already pressed.
    fn press(&mut self, key: KeyCode, timestamp_us: u64, is_repeat: bool) -> bool;

    /// Record a key release.
    ///
    /// # Returns
    /// The original press timestamp if the key was pressed, `None` otherwise.
    fn release(&mut self, key: KeyCode) -> Option<u64>;

    /// Returns the timestamp when the key was pressed.
    fn press_time(&self, key: KeyCode) -> Option<u64>;

    /// Returns an iterator over currently pressed keys.
    fn pressed_keys(&self) -> Box<dyn Iterator<Item = KeyCode> + '_>;
}

/// Manages modifier state (standard and virtual modifiers).
///
/// This trait abstracts over modifier state management, allowing tests
/// to inject mock implementations that track modifier changes.
///
/// # Modifiers
///
/// Modifiers come in two varieties:
/// - **Standard**: OS-level modifiers (Shift, Control, Alt, Meta)
/// - **Virtual**: User-defined modifiers for script logic (IDs 0-255)
///
/// # One-Shot Modifiers
///
/// One-shot modifiers apply to the next key press only, then deactivate.
/// This is useful for implementing sticky modifier behavior.
pub trait ModifierProvider {
    /// Returns true if the modifier is currently active.
    fn is_active(&self, modifier: Modifier) -> bool;

    /// Activate a modifier.
    fn activate(&mut self, modifier: Modifier);

    /// Deactivate a modifier.
    fn deactivate(&mut self, modifier: Modifier);

    /// Arm a one-shot modifier (active until next key press).
    fn arm_one_shot(&mut self, modifier: Modifier);

    /// Clear all modifiers.
    fn clear(&mut self);
}

/// Controls the layer stack for key mapping lookup.
///
/// This trait abstracts over layer management, allowing tests to inject
/// mock implementations that simulate specific layer configurations.
///
/// # Layer Stack
///
/// Layers are arranged in a stack with the base layer (ID 0) at the bottom.
/// Key lookups check layers from top to bottom, with transparent layers
/// falling through to lower layers for unmapped keys.
///
/// # Operations
///
/// - `push`: Add a layer to the top of the stack
/// - `pop`: Remove the topmost non-base layer
/// - `toggle`: Add or remove a layer from the stack
/// - `lookup`: Find the action for a key across active layers
pub trait LayerProvider {
    /// Get the ID of the currently active (topmost) layer.
    fn active_layer(&self) -> LayerId;

    /// Get all active layer IDs in priority order (last = highest).
    fn active_layer_ids(&self) -> Vec<LayerId>;

    /// Push a layer to the top of the stack.
    ///
    /// # Returns
    /// `true` if the layer was pushed, `false` if it doesn't exist or is base.
    fn push(&mut self, layer_id: LayerId) -> bool;

    /// Pop the topmost non-base layer.
    ///
    /// # Returns
    /// The ID of the popped layer, or `None` if only base layer remains.
    fn pop(&mut self) -> Option<LayerId>;

    /// Toggle a layer on/off.
    ///
    /// # Returns
    /// `true` if the toggle succeeded, `false` if layer doesn't exist.
    fn toggle(&mut self, layer_id: LayerId) -> bool;

    /// Check if a layer is currently active.
    fn is_active(&self, layer_id: LayerId) -> bool;

    /// Look up the action for a key across active layers.
    ///
    /// Searches from the topmost layer down. Transparent layers fall through
    /// to lower layers for unmapped keys.
    fn lookup(&self, key: KeyCode) -> Option<&LayerAction>;
}
