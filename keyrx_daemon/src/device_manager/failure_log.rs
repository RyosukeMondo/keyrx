//! Log policy for devices that match a `device_start` pattern but cannot yet
//! be opened or grabbed.
//!
//! A freshly created `/dev/input/eventN` node is briefly root-only (udev has
//! not applied permissions yet) or already gone (`ENODEV`) during a replug
//! storm; hotplug retries on the next inotify event anyway. Warning for each
//! attempt produced ~800 lines with a misleading "add yourself to `input`"
//! hint. A failure is therefore logged at `debug` until it has persisted for
//! [`HOTPLUG_GRACE`], warned about exactly once, then stays at `debug`.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// How long a hotplugged node may fail before it is worth a warning.
pub const HOTPLUG_GRACE: Duration = Duration::from_secs(2);

/// How loudly to report one failed open/grab attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Loudness {
    Debug,
    Warn,
}

/// Pure policy: `since_first` is how long this path has been failing.
pub fn failure_loudness(since_first: Duration, already_warned: bool, grace: Duration) -> Loudness {
    if already_warned || since_first < grace {
        Loudness::Debug
    } else {
        Loudness::Warn
    }
}

struct Failing {
    first: Instant,
    /// Grace chosen when the failure started; later attempts reuse it.
    grace: Duration,
    warned: bool,
}

/// Remembers, per device path, since when it has been failing.
#[derive(Default)]
pub struct FailureTracker {
    failing: HashMap<PathBuf, Failing>,
}

impl FailureTracker {
    /// Records a failed attempt on `path` and returns how loudly to report it.
    /// `grace` applies when this is the path's first failure and is zero for
    /// devices that were already present (startup, config reload): those are
    /// real failures, not udev races. Later attempts keep the original grace.
    pub fn record(&mut self, path: &Path, now: Instant, grace: Duration) -> Loudness {
        let entry = self.failing.entry(path.to_path_buf()).or_insert(Failing {
            first: now,
            grace,
            warned: false,
        });
        let loudness = failure_loudness(now.duration_since(entry.first), entry.warned, entry.grace);
        entry.warned |= loudness == Loudness::Warn;
        loudness
    }

    /// Forgets paths that were not failing in the latest pass (opened fine,
    /// unplugged, or no longer matched).
    pub fn retain_failing(&mut self, still_failing: &[&Path]) {
        self.failing
            .retain(|p, _| still_failing.contains(&p.as_path()));
    }

    /// True when a not-yet-warned failure has outlived `grace`, so a retry
    /// would now be reported. Lets the caller re-scan once without waiting
    /// for another hotplug event.
    pub fn retry_due(&self, now: Instant) -> bool {
        self.failing
            .values()
            .any(|f| !f.warned && now.duration_since(f.first) >= f.grace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const G: Duration = Duration::from_secs(2);

    #[test]
    fn young_failures_are_debug() {
        assert_eq!(
            failure_loudness(Duration::from_millis(500), false, G),
            Loudness::Debug
        );
    }

    #[test]
    fn persistent_failure_warns_once() {
        assert_eq!(failure_loudness(G, false, G), Loudness::Warn);
        assert_eq!(
            failure_loudness(Duration::from_secs(60), true, G),
            Loudness::Debug
        );
    }

    #[test]
    fn zero_grace_warns_immediately() {
        assert_eq!(
            failure_loudness(Duration::ZERO, false, Duration::ZERO),
            Loudness::Warn
        );
    }

    #[test]
    fn storm_of_retries_yields_one_warning() {
        let mut t = FailureTracker::default();
        let p = Path::new("/dev/input/event99");
        let t0 = Instant::now();
        let mut warns = 0;
        for i in 0..800u64 {
            let now = t0 + Duration::from_millis(i * 20); // 16 s of retries
            if t.record(p, now, G) == Loudness::Warn {
                warns += 1;
            }
        }
        assert_eq!(warns, 1);
    }

    #[test]
    fn recovered_paths_are_forgotten_and_start_a_new_grace() {
        let mut t = FailureTracker::default();
        let p = Path::new("/dev/input/event7");
        let t0 = Instant::now();
        t.record(p, t0, G);
        t.retain_failing(&[]);
        assert!(!t.retry_due(t0 + G));
        assert_eq!(
            t.record(p, t0 + Duration::from_secs(10), G),
            Loudness::Debug
        );
    }

    #[test]
    fn a_later_attempt_does_not_shorten_the_original_grace() {
        let mut t = FailureTracker::default();
        let p = Path::new("/dev/input/event9");
        let t0 = Instant::now();
        t.record(p, t0, G);
        // The caller now sees the path as "already known" and passes zero.
        let l = t.record(p, t0 + Duration::from_millis(300), Duration::ZERO);
        assert_eq!(l, Loudness::Debug);
    }

    #[test]
    fn retry_is_due_only_for_unwarned_failures_past_grace() {
        let mut t = FailureTracker::default();
        let p = Path::new("/dev/input/event8");
        let t0 = Instant::now();
        t.record(p, t0, G);
        assert!(!t.retry_due(t0 + Duration::from_secs(1)));
        assert!(t.retry_due(t0 + G));
        t.record(p, t0 + G, G); // warns
        assert!(!t.retry_due(t0 + Duration::from_secs(9)));
    }
}
