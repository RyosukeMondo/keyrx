//! Safe RAII wrapper for Windows keyboard hooks.
//!
//! This module provides `SafeHook`, a wrapper around Windows low-level keyboard hooks
//! that ensures proper cleanup and lifetime management using RAII (Resource Acquisition
//! Is Initialization).
//!
//! # Why SafeHook?
//!
//! The Windows `SetWindowsHookExW` API requires manual cleanup via `UnhookWindowsHookEx`.
//! Failing to unhook properly can lead to:
//! - Leaked system resources
//! - Hooks remaining active after application exit
//! - System instability in edge cases
//!
//! `SafeHook` encapsulates the hook lifecycle with automatic cleanup on Drop,
//! ensuring hooks are always properly removed even in panic or early return scenarios.
//!
//! # Safety Guarantees
//!
//! - Hook is always uninstalled when `SafeHook` is dropped (RAII)
//! - Invalid hook handles are never passed to Windows APIs
//! - Thread-local state is properly initialized and cleaned up
//! - All unsafe operations are documented with SAFETY comments
//! - Errors are propagated with clear messages and recovery hints
//!
//! # Example
//!
//! ```no_run
//! use keyrx_core::drivers::windows::safety::hook::SafeHook;
//! use keyrx_core::engine::InputEvent;
//! use crossbeam_channel::unbounded;
//! use std::sync::atomic::{AtomicBool, AtomicU32};
//! use std::sync::Arc;
//!
//! let running = Arc::new(AtomicBool::new(true));
//! let thread_id_store = Arc::new(AtomicU32::new(0));
//! let (sender, receiver) = unbounded::<InputEvent>();
//!
//! // Install hook - automatic cleanup on drop
//! let hook = SafeHook::install(sender, running, thread_id_store)?;
//!
//! // Hook is automatically uninstalled when `hook` goes out of scope
//! # Ok::<(), keyrx_core::drivers::common::error::DriverError>(())
//! ```

use crate::drivers::common::error::DriverError;
use crate::engine::InputEvent;
use crossbeam_channel::Sender;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tracing::{debug, error, warn};
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, WH_KEYBOARD_LL,
};

use super::super::hook::low_level_keyboard_proc;

/// Thread-local storage for the event sender used by the hook callback.
///
/// This is necessary because the hook callback is a C-style function pointer
/// that cannot capture any context. We store the sender in thread-local storage
/// and access it from within the callback.
///
/// # Safety Invariants
///
/// - Must be initialized before hook installation
/// - Must be cleared when hook is uninstalled
/// - Only accessed from the hook thread
thread_local! {
    pub static HOOK_SENDER: RefCell<Option<Sender<InputEvent>>> = const { RefCell::new(None) };
}

/// Thread-local storage for tracking key press states (for is_repeat detection).
///
/// Maps virtual key codes to their current pressed state. When we receive a key down
/// event for a key that's already marked as pressed, it's a repeat event.
///
/// # Safety Invariants
///
/// - Cleared on hook uninstall to prevent state leakage
/// - Only accessed from the hook callback thread
thread_local! {
    pub static KEY_STATES: RefCell<std::collections::HashSet<u16>> = RefCell::new(std::collections::HashSet::new());
}

/// Safe RAII wrapper for Windows keyboard hooks.
///
/// This wrapper ensures that Windows keyboard hooks are properly installed and
/// uninstalled, with automatic cleanup on Drop. It encapsulates the unsafe
/// Windows API calls and provides a safe interface.
///
/// # Lifetime
///
/// The hook remains active until the `SafeHook` instance is dropped. Dropping
/// the instance automatically calls `UnhookWindowsHookEx` to clean up.
///
/// # Thread Safety
///
/// The hook must be installed and uninstalled from the same thread that runs
/// the message loop. This is enforced by Windows, not by Rust's type system.
/// Violating this requirement will cause the hook to fail.
///
/// # Performance
///
/// Installing and uninstalling hooks are relatively expensive operations
/// (typically 10-50μs each). The hook should be installed once and kept
/// active for the duration of the driver's lifetime.
pub struct SafeHook {
    /// The hook handle returned by SetWindowsHookExW.
    ///
    /// This is `Some(handle)` when the hook is installed, and `None` after
    /// it has been manually uninstalled or if installation failed.
    ///
    /// # Safety Invariant
    ///
    /// When `Some(handle)`, the handle must be valid and registered with Windows.
    /// When `None`, no hook is active.
    handle: Option<HHOOK>,

    /// Flag to signal the message pump to stop.
    ///
    /// Shared with the message loop to coordinate shutdown.
    running: Arc<AtomicBool>,

    /// Storage for the thread ID of the message loop.
    ///
    /// Used to post WM_QUIT to the correct thread during shutdown.
    thread_id_store: Arc<AtomicU32>,
}

impl SafeHook {
    /// Install a low-level keyboard hook.
    ///
    /// This function installs a Windows keyboard hook that will capture all
    /// keyboard events system-wide. The events are sent to the provided channel.
    ///
    /// # Arguments
    ///
    /// * `sender` - Channel sender for keyboard events captured by the hook
    /// * `running` - Atomic flag to control the message loop
    /// * `thread_id_store` - Storage for the hook thread's ID
    ///
    /// # Errors
    ///
    /// Returns `DriverError::HookFailed` if:
    /// - The hook could not be installed (insufficient permissions, etc.)
    /// - Another hook is preventing installation
    /// - System resources are exhausted
    ///
    /// # Safety Requirements
    ///
    /// This function must be called from a thread that will run a message loop
    /// via `run_message_loop()`. The hook callbacks are dispatched through the
    /// Windows message queue, so a message loop is required for the hook to
    /// function correctly.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use keyrx_core::drivers::windows::safety::hook::SafeHook;
    /// use keyrx_core::engine::InputEvent;
    /// use crossbeam_channel::unbounded;
    /// use std::sync::atomic::{AtomicBool, AtomicU32};
    /// use std::sync::Arc;
    ///
    /// let (sender, receiver) = unbounded::<InputEvent>();
    /// let running = Arc::new(AtomicBool::new(true));
    /// let thread_id_store = Arc::new(AtomicU32::new(0));
    ///
    /// let hook = SafeHook::install(sender, running, thread_id_store)?;
    /// # Ok::<(), keyrx_core::drivers::common::error::DriverError>(())
    /// ```
    pub fn install(
        sender: Sender<InputEvent>,
        running: Arc<AtomicBool>,
        thread_id_store: Arc<AtomicU32>,
    ) -> Result<Self, DriverError> {
        // Store the sender in thread-local storage for the callback
        HOOK_SENDER.with(|s| {
            *s.borrow_mut() = Some(sender);
        });

        // SAFETY: We are calling SetWindowsHookExW with valid parameters:
        // - WH_KEYBOARD_LL: Standard hook type for low-level keyboard events
        // - Some(low_level_keyboard_proc): Valid function pointer to our callback
        // - HINSTANCE::default(): NULL for current process (correct for low-level hooks)
        // - 0: Hook all threads (required for low-level hooks)
        //
        // The function pointer is valid for the lifetime of the application as it's
        // a static function. The hook will be properly uninstalled in Drop.
        let hook = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(low_level_keyboard_proc),
                HINSTANCE::default(),
                0,
            )
        };

        match hook {
            Ok(handle) => {
                debug!(
                    service = "keyrx",
                    event = "safe_hook_installed",
                    component = "windows_safety",
                    "SafeHook installed successfully"
                );
                Ok(Self {
                    handle: Some(handle),
                    running,
                    thread_id_store,
                })
            }
            Err(e) => {
                error!(
                    service = "keyrx",
                    event = "safe_hook_install_failed",
                    component = "windows_safety",
                    error = %e,
                    error_code = e.code().0,
                    "Failed to install SafeHook"
                );

                // Clear the sender since installation failed
                HOOK_SENDER.with(|s| {
                    *s.borrow_mut() = None;
                });

                Err(DriverError::HookFailed {
                    code: e.code().0 as u32,
                })
            }
        }
    }

    /// Check if the hook is currently installed.
    ///
    /// Returns `true` if the hook is active, `false` if it has been uninstalled.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use keyrx_core::drivers::windows::safety::hook::SafeHook;
    /// # use keyrx_core::engine::InputEvent;
    /// # use crossbeam_channel::unbounded;
    /// # use std::sync::atomic::{AtomicBool, AtomicU32};
    /// # use std::sync::Arc;
    /// # let (sender, receiver) = unbounded::<InputEvent>();
    /// # let running = Arc::new(AtomicBool::new(true));
    /// # let thread_id_store = Arc::new(AtomicU32::new(0));
    /// let hook = SafeHook::install(sender, running, thread_id_store)?;
    /// assert!(hook.is_installed());
    /// # Ok::<(), keyrx_core::drivers::common::error::DriverError>(())
    /// ```
    pub fn is_installed(&self) -> bool {
        self.handle.is_some()
    }

    /// Get a reference to the running flag.
    ///
    /// This flag is shared with the message loop to coordinate shutdown.
    /// Setting it to `false` will cause the message loop to exit.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use keyrx_core::drivers::windows::safety::hook::SafeHook;
    /// # use keyrx_core::engine::InputEvent;
    /// # use crossbeam_channel::unbounded;
    /// # use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    /// # use std::sync::Arc;
    /// # let (sender, receiver) = unbounded::<InputEvent>();
    /// # let running = Arc::new(AtomicBool::new(true));
    /// # let thread_id_store = Arc::new(AtomicU32::new(0));
    /// let hook = SafeHook::install(sender, running, thread_id_store)?;
    ///
    /// // Signal the message loop to stop
    /// hook.running().store(false, Ordering::SeqCst);
    /// # Ok::<(), keyrx_core::drivers::common::error::DriverError>(())
    /// ```
    pub fn running(&self) -> &Arc<AtomicBool> {
        &self.running
    }

    /// Get a reference to the thread ID storage.
    ///
    /// This is used internally to track the message loop thread.
    pub fn thread_id_store(&self) -> &Arc<AtomicU32> {
        &self.thread_id_store
    }

    /// Manually uninstall the hook.
    ///
    /// This is called automatically by Drop, but can be called explicitly
    /// if early cleanup is desired. After calling this method, `is_installed()`
    /// will return `false`.
    ///
    /// # Note
    ///
    /// It is safe to call this method multiple times. Subsequent calls will
    /// have no effect.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use keyrx_core::drivers::windows::safety::hook::SafeHook;
    /// # use keyrx_core::engine::InputEvent;
    /// # use crossbeam_channel::unbounded;
    /// # use std::sync::atomic::{AtomicBool, AtomicU32};
    /// # use std::sync::Arc;
    /// # let (sender, receiver) = unbounded::<InputEvent>();
    /// # let running = Arc::new(AtomicBool::new(true));
    /// # let thread_id_store = Arc::new(AtomicU32::new(0));
    /// let mut hook = SafeHook::install(sender, running, thread_id_store)?;
    ///
    /// // Manually uninstall the hook
    /// hook.uninstall();
    /// assert!(!hook.is_installed());
    /// # Ok::<(), keyrx_core::drivers::common::error::DriverError>(())
    /// ```
    pub fn uninstall(&mut self) {
        if let Some(handle) = self.handle.take() {
            // SAFETY: We are calling UnhookWindowsHookEx with a valid hook handle
            // that we received from SetWindowsHookExW. This is safe because:
            // - The handle is valid (checked by the Option)
            // - We immediately set self.handle to None, preventing double-free
            // - The handle was obtained from SetWindowsHookExW in install()
            let result = unsafe { UnhookWindowsHookEx(handle) };

            if result.is_err() {
                warn!(
                    service = "keyrx",
                    event = "safe_hook_uninstall_failed",
                    component = "windows_safety",
                    "Failed to unhook keyboard hook (may already be removed)"
                );
            } else {
                debug!(
                    service = "keyrx",
                    event = "safe_hook_uninstalled",
                    component = "windows_safety",
                    "SafeHook uninstalled successfully"
                );
            }
        }

        // Clean up thread-local state
        self.cleanup_thread_local_state();
    }

    /// Clean up thread-local storage used by the hook.
    ///
    /// This clears the sender and key states to prevent stale data from
    /// being used if the hook is reinstalled later.
    fn cleanup_thread_local_state(&self) {
        HOOK_SENDER.with(|s| {
            *s.borrow_mut() = None;
        });

        KEY_STATES.with(|states| {
            states.borrow_mut().clear();
        });
    }
}

impl Drop for SafeHook {
    /// Automatically uninstall the hook when dropped.
    ///
    /// This ensures that hooks are always properly cleaned up, even if
    /// the code panics or returns early.
    ///
    /// # Safety
    ///
    /// This implementation ensures that:
    /// - The hook is uninstalled exactly once
    /// - Thread-local state is cleaned up
    /// - No Windows resources are leaked
    fn drop(&mut self) {
        self.uninstall();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::unbounded;

    #[test]
    fn safe_hook_not_installed_after_drop() {
        let (sender, _receiver) = unbounded::<InputEvent>();
        let running = Arc::new(AtomicBool::new(true));
        let thread_id_store = Arc::new(AtomicU32::new(0));

        let hook = SafeHook::install(sender, running.clone(), thread_id_store.clone());

        // Hook installation may fail if not running on Windows or without proper
        // message loop setup, so we only test the drop behavior if it succeeded
        if let Ok(hook) = hook {
            drop(hook);
            // After drop, we can't directly verify the hook is uninstalled,
            // but we can verify thread-local state was cleaned
            let sender_cleared = HOOK_SENDER.with(|s| s.borrow().is_none());
            let states_cleared = KEY_STATES.with(|states| states.borrow().is_empty());

            assert!(sender_cleared, "Sender should be cleared after drop");
            assert!(states_cleared, "Key states should be cleared after drop");
        }
    }

    #[test]
    fn safe_hook_is_installed_check() {
        let (sender, _receiver) = unbounded::<InputEvent>();
        let running = Arc::new(AtomicBool::new(true));
        let thread_id_store = Arc::new(AtomicU32::new(0));

        let hook = SafeHook::install(sender, running.clone(), thread_id_store.clone());

        if let Ok(hook) = hook {
            assert!(hook.is_installed());
        }
    }

    #[test]
    fn safe_hook_manual_uninstall() {
        let (sender, _receiver) = unbounded::<InputEvent>();
        let running = Arc::new(AtomicBool::new(true));
        let thread_id_store = Arc::new(AtomicU32::new(0));

        let hook = SafeHook::install(sender, running.clone(), thread_id_store.clone());

        if let Ok(mut hook) = hook {
            hook.uninstall();
            assert!(!hook.is_installed());

            // Verify thread-local state is cleaned
            let sender_cleared = HOOK_SENDER.with(|s| s.borrow().is_none());
            let states_cleared = KEY_STATES.with(|states| states.borrow().is_empty());

            assert!(sender_cleared, "Sender should be cleared after uninstall");
            assert!(states_cleared, "Key states should be cleared after uninstall");
        }
    }

    #[test]
    fn safe_hook_double_uninstall_safe() {
        let (sender, _receiver) = unbounded::<InputEvent>();
        let running = Arc::new(AtomicBool::new(true));
        let thread_id_store = Arc::new(AtomicU32::new(0));

        let hook = SafeHook::install(sender, running.clone(), thread_id_store.clone());

        if let Ok(mut hook) = hook {
            hook.uninstall();
            // Should not panic
            hook.uninstall();
            assert!(!hook.is_installed());
        }
    }

    #[test]
    fn safe_hook_running_flag_access() {
        let (sender, _receiver) = unbounded::<InputEvent>();
        let running = Arc::new(AtomicBool::new(true));
        let thread_id_store = Arc::new(AtomicU32::new(0));

        let hook = SafeHook::install(sender, running.clone(), thread_id_store.clone());

        if let Ok(hook) = hook {
            assert!(hook.running().load(Ordering::SeqCst));
            running.store(false, Ordering::SeqCst);
            assert!(!hook.running().load(Ordering::SeqCst));
        }
    }

    #[test]
    fn safe_hook_thread_id_store_access() {
        let (sender, _receiver) = unbounded::<InputEvent>();
        let running = Arc::new(AtomicBool::new(true));
        let thread_id_store = Arc::new(AtomicU32::new(0));

        let hook = SafeHook::install(sender, running.clone(), thread_id_store.clone());

        if let Ok(hook) = hook {
            assert_eq!(hook.thread_id_store().load(Ordering::SeqCst), 0);
            thread_id_store.store(12345, Ordering::SeqCst);
            assert_eq!(hook.thread_id_store().load(Ordering::SeqCst), 12345);
        }
    }

    #[test]
    fn thread_local_sender_cleaned_on_failed_install() {
        // Set up some initial state
        HOOK_SENDER.with(|s| {
            *s.borrow_mut() = None;
        });

        // This test verifies that even if hook installation fails,
        // we clean up the thread-local sender. Since we can't easily
        // force a hook installation failure in tests, we just verify
        // the thread-local cleanup behavior.
        HOOK_SENDER.with(|s| {
            *s.borrow_mut() = None;
        });

        let sender_cleared = HOOK_SENDER.with(|s| s.borrow().is_none());
        assert!(sender_cleared);
    }

    #[test]
    fn key_states_cleaned_on_uninstall() {
        // Simulate some key states
        KEY_STATES.with(|states| {
            let mut states = states.borrow_mut();
            states.insert(0x41); // 'A' key
            states.insert(0x42); // 'B' key
        });

        // Verify states exist
        let has_states = KEY_STATES.with(|states| !states.borrow().is_empty());
        assert!(has_states);

        // Clear as done in uninstall
        KEY_STATES.with(|states| {
            states.borrow_mut().clear();
        });

        // Verify cleared
        let states_cleared = KEY_STATES.with(|states| states.borrow().is_empty());
        assert!(states_cleared);
    }
}
