//! Linux implementation of the device hotplug watcher.
//!
//! Uses inotify (via the `notify` crate) to observe `/dev/input` for keyboard
//! device additions/removals and broadcasts `DeviceEvent` updates.

use crate::discovery::{DeviceEvent, DeviceState, DeviceWatchError, DeviceWatcher, WatcherResult};
use crate::drivers::linux::list_keyboards;
use crate::drivers::DeviceInfo;
use crossbeam_channel::{unbounded, Receiver as CrossbeamReceiver, RecvTimeoutError, Sender};
use notify::event::{ModifyKind, RenameMode};
use notify::{
    recommended_watcher, Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};

#[cfg(all(target_os = "linux", feature = "linux-driver"))]
const DEFAULT_INPUT_ROOT: &str = "/dev/input";

/// Linux device watcher backed by inotify.
#[cfg(all(target_os = "linux", feature = "linux-driver"))]
pub struct LinuxDeviceWatcher {
    root: PathBuf,
    devices: Arc<RwLock<HashMap<PathBuf, (DeviceInfo, DeviceState)>>>,
    subscribers: Arc<Mutex<Vec<Sender<DeviceEvent>>>>,
    running: Arc<AtomicBool>,
    worker_handle: Mutex<Option<JoinHandle<()>>>,
    watcher: Mutex<Option<RecommendedWatcher>>,
}

#[cfg(all(target_os = "linux", feature = "linux-driver"))]
impl Default for LinuxDeviceWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(target_os = "linux", feature = "linux-driver"))]
impl LinuxDeviceWatcher {
    /// Create a watcher rooted at `/dev/input`.
    pub fn new() -> Self {
        Self::with_root(PathBuf::from(DEFAULT_INPUT_ROOT))
    }

    /// Create a watcher rooted at a specific directory (useful for tests).
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            devices: Arc::new(RwLock::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(AtomicBool::new(false)),
            worker_handle: Mutex::new(None),
            watcher: Mutex::new(None),
        }
    }

    fn seed_devices(&self) -> WatcherResult<()> {
        let devices =
            list_keyboards().map_err(|err| DeviceWatchError::new(err.to_string(), false))?;

        let mut map_guard = match self.devices.write() {
            Ok(guard) => guard,
            Err(err) => {
                return Err(DeviceWatchError::new(
                    format!("device map poisoned: {err}"),
                    false,
                ))
            }
        };

        for device in devices {
            let path = device.path.clone();
            let inserted = map_guard
                .insert(path.clone(), (device.clone(), DeviceState::Active))
                .is_none();

            if inserted {
                self.broadcast(DeviceEvent::Connected(device));
            }
        }

        Ok(())
    }

    fn spawn_worker(
        &self,
        notify_rx: CrossbeamReceiver<notify::Result<Event>>,
        root: PathBuf,
    ) -> WatcherResult<()> {
        let devices = Arc::clone(&self.devices);
        let subscribers = Arc::clone(&self.subscribers);
        let running = Arc::clone(&self.running);

        let handle = thread::Builder::new()
            .name("linux-device-watcher".to_string())
            .spawn(move || {
                Self::event_loop(notify_rx, root, devices, subscribers, running);
            })
            .map_err(|err| DeviceWatchError::new(err.to_string(), false))?;

        let mut guard = self
            .worker_handle
            .lock()
            .map_err(|err| DeviceWatchError::new(err.to_string(), false))?;
        *guard = Some(handle);
        Ok(())
    }

    fn event_loop(
        notify_rx: CrossbeamReceiver<notify::Result<Event>>,
        root: PathBuf,
        devices: Arc<RwLock<HashMap<PathBuf, (DeviceInfo, DeviceState)>>>,
        subscribers: Arc<Mutex<Vec<Sender<DeviceEvent>>>>,
        running: Arc<AtomicBool>,
    ) {
        while running.load(Ordering::SeqCst) {
            match notify_rx.recv_timeout(Duration::from_secs(1)) {
                Ok(Ok(event)) => {
                    Self::handle_event(&root, event, &devices, &subscribers);
                }
                Ok(Err(err)) => {
                    let error = DeviceWatchError::new(err.to_string(), true);
                    Self::broadcast_static(
                        &subscribers,
                        DeviceEvent::Error {
                            device: None,
                            error,
                        },
                    );
                }
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    }

    fn handle_event(
        root: &Path,
        event: Event,
        devices: &Arc<RwLock<HashMap<PathBuf, (DeviceInfo, DeviceState)>>>,
        subscribers: &Arc<Mutex<Vec<Sender<DeviceEvent>>>>,
    ) {
        let Event { kind, paths, .. } = event;

        if matches!(kind, EventKind::Modify(ModifyKind::Name(RenameMode::Both))) && paths.len() == 2
        {
            let from = &paths[0];
            let to = &paths[1];

            if from.starts_with(root) {
                Self::handle_remove(from, devices, subscribers);
            }
            if to.starts_with(root) {
                Self::handle_add(to, devices, subscribers);
            }
            return;
        }

        for path in paths {
            if !path.starts_with(root) {
                continue;
            }

            match kind {
                EventKind::Create(_) => {
                    Self::handle_add(&path, devices, subscribers);
                }
                EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
                    Self::handle_add(&path, devices, subscribers);
                }
                EventKind::Remove(_) => {
                    Self::handle_remove(&path, devices, subscribers);
                }
                EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
                    Self::handle_remove(&path, devices, subscribers);
                }
                _ => {}
            }
        }
    }

    fn handle_add(
        path: &Path,
        devices: &Arc<RwLock<HashMap<PathBuf, (DeviceInfo, DeviceState)>>>,
        subscribers: &Arc<Mutex<Vec<Sender<DeviceEvent>>>>,
    ) {
        let info = match Self::find_device(path) {
            Ok(Some(info)) => info,
            Ok(None) => return,
            Err(err) => {
                Self::broadcast_static(
                    subscribers,
                    DeviceEvent::Error {
                        device: None,
                        error: err,
                    },
                );
                return;
            }
        };

        let mut guard = match devices.write() {
            Ok(lock) => lock,
            Err(err) => {
                let error = DeviceWatchError::new(
                    format!("device map poisoned while adding: {err}"),
                    false,
                );
                Self::broadcast_static(
                    subscribers,
                    DeviceEvent::Error {
                        device: Some(info),
                        error,
                    },
                );
                return;
            }
        };

        let previous_state = guard.get(&info.path).map(|(_, state)| state.clone());

        guard.insert(info.path.clone(), (info.clone(), DeviceState::Active));

        let should_emit = match previous_state {
            None => true,
            Some(DeviceState::Active) => false,
            Some(DeviceState::Paused { .. }) | Some(DeviceState::Failed { .. }) => true,
        };

        if should_emit {
            Self::broadcast_static(subscribers, DeviceEvent::Connected(info));
        }
    }

    fn handle_remove(
        path: &Path,
        devices: &Arc<RwLock<HashMap<PathBuf, (DeviceInfo, DeviceState)>>>,
        subscribers: &Arc<Mutex<Vec<Sender<DeviceEvent>>>>,
    ) {
        let mut guard = match devices.write() {
            Ok(lock) => lock,
            Err(err) => {
                let error = DeviceWatchError::new(
                    format!("device map poisoned while removing: {err}"),
                    false,
                );
                Self::broadcast_static(
                    subscribers,
                    DeviceEvent::Error {
                        device: None,
                        error,
                    },
                );
                return;
            }
        };

        if let Some((info, state)) = guard.get_mut(path) {
            *state = DeviceState::Paused {
                since: Instant::now(),
            };
            Self::broadcast_static(subscribers, DeviceEvent::Disconnected(info.clone()));
        } else {
            // If we don't know the device, attempt a best-effort snapshot to locate it.
            debug!(
                service = "keyrx",
                event = "unknown_device_removed",
                component = "linux_device_watcher",
                path = %path.display(),
                "remove event for untracked device"
            );
        }
    }

    fn find_device(path: &Path) -> WatcherResult<Option<DeviceInfo>> {
        let devices =
            list_keyboards().map_err(|err| DeviceWatchError::new(err.to_string(), false))?;
        Ok(devices.into_iter().find(|dev| dev.path == path))
    }

    fn broadcast(&self, event: DeviceEvent) {
        Self::broadcast_static(&self.subscribers, event);
    }

    fn broadcast_static(subscribers: &Arc<Mutex<Vec<Sender<DeviceEvent>>>>, event: DeviceEvent) {
        let mut closed = Vec::new();
        if let Ok(mut subs) = subscribers.lock() {
            for (idx, sender) in subs.iter().enumerate() {
                if sender.send(event.clone()).is_err() {
                    closed.push(idx);
                }
            }

            // Remove closed subscribers in reverse order to keep indices valid.
            for idx in closed.into_iter().rev() {
                subs.remove(idx);
            }
        } else {
            warn!(
                service = "keyrx",
                event = "subscriber_lock_poisoned",
                component = "linux_device_watcher",
                "Unable to broadcast device events due to poisoned lock"
            );
        }
    }
}

#[cfg(all(target_os = "linux", feature = "linux-driver"))]
impl DeviceWatcher for LinuxDeviceWatcher {
    fn start(&self) -> WatcherResult<()> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let start_result = (|| {
            let (notify_tx, notify_rx) = unbounded();
            let mut watcher = recommended_watcher(move |res| {
                if notify_tx.send(res).is_err() {
                    error!(
                        service = "keyrx",
                        event = "watcher_channel_closed",
                        component = "linux_device_watcher",
                        "notify callback failed to send event"
                    );
                }
            })
            .map_err(|err| DeviceWatchError::new(err.to_string(), false))?;

            watcher
                .configure(Config::default().with_poll_interval(Duration::from_secs(1)))
                .map_err(|err| DeviceWatchError::new(err.to_string(), false))?;

            watcher
                .watch(&self.root, RecursiveMode::NonRecursive)
                .map_err(|err| DeviceWatchError::new(err.to_string(), false))?;

            self.seed_devices()?;
            self.spawn_worker(notify_rx, self.root.clone())?;

            let mut guard = self
                .watcher
                .lock()
                .map_err(|err| DeviceWatchError::new(err.to_string(), false))?;
            *guard = Some(watcher);

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

        if let Ok(mut guard) = self.watcher.lock() {
            *guard = None;
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
                component = "linux_device_watcher",
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
