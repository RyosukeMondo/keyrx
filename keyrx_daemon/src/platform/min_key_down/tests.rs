//! Fake-clock tests for [`MinKeyDown`].

use super::*;
use std::sync::{Arc, Mutex};

type Log = Arc<Mutex<Vec<(KeyCode, bool)>>>;

/// Records injected events as (key, is_press).
struct Recorder(Log);

impl Platform for Recorder {
    fn initialize(&mut self) -> PlatformResult<()> {
        Ok(())
    }
    fn capture_input(&mut self) -> PlatformResult<KeyEvent> {
        Err(super::super::PlatformError::NoInput)
    }
    fn inject_output(&mut self, event: KeyEvent) -> PlatformResult<()> {
        self.0
            .lock()
            .unwrap()
            .push((event.keycode(), event.is_press()));
        Ok(())
    }
    fn list_devices(&self) -> PlatformResult<Vec<DeviceInfo>> {
        Ok(Vec::new())
    }
    fn shutdown(&mut self) -> PlatformResult<()> {
        Ok(())
    }
}

const MIN: Duration = Duration::from_millis(5);

fn rig() -> (MinKeyDown, Log, Instant) {
    let log: Log = Arc::default();
    let p = MinKeyDown::new(Box::new(Recorder(Arc::clone(&log))), MIN);
    (p, log, Instant::now())
}

fn ms(base: Instant, n: u64) -> Instant {
    base + Duration::from_millis(n)
}

fn got(log: &Log) -> Vec<(KeyCode, bool)> {
    log.lock().unwrap().clone()
}

fn press(p: &mut MinKeyDown, k: KeyCode, t: Instant) {
    p.inject_at(KeyEvent::press(k), t).unwrap();
}
fn release(p: &mut MinKeyDown, k: KeyCode, t: Instant) {
    p.inject_at(KeyEvent::release(k), t).unwrap();
}

#[test]
fn a_pulse_shorter_than_the_minimum_is_stretched() {
    let (mut p, log, t) = rig();
    press(&mut p, KeyCode::A, t);
    release(&mut p, KeyCode::A, ms(t, 0));
    assert_eq!(got(&log), vec![(KeyCode::A, true)], "release is held back");
    p.flush_at(ms(t, 4)).unwrap();
    assert_eq!(got(&log).len(), 1, "still inside the window");
    p.flush_at(ms(t, 5)).unwrap();
    assert_eq!(got(&log), vec![(KeyCode::A, true), (KeyCode::A, false)]);
}

#[test]
fn a_key_held_longer_than_the_minimum_is_not_delayed() {
    let (mut p, log, t) = rig();
    press(&mut p, KeyCode::A, t);
    release(&mut p, KeyCode::A, ms(t, 50));
    assert_eq!(got(&log), vec![(KeyCode::A, true), (KeyCode::A, false)]);
}

#[test]
fn a_release_with_no_known_press_is_never_delayed() {
    let (mut p, log, t) = rig();
    release(&mut p, KeyCode::B, t);
    assert_eq!(got(&log), vec![(KeyCode::B, false)]);
}

#[test]
fn a_different_key_goes_down_at_once_while_a_tapped_key_waits() {
    let (mut p, log, t) = rig();
    for key in [KeyCode::A, KeyCode::B, KeyCode::C] {
        press(&mut p, key, t);
        release(&mut p, key, t); // each deferred to t+5
    }
    assert_eq!(
        got(&log),
        vec![(KeyCode::A, true), (KeyCode::B, true), (KeyCode::C, true)],
        "presses overlap instead of costing a minimum each"
    );
    p.flush_at(ms(t, 5)).unwrap();
    assert_eq!(
        got(&log)[3..],
        [
            (KeyCode::A, false),
            (KeyCode::B, false),
            (KeyCode::C, false)
        ],
        "releases leave in arrival order"
    );
}

#[test]
fn nothing_overtakes_a_deferred_modifier_release() {
    let (mut p, log, t) = rig();
    press(&mut p, KeyCode::LShift, t);
    release(&mut p, KeyCode::LShift, t); // deferred to t+5
    press(&mut p, KeyCode::A, ms(t, 1)); // an unshifted 'a': must stay unshifted
    assert_eq!(got(&log), vec![(KeyCode::LShift, true)]);
    p.flush_at(ms(t, 5)).unwrap();
    assert_eq!(
        got(&log),
        vec![
            (KeyCode::LShift, true),
            (KeyCode::LShift, false),
            (KeyCode::A, true)
        ]
    );
}

#[test]
fn a_press_never_overtakes_its_own_keys_release() {
    let (mut p, log, t) = rig();
    press(&mut p, KeyCode::A, t);
    release(&mut p, KeyCode::A, t);
    press(&mut p, KeyCode::B, t); // overtakes A's release
    press(&mut p, KeyCode::A, t); // must wait for A's release
    press(&mut p, KeyCode::C, t); // presses never reorder: waits behind A
    assert_eq!(got(&log), vec![(KeyCode::A, true), (KeyCode::B, true)]);
    p.flush_at(ms(t, 5)).unwrap();
    assert_eq!(
        got(&log)[2..],
        [(KeyCode::A, false), (KeyCode::A, true), (KeyCode::C, true)]
    );
}

#[test]
fn nothing_else_is_delayed_when_nothing_is_pending() {
    let (mut p, log, t) = rig();
    press(&mut p, KeyCode::A, t);
    press(&mut p, KeyCode::B, ms(t, 1));
    assert_eq!(got(&log).len(), 2, "presses pass straight through");
}

#[test]
fn a_late_flush_still_gives_the_key_a_full_minimum() {
    let (mut p, log, t) = rig();
    press(&mut p, KeyCode::A, t);
    release(&mut p, KeyCode::A, t); // A up due t+5
    press(&mut p, KeyCode::LCtrl, t); // a modifier waits behind A up
    release(&mut p, KeyCode::LCtrl, t); // due 5 ms after Ctrl really went down
    p.flush_at(ms(t, 8)).unwrap(); // loop was late: A up + Ctrl down go out at t+8
    assert_eq!(got(&log).len(), 3);
    p.flush_at(ms(t, 10)).unwrap();
    assert_eq!(got(&log).len(), 3, "Ctrl up must wait until t+13");
    p.flush_at(ms(t, 13)).unwrap();
    assert_eq!(got(&log).len(), 4);
}

#[test]
fn release_held_outputs_and_shutdown_flush_everything_at_once() {
    let (mut p, log, t) = rig();
    press(&mut p, KeyCode::A, t);
    release(&mut p, KeyCode::A, t);
    press(&mut p, KeyCode::LShift, t);
    p.release_held_outputs().unwrap();
    assert_eq!(
        got(&log),
        vec![
            (KeyCode::A, true),
            (KeyCode::A, false),
            (KeyCode::LShift, true)
        ]
    );
    release(&mut p, KeyCode::LShift, t);
    p.shutdown().unwrap();
    assert_eq!(got(&log).len(), 4);
    assert!(p.queue.is_empty());
}

#[test]
fn repeated_taps_of_one_key_each_get_a_full_minimum() {
    let (mut p, log, t) = rig();
    for _ in 0..2 {
        press(&mut p, KeyCode::A, t);
        release(&mut p, KeyCode::A, t);
    }
    p.flush_at(ms(t, 5)).unwrap();
    p.flush_at(ms(t, 10)).unwrap();
    let events = got(&log);
    assert_eq!(events.iter().filter(|e| e.1).count(), 2);
    assert_eq!(events.iter().filter(|e| !e.1).count(), 2);
    for pair in events.chunks(2) {
        assert_eq!(pair[0], (KeyCode::A, true));
    }
}

/// Regression: a release used to take its deadline from its key's NEXT
/// queued press once any other press flushed, so under a burst deadlines
/// ran away and the output froze with a key held (the SYN_DROPPED live
/// test caught it as a key stuck DOWN).
#[test]
fn a_queued_release_waits_for_its_own_press_not_the_next_one() {
    let (mut p, log, t) = rig();
    press(&mut p, KeyCode::LShift, t);
    release(&mut p, KeyCode::LShift, t); // due t+5; all below queue behind it
    press(&mut p, KeyCode::K, t);
    press(&mut p, KeyCode::X, t);
    release(&mut p, KeyCode::K, t);
    release(&mut p, KeyCode::X, t);
    press(&mut p, KeyCode::K, t); // K's NEXT press, still queued
    release(&mut p, KeyCode::K, t);
    p.flush_at(ms(t, 6)).unwrap(); // Shift up, K down, X down at t+6
    assert_eq!(got(&log).len(), 4);
    p.flush_at(ms(t, 11)).unwrap();
    assert_eq!(
        got(&log)[4..],
        [(KeyCode::K, false), (KeyCode::X, false), (KeyCode::K, true)],
        "K up is due 5 ms after K's first press, not after its second"
    );
    p.flush_at(ms(t, 16)).unwrap();
    assert_eq!(got(&log).len(), 8);
}

/// Recorder that timestamps each injected event with a shared fake clock.
struct Timed(
    Arc<Mutex<Instant>>,
    Arc<Mutex<Vec<(KeyCode, bool, Instant)>>>,
);

impl Platform for Timed {
    fn initialize(&mut self) -> PlatformResult<()> {
        Ok(())
    }
    fn capture_input(&mut self) -> PlatformResult<KeyEvent> {
        Err(super::super::PlatformError::NoInput)
    }
    fn inject_output(&mut self, event: KeyEvent) -> PlatformResult<()> {
        let now = *self.0.lock().unwrap();
        self.1
            .lock()
            .unwrap()
            .push((event.keycode(), event.is_press(), now));
        Ok(())
    }
    fn list_devices(&self) -> PlatformResult<Vec<DeviceInfo>> {
        Ok(Vec::new())
    }
    fn shutdown(&mut self) -> PlatformResult<()> {
        Ok(())
    }
}

/// The `i`th key of [`flood_and_drain`]'s burst.
fn rolled_key(keys: &[KeyCode], i: usize) -> KeyCode {
    keys[(i * 3) % keys.len()]
}

/// Floods `taps` rolled-over taps over `keys` (each key pressed before
/// the previous one is released, as in fast typing or a resync), one
/// every 100 us - far faster than the minimum allows - then ticks the
/// fake clock 1 ms at a time until drained and returns the output.
fn flood_and_drain(keys: &[KeyCode], taps: usize) -> (Vec<(KeyCode, bool, Instant)>, Instant) {
    let clock = Arc::new(Mutex::new(Instant::now()));
    let log = Arc::default();
    let timed = Timed(Arc::clone(&clock), Arc::clone(&log));
    let mut p = MinKeyDown::new(Box::new(timed), MIN);
    let t = *clock.lock().unwrap();
    let mut now = t;
    let key = |i: usize| rolled_key(keys, i);
    for i in 0..taps {
        now = t + Duration::from_micros(100 * i as u64);
        *clock.lock().unwrap() = now;
        press(&mut p, key(i), now);
        if i > 0 {
            release(&mut p, key(i - 1), now);
        }
    }
    release(&mut p, key(taps - 1), now);
    for _ in 0..(taps as u64 * 50) {
        if p.queue.is_empty() {
            break;
        }
        now += Duration::from_millis(1);
        *clock.lock().unwrap() = now;
        p.flush_at(now).unwrap();
    }
    assert!(
        p.queue.is_empty(),
        "queue never drained: {} left",
        p.queue.len()
    );
    let out = log.lock().unwrap().clone();
    (out, t)
}

/// Presses come out in input order, every key alternates press/release,
/// each is down at least `MIN`, and all end up released.
fn assert_well_formed(out: &[(KeyCode, bool, Instant)], presses: &[KeyCode]) {
    assert_eq!(out.len(), presses.len() * 2);
    let pressed: Vec<KeyCode> = out.iter().filter(|e| e.1).map(|e| e.0).collect();
    assert_eq!(pressed, presses, "presses reordered");
    let mut down: HashMap<KeyCode, Instant> = HashMap::new();
    for (key, is_press, at) in out {
        if *is_press {
            assert!(down.insert(*key, *at).is_none(), "{key:?} pressed twice");
        } else {
            let since = down.remove(key).expect("release without press");
            assert!(*at >= since + MIN, "{key:?} down for less than the minimum");
        }
    }
    assert!(down.is_empty(), "keys left down: {down:?}");
}

#[test]
fn a_burst_of_repeated_keys_drains_completely_and_in_bounded_time() {
    let keys = [
        KeyCode::A,
        KeyCode::S,
        KeyCode::D,
        KeyCode::F,
        KeyCode::Escape,
        KeyCode::J,
        KeyCode::K,
        KeyCode::L,
    ];
    let taps = 500;
    let (out, t) = flood_and_drain(&keys, taps);
    let presses: Vec<KeyCode> = (0..taps).map(|i| rolled_key(&keys, i)).collect();
    assert_well_formed(&out, &presses);
    let last = out.last().map(|e| e.2).unwrap();
    // Only a key's own previous tap can hold it back: one minimum (plus a
    // 1 ms flush tick) per round over the keys - not one minimum per tap.
    let rounds = taps.div_ceil(keys.len()) as u32;
    let bound = (MIN + Duration::from_millis(1)) * rounds + MIN;
    assert!(last <= t + bound, "drain took {:?}", last - t);
}

#[test]
fn next_due_tells_the_capture_loop_how_long_it_may_sleep() {
    let (mut p, _log, t) = rig();
    assert_eq!(p.next_due_in(t), None);
    press(&mut p, KeyCode::A, t);
    release(&mut p, KeyCode::A, t);
    assert_eq!(p.next_due_in(t), Some(MIN));
    assert_eq!(p.next_due_in(ms(t, 9)), Some(Duration::ZERO));
}
