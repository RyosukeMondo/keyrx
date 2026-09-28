//! inotify-backed watch on `/dev/input`, so the event loop notices a
//! keyboard being plugged in or unplugged without a restart.
//!
//! [`HotplugWatcher::drain`] is only called from the event loop's idle path
//! (no input event was ready), so hotplug adds no latency to the hot path of
//! actually typing.

use std::path::Path;

use nix::sys::inotify::{AddWatchFlags, InitFlags, Inotify};

/// Watches `/dev/input` for device nodes appearing or disappearing.
pub struct HotplugWatcher {
    inotify: Inotify,
}

impl HotplugWatcher {
    /// Starts watching `/dev/input`. Returns `None` (after logging why) if
    /// inotify is unavailable - hotplug is then simply off, startup is
    /// otherwise unaffected: a plugged-in keyboard needs a daemon restart.
    pub fn start() -> Option<Self> {
        Self::start_in(Path::new("/dev/input"))
    }

    fn start_in(dir: &Path) -> Option<Self> {
        let inotify = Inotify::init(InitFlags::IN_NONBLOCK)
            .map_err(|e| log::warn!("Hotplug disabled: could not start inotify: {e}"))
            .ok()?;
        let flags = AddWatchFlags::IN_CREATE
            | AddWatchFlags::IN_DELETE
            | AddWatchFlags::IN_ATTRIB
            | AddWatchFlags::IN_MOVED_TO
            | AddWatchFlags::IN_MOVED_FROM;
        inotify
            .add_watch(dir, flags)
            .map_err(|e| log::warn!("Hotplug disabled: could not watch {}: {e}", dir.display()))
            .ok()?;
        log::info!("Hotplug: watching {} for keyboard changes", dir.display());
        Some(Self { inotify })
    }

    /// Drains pending events; returns whether an `eventN` node changed
    /// (other `/dev/input` entries, e.g. `js0`, are ignored).
    pub fn drain(&self) -> bool {
        match self.inotify.read_events() {
            Ok(events) => events.iter().any(is_event_node_change),
            Err(nix::errno::Errno::EAGAIN) => false,
            Err(e) => {
                log::warn!("Hotplug: failed to read inotify events: {e}");
                false
            }
        }
    }
}

fn is_event_node_change(event: &nix::sys::inotify::InotifyEvent) -> bool {
    event
        .name
        .as_deref()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.starts_with("event"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// `/dev/input` always exists on Linux; the real integration is covered
    /// by the in-process hotplug e2e test.
    #[test]
    fn start_succeeds_or_logs_and_returns_none() {
        // Just exercises the code path; either outcome is acceptable in a
        // sandboxed CI container.
        let _ = HotplugWatcher::start();
    }

    #[test]
    fn watching_a_missing_directory_returns_none() {
        assert!(HotplugWatcher::start_in(Path::new("/nonexistent-keyrx-test-dir")).is_none());
    }

    #[test]
    fn detects_event_node_creation() {
        let dir = tempfile::tempdir().unwrap();
        let watcher = HotplugWatcher::start_in(dir.path()).expect("watch temp dir");
        fs::write(dir.path().join("event99"), b"").unwrap();
        // Give the kernel a moment to deliver the event.
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(watcher.drain());
    }

    #[test]
    fn ignores_non_event_node_changes() {
        let dir = tempfile::tempdir().unwrap();
        let watcher = HotplugWatcher::start_in(dir.path()).expect("watch temp dir");
        fs::write(dir.path().join("mice"), b"").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(50));
        assert!(!watcher.drain());
    }
}
