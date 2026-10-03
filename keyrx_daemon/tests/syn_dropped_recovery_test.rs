//! End-to-end: a burst of input that overflows the kernel's evdev buffer
//! (`SYN_DROPPED`) must not leave the daemon wedged.
//!
//! Observed in the field: after an unthrottled ~20k-event burst, 7 keys stayed
//! DOWN on the daemon's output, every later press was "(suppressed)" and
//! nothing was logged. The daemon has to resync its view of the keyboard
//! (what is really held), release stale output keys, and keep remapping.
//!
//! Requires access to /dev/uinput and /dev/input (the `input` group); skipped
//! otherwise.

#![cfg(target_os = "linux")]

use std::time::{Duration, Instant};

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEvent;
use keyrx_daemon::config::{ProfileManager, ProfileTemplate};
use keyrx_daemon::daemon::ConfigSource;
use tempfile::TempDir;

#[path = "common/live_daemon.rs"]
mod live_daemon;
use live_daemon::{tapped, Harness};

/// Keys cycled through in the flood: a mix of remapped (CapsLock) and
/// pass-through letters.
const FLOOD_KEYS: [KeyCode; 8] = [
    KeyCode::A,
    KeyCode::S,
    KeyCode::D,
    KeyCode::F,
    KeyCode::CapsLock,
    KeyCode::J,
    KeyCode::K,
    KeyCode::L,
];

/// Presses then releases every key in a tight loop, no delay: 2 events per
/// iteration, `iterations * 2` in total.
fn flood(h: &mut Harness, iterations: usize) {
    let mut events = Vec::with_capacity(iterations * 2);
    for i in 0..iterations {
        let key = FLOOD_KEYS[i % FLOOD_KEYS.len()];
        events.push(KeyEvent::press(key));
        events.push(KeyEvent::release(key));
    }
    h.keyboard.inject_sequence(&events, None).expect("flood");
}

/// Keys the daemon's output device currently reports as held.
fn held_on_output(h: &Harness) -> Vec<evdev::Key> {
    let state = h.capture.device().get_key_state().expect("EVIOCGKEY");
    state.iter().collect()
}

#[test]
fn daemon_recovers_after_an_input_buffer_overflow() {
    keyrx_daemon::skip_if_no_uinput!();
    let dir = TempDir::new().unwrap();
    let manager = ProfileManager::new(dir.path().to_path_buf()).unwrap();
    manager
        .create("a", ProfileTemplate::CapslockEscape)
        .unwrap();
    assert!(manager.activate("a").unwrap().success);

    let mut h = Harness::start("syndrop", ConfigSource::ActiveProfile, dir.path());
    assert_eq!(h.tap(KeyCode::CapsLock), tapped(KeyCode::Escape));

    // 20k events, far more than the kernel keeps per reader.
    flood(&mut h, 10_000);

    // Let the daemon work through whatever it kept: wait until its output
    // has been quiet for a while (a condition, not a fixed sleep - how long
    // the drain takes depends on host load), then look.
    h.reopen_capture();
    let started = Instant::now();
    let drained = h
        .capture
        .collect_events(Duration::from_millis(300))
        .expect("capture");
    assert!(
        started.elapsed() < Duration::from_secs(60),
        "the daemon was still emitting {} events after {:?}",
        drained.len(),
        started.elapsed()
    );

    assert_eq!(
        held_on_output(&h),
        Vec::<evdev::Key>::new(),
        "keys left DOWN on the daemon's output after the overflow"
    );
    assert_eq!(
        h.tap(KeyCode::CapsLock),
        tapped(KeyCode::Escape),
        "remapping stopped working after the overflow"
    );
    assert_eq!(h.tap(KeyCode::A), tapped(KeyCode::A));
    assert!(
        h.shared.input_overflow_count() > 0,
        "the overflow must be counted so it is visible in status"
    );
}
