#![allow(unsafe_code)]
use crate::engine::{InputEvent, KeyCode, OutputAction};
use crate::errors::KeyrxError;
use crate::traits::InputSource;
use anyhow::{Context, Result};
use crossbeam_channel::{Receiver, Sender};
use regex::Regex;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::sync::OnceLock;
use std::thread::{self, JoinHandle};
use tracing::{debug, error};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{GetLastError, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Input::{
    GetRawInputData, GetRawInputDeviceInfoW, RegisterRawInputDevices, HRAWINPUT, RAWINPUT,
    RAWINPUTDEVICE, RAWINPUTHEADER, RIDEV_INPUTSINK, RIDI_DEVICENAME, RID_INPUT, RIM_TYPEKEYBOARD,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, PostMessageW,
    PostThreadMessageW, RegisterClassExW, TranslateMessage, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT,
    MSG, WINDOW_EX_STYLE, WM_DESTROY, WM_INPUT, WM_QUIT, WNDCLASSEXW, WS_OVERLAPPEDWINDOW,
};

const CLASS_NAME: PCWSTR = w!("KeyRxRawInputClass");

pub struct WindowsRawInput {
    thread: Option<JoinHandle<()>>,
    rx: Receiver<InputEvent>,
    tx: Sender<InputEvent>,
    running: Arc<AtomicBool>,
    target_path: Option<String>,
    thread_id: Arc<AtomicU32>,
}

impl WindowsRawInput {
    pub fn new(target_path: Option<String>) -> Result<Self> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let running = Arc::new(AtomicBool::new(false));
        let thread_id = Arc::new(AtomicU32::new(0));

        Ok(Self {
            thread: None,
            rx,
            tx,
            running,
            target_path,
            thread_id,
        })
    }

    fn spawn_thread(&mut self) -> Result<()> {
        let tx = self.tx.clone();
        let running = self.running.clone();
        let thread_id_store = self.thread_id.clone();

        let handle = thread::Builder::new()
            .name("keyrx-raw-input".to_string())
            .spawn(move || {
                debug!("Starting Raw Input thread");
                // Store thread ID
                let tid = unsafe { GetCurrentThreadId() };
                thread_id_store.store(tid, Ordering::SeqCst);

                let h_instance = unsafe { GetModuleHandleW(None).unwrap_or_default() };

                unsafe {
                    let wnd_class = WNDCLASSEXW {
                        cbSize: size_of::<WNDCLASSEXW>() as u32,
                        style: CS_HREDRAW | CS_VREDRAW,
                        lpfnWndProc: Some(wnd_proc),
                        hInstance: HINSTANCE(h_instance.0),
                        lpszClassName: CLASS_NAME,
                        ..Default::default()
                    };

                    if RegisterClassExW(&wnd_class) == 0 {
                        // Ignore error, might be "Class already exists" (1410)
                        // If it failed for another reason, CreateWindowExW will fail.
                    }
                }

                let hwnd = unsafe {
                    CreateWindowExW(
                        WINDOW_EX_STYLE::default(),
                        CLASS_NAME,
                        w!("KeyRx Hidden Window"),
                        WS_OVERLAPPEDWINDOW,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        CW_USEDEFAULT,
                        None,
                        None,
                        h_instance,
                        None,
                    )
                };

                if hwnd.0 == 0 {
                    error!("Failed to create window: {:?}", unsafe { GetLastError() });
                    return;
                }

                // Register for Raw Input
                // Usage Page 1 (Generic Desktop), Usage 6 (Keyboard)
                let rid = RAWINPUTDEVICE {
                    usUsagePage: 1,
                    usUsage: 6,
                    dwFlags: RIDEV_INPUTSINK, // InputSink to get input in background
                    hwndTarget: hwnd,
                };

                unsafe {
                    if RegisterRawInputDevices(&[rid], size_of::<RAWINPUTDEVICE>() as u32).is_err()
                    {
                        error!("Failed to register raw input devices: {:?}", GetLastError());
                        let _ = DestroyWindow(hwnd);
                        return;
                    }
                }

                CONTEXT.with(|ctx| {
                    *ctx.borrow_mut() = Some(ThreadContext { tx });
                });

                let mut msg = MSG::default();
                while running.load(Ordering::Relaxed) {
                    let res = unsafe { GetMessageW(&mut msg, hwnd, 0, 0) };
                    if res.0 == 0 || res.0 == -1 {
                        break;
                    }

                    unsafe {
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }

                debug!("Raw Input thread exiting");
                unsafe {
                    let _ = DestroyWindow(hwnd);
                }
            })
            .context("Failed to spawn raw input thread")?;

        self.thread = Some(handle);
        Ok(())
    }
}

use std::cell::RefCell;

struct ThreadContext {
    tx: Sender<InputEvent>,
}

thread_local! {
    static CONTEXT: RefCell<Option<ThreadContext>> = const { RefCell::new(None) };
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_INPUT => {
            let mut size = 0u32;
            let h_raw_input = HRAWINPUT(lparam.0);
            // Get size
            if unsafe {
                GetRawInputData(
                    h_raw_input,
                    RID_INPUT,
                    None,
                    &mut size,
                    size_of::<RAWINPUTHEADER>() as u32,
                )
            } == 0
            {
                let mut buffer = vec![0u8; size as usize];
                if unsafe {
                    GetRawInputData(
                        h_raw_input,
                        RID_INPUT,
                        Some(buffer.as_mut_ptr() as _),
                        &mut size,
                        size_of::<RAWINPUTHEADER>() as u32,
                    )
                } == size
                {
                    let raw = unsafe { &*(buffer.as_ptr() as *const RAWINPUT) };
                    if raw.header.dwType == RIM_TYPEKEYBOARD.0 {
                        process_raw_input(raw);
                    }
                }
            }
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
        WM_DESTROY => {
            unsafe { PostMessageW(None, WM_QUIT, WPARAM(0), LPARAM(0)).unwrap_or_default() };
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn parse_vid_pid(path: &str) -> (Option<u16>, Option<u16>) {
    static RE: OnceLock<Regex> = OnceLock::new();

    if path.is_empty() {
        return (None, None);
    }

    let re = RE.get_or_init(|| {
        #[allow(clippy::unwrap_used)]
        Regex::new(r"VID_([0-9A-Fa-f]{4}).*PID_([0-9A-Fa-f]{4})").unwrap()
    });

    let captures = re.captures(path);

    let vendor_id = captures
        .as_ref()
        .and_then(|caps| caps.get(1))
        .and_then(|m| u16::from_str_radix(m.as_str(), 16).ok());
    let product_id = captures
        .as_ref()
        .and_then(|caps| caps.get(2))
        .and_then(|m| u16::from_str_radix(m.as_str(), 16).ok());

    (vendor_id, product_id)
}

fn process_raw_input(raw: &RAWINPUT) {
    let h_device = raw.header.hDevice;

    // Get Device Path
    let mut name_len = 0;
    let path = unsafe {
        GetRawInputDeviceInfoW(h_device, RIDI_DEVICENAME, None, &mut name_len);
        if name_len > 0 {
            let mut name_buffer = vec![0u16; name_len as usize];
            if GetRawInputDeviceInfoW(
                h_device,
                RIDI_DEVICENAME,
                Some(name_buffer.as_mut_ptr() as _),
                &mut name_len,
            ) > 0
            {
                String::from_utf16_lossy(&name_buffer)
                    .trim_matches(char::from(0))
                    .to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    };

    // Extract serial number from device path
    let serial_number = if !path.is_empty() {
        match crate::identity::windows::extract_serial_number(&path) {
            Ok(serial) => Some(serial),
            Err(e) => {
                // Log warning but continue without serial
                debug!(
                    "Failed to extract serial number from device path '{}': {}",
                    path, e
                );
                None
            }
        }
    } else {
        None
    };

    let keyboard = unsafe { raw.data.keyboard };
    let vkey = keyboard.VKey;
    let flags = keyboard.Flags;
    let scan_code = keyboard.MakeCode;

    let pressed = (flags & 1) == 0; // RI_KEY_BREAK = 1 (key up), 0 (key down)
                                    // E0 prefix
    let is_e0 = (flags & 2) != 0; // RI_KEY_E0 = 2

    use super::keymap::vk_to_keycode;

    // Handle extended keys
    let mut key_code = vk_to_keycode(vkey);
    if is_e0 && vkey == 0x0D {
        // VK_RETURN
        key_code = KeyCode::NumpadEnter;
    }

    let (vendor_id, product_id) = parse_vid_pid(&path);

    let event = InputEvent {
        key: key_code,
        pressed,
        timestamp_us: 0,
        device_id: if path.is_empty() { None } else { Some(path) },
        is_repeat: false,
        is_synthetic: false,
        scan_code,
        serial_number,
        vendor_id,
        product_id,
    };

    CONTEXT.with(|ctx| {
        if let Some(c) = ctx.borrow().as_ref() {
            let _ = c.tx.send(event);
        }
    });
}

#[async_trait::async_trait]
impl InputSource for WindowsRawInput {
    async fn start(&mut self) -> Result<(), KeyrxError> {
        if self.running.swap(true, Ordering::Relaxed) {
            return Ok(());
        }
        self.spawn_thread()
            .map_err(|e| KeyrxError::from(anyhow::anyhow!(e)))?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), KeyrxError> {
        if !self.running.swap(false, Ordering::Relaxed) {
            return Ok(());
        }
        let thread_id = self.thread_id.load(Ordering::SeqCst);
        if thread_id != 0 {
            unsafe {
                let _ = PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }

        if let Some(handle) = self.thread.take() {
            let _ = handle.join();
        }
        Ok(())
    }

    async fn poll_events(&mut self) -> Result<Vec<InputEvent>, KeyrxError> {
        #[cfg(feature = "otel-tracing")]
        let poll_span = tracing::trace_span!(
            "driver.poll_events",
            driver = "windows_raw",
            target = self.target_path.as_deref().unwrap_or("any"),
            running = self.running.load(Ordering::Relaxed)
        );
        #[cfg(feature = "otel-tracing")]
        let _poll_guard = poll_span.enter();

        let mut events = Vec::new();
        while let Ok(event) = self.rx.try_recv() {
            if let Some(target) = &self.target_path {
                if event.device_id.as_ref() != Some(target) {
                    continue;
                }
            }
            #[cfg(feature = "otel-tracing")]
            let event_span = tracing::trace_span!(
                "driver.input_event",
                driver = "windows_raw",
                key = ?event.key,
                pressed = event.pressed,
                device_id = event.device_id.as_deref().unwrap_or(""),
                scan_code = event.scan_code as u64,
            );
            #[cfg(feature = "otel-tracing")]
            let _event_guard = event_span.enter();
            events.push(event);
        }
        Ok(events)
    }

    async fn send_output(&mut self, _action: OutputAction) -> Result<(), KeyrxError> {
        Ok(())
    }
}
