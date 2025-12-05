//! Thread-local state management for Windows hooks.
//!
//! This module provides `ThreadLocalState`, a safe wrapper around thread-local storage
//! used by Windows keyboard hooks. Windows hook callbacks are invoked by the OS on the
//! same thread that installed the hook, and they cannot directly access captured state
//! due to the `extern "system"` function signature requirement.
//!
//! # Why Thread-Local Storage?
//!
//! Windows hook callbacks must be `extern "system"` functions that match a specific
//! signature. They cannot:
//! - Capture variables from their environment
//! - Be closures with state
//! - Access instance methods directly
//!
//! Thread-local storage solves this by providing a way to store per-thread state that
//! the callback can access. Since the callback is always invoked on the hook thread,
//! thread-local storage is safe and appropriate.
//!
//! # Safety Guarantees
//!
//! - No panics on access (returns `Option` instead)
//! - Automatic cleanup when thread exits
//! - Type-safe access patterns
//! - No data races (each thread has its own copy)
//!
//! # Example
//!
//! ```no_run
//! use keyrx_core::drivers::windows::safety::thread_local::{
//!     ThreadLocalState, HookSender, KeyStates
//! };
//! use crossbeam_channel::unbounded;
//!
//! // Initialize the thread-local state
//! let (tx, rx) = unbounded();
//! ThreadLocalState::init_sender(tx);
//!
//! // Access from the same thread (e.g., in hook callback)
//! if let Some(sender) = ThreadLocalState::with_sender(|sender| sender.clone()) {
//!     // Use sender...
//! }
//!
//! // Cleanup when done
//! ThreadLocalState::cleanup();
//! ```

use crate::engine::InputEvent;
use crossbeam_channel::Sender;
use std::cell::RefCell;
use std::collections::HashSet;

/// Type alias for the hook event sender.
pub type HookSender = Sender<InputEvent>;

/// Type alias for the key state tracking set.
pub type KeyStates = HashSet<u16>;

thread_local! {
    /// Thread-local storage for the event sender.
    ///
    /// This stores the channel sender used to communicate keyboard events from the
    /// hook callback to the main event processing loop.
    static HOOK_SENDER: RefCell<Option<HookSender>> = const { RefCell::new(None) };
}

thread_local! {
    /// Thread-local storage for key press state tracking.
    ///
    /// Maps virtual key codes (u16) to their current pressed state. Used to detect
    /// key repeat events by checking if a key-down event is received for a key that's
    /// already marked as pressed.
    static KEY_STATES: RefCell<KeyStates> = RefCell::new(HashSet::new());
}

/// Safe wrapper around thread-local storage for Windows hooks.
///
/// This type provides a safe API for accessing and modifying thread-local state
/// used by the Windows keyboard hook. All operations are designed to be panic-free
/// and return `Option` or `Result` types to handle errors gracefully.
///
/// # Thread Safety
///
/// This type is NOT `Send` or `Sync` because it wraps thread-local storage.
/// Each thread has its own independent state. This is intentional and correct
/// for the Windows hook architecture.
pub struct ThreadLocalState;

impl ThreadLocalState {
    /// Initialize the thread-local sender.
    ///
    /// This must be called once on the hook thread before the hook is installed.
    /// Calling it multiple times will replace the previous sender.
    ///
    /// # Arguments
    ///
    /// * `sender` - The channel sender for keyboard events
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_core::drivers::windows::safety::thread_local::ThreadLocalState;
    /// use crossbeam_channel::unbounded;
    ///
    /// let (tx, rx) = unbounded();
    /// ThreadLocalState::init_sender(tx);
    /// ```
    pub fn init_sender(sender: HookSender) {
        HOOK_SENDER.with(|s| {
            *s.borrow_mut() = Some(sender);
        });
    }

    /// Access the thread-local sender with a closure.
    ///
    /// This provides safe access to the sender without exposing interior mutability.
    /// Returns `None` if the sender has not been initialized on this thread.
    ///
    /// # Arguments
    ///
    /// * `f` - Closure that receives a reference to the sender
    ///
    /// # Returns
    ///
    /// The result of the closure if the sender exists, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_core::drivers::windows::safety::thread_local::ThreadLocalState;
    /// use keyrx_core::engine::InputEvent;
    ///
    /// let event = InputEvent::default();
    /// ThreadLocalState::with_sender(|sender| {
    ///     let _ = sender.try_send(event);
    /// });
    /// ```
    pub fn with_sender<F, R>(f: F) -> Option<R>
    where
        F: FnOnce(&HookSender) -> R,
    {
        HOOK_SENDER.with(|s| {
            let sender_ref = s.borrow();
            sender_ref.as_ref().map(f)
        })
    }

    /// Try to send an event through the thread-local sender.
    ///
    /// This is a convenience method that handles the common case of sending
    /// an event through the sender. Returns `true` if the event was sent
    /// successfully, `false` if the sender is not initialized or sending failed.
    ///
    /// # Arguments
    ///
    /// * `event` - The input event to send
    ///
    /// # Returns
    ///
    /// `true` if the event was sent, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_core::drivers::windows::safety::thread_local::ThreadLocalState;
    /// use keyrx_core::engine::InputEvent;
    ///
    /// let event = InputEvent::default();
    /// let sent = ThreadLocalState::try_send(event);
    /// if !sent {
    ///     // Handle send failure...
    /// }
    /// ```
    pub fn try_send(event: InputEvent) -> bool {
        Self::with_sender(|sender| sender.try_send(event).is_ok()).unwrap_or(false)
    }

    /// Check if a key is currently pressed.
    ///
    /// Returns `true` if the key (identified by its virtual key code) is
    /// marked as pressed in the thread-local state.
    ///
    /// # Arguments
    ///
    /// * `vk_code` - The virtual key code to check
    ///
    /// # Returns
    ///
    /// `true` if the key is pressed, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_core::drivers::windows::safety::thread_local::ThreadLocalState;
    ///
    /// let is_pressed = ThreadLocalState::is_key_pressed(0x41); // 'A' key
    /// ```
    pub fn is_key_pressed(vk_code: u16) -> bool {
        KEY_STATES.with(|states| states.borrow().contains(&vk_code))
    }

    /// Mark a key as pressed.
    ///
    /// Adds the key to the pressed state set. Returns `true` if the key was
    /// already pressed (indicating a repeat event), `false` if this is a new
    /// key press.
    ///
    /// # Arguments
    ///
    /// * `vk_code` - The virtual key code to mark as pressed
    ///
    /// # Returns
    ///
    /// `true` if the key was already pressed (repeat), `false` for new press.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_core::drivers::windows::safety::thread_local::ThreadLocalState;
    ///
    /// let is_repeat = ThreadLocalState::mark_key_pressed(0x41); // 'A' key
    /// if is_repeat {
    ///     // This is a key repeat event
    /// }
    /// ```
    pub fn mark_key_pressed(vk_code: u16) -> bool {
        KEY_STATES.with(|states| {
            let mut states = states.borrow_mut();
            !states.insert(vk_code) // insert returns false if already present
        })
    }

    /// Mark a key as released.
    ///
    /// Removes the key from the pressed state set.
    ///
    /// # Arguments
    ///
    /// * `vk_code` - The virtual key code to mark as released
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_core::drivers::windows::safety::thread_local::ThreadLocalState;
    ///
    /// ThreadLocalState::mark_key_released(0x41); // 'A' key
    /// ```
    pub fn mark_key_released(vk_code: u16) {
        KEY_STATES.with(|states| {
            states.borrow_mut().remove(&vk_code);
        });
    }

    /// Track a key press and return whether it's a repeat.
    ///
    /// This is a convenience method that combines press/release tracking.
    /// For key-down events, it marks the key as pressed and returns whether
    /// it was already pressed (repeat). For key-up events, it marks the key
    /// as released and always returns `false`.
    ///
    /// # Arguments
    ///
    /// * `vk_code` - The virtual key code
    /// * `pressed` - `true` for key-down, `false` for key-up
    ///
    /// # Returns
    ///
    /// `true` if this is a repeat key-down event, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_core::drivers::windows::safety::thread_local::ThreadLocalState;
    ///
    /// // Key down event
    /// let is_repeat = ThreadLocalState::track_key_state(0x41, true);
    ///
    /// // Key up event
    /// ThreadLocalState::track_key_state(0x41, false);
    /// ```
    pub fn track_key_state(vk_code: u16, pressed: bool) -> bool {
        if pressed {
            Self::mark_key_pressed(vk_code)
        } else {
            Self::mark_key_released(vk_code);
            false
        }
    }

    /// Get the number of currently pressed keys.
    ///
    /// This is primarily useful for testing and debugging.
    ///
    /// # Returns
    ///
    /// The number of keys currently marked as pressed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_core::drivers::windows::safety::thread_local::ThreadLocalState;
    ///
    /// let count = ThreadLocalState::pressed_key_count();
    /// println!("Currently pressed keys: {}", count);
    /// ```
    pub fn pressed_key_count() -> usize {
        KEY_STATES.with(|states| states.borrow().len())
    }

    /// Clear all thread-local state.
    ///
    /// This removes the sender and clears all key states. It should be called
    /// when the hook is uninstalled to ensure clean state for potential reinstall.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_core::drivers::windows::safety::thread_local::ThreadLocalState;
    ///
    /// // When shutting down the hook
    /// ThreadLocalState::cleanup();
    /// ```
    pub fn cleanup() {
        HOOK_SENDER.with(|s| {
            *s.borrow_mut() = None;
        });
        KEY_STATES.with(|states| {
            states.borrow_mut().clear();
        });
    }

    /// Check if the sender is initialized.
    ///
    /// Returns `true` if the thread-local sender has been initialized.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_core::drivers::windows::safety::thread_local::ThreadLocalState;
    ///
    /// if ThreadLocalState::is_sender_initialized() {
    ///     // Sender is ready to use
    /// }
    /// ```
    pub fn is_sender_initialized() -> bool {
        HOOK_SENDER.with(|s| s.borrow().is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::KeyCode;
    use crossbeam_channel::unbounded;

    // Helper to ensure clean state before each test
    fn setup() {
        ThreadLocalState::cleanup();
    }

    #[test]
    fn init_sender_stores_sender() {
        setup();
        let (tx, _rx) = unbounded();
        ThreadLocalState::init_sender(tx);

        assert!(ThreadLocalState::is_sender_initialized());
    }

    #[test]
    fn with_sender_none_when_not_initialized() {
        setup();
        let result = ThreadLocalState::with_sender(|_| 42);
        assert_eq!(result, None);
    }

    #[test]
    fn with_sender_some_when_initialized() {
        setup();
        let (tx, _rx) = unbounded();
        ThreadLocalState::init_sender(tx);

        let result = ThreadLocalState::with_sender(|_sender| 42);
        assert_eq!(result, Some(42));
    }

    #[test]
    fn try_send_success() {
        setup();
        let (tx, rx) = unbounded();
        ThreadLocalState::init_sender(tx);

        let event = InputEvent {
            key: KeyCode::A,
            pressed: true,
            timestamp_us: 0,
            device_id: None,
            is_repeat: false,
            is_synthetic: false,
            scan_code: 0,
        };

        assert!(ThreadLocalState::try_send(event));
        assert_eq!(rx.len(), 1);
    }

    #[test]
    fn try_send_fails_when_not_initialized() {
        setup();
        let event = InputEvent {
            key: KeyCode::A,
            pressed: true,
            timestamp_us: 0,
            device_id: None,
            is_repeat: false,
            is_synthetic: false,
            scan_code: 0,
        };

        assert!(!ThreadLocalState::try_send(event));
    }

    #[test]
    fn is_key_pressed_initially_false() {
        setup();
        assert!(!ThreadLocalState::is_key_pressed(0x41)); // 'A' key
    }

    #[test]
    fn mark_key_pressed_first_time() {
        setup();
        let is_repeat = ThreadLocalState::mark_key_pressed(0x41);
        assert!(!is_repeat, "First press should not be a repeat");
        assert!(ThreadLocalState::is_key_pressed(0x41));
    }

    #[test]
    fn mark_key_pressed_repeat() {
        setup();
        ThreadLocalState::mark_key_pressed(0x41);
        let is_repeat = ThreadLocalState::mark_key_pressed(0x41);
        assert!(is_repeat, "Second press should be a repeat");
    }

    #[test]
    fn mark_key_released_removes_state() {
        setup();
        ThreadLocalState::mark_key_pressed(0x41);
        assert!(ThreadLocalState::is_key_pressed(0x41));

        ThreadLocalState::mark_key_released(0x41);
        assert!(!ThreadLocalState::is_key_pressed(0x41));
    }

    #[test]
    fn track_key_state_press() {
        setup();
        let is_repeat = ThreadLocalState::track_key_state(0x41, true);
        assert!(!is_repeat, "First press should not be repeat");
        assert!(ThreadLocalState::is_key_pressed(0x41));

        let is_repeat = ThreadLocalState::track_key_state(0x41, true);
        assert!(is_repeat, "Second press should be repeat");
    }

    #[test]
    fn track_key_state_release() {
        setup();
        ThreadLocalState::track_key_state(0x41, true);
        assert!(ThreadLocalState::is_key_pressed(0x41));

        let is_repeat = ThreadLocalState::track_key_state(0x41, false);
        assert!(!is_repeat, "Release should never be repeat");
        assert!(!ThreadLocalState::is_key_pressed(0x41));
    }

    #[test]
    fn pressed_key_count_empty() {
        setup();
        assert_eq!(ThreadLocalState::pressed_key_count(), 0);
    }

    #[test]
    fn pressed_key_count_multiple_keys() {
        setup();
        ThreadLocalState::mark_key_pressed(0x41); // A
        ThreadLocalState::mark_key_pressed(0x42); // B
        ThreadLocalState::mark_key_pressed(0x43); // C

        assert_eq!(ThreadLocalState::pressed_key_count(), 3);

        ThreadLocalState::mark_key_released(0x42); // Release B
        assert_eq!(ThreadLocalState::pressed_key_count(), 2);
    }

    #[test]
    fn cleanup_clears_sender() {
        setup();
        let (tx, _rx) = unbounded();
        ThreadLocalState::init_sender(tx);
        assert!(ThreadLocalState::is_sender_initialized());

        ThreadLocalState::cleanup();
        assert!(!ThreadLocalState::is_sender_initialized());
    }

    #[test]
    fn cleanup_clears_key_states() {
        setup();
        ThreadLocalState::mark_key_pressed(0x41);
        ThreadLocalState::mark_key_pressed(0x42);
        assert_eq!(ThreadLocalState::pressed_key_count(), 2);

        ThreadLocalState::cleanup();
        assert_eq!(ThreadLocalState::pressed_key_count(), 0);
    }

    #[test]
    fn cleanup_is_idempotent() {
        setup();
        ThreadLocalState::cleanup();
        ThreadLocalState::cleanup(); // Should not panic
        assert_eq!(ThreadLocalState::pressed_key_count(), 0);
        assert!(!ThreadLocalState::is_sender_initialized());
    }

    #[test]
    fn multiple_keys_independent() {
        setup();
        // Press A and B
        ThreadLocalState::mark_key_pressed(0x41);
        ThreadLocalState::mark_key_pressed(0x42);

        assert!(ThreadLocalState::is_key_pressed(0x41));
        assert!(ThreadLocalState::is_key_pressed(0x42));

        // Release A
        ThreadLocalState::mark_key_released(0x41);

        assert!(!ThreadLocalState::is_key_pressed(0x41));
        assert!(ThreadLocalState::is_key_pressed(0x42));
    }

    #[test]
    fn reinit_sender_replaces_old() {
        setup();
        let (tx1, _rx1) = unbounded();
        ThreadLocalState::init_sender(tx1);

        let (tx2, rx2) = unbounded();
        ThreadLocalState::init_sender(tx2);

        let event = InputEvent {
            key: KeyCode::A,
            pressed: true,
            timestamp_us: 0,
            device_id: None,
            is_repeat: false,
            is_synthetic: false,
            scan_code: 0,
        };

        ThreadLocalState::try_send(event);

        // Should go to the second receiver
        assert_eq!(rx2.len(), 1);
    }

    #[test]
    fn try_send_multiple_events() {
        setup();
        let (tx, rx) = unbounded();
        ThreadLocalState::init_sender(tx);

        for i in 0..10 {
            let event = InputEvent {
                key: KeyCode::A,
                pressed: i % 2 == 0,
                timestamp_us: i as u64,
                device_id: None,
                is_repeat: false,
                is_synthetic: false,
                scan_code: 0,
            };
            assert!(ThreadLocalState::try_send(event));
        }

        assert_eq!(rx.len(), 10);
    }
}
