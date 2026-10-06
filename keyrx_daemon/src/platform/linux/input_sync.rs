//! `SYN_DROPPED` handling for one evdev device.
//!
//! When a reader falls behind, the kernel drops its queued events and puts a
//! `SYN_DROPPED` marker in the stream. Everything up to the next `SYN_REPORT`
//! is untrustworthy, and any key release inside the lost span is gone for
//! good - without a resync the daemon believes the key is still held, its
//! output keeps the key DOWN, and later presses are swallowed.
//!
//! [`DropSync`] sits between the raw event stream and the rest of the daemon
//! and keeps one invariant: **per key, the events it emits strictly alternate
//! press, release**. It gets there by
//!
//! 1. discarding the damaged packet that follows a `SYN_DROPPED`,
//! 2. diffing the keys it has reported as held against the kernel's real key
//!    state (`EVIOCGKEY`) and synthesising the missing releases (and presses),
//! 3. discarding duplicate presses / orphan releases that the race between
//!    the ioctl and the still-queued events can produce.
//!
//! The crate-level `evdev::Device` also does a resync, but over a non-blocking
//! fd it loses the synthetic events when the follow-up read has nothing to
//! read (the sync is computed, then `WouldBlock` aborts the call) - exactly the
//! stuck-keys failure. Owning the logic here makes it testable and fixes that.

use std::collections::BTreeSet;
use std::io;

use evdev::{EventType, InputEvent, Synchronization};

/// Outcome of feeding a batch of raw events through [`DropSync::process`].
#[derive(Debug, Default)]
pub(crate) struct SyncOutput {
    /// Trustworthy events, in order, with synthetic resync events included.
    pub events: Vec<InputEvent>,
    /// How many `SYN_DROPPED` markers were resolved in this batch.
    pub overflows: u64,
    /// How many stale keys were released by resyncs in this batch.
    pub released: usize,
}

/// The device's real held-key codes (`EVIOCGKEY`).
pub(crate) fn kernel_keys(device: &evdev::raw_stream::RawDevice) -> io::Result<BTreeSet<u16>> {
    device
        .get_key_state()
        .map(|keys| keys.iter().map(|key| key.code()).collect())
}

/// Per-device `SYN_DROPPED` filter. See the module docs.
#[derive(Debug, Default)]
pub(crate) struct DropSync {
    /// `EV_KEY` codes reported downstream as held.
    held: BTreeSet<u16>,
    /// Events of the packet currently being assembled.
    packet: Vec<InputEvent>,
    /// A `SYN_DROPPED` was seen; the rest of its packet is discarded and a
    /// resync happens at the closing `SYN_REPORT`.
    dropped: bool,
}

impl DropSync {
    /// Creates a filter that believes no key is held.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Keys currently reported as held (tests and diagnostics).
    #[cfg(test)]
    pub(crate) fn held(&self) -> &BTreeSet<u16> {
        &self.held
    }

    /// Whether any key is currently reported as held.
    pub(crate) fn has_held(&self) -> bool {
        !self.held.is_empty()
    }

    /// Idle reconciliation against the kernel's real key state. Only valid
    /// when the device has nothing left to read (the caller just got
    /// `WouldBlock`). Does nothing mid-packet or if the key state cannot be
    /// read.
    ///
    /// `EVIOCGKEY` is not a passive read: the kernel copies the key bitmap
    /// AND flushes every pending `EV_KEY` event from this client's buffer in
    /// one step (so the state is consistent with what is left to read). A
    /// key event that lands between our last read and the ioctl is therefore
    /// gone, and only the returned state shows its effect. So both
    /// directions are repaired from that state: a key we think is held but
    /// the kernel says is up gets a synthetic release, and a key the kernel
    /// says is down that we never saw pressed gets a synthetic press (its
    /// real release then arrives balanced). Dropping the press instead made
    /// a keystroke typed exactly when the check ran vanish.
    pub(crate) fn reconcile_idle(
        &mut self,
        mut kernel_keys: impl FnMut() -> io::Result<BTreeSet<u16>>,
    ) -> SyncOutput {
        let mut out = SyncOutput::default();
        if self.dropped || !self.packet.is_empty() {
            return out;
        }
        let Ok(actual) = kernel_keys() else {
            return out;
        };
        let stale: Vec<u16> = self.held.difference(&actual).copied().collect();
        let missed: Vec<u16> = actual.difference(&self.held).copied().collect();
        if stale.is_empty() && missed.is_empty() {
            return out;
        }
        out.overflows = 1;
        out.released = stale.len();
        for code in stale {
            self.held.remove(&code);
            out.events
                .push(InputEvent::new_now(EventType::KEY, code, 0));
        }
        for code in missed {
            self.held.insert(code);
            out.events
                .push(InputEvent::new_now(EventType::KEY, code, 1));
        }
        out
    }

    /// Filters `raw`. `kernel_keys` returns the device's real held-key codes
    /// (`EVIOCGKEY`); it is only called when a resync is needed. If it fails
    /// every key we believed held is released - the safe direction.
    pub(crate) fn process(
        &mut self,
        raw: impl IntoIterator<Item = InputEvent>,
        mut kernel_keys: impl FnMut() -> io::Result<BTreeSet<u16>>,
    ) -> SyncOutput {
        let mut out = SyncOutput::default();
        for ev in raw {
            if ev.event_type() == EventType::SYNCHRONIZATION {
                match Synchronization(ev.code()) {
                    Synchronization::SYN_DROPPED => {
                        self.packet.clear();
                        self.dropped = true;
                    }
                    Synchronization::SYN_REPORT => self.close_packet(&mut kernel_keys, &mut out),
                    _ => {}
                }
            } else {
                self.packet.push(ev);
            }
        }
        out
    }

    fn close_packet(
        &mut self,
        kernel_keys: &mut impl FnMut() -> io::Result<BTreeSet<u16>>,
        out: &mut SyncOutput,
    ) {
        if self.dropped {
            self.dropped = false;
            self.packet.clear();
            out.overflows += 1;
            self.resync(kernel_keys, out);
            return;
        }
        for ev in std::mem::take(&mut self.packet) {
            if self.admit(&ev) {
                out.events.push(ev);
            }
        }
    }

    /// Whether `ev` keeps the press/release alternation intact.
    fn admit(&mut self, ev: &InputEvent) -> bool {
        if ev.event_type() != EventType::KEY {
            return true;
        }
        match ev.value() {
            1 => self.held.insert(ev.code()),
            0 => self.held.remove(&ev.code()),
            // Autorepeat is only meaningful for a key we know is down.
            _ => self.held.contains(&ev.code()),
        }
    }

    fn resync(
        &mut self,
        kernel_keys: &mut impl FnMut() -> io::Result<BTreeSet<u16>>,
        out: &mut SyncOutput,
    ) {
        let actual = kernel_keys().unwrap_or_else(|e| {
            log::warn!("Could not read the keyboard's key state to resync ({e}); releasing all");
            BTreeSet::new()
        });
        let stale: Vec<u16> = self.held.difference(&actual).copied().collect();
        let missed: Vec<u16> = actual.difference(&self.held).copied().collect();
        out.released += stale.len();
        for code in stale {
            self.held.remove(&code);
            out.events
                .push(InputEvent::new_now(EventType::KEY, code, 0));
        }
        for code in missed {
            self.held.insert(code);
            out.events
                .push(InputEvent::new_now(EventType::KEY, code, 1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: u16 = 30;
    const S: u16 = 31;

    fn key(code: u16, value: i32) -> InputEvent {
        InputEvent::new_now(EventType::KEY, code, value)
    }
    fn syn(kind: Synchronization) -> InputEvent {
        InputEvent::new_now(EventType::SYNCHRONIZATION, kind.0, 0)
    }
    fn report() -> InputEvent {
        syn(Synchronization::SYN_REPORT)
    }
    fn dropped() -> InputEvent {
        syn(Synchronization::SYN_DROPPED)
    }
    fn keys(codes: &[u16]) -> impl FnMut() -> io::Result<BTreeSet<u16>> + '_ {
        move || Ok(codes.iter().copied().collect())
    }
    fn pairs(out: &SyncOutput) -> Vec<(u16, i32)> {
        out.events.iter().map(|e| (e.code(), e.value())).collect()
    }

    #[test]
    fn well_formed_packets_pass_through_untouched() {
        let mut sync = DropSync::new();
        let out = sync.process([key(A, 1), report(), key(A, 0), report()], keys(&[]));
        assert_eq!(pairs(&out), vec![(A, 1), (A, 0)]);
        assert_eq!(out.overflows, 0);
    }

    #[test]
    fn a_release_lost_in_the_dropped_span_is_synthesised() {
        let mut sync = DropSync::new();
        sync.process([key(A, 1), report()], keys(&[]));
        assert!(sync.held().contains(&A));
        // The release of A was in the lost span; kernel says nothing is held.
        let out = sync.process([dropped(), key(S, 1), report()], keys(&[]));
        assert_eq!(pairs(&out), vec![(A, 0)]);
        assert_eq!(out.overflows, 1);
        assert_eq!(out.released, 1);
        assert!(sync.held().is_empty());
    }

    #[test]
    fn a_press_lost_in_the_dropped_span_is_synthesised() {
        let mut sync = DropSync::new();
        let out = sync.process([dropped(), report()], keys(&[S]));
        assert_eq!(pairs(&out), vec![(S, 1)]);
        // ...so its later release is balanced rather than an orphan.
        let out = sync.process([key(S, 0), report()], keys(&[]));
        assert_eq!(pairs(&out), vec![(S, 0)]);
    }

    #[test]
    fn the_damaged_packet_after_syn_dropped_is_discarded() {
        let mut sync = DropSync::new();
        let out = sync.process([key(A, 1), dropped(), key(S, 1), report()], keys(&[]));
        assert!(out.events.is_empty(), "{:?}", pairs(&out));
        assert!(sync.held().is_empty());
    }

    #[test]
    fn events_queued_after_the_resync_are_filtered_against_the_new_state() {
        let mut sync = DropSync::new();
        // The kernel already reports A held when we resync; its press is then
        // also still in the queue behind the SYN_DROPPED packet.
        let out = sync.process(
            [
                dropped(),
                report(),
                key(A, 1),
                report(),
                key(A, 0),
                report(),
            ],
            keys(&[A]),
        );
        assert_eq!(pairs(&out), vec![(A, 1), (A, 0)]);
    }

    #[test]
    fn orphan_releases_and_stray_autorepeat_are_dropped() {
        let mut sync = DropSync::new();
        let out = sync.process([key(A, 0), key(S, 2), report()], keys(&[]));
        assert!(out.events.is_empty());
    }

    #[test]
    fn a_syn_dropped_split_across_reads_still_resyncs() {
        let mut sync = DropSync::new();
        sync.process([key(A, 1), report()], keys(&[]));
        assert!(sync.process([dropped()], keys(&[])).events.is_empty());
        let out = sync.process([report()], keys(&[]));
        assert_eq!(pairs(&out), vec![(A, 0)]);
    }

    #[test]
    fn if_the_key_state_cannot_be_read_everything_is_released() {
        let mut sync = DropSync::new();
        sync.process([key(A, 1), key(S, 1), report()], keys(&[]));
        let out = sync.process([dropped(), report()], || {
            Err(io::Error::other("ioctl failed"))
        });
        assert_eq!(pairs(&out), vec![(A, 0), (S, 0)]);
    }

    #[test]
    fn a_release_lost_without_any_marker_is_caught_when_idle() {
        let mut sync = DropSync::new();
        sync.process([key(A, 1), key(S, 1), report()], keys(&[]));
        // The kernel says only S is still down and there is nothing to read.
        let out = sync.reconcile_idle(keys(&[S]));
        assert_eq!(pairs(&out), vec![(A, 0)]);
        assert_eq!(out.overflows, 1);
        assert!(sync.held().contains(&S));
        // Nothing stale left: a second check is a no-op.
        assert!(sync.reconcile_idle(keys(&[S])).events.is_empty());
    }

    #[test]
    fn a_press_flushed_by_the_idle_check_is_not_lost() {
        // A is held; B's press lands between our last read and EVIOCGKEY, so
        // the ioctl flushed it from the buffer and only the state shows it.
        let mut sync = DropSync::new();
        sync.process([key(A, 1), report()], keys(&[]));
        let out = sync.reconcile_idle(keys(&[A, S]));
        assert_eq!(pairs(&out), vec![(S, 1)]);
        assert!(sync.held().contains(&S));
        // Its real release is balanced, not an orphan.
        let out = sync.process([key(S, 0), report()], keys(&[]));
        assert_eq!(pairs(&out), vec![(S, 0)]);
    }

    #[test]
    fn the_idle_check_never_touches_a_packet_in_flight() {
        let mut sync = DropSync::new();
        sync.process([key(A, 1), report()], keys(&[]));
        sync.process([key(S, 1)], keys(&[])); // packet not closed yet
        assert!(sync.reconcile_idle(keys(&[])).events.is_empty());
    }

    #[test]
    fn non_key_events_are_never_filtered() {
        let mut sync = DropSync::new();
        let rel = InputEvent::new_now(EventType::RELATIVE, 0, 5);
        let out = sync.process([rel, report()], keys(&[]));
        assert_eq!(out.events.len(), 1);
    }
}
