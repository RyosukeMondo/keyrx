#![cfg(test)]

use super::*;
use crate::engine::KeyCode;
use crossbeam_channel::unbounded;
use std::sync::atomic::AtomicBool;

fn sample_event(code: u16, value: i32) -> evdev::InputEvent {
    evdev::InputEvent::new(evdev::EventType::KEY, code, value)
}

#[test]
fn build_input_event_sets_flags_and_metadata() {
    let key_down = build_input_event("dev0", &sample_event(30, 1));
    assert_eq!(key_down.key, KeyCode::A);
    assert!(key_down.pressed);
    assert!(!key_down.is_repeat);
    assert_eq!(key_down.device_id.as_deref(), Some("dev0"));

    let key_repeat = build_input_event("dev0", &sample_event(30, 2));
    assert!(key_repeat.pressed);
    assert!(key_repeat.is_repeat);

    let key_up = build_input_event("dev0", &sample_event(30, 0));
    assert!(!key_up.pressed);
    assert!(!key_up.is_repeat);
}

#[test]
fn process_events_internal_sends_all_events() {
    let (tx, rx) = unbounded();
    let events = vec![sample_event(1, 1), sample_event(1, 0)];

    let keep_running =
        process_events_internal(&tx, |event| build_input_event("dev1", event), &events);
    assert!(keep_running);

    let received: Vec<_> = rx.try_iter().collect();
    assert_eq!(received.len(), 2);
    assert_eq!(received[0].key, KeyCode::Escape);
    assert!(received[0].pressed);
    assert_eq!(received[1].key, KeyCode::Escape);
    assert!(!received[1].pressed);
}

#[test]
fn process_events_internal_stops_on_disconnected_channel() {
    let (tx, rx) = unbounded();
    drop(rx);

    let events = vec![sample_event(1, 1)];
    let keep_running =
        process_events_internal(&tx, |event| build_input_event("dev1", event), &events);
    assert!(!keep_running);
}

#[test]
fn handle_read_error_internal_respects_running_flag() {
    let running = Arc::new(AtomicBool::new(false));
    let path = PathBuf::from("/dev/input/event-test");
    let err = std::io::Error::new(std::io::ErrorKind::Other, "boom");

    // When not running, returns false without sleeping
    assert!(!handle_read_error_internal(&running, &path, &err));

    running.store(true, Ordering::Relaxed);
    assert!(handle_read_error_internal(&running, &path, &err));
}

#[test]
fn evdev_key_constants_correct() {
    // Verify evdev key constants match expected values
    assert_eq!(KEY_ESC, 1);
    assert_eq!(KEY_LEFTCTRL, 29);
    assert_eq!(KEY_RIGHTCTRL, 97);
    assert_eq!(KEY_LEFTSHIFT, 42);
    assert_eq!(KEY_RIGHTSHIFT, 54);
    assert_eq!(KEY_LEFTALT, 56);
    assert_eq!(KEY_RIGHTALT, 100);
}

#[test]
fn modifier_state_tracker_tracks_left_modifiers() {
    let mut tracker = ModifierStateTracker::default();

    // Initially no modifiers down
    assert!(!tracker.all_modifiers_down());

    // Press left ctrl
    tracker.update(KEY_LEFTCTRL, true);
    assert!(!tracker.all_modifiers_down());

    // Press left shift
    tracker.update(KEY_LEFTSHIFT, true);
    assert!(!tracker.all_modifiers_down());

    // Press left alt - now all are down
    tracker.update(KEY_LEFTALT, true);
    assert!(tracker.all_modifiers_down());

    // Release ctrl - no longer all down
    tracker.update(KEY_LEFTCTRL, false);
    assert!(!tracker.all_modifiers_down());
}

#[test]
fn modifier_state_tracker_tracks_right_modifiers() {
    let mut tracker = ModifierStateTracker::default();

    // Press right ctrl
    tracker.update(KEY_RIGHTCTRL, true);
    // Press right shift
    tracker.update(KEY_RIGHTSHIFT, true);
    // Press right alt
    tracker.update(KEY_RIGHTALT, true);

    // All modifiers down via right keys
    assert!(tracker.all_modifiers_down());
}

#[test]
fn modifier_state_tracker_mixed_left_right() {
    let mut tracker = ModifierStateTracker::default();

    // Mix of left and right modifiers
    tracker.update(KEY_LEFTCTRL, true);
    tracker.update(KEY_RIGHTSHIFT, true);
    tracker.update(KEY_LEFTALT, true);

    assert!(tracker.all_modifiers_down());

    // Note: Our tracker uses a single flag per modifier type (ctrl/shift/alt),
    // not separate flags for left/right. This is intentional for simplicity -
    // releasing ANY ctrl key clears the ctrl flag.
    tracker.update(KEY_RIGHTCTRL, false); // Releases the ctrl flag
    assert!(!tracker.all_modifiers_down());

    // Re-enable ctrl
    tracker.update(KEY_LEFTCTRL, true);
    assert!(tracker.all_modifiers_down());

    tracker.update(KEY_LEFTCTRL, false); // Left ctrl release
    assert!(!tracker.all_modifiers_down());
}

#[test]
fn modifier_state_tracker_ignores_other_keys() {
    let mut tracker = ModifierStateTracker::default();

    // Other keys don't affect modifier state
    tracker.update(KEY_ESC, true);
    tracker.update(30, true); // Key A
    tracker.update(57, true); // Space

    assert!(!tracker.all_modifiers_down());
}
