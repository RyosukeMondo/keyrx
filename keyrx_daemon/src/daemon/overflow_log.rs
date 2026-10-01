//! Rate limiting for the input-overflow (`SYN_DROPPED`) warning.
//!
//! A flood (a stuck macro tool, a key held on a noisy device) can overflow the
//! kernel's event buffer hundreds of times a second; one log line per overflow
//! then buries every other message. The first overflow is reported at once,
//! later ones are counted and summarised once per [`SUMMARY_INTERVAL`]. The
//! running total is always available in status (`input_overflows`).

use std::time::{Duration, Instant};

/// Longest gap between two overflow log lines.
pub const SUMMARY_INTERVAL: Duration = Duration::from_secs(5);

/// One line's worth of overflows.
#[derive(Debug, PartialEq, Eq)]
pub struct OverflowSummary {
    /// Overflows since the previous line.
    pub count: u64,
    /// The window the count covers (zero for the first, immediate line).
    pub window: Duration,
}

/// Counts overflows and decides when a log line is due. Pure of any clock or
/// logger: callers pass `now`, so it is tested with a fake clock.
#[derive(Debug)]
pub struct OverflowLog {
    interval: Duration,
    pending: u64,
    last_logged: Option<Instant>,
}

impl OverflowLog {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            pending: 0,
            last_logged: None,
        }
    }

    /// Records `count` new overflows seen at `now`; returns a summary if a
    /// line is due.
    pub fn record(&mut self, count: u64, now: Instant) -> Option<OverflowSummary> {
        self.pending += count;
        self.poll(now)
    }

    /// Returns a summary if overflows are pending and the interval since the
    /// last line has passed (call this regularly so a trailing burst is still
    /// reported after the flood stops).
    pub fn poll(&mut self, now: Instant) -> Option<OverflowSummary> {
        if self.pending == 0 {
            return None;
        }
        let window = match self.last_logged {
            None => Duration::ZERO,
            Some(last) if now.duration_since(last) >= self.interval => now.duration_since(last),
            Some(_) => return None,
        };
        self.last_logged = Some(now);
        Some(OverflowSummary {
            count: std::mem::take(&mut self.pending),
            window,
        })
    }
}

impl Default for OverflowLog {
    fn default() -> Self {
        Self::new(SUMMARY_INTERVAL)
    }
}

impl std::fmt::Display for OverflowSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.window.is_zero() {
            write!(f, "{} input overflow(s)", self.count)
        } else {
            write!(
                f,
                "{} input overflow(s) in the last {:.0}s",
                self.count,
                self.window.as_secs_f64()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STEP: Duration = Duration::from_secs(1);

    #[test]
    fn first_overflow_is_reported_immediately() {
        let t0 = Instant::now();
        let mut log = OverflowLog::new(Duration::from_secs(5));
        let s = log.record(1, t0).expect("first is immediate");
        assert_eq!(s.count, 1);
        assert_eq!(s.window, Duration::ZERO);
    }

    #[test]
    fn a_flood_produces_one_summary_per_interval_with_the_count() {
        let t0 = Instant::now();
        let mut log = OverflowLog::new(Duration::from_secs(5));
        log.record(1, t0);
        let mut lines = 0;
        for i in 1..=4 {
            assert_eq!(log.record(100, t0 + STEP * i), None, "suppressed");
        }
        if let Some(s) = log.record(100, t0 + STEP * 5) {
            lines += 1;
            assert_eq!(s.count, 500);
            assert_eq!(s.window, STEP * 5);
        }
        assert_eq!(lines, 1);
    }

    #[test]
    fn a_trailing_burst_is_flushed_by_polling_after_the_flood_stops() {
        let t0 = Instant::now();
        let mut log = OverflowLog::new(Duration::from_secs(5));
        log.record(1, t0);
        assert_eq!(log.record(7, t0 + STEP), None);
        assert_eq!(log.poll(t0 + STEP * 3), None);
        assert_eq!(log.poll(t0 + STEP * 6).map(|s| s.count), Some(7));
        assert_eq!(log.poll(t0 + STEP * 20), None, "nothing left to report");
    }

    #[test]
    fn summary_text_names_the_count_and_window() {
        let s = OverflowSummary {
            count: 12,
            window: Duration::from_secs(5),
        };
        assert_eq!(s.to_string(), "12 input overflow(s) in the last 5s");
    }
}
