//! Message-only window creation and the Raw Input message pump.
//!
//! This is the low-level Win32 half of Raw Input handling: creating the
//! hidden message-only window `RawInputManager` registers for, subscribing
//! it to `WM_INPUT`/`WM_INPUT_DEVICE_CHANGE`, and turning a raw
//! `RAWKEYBOARD` payload into a `KeyEvent` dispatched to subscribers.

use std::ffi::c_void;
use std::mem::size_of;
use std::ptr;
use std::sync::Once;

use windows_sys::Win32::Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::{
    GetRawInputData, RegisterRawInputDevices, HRAWINPUT, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER,
    RAWKEYBOARD, RIDEV_DEVNOTIFY, RIDEV_INPUTSINK, RID_INPUT, RIM_TYPEKEYBOARD,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, GetWindowLongPtrW, RegisterClassExW, CS_DBLCLKS,
    GWLP_USERDATA, WM_INPUT, WM_INPUT_DEVICE_CHANGE, WNDCLASSEXW, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW,
};

use crate::platform::windows::keycode::scancode_to_keycode;
use crate::platform::PlatformError;
use keyrx_core::runtime::KeyEvent;

use super::RawInputContext;

static REGISTER_CLASS: Once = Once::new();
const CLASS_NAME: &[u16] = &[
    'K' as u16, 'e' as u16, 'y' as u16, 'R' as u16, 'x' as u16, 'R' as u16, 'a' as u16, 'w' as u16,
    'I' as u16, 'n' as u16, 'p' as u16, 'u' as u16, 't' as u16, 0,
];

pub(super) unsafe fn create_message_window() -> Result<HWND, PlatformError> {
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
        return Err(PlatformError::InitializationFailed {
            reason: format!(
                "Failed to create message window: {}",
                std::io::Error::last_os_error()
            ),
        });
    }

    Ok(hwnd)
}

pub(super) unsafe fn register_raw_input(hwnd: HWND) -> Result<(), PlatformError> {
    let rid = RAWINPUTDEVICE {
        usUsagePage: 1, // Generic Desktop Controls
        usUsage: 6,     // Keyboard
        dwFlags: RIDEV_INPUTSINK | RIDEV_DEVNOTIFY,
        hwndTarget: hwnd,
    };

    if RegisterRawInputDevices(&rid, 1, size_of::<RAWINPUTDEVICE>() as u32) == 0 {
        return Err(PlatformError::InitializationFailed {
            reason: format!(
                "RegisterRawInputDevices failed: {}",
                std::io::Error::last_os_error()
            ),
        });
    }

    Ok(())
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
            let _context = &*context_ptr;

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
                        // Keyboard events are now handled exclusively by the
                        // low-level hook (key_blocker). Raw Input cannot distinguish
                        // injected events (SendInput) from physical keypresses,
                        // which causes feedback loops when remapped output gets
                        // re-captured. The hook has LLKHF_INJECTED flag access
                        // and is the sole event source for the remapping engine.
                        //
                        // Raw Input is still used for device arrival/removal
                        // notifications (WM_INPUT_DEVICE_CHANGE).
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
                        log::warn!("Failed to query device info (handle={:x}): {} - This is normal for non-keyboard devices or devices that disconnect quickly", lparam, e);
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

pub(super) fn process_raw_keyboard(
    raw: &RAWKEYBOARD,
    device_handle: usize,
    context: &RawInputContext,
) {
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
