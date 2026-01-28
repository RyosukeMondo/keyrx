

//! Low-level keyboard hook for blocking remapped keys on Windows
//!
//! This module implements key blocking using SetWindowsHookExW with WH_KEYBOARD_LL.
//! When a key is remapped, we block the original hardware key to prevent double input.
//!
//! # Architecture
//!
//! 1. Install low-level keyboard hook (WH_KEYBOARD_LL)
//! 2. On each keyboard event, check if key is being remapped
//! 3. If remapped: Return 1 (block the key from propagating)
//! 4. If not remapped: Call CallNextHookEx (allow normal processing)
//!
//! # Thread Safety
//!
//! The hook runs in the Windows message loop thread. We use atomic operations
//! and lock-free data structures for fast, thread-safe key lookups.

use std::collections::HashSet;
use std::ptr;
use std::sync::{Arc, Mutex};
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HC_ACTION, HHOOK,
    KBDLLHOOKSTRUCT, WH_KEYBOARD_LL, WM_KEYDOWN, WM_SYSKEYDOWN,
};

/// Global state for key blocking
/// Uses thread_local to avoid cross-thread issues in the hook
thread_local! {
    static BLOCKER_STATE: std::cell::RefCell<Option<Arc<Mutex<KeyBlockerState>>>> =
        std::cell::RefCell::new(None);
}

/// Tracks which scan codes should be blocked (currently being remapped)
struct KeyBlockerState {
    /// Set of scan codes that are currently remapped and should be blocked
    blocked_keys: HashSet<u32>,
}

impl KeyBlockerState {
    fn new() -> Self {
        Self {
            blocked_keys: HashSet::new(),
        }
    }

    /// Mark a scan code as remapped (should be blocked)
    fn block_key(&mut self, scan_code: u32) {
        self.blocked_keys.insert(scan_code);
    }

    /// Unmark a scan code (should not be blocked)
    fn unblock_key(&mut self, scan_code: u32) {
        self.blocked_keys.remove(&scan_code);
    }

    /// Check if a scan code is currently blocked
    fn is_blocked(&self, scan_code: u32) -> bool {
        self.blocked_keys.contains(&scan_code)
    }
}

/// Key blocker manager - installs and manages the low-level keyboard hook
pub struct KeyBlocker {
    hook: HHOOK,
    state: Arc<Mutex<KeyBlockerState>>,
}

impl KeyBlocker {
    /// Install the low-level keyboard hook
    ///
    /// # Safety
    ///
    /// This function installs a Windows system hook which affects all keyboard input.
    /// The hook must be properly uninstalled when dropped.
    pub fn new() -> Result<Self, String> {
        let state = Arc::new(Mutex::new(KeyBlockerState::new()));
        let state_clone = state.clone();

        // Set thread-local state for the hook
        BLOCKER_STATE.with(|tls| {
            *tls.borrow_mut() = Some(state_clone);
        });

        // Install low-level keyboard hook
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

        log::info!("✓ Keyboard blocker installed (hook: {:p})", hook as *const ());

        Ok(Self { hook, state })
    }

    /// Mark a key as remapped (will be blocked until unblocked)
    pub fn block_key(&self, scan_code: u32) {
        if let Ok(mut state) = self.state.lock() {
            state.block_key(scan_code);
            log::trace!("Blocking scan code: 0x{:04X}", scan_code);
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
}

impl Drop for KeyBlocker {
    fn drop(&mut self) {
        if self.hook != 0 as HHOOK {
            unsafe {
                UnhookWindowsHookEx(self.hook);
            }
            log::info!("✓ Keyboard blocker uninstalled");
        }

        // Clear thread-local state
        BLOCKER_STATE.with(|tls| {
            *tls.borrow_mut() = None;
        });
    }
}

/// Low-level keyboard hook procedure
///
/// This is called by Windows for every keyboard event system-wide.
/// We check if the key is being remapped and block it if necessary.
unsafe extern "system" fn keyboard_hook_proc(
    code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    // Only process if code is HC_ACTION
    if code != HC_ACTION as i32 {
        return CallNextHookEx(0 as HHOOK, code, w_param, l_param);
    }

    // Get keyboard event details
    let kbd = *(l_param as *const KBDLLHOOKSTRUCT);
    let scan_code = kbd.scanCode;
    let is_extended = (kbd.flags & 0x01) != 0; // LLKHF_EXTENDED

    // Construct full scan code (extended keys use high byte)
    let full_scan_code = if is_extended {
        (scan_code as u32) | 0xE000
    } else {
        scan_code as u32
    };

    // Check if this key should be blocked
    let should_block = BLOCKER_STATE.with(|tls| {
        if let Some(state_arc) = tls.borrow().as_ref() {
            if let Ok(state) = state_arc.lock() {
                return state.is_blocked(full_scan_code);
            }
        }
        false
    });

    if should_block {
        // Block the key by returning 1
        log::trace!(
            "✋ Blocked scan code: 0x{:04X} ({})",
            full_scan_code,
            if matches!(w_param as u32, WM_KEYDOWN | WM_SYSKEYDOWN) {
                "press"
            } else {
                "release"
            }
        );
        return 1;
    }

    // Allow the key through
    CallNextHookEx(0 as HHOOK, code, w_param, l_param)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocker_state() {
        let mut state = KeyBlockerState::new();

        // Initially nothing is blocked
        assert!(!state.is_blocked(0x1E)); // A key

        // Block a key
        state.block_key(0x1E);
        assert!(state.is_blocked(0x1E));

        // Unblock it
        state.unblock_key(0x1E);
        assert!(!state.is_blocked(0x1E));
    }

    #[test]
    fn test_extended_scan_codes() {
        let mut state = KeyBlockerState::new();

        // Block extended key (e.g., Right Ctrl = 0xE01D)
        state.block_key(0xE01D);
        assert!(state.is_blocked(0xE01D));
        assert!(!state.is_blocked(0x1D)); // Left Ctrl should not be blocked
    }
}
