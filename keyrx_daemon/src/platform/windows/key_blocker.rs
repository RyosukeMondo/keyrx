//! Low-level keyboard hook for blocking and capturing remapped keys on Windows
//!
//! This module implements key blocking using SetWindowsHookExW with WH_KEYBOARD_LL.
//! When a key is remapped, we:
//! 1. Forward the event to the remapping engine via a channel
//! 2. Block the original hardware key to prevent double input
//!
//! # Architecture
//!
//! The hook serves dual purpose:
//! - **Capture**: Forward blocked key events to the event processing channel
//! - **Block**: Return 1 to prevent the original key from reaching applications
//!
//! This is necessary because WH_KEYBOARD_LL blocking prevents WM_INPUT (Raw Input)
//! from seeing the blocked keys. So the hook must be the event source for blocked keys.
//!
//! # Thread Safety
//!
//! The hook runs in the Windows message loop thread. We use OnceLock + Mutex
//! for thread-safe access to the blocker state and event sender.

use std::collections::HashSet;
use std::ptr;
use std::sync::{Arc, Mutex, OnceLock};

use crossbeam_channel::Sender;
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT,
    WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::platform::windows::keycode::scancode_to_keycode;
use keyrx_core::runtime::KeyEvent;

/// Global state for key blocking and event forwarding
static BLOCKER_STATE: OnceLock<Arc<Mutex<KeyBlockerState>>> = OnceLock::new();

/// Tracks which scan codes should be blocked and forwards events
struct KeyBlockerState {
    /// Set of scan codes that are currently remapped and should be blocked
    blocked_keys: HashSet<u32>,
    /// Channel to forward blocked key events to the remapping engine
    event_sender: Option<Sender<KeyEvent>>,
}

impl KeyBlockerState {
    fn new() -> Self {
        Self {
            blocked_keys: HashSet::new(),
            event_sender: None,
        }
    }

    fn block_key(&mut self, scan_code: u32) {
        self.blocked_keys.insert(scan_code);
    }

    fn unblock_key(&mut self, scan_code: u32) {
        self.blocked_keys.remove(&scan_code);
    }

    fn is_blocked(&self, scan_code: u32) -> bool {
        self.blocked_keys.contains(&scan_code)
    }
}

/// Key blocker manager - installs and manages the low-level keyboard hook
pub struct KeyBlocker {
    hook: HHOOK,
    state: Arc<Mutex<KeyBlockerState>>,
}

// SAFETY: HHOOK is a Windows handle (raw pointer) that is safe to send across threads.
// The handle is only used for UnhookWindowsHookEx in Drop, which is thread-safe.
unsafe impl Send for KeyBlocker {}

impl KeyBlocker {
    /// Install the low-level keyboard hook
    pub fn new() -> Result<Self, String> {
        let state = Arc::new(Mutex::new(KeyBlockerState::new()));

        match BLOCKER_STATE.set(state.clone()) {
            Ok(()) => {
                log::debug!("✓ Initialized global blocker state");
            }
            Err(_) => {
                log::debug!("Global blocker state already initialized (reusing existing)");
            }
        }

        let hook = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_hook_proc),
                GetModuleHandleW(ptr::null()),
                0,
            )
        };

        if hook == 0 as HHOOK {
            return Err("Failed to install keyboard hook".to_string());
        }

        log::info!(
            "✓ Keyboard blocker installed (hook: {:p})",
            hook as *const ()
        );

        Ok(Self { hook, state })
    }

    /// Set the event sender for forwarding blocked key events
    ///
    /// This must be called after construction to wire up the hook to the
    /// event processing pipeline. Without this, blocked keys are captured
    /// but not forwarded for remapping.
    pub fn set_event_sender(&self, sender: Sender<KeyEvent>) {
        if let Ok(mut state) = self.state.lock() {
            state.event_sender = Some(sender);
            log::info!("✓ Key blocker event forwarding enabled");
        }
    }

    /// Mark a key as remapped (will be blocked until unblocked)
    pub fn block_key(&self, scan_code: u32) {
        if let Ok(mut state) = self.state.lock() {
            let was_blocked = state.is_blocked(scan_code);
            state.block_key(scan_code);
            if !was_blocked {
                log::debug!("➕ Added scan code to blocker: 0x{:04X}", scan_code);
            }
        } else {
            log::error!(
                "✗ Failed to lock blocker state when adding scan code: 0x{:04X}",
                scan_code
            );
        }
    }

    /// Unmark a key (will no longer be blocked)
    pub fn unblock_key(&self, scan_code: u32) {
        if let Ok(mut state) = self.state.lock() {
            state.unblock_key(scan_code);
            log::trace!("Unblocking scan code: 0x{:04X}", scan_code);
        }
    }

    /// Clear all blocked keys
    pub fn clear_all(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.blocked_keys.clear();
            log::debug!("Cleared all blocked keys");
        }
    }

    /// Get count of currently blocked keys (for diagnostics/testing)
    pub fn blocked_count(&self) -> usize {
        if let Ok(state) = self.state.lock() {
            state.blocked_keys.len()
        } else {
            0
        }
    }

    /// Check if a specific scan code is blocked (for diagnostics/testing)
    #[allow(dead_code)]
    pub fn is_key_blocked(&self, scan_code: u32) -> bool {
        if let Ok(state) = self.state.lock() {
            state.is_blocked(scan_code)
        } else {
            false
        }
    }
}

impl Drop for KeyBlocker {
    fn drop(&mut self) {
        if self.hook != 0 as HHOOK {
            unsafe {
                UnhookWindowsHookEx(self.hook);
            }
            log::info!("✓ Keyboard blocker uninstalled");
        }
    }
}

/// Low-level keyboard hook procedure
///
/// Called by Windows for every keyboard event system-wide.
/// For blocked keys: forwards the event to the remapping engine, then blocks.
/// For unblocked keys: passes through to the next hook.
unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if code != HC_ACTION as i32 {
        return CallNextHookEx(0 as HHOOK, code, w_param, l_param);
    }

    let kbd = *(l_param as *const KBDLLHOOKSTRUCT);

    // Skip injected events to avoid feedback loops
    // LLKHF_INJECTED = 0x10, LLKHF_LOWER_IL_INJECTED = 0x02
    if (kbd.flags & 0x10) != 0 {
        return CallNextHookEx(0 as HHOOK, code, w_param, l_param);
    }

    let scan_code = kbd.scanCode;
    let is_extended = (kbd.flags & 0x01) != 0;

    let full_scan_code = if is_extended {
        scan_code | 0xE000
    } else {
        scan_code
    };

    let Some(state_arc) = BLOCKER_STATE.get() else {
        return CallNextHookEx(0 as HHOOK, code, w_param, l_param);
    };

    let Ok(state) = state_arc.lock() else {
        log::error!("✗ Failed to lock BLOCKER_STATE in hook callback!");
        return CallNextHookEx(0 as HHOOK, code, w_param, l_param);
    };

    let is_blocked = state.is_blocked(full_scan_code);

    // Forward ALL physical (non-injected) key events to the remapping engine.
    // The hook is the sole event source — Raw Input also sees injected events
    // and cannot distinguish them from physical, causing feedback loops.
    if let Some(ref sender) = state.event_sender {
        if let Some(keycode) = scancode_to_keycode(full_scan_code) {
            let is_release = matches!(w_param as u32, WM_KEYUP | WM_SYSKEYUP);
            let event = if is_release {
                KeyEvent::release(keycode)
            } else {
                KeyEvent::press(keycode)
            };

            let _ = sender.try_send(event);
        }
    }

    if !is_blocked {
        // Not blocked — let Windows process it normally AND we forwarded it
        // Raw Input will also see this, but we'll filter duplicates there
        return CallNextHookEx(0 as HHOOK, code, w_param, l_param);
    }

    log::trace!(
        "✋ BLOCKED scan code: 0x{:04X} ({})",
        full_scan_code,
        if matches!(w_param as u32, WM_KEYDOWN | WM_SYSKEYDOWN) {
            "press"
        } else {
            "release"
        }
    );

    // Block the original key (prevents double input for remapped keys)
    1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocker_state() {
        let mut state = KeyBlockerState::new();

        assert!(!state.is_blocked(0x1E));

        state.block_key(0x1E);
        assert!(state.is_blocked(0x1E));

        state.unblock_key(0x1E);
        assert!(!state.is_blocked(0x1E));
    }

    #[test]
    fn test_extended_scan_codes() {
        let mut state = KeyBlockerState::new();

        state.block_key(0xE01D);
        assert!(state.is_blocked(0xE01D));
        assert!(!state.is_blocked(0x1D));
    }
}
