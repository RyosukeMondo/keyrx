//! One-shot (sticky) modifiers: `one_shot("CapsLock", "VK_LShift")`.
//!
//! A quick tap LATCHES the modifier for the next key typed; holding the key
//! is the plain modifier. For anyone who cannot hold two keys at once.
//!
//! ```text
//!   press  -> modifier goes DOWN (a hold works like the real modifier)
//!   release, nothing typed meanwhile -> LATCHED: modifier stays down
//!   next non-modifier key press      -> modifier goes UP right after it
//!   press again while latched        -> cancels the latch
//!   timeout (optional) while latched -> modifier goes UP
//! ```
//!
//! The modifier is always released by the same state machine that pressed
//! it, so no input sequence leaves it stuck (see
//! `tests/remapper_balance_proptest.rs`).

extern crate alloc;

use alloc::vec::Vec;
use arrayvec::ArrayVec;

use crate::config::KeyCode;
use crate::runtime::KeyEvent;

/// Most one-shot keys tracked at once (one per physical modifier).
const MAX_ONE_SHOTS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Key is down; `used` once something was typed while it was.
    Held { used: bool },
    /// Key was tapped; the modifier waits for the next key.
    Latched { since_us: u64 },
    /// Key is still down after cancelling a latch; its release is silent.
    Swallow,
}

#[derive(Debug, Clone, Copy)]
struct Slot {
    from: KeyCode,
    modifier: KeyCode,
    timeout_us: u64,
    phase: Phase,
}

/// The one-shot keys of one device. See the module docs.
#[derive(Debug, Clone, Default)]
pub struct OneShots {
    slots: ArrayVec<Slot, MAX_ONE_SHOTS>,
}

fn press(key: KeyCode, ts: u64) -> KeyEvent {
    KeyEvent::press(key).with_timestamp(ts)
}

fn release(key: KeyCode, ts: u64) -> KeyEvent {
    KeyEvent::release(key).with_timestamp(ts)
}

impl OneShots {
    /// Whether `from` is a one-shot key whose press is being tracked.
    pub fn is_tracked(&self, from: KeyCode) -> bool {
        self.slots.iter().any(|s| s.from == from)
    }

    /// The one-shot key `from` was pressed at `ts`; returns the output.
    pub fn on_press(
        &mut self,
        from: KeyCode,
        modifier: KeyCode,
        timeout_ms: u16,
        ts: u64,
    ) -> Vec<KeyEvent> {
        if let Some(slot) = self.slots.iter_mut().find(|s| s.from == from) {
            return match slot.phase {
                Phase::Latched { .. } => {
                    slot.phase = Phase::Swallow;
                    alloc::vec![release(slot.modifier, ts)]
                }
                // Auto-repeat of a key already down.
                Phase::Held { .. } | Phase::Swallow => Vec::new(),
            };
        }
        let slot = Slot {
            from,
            modifier,
            timeout_us: u64::from(timeout_ms) * 1000,
            phase: Phase::Held { used: false },
        };
        if self.slots.try_push(slot).is_err() {
            return Vec::new();
        }
        alloc::vec![press(modifier, ts)]
    }

    /// The one-shot key `from` was released at `ts`; returns the output.
    pub fn on_release(&mut self, from: KeyCode, ts: u64) -> Vec<KeyEvent> {
        let Some(index) = self.slots.iter().position(|s| s.from == from) else {
            return Vec::new();
        };
        let slot = self.slots[index];
        match slot.phase {
            Phase::Held { used: false } => {
                self.slots[index].phase = Phase::Latched { since_us: ts };
                Vec::new()
            }
            Phase::Held { used: true } => {
                self.slots.remove(index);
                alloc::vec![release(slot.modifier, ts)]
            }
            Phase::Latched { .. } | Phase::Swallow => {
                self.slots.remove(index);
                Vec::new()
            }
        }
    }

    /// Call with the outputs of every processed event: typing a
    /// non-modifier key marks held one-shots as used and releases latched
    /// ones (their release is appended AFTER the key, so the key was typed
    /// with the modifier down).
    pub fn after_outputs(&mut self, outputs: &mut Vec<KeyEvent>, ts: u64) {
        if self.slots.is_empty() {
            return;
        }
        let typed = outputs
            .iter()
            .any(|e| e.is_press() && !e.keycode().is_modifier());
        if !typed {
            return;
        }
        for slot in self.slots.iter_mut() {
            if let Phase::Held { .. } = slot.phase {
                slot.phase = Phase::Held { used: true };
            }
        }
        let latched: ArrayVec<KeyCode, MAX_ONE_SHOTS> = self
            .slots
            .iter()
            .filter(|s| matches!(s.phase, Phase::Latched { .. }))
            .map(|s| s.modifier)
            .collect();
        self.slots
            .retain(|s| !matches!(s.phase, Phase::Latched { .. }));
        outputs.extend(latched.into_iter().map(|m| release(m, ts)));
    }

    /// Releases latches that were not used within their timeout.
    pub fn expire(&mut self, now_us: u64) -> Vec<KeyEvent> {
        let mut out = Vec::new();
        self.slots.retain(|slot| match slot.phase {
            Phase::Latched { since_us }
                if slot.timeout_us > 0 && now_us.saturating_sub(since_us) >= slot.timeout_us =>
            {
                out.push(release(slot.modifier, now_us));
                false
            }
            _ => true,
        });
        out
    }

    /// Forgets every slot and returns the releases for modifiers it holds
    /// down, so a reset can never leave one stuck.
    pub fn reset(&mut self, ts: u64) -> Vec<KeyEvent> {
        let out = self.slots.iter().map(|s| release(s.modifier, ts)).collect();
        self.slots.clear();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use KeyCode::{CapsLock, LShift, A};

    fn tap(os: &mut OneShots, t: u64) {
        assert_eq!(os.on_press(CapsLock, LShift, 0, t), [press(LShift, t)]);
        assert!(os.on_release(CapsLock, t + 10).is_empty());
    }

    #[test]
    fn tap_latches_until_next_key_then_releases_after_it() {
        let mut os = OneShots::default();
        tap(&mut os, 0);
        let mut out = alloc::vec![press(A, 100), release(A, 110)];
        os.after_outputs(&mut out, 120);
        let seq: Vec<_> = out.iter().map(|e| (e.keycode(), e.is_press())).collect();
        assert_eq!(seq, [(A, true), (A, false), (LShift, false)]);
        assert!(!os.is_tracked(CapsLock));
    }

    #[test]
    fn modifier_presses_do_not_consume_the_latch() {
        let mut os = OneShots::default();
        tap(&mut os, 0);
        let mut out = alloc::vec![press(KeyCode::LCtrl, 5)];
        os.after_outputs(&mut out, 5);
        assert_eq!(out.len(), 1);
        assert!(os.is_tracked(CapsLock));
    }

    #[test]
    fn hold_is_a_plain_modifier() {
        let mut os = OneShots::default();
        os.on_press(CapsLock, LShift, 0, 0);
        let mut out = alloc::vec![press(A, 50)];
        os.after_outputs(&mut out, 50);
        assert_eq!(out.len(), 1, "a held one-shot releases with its key");
        assert_eq!(os.on_release(CapsLock, 90), [release(LShift, 90)]);
    }

    #[test]
    fn second_tap_cancels_the_latch() {
        let mut os = OneShots::default();
        tap(&mut os, 0);
        assert_eq!(os.on_press(CapsLock, LShift, 0, 50), [release(LShift, 50)]);
        assert!(os.on_release(CapsLock, 60).is_empty());
        assert!(!os.is_tracked(CapsLock));
    }

    #[test]
    fn unused_latch_expires_after_its_timeout() {
        let mut os = OneShots::default();
        os.on_press(CapsLock, LShift, 2, 0);
        os.on_release(CapsLock, 100);
        assert!(os.expire(2_099).is_empty());
        assert_eq!(os.expire(2_100), [release(LShift, 2_100)]);
        assert!(!os.is_tracked(CapsLock));
    }
}
