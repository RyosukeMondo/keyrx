//! Multi-device coordination for hotplug handling.
//!
//! Tracks multiple devices independently so that a failure on one keyboard
//! does not halt others. Emits coordination actions to help callers react to
//! partial failure or full outage scenarios.

use crate::discovery::{DeviceEvent, DeviceState, DeviceWatchError};
use crate::drivers::DeviceInfo;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::time::Instant;

/// Action emitted after handling a device event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoordinationAction {
    /// No state change occurred.
    NoChange,
    /// A new device was added to tracking.
    Added { device: DeviceInfo },
    /// A known device disconnected and is now paused.
    Disconnected { device: DeviceInfo },
    /// A device entered a failed state; `all_failed` signals total outage.
    Failed {
        device: DeviceInfo,
        all_failed: bool,
    },
    /// A previously failed or paused device became active again.
    Recovered { device: DeviceInfo },
}

/// Coordinate multiple devices and track their independent health states.
#[derive(Debug, Default)]
pub struct MultiDeviceCoordinator {
    devices: HashMap<String, (DeviceInfo, DeviceState)>,
}

impl MultiDeviceCoordinator {
    /// Create an empty coordinator.
    pub fn new() -> Self {
        Self {
            devices: HashMap::new(),
        }
    }

    /// Handle a device hotplug event and return the resulting coordination action.
    pub fn handle_event(&mut self, event: &DeviceEvent) -> CoordinationAction {
        match event {
            DeviceEvent::Connected(device) => self.mark_active(device),
            DeviceEvent::Disconnected(device) => self.mark_disconnected(device),
            DeviceEvent::Error {
                device: Some(device),
                error,
            } => self.mark_failed(device, error.clone()),
            DeviceEvent::Error { device: None, .. } => CoordinationAction::NoChange,
        }
    }

    /// Devices that are not in a failed state (active or paused).
    pub fn operational_devices(&self) -> Vec<DeviceInfo> {
        self.devices
            .values()
            .filter_map(|(info, state)| match state {
                DeviceState::Failed { .. } => None,
                _ => Some(info.clone()),
            })
            .collect()
    }

    /// Devices currently marked as failed.
    pub fn failed_devices(&self) -> Vec<DeviceInfo> {
        self.devices
            .values()
            .filter_map(|(info, state)| match state {
                DeviceState::Failed { .. } => Some(info.clone()),
                _ => None,
            })
            .collect()
    }

    /// Snapshot of all tracked devices and their states.
    pub fn tracked_devices(&self) -> Vec<(DeviceInfo, DeviceState)> {
        self.devices.values().cloned().collect()
    }

    /// True when every tracked device is failed (partial failure escalated to outage).
    pub fn is_all_failed(&self) -> bool {
        !self.devices.is_empty()
            && self
                .devices
                .values()
                .all(|(_, state)| matches!(state, DeviceState::Failed { .. }))
    }

    fn mark_active(&mut self, device: &DeviceInfo) -> CoordinationAction {
        let key = device_key(device);
        match self.devices.entry(key) {
            Entry::Vacant(vacant) => {
                vacant.insert((device.clone(), DeviceState::Active));
                CoordinationAction::Added {
                    device: device.clone(),
                }
            }
            Entry::Occupied(mut occupied) => {
                let (info, state) = occupied.get_mut();
                *info = device.clone();
                let was_failed = matches!(state, DeviceState::Failed { .. });
                let was_paused = matches!(state, DeviceState::Paused { .. });
                *state = DeviceState::Active;

                if was_failed || was_paused {
                    CoordinationAction::Recovered {
                        device: device.clone(),
                    }
                } else {
                    CoordinationAction::NoChange
                }
            }
        }
    }

    fn mark_disconnected(&mut self, device: &DeviceInfo) -> CoordinationAction {
        let key = device_key(device);
        if let Some((info, state)) = self.devices.get_mut(&key) {
            *info = device.clone();
            *state = DeviceState::Paused {
                since: Instant::now(),
            };
            return CoordinationAction::Disconnected {
                device: device.clone(),
            };
        }

        CoordinationAction::NoChange
    }

    fn mark_failed(&mut self, device: &DeviceInfo, error: DeviceWatchError) -> CoordinationAction {
        let key = device_key(device);
        let (info, state) = self
            .devices
            .entry(key)
            .or_insert_with(|| (device.clone(), DeviceState::Active));

        *info = device.clone();
        *state = DeviceState::Failed { error };

        CoordinationAction::Failed {
            device: info.clone(),
            all_failed: self.is_all_failed(),
        }
    }
}

fn device_key(device: &DeviceInfo) -> String {
    if device.vendor_id != 0 && device.product_id != 0 {
        format!(
            "{:04x}:{:04x}:{}",
            device.vendor_id, device.product_id, device.name
        )
    } else {
        device.path.to_string_lossy().into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::DeviceWatchError;
    use std::path::PathBuf;

    fn device(path: &str, name: &str, vendor: u16, product: u16) -> DeviceInfo {
        DeviceInfo::new(PathBuf::from(path), name.to_string(), vendor, product, true)
    }

    #[test]
    fn single_failure_does_not_affect_others() {
        let mut coordinator = MultiDeviceCoordinator::new();
        let dev_a = device("/dev/input/event0", "kbd-a", 0x1, 0x2);
        let dev_b = device("/dev/input/event1", "kbd-b", 0x3, 0x4);

        coordinator.handle_event(&DeviceEvent::Connected(dev_a.clone()));
        coordinator.handle_event(&DeviceEvent::Connected(dev_b.clone()));

        let action = coordinator.handle_event(&DeviceEvent::Error {
            device: Some(dev_a.clone()),
            error: DeviceWatchError::new("io failure", true),
        });

        assert_eq!(
            action,
            CoordinationAction::Failed {
                device: dev_a.clone(),
                all_failed: false,
            }
        );
        assert!(coordinator.failed_devices().contains(&dev_a));
        assert!(coordinator.operational_devices().contains(&dev_b));
        assert!(!coordinator.is_all_failed());
    }

    #[test]
    fn signals_when_all_devices_have_failed() {
        let mut coordinator = MultiDeviceCoordinator::new();
        let dev_a = device("/dev/input/event0", "kbd-a", 0x1, 0x2);
        let dev_b = device("/dev/input/event1", "kbd-b", 0x3, 0x4);

        coordinator.handle_event(&DeviceEvent::Connected(dev_a.clone()));
        coordinator.handle_event(&DeviceEvent::Connected(dev_b.clone()));

        coordinator.handle_event(&DeviceEvent::Error {
            device: Some(dev_a.clone()),
            error: DeviceWatchError::new("io failure", true),
        });
        let action = coordinator.handle_event(&DeviceEvent::Error {
            device: Some(dev_b.clone()),
            error: DeviceWatchError::new("fatal", false),
        });

        assert_eq!(
            action,
            CoordinationAction::Failed {
                device: dev_b.clone(),
                all_failed: true,
            }
        );
        assert!(coordinator.is_all_failed());
        let failed = coordinator.failed_devices();
        assert!(failed.contains(&dev_a));
        assert!(failed.contains(&dev_b));
    }

    #[test]
    fn reconnect_recovers_failed_device_and_updates_path() {
        let mut coordinator = MultiDeviceCoordinator::new();
        let dev = device("/dev/input/event0", "kbd-a", 0x1234, 0xabcd);
        let reconnected = device("/dev/input/event5", "kbd-a", 0x1234, 0xabcd);

        coordinator.handle_event(&DeviceEvent::Connected(dev.clone()));
        coordinator.handle_event(&DeviceEvent::Error {
            device: Some(dev.clone()),
            error: DeviceWatchError::new("transient", true),
        });

        let action = coordinator.handle_event(&DeviceEvent::Connected(reconnected.clone()));
        assert_eq!(
            action,
            CoordinationAction::Recovered {
                device: reconnected.clone()
            }
        );

        let tracked = coordinator.tracked_devices();
        assert_eq!(tracked.len(), 1);
        assert_eq!(tracked[0].0, reconnected);
        assert!(matches!(tracked[0].1, DeviceState::Active));
        assert!(!coordinator.is_all_failed());
    }

    #[test]
    fn disconnect_marks_paused_without_triggering_failure() {
        let mut coordinator = MultiDeviceCoordinator::new();
        let dev = device("/dev/input/event0", "kbd-a", 0x1, 0x2);

        coordinator.handle_event(&DeviceEvent::Connected(dev.clone()));
        let action = coordinator.handle_event(&DeviceEvent::Disconnected(dev.clone()));

        assert_eq!(
            action,
            CoordinationAction::Disconnected {
                device: dev.clone()
            }
        );
        let tracked = coordinator.tracked_devices();
        assert_eq!(tracked.len(), 1);
        assert!(matches!(tracked[0].1, DeviceState::Paused { .. }));
        assert!(!coordinator.is_all_failed());
    }
}
