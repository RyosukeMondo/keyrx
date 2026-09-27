//! E2E test bridge: replays simulated physical key events through the
//! low-level keyboard hook so they reach the same `RawInputManager`
//! subscribers a real physical keypress would.
//!
//! Raw Input cannot distinguish injected events (`SendInput`) from physical
//! keypresses, so tests that need to simulate a *physical* key must go
//! through this low-level hook instead, which does have `LLKHF_INJECTED`
//! flag access. Only events tagged with `TEST_SIMULATED_PHYSICAL_MARKER` are
//! bridged; everything else passes through untouched.

use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, HC_ACTION, KBDLLHOOKSTRUCT, LLKHF_EXTENDED, WM_KEYDOWN, WM_KEYUP,
    WM_SYSKEYDOWN, WM_SYSKEYUP,
};

use crate::platform::windows::keycode::scancode_to_keycode;
use keyrx_core::runtime::KeyEvent;

use super::BRIDGE_CONTEXT_TLS;

const TEST_SIMULATED_PHYSICAL_MARKER: usize = 0x54455354; // "TEST"

// Low-level hook callback to bridge E2E test events into RawInput stream.
// This is ONLY used for events marked with TEST_SIMULATED_PHYSICAL_MARKER.
pub(super) unsafe extern "system" fn test_bridge_hook(
    code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if code == (HC_ACTION as i32) {
        let kbd = *(l_param as *const KBDLLHOOKSTRUCT);
        log::debug!("Hook: dwExtraInfo={:x}", kbd.dwExtraInfo);
        if kbd.dwExtraInfo == TEST_SIMULATED_PHYSICAL_MARKER {
            log::info!("Bridge Hook triggered for test event!");

            // Access bridge context from thread-local storage
            BRIDGE_CONTEXT_TLS.with(|tls| {
                if let Some(bridge_context_arc) = tls.borrow().as_ref() {
                    if let Ok(context_guard) = bridge_context_arc.lock() {
                        if let Some(ctx) = context_guard.as_ref() {
                            let is_release = match w_param as u32 {
                                WM_KEYUP | WM_SYSKEYUP => true,
                                WM_KEYDOWN | WM_SYSKEYDOWN => false,
                                _ => false,
                            };

                            let scan_code = kbd.scanCode as u16;
                            let extended = (kbd.flags & LLKHF_EXTENDED) != 0;

                            // Adjust scan code for extended keys to match Raw Input expectations
                            let sc = if extended {
                                scan_code as u32 | 0xE000
                            } else {
                                scan_code as u32
                            };

                            if let Some(keycode) = scancode_to_keycode(sc) {
                                let event = if is_release {
                                    KeyEvent::release(keycode)
                                } else {
                                    KeyEvent::press(keycode)
                                };

                                log::debug!("Bridge Hook event: {:?}", event);

                                // Broadcast to ALL subscribers since we don't know the device ID for a test event
                                if let Ok(subs) = ctx.subscribers.read() {
                                    for sender in subs.values() {
                                        let _ = sender.try_send(event.clone());
                                    }
                                }

                                // Also send to global sender
                                let _ = ctx.global_sender.try_send(event);
                            }
                        }
                    }
                }
            });
        }
    }
    CallNextHookEx(0 as _, code, w_param, l_param)
}
