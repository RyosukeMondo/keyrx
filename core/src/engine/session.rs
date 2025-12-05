//! Session controller for hotplug-aware pause/resume.
//!
//! This module wires device hotplug signals into the shared [`SessionState`]
//! so callers can pause processing when the active keyboard disappears and
//! resume seamlessly when it returns.

use crate::discovery::DeviceEvent;
use crate::drivers::DeviceInfo;
use crate::engine::{SessionState, SessionStatus};
use std::time::{Duration, Instant};

/// Result of handling a device hotplug event.
#[derive(Debug, Clone, PartialEq)]
pub enum HotplugAction {
    /// No change to the session state.
    NoChange,
    /// Session paused because the primary device was removed or failed.
    Paused { device: DeviceInfo },
    /// Session resumed after the primary device returned.
    Resumed {
        device: DeviceInfo,
        paused_for: Duration,
    },
}

/// Hotplug-aware session controller.
///
/// Tracks a primary device fingerprint and maps connect/disconnect events to
/// pause/resume transitions while keeping the underlying [`SessionState`]
/// timestamps intact.
#[derive(Debug, Default)]
pub struct HotplugSession {
    state: SessionState,
    primary_device: Option<DeviceInfo>,
    paused_since: Option<Instant>,
}

impl HotplugSession {
    /// Create a new idle session with no tracked device.
    pub fn new() -> Self {
        Self::default()
    }

    /// Start the session and register the primary device to monitor.
    pub fn start(&mut self, device: DeviceInfo) {
        self.primary_device = Some(device);
        self.state.start();
        self.paused_since = None;
    }

    /// Update the tracked primary device without changing session status.
    pub fn set_primary_device(&mut self, device: DeviceInfo) {
        self.primary_device = Some(device);
    }

    /// Current session status.
    pub fn status(&self) -> SessionStatus {
        self.state.status()
    }

    /// Timestamp when the session was paused, if ever.
    pub fn paused_since(&self) -> Option<Instant> {
        self.paused_since
    }

    /// Start time recorded by the underlying session state.
    pub fn start_time(&self) -> Option<Instant> {
        self.state.start_time()
    }

    /// Handle a device hotplug event and apply pause/resume transitions.
    ///
    /// Returns a [`HotplugAction`] that describes what happened so callers can
    /// react (e.g., stopping event processing while paused).
    pub fn handle_hotplug(&mut self, event: &DeviceEvent) -> HotplugAction {
        match event {
            DeviceEvent::Disconnected(device) => self.pause_for_device(device),
            DeviceEvent::Connected(device) => self.resume_for_device(device),
            DeviceEvent::Error {
                device: Some(device),
                ..
            } => self.pause_for_device(device),
            _ => HotplugAction::NoChange,
        }
    }

    fn pause_for_device(&mut self, device: &DeviceInfo) -> HotplugAction {
        if !self.state.is_active() || !self.matches_primary(device) {
            return HotplugAction::NoChange;
        }

        self.state.pause();
        if self.paused_since.is_none() {
            self.paused_since = Some(Instant::now());
        }

        HotplugAction::Paused {
            device: device.clone(),
        }
    }

    fn resume_for_device(&mut self, device: &DeviceInfo) -> HotplugAction {
        if !self.state.is_paused() || !self.matches_primary(device) {
            return HotplugAction::NoChange;
        }

        self.state.resume();
        let paused_for = self.paused_since.map(|t| t.elapsed()).unwrap_or_default();
        self.paused_since = None;

        HotplugAction::Resumed {
            device: device.clone(),
            paused_for,
        }
    }

    fn matches_primary(&self, device: &DeviceInfo) -> bool {
        self.primary_device
            .as_ref()
            .map(|primary| device_matches(primary, device))
            .unwrap_or(false)
    }
}

fn device_matches(primary: &DeviceInfo, other: &DeviceInfo) -> bool {
    let path_match = primary.path == other.path;
    let vid_pid_match = primary.vendor_id != 0
        && other.vendor_id != 0
        && primary.vendor_id == other.vendor_id
        && primary.product_id == other.product_id;
    let name_match = vid_pid_match && primary.name == other.name;

    path_match || vid_pid_match || name_match
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    fn device(path: &str, name: &str, vendor: u16, product: u16) -> DeviceInfo {
        DeviceInfo::new(PathBuf::from(path), name.to_string(), vendor, product, true)
    }

    #[test]
    fn pauses_on_primary_disconnect_and_preserves_start_time() {
        let mut session = HotplugSession::new();
        let keyboard = device("/dev/input/event0", "kbd", 0x1, 0x2);

        session.start(keyboard.clone());
        let started_at = session.start_time();

        let action = session.handle_hotplug(&DeviceEvent::Disconnected(keyboard.clone()));

        assert_eq!(
            action,
            HotplugAction::Paused {
                device: keyboard.clone()
            }
        );
        assert_eq!(session.status(), SessionStatus::Paused);
        assert!(session.paused_since().is_some());
        assert_eq!(session.start_time(), started_at);
    }

    #[test]
    fn resumes_on_reconnect_matching_identity() {
        let mut session = HotplugSession::new();
        let keyboard = device("/dev/input/event0", "kbd", 0x1234, 0xabcd);

        session.start(keyboard.clone());
        session.handle_hotplug(&DeviceEvent::Disconnected(keyboard.clone()));
        thread::sleep(Duration::from_millis(1));

        // Path changed but identity matches (VID/PID)
        let reconnected = device("/dev/input/event5", "kbd", 0x1234, 0xabcd);
        let action = session.handle_hotplug(&DeviceEvent::Connected(reconnected.clone()));

        match action {
            HotplugAction::Resumed { paused_for, .. } => assert!(paused_for > Duration::ZERO),
            other => panic!("expected resumed action, got {other:?}"),
        }
        assert_eq!(session.status(), SessionStatus::Active);
        assert!(session.paused_since().is_none());
    }

    #[test]
    fn ignores_unrelated_devices() {
        let mut session = HotplugSession::new();
        let keyboard = device("/dev/input/event0", "kbd", 0x1, 0x2);
        let mouse = device("/dev/input/event1", "mouse", 0x3, 0x4);

        session.start(keyboard);

        let action = session.handle_hotplug(&DeviceEvent::Disconnected(mouse));

        assert_eq!(action, HotplugAction::NoChange);
        assert_eq!(session.status(), SessionStatus::Active);
        assert!(session.paused_since().is_none());
    }
}
