//! Raw Input registration and event routing for Windows keyboard capture.
//!
//! `RawInputManager` owns the hidden message-only window Raw Input requires,
//! subscribes callers to per-device key events, and (for E2E tests) bridges
//! simulated physical key presses through a low-level keyboard hook so they
//! reach the same subscribers a real keypress would.
//!
//! Split by responsibility:
//! - [`window`]: the Win32 message-only window and its message pump
//! - [`test_bridge`]: the E2E test hook that replays simulated key events

mod test_bridge;
mod window;

#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::size_of;
use std::ptr;
use std::sync::{Arc, Mutex, RwLock};

use crossbeam_channel::{unbounded, Receiver, Sender};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::{RegisterRawInputDevices, RAWINPUTDEVICE, RAWKEYBOARD};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DestroyWindow, GetWindowLongPtrW, SetWindowLongPtrW, SetWindowsHookExW, UnhookWindowsHookEx,
    GWLP_USERDATA, HHOOK, WH_KEYBOARD_LL,
};

use crate::platform::recovery::recover_lock_with_context;
use crate::platform::windows::device_map::DeviceMap;
use crate::platform::PlatformError;
use keyrx_core::runtime::KeyEvent;

// Thread-local storage for bridge context used by the hook callback
thread_local! {
    static BRIDGE_CONTEXT_TLS: RefCell<Option<Arc<Mutex<Option<BridgeContextHandle>>>>> = const { RefCell::new(None) };
}

pub struct BridgeContextHandle {
    global_sender: Sender<KeyEvent>,
    subscribers: Arc<RwLock<HashMap<usize, Sender<KeyEvent>>>>,
}

/// Manages Raw Input registration and routes events to device-specific channels.
pub struct RawInputManager {
    pub hwnd: HWND,
    _device_map: DeviceMap,
    subscribers: Arc<RwLock<HashMap<usize, Sender<KeyEvent>>>>,
    _global_sender: Sender<KeyEvent>,
    bridge_context: Arc<Mutex<Option<BridgeContextHandle>>>,
    bridge_hook: Arc<Mutex<Option<isize>>>,
}

impl RawInputManager {
    pub fn new(
        device_map: DeviceMap,
        global_sender: Sender<KeyEvent>,
        bridge_context: Arc<Mutex<Option<BridgeContextHandle>>>,
        bridge_hook: Arc<Mutex<Option<isize>>>,
    ) -> Result<Self, PlatformError> {
        // 1. Create message-only window
        let hwnd = unsafe { window::create_message_window()? };

        let subscribers = Arc::new(RwLock::new(HashMap::new()));

        let manager = Self {
            hwnd,
            _device_map: device_map.clone(),
            subscribers: subscribers.clone(),
            _global_sender: global_sender.clone(),
            bridge_context: bridge_context.clone(),
            bridge_hook: bridge_hook.clone(),
        };

        let context = Box::new(RawInputContext {
            subscribers: subscribers.clone(),
            device_map: device_map.clone(),
            global_sender: global_sender.clone(),
        });

        unsafe {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(context) as isize);
        }

        // 3. Register for Raw Input
        unsafe { window::register_raw_input(hwnd)? };

        // 4. Install Test Bridge Hook for E2E testing
        {
            let mut context_guard =
                recover_lock_with_context(&bridge_context, "RawInputManager::new bridge_context")?;
            *context_guard = Some(BridgeContextHandle {
                global_sender: global_sender.clone(),
                subscribers: subscribers.clone(),
            });

            // Set thread-local storage for hook callback access
            BRIDGE_CONTEXT_TLS.with(|tls| {
                *tls.borrow_mut() = Some(bridge_context.clone());
            });

            let mut hook_guard =
                recover_lock_with_context(&bridge_hook, "RawInputManager::new bridge_hook")?;
            unsafe {
                let hook = SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(test_bridge::test_bridge_hook),
                    GetModuleHandleW(ptr::null()),
                    0,
                );
                if hook != 0 as _ {
                    *hook_guard = Some(hook as isize);
                    log::info!("E2E Test Bridge Hook installed");
                } else {
                    log::warn!("Failed to install E2E Test Bridge Hook");
                }
            }
        }

        Ok(manager)
    }

    /// Subscribes to events from a specific device handle.
    /// Returns a Receiver that will receive KeyEvents for that device.
    pub fn subscribe(&self, device_handle: usize) -> Receiver<KeyEvent> {
        let (sender, receiver) = unbounded();
        match self.subscribers.write() {
            Ok(mut subscribers) => {
                subscribers.insert(device_handle, sender);
            }
            Err(_) => {
                log::error!("Subscribers lock poisoned in subscribe");
            }
        }
        receiver
    }

    /// Unsubscribes a device (e.g., on removal).
    pub fn unsubscribe(&self, device_handle: usize) {
        match self.subscribers.write() {
            Ok(mut subscribers) => {
                subscribers.remove(&device_handle);
            }
            Err(_) => {
                log::error!("Subscribers lock poisoned in unsubscribe");
            }
        }
    }

    /// Simulates a raw input event for testing purposes.
    /// This bypasses the Win32 message loop and directly processes the event.
    /// Simulates a raw input event for testing purposes.
    /// This bypasses the Win32 message loop and directly processes the event.
    pub fn simulate_raw_input(&self, device_handle: usize, make_code: u16, flags: u16) {
        unsafe {
            let context_ptr = GetWindowLongPtrW(self.hwnd, GWLP_USERDATA) as *mut RawInputContext;
            if !context_ptr.is_null() {
                let context = &*context_ptr;
                let raw_keyboard = RAWKEYBOARD {
                    MakeCode: make_code,
                    Flags: flags,
                    Reserved: 0,
                    VKey: 0,
                    Message: 0,
                    ExtraInformation: 0,
                };
                window::process_raw_keyboard(&raw_keyboard, device_handle, context);
            }
        }
    }
}

impl Drop for RawInputManager {
    fn drop(&mut self) {
        unsafe {
            // WIN-BUG #1: Clear GWLP_USERDATA before destroying the window
            // to prevent wnd_proc from accessing the context during destruction.
            let ptr = SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0) as *mut RawInputContext;

            // WIN-BUG #8: Unregister Raw Input
            let rid = RAWINPUTDEVICE {
                usUsagePage: 1,
                usUsage: 6,
                dwFlags: windows_sys::Win32::UI::Input::RIDEV_REMOVE,
                hwndTarget: 0 as _,
            };
            let _ = RegisterRawInputDevices(&rid, 1, size_of::<RAWINPUTDEVICE>() as u32);

            DestroyWindow(self.hwnd);

            if !ptr.is_null() {
                let _ = Box::from_raw(ptr);
            }
        }

        // Uninstall bridge hook
        {
            match recover_lock_with_context(
                &self.bridge_context,
                "RawInputManager::drop bridge_context",
            ) {
                Ok(mut context_guard) => {
                    *context_guard = None;

                    // Clear thread-local storage
                    BRIDGE_CONTEXT_TLS.with(|tls| {
                        *tls.borrow_mut() = None;
                    });
                }
                Err(e) => {
                    log::error!(
                        "Failed to acquire bridge_context lock during cleanup: {}",
                        e
                    );
                }
            }

            match recover_lock_with_context(&self.bridge_hook, "RawInputManager::drop bridge_hook")
            {
                Ok(mut hook_guard) => {
                    if let Some(hook) = hook_guard.take() {
                        unsafe {
                            UnhookWindowsHookEx(hook as HHOOK);
                        }
                        log::info!("E2E Test Bridge Hook uninstalled");
                    }
                }
                Err(e) => {
                    log::error!("Failed to acquire bridge_hook lock during cleanup: {}", e);
                }
            }
        }
    }
}

struct RawInputContext {
    subscribers: Arc<RwLock<HashMap<usize, Sender<KeyEvent>>>>,
    device_map: DeviceMap,
    global_sender: Sender<KeyEvent>,
}
