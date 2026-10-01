//! Output keys the daemon is holding down, so a config swap can release them.
//!
//! If a profile switch lands while a remapped key is down, the key's release
//! is processed by the NEW mapping and may release a different output, leaving
//! the old output stuck on the virtual keyboard (e.g. CapsLock→Esc switched to
//! CapsLock→LCtrl mid-press: Esc never comes up). [`HeldOutputs`] wraps any
//! [`Platform`], records injected presses/releases, and releases whatever is
//! still down when asked. One decorator, so every platform gets it.

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEvent;

use super::{DeviceInfo, Platform, PlatformResult};

/// A [`Platform`] that remembers which injected output keys are still pressed.
pub struct HeldOutputs {
    inner: Box<dyn Platform>,
    /// Pressed output keys, in press order.
    held: Vec<KeyCode>,
}

impl HeldOutputs {
    pub fn new(inner: Box<dyn Platform>) -> Self {
        Self {
            inner,
            held: Vec::new(),
        }
    }

    fn record(&mut self, event: &KeyEvent) {
        let key = event.keycode();
        if event.is_press() {
            if !self.held.contains(&key) {
                self.held.push(key);
            }
        } else {
            self.held.retain(|&k| k != key);
        }
    }
}

impl Platform for HeldOutputs {
    fn initialize(&mut self) -> PlatformResult<()> {
        self.inner.initialize()
    }

    fn capture_input(&mut self) -> PlatformResult<KeyEvent> {
        self.inner.capture_input()
    }

    fn inject_output(&mut self, event: KeyEvent) -> PlatformResult<()> {
        self.inner.inject_output(event.clone())?;
        self.record(&event);
        Ok(())
    }

    fn list_devices(&self) -> PlatformResult<Vec<DeviceInfo>> {
        self.inner.list_devices()
    }

    fn shutdown(&mut self) -> PlatformResult<()> {
        self.inner.shutdown()
    }

    fn query_ime_state(&self) -> Option<keyrx_core::config::ImeState> {
        self.inner.query_ime_state()
    }

    fn release_held_outputs(&mut self) -> PlatformResult<usize> {
        let held = std::mem::take(&mut self.held);
        // Most recent first, like a user letting go.
        for &key in held.iter().rev() {
            self.inner.inject_output(KeyEvent::release(key))?;
        }
        Ok(held.len())
    }

    fn reconfigure_devices(
        &mut self,
        configs: &[keyrx_core::config::DeviceConfig],
    ) -> PlatformResult<()> {
        self.inner.reconfigure_devices(configs)
    }

    fn take_devices_changed(&mut self) -> bool {
        self.inner.take_devices_changed()
    }

    fn take_input_overflows(&mut self) -> u64 {
        self.inner.take_input_overflows()
    }

    fn set_input_wait_limit(&mut self, limit: Option<std::time::Duration>) {
        self.inner.set_input_wait_limit(limit);
    }

    fn output_device(&self) -> Option<crate::platform::OutputDeviceInfo> {
        self.inner.output_device()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Records injected events; everything else is inert.
    struct Recorder(Arc<Mutex<Vec<KeyEvent>>>);

    impl Platform for Recorder {
        fn initialize(&mut self) -> PlatformResult<()> {
            Ok(())
        }
        fn capture_input(&mut self) -> PlatformResult<KeyEvent> {
            Err(crate::platform::PlatformError::NoInput)
        }
        fn inject_output(&mut self, event: KeyEvent) -> PlatformResult<()> {
            self.0.lock().unwrap().push(event);
            Ok(())
        }
        fn list_devices(&self) -> PlatformResult<Vec<DeviceInfo>> {
            Ok(Vec::new())
        }
        fn shutdown(&mut self) -> PlatformResult<()> {
            Ok(())
        }
    }

    fn platform() -> (HeldOutputs, Arc<Mutex<Vec<KeyEvent>>>) {
        let log = Arc::new(Mutex::new(Vec::new()));
        (HeldOutputs::new(Box::new(Recorder(Arc::clone(&log)))), log)
    }

    #[test]
    fn releases_only_keys_still_down_most_recent_first() {
        let (mut p, log) = platform();
        p.inject_output(KeyEvent::press(KeyCode::Escape)).unwrap();
        p.inject_output(KeyEvent::press(KeyCode::LShift)).unwrap();
        p.inject_output(KeyEvent::press(KeyCode::A)).unwrap();
        p.inject_output(KeyEvent::release(KeyCode::A)).unwrap();
        log.lock().unwrap().clear();

        assert_eq!(p.release_held_outputs().unwrap(), 2);
        let released: Vec<_> = log
            .lock()
            .unwrap()
            .iter()
            .map(|e| (e.keycode(), e.is_release()))
            .collect();
        assert_eq!(
            released,
            vec![(KeyCode::LShift, true), (KeyCode::Escape, true)]
        );
        // Nothing left to release.
        assert_eq!(p.release_held_outputs().unwrap(), 0);
    }

    #[test]
    fn repeated_press_is_one_held_key() {
        let (mut p, _log) = platform();
        p.inject_output(KeyEvent::press(KeyCode::B)).unwrap();
        p.inject_output(KeyEvent::press(KeyCode::B)).unwrap(); // autorepeat
        assert_eq!(p.release_held_outputs().unwrap(), 1);
    }

    /// A decorator that overrides only some `Platform` methods must still
    /// forward every other one to `inner` - the bug this guards against:
    /// `HeldOutputs` implementing `Platform` by hand means a new trait
    /// method with a default silently falls back to that default (here,
    /// "never changed") instead of the inner platform's real answer,
    /// unless someone remembers to add a delegating line here too. Found
    /// via H8's hotplug device-count republishing going through
    /// `HeldOutputs` and always reporting no change.
    struct ChangedFlagPlatform(std::sync::atomic::AtomicBool);

    impl Platform for ChangedFlagPlatform {
        fn initialize(&mut self) -> PlatformResult<()> {
            Ok(())
        }
        fn capture_input(&mut self) -> PlatformResult<KeyEvent> {
            Err(crate::platform::PlatformError::NoInput)
        }
        fn inject_output(&mut self, _event: KeyEvent) -> PlatformResult<()> {
            Ok(())
        }
        fn list_devices(&self) -> PlatformResult<Vec<DeviceInfo>> {
            Ok(Vec::new())
        }
        fn shutdown(&mut self) -> PlatformResult<()> {
            Ok(())
        }
        fn take_devices_changed(&mut self) -> bool {
            self.0.swap(false, std::sync::atomic::Ordering::SeqCst)
        }
    }

    #[test]
    fn take_devices_changed_delegates_to_inner_platform() {
        let inner = ChangedFlagPlatform(std::sync::atomic::AtomicBool::new(true));
        let mut p = HeldOutputs::new(Box::new(inner));

        assert!(
            p.take_devices_changed(),
            "must report the inner platform's real change, not the trait default (false)"
        );
        // The inner flag was consumed; a second call must not re-report it.
        assert!(!p.take_devices_changed());
    }
}
