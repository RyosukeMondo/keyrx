//! Windows low-level keyboard hook management.
//!
//! This module provides the `HookManager` for managing the lifecycle of a Windows
//! keyboard hook, and the `low_level_keyboard_proc` callback function.

#![allow(unsafe_code)]

use crate::drivers::common::error::DriverError;
use crate::engine::{InputEvent, KeyCode};
use crate::safety::panic_guard::PanicGuard;
use crossbeam_channel::Sender;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tracing::debug;
use windows::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, PeekMessageW, TranslateMessage, HHOOK, KBDLLHOOKSTRUCT,
    LLKHF_EXTENDED, LLKHF_INJECTED, MSG, PM_REMOVE, WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN,
};

use super::keymap::vk_to_keycode;
use super::safety::hook::SafeHook;
use super::safety::thread_local::ThreadLocalState;
use crate::config::{VK_CONTROL, VK_ESCAPE, VK_MENU, VK_SHIFT};
use crate::drivers::emergency_exit::{is_bypass_active, toggle_bypass_mode};

/// Low-level keyboard hook manager.
///
/// Manages the lifecycle of a Windows keyboard hook, including installation,
/// event capture, and cleanup.
///
/// This manager now uses the `SafeHook` wrapper to ensure proper RAII
/// semantics and safe hook lifecycle management.
pub struct HookManager {
    /// Safe RAII wrapper for the Windows keyboard hook.
    safe_hook: Option<SafeHook>,
    /// Flag to signal the message pump to stop.
    running: Arc<AtomicBool>,
    /// Storage for the thread ID of the message loop
    thread_id_store: Arc<AtomicU32>,
}

impl HookManager {
    /// Create a new HookManager.
    ///
    /// The hook is not installed until `install()` is called.
    pub fn new(running: Arc<AtomicBool>, thread_id_store: Arc<AtomicU32>) -> Self {
        Self {
            safe_hook: None,
            running,
            thread_id_store,
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
    /// * `cache` - LRU cache for VK code to KeyCode mappings
    ///
    /// # Errors
    ///
    /// Returns `DriverError::HookFailed` if the hook cannot be installed.
    pub fn install(
        &mut self,
        sender: Sender<InputEvent>,
        cache: Arc<crate::drivers::common::cache::LruKeymapCache>,
    ) -> Result<(), DriverError> {
        // Use SafeHook to install the hook with proper RAII semantics
        let hook = SafeHook::install(
            sender,
            self.running.clone(),
            self.thread_id_store.clone(),
            cache,
        )?;

        self.safe_hook = Some(hook);
        Ok(())
    }

    /// Uninstall the keyboard hook.
    ///
    /// This should be called before the thread exits to properly clean up.
    /// The SafeHook wrapper will automatically uninstall on drop, but this
    /// provides explicit control over the timing.
    pub fn uninstall(&mut self) {
        if let Some(mut hook) = self.safe_hook.take() {
            hook.uninstall();
        }
    }

    /// Check if the hook is currently installed.
    #[allow(dead_code)]
    pub fn is_installed(&self) -> bool {
        self.safe_hook
            .as_ref()
            .map(|h| h.is_installed())
            .unwrap_or(false)
    }

    /// Get the running flag.
    #[allow(dead_code)]
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
        // SAFETY: GetCurrentThreadId is always safe to call. It returns the thread ID
        // of the calling thread and has no preconditions or failure modes.
        let thread_id = unsafe { GetCurrentThreadId() };
        self.thread_id_store.store(thread_id, Ordering::SeqCst);
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
        // SAFETY: PeekMessageW is safe to call with a valid MSG pointer.
        // We pass a mutable reference to our MSG struct, None for any window,
        // and PM_REMOVE to retrieve and remove messages from the queue.
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
            // SAFETY: TranslateMessage and DispatchMessageW are safe to call with
            // a valid MSG pointer that was filled by PeekMessageW. These functions
            // process Windows messages and dispatch them to window procedures.
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
        self.thread_id_store.store(0, Ordering::SeqCst);
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
/// The callback is wrapped in PanicGuard to prevent panics from escaping to Windows,
/// which would cause the entire hook to be uninstalled. The emergency exit check
/// remains outside PanicGuard to ensure it always works even if the main processing
/// panics.
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

    // EMERGENCY EXIT CHECK - must be FIRST and OUTSIDE PanicGuard
    // This ensures emergency exit always works, even if processing panics
    let kb_struct = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
    let pressed = matches!(wparam.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN);

    if pressed && check_emergency_exit_combo(kb_struct.vkCode as i32) {
        toggle_bypass_mode();
        // Pass through the Escape key so it doesn't get stuck
        return CallNextHookEx(HHOOK::default(), ncode, wparam, lparam);
    }

    // If bypass mode is active, pass through all keys without processing
    if is_bypass_active() {
        return CallNextHookEx(HHOOK::default(), ncode, wparam, lparam);
    }

    // Wrap main processing in PanicGuard to prevent panics from uninstalling the hook
    let result = PanicGuard::new("windows_keyboard_hook")
        .execute(|| process_keyboard_event(kb_struct, pressed));

    match result {
        Ok(_) => CallNextHookEx(HHOOK::default(), ncode, wparam, lparam),
        Err(err) => {
            // Panic was caught - log it (PanicGuard already logged details)
            // and pass through the key to maintain keyboard functionality
            tracing::error!(
                service = "keyrx",
                event = "hook_panic_recovered",
                error = %err,
                "Hook callback panicked but recovered - passing through key"
            );
            CallNextHookEx(HHOOK::default(), ncode, wparam, lparam)
        }
    }
}

/// Process a keyboard event - separated for PanicGuard wrapping.
///
/// This is the main event processing logic that is wrapped in PanicGuard
/// to prevent panics from escaping the hook callback.
fn process_keyboard_event(kb_struct: &KBDLLHOOKSTRUCT, pressed: bool) {
    let event = build_input_event(kb_struct, pressed);
    send_event(event);
}

/// Check if the emergency exit key combination is active.
///
/// Returns true if Escape is pressed while Ctrl+Alt+Shift are all held down.
#[inline]
fn check_emergency_exit_combo(vk_code: i32) -> bool {
    if vk_code != VK_ESCAPE {
        return false;
    }

    // SAFETY: GetAsyncKeyState is safe to call with valid virtual key codes.
    // It returns the state of the specified key, with the high bit (0x8000) set
    // if the key is currently pressed. We use valid VK_* constants from config.
    unsafe {
        let ctrl_down = (GetAsyncKeyState(VK_CONTROL) as u16 & 0x8000) != 0;
        let alt_down = (GetAsyncKeyState(VK_MENU) as u16 & 0x8000) != 0;
        let shift_down = (GetAsyncKeyState(VK_SHIFT) as u16 & 0x8000) != 0;

        ctrl_down && alt_down && shift_down
    }
}

fn build_input_event(kb_struct: &KBDLLHOOKSTRUCT, pressed: bool) -> InputEvent {
    let vk_code = kb_struct.vkCode as u16;
    let is_extended = kb_struct.flags.contains(LLKHF_EXTENDED);

    // Use ThreadLocalState for safe key state tracking
    let is_repeat = ThreadLocalState::track_key_state(vk_code, pressed);
    let identity = ThreadLocalState::device_identity();

    InputEvent {
        key: map_vk_to_keycode(vk_code, is_extended),
        pressed,
        timestamp_us: (kb_struct.time as u64) * 1000,
        device_id: None,
        is_repeat,
        is_synthetic: kb_struct.flags.contains(LLKHF_INJECTED),
        scan_code: kb_struct.scanCode as u16,
        serial_number: identity.as_ref().map(|id| id.serial_number.clone()),
        vendor_id: identity.as_ref().map(|id| id.vendor_id),
        product_id: identity.as_ref().map(|id| id.product_id),
    }
}

fn map_vk_to_keycode(vk_code: u16, is_extended: bool) -> KeyCode {
    // Distinguish numpad Enter (VK_RETURN with extended flag) from main Enter.
    if vk_code == 0x0D && is_extended {
        return KeyCode::NumpadEnter;
    }

    // Try to get KeyCode from cache first
    use super::safety::hook::KEYMAP_CACHE;
    use crate::drivers::common::cache::KeymapCache;

    // Use a fixed device ID for Windows since we don't track individual devices
    const WINDOWS_DEVICE_ID: &str = "windows";

    KEYMAP_CACHE.with(|cache_cell| {
        if let Some(cache) = cache_cell.borrow().as_ref() {
            // Try cache first
            cache
                .get(vk_code as u32, WINDOWS_DEVICE_ID)
                .unwrap_or_else(|| {
                    // Cache miss - perform actual lookup
                    let keycode = vk_to_keycode(vk_code);
                    // Store in cache for future lookups
                    cache.insert(vk_code as u32, WINDOWS_DEVICE_ID, keycode);
                    keycode
                })
        } else {
            // No cache available (shouldn't happen in normal operation)
            vk_to_keycode(vk_code)
        }
    })
}

fn send_event(event: InputEvent) {
    // Use ThreadLocalState for safe event sending
    ThreadLocalState::try_send(event);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hook_manager_new() {
        let running = Arc::new(AtomicBool::new(true));
        let thread_id_store = Arc::new(AtomicU32::new(0));
        let manager = HookManager::new(running.clone(), thread_id_store);
        assert!(!manager.is_installed());
        assert!(manager.running().load(Ordering::SeqCst));
    }

    #[test]
    fn hook_manager_running_flag() {
        let running = Arc::new(AtomicBool::new(true));
        let thread_id_store = Arc::new(AtomicU32::new(0));
        let manager = HookManager::new(running.clone(), thread_id_store);

        // Check initial state
        assert!(manager.running().load(Ordering::SeqCst));

        // Modify the flag
        running.store(false, Ordering::SeqCst);
        assert!(!manager.running().load(Ordering::SeqCst));
    }

    #[test]
    fn hook_manager_uninstall_when_not_installed() {
        let running = Arc::new(AtomicBool::new(true));
        let thread_id_store = Arc::new(AtomicU32::new(0));
        let mut manager = HookManager::new(running, thread_id_store);
        // Should not panic when uninstalling a hook that was never installed
        manager.uninstall();
        assert!(!manager.is_installed());
    }

    #[test]
    fn key_states_tracking_basic() {
        // Clear any existing state
        ThreadLocalState::cleanup();

        // Test: First key down should NOT be a repeat
        let is_repeat = ThreadLocalState::track_key_state(0x41, true); // 'A' key
        assert!(!is_repeat, "First key down should not be a repeat");

        // Test: Second key down (while still pressed) SHOULD be a repeat
        let is_repeat = ThreadLocalState::track_key_state(0x41, true);
        assert!(is_repeat, "Second key down should be a repeat");

        // Test: Key up should remove the state
        ThreadLocalState::track_key_state(0x41, false);
        let is_pressed = ThreadLocalState::is_key_pressed(0x41);
        assert!(!is_pressed, "Key should be removed after key up");

        // Clean up
        ThreadLocalState::cleanup();
    }

    #[test]
    fn key_states_tracking_multiple_keys() {
        // Clear any existing state
        ThreadLocalState::cleanup();

        // Press multiple keys
        ThreadLocalState::mark_key_pressed(0x41); // A
        ThreadLocalState::mark_key_pressed(0x42); // B
        ThreadLocalState::mark_key_pressed(0x43); // C

        // Verify all are tracked
        let count = ThreadLocalState::pressed_key_count();
        assert_eq!(count, 3, "Should have 3 keys tracked");

        // Release one key
        ThreadLocalState::mark_key_released(0x42); // Release B

        let count = ThreadLocalState::pressed_key_count();
        assert_eq!(count, 2, "Should have 2 keys tracked after release");

        // Verify A and C are still tracked, B is not
        assert!(
            ThreadLocalState::is_key_pressed(0x41),
            "A should still be tracked"
        );
        assert!(
            !ThreadLocalState::is_key_pressed(0x42),
            "B should not be tracked"
        );
        assert!(
            ThreadLocalState::is_key_pressed(0x43),
            "C should still be tracked"
        );

        // Clean up
        ThreadLocalState::cleanup();
    }

    #[test]
    fn key_states_clear_on_uninstall() {
        // Simulate some pressed keys
        ThreadLocalState::mark_key_pressed(0x41);
        ThreadLocalState::mark_key_pressed(0x42);

        // Verify keys are tracked
        let count = ThreadLocalState::pressed_key_count();
        assert!(count > 0, "Should have keys tracked before clear");

        // Clear (as done in uninstall)
        ThreadLocalState::cleanup();

        // Verify cleared
        let count = ThreadLocalState::pressed_key_count();
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

    #[test]
    fn map_vk_to_keycode_distinguishes_numpad_enter() {
        let key = map_vk_to_keycode(0x0D, true);
        assert_eq!(key, KeyCode::NumpadEnter);

        let key = map_vk_to_keycode(0x0D, false);
        assert_eq!(key, KeyCode::Enter);
    }

    #[test]
    fn build_input_event_handles_extended_enter() {
        // Clear state before test
        ThreadLocalState::cleanup();

        let kb_struct = KBDLLHOOKSTRUCT {
            vkCode: 0x0D,
            scanCode: 0x1C,
            flags: LLKHF_EXTENDED,
            time: 123,
            dwExtraInfo: 0,
        };

        let event = build_input_event(&kb_struct, true);
        assert_eq!(event.key, KeyCode::NumpadEnter);
        assert!(event.pressed);
        assert_eq!(event.timestamp_us, 123_000);
        assert!(!event.is_synthetic);
        assert!(!event.is_repeat); // First press should not be a repeat

        // Clean up
        ThreadLocalState::cleanup();
    }

    #[test]
    fn vk_constants_from_config_correct() {
        // Verify VK constants from config module match Windows API values
        use crate::config::keys::{VK_CONTROL, VK_ESCAPE, VK_MENU, VK_SHIFT};
        assert_eq!(VK_ESCAPE, 0x1B);
        assert_eq!(VK_CONTROL, 0x11);
        assert_eq!(VK_SHIFT, 0x10);
        assert_eq!(VK_MENU, 0x12);
    }

    #[test]
    fn check_emergency_exit_combo_wrong_key() {
        // When not pressing Escape, should return false regardless of modifiers
        // This test only checks the key code part, not GetAsyncKeyState behavior
        assert!(!check_emergency_exit_combo(0x41)); // 'A' key
        assert!(!check_emergency_exit_combo(VK_CONTROL)); // Ctrl key itself
        assert!(!check_emergency_exit_combo(VK_SHIFT)); // Shift key itself
        assert!(!check_emergency_exit_combo(VK_MENU)); // Alt key itself
    }
}
