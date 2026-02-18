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
//! The hook runs in the Windows message loop thread. We use lock-free atomics
//! for the blocked keys bitset to ensure the hook callback never blocks.

use std::ptr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use crossbeam_channel::Sender;
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT,
    WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::platform::windows::keycode::scancode_to_keycode;
use keyrx_core::runtime::KeyEvent;

/// Number of AtomicU64 words needed to cover 65536 scan codes (1024 words * 64 bits)
const BITSET_WORDS: usize = 1024;

/// Global atomic bitset for blocked scan codes. Lock-free.
static BLOCKED_KEYS: OnceLock<Vec<AtomicU64>> = OnceLock::new();

/// Global event sender for forwarding key events to the remapping engine.
static EVENT_SENDER: OnceLock<Sender<KeyEvent>> = OnceLock::new();

/// Initialize the global blocked keys bitset (all zeros = nothing blocked).
fn init_blocked_keys() -> Vec<AtomicU64> {
    (0..BITSET_WORDS).map(|_| AtomicU64::new(0)).collect()
}

/// Get the bitset word index and bit mask for a scan code.
fn scan_code_position(scan_code: u32) -> (usize, u64) {
    let index = (scan_code as usize) / 64;
    let bit = 1u64 << ((scan_code as usize) % 64);
    (index.min(BITSET_WORDS - 1), bit)
}

/// Check if a scan code is blocked. Lock-free atomic load.
fn is_blocked(scan_code: u32) -> bool {
    let keys = match BLOCKED_KEYS.get() {
        Some(k) => k,
        None => return false,
    };
    let (index, bit) = scan_code_position(scan_code);
    (keys[index].load(Ordering::Relaxed) & bit) != 0
}

/// Key blocker manager - installs and manages the low-level keyboard hook
pub struct KeyBlocker {
    hook: HHOOK,
}

// SAFETY: HHOOK is a Windows handle (raw pointer) that is safe to send across threads.
// The handle is only used for UnhookWindowsHookEx in Drop, which is thread-safe.
unsafe impl Send for KeyBlocker {}

impl KeyBlocker {
    /// Install the low-level keyboard hook
    pub fn new() -> Result<Self, String> {
        // Initialize global bitset (idempotent via OnceLock)
        BLOCKED_KEYS.get_or_init(init_blocked_keys);

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

        Ok(Self { hook })
    }

    /// Set the event sender for forwarding blocked key events
    ///
    /// This must be called after construction to wire up the hook to the
    /// event processing pipeline. Without this, blocked keys are captured
    /// but not forwarded for remapping.
    pub fn set_event_sender(&self, sender: Sender<KeyEvent>) {
        match EVENT_SENDER.set(sender) {
            Ok(()) => log::info!("✓ Key blocker event forwarding enabled"),
            Err(_) => log::debug!("Event sender already set (reusing existing)"),
        }
    }

    /// Mark a key as remapped (will be blocked until unblocked). Lock-free.
    pub fn block_key(&self, scan_code: u32) {
        let keys = BLOCKED_KEYS.get().expect("BLOCKED_KEYS not initialized");
        let (index, bit) = scan_code_position(scan_code);
        let prev = keys[index].fetch_or(bit, Ordering::Relaxed);
        if (prev & bit) == 0 {
            log::debug!("➕ Added scan code to blocker: 0x{:04X}", scan_code);
        }
    }

    /// Unmark a key (will no longer be blocked). Lock-free.
    pub fn unblock_key(&self, scan_code: u32) {
        let keys = BLOCKED_KEYS.get().expect("BLOCKED_KEYS not initialized");
        let (index, bit) = scan_code_position(scan_code);
        keys[index].fetch_and(!bit, Ordering::Relaxed);
        log::trace!("Unblocking scan code: 0x{:04X}", scan_code);
    }

    /// Clear all blocked keys. Lock-free.
    pub fn clear_all(&self) {
        let keys = BLOCKED_KEYS.get().expect("BLOCKED_KEYS not initialized");
        for word in keys.iter() {
            word.store(0, Ordering::Relaxed);
        }
        log::debug!("Cleared all blocked keys");
    }

    /// Get count of currently blocked keys (for diagnostics/testing)
    pub fn blocked_count(&self) -> usize {
        let keys = BLOCKED_KEYS.get().expect("BLOCKED_KEYS not initialized");
        keys.iter()
            .map(|w| w.load(Ordering::Relaxed).count_ones() as usize)
            .sum()
    }

    /// Check if a specific scan code is blocked (for diagnostics/testing)
    #[allow(dead_code)]
    pub fn is_key_blocked(&self, scan_code: u32) -> bool {
        is_blocked(scan_code)
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
///
/// # Safety
/// This is wait-free — no locks, no allocations, only atomic loads and channel try_send.
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

    let blocked = is_blocked(full_scan_code);

    // Forward ALL physical (non-injected) key events to the remapping engine.
    // The hook is the sole event source — Raw Input also sees injected events
    // and cannot distinguish them from physical, causing feedback loops.
    if let Some(sender) = EVENT_SENDER.get() {
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

    if !blocked {
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
    fn test_scan_code_position() {
        let (idx, bit) = scan_code_position(0);
        assert_eq!(idx, 0);
        assert_eq!(bit, 1);

        let (idx, bit) = scan_code_position(63);
        assert_eq!(idx, 0);
        assert_eq!(bit, 1u64 << 63);

        let (idx, bit) = scan_code_position(64);
        assert_eq!(idx, 1);
        assert_eq!(bit, 1);
    }

    #[test]
    fn test_is_blocked_without_init() {
        // Before BLOCKED_KEYS is initialized, is_blocked returns false
        // Note: OnceLock may already be initialized from other tests
        // Just verify it doesn't panic
        let _ = is_blocked(0x1E);
    }

    #[test]
    fn test_blocked_keys_bitset() {
        // Initialize
        let keys = BLOCKED_KEYS.get_or_init(init_blocked_keys);

        // Nothing blocked initially
        assert!(!is_blocked(0x1E));

        // Block a key
        let (idx, bit) = scan_code_position(0x1E);
        keys[idx].fetch_or(bit, Ordering::Relaxed);
        assert!(is_blocked(0x1E));

        // Unblock
        keys[idx].fetch_and(!bit, Ordering::Relaxed);
        assert!(!is_blocked(0x1E));
    }

    #[test]
    fn test_extended_scan_codes() {
        let keys = BLOCKED_KEYS.get_or_init(init_blocked_keys);

        let (idx, bit) = scan_code_position(0xE01D);
        keys[idx].fetch_or(bit, Ordering::Relaxed);
        assert!(is_blocked(0xE01D));
        assert!(!is_blocked(0x1D));

        // Cleanup
        keys[idx].fetch_and(!bit, Ordering::Relaxed);
    }
}
