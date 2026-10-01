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

use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

use keyrx_core::config::KeyCode;
use keyrx_core::runtime::KeyEvent;

use super::{DeviceInfo, OutputDeviceInfo, Platform, PlatformResult};

/// Default minimum key-down time.
pub const DEFAULT_MIN_KEY_DOWN: Duration = Duration::from_millis(5);

/// Largest allowed minimum: beyond this typing visibly lags.
pub const MAX_MIN_KEY_DOWN: Duration = Duration::from_millis(200);

/// An event waiting for its turn.
struct Queued {
    due: Instant,
    event: KeyEvent,
}

/// A [`Platform`] that keeps every injected key down for at least `min`.
pub struct MinKeyDown {
    inner: Box<dyn Platform>,
    min: Duration,
    /// Events held back, oldest first, with non-decreasing due times.
    queue: VecDeque<Queued>,
    /// When each key was (or is scheduled to be) pressed on the output.
    pressed_at: HashMap<KeyCode, Instant>,
}

impl MinKeyDown {
    pub fn new(inner: Box<dyn Platform>, min: Duration) -> Self {
        Self {
            inner,
            min,
            queue: VecDeque::new(),
            pressed_at: HashMap::new(),
        }
    }

    /// Injects `event` (or queues it behind a deferred release) at `now`.
    fn inject_at(&mut self, event: KeyEvent, now: Instant) -> PlatformResult<()> {
        self.flush_at(now);
        let key = event.keycode();
        if event.is_press() {
            if self.queue.is_empty() {
                self.pressed_at.insert(key, now);
                return self.inner.inject_output(event);
            }
            let due = self.queue.back().map_or(now, |q| q.due);
            self.pressed_at.insert(key, due);
            self.queue.push_back(Queued { due, event });
            return Ok(());
        }
        let after_press = self.pressed_at.get(&key).map(|t| *t + self.min);
        let after_queue = self.queue.back().map(|q| q.due);
        match after_press.into_iter().chain(after_queue).max() {
            Some(due) if due > now => {
                self.queue.push_back(Queued { due, event });
                Ok(())
            }
            _ => self.inner.inject_output(event),
        }
    }

    /// Injects every queued event whose time has come, in order.
    fn flush_at(&mut self, now: Instant) {
        while self.queue.front().is_some_and(|q| q.due <= now) {
            let Some(item) = self.queue.pop_front() else {
                break;
            };
            if item.event.is_press() {
                // It went out later than scheduled; its release must still
                // wait a full minimum after the REAL press.
                self.pressed_at.insert(item.event.keycode(), now);
                self.reschedule();
            }
            if let Err(e) = self.inner.inject_output(item.event) {
                log::warn!("Failed to inject a deferred key event: {e}");
            }
        }
    }

    /// Injects everything still queued, immediately and in order.
    fn flush_all(&mut self) {
        while let Some(item) = self.queue.pop_front() {
            if let Err(e) = self.inner.inject_output(item.event) {
                log::warn!("Failed to inject a deferred key event: {e}");
            }
        }
    }

    /// Recomputes queued due times after a press went out late, keeping them
    /// non-decreasing and each release a full minimum after its press.
    fn reschedule(&mut self) {
        let mut prev: Option<Instant> = None;
        for item in &mut self.queue {
            let key = item.event.keycode();
            let mut due = prev.map_or(item.due, |p| item.due.max(p));
            if item.event.is_press() {
                self.pressed_at.insert(key, due);
            } else if let Some(t) = self.pressed_at.get(&key) {
                due = due.max(*t + self.min);
            }
            item.due = due;
            prev = Some(due);
        }
    }

    /// Time until the oldest queued event is due.
    fn next_due_in(&self, now: Instant) -> Option<Duration> {
        self.queue
            .front()
            .map(|q| q.due.saturating_duration_since(now))
    }
}

impl Platform for MinKeyDown {
    fn initialize(&mut self) -> PlatformResult<()> {
        self.inner.initialize()
    }

    fn capture_input(&mut self) -> PlatformResult<KeyEvent> {
        self.flush_at(Instant::now());
        let limit = self.next_due_in(Instant::now());
        self.inner.set_input_wait_limit(limit);
        let result = self.inner.capture_input();
        self.flush_at(Instant::now());
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
        p.flush_at(ms(t, 4));
        assert_eq!(got(&log).len(), 1, "still inside the window");
        p.flush_at(ms(t, 5));
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
        p.flush_at(ms(t, 5));
        assert_eq!(
            got(&log),
            vec![(KeyCode::A, true), (KeyCode::A, false), (KeyCode::B, true)]
        );
        p.flush_at(ms(t, 10));
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
        p.flush_at(ms(t, 8)); // loop was late: A up + B down go out at t+8
        assert_eq!(got(&log).len(), 3);
        p.flush_at(ms(t, 10));
        assert_eq!(got(&log).len(), 3, "B up must wait until t+13");
        p.flush_at(ms(t, 13));
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
        p.flush_at(ms(t, 5));
        p.flush_at(ms(t, 10));
        let events = got(&log);
        assert_eq!(events.iter().filter(|e| e.1).count(), 2);
        assert_eq!(events.iter().filter(|e| !e.1).count(), 2);
        for pair in events.chunks(2) {
            assert_eq!(pair[0], (KeyCode::A, true));
        }
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
