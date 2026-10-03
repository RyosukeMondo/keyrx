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
//! - the hold is per key, so different keys may overlap: while a tapped key
//!   waits out its minimum, the press of a DIFFERENT ordinary key goes out
//!   at once (the same rollover fast human typing produces). Without that,
//!   every tap would cost a full minimum and output would top out at about
//!   200 keys/s, queueing bursts for seconds;
//! - everything else keeps its order: presses never reorder, releases leave
//!   in arrival order, a key is never pressed again before its own release,
//!   and modifiers (Shift/Ctrl/Alt/Meta) are barriers - a modifier event
//!   never overtakes anything, and nothing overtakes a deferred modifier
//!   release - so the modifier state every key sees is exactly the input's;
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

    /// Index of the next queued event allowed out at `now`, if any: the
    /// oldest one once it is due, or else the first press that may overtake
    /// the deferred releases ahead of it (see the module docs).
    fn next_ready(&self, now: Instant) -> Option<usize> {
        let front = self.queue.front()?;
        if self.due(front).is_none_or(|due| due <= now) {
            return Some(0);
        }
        let mut waiting: Vec<KeyCode> = Vec::new();
        for (i, event) in self.queue.iter().enumerate() {
            let key = event.keycode();
            if key.is_modifier() {
                return None;
            }
            if event.is_press() {
                return (!waiting.contains(&key)).then_some(i);
            }
            waiting.push(key);
        }
        None
    }

    /// Emits every queued event whose time has come.
    fn flush_at(&mut self, now: Instant) -> PlatformResult<()> {
        let mut result = Ok(());
        while let Some(i) = self.next_ready(now) {
            let Some(event) = self.queue.remove(i) else {
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
mod tests;
