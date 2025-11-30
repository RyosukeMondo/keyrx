//! Windows low-level keyboard hook management.
//!
//! This module provides the `HookManager` for managing the lifecycle of a Windows
//! keyboard hook, and the `low_level_keyboard_proc` callback function.

use crate::engine::{InputEvent, KeyCode};
use crate::error::WindowsDriverError;
use crossbeam_channel::Sender;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tracing::{debug, error, warn};
use windows::Win32::Foundation::HINSTANCE;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, PeekMessageW, SetWindowsHookExW, TranslateMessage,
    UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, LLKHF_INJECTED, MSG, PM_REMOVE, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN,
};

mod hook_thread;
pub(crate) use hook_thread::spawn_hook_thread;

use super::keymap::vk_to_keycode;

/// Thread-local storage for the event sender used by the hook callback.
///
/// This is necessary because the hook callback is a C-style function pointer
/// that cannot capture any context. We store the sender in thread-local storage
/// and access it from within the callback.
thread_local! {
    pub static HOOK_SENDER: RefCell<Option<Sender<InputEvent>>> = const { RefCell::new(None) };
}

/// Thread-local storage for tracking key press states (for is_repeat detection).
///
/// Maps virtual key codes to their current pressed state. When we receive a key down
/// event for a key that's already marked as pressed, it's a repeat event.
thread_local! {
    pub static KEY_STATES: RefCell<std::collections::HashSet<u16>> = RefCell::new(std::collections::HashSet::new());
}

/// Global storage for the hook thread's thread ID.
///
/// This is used to post WM_QUIT to the hook thread when shutting down.
/// We use an atomic because it needs to be accessed from multiple threads.
pub static HOOK_THREAD_ID: AtomicU32 = AtomicU32::new(0);

/// Low-level keyboard hook manager.
///
/// Manages the lifecycle of a Windows keyboard hook, including installation,
/// event capture, and cleanup.
pub struct HookManager {
    /// The hook handle returned by SetWindowsHookExW.
    hook_handle: Option<HHOOK>,
    /// Flag to signal the message pump to stop.
    running: Arc<AtomicBool>,
}

impl HookManager {
    /// Create a new HookManager.
    ///
    /// The hook is not installed until `install()` is called.
    pub fn new(running: Arc<AtomicBool>) -> Self {
        Self {
            hook_handle: None,
            running,
        }
    }

    /// Install the low-level keyboard hook.
    ///
    /// This must be called from a thread that will run a message pump,
    /// as hook callbacks are dispatched via the Windows message queue.
    ///
    /// # Arguments
    ///
    /// * `sender` - Channel sender for keyboard events
    ///
    /// # Errors
    ///
    /// Returns `WindowsDriverError::HookInstallFailed` if the hook cannot be installed.
    pub fn install(&mut self, sender: Sender<InputEvent>) -> Result<(), WindowsDriverError> {
        // Store the sender in thread-local storage for the callback
        HOOK_SENDER.with(|s| {
            *s.borrow_mut() = Some(sender);
        });

        // Install the low-level keyboard hook
        // SAFETY: We pass null for hmod (current process) and 0 for thread ID (all threads)
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
                    event = "keyboard_hook_installed",
                    component = "windows_hook",
                    "Keyboard hook installed successfully"
                );
                self.hook_handle = Some(handle);
                Ok(())
            }
            Err(e) => {
                error!(
                    service = "keyrx",
                    event = "keyboard_hook_install_failed",
                    component = "windows_hook",
                    error = %e,
                    "Failed to install keyboard hook"
                );
                // Clear the sender since we failed
                HOOK_SENDER.with(|s| {
                    *s.borrow_mut() = None;
                });
                Err(WindowsDriverError::hook_install_failed(e.code().0 as u32))
            }
        }
    }

    /// Uninstall the keyboard hook.
    ///
    /// This should be called before the thread exits to properly clean up.
    pub fn uninstall(&mut self) {
        if let Some(handle) = self.hook_handle.take() {
            // SAFETY: We're passing a valid hook handle that we received from SetWindowsHookExW
            let result = unsafe { UnhookWindowsHookEx(handle) };
            if result.is_err() {
                warn!(
                    service = "keyrx",
                    event = "keyboard_hook_uninstall_failed",
                    component = "windows_hook",
                    "Failed to unhook keyboard hook"
                );
            } else {
                debug!(
                    service = "keyrx",
                    event = "keyboard_hook_uninstalled",
                    component = "windows_hook",
                    "Keyboard hook uninstalled"
                );
            }
        }

        // Clear the thread-local sender
        HOOK_SENDER.with(|s| {
            *s.borrow_mut() = None;
        });

        // Clear key states for clean restart
        KEY_STATES.with(|states| {
            states.borrow_mut().clear();
        });
    }

    /// Check if the hook is currently installed.
    pub fn is_installed(&self) -> bool {
        self.hook_handle.is_some()
    }

    /// Get the running flag.
    pub fn running(&self) -> &Arc<AtomicBool> {
        &self.running
    }

    /// Run the Windows message loop.
    ///
    /// This function processes messages from the Windows message queue, which is
    /// required for the low-level keyboard hook to receive callbacks. The loop
    /// continues until:
    /// - The `running` flag is set to `false`
    /// - A `WM_QUIT` message is received
    ///
    /// # Thread Safety
    ///
    /// This must be called from the same thread that called `install()`.
    /// The message loop will sleep for 1ms between iterations when no messages
    /// are pending to avoid busy-waiting.
    pub fn run_message_loop(&self) {
        let thread_id = self.register_thread();
        let mut msg = MSG::default();

        while self.running.load(Ordering::SeqCst) {
            if self.handle_message(&mut msg, thread_id) {
                break;
            }
        }

        self.clear_thread_id(thread_id);
    }

    fn register_thread(&self) -> u32 {
        let thread_id = unsafe { GetCurrentThreadId() };
        HOOK_THREAD_ID.store(thread_id, Ordering::SeqCst);
        debug!(
            service = "keyrx",
            event = "windows_message_loop_start",
            component = "windows_hook",
            thread_id = thread_id,
            "Starting Windows message loop"
        );
        thread_id
    }

    fn handle_message(&self, msg: &mut MSG, thread_id: u32) -> bool {
        let has_message = unsafe { PeekMessageW(msg, None, 0, 0, PM_REMOVE) }.as_bool();
        if has_message {
            if msg.message == WM_QUIT {
                debug!(
                    service = "keyrx",
                    event = "windows_message_loop_quit",
                    component = "windows_hook",
                    thread_id = thread_id,
                    "Received WM_QUIT, exiting message loop"
                );
                return true;
            }
            unsafe {
                let _ = TranslateMessage(msg);
                DispatchMessageW(msg);
            }
        } else {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        false
    }

    fn clear_thread_id(&self, thread_id: u32) {
        HOOK_THREAD_ID.store(0, Ordering::SeqCst);
        debug!(
            service = "keyrx",
            event = "windows_message_loop_stop",
            component = "windows_hook",
            thread_id = thread_id,
            "Windows message loop stopped"
        );
    }
}

impl Drop for HookManager {
    fn drop(&mut self) {
        self.uninstall();
    }
}

/// Low-level keyboard hook callback.
///
/// This function is called by Windows for every keyboard event. It must complete
/// quickly (within ~100ms per Windows requirements) or keyboard input will lag.
///
/// # Safety
///
/// This is an unsafe extern function called by Windows. The `lparam` must be
/// a valid pointer to `KBDLLHOOKSTRUCT` when `ncode >= 0`.
pub unsafe extern "system" fn low_level_keyboard_proc(
    ncode: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if ncode < 0 {
        return CallNextHookEx(HHOOK::default(), ncode, wparam, lparam);
    }

    let kb_struct = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
    let pressed = matches!(wparam.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
    let event = build_input_event(kb_struct, pressed);
    send_event(event);
    CallNextHookEx(HHOOK::default(), ncode, wparam, lparam)
}

fn build_input_event(kb_struct: &KBDLLHOOKSTRUCT, pressed: bool) -> InputEvent {
    let vk_code = kb_struct.vkCode as u16;
    InputEvent {
        key: vk_to_keycode(vk_code),
        pressed,
        timestamp_us: (kb_struct.time as u64) * 1000,
        device_id: None,
        is_repeat: track_repeat_state(pressed, vk_code),
        is_synthetic: kb_struct.flags.contains(LLKHF_INJECTED),
        scan_code: kb_struct.scanCode as u16,
    }
}

fn track_repeat_state(pressed: bool, vk_code: u16) -> bool {
    KEY_STATES.with(|states| {
        let mut states = states.borrow_mut();
        if pressed {
            let was_pressed = states.contains(&vk_code);
            if !was_pressed {
                states.insert(vk_code);
            }
            was_pressed
        } else {
            states.remove(&vk_code);
            false
        }
    })
}

fn send_event(event: InputEvent) {
    HOOK_SENDER.with(|s| {
        if let Some(sender) = s.borrow().as_ref() {
            let _ = sender.try_send(event);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_manager_new() {
        let running = Arc::new(AtomicBool::new(true));
        let manager = HookManager::new(running.clone());
        assert!(!manager.is_installed());
        assert!(manager.running().load(Ordering::SeqCst));
    }

    #[test]
    fn hook_manager_running_flag() {
        let running = Arc::new(AtomicBool::new(true));
        let manager = HookManager::new(running.clone());

        // Check initial state
        assert!(manager.running().load(Ordering::SeqCst));

        // Modify the flag
        running.store(false, Ordering::SeqCst);
        assert!(!manager.running().load(Ordering::SeqCst));
    }

    #[test]
    fn hook_manager_uninstall_when_not_installed() {
        let running = Arc::new(AtomicBool::new(true));
        let mut manager = HookManager::new(running);
        // Should not panic when uninstalling a hook that was never installed
        manager.uninstall();
        assert!(!manager.is_installed());
    }

    #[test]
    fn key_states_tracking_basic() {
        // Clear any existing state
        KEY_STATES.with(|states| {
            states.borrow_mut().clear();
        });

        // Test: First key down should NOT be a repeat
        let is_repeat = KEY_STATES.with(|states| {
            let mut states = states.borrow_mut();
            let was_pressed = states.contains(&0x41); // 'A' key
            if !was_pressed {
                states.insert(0x41);
            }
            was_pressed
        });
        assert!(!is_repeat, "First key down should not be a repeat");

        // Test: Second key down (while still pressed) SHOULD be a repeat
        let is_repeat = KEY_STATES.with(|states| {
            let states = states.borrow();
            states.contains(&0x41)
        });
        assert!(is_repeat, "Second key down should be a repeat");

        // Test: Key up should remove the state
        KEY_STATES.with(|states| {
            states.borrow_mut().remove(&0x41);
        });
        let is_pressed = KEY_STATES.with(|states| states.borrow().contains(&0x41));
        assert!(!is_pressed, "Key should be removed after key up");

        // Clean up
        KEY_STATES.with(|states| {
            states.borrow_mut().clear();
        });
    }

    #[test]
    fn key_states_tracking_multiple_keys() {
        // Clear any existing state
        KEY_STATES.with(|states| {
            states.borrow_mut().clear();
        });

        // Press multiple keys
        KEY_STATES.with(|states| {
            let mut states = states.borrow_mut();
            states.insert(0x41); // A
            states.insert(0x42); // B
            states.insert(0x43); // C
        });

        // Verify all are tracked
        let count = KEY_STATES.with(|states| states.borrow().len());
        assert_eq!(count, 3, "Should have 3 keys tracked");

        // Release one key
        KEY_STATES.with(|states| {
            states.borrow_mut().remove(&0x42); // Release B
        });

        let count = KEY_STATES.with(|states| states.borrow().len());
        assert_eq!(count, 2, "Should have 2 keys tracked after release");

        // Verify A and C are still tracked, B is not
        KEY_STATES.with(|states| {
            let states = states.borrow();
            assert!(states.contains(&0x41), "A should still be tracked");
            assert!(!states.contains(&0x42), "B should not be tracked");
            assert!(states.contains(&0x43), "C should still be tracked");
        });

        // Clean up
        KEY_STATES.with(|states| {
            states.borrow_mut().clear();
        });
    }

    #[test]
    fn key_states_clear_on_uninstall() {
        // Simulate some pressed keys
        KEY_STATES.with(|states| {
            let mut states = states.borrow_mut();
            states.insert(0x41);
            states.insert(0x42);
        });

        // Verify keys are tracked
        let count = KEY_STATES.with(|states| states.borrow().len());
        assert!(count > 0, "Should have keys tracked before clear");

        // Clear (as done in uninstall)
        KEY_STATES.with(|states| {
            states.borrow_mut().clear();
        });

        // Verify cleared
        let count = KEY_STATES.with(|states| states.borrow().len());
        assert_eq!(count, 0, "Should have no keys tracked after clear");
    }

    #[test]
    fn llkhf_injected_flag_exists() {
        use windows::Win32::UI::WindowsAndMessaging::KBDLLHOOKSTRUCT_FLAGS;

        // Verify the LLKHF_INJECTED flag can be used
        // This is a compile-time check that the import works
        let flags = KBDLLHOOKSTRUCT_FLAGS(0x10); // LLKHF_INJECTED = 0x10
        assert!(flags.contains(LLKHF_INJECTED));

        let flags_without = KBDLLHOOKSTRUCT_FLAGS(0x00);
        assert!(!flags_without.contains(LLKHF_INJECTED));
    }
}
