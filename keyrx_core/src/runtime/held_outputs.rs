//! The output stream invariant: every key is down at most once.
//!
//! All devices a [`crate::runtime::Remapper`] routes write to ONE output
//! device, and the OS tracks one up/down bit per key. Several sources can hold
//! the same output key at once - the user physically holds LShift while a
//! `with_shift(..)` mapping also presses it, or two keys map to the same
//! output. Passing every press/release through verbatim lets the first
//! release lift the key while another source still holds it: real LShift held
//! + `/`→`with_shift(2)` + H used to type a lowercase h.
//!
//! [`HeldOutputs`] reference-counts output keys: a press reaches the OS only
//! for the first holder, a release only when the last holder lets go. A
//! release of a key it never saw pressed (held before the engine started, or
//! before a config swap) is passed through, so nothing can get stuck.

extern crate alloc;

use alloc::vec::Vec;

use hashbrown::HashMap;

use crate::config::KeyCode;
use crate::runtime::KeyEvent;

/// Reference counts of output keys currently held (see the module docs).
#[derive(Debug, Default)]
pub struct HeldOutputs {
    counts: HashMap<KeyCode, u16>,
}

impl HeldOutputs {
    /// An empty tracker (nothing held).
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters `events` down to the transitions the OS must see.
    pub fn normalize(&mut self, events: Vec<KeyEvent>) -> Vec<KeyEvent> {
        events.into_iter().filter(|e| self.admit(e)).collect()
    }

    /// Updates the count for one event; `true` if it changes the key's
    /// up/down state as the OS sees it.
    fn admit(&mut self, event: &KeyEvent) -> bool {
        let key = event.keycode();
        if event.is_press() {
            let count = self.counts.entry(key).or_insert(0);
            *count = count.saturating_add(1);
            return *count == 1;
        }
        match self.counts.get_mut(&key) {
            // Never seen pressed: pass the release through.
            None => true,
            Some(count) if *count <= 1 => {
                self.counts.remove(&key);
                true
            }
            Some(count) => {
                *count -= 1;
                false
            }
        }
    }

    /// Keys currently held by at least one source.
    pub fn held(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.counts.keys().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn run(events: &[(bool, KeyCode)]) -> Vec<(bool, KeyCode)> {
        let mut held = HeldOutputs::new();
        let input = events
            .iter()
            .map(|&(press, key)| {
                if press {
                    KeyEvent::press(key)
                } else {
                    KeyEvent::release(key)
                }
            })
            .collect();
        held.normalize(input)
            .iter()
            .map(|e| (e.is_press(), e.keycode()))
            .collect()
    }

    #[test]
    fn inner_holder_does_not_release_an_outer_one() {
        use KeyCode::{LShift, Num2, H};
        let out = run(&[
            (true, LShift), // physical
            (true, LShift), // with_shift(2)
            (true, Num2),
            (false, Num2),
            (false, LShift), // with_shift(2) done: LShift must stay down
            (true, H),
            (false, H),
            (false, LShift), // physical release
        ]);
        assert_eq!(
            out,
            vec![
                (true, LShift),
                (true, Num2),
                (false, Num2),
                (true, H),
                (false, H),
                (false, LShift),
            ]
        );
    }

    #[test]
    fn unknown_release_passes_through() {
        assert_eq!(run(&[(false, KeyCode::A)]), vec![(false, KeyCode::A)]);
    }

    #[test]
    fn balanced_stream_is_unchanged_and_nothing_stays_held() {
        let mut held = HeldOutputs::new();
        let events = vec![
            KeyEvent::press(KeyCode::A),
            KeyEvent::press(KeyCode::B),
            KeyEvent::release(KeyCode::A),
            KeyEvent::release(KeyCode::B),
        ];
        assert_eq!(held.normalize(events.clone()), events);
        assert_eq!(held.held().count(), 0);
    }

    #[test]
    fn remapper_keeps_a_physically_held_modifier_down() {
        use crate::config::{DeviceConfig, DeviceIdentifier, KeyMapping};
        use crate::runtime::Remapper;
        let config = DeviceConfig {
            identifier: DeviceIdentifier {
                pattern: "*".into(),
            },
            mappings: vec![KeyMapping::modified_output(
                KeyCode::Slash,
                KeyCode::Num2,
                true,
                false,
                false,
                false,
            )],
        };
        let mut remapper = Remapper::new(&config);
        let mut out = Vec::new();
        for (press, key) in [
            (true, KeyCode::LShift),
            (true, KeyCode::Slash),
            (false, KeyCode::Slash),
            (true, KeyCode::H),
        ] {
            let event = if press {
                KeyEvent::press(key)
            } else {
                KeyEvent::release(key)
            };
            out.extend(remapper.process(event, |_| Vec::new(), None).outputs);
        }
        let lshift_released = out
            .iter()
            .any(|e| e.keycode() == KeyCode::LShift && !e.is_press());
        assert!(!lshift_released, "LShift lifted while still held: {out:?}");
    }
}
