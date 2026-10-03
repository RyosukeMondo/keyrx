//! Minimum key-down time for output keys.
//!
//! A tap, a sequence or a remapped tap is emitted as a press and a release a
//! few microseconds apart. Programs that poll key state once per frame (games,
//! some remote-desktop and accessibility tools) can miss a pulse that short.
//! [`MinKeyDown`] wraps any [`Platform`] and, when a release would follow its
//! press by less than the configured minimum, holds that release back until
//! the minimum has passed.
//!
//! Rules, all covered by tests with a fake clock:
//!
//! - only a release that arrives inside the window is deferred; a key held
//!   longer, and every press, passes through immediately;
//! - order is preserved: anything emitted after a deferred release waits
//!   behind it (at most the window, default 5 ms), so output never reorders;
//! - a profile switch and shutdown flush every deferred release at once;
//! - the deferred release is serviced by the event loop's capture tick, which
//!   this decorator shortens (see [`Platform::set_input_wait_limit`]).
//!
//! A release's deadline is derived from ONE fact: when its key's press really
//! went out on the output ([`MinKeyDown::down_since`]). Nothing scheduled is
//! stored, so no stale schedule can push a release into the future. (An
//! earlier design stored per-key "scheduled press" times and recomputed queued
//! deadlines from them; under a burst with repeated keys a release picked up
//! the time of its key's NEXT queued press, deadlines ran away and the output
//! froze with a key held down.)

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEvent;

use super::{DeviceInfo, OutputDeviceInfo, Platform, PlatformResult};

/// Default minimum key-down time.
pub const DEFAULT_MIN_KEY_DOWN: Duration = Duration::from_millis(5);

/// Largest allowed minimum: beyond this typing visibly lags.
pub const MAX_MIN_KEY_DOWN: Duration = Duration::from_millis(200);

/// A [`Platform`] that keeps every injected key down for at least `min`.
pub struct MinKeyDown {
    inner: Box<dyn Platform>,
    min: Duration,
    /// Events held back, oldest first.
    queue: VecDeque<KeyEvent>,
    /// Keys down on the output, with the time their press actually went out.
    down_since: HashMap<KeyCode, Instant>,
}

impl MinKeyDown {
    pub fn new(inner: Box<dyn Platform>, min: Duration) -> Self {
        Self {
            inner,
            min,
            queue: VecDeque::new(),
            down_since: HashMap::new(),
        }
    }

    /// Queues `event` and emits everything that may go out at `now`. Returns
    /// the first injection failure, if any (later events are still tried).
    fn inject_at(&mut self, event: KeyEvent, now: Instant) -> PlatformResult<()> {
        self.queue.push_back(event);
        self.flush_at(now)
    }

    /// When `event` may go out: a press at once, a release a full minimum
    /// after its key's real press (at once if that press is not known).
    fn due(&self, event: &KeyEvent) -> Option<Instant> {
        if event.is_press() {
            return None;
        }
        self.down_since.get(&event.keycode()).map(|t| *t + self.min)
    }

    /// Emits every queued event whose time has come, in order.
    fn flush_at(&mut self, now: Instant) -> PlatformResult<()> {
        let mut result = Ok(());
        while let Some(front) = self.queue.front() {
            if self.due(front).is_some_and(|due| due > now) {
                break;
            }
            let Some(event) = self.queue.pop_front() else {
                break;
            };
            let emitted = self.emit(event, now);
            if result.is_ok() {
                result = emitted;
            }
        }
        result
    }

    /// Injects `event` and records the output key state it creates.
    fn emit(&mut self, event: KeyEvent, now: Instant) -> PlatformResult<()> {
        if event.is_press() {
            self.down_since.insert(event.keycode(), now);
        } else {
            self.down_since.remove(&event.keycode());
        }
        self.inner.inject_output(event)
    }

    /// Injects everything still queued, immediately and in order.
    fn flush_all(&mut self) {
        let now = Instant::now();
        while let Some(event) = self.queue.pop_front() {
            if let Err(e) = self.emit(event, now) {
                log::warn!("Failed to inject a deferred key event: {e}");
            }
        }
    }

    /// Time until the oldest queued event is due.
    fn next_due_in(&self, now: Instant) -> Option<Duration> {
        let front = self.queue.front()?;
        Some(
            self.due(front)
                .map_or(Duration::ZERO, |due| due.saturating_duration_since(now)),
        )
    }

    /// Like [`Self::flush_at`], logging a failure instead of returning it
    /// (the capture path has no caller that could act on it).
    fn flush_logged(&mut self, now: Instant) {
        if let Err(e) = self.flush_at(now) {
            log::warn!("Failed to inject a deferred key event: {e}");
        }
    }
}

impl Platform for MinKeyDown {
    fn initialize(&mut self) -> PlatformResult<()> {
        self.inner.initialize()
    }

    fn capture_input(&mut self) -> PlatformResult<KeyEvent> {
        self.flush_logged(Instant::now());
        let limit = self.next_due_in(Instant::now());
        self.inner.set_input_wait_limit(limit);
        let result = self.inner.capture_input();
        self.flush_logged(Instant::now());
        result
    }

    fn inject_output(&mut self, event: KeyEvent) -> PlatformResult<()> {
        self.inject_at(event, Instant::now())
    }

    fn list_devices(&self) -> PlatformResult<Vec<DeviceInfo>> {
        self.inner.list_devices()
    }

    fn shutdown(&mut self) -> PlatformResult<()> {
        self.flush_all();
        self.inner.shutdown()
    }

    fn query_ime_state(&self) -> Option<keyrx_core::config::ImeState> {
        self.inner.query_ime_state()
    }

    fn release_held_outputs(&mut self) -> PlatformResult<usize> {
        self.flush_all();
        self.inner.release_held_outputs()
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

    fn set_input_wait_limit(&mut self, limit: Option<Duration>) {
        self.inner.set_input_wait_limit(limit);
    }

    fn output_device(&self) -> Option<OutputDeviceInfo> {
        self.inner.output_device()
    }
}

#[cfg(test)]
mod tests {
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
    fn order_is_preserved_events_after_a_deferred_release_wait_behind_it() {
        let (mut p, log, t) = rig();
        press(&mut p, KeyCode::A, t);
        release(&mut p, KeyCode::A, t); // deferred to t+5
        press(&mut p, KeyCode::B, ms(t, 1)); // must not overtake A's release
        release(&mut p, KeyCode::B, ms(t, 1));
        assert_eq!(got(&log), vec![(KeyCode::A, true)]);
        p.flush_at(ms(t, 5)).unwrap();
        assert_eq!(
            got(&log),
            vec![(KeyCode::A, true), (KeyCode::A, false), (KeyCode::B, true)]
        );
        p.flush_at(ms(t, 10)).unwrap();
        assert_eq!(got(&log).last(), Some(&(KeyCode::B, false)));
        assert_eq!(got(&log).len(), 4);
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
        press(&mut p, KeyCode::B, t); // due t+5 (behind A up)
        release(&mut p, KeyCode::B, t); // due t+10 (B must be down 5 ms)
        p.flush_at(ms(t, 8)).unwrap(); // loop was late: A up + B down go out at t+8
        assert_eq!(got(&log).len(), 3);
        p.flush_at(ms(t, 10)).unwrap();
        assert_eq!(got(&log).len(), 3, "B up must wait until t+13");
        p.flush_at(ms(t, 13)).unwrap();
        assert_eq!(got(&log).len(), 4);
    }

    #[test]
    fn release_held_outputs_and_shutdown_flush_everything_at_once() {
        let (mut p, log, t) = rig();
        press(&mut p, KeyCode::A, t);
        release(&mut p, KeyCode::A, t);
        press(&mut p, KeyCode::B, t);
        p.release_held_outputs().unwrap();
        assert_eq!(
            got(&log),
            vec![(KeyCode::A, true), (KeyCode::A, false), (KeyCode::B, true)]
        );
        release(&mut p, KeyCode::B, t);
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
        press(&mut p, KeyCode::A, t);
        release(&mut p, KeyCode::A, t); // A up due t+5; all below queue behind it
        press(&mut p, KeyCode::K, t);
        press(&mut p, KeyCode::X, t);
        release(&mut p, KeyCode::K, t);
        release(&mut p, KeyCode::X, t);
        press(&mut p, KeyCode::K, t); // K's NEXT press, still queued
        release(&mut p, KeyCode::K, t);
        p.flush_at(ms(t, 6)).unwrap(); // A up, K down, X down at t+6
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
        let key = |i: usize| keys[(i * 3) % keys.len()];
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

    /// Every key alternates press/release, each down at least `MIN`, and
    /// ends up released.
    fn assert_well_formed(out: &[(KeyCode, bool, Instant)], taps: usize) {
        assert_eq!(out.len(), taps * 2);
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
        assert_well_formed(&out, taps);
        let last = out.last().map(|e| e.2).unwrap();
        // Strictly ordered output: one minimum per tap, plus a few ticks.
        assert!(
            last <= t + MIN * taps as u32 + Duration::from_millis(10),
            "drain took {:?}",
            last - t
        );
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
}
