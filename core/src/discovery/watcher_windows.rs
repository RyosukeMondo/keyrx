//! Windows implementation of the device hotplug watcher.
//!
//! Listens for `WM_DEVICECHANGE` broadcasts, keeps an in-memory snapshot of
//! connected keyboards, and emits `DeviceEvent` updates on changes.

use crate::discovery::{DeviceEvent, DeviceState, DeviceWatchError, DeviceWatcher, WatcherResult};
use crate::drivers::windows::list_keyboards;
use crate::drivers::DeviceInfo;
use crossbeam_channel::{unbounded, Sender};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::Instant;
use tracing::{debug, error, warn};
use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
    PostThreadMessageW, RegisterClassW, TranslateMessage, UnregisterClassW, CS_HREDRAW, CS_VREDRAW,
    CW_USEDEFAULT, DBT_DEVICEARRIVAL, DBT_DEVICEREMOVECOMPLETE, DBT_DEVNODES_CHANGED, HWND_MESSAGE,
    MSG, WM_DEVICECHANGE, WM_QUIT, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};

#[cfg(all(windows, feature = "windows-driver"))]
pub struct WindowsDeviceWatcher {
    devices: Arc<RwLock<HashMap<PathBuf, (DeviceInfo, DeviceState)>>>,
    subscribers: Arc<Mutex<Vec<Sender<DeviceEvent>>>>,
    running: Arc<AtomicBool>,
    thread_id_store: Arc<AtomicU32>,
    worker_handle: Mutex<Option<JoinHandle<()>>>,
}

#[cfg(all(windows, feature = "windows-driver"))]
impl Default for WindowsDeviceWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(windows, feature = "windows-driver"))]
impl WindowsDeviceWatcher {
    /// Create a new watcher with no registered devices.
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(AtomicBool::new(false)),
            thread_id_store: Arc::new(AtomicU32::new(0)),
            worker_handle: Mutex::new(None),
        }
    }

    fn seed_devices(&self) -> WatcherResult<()> {
        Self::sync_devices(&self.devices, &self.subscribers)
    }

    fn spawn_worker(&self) -> WatcherResult<()> {
        let devices = Arc::clone(&self.devices);
        let subscribers = Arc::clone(&self.subscribers);
        let running = Arc::clone(&self.running);
        let thread_id_store = Arc::clone(&self.thread_id_store);

        let handle = thread::Builder::new()
            .name("windows-device-watcher".to_string())
            .spawn(move || {
                if let Err(err) =
                    Self::message_loop(devices, subscribers, running.clone(), thread_id_store)
                {
                    error!(
                        service = "keyrx",
                        event = "windows_device_watcher_failed",
                        component = "windows_device_watcher",
                        error = %err,
                        "Windows device watcher exited with error"
                    );
                }
                running.store(false, Ordering::SeqCst);
            })
            .map_err(|err| DeviceWatchError::new(err.to_string(), false))?;

        let mut guard = self
            .worker_handle
            .lock()
            .map_err(|err| DeviceWatchError::new(err.to_string(), false))?;
        *guard = Some(handle);
        Ok(())
    }

    fn message_loop(
        devices: Arc<RwLock<HashMap<PathBuf, (DeviceInfo, DeviceState)>>>,
        subscribers: Arc<Mutex<Vec<Sender<DeviceEvent>>>>,
        running: Arc<AtomicBool>,
        thread_id_store: Arc<AtomicU32>,
    ) -> WatcherResult<()> {
        unsafe {
            thread_id_store.store(GetCurrentThreadId(), Ordering::SeqCst);

            let class_name = w!("KeyRxDeviceWatcher");
            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(Self::wnd_proc),
                hInstance: HINSTANCE::default(),
                lpszClassName: class_name,
                ..Default::default()
            };

            if RegisterClassW(&wnd_class) == 0 {
                return Err(DeviceWatchError::new(
                    "failed to register watcher window class",
                    false,
                ));
            }

            let hwnd = CreateWindowExW(
                Default::default(),
                class_name,
                class_name,
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                HWND_MESSAGE,
                None,
                HINSTANCE::default(),
                None,
            );

            if hwnd.0 == 0 {
                let _ = UnregisterClassW(class_name, HINSTANCE::default());
                return Err(DeviceWatchError::new(
                    "failed to create watcher window",
                    false,
                ));
            }

            let mut msg = MSG::default();
            while running.load(Ordering::SeqCst) {
                let status = GetMessageW(&mut msg, HWND(0), 0, 0);
                if status.0 == -1 {
                    let error = DeviceWatchError::new("GetMessageW failed", true);
                    Self::broadcast_static(
                        &subscribers,
                        DeviceEvent::Error {
                            device: None,
                            error: error.clone(),
                        },
                    );
                    return Err(error);
                }

                if status.0 == 0 {
                    break;
                }

                if msg.message == WM_DEVICECHANGE {
                    Self::handle_device_change(msg.wParam.0 as u32, &devices, &subscribers);
                }

                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            DestroyWindow(hwnd);
            let _ = UnregisterClassW(class_name, HINSTANCE::default());
            thread_id_store.store(0, Ordering::SeqCst);
        }

        Ok(())
    }

    fn handle_device_change(
        reason: u32,
        devices: &Arc<RwLock<HashMap<PathBuf, (DeviceInfo, DeviceState)>>>,
        subscribers: &Arc<Mutex<Vec<Sender<DeviceEvent>>>>,
    ) {
        match reason {
            DBT_DEVICEARRIVAL | DBT_DEVNODES_CHANGED => {
                if let Err(err) = Self::sync_devices(devices, subscribers) {
                    Self::broadcast_static(
                        subscribers,
                        DeviceEvent::Error {
                            device: None,
                            error: err,
                        },
                    );
                }
            }
            DBT_DEVICEREMOVECOMPLETE => {
                let disconnected = Self::mark_disconnected(devices);
                for info in disconnected {
                    Self::broadcast_static(subscribers, DeviceEvent::Disconnected(info));
                }
            }
            _ => {}
        }
    }

    fn mark_disconnected(
        devices: &Arc<RwLock<HashMap<PathBuf, (DeviceInfo, DeviceState)>>>,
    ) -> Vec<DeviceInfo> {
        let mut lost = Vec::new();
        let mut guard = match devices.write() {
            Ok(lock) => lock,
            Err(err) => {
                warn!(
                    service = "keyrx",
                    event = "device_map_poisoned",
                    component = "windows_device_watcher",
                    error = %err,
                    "Unable to mark disconnected devices"
                );
                return lost;
            }
        };

        for (_path, (info, state)) in guard.iter_mut() {
            if !matches!(state, DeviceState::Paused { .. }) {
                *state = DeviceState::Paused {
                    since: Instant::now(),
                };
                lost.push(info.clone());
            }
        }

        lost
    }

    fn sync_devices(
        devices: &Arc<RwLock<HashMap<PathBuf, (DeviceInfo, DeviceState)>>>,
        subscribers: &Arc<Mutex<Vec<Sender<DeviceEvent>>>>,
    ) -> WatcherResult<()> {
        let current =
            list_keyboards().map_err(|err| DeviceWatchError::new(err.to_string(), false))?;
        let mut guard = devices
            .write()
            .map_err(|err| DeviceWatchError::new(format!("device map poisoned: {err}"), false))?;

        let mut active_paths = HashSet::new();
        for device in current {
            active_paths.insert(device.path.clone());
            match guard.get_mut(&device.path) {
                Some((info, state)) => {
                    *info = device.clone();
                    if !matches!(state, DeviceState::Active) {
                        *state = DeviceState::Active;
                        Self::broadcast_static(subscribers, DeviceEvent::Connected(device.clone()));
                    }
                }
                None => {
                    guard.insert(device.path.clone(), (device.clone(), DeviceState::Active));
                    Self::broadcast_static(subscribers, DeviceEvent::Connected(device));
                }
            }
        }

        let mut disconnected = Vec::new();
        for (path, (info, state)) in guard.iter_mut() {
            if !active_paths.contains(path) && !matches!(state, DeviceState::Paused { .. }) {
                *state = DeviceState::Paused {
                    since: Instant::now(),
                };
                disconnected.push(info.clone());
            }
        }

        drop(guard);

        for info in disconnected {
            Self::broadcast_static(subscribers, DeviceEvent::Disconnected(info));
        }

        Ok(())
    }

    fn broadcast_static(subscribers: &Arc<Mutex<Vec<Sender<DeviceEvent>>>>, event: DeviceEvent) {
        let mut closed = Vec::new();
        if let Ok(mut subs) = subscribers.lock() {
            for (idx, sender) in subs.iter().enumerate() {
                if sender.send(event.clone()).is_err() {
                    closed.push(idx);
                }
            }

            for idx in closed.into_iter().rev() {
                subs.remove(idx);
            }
        } else {
            warn!(
                service = "keyrx",
                event = "subscriber_lock_poisoned",
                component = "windows_device_watcher",
                "Unable to broadcast device events due to poisoned lock"
            );
        }
    }

    fn broadcast(&self, event: DeviceEvent) {
        Self::broadcast_static(&self.subscribers, event);
    }

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

#[cfg(all(windows, feature = "windows-driver"))]
impl DeviceWatcher for WindowsDeviceWatcher {
    fn start(&self) -> WatcherResult<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let start_result = (|| {
            self.seed_devices()?;
            self.spawn_worker()?;
            Ok(())
        })();

        if let Err(err) = start_result {
            self.running.store(false, Ordering::SeqCst);
            return Err(err);
        }

        Ok(())
    }

    fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        let thread_id = self.thread_id_store.load(Ordering::SeqCst);
        if thread_id != 0 {
            let result = unsafe { PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0)) };
            if let Err(err) = result {
                warn!(
                    service = "keyrx",
                    event = "post_quit_failed",
                    component = "windows_device_watcher",
                    thread_id = thread_id,
                    error = %err,
                    "Failed to post WM_QUIT to watcher thread"
                );
            }
        }

        if let Ok(mut handle_guard) = self.worker_handle.lock() {
            if let Some(handle) = handle_guard.take() {
                let _ = handle.join();
            }
        }
    }

    fn subscribe(&self) -> crate::discovery::DeviceEventReceiver {
        let (tx, rx) = unbounded();
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.push(tx);
        } else {
            warn!(
                service = "keyrx",
                event = "subscriber_lock_poisoned",
                component = "windows_device_watcher",
                "subscribe failed to record receiver; events may be dropped"
            );
        }
        rx
    }

    fn snapshot(&self) -> Vec<(DeviceInfo, DeviceState)> {
        match self.devices.read() {
            Ok(devices) => devices.values().cloned().collect(),
            Err(_) => Vec::new(),
        }
    }
}
