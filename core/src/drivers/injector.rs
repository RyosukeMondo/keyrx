//! Key injection trait for OS-level keyboard output.
//!
//! This module defines the [`KeyInjector`] trait, which abstracts over platform-specific
//! keyboard output injection mechanisms. Implementations handle the low-level details of
//! sending synthetic key presses to the operating system.
//!
//! # Thread Safety
//!
//! The `KeyInjector` trait requires `Send` because implementations may be used
//! across thread boundaries. The remapping engine may run on a dedicated thread
//! separate from where the injector was created.
//!
//! # Design
//!
//! This trait enables dependency injection for key output, allowing:
//! - Unit testing without hardware access
//! - Mocking key injection for simulation
//! - Swapping implementations for different platforms
//!
//! # Example Implementation Sketch
//!
//! ```ignore
//! use keyrx_core::drivers::KeyInjector;
//! use keyrx_core::engine::KeyCode;
//! use anyhow::Result;
//!
//! struct MyInjector {
//!     // Platform-specific handles
//! }
//!
//! impl KeyInjector for MyInjector {
//!     fn inject(&mut self, key: KeyCode, pressed: bool) -> Result<()> {
//!         // Send synthetic key event to OS
//!         Ok(())
//!     }
//!
//!     fn sync(&mut self) -> Result<()> {
//!         // Flush pending events (if applicable)
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # Platform Implementations
//!
//! - **Windows**: Uses `SendInput` API for key injection
//! - **Linux**: Uses uinput virtual device for synthetic events
//! - **Testing**: Use [`MockKeyInjector`] for unit tests

use crate::engine::KeyCode;
use anyhow::Result;
use std::sync::{Arc, Mutex};

/// Trait for key injection (synthetic keyboard output).
///
/// This trait abstracts platform-specific keyboard output injection, allowing
/// the remapping engine to inject synthetic key events across Windows, Linux,
/// and in tests.
///
/// # Thread Safety
///
/// Implementations must be `Send` because the injector may be used across thread
/// boundaries in async contexts.
///
/// # Error Handling
///
/// All methods return `Result<()>` using the `anyhow` crate. Implementations should:
/// - Return meaningful error messages that help diagnose the issue
/// - Not panic on recoverable errors
/// - Clean up resources appropriately when errors occur
///
/// # Implementations
///
/// - `UinputWriter`: Linux uinput-based injection
/// - `SendInputInjector`: Windows SendInput-based injection
/// - [`MockKeyInjector`]: Test mock for simulation and verification
pub trait KeyInjector: Send {
    /// Inject a key event (press or release).
    ///
    /// Sends a synthetic keyboard event to the operating system.
    ///
    /// # Arguments
    ///
    /// * `key` - The key code to inject
    /// * `pressed` - `true` for key press (down), `false` for key release (up)
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Event was successfully injected
    /// - `Err(_)` - Failed to inject the event
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The virtual input device is not available
    /// - The OS rejected the synthetic event
    /// - Permission to inject input was denied
    /// - The injection API call failed
    fn inject(&mut self, key: KeyCode, pressed: bool) -> Result<()>;

    /// Synchronize pending events.
    ///
    /// Flushes any buffered events to ensure they are processed by the OS.
    /// On Linux, this sends an EV_SYN event. On Windows, this is typically a no-op
    /// since SendInput processes events immediately.
    ///
    /// # Returns
    ///
    /// - `Ok(())` - Sync completed successfully
    /// - `Err(_)` - Failed to sync events
    ///
    /// # Note
    ///
    /// For most implementations, `inject()` already includes synchronization.
    /// This method is provided for cases where manual sync control is needed,
    /// such as batching multiple events before flushing.
    fn sync(&mut self) -> Result<()>;
}

/// A recorded key injection event for testing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InjectedKey {
    /// The key that was injected.
    pub key: KeyCode,
    /// Whether the key was pressed (true) or released (false).
    pub pressed: bool,
}

impl InjectedKey {
    /// Create a new InjectedKey record.
    pub fn new(key: KeyCode, pressed: bool) -> Self {
        Self { key, pressed }
    }

    /// Create a key press record.
    pub fn press(key: KeyCode) -> Self {
        Self { key, pressed: true }
    }

    /// Create a key release record.
    pub fn release(key: KeyCode) -> Self {
        Self {
            key,
            pressed: false,
        }
    }
}

/// Mock implementation of KeyInjector for testing.
///
/// Records all injected keys for later verification. This allows unit tests
/// to verify that the correct keys were injected without requiring hardware
/// access or root privileges.
///
/// # Thread Safety
///
/// The mock uses internal synchronization via `Arc<Mutex<>>` to allow
/// safe access to injected keys from multiple threads.
///
/// # Example
///
/// ```
/// use keyrx_core::drivers::{KeyInjector, MockKeyInjector, InjectedKey};
/// use keyrx_core::engine::KeyCode;
///
/// let mut injector = MockKeyInjector::new();
///
/// // Inject some keys
/// injector.inject(KeyCode::A, true).unwrap();
/// injector.inject(KeyCode::A, false).unwrap();
///
/// // Verify injections
/// let injected = injector.injected_keys();
/// assert_eq!(injected.len(), 2);
/// assert_eq!(injected[0], InjectedKey::press(KeyCode::A));
/// assert_eq!(injected[1], InjectedKey::release(KeyCode::A));
/// ```
#[derive(Debug, Clone)]
pub struct MockKeyInjector {
    /// Recorded key injections.
    injections: Arc<Mutex<Vec<InjectedKey>>>,
    /// Count of sync() calls.
    sync_count: Arc<Mutex<usize>>,
    /// Whether to fail on next injection (for error testing).
    fail_next: Arc<Mutex<bool>>,
}

impl Default for MockKeyInjector {
    fn default() -> Self {
        Self::new()
    }
}

impl MockKeyInjector {
    /// Create a new MockKeyInjector.
    pub fn new() -> Self {
        Self {
            injections: Arc::new(Mutex::new(Vec::new())),
            sync_count: Arc::new(Mutex::new(0)),
            fail_next: Arc::new(Mutex::new(false)),
        }
    }

    /// Get a copy of all injected keys.
    #[allow(clippy::expect_used)] // Mock lock should never be poisoned
    pub fn injected_keys(&self) -> Vec<InjectedKey> {
        self.injections
            .lock()
            .expect("mock injections lock poisoned")
            .clone()
    }

    /// Get the number of times sync() was called.
    #[allow(clippy::expect_used)] // Mock lock should never be poisoned
    pub fn sync_count(&self) -> usize {
        *self
            .sync_count
            .lock()
            .expect("mock sync_count lock poisoned")
    }

    /// Clear all recorded injections.
    #[allow(clippy::expect_used)] // Mock lock should never be poisoned
    pub fn clear(&mut self) {
        self.injections
            .lock()
            .expect("mock injections lock poisoned")
            .clear();
        *self
            .sync_count
            .lock()
            .expect("mock sync_count lock poisoned") = 0;
    }

    /// Set the mock to fail on the next injection.
    ///
    /// This is useful for testing error handling paths.
    #[allow(clippy::expect_used)] // Mock lock should never be poisoned
    pub fn fail_next_injection(&mut self) {
        *self.fail_next.lock().expect("mock fail_next lock poisoned") = true;
    }

    /// Check if a specific key press was injected.
    #[allow(clippy::expect_used)] // Mock lock should never be poisoned
    pub fn was_pressed(&self, key: KeyCode) -> bool {
        self.injections
            .lock()
            .expect("mock injections lock poisoned")
            .iter()
            .any(|i| i.key == key && i.pressed)
    }

    /// Check if a specific key release was injected.
    #[allow(clippy::expect_used)] // Mock lock should never be poisoned
    pub fn was_released(&self, key: KeyCode) -> bool {
        self.injections
            .lock()
            .expect("mock injections lock poisoned")
            .iter()
            .any(|i| i.key == key && !i.pressed)
    }

    /// Check if a key tap (press followed by release) was injected.
    #[allow(clippy::expect_used)] // Mock lock should never be poisoned
    pub fn was_tapped(&self, key: KeyCode) -> bool {
        let injections = self
            .injections
            .lock()
            .expect("mock injections lock poisoned");
        let mut saw_press = false;
        for injection in injections.iter() {
            if injection.key == key {
                if injection.pressed {
                    saw_press = true;
                } else if saw_press {
                    return true;
                }
            }
        }
        false
    }
}

impl KeyInjector for MockKeyInjector {
    #[allow(clippy::expect_used)] // Mock lock should never be poisoned
    fn inject(&mut self, key: KeyCode, pressed: bool) -> Result<()> {
        // Check if we should fail
        {
            let mut fail = self.fail_next.lock().expect("mock fail_next lock poisoned");
            if *fail {
                *fail = false;
                return Err(anyhow::anyhow!("Mock injection failure (intentional)"));
            }
        }

        // Record the injection
        self.injections
            .lock()
            .expect("mock injections lock poisoned")
            .push(InjectedKey::new(key, pressed));
        Ok(())
    }

    #[allow(clippy::expect_used)] // Mock lock should never be poisoned
    fn sync(&mut self) -> Result<()> {
        *self
            .sync_count
            .lock()
            .expect("mock sync_count lock poisoned") += 1;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)] // Tests use unwrap for clarity
mod tests {
    use super::*;

    #[test]
    fn mock_injector_records_injections() {
        let mut injector = MockKeyInjector::new();

        injector.inject(KeyCode::A, true).unwrap();
        injector.inject(KeyCode::A, false).unwrap();
        injector.inject(KeyCode::B, true).unwrap();

        let injected = injector.injected_keys();
        assert_eq!(injected.len(), 3);
        assert_eq!(injected[0], InjectedKey::press(KeyCode::A));
        assert_eq!(injected[1], InjectedKey::release(KeyCode::A));
        assert_eq!(injected[2], InjectedKey::press(KeyCode::B));
    }

    #[test]
    fn mock_injector_tracks_sync_calls() {
        let mut injector = MockKeyInjector::new();

        assert_eq!(injector.sync_count(), 0);

        injector.sync().unwrap();
        assert_eq!(injector.sync_count(), 1);

        injector.sync().unwrap();
        injector.sync().unwrap();
        assert_eq!(injector.sync_count(), 3);
    }

    #[test]
    fn mock_injector_clear_resets_state() {
        let mut injector = MockKeyInjector::new();

        injector.inject(KeyCode::A, true).unwrap();
        injector.sync().unwrap();

        assert_eq!(injector.injected_keys().len(), 1);
        assert_eq!(injector.sync_count(), 1);

        injector.clear();

        assert!(injector.injected_keys().is_empty());
        assert_eq!(injector.sync_count(), 0);
    }

    #[test]
    fn mock_injector_fail_next() {
        let mut injector = MockKeyInjector::new();

        injector.fail_next_injection();
        let result = injector.inject(KeyCode::A, true);
        assert!(result.is_err());

        // Should succeed now
        let result = injector.inject(KeyCode::A, true);
        assert!(result.is_ok());
    }

    #[test]
    fn mock_injector_was_pressed() {
        let mut injector = MockKeyInjector::new();

        assert!(!injector.was_pressed(KeyCode::A));

        injector.inject(KeyCode::A, true).unwrap();
        assert!(injector.was_pressed(KeyCode::A));
        assert!(!injector.was_pressed(KeyCode::B));
    }

    #[test]
    fn mock_injector_was_released() {
        let mut injector = MockKeyInjector::new();

        assert!(!injector.was_released(KeyCode::A));

        injector.inject(KeyCode::A, false).unwrap();
        assert!(injector.was_released(KeyCode::A));
        assert!(!injector.was_released(KeyCode::B));
    }

    #[test]
    fn mock_injector_was_tapped() {
        let mut injector = MockKeyInjector::new();

        // Just press - not a tap
        injector.inject(KeyCode::A, true).unwrap();
        assert!(!injector.was_tapped(KeyCode::A));

        // Release completes the tap
        injector.inject(KeyCode::A, false).unwrap();
        assert!(injector.was_tapped(KeyCode::A));
    }

    #[test]
    fn mock_injector_was_tapped_requires_order() {
        let mut injector = MockKeyInjector::new();

        // Release without prior press - not a tap
        injector.inject(KeyCode::A, false).unwrap();
        injector.inject(KeyCode::A, true).unwrap();
        assert!(!injector.was_tapped(KeyCode::A));
    }

    #[test]
    fn mock_injector_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<MockKeyInjector>();
    }

    #[test]
    fn mock_injector_default() {
        let injector = MockKeyInjector::default();
        assert!(injector.injected_keys().is_empty());
        assert_eq!(injector.sync_count(), 0);
    }

    #[test]
    fn injected_key_constructors() {
        let press = InjectedKey::press(KeyCode::A);
        assert_eq!(press.key, KeyCode::A);
        assert!(press.pressed);

        let release = InjectedKey::release(KeyCode::B);
        assert_eq!(release.key, KeyCode::B);
        assert!(!release.pressed);

        let manual = InjectedKey::new(KeyCode::C, true);
        assert_eq!(manual.key, KeyCode::C);
        assert!(manual.pressed);
    }
}
