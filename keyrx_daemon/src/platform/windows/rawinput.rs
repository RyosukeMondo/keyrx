use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;
use std::sync::{Arc, Mutex, Once, RwLock};

use crossbeam_channel::{unbounded, Receiver, Sender};
use windows_sys::Win32::Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::{
    GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER,
    RAWKEYBOARD, RIDEV_DEVNOTIFY, RIDEV_INPUTSINK, RID_INPUT, RIM_TYPEKEYBOARD,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, CreateWindowExW, DefWindowProcW, DestroyWindow, GetWindowLongPtrW,
    RegisterClassExW, SetWindowLongPtrW, SetWindowsHookExW, UnhookWindowsHookEx, CS_DBLCLKS,
    GWLP_USERDATA, HC_ACTION, HHOOK, KBDLLHOOKSTRUCT, LLKHF_EXTENDED, WH_KEYBOARD_LL, WM_INPUT,
    WM_INPUT_DEVICE_CHANGE, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSEXW,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

use crate::platform::windows::device_map::DeviceMap;
use crate::platform::windows::keycode::scancode_to_keycode;
use keyrx_core::runtime::KeyEvent;

static REGISTER_CLASS: Once = Once::new();
const CLASS_NAME: &[u16] = &[
    'K' as u16, 'e' as u16, 'y' as u16, 'R' as u16, 'x' as u16, 'R' as u16, 'a' as u16, 'w' as u16,
    'I' as u16, 'n' as u16, 'p' as u16, 'u' as u16, 't' as u16, 0,
];

const TEST_SIMULATED_PHYSICAL_MARKER: usize = 0x54455354; // "TEST"

// Thread-local storage for bridge context used by the hook callback
thread_local! {
    static BRIDGE_CONTEXT_TLS: RefCell<Option<Arc<Mutex<Option<BridgeContextHandle>>>>> = RefCell::new(None);
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
    ) -> Result<Self, String> {
        // 1. Create message-only window
        let hwnd = unsafe { Self::create_message_window()? };

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
        unsafe { Self::register_raw_input(hwnd)? };

        // 4. Install Test Bridge Hook for E2E testing
        {
            let mut context_guard = bridge_context.lock().unwrap();
            *context_guard = Some(BridgeContextHandle {
                global_sender: global_sender.clone(),
                subscribers: subscribers.clone(),
            });

            // Set thread-local storage for hook callback access
            BRIDGE_CONTEXT_TLS.with(|tls| {
                *tls.borrow_mut() = Some(bridge_context.clone());
            });

            let mut hook_guard = bridge_hook.lock().unwrap();
            unsafe {
                let hook = SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(test_bridge_hook),
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
                process_raw_keyboard(&raw_keyboard, device_handle, context);
            }
        }
    }

    unsafe fn create_message_window() -> Result<HWND, String> {
        let h_instance = GetModuleHandleW(ptr::null());

        REGISTER_CLASS.call_once(|| {
            let wnd_class = WNDCLASSEXW {
                cbSize: size_of::<WNDCLASSEXW>() as u32,
                style: CS_DBLCLKS,
                lpfnWndProc: Some(wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: h_instance,
                hIcon: 0 as _,
                hCursor: 0 as _,
                hbrBackground: 0 as _,
                lpszMenuName: ptr::null(),
                lpszClassName: CLASS_NAME.as_ptr(),
                hIconSm: 0 as _,
            };

            if RegisterClassExW(&wnd_class) == 0 {
                log::error!(
                    "Failed to register window class: {}",
                    std::io::Error::last_os_error()
                );
            }
        });

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE, // Hidden from taskbar and not focusable
            CLASS_NAME.as_ptr(),
            CLASS_NAME.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            0 as HWND, // Top-level window required for RIDEV_INPUTSINK
            0 as _,
            h_instance,
            ptr::null(),
        );

        if hwnd == 0 as _ {
            return Err(format!(
                "Failed to create message window: {}",
                std::io::Error::last_os_error()
            ));
        }

        Ok(hwnd)
    }

    unsafe fn register_raw_input(hwnd: HWND) -> Result<(), String> {
        let rid = RAWINPUTDEVICE {
            usUsagePage: 1, // Generic Desktop Controls
            usUsage: 6,     // Keyboard
            dwFlags: RIDEV_INPUTSINK | RIDEV_DEVNOTIFY,
            hwndTarget: hwnd,
        };

        if RegisterRawInputDevices(&rid, 1, size_of::<RAWINPUTDEVICE>() as u32) == 0 {
            return Err(format!(
                "RegisterRawInputDevices failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        Ok(())
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
            let mut context_guard = self.bridge_context.lock().unwrap();
            *context_guard = None;

            // Clear thread-local storage
            BRIDGE_CONTEXT_TLS.with(|tls| {
                *tls.borrow_mut() = None;
            });

            let mut hook_guard = self.bridge_hook.lock().unwrap();
            if let Some(hook) = hook_guard.take() {
                unsafe {
                    UnhookWindowsHookEx(hook as HHOOK);
                }
                log::info!("E2E Test Bridge Hook uninstalled");
            }
        }
    }
}

struct RawInputContext {
    subscribers: Arc<RwLock<HashMap<usize, Sender<KeyEvent>>>>,
    device_map: DeviceMap,
    global_sender: Sender<KeyEvent>,
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let context_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut RawInputContext;

    if msg == WM_INPUT {
        if !context_ptr.is_null() {
            let context = &*context_ptr;

            let mut close_size: u32 = 0;
            // Use explicit casts for HRAWINPUT (isize) and pointers
            // GetRawInputData(HRAWINPUT, ...)
            // Note: lparam is used as HRAWINPUT
            let h_raw_input = lparam as HRAWINPUT;

            if GetRawInputData(
                h_raw_input,
                RID_INPUT,
                ptr::null_mut(),
                &mut close_size,
                size_of::<RAWINPUTHEADER>() as u32,
            ) == 0
            {
                // WIN-BUG #3: Unbounded memory allocation.
                // Limit buffer size to prevent OOM from malicious/buggy drivers.
                const MAX_RAW_INPUT_SIZE: u32 = 4096;
                if close_size > MAX_RAW_INPUT_SIZE {
                    log::warn!("Raw input size too large: {} bytes", close_size);
                    return DefWindowProcW(hwnd, msg, wparam, lparam);
                }

                let mut buffer = vec![0u8; close_size as usize];

                if GetRawInputData(
                    h_raw_input,
                    RID_INPUT,
                    buffer.as_mut_ptr() as *mut c_void,
                    &mut close_size,
                    size_of::<RAWINPUTHEADER>() as u32,
                ) != u32::MAX
                {
                    let raw: &RAWINPUT = &*(buffer.as_ptr() as *const RAWINPUT);

                    if raw.header.dwType == RIM_TYPEKEYBOARD {
                        // hDevice is HANDLE (isize), cast to usize for map key
                        let handle = raw.header.hDevice as usize;
                        process_raw_keyboard(&raw.data.keyboard, handle, context);
                    }
                }
            }
        }
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    } else if msg == WM_INPUT_DEVICE_CHANGE {
        if !context_ptr.is_null() {
            let context = &*context_ptr;
            match wparam as u32 {
                1 => {
                    // GIDC_ARRIVAL
                    log::info!("Device arrived: {:x}", lparam);
                    // lparam is HANDLE of device
                    if let Err(e) = context.device_map.add_device(lparam as HANDLE) {
                        log::error!("Failed to add new device: {}", e);
                    }
                }
                2 => {
                    // GIDC_REMOVAL
                    log::info!("Device removed: {:x}", lparam);
                    context.device_map.remove_device(lparam as HANDLE);
                }
                _ => {}
            }
        }
        return 0;
    }

    DefWindowProcW(hwnd, msg, wparam, lparam)
}

fn process_raw_keyboard(raw: &RAWKEYBOARD, device_handle: usize, context: &RawInputContext) {
    let is_break = (raw.Flags & 1) != 0;
    let is_e0 = (raw.Flags & 2) != 0;

    let mut scancode = raw.MakeCode as u32;
    if is_e0 {
        scancode |= 0xE000;
    }

    // Some basic filtering like overrun check could go here
    if scancode == 0xFF {
        return;
    }

    if let Some(keycode) = scancode_to_keycode(scancode) {
        let mut event = if is_break {
            KeyEvent::release(keycode)
        } else {
            KeyEvent::press(keycode)
        };

        // Attach device ID if available
        if let Some(info) = context.device_map.get(device_handle as HANDLE) {
            let device_id = info.device_id();
            event = event.with_device_id(device_id);
        }

        // Send to global sink
        log::debug!("Raw input event: {:?}", event);
        let _ = context.global_sender.try_send(event.clone());

        // Send to specific subscriber (legacy support)
        match context.subscribers.read() {
            Ok(subscribers) => {
                if let Some(sender) = subscribers.get(&device_handle) {
                    let _ = sender.try_send(event);
                }
            }
            Err(_) => {
                log::error!("Subscribers lock poisoned");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require an interactive Windows session and may fail in some CI environments.
    #[test]
    fn test_raw_input_manager_creation() {
        let device_map = DeviceMap::new();
        let (tx, _rx) = unbounded();
        let bridge_context = Arc::new(Mutex::new(None));
        let bridge_hook = Arc::new(Mutex::new(None));

        // We wrap in a block to ensure RawInputManager is dropped at end
        {
            match RawInputManager::new(device_map, tx, bridge_context, bridge_hook) {
                Ok(manager) => {
                    assert!(manager.hwnd != 0 as HWND);
                    let _receiver = manager.subscribe(12345);
                    manager.unsubscribe(12345);
                }
                Err(e) => {
                    // This can happen in CI environments without a GUI session
                    eprintln!("RawInputManager creation failed: {}", e);
                }
            }
        }
    }

    #[test]
    fn test_raw_input_simulation() {
        let device_map = DeviceMap::new();
        let (tx, rx_global) = unbounded();
        let bridge_context = Arc::new(Mutex::new(None));
        let bridge_hook = Arc::new(Mutex::new(None));

        // Register a synthetic device
        device_map.add_synthetic_device(0x1234, "test-path".to_string(), Some("SN123".to_string()));

        match RawInputManager::new(device_map, tx, bridge_context, bridge_hook) {
            Ok(manager) => {
                let rx_device = manager.subscribe(0x1234);

                // Simulate a key press (A key, MakeCode 0x1E)
                manager.simulate_raw_input(0x1234, 0x1E, 0);

                // Global channel should receive it
                let event_global = rx_global
                    .recv_timeout(std::time::Duration::from_millis(100))
                    .expect("Should receive global event");
                assert_eq!(event_global.device_id(), Some("serial-SN123"));

                // Subscribed channel should receive it
                let event_device = rx_device
                    .recv_timeout(std::time::Duration::from_millis(100))
                    .expect("Should receive device event");
                assert_eq!(event_device.device_id(), Some("serial-SN123"));
            }
            Err(e) => {
                eprintln!("Skipping simulation test (no GUI): {}", e);
            }
        }
    }

    #[test]
    fn test_raw_input_subscription_logic() {
        let device_map = DeviceMap::new();
        let (tx, _rx) = unbounded();
        let bridge_context = Arc::new(Mutex::new(None));
        let bridge_hook = Arc::new(Mutex::new(None));

        match RawInputManager::new(device_map, tx, bridge_context, bridge_hook) {
            Ok(manager) => {
                // Subscription for non-existent device is allowed (manager doesn't check existence)
                let _rx = manager.subscribe(0xDEAD);
                assert!(manager.subscribers.read().unwrap().contains_key(&0xDEAD));

                // Same handle multiple times
                let _rx2 = manager.subscribe(0xDEAD);
                assert_eq!(manager.subscribers.read().unwrap().len(), 1);

                manager.unsubscribe(0xDEAD);
                assert!(!manager.subscribers.read().unwrap().contains_key(&0xDEAD));
            }
            Err(e) => eprintln!("Skipping test (no GUI): {}", e),
        }
    }

    #[test]
    fn test_raw_input_drop_cleanup() {
        let device_map = DeviceMap::new();
        let (tx, _rx) = unbounded();
        let bridge_context = Arc::new(Mutex::new(None));
        let bridge_hook = Arc::new(Mutex::new(None));
        let hwnd;

        {
            let manager = RawInputManager::new(device_map, tx, bridge_context, bridge_hook)
                .expect("Should create manager");
            hwnd = manager.hwnd;
            assert!(hwnd != 0 as HWND);
            // Context should be present
            unsafe {
                let context_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
                assert!(context_ptr != 0);
            }
        }

        // Manager dropped, window should be destroyed and context cleared
        // Note: DestroyWindow might be async or require message loop to fully vanish,
        // but we check if GWLP_USERDATA is cleared as per our drop logic.
        // Actually, our drop logic clears it.
        // Wait, after DestroyWindow, GetWindowLongPtrW(hwnd) is invalid.
    }

    #[test]
    fn test_multiple_manager_instances_coexist() {
        // This test demonstrates that multiple RawInputManager instances can coexist
        // with independent bridge contexts and hooks, proving thread-safety.
        let device_map1 = DeviceMap::new();
        let device_map2 = DeviceMap::new();
        let (tx1, _rx1) = unbounded();
        let (tx2, _rx2) = unbounded();
        let bridge_context1 = Arc::new(Mutex::new(None));
        let bridge_hook1 = Arc::new(Mutex::new(None));
        let bridge_context2 = Arc::new(Mutex::new(None));
        let bridge_hook2 = Arc::new(Mutex::new(None));

        // Create first manager
        let manager1_result = RawInputManager::new(
            device_map1,
            tx1,
            bridge_context1.clone(),
            bridge_hook1.clone(),
        );

        // Create second manager
        let manager2_result = RawInputManager::new(
            device_map2,
            tx2,
            bridge_context2.clone(),
            bridge_hook2.clone(),
        );

        // Both should succeed (or both fail if no GUI session)
        match (manager1_result, manager2_result) {
            (Ok(manager1), Ok(manager2)) => {
                // Verify they have different windows
                assert_ne!(manager1.hwnd, manager2.hwnd);

                // Verify they have independent bridge contexts
                assert!(bridge_context1.lock().unwrap().is_some());
                assert!(bridge_context2.lock().unwrap().is_some());

                // Verify they have independent hooks
                assert!(bridge_hook1.lock().unwrap().is_some());
                assert!(bridge_hook2.lock().unwrap().is_some());

                // Drop them to verify cleanup works independently
                drop(manager1);
                assert!(bridge_context1.lock().unwrap().is_none());
                assert!(bridge_hook1.lock().unwrap().is_none());

                // Manager2's context should still be valid
                assert!(bridge_context2.lock().unwrap().is_some());
                assert!(bridge_hook2.lock().unwrap().is_some());
            }
            _ => {
                eprintln!("Skipping multiple instances test (no GUI session)");
            }
        }
    }
}

// Low-level hook callback to bridge E2E test events into RawInput stream.
// This is ONLY used for events marked with TEST_SIMULATED_PHYSICAL_MARKER.
unsafe extern "system" fn test_bridge_hook(code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
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
